use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub user: String,
    pub token: String,
    pub site: String,
}

#[derive(Deserialize)]
struct FileConfig {
    auth: AuthConfig,
}

#[derive(Deserialize)]
struct AuthConfig {
    username: String,
    token: String,
    site: String,
}

impl Config {
    /// Load config: try config file first, fall back to env vars.
    pub fn load() -> Result<Self, String> {
        Self::from_file().or_else(|_| Self::from_env())
    }

    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("jiri").join("config.toml"))
    }

    fn from_file() -> Result<Self, String> {
        let path = Self::config_path().ok_or("Could not determine config directory")?;
        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
        let file_config: FileConfig = toml::from_str(&contents)
            .map_err(|e| format!("Invalid config at {}: {}", path.display(), e))?;

        Ok(Config {
            user: file_config.auth.username,
            token: file_config.auth.token,
            site: file_config.auth.site,
        })
    }

    fn from_env() -> Result<Self, String> {
        let user = env::var("JIRA_API_USERNAME")
            .map_err(|_| "Missing JIRA_API_USERNAME environment variable")?;
        let token = env::var("JIRA_API_TOKEN")
            .map_err(|_| "Missing JIRA_API_TOKEN environment variable")?;
        let site = env::var("JIRA_SITE")
            .map_err(|_| "Missing JIRA_SITE environment variable")?;

        Ok(Config { user, token, site })
    }
}
