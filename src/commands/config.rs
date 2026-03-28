use crate::config::{mask_token, Config, FileConfig};
use owo_colors::OwoColorize;
use std::path::PathBuf;

pub async fn run_show(global: bool, local: bool) -> Result<(), String> {
    if global {
        let path = Config::global_config_path().ok_or("Could not determine global config path")?;
        show_file("Global", &path)?;
    } else if local {
        let path = Config::local_config_path();
        show_file("Local", &path)?;
    } else {
        // Show effective config and source
        let config = Config::load()?;
        println!(
            "{} (from {})",
            "Effective Configuration".yellow().bold(),
            config.source.to_string().cyan()
        );
        println!("--------------------------------------------------");
        println!("{} {}", "Username:".cyan().bold(), config.user);
        println!("{} {}", "Site:".cyan().bold(), config.site);
        println!(
            "{} {}",
            "Token:".cyan().bold(),
            mask_token(&config.token).dimmed()
        );
        println!(
            "{} {}",
            "Project:".cyan().bold(),
            config.default_project.as_deref().unwrap_or("(none)")
        );

        println!("\n{}", "[Locations]".blue().bold());
        if let Some(gp) = Config::global_config_path() {
            println!(
                "  {} {}",
                "Global:".cyan().bold(),
                if gp.exists() {
                    gp.display().to_string().cyan().to_string()
                } else {
                    "NOT FOUND".red().bold().to_string()
                }
            );
        }
        let lp = Config::local_config_path();
        println!(
            "  {} {}",
            "Local:".cyan().bold(),
            if lp.exists() {
                lp.display().to_string().cyan().to_string()
            } else {
                "NOT FOUND".red().bold().to_string()
            }
        );
    }

    Ok(())
}

fn show_file(label: &str, path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Err(format!(
            "{} config file not found at {}",
            label,
            path.display()
        ));
    }

    let file_config = FileConfig::load_path(path)?;
    println!(
        "{} Configuration ({})",
        label.yellow().bold(),
        path.display()
    );
    println!("--------------------------------------------------");

    println!("{}", "[auth]".cyan().bold());
    println!(
        "  {} {}",
        "username =".cyan().bold(),
        file_config.auth.username.as_deref().unwrap_or("")
    );
    println!(
        "  {} {}",
        "site     =".cyan().bold(),
        file_config.auth.site.as_deref().unwrap_or("")
    );
    println!(
        "  {} {}",
        "token    =".cyan().bold(),
        file_config
            .auth
            .token
            .as_ref()
            .map(|t| mask_token(t).dimmed().to_string())
            .unwrap_or_default()
    );

    if let Some(general) = file_config.general {
        println!("\n{}", "[general]".cyan().bold());
        println!(
            "  {} {}",
            "default_project =".cyan().bold(),
            general.default_project.as_deref().unwrap_or("")
        );
    }

    Ok(())
}

pub async fn run_set(key: String, value: String, global: bool, local: bool) -> Result<(), String> {
    let path = if global {
        Config::global_config_path().ok_or("Could not determine global config path")?
    } else if local {
        Config::local_config_path()
    } else {
        // Default to local if it exists, otherwise global?
        // Actually, let's follow git's lead: you must specify or it defaults to one.
        // Let's default to global for convenience if nothing specified, or local if jiri.toml exists.
        if Config::local_config_path().exists() {
            Config::local_config_path()
        } else {
            Config::global_config_path().ok_or("Could not determine global config path")?
        }
    };

    let mut file_config = FileConfig::load_path(&path)?;

    match key.as_str() {
        "auth.username" | "username" | "user" => {
            file_config.auth.username = Some(value);
        }
        "auth.token" | "token" => {
            file_config.auth.token = Some(value);
        }
        "auth.site" | "site" => {
            file_config.auth.site = Some(value);
        }
        "general.default_project" | "project" | "default_project" => {
            if file_config.general.is_none() {
                file_config.general = Some(Default::default());
            }
            file_config.general.as_mut().unwrap().default_project = Some(value);
        }
        _ => return Err(format!("Unknown configuration key: {}", key)),
    }

    file_config.save_path(&path)?;
    println!(
        "{} {} in {}",
        "Successfully set".green().bold(),
        key.cyan().bold(),
        path.display().to_string().dimmed()
    );

    Ok(())
}
