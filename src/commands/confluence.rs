use crate::adf;
use crate::client::{AtlassianClient, AtlassianApi};
use crate::formatter::Formatter;
use serde_json::Value;

/// Execute Confluence commands.
pub async fn run_search(client: &AtlassianClient, formatter: &Formatter, title: String, space_id: Option<String>) -> Result<(), String> {
    let data = client.search_pages(&title, space_id.as_deref()).await?;
    let pages = data["results"].as_array().ok_or("No pages found in response")?;

    let mut rows = vec![vec!["ID".to_string(), "TITLE".to_string(), "SPACE".to_string()]];
    for p in pages {
        rows.push(vec![
            p["id"].as_str().unwrap_or_default().to_string(),
            p["title"].as_str().unwrap_or_default().to_string(),
            p["spaceId"].as_str().unwrap_or_default().to_string(),
        ]);
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
