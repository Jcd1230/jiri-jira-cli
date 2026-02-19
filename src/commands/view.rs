use crate::adf;
use crate::client::JiraClient;
use textwrap::wrap;

/// Execute the view command to show issue details.
pub async fn run(client: &JiraClient, key: String) -> Result<(), String> {
    let issue = client.get_issue(&key).await?;

    let issue_key = issue["key"].as_str().unwrap_or("?");
    let summary = issue["fields"]["summary"].as_str().unwrap_or("(no summary)");
    let status = issue["fields"]["status"]["name"].as_str().unwrap_or("?");
    let issue_type = issue["fields"]["issuetype"]["name"].as_str().unwrap_or("?");
    let priority = issue["fields"]["priority"]["name"].as_str().unwrap_or("?");
    let assignee = issue["fields"]["assignee"]["displayName"].as_str().unwrap_or("Unassigned");
    let reporter = issue["fields"]["reporter"]["displayName"].as_str().unwrap_or("?");
    let created = issue["fields"]["created"].as_str().unwrap_or("?");
    let updated = issue["fields"]["updated"].as_str().unwrap_or("?");

    println!("  {} — {}", issue_key, summary);
    println!();
    println!("  Type:       {}", issue_type);
    println!("  Status:     {}", status);
    println!("  Priority:   {}", priority);
    println!("  Assignee:   {}", assignee);
    println!("  Reporter:   {}", reporter);
    println!("  Created:    {}", created);
    println!("  Updated:    {}", updated);

    // Description
    let desc = adf::to_plain_text(&issue["fields"]["description"]);
    if !desc.is_empty() {
        println!();
        println!("  Description:");
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
        println!("  Comments ({} total, showing last {}):", comments.len(), recent.len());
        for c in recent.iter().rev() {
            let author = c["author"]["displayName"].as_str().unwrap_or("?");
            let created = c["created"].as_str().unwrap_or("?");
            let body = adf::to_plain_text(&c["body"]);
            println!();
            println!("    {} ({})", author, created);
            for line in wrap(&body, 72) {
                println!("      {}", line);
            }
        }
    }

    Ok(())
}
