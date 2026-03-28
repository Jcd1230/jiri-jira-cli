use crate::client::AtlassianClient;
use crate::formatter::Formatter;

/// Execute the projects command to list projects.
pub async fn run(client: &AtlassianClient, formatter: &Formatter) -> Result<(), String> {
    let projects = client.projects_all().await?;

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
