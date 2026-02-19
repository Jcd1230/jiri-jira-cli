use crate::client::JiraClient;
use crate::formatter::Formatter;

/// Execute the projects command to list projects.
pub async fn run(client: &JiraClient, formatter: &Formatter) -> Result<(), String> {
    let data = client.projects().await?;
    let projects = data["values"].as_array().ok_or("No projects found in response")?;

    let mut rows = vec![vec!["KEY".to_string(), "NAME".to_string()]];
    for p in projects {
        rows.push(vec![
            p["key"].as_str().unwrap_or_default().to_string(),
            p["name"].as_str().unwrap_or_default().to_string(),
        ]);
    }

    println!("{}", formatter.render(rows));
    Ok(())
}
