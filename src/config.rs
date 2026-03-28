use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ConfigSource {
    LocalFile(PathBuf),
    GlobalFile(PathBuf),
    Env,
}

impl std::fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigSource::LocalFile(p) => write!(f, "Local file ({})", p.display()),
            ConfigSource::GlobalFile(p) => write!(f, "Global file ({})", p.display()),
            ConfigSource::Env => write!(f, "Environment variables"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub user: String,
    pub token: String,
    pub site: String,
    pub default_project: Option<String>,
    pub source: ConfigSource,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct FileConfig {
    pub auth: AuthConfig,
    pub general: Option<GeneralConfig>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AuthConfig {
    pub username: Option<String>,
    pub token: Option<String>,
    pub site: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GeneralConfig {
    pub default_project: Option<String>,
}

impl FileConfig {
    pub fn load_path(path: &PathBuf) -> Result<Self, String> {
        if !path.exists() {
            return Ok(FileConfig::default());
        }
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
        toml::from_str(&contents)
            .map_err(|e| format!("Invalid config at {}: {}", path.display(), e))
    }

    pub fn save_path(&self, path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let contents = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, contents).map_err(|e| e.to_string())
    }
}

impl Config {
    /// Load config with layering: Env < Global < Local.
    pub fn load() -> Result<Self, String> {
        // 1. Try to build a base from any source that provides a complete configuration.
        // We try them in order of priority (lowest to highest) as a fallback mechanism,
        // but the layering below will ensure the correct final priority.
        let mut config = Self::from_env()
            .or_else(|_| Self::from_global_file())
            .or_else(|_| Self::from_local_file())
            .map_err(|e| format!("Could not find a complete configuration source: {}", e))?;

        // 2. Layer Global overrides if they exist
        if let Some(global_path) = Self::global_config_path() {
            if global_path.exists() {
                if let Ok(global_file) = FileConfig::load_path(&global_path) {
                    if let Some(u) = global_file.auth.username {
                        config.user = u;
                        config.source = ConfigSource::GlobalFile(global_path.clone());
                    }
                    if let Some(t) = global_file.auth.token {
                        config.token = t;
                        config.source = ConfigSource::GlobalFile(global_path.clone());
                    }
                    if let Some(s) = global_file.auth.site {
                        config.site = s;
                        config.source = ConfigSource::GlobalFile(global_path.clone());
                    }
                    if let Some(g) = global_file.general {
                        if let Some(p) = g.default_project {
                            config.default_project = Some(p);
                            config.source = ConfigSource::GlobalFile(global_path.clone());
                        }
                    }
                }
            }
        }

        // 3. Layer Local overrides if they exist
        let local_path = Self::local_config_path();
        if local_path.exists() {
            if let Ok(local_file) = FileConfig::load_path(&local_path) {
                if let Some(u) = local_file.auth.username {
                    config.user = u;
                    config.source = ConfigSource::LocalFile(local_path.clone());
                }
                if let Some(t) = local_file.auth.token {
                    config.token = t;
                    config.source = ConfigSource::LocalFile(local_path.clone());
                }
                if let Some(s) = local_file.auth.site {
                    config.site = s;
                    config.source = ConfigSource::LocalFile(local_path.clone());
                }
                if let Some(g) = local_file.general {
                    if let Some(p) = g.default_project {
                        config.default_project = Some(p);
                        config.source = ConfigSource::LocalFile(local_path.clone());
                    }
                }
            }
        }

        Ok(config)
    }

    pub fn local_config_path() -> PathBuf {
        PathBuf::from("jiri.toml")
    }

    fn from_local_file() -> Result<Self, String> {
        let path = Self::local_config_path();
        if !path.exists() {
            return Err("Local config not found".to_string());
        }
        Self::parse_file(path.clone(), ConfigSource::LocalFile(path))
    }

    pub fn global_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("jiri").join("config.toml"))
    }

    fn from_global_file() -> Result<Self, String> {
        let path = Self::global_config_path().ok_or("Could not determine config directory")?;

        if !path.exists() {
            return Err("Global config not found".to_string());
        }
        Self::parse_file(path.clone(), ConfigSource::GlobalFile(path))
    }

    fn parse_file(path: PathBuf, source: ConfigSource) -> Result<Self, String> {
        let file_config = FileConfig::load_path(&path)?;

        let user = file_config
            .auth
            .username
            .ok_or_else(|| format!("Missing auth.username in {}", path.display()))?;
        let token = file_config
            .auth
            .token
            .ok_or_else(|| format!("Missing auth.token in {}", path.display()))?;
        let site = file_config
            .auth
            .site
            .ok_or_else(|| format!("Missing auth.site in {}", path.display()))?;

        Ok(Config {
            user,
            token,
            site,
            default_project: file_config.general.and_then(|g| g.default_project),
            source,
        })
    }

    fn from_env() -> Result<Self, String> {
        let user = env::var("JIRA_API_USERNAME")
            .map_err(|_| "Missing JIRA_API_USERNAME environment variable")?;
        let token = env::var("JIRA_API_TOKEN")
            .map_err(|_| "Missing JIRA_API_TOKEN environment variable")?;
        let site = env::var("JIRA_SITE").map_err(|_| "Missing JIRA_SITE environment variable")?;
        let default_project = env::var("JIRA_DEFAULT_PROJECT").ok();

        Ok(Config {
            user,
            token,
            site,
            default_project,
            source: ConfigSource::Env,
        })
    }
}

pub fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        return "****".to_string();
    }
    format!("{}...{}", &token[..4], &token[token.len() - 4..])
}
