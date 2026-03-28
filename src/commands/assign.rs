use crate::client::AtlassianClient;
use owo_colors::OwoColorize;

/// Execute the assign command to set issue assignee.
pub async fn run(client: &AtlassianClient, key: String, user: String) -> Result<(), String> {
    let account_id = super::edit::resolve_account_id(client, &user).await?;
    client
        .update_issue(
            &key,
            serde_json::json!({ "assignee": { "accountId": account_id } }),
        )
        .await?;

    println!(
        "{} {}",
        "Assigned issue:".green().bold(),
        key.cyan().bold()
    );
    Ok(())
}
