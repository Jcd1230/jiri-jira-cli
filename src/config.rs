use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub user: String,
    pub token: String,
    pub site: String,
    pub default_project: Option<String>,
}

#[derive(Deserialize)]
struct FileConfig {
    auth: AuthConfig,
    general: Option<GeneralConfig>,
}

#[derive(Deserialize)]
struct AuthConfig {
    username: String,
    token: String,
    site: String,
}

#[derive(Deserialize)]
struct GeneralConfig {
    default_project: Option<String>,
}

impl Config {
    /// Load config: try local jiri.toml, then global config file, then env vars.
    pub fn load() -> Result<Self, String> {
        Self::from_local_file()
            .or_else(|_| Self::from_global_file())
            .or_else(|_| Self::from_env())
    }

    fn from_local_file() -> Result<Self, String> {
        let path = PathBuf::from("jiri.toml");
        if !path.exists() {
            return Err("Local config not found".to_string());
        }
        Self::parse_file(path)
    }

    fn from_global_file() -> Result<Self, String> {
        let path = dirs::config_dir()
            .map(|d| d.join("jiri").join("config.toml"))
            .ok_or("Could not determine config directory")?;
        
        if !path.exists() {
            return Err("Global config not found".to_string());
        }
        Self::parse_file(path)
    }

    fn parse_file(path: PathBuf) -> Result<Self, String> {
        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
        let file_config: FileConfig = toml::from_str(&contents)
            .map_err(|e| format!("Invalid config at {}: {}", path.display(), e))?;

        Ok(Config {
            user: file_config.auth.username,
            token: file_config.auth.token,
            site: file_config.auth.site,
            default_project: file_config.general.and_then(|g| g.default_project),
        })
    }

    fn from_env() -> Result<Self, String> {
        let user = env::var("JIRA_API_USERNAME")
            .map_err(|_| "Missing JIRA_API_USERNAME environment variable")?;
        let token = env::var("JIRA_API_TOKEN")
            .map_err(|_| "Missing JIRA_API_TOKEN environment variable")?;
        let site = env::var("JIRA_SITE")
            .map_err(|_| "Missing JIRA_SITE environment variable")?;
        let default_project = env::var("JIRA_DEFAULT_PROJECT").ok();

        Ok(Config { user, token, site, default_project })
    }
}
