use crate::client::JiraClient;

pub async fn run(client: &JiraClient, key: String, message: String) -> Result<(), String> {
    client.add_comment(&key, &message).await?;
    println!("Comment added to {}", key);
    Ok(())
}
