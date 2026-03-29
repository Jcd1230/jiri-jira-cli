use crate::client::AtlassianClient;
use owo_colors::OwoColorize;
use serde_json::Value;

/// Execute the edit command to update issue fields.
pub async fn run(
    client: &AtlassianClient,
    key: String,
    summary: Option<String>,
    description: Option<String>,
    labels: Option<String>,
    assignee: Option<String>,
) -> Result<(), String> {
    let mut fields = serde_json::Map::new();

    if let Some(summary) = summary {
        fields.insert("summary".to_string(), Value::String(summary));
    }

    if let Some(description) = description {
        fields.insert(
            "description".to_string(),
            crate::adf::from_plain_text(&description),
        );
    }

    if let Some(labels) = labels {
        let labels = labels
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| Value::String(s.to_string()))
            .collect::<Vec<_>>();
        fields.insert("labels".to_string(), Value::Array(labels));
    }

    if let Some(assignee_query) = assignee {
        let account_id = resolve_account_id(client, &assignee_query).await?;
        fields.insert(
            "assignee".to_string(),
            serde_json::json!({ "accountId": account_id }),
        );
    }

    if fields.is_empty() {
        return Err(
            "No fields provided. Use --summary, --description, --labels, or --assignee."
                .to_string(),
        );
    }

    client.update_issue(&key, Value::Object(fields)).await?;
    println!("{} {}", "Updated issue:".green().bold(), key.cyan().bold());
    Ok(())
}

pub(crate) async fn resolve_account_id(
    client: &AtlassianClient,
    query: &str,
) -> Result<String, String> {
    if query.starts_with("acct:")
        || query.len() > 20 && query.chars().all(|c| c.is_ascii_alphanumeric())
    {
        return Ok(query.to_string());
    }

    let users = client.search_users(query).await?;
    let users = users
        .as_array()
        .ok_or("User search returned an unexpected response")?;

    if users.is_empty() {
        return Err(format!("No Jira users matched '{}'", query));
    }

    if users.len() > 1 {
        let matches: Vec<String> = users
            .iter()
            .take(5)
            .filter_map(|u| {
                let name = u["displayName"].as_str().unwrap_or("?");
                let email = u["emailAddress"].as_str().unwrap_or("");
                let account_id = u["accountId"].as_str().unwrap_or("?");
                Some(if email.is_empty() {
                    format!("{} ({})", name, account_id)
                } else {
                    format!("{} <{}> ({})", name, email, account_id)
                })
            })
            .collect();
        return Err(format!(
            "Multiple Jira users matched '{}': {}",
            query,
            matches.join(", ")
        ));
    }

    users[0]["accountId"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Matched Jira user had no accountId".to_string())
}
