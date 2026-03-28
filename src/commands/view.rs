use crate::adf;
use crate::client::AtlassianClient;
use owo_colors::OwoColorize;
use textwrap::wrap;

/// Execute the view command to show issue details.
pub async fn run(client: &AtlassianClient, key: String) -> Result<(), String> {
    let issue = client.get_issue(&key).await?;

    let issue_key = issue["key"].as_str().unwrap_or("?");
    let summary = issue["fields"]["summary"]
        .as_str()
        .unwrap_or("(no summary)");
    let status = issue["fields"]["status"]["name"].as_str().unwrap_or("?");
    let issue_type = issue["fields"]["issuetype"]["name"].as_str().unwrap_or("?");
    let priority = issue["fields"]["priority"]["name"].as_str().unwrap_or("?");
    let assignee = issue["fields"]["assignee"]["displayName"]
        .as_str()
        .unwrap_or("Unassigned");
    let reporter = issue["fields"]["reporter"]["displayName"]
        .as_str()
        .unwrap_or("?");
    let created = issue["fields"]["created"].as_str().unwrap_or("?");
    let updated = issue["fields"]["updated"].as_str().unwrap_or("?");

    println!("  {} — {}", issue_key.cyan().bold(), summary.bold());
    println!();
    println!("  {} {}", "Type:".cyan().bold(), issue_type);
    println!("  {} {}", "Status:".cyan().bold(), stylize_status(status));
    println!(
        "  {} {}",
        "Priority:".cyan().bold(),
        stylize_priority(priority)
    );
    println!("  {} {}", "Assignee:".cyan().bold(), assignee);
    println!("  {} {}", "Reporter:".cyan().bold(), reporter);
    println!("  {} {}", "Created:".cyan().bold(), created.dimmed());
    println!("  {} {}", "Updated:".cyan().bold(), updated.dimmed());

    // Description
    let desc = adf::to_plain_text(&issue["fields"]["description"]);
    if !desc.is_empty() {
        println!();
        println!("  {}", "Description:".cyan().bold());
        for line in wrap(&desc, 76) {
            println!("    {}", line);
        }
    }

    // Recent comments
    let comments = issue["fields"]["comment"]["comments"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !comments.is_empty() {
        println!();
        let recent: Vec<_> = comments.iter().rev().take(5).collect();
        println!(
            "  {} ({} total, showing last {}):",
            "Comments".cyan().bold(),
            comments.len(),
            recent.len()
        );
        for c in recent.iter().rev() {
            let author = c["author"]["displayName"].as_str().unwrap_or("?");
            let created = c["created"].as_str().unwrap_or("?");
            let body = adf::to_plain_text(&c["body"]);
            println!();
            println!("    {} ({})", author.bold(), created.dimmed());
            for line in wrap(&body, 72) {
                println!("      {}", line);
            }
        }
    }

    Ok(())
}

fn stylize_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "done" | "closed" | "resolved" => status.green().bold().to_string(),
        "in progress" => status.yellow().bold().to_string(),
        "open" | "to do" => status.cyan().bold().to_string(),
        "blocked" | "on hold" => status.red().bold().to_string(),
        _ => status.to_string(),
    }
}

fn stylize_priority(priority: &str) -> String {
    match priority.to_lowercase().as_str() {
        "highest" | "critical" | "blocker" => priority.red().bold().to_string(),
        "high" => priority.yellow().bold().to_string(),
        "medium" | "normal" => priority.cyan().bold().to_string(),
        "low" | "lowest" => priority.green().bold().to_string(),
        _ => priority.to_string(),
    }
}
