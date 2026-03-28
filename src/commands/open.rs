use crate::client::AtlassianClient;
use owo_colors::OwoColorize;
use std::process::Command;

/// Open a Jira issue in the browser.
pub async fn run(client: &AtlassianClient, key: String) -> Result<(), String> {
    let url = format!(
        "{}/browse/{}",
        client.config().site.trim_end_matches('/'),
        key
    );

    open_url(&url)?;
    println!("{} {}", "Opened issue:".green().bold(), key.cyan().bold());
    println!("  {}", url.dimmed());

    Ok(())
}

fn open_url(url: &str) -> Result<(), String> {
    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", url]).status()
    } else {
        Command::new("xdg-open").arg(url).status()
    };

    match result {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("Failed to open browser (status: {})", status)),
        Err(e) => Err(format!("Failed to open browser: {}", e)),
    }
}
