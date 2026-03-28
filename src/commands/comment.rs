use crate::client::AtlassianClient;
use owo_colors::OwoColorize;

/// Execute the comment command to add a comment to an issue.
pub async fn run(client: &AtlassianClient, key: String, message: String) -> Result<(), String> {
    client.add_comment(&key, &message).await?;
    println!(
        "{} {}",
        "Comment added to".green().bold(),
        key.cyan().bold()
    );
    Ok(())
}
