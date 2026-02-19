use crate::client::JiraClient;

pub async fn run(
    client: &JiraClient,
    project: String,
    summary: String,
    issue_type: String,
    description: Option<String>,
) -> Result<(), String> {
    let result = client
        .create_issue(&project, &summary, &issue_type, description.as_deref())
        .await?;

    let key = result["key"].as_str().unwrap_or("?");
    let url = result["self"].as_str().unwrap_or("");

    println!("Created issue: {}", key);
    if !url.is_empty() {
        println!("  {}", url);
    }

    Ok(())
}
