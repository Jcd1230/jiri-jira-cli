use crate::adf;
use crate::client::AtlassianClient;
use crate::formatter::Formatter;
use serde_json::Value;

/// Execute Confluence commands.
pub async fn run_search(client: &AtlassianClient, formatter: &Formatter, title: String, space_id: Option<String>) -> Result<(), String> {
    let data = client.search_pages(&title, space_id.as_deref()).await?;
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
