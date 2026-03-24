use crate::adf;
use crate::client::AtlassianClient;
use crate::formatter::Formatter;
use serde_json::Value;

/// Execute Confluence commands.
pub async fn run_search(
    client: &AtlassianClient,
    formatter: &Formatter,
    query: Option<String>,
    space_id: Option<String>,
    limit: i64,
    is_cql: bool,
) -> Result<(), String> {
    let cql = if is_cql {
        query.ok_or("Query is required when --cql is used")?
    } else {
        let title_query = query.as_deref().unwrap_or("*");
        let mut base = format!("type=page and title ~ \"{}\"", title_query);
        if let Some(space) = space_id {
            base.push_str(&format!(" and space = \"{}\"", space));
        }
        base
    };

    let data = client.search_pages(&cql, limit).await?;
    let results = data["results"].as_array().ok_or("No results found in response")?;

    let mut rows = vec![vec!["ID".to_string(), "TITLE".to_string(), "SPACE".to_string()]];
    for r in results {
        let content = &r["content"];
        let id = content["id"].as_str().unwrap_or_default().to_string();
        let title = content["title"].as_str().unwrap_or_default().to_string();
        let space = r["resultGlobalContainer"]["title"].as_str().unwrap_or_default().to_string();
        
        rows.push(vec![id, title, space]);
    }

    println!("{}", formatter.render(rows));
    Ok(())
}

pub async fn run_create(
    client: &AtlassianClient,
    title: String,
    space: String,
    parent: Option<String>,
    content: Option<String>,
    is_adf: bool,
) -> Result<(), String> {
    // 1. Resolve Space ID
    let space_id = client.get_space_id(&space).await?;

    // 2. Prepare ADF Body
    let nodes = if let Some(text) = content {
        if is_adf {
            serde_json::from_str(&text).map_err(|e| format!("Invalid ADF JSON: {}", e))?
        } else {
            adf::from_markdown(&text)
        }
    } else {
        vec![serde_json::json!({
            "type": "paragraph",
            "content": [{"type": "text", "text": ""}]
        })]
    };

    let adf_body = serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": nodes
    });

    // 3. Create Page
    let result = client.create_page(&space_id, &title, parent.as_deref(), &adf_body).await?;
    let id = result["id"].as_str().unwrap_or("?");
    
    println!("Successfully created page: {}", title);
    println!("  ID: {}", id);
    if let Some(links) = result["_links"].as_object() {
        if let Some(base) = links.get("base") {
            if let Some(webui) = links.get("webui") {
                println!("  URL: {}{}", base.as_str().unwrap_or(""), webui.as_str().unwrap_or(""));
            }
        }
    }

    Ok(())
}

pub async fn run_view(client: &AtlassianClient, id: String, raw: bool) -> Result<(), String> {
    let page = client.get_page(&id).await?;
    
    let title = page["title"].as_str().unwrap_or("(no title)");
    let space_id = page["spaceId"].as_str().unwrap_or("?");
    let version = page["version"]["number"].as_i64().unwrap_or(0);

    println!("{} (ID: {}, Space: {}, Version: {})", title, id, space_id, version);
    println!("{}", "=".repeat(title.len()));
    println!();

    let adf_body_str = page["body"]["atlas_doc_format"]["value"].as_str().ok_or("No ADF body found")?;
    let adf_body: Value = serde_json::from_str(adf_body_str).map_err(|e| e.to_string())?;

    if raw {
        println!("{}", serde_json::to_string_pretty(&adf_body).unwrap());
    } else {
        println!("{}", adf::to_plain_text(&adf_body));
    }

    Ok(())
}

pub async fn run_edit(
    client: &AtlassianClient,
    id: String,
    append: Option<String>,
    prepend: Option<String>,
    replace: Option<String>,
    new_title: Option<String>,
    is_adf: bool,
    minor: bool,
) -> Result<(), String> {
    let mut retries = 3;
    
    loop {
        // 1. Fetch
        let page = client.get_page(&id).await?;
        let current_title = page["title"].as_str().ok_or("No title found")?.to_string();
        let space_id = page["spaceId"].as_str().ok_or("No spaceId found")?.to_string();
        let version = page["version"]["number"].as_i64().ok_or("No version found")?;
        
        let adf_body_str = page["body"]["atlas_doc_format"]["value"].as_str().ok_or("No ADF body found")?;
        let mut adf_body: Value = serde_json::from_str(adf_body_str).map_err(|e| e.to_string())?;

        // 2. Modify
        if let Some(ref content) = append {
            let nodes = if is_adf {
                serde_json::from_str(content).map_err(|e| format!("Invalid ADF in --append: {}", e))?
            } else {
                adf::from_markdown(content)
            };
            adf::append_nodes(&mut adf_body, nodes);
        }

        if let Some(ref content) = prepend {
            let nodes = if is_adf {
                serde_json::from_str(content).map_err(|e| format!("Invalid ADF in --prepend: {}", e))?
            } else {
                adf::from_markdown(content)
            };
            adf::prepend_nodes(&mut adf_body, nodes);
        }

        if let Some(ref r) = replace {
            let parts: Vec<&str> = r.splitn(2, ':').collect();
            if parts.len() == 2 {
                adf::replace_text(&mut adf_body, parts[0], parts[1]);
            } else {
                return Err("Replace format must be OLD:NEW".to_string());
            }
        }

        let title_to_use = new_title.clone().unwrap_or(current_title);

        // 3. Update
        match client.update_page(&id, &title_to_use, &space_id, &adf_body, version + 1, minor).await {
            Ok(_) => {
                println!("Successfully updated page {}", id);
                return Ok(());
            }
            Err(e) if e.contains("409") && retries > 0 => {
                eprintln!("Version conflict, retrying ({} retries left)...", retries);
                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
