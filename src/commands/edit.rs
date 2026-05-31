use jiri_core::client::AtlassianClient;
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
    let payload = jiri_core::fields::build_update_payload(
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
