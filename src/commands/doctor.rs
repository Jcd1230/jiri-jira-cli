use crate::client::AtlassianClient;
use crate::config::{Config, mask_token};
use std::env;
use std::path::PathBuf;
use clap::builder::styling::{AnsiColor, Reset};

/// Diagnostic tool to check configuration and connectivity.
pub async fn run(client: &AtlassianClient) -> Result<(), String> {
    let yellow = AnsiColor::Yellow.render_fg();
    let blue = AnsiColor::Blue.render_fg();
    let cyan = AnsiColor::Cyan.render_fg();
    let green = AnsiColor::Green.render_fg();
    let red = AnsiColor::Red.render_fg();
    let reset = Reset.render();

    println!("{}Jiri Doctor - Diagnostic Information{}", yellow, reset);
    println!("====================================");

    println!("\n{}[Authentication Source]{}", blue, reset);
    println!("Active source: {}{}{}", cyan, client.config().source, reset);

    println!("\n{}[Environment Variables]{}", blue, reset);
    check_env("JIRA_API_USERNAME");
    check_env("JIRA_API_TOKEN");
    check_env("JIRA_SITE");
    check_env("JIRA_DEFAULT_PROJECT");

    println!("\n{}[Configuration Files]{}", blue, reset);
    check_config_file("Local (jiri.toml)", PathBuf::from("jiri.toml"));
    if let Some(path) = Config::global_config_path() {
        check_config_file("Global", path);
    }

    println!("\n{}[Effective Configuration]{}", blue, reset);
    println!("Username: {}{}{}", cyan, client.config().user, reset);
    println!("Site:     {}{}{}", cyan, client.config().site, reset);
    println!("Token:    {}{}{}", cyan, mask_token(&client.config().token), reset);
    println!("Project:  {}{}{}", cyan, client.config().default_project.as_deref().unwrap_or("(none)"), reset);

    println!("\n{}[Connectivity]{}", blue, reset);
    print!("Connecting to {}... ", client.config().site);
    match client.myself().await {
        Ok(me) => {
            println!("{}OK{}", green, reset);
            if let Some(name) = me["displayName"].as_str() {
                println!("Logged in as: {}{}{}", cyan, name, reset);
            }
            if let Some(email) = me["emailAddress"].as_str() {
                println!("Account email: {}{}{}", cyan, email, reset);
            }
        }
        Err(e) => {
            println!("{}FAILED{}", red, reset);
            println!("{}Error: {}{}", red, e, reset);
        }
    }

    Ok(())
}

fn check_env(name: &str) {
    let cyan = AnsiColor::Cyan.render_fg();
    let green = AnsiColor::Green.render_fg();
    let red = AnsiColor::Red.render_fg();
    let reset = Reset.render();

    match env::var(name) {
        Ok(val) => {
            let display = if name.contains("TOKEN") { mask_token(&val) } else { val };
            println!("  {:20} : {}SET{} ({}{}{})", name, green, reset, cyan, display, reset);
        }
        Err(_) => {
            println!("  {:20} : {}NOT SET{}", name, red, reset);
        }
    }
}

fn check_config_file(label: &str, path: PathBuf) {
    let cyan = AnsiColor::Cyan.render_fg();
    let green = AnsiColor::Green.render_fg();
    let red = AnsiColor::Red.render_fg();
    let reset = Reset.render();

    if path.exists() {
        println!("  {:20} : {}FOUND{} ({}{}{})", label, green, reset, cyan, path.display(), reset);
    } else {
        println!("  {:20} : {}NOT FOUND{}", label, red, reset);
    }
}
