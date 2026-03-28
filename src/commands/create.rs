use crate::client::AtlassianClient;
use owo_colors::OwoColorize;

/// Execute the create command to create a new issue.
pub async fn run(
    client: &AtlassianClient,
    project: String,
    summary: String,
    issue_type: String,
    description: Option<String>,
) -> Result<(), String> {
    let result = client
        .create_issue(&project, &summary, &issue_type, description.as_deref())
        .await?;

    let key = result["key"].as_str().unwrap_or("?");
    let url = format!(
        "{}/browse/{}",
        client.config().site.trim_end_matches('/'),
        key
    );

    println!("{} {}", "Created issue:".green().bold(), key.cyan().bold());
    println!("  {}", url.dimmed());

    Ok(())
}
