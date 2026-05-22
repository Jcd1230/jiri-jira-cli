use crate::client::AtlassianClient;
use owo_colors::OwoColorize;
use serde_json::Value;
use std::io::{self, Write};

/// Execute the bulk-edit command to update fields on multiple issues.
pub async fn run(
    client: &AtlassianClient,
    jql: Option<String>,
    issues_arg: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    labels: Option<String>,
    assignee: Option<String>,
    fields: Vec<String>,
    fields_json: Vec<String>,
    yes: bool,
    max_failures: Option<usize>,
) -> Result<(), String> {
    if jql.is_none() && issues_arg.is_none() {
        return Err("Either --jql or --issues must be specified for bulk editing.".to_string());
    }
    if jql.is_some() && issues_arg.is_some() {
        return Err("Cannot specify both --jql and --issues.".to_string());
    }

    let mut issue_keys = Vec::new();

    if let Some(keys_str) = issues_arg {
        issue_keys = keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    } else if let Some(jql_query) = jql {
        println!("Searching for issues matching JQL: {}", jql_query.cyan());
        let (issues, _) = client
            .search_all(&jql_query, vec!["key".to_string()], 1000)
            .await?;
        for issue in issues {
            if let Some(key) = issue["key"].as_str() {
                issue_keys.push(key.to_string());
            }
        }
    }

    if issue_keys.is_empty() {
        println!("{}", "No matching issues found for bulk editing.".yellow());
        return Ok(());
    }

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
            "No fields provided to update. Use --summary, --description, --labels, --assignee, --field, or --field-json."
                .to_string(),
        );
    }

    println!(
        "\n{} bulk-edit of {} issue(s):",
        "Targeting".yellow().bold(),
        issue_keys.len().bold()
    );
    for key in &issue_keys {
        println!("  - {}", key.cyan());
    }

    println!("\nProposed updates:");
    for (k, v) in &payload {
        println!("  - {}: {}", k.yellow(), v);
    }
    println!();

    if !yes {
        print!("Are you sure you want to proceed? (y/N): ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return Err("Failed to read user input".to_string());
        }
        let response = input.trim().to_lowercase();
        if response != "y" && response != "yes" {
            println!("{}", "Bulk edit cancelled.".red());
            return Ok(());
        }
    }

    let max_fail = max_failures.unwrap_or(3);
    let mut consecutive_failures = 0;
    let mut total_failures = 0;
    let mut total_successes = 0;

    for key in &issue_keys {
        println!("Updating {}...", key.cyan());
        match client.update_issue(key, Value::Object(payload.clone())).await {
            Ok(_) => {
                println!("{} {}", "Successfully updated".green(), key.cyan());
                consecutive_failures = 0;
                total_successes += 1;
            }
            Err(err) => {
                println!("{} {}: {}", "Error updating".red(), key.cyan(), err);
                consecutive_failures += 1;
                total_failures += 1;

                if consecutive_failures >= max_fail {
                    return Err(format!(
                        "Aborting: reached {} consecutive failures.",
                        consecutive_failures
                    ));
                }

                if !yes {
                    print!("Continue updating remaining issues? (y/N): ");
                    let _ = io::stdout().flush();
                    let mut input = String::new();
                    if io::stdin().read_line(&mut input).is_err() {
                        return Err("Failed to read user input".to_string());
                    }
                    let response = input.trim().to_lowercase();
                    if response != "y" && response != "yes" {
                        println!("{}", "Bulk edit aborted by user.".red());
                        break;
                    }
                }
            }
        }
    }

    println!(
        "\n{} Bulk edit completed: {} succeeded, {} failed.",
        "Summary:".bold(),
        total_successes.green(),
        total_failures.red()
    );

    Ok(())
}
