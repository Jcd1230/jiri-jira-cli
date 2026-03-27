use crate::config::{Config, FileConfig, mask_token};
use std::path::PathBuf;
use clap::builder::styling::{AnsiColor, Reset};

pub async fn run_show(global: bool, local: bool) -> Result<(), String> {
    let yellow = AnsiColor::Yellow.render_fg();
    let cyan = AnsiColor::Cyan.render_fg();
    let blue = AnsiColor::Blue.render_fg();
    let reset = Reset.render();

    if global {
        let path = Config::global_config_path().ok_or("Could not determine global config path")?;
        show_file("Global", &path)?;
    } else if local {
        let path = Config::local_config_path();
        show_file("Local", &path)?;
    } else {
        // Show effective config and source
        let config = Config::load()?;
        println!("{}Effective Configuration{} (from {})", yellow, reset, config.source);
        println!("--------------------------------------------------");
        println!("{}Username:{} {}", cyan, reset, config.user);
        println!("{}Site:    {} {}", cyan, reset, config.site);
        println!("{}Token:   {} {}", cyan, reset, mask_token(&config.token));
        println!("{}Project: {} {}", cyan, reset, config.default_project.as_deref().unwrap_or("(none)"));
        
        println!("\n{}[Locations]{}", blue, reset);
        if let Some(gp) = Config::global_config_path() {
            println!("  Global: {}", if gp.exists() { gp.display().to_string() } else { format!("{}NOT FOUND{}", AnsiColor::Red.render_fg(), reset) });
        }
        let lp = Config::local_config_path();
        println!("  Local:  {}", if lp.exists() { lp.display().to_string() } else { format!("{}NOT FOUND{}", AnsiColor::Red.render_fg(), reset) });
    }

    Ok(())
}

fn show_file(label: &str, path: &PathBuf) -> Result<(), String> {
    let yellow = AnsiColor::Yellow.render_fg();
    let cyan = AnsiColor::Cyan.render_fg();
    let reset = Reset.render();

    if !path.exists() {
        return Err(format!("{} config file not found at {}", label, path.display()));
    }

    let file_config = FileConfig::load_path(path)?;
    println!("{}{} Configuration{} ({})", yellow, label, reset, path.display());
    println!("--------------------------------------------------");
    
    println!("{}[auth]{}", cyan, reset);
    println!("  username = {:?}", file_config.auth.username.as_deref().unwrap_or(""));
    println!("  site     = {:?}", file_config.auth.site.as_deref().unwrap_or(""));
    println!("  token    = {:?}", file_config.auth.token.as_ref().map(|t| mask_token(t)).unwrap_or_default());

    if let Some(general) = file_config.general {
        println!("\n{}[general]{}", cyan, reset);
        println!("  default_project = {:?}", general.default_project.as_deref().unwrap_or(""));
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
    println!("Successfully set {} in {}", key, path.display());

    Ok(())
}
