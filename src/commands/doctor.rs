use crate::client::AtlassianClient;
use crate::config::{mask_token, Config};
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;

/// Diagnostic tool to check configuration and connectivity.
pub async fn run(client: &AtlassianClient) -> Result<(), String> {
    println!("{}", "Jiri Doctor - Diagnostic Information".yellow().bold());
    println!("====================================");

    println!("\n{}", "[Authentication Source]".blue().bold());
    println!(
        "{} {}",
        "Active source:".cyan().bold(),
        client.config().source.to_string().cyan()
    );

    println!("\n{}", "[Environment Variables]".blue().bold());
    check_env("JIRA_API_USERNAME");
    check_env("JIRA_API_TOKEN");
    check_env("JIRA_SITE");
    check_env("JIRA_DEFAULT_PROJECT");

    println!("\n{}", "[Configuration Files]".blue().bold());
    check_config_file("Local (jiri.toml)", PathBuf::from("jiri.toml"));
    if let Some(path) = Config::global_config_path() {
        check_config_file("Global", path);
    }

    println!("\n{}", "[Effective Configuration]".blue().bold());
    println!("{} {}", "Username:".cyan().bold(), client.config().user);
    println!("{} {}", "Site:".cyan().bold(), client.config().site);
    println!(
        "{} {}",
        "Token:".cyan().bold(),
        mask_token(&client.config().token).dimmed()
    );
    println!(
        "{} {}",
        "Project:".cyan().bold(),
        client
            .config()
            .default_project
            .as_deref()
            .unwrap_or("(none)")
    );

    println!("\n{}", "[Connectivity]".blue().bold());
    print!(
        "{} {}... ",
        "Connecting to".cyan().bold(),
        client.config().site
    );
    match client.myself().await {
        Ok(me) => {
            println!("{}", "OK".green().bold());
            if let Some(name) = me["displayName"].as_str() {
                println!("{} {}", "Logged in as:".cyan().bold(), name);
            }
            if let Some(email) = me["emailAddress"].as_str() {
                println!("{} {}", "Account email:".cyan().bold(), email);
            }
        }
        Err(e) => {
            println!("{}", "FAILED".red().bold());
            println!("{} {}", "Error:".red().bold(), e);
        }
    }

    Ok(())
}

fn check_env(name: &str) {
    match env::var(name) {
        Ok(val) => {
            let display = if name.contains("TOKEN") {
                mask_token(&val)
            } else {
                val
            };
            println!(
                "  {:20} : {} ({})",
                name,
                "SET".green().bold(),
                display.cyan()
            );
        }
        Err(_) => {
            println!("  {:20} : {}", name, "NOT SET".red().bold());
        }
    }
}

fn check_config_file(label: &str, path: PathBuf) {
    if path.exists() {
        println!(
            "  {:20} : {} ({})",
            label,
            "FOUND".green().bold(),
            path.display().to_string().cyan()
        );
    } else {
        println!("  {:20} : {}", label, "NOT FOUND".red().bold());
    }
}
