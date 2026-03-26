use crate::client::AtlassianClient;
use crate::config::Config;
use std::env;
use std::path::PathBuf;

/// Diagnostic tool to check configuration and connectivity.
pub async fn run(client: &AtlassianClient) -> Result<(), String> {
    println!("Jiri Doctor - Diagnostic Information");
    println!("====================================");

    println!("\n[Authentication Source]");
    println!("Active source: {}", client.config().source);

    println!("\n[Environment Variables]");
    check_env("JIRA_API_USERNAME");
    check_env("JIRA_API_TOKEN");
    check_env("JIRA_SITE");
    check_env("JIRA_DEFAULT_PROJECT");

    println!("\n[Configuration Files]");
    check_config_file("Local (jiri.toml)", PathBuf::from("jiri.toml"));
    if let Some(path) = Config::global_config_path() {
        check_config_file("Global", path);
    }

    println!("\n[Effective Configuration]");
    println!("Username: {}", client.config().user);
    println!("Site:     {}", client.config().site);
    println!("Token:    {}", mask_token(&client.config().token));
    println!("Project:  {}", client.config().default_project.as_deref().unwrap_or("(none)"));

    println!("\n[Connectivity]");
    print!("Connecting to {}... ", client.config().site);
    match client.myself().await {
        Ok(me) => {
            println!("OK");
            if let Some(name) = me["displayName"].as_str() {
                println!("Logged in as: {}", name);
            }
            if let Some(email) = me["emailAddress"].as_str() {
                println!("Account email: {}", email);
            }
        }
        Err(e) => {
            println!("FAILED");
            println!("Error: {}", e);
        }
    }

    Ok(())
}

fn check_env(name: &str) {
    match env::var(name) {
        Ok(val) => {
            let display = if name.contains("TOKEN") { mask_token(&val) } else { val };
            println!("  {:20} : SET ({})", name, display);
        }
        Err(_) => {
            println!("  {:20} : NOT SET", name);
        }
    }
}

fn check_config_file(label: &str, path: PathBuf) {
    if path.exists() {
        println!("  {:20} : FOUND ({})", label, path.display());
    } else {
        println!("  {:20} : NOT FOUND", label);
    }
}

fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        return "****".to_string();
    }
    format!("{}...{}", &token[..4], &token[token.len() - 4..])
}
