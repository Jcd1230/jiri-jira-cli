use crate::client::JiraClient;

/// Execute the comment command to add a comment to an issue.
pub async fn run(client: &JiraClient, key: String, message: String) -> Result<(), String> {
    client.add_comment(&key, &message).await?;
    println!("Comment added to {}", key);
    Ok(())
}
