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
    fields: Vec<String>,
    fields_json: Vec<String>,
) -> Result<(), String> {
    let payload = crate::fields::build_update_payload(
        client,
        summary,
        description,
        labels,
        assignee,
        &fields,
        &fields_json,
    )
    .await?;

    if payload.is_empty() {
        return Err(
            "No fields provided. Use --summary, --description, --labels, --assignee, --field, or --field-json."
                .to_string(),
        );
    }

    client.update_issue(&key, Value::Object(payload)).await?;
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
