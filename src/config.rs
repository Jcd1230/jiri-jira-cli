use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub user: String,
    pub token: String,
    pub site: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let user = env::var("JIRA_API_USERNAME")
            .map_err(|_| "Missing JIRA_API_USERNAME environment variable")?;
        let token = env::var("JIRA_API_TOKEN")
            .map_err(|_| "Missing JIRA_API_TOKEN environment variable")?;
        let site = env::var("JIRA_SITE")
            .map_err(|_| "Missing JIRA_SITE environment variable")?;

        Ok(Config { user, token, site })
    }
}
