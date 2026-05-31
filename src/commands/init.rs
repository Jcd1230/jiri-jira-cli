use jiri_core::client::AtlassianClient;
use jiri_core::config::{AuthConfig, Config, FileConfig, GeneralConfig};
use owo_colors::OwoColorize;
use std::io::{self, Write};

/// Interactive onboarding wizard: configure auth, validate, and set up shell completions.
pub async fn run() -> Result<(), String> {
    println!(
        "{}\n",
        "Welcome to Jiri! Let's get you set up.".yellow().bold()
    );

    // 1. Determine config destination
    let global_path = Config::global_config_path()
        .ok_or_else(|| "Could not determine config directory".to_string())?;
    let local_path = Config::local_config_path();

    let use_global = if local_path.exists() {
        println!(
            "  {} found at {}",
            "Local config".cyan().bold(),
            local_path.display()
        );
        false
    } else if global_path.exists() {
        println!(
            "  {} found at {}",
            "Global config".cyan().bold(),
            global_path.display()
        );
        println!("  Config already exists. Re-running init will overwrite values you provide.\n");
        true
    } else {
        println!(
            "  No existing configuration found. Creating global config at:\n  {}\n",
            global_path.display().to_string().cyan()
        );
        true
    };

    let config_path = if use_global {
        global_path
    } else {
        local_path
    };

    // Load existing config or start fresh
    let mut file_config = FileConfig::load_path(&config_path).unwrap_or_default();

    // 2. Gather credentials
    println!("{}", "[Step 1/4] Atlassian Credentials".blue().bold());
    println!("  Generate an API token at: https://id.atlassian.com/manage-profile/security/api-tokens\n");

    let site = prompt_with_default(
        "  Atlassian site URL",
        file_config.auth.site.as_deref().unwrap_or(""),
    )?;
    let username = prompt_with_default(
        "  Email (username)",
        file_config.auth.username.as_deref().unwrap_or(""),
    )?;
    let token = prompt_with_default(
        "  API token",
        "", // Never show existing token
    )?;

    if site.is_empty() || username.is_empty() || token.is_empty() {
        return Err("Site, username, and token are all required.".to_string());
    }

    file_config.auth = AuthConfig {
        username: Some(username.clone()),
        token: Some(token.clone()),
        site: Some(site.clone()),
    };

    // 3. Default project
    println!("\n{}", "[Step 2/4] Default Project".blue().bold());
    let existing_project = file_config
        .general
        .as_ref()
        .and_then(|g| g.default_project.as_deref())
        .unwrap_or("");
    let default_project = prompt_with_default(
        "  Default project key (optional, e.g. PROJ)",
        existing_project,
    )?;

    if !default_project.is_empty() {
        file_config.general = Some(GeneralConfig {
            default_project: Some(default_project),
        });
    }

    // 4. Save config
    file_config.save_path(&config_path)?;
    println!(
        "\n  {} {}",
        "Config saved to".green().bold(),
        config_path.display().to_string().cyan()
    );

    // 5. Validate connectivity
    println!("\n{}", "[Step 3/4] Connectivity Check".blue().bold());
    let config = Config::load()?;
    let client = AtlassianClient::new(config);

    print!("  Connecting to {}... ", site.cyan());
    io::stdout().flush().ok();

    match client.myself().await {
        Ok(me) => {
            let name = me["displayName"].as_str().unwrap_or("unknown");
            println!("{}", "OK".green().bold());
            println!(
                "  {} {}",
                "Authenticated as:".cyan().bold(),
                name.bold()
            );
        }
        Err(e) => {
            println!("{}", "FAILED".red().bold());
            println!("  {}", e.dimmed());
            println!(
                "\n  {} Config was saved but authentication failed. Double-check your credentials.",
                "hint:".yellow().bold()
            );
        }
    }

    // 6. Shell completions guidance
    println!("\n{}", "[Step 4/4] Shell Completions".blue().bold());
    let shell = detect_shell();
    match shell.as_deref() {
        Some("fish") => {
            println!("  Detected shell: {}", "fish".cyan().bold());
            println!(
                "  Run: {}",
                "jiri completions fish > ~/.config/fish/completions/jiri.fish"
                    .dimmed()
            );
        }
        Some("zsh") => {
            println!("  Detected shell: {}", "zsh".cyan().bold());
            println!(
                "  Run: {}",
                "jiri completions zsh >> ~/.zshrc".dimmed()
            );
        }
        Some("bash") => {
            println!("  Detected shell: {}", "bash".cyan().bold());
            println!(
                "  Run: {}",
                "jiri completions bash >> ~/.bashrc".dimmed()
            );
        }
        _ => {
            println!("  Could not detect shell. Install completions manually:");
            println!("    {}", "jiri completions <bash|zsh|fish>".dimmed());
        }
    }

    println!(
        "\n{}",
        "Setup complete! Run `jiri doctor` to verify anytime.".green().bold()
    );

    Ok(())
}

fn prompt_with_default(label: &str, default: &str) -> Result<String, String> {
    if default.is_empty() {
        print!("{}: ", label);
    } else {
        print!("{} [{}]: ", label, default.dimmed());
    }
    io::stdout()
        .flush()
        .map_err(|e| format!("IO error: {}", e))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;

    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed)
    }
}

fn detect_shell() -> Option<String> {
    std::env::var("SHELL").ok().and_then(|s| {
        s.rsplit('/')
            .next()
            .map(|name| name.to_string())
    })
}
