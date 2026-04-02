use crate::adf;
use crate::config::Config;
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::collections::HashMap;

/// API types supported by the Atlassian client.
pub enum AtlassianApi {
    Jira,
    Confluence,
    ConfluenceV1,
}

/// Client for interacting with Atlassian Cloud REST APIs (Jira and Confluence).
pub struct AtlassianClient {
    client: reqwest::Client,
    config: Config,
    field_cache: std::sync::Mutex<Option<FieldLookup>>,
}

/// Metadata lookup table for Jira fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLookup {
    /// Map of Jira internal field ID to human-readable name.
    pub id_to_name: HashMap<String, String>,
    /// Map of lowercase human-readable name to Jira internal field ID.
    pub name_to_id: HashMap<String, String>,
}

impl AtlassianClient {
    /// Create a new AtlassianClient with the provided configuration.
    pub fn new(config: Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
            field_cache: std::sync::Mutex::new(None),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get current user information (Jira).
    pub async fn myself(&self) -> Result<Value, String> {
        self.request(AtlassianApi::Jira, reqwest::Method::GET, "/myself", None)
            .await
    }

    /// Perform a generic authenticated request to the Atlassian API.
    async fn request(
        &self,
        api: AtlassianApi,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, String> {
        let prefix = match api {
            AtlassianApi::Jira => "/rest/api/3",
            AtlassianApi::Confluence => "/wiki/api/v2",
            AtlassianApi::ConfluenceV1 => "/wiki/rest/api",
        };

        let url = format!("{}{}{}", self.config.site, prefix, path);

        if std::env::var("JIRI_VERBOSE").is_ok() {
            eprintln!("DEBUG: {} {}", method, url);
            if let Some(ref b) = body {
                eprintln!(
                    "DEBUG: Body: {}",
                    serde_json::to_string_pretty(b).unwrap_or_default()
                );
            }
        }

        let mut headers = HeaderMap::new();

        let auth = format!("{}:{}", self.config.user, self.config.token);
        let encoded_auth = general_purpose::STANDARD.encode(auth);

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", encoded_auth)).unwrap(),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        if body.is_some() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }

        let mut request_builder = self.client.request(method, &url).headers(headers);
        if let Some(b) = body {
            request_builder = request_builder.json(&b);
        }

        let response = request_builder.send().await.map_err(|e| e.to_string())?;

        if std::env::var("JIRI_VERBOSE").is_ok() {
            eprintln!("DEBUG: Response: {}", response.status());
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Atlassian request failed ({}): {}", status, text));
        }

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(Value::Null);
        }

        let json: Value = response.json().await.map_err(|e| e.to_string())?;
        if std::env::var("JIRI_VERBOSE").is_ok() {
            eprintln!(
                "DEBUG: JSON: {}",
                serde_json::to_string_pretty(&json).unwrap_or_default()
            );
        }
        Ok(json)
    }

    /// Perform a multipart upload request to the Atlassian API.
    async fn request_multipart(
        &self,
        api: AtlassianApi,
        path: &str,
        file_path: &str,
        comment: Option<String>,
    ) -> Result<Value, String> {
        let prefix = match api {
            AtlassianApi::Jira => "/rest/api/3",
            AtlassianApi::Confluence => "/wiki/api/v2",
            AtlassianApi::ConfluenceV1 => "/wiki/rest/api",
        };

        let url = format!("{}{}{}", self.config.site, prefix, path);

        if std::env::var("JIRI_VERBOSE").is_ok() {
            eprintln!("DEBUG: POST (multipart) {}", url);
            eprintln!("DEBUG: File: {}", file_path);
        }

        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid file path: {}", file_path))?
            .to_string();

        let file_content = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;

        let mime = mime_guess::from_path(file_path)
            .first_raw()
            .unwrap_or("application/octet-stream");

        let part = reqwest::multipart::Part::bytes(file_content)
            .file_name(file_name)
            .mime_str(mime)
            .map_err(|e| e.to_string())?;

        let mut form = reqwest::multipart::Form::new().part("file", part);

        // Add comment only for Confluence v2 attachments
        if let (AtlassianApi::Confluence, Some(c)) = (&api, comment) {
            form = form.text("comment", c);
        }

        let mut headers = HeaderMap::new();
        let auth = format!("{}:{}", self.config.user, self.config.token);
        let encoded_auth = general_purpose::STANDARD.encode(auth);

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", encoded_auth)).unwrap(),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert("X-Atlassian-Token", HeaderValue::from_static("no-check"));

        let response = self.client
            .post(&url)
            .headers(headers)
            .multipart(form)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if std::env::var("JIRI_VERBOSE").is_ok() {
            eprintln!("DEBUG: Response: {}", response.status());
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Atlassian request failed ({}): {}", status, text));
        }

        let json: Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(json)
    }

    /// List all projects visible to the user, fetching every page.
    pub async fn projects_all(&self) -> Result<Vec<Value>, String> {
        let page_size = 100;
        let mut start_at = 0;
        let mut projects = Vec::new();

        loop {
            let path = format!(
                "/project/search?startAt={}&maxResults={}",
                start_at, page_size
            );
            let data = self
                .request(AtlassianApi::Jira, reqwest::Method::GET, &path, None)
                .await?;

            let page_projects = data["values"].as_array().cloned().unwrap_or_default();
            let returned = page_projects.len() as i64;
            projects.extend(page_projects);

            let total = data["total"].as_i64().unwrap_or(projects.len() as i64);
            start_at += returned;

            if returned == 0 || start_at >= total {
                break;
            }
        }

        Ok(projects)
    }

    /// Perform a JQL search.
    pub async fn search(
        &self,
        jql: &str,
        fields: Vec<String>,
        max_results: i64,
        next_page_token: Option<String>,
    ) -> Result<Value, String> {
        let mut body = serde_json::json!({
            "jql": jql,
            "fields": fields,
            "maxResults": max_results,
        });

        if let Some(token) = next_page_token {
            body.as_object_mut()
                .unwrap()
                .insert("nextPageToken".to_string(), Value::String(token));
        }

        self.request(
            AtlassianApi::Jira,
            reqwest::Method::POST,
            "/search/jql",
            Some(body),
        )
        .await
    }

    /// Search for all issues matching JQL up to a limit, handling pagination automatically.
    pub async fn search_all(
        &self,
        jql: &str,
        fields: Vec<String>,
        limit: i64,
    ) -> Result<(Vec<Value>, bool), String> {
        let page_size = 100;
        let mut issues = Vec::new();
        let mut next_page_token: Option<String> = None;
        let mut more_available = false;

        while (issues.len() as i64) < limit {
            let remaining = limit - (issues.len() as i64);
            let page = self
                .search(
                    jql,
                    fields.clone(),
                    remaining.min(page_size),
                    next_page_token,
                )
                .await?;

            let page_issues = page["issues"].as_array().cloned().unwrap_or_default();
            let page_is_empty = page_issues.is_empty();
            issues.extend(page_issues);

            next_page_token = page["nextPageToken"].as_str().map(|s| s.to_string());
            more_available = next_page_token.is_some();

            if next_page_token.is_none() || page_is_empty {
                break;
            }
        }

        Ok((issues, more_available))
    }

    /// Fetch field definitions and build a lookup table. Caches the result.
    pub async fn field_lookup(&self) -> Result<FieldLookup, String> {
        {
            let cache = self.field_cache.lock().unwrap();
            if let Some(ref lookup) = *cache {
                return Ok(lookup.clone());
            }
        }

        let data = self
            .request(AtlassianApi::Jira, reqwest::Method::GET, "/field", None)
            .await?;
        let mut id_to_name = HashMap::new();
        let mut name_to_id = HashMap::new();

        if let Some(fields) = data.as_array() {
            for f in fields {
                let id = f["id"].as_str().unwrap_or_default().to_string();
                let name = f["name"].as_str().unwrap_or_default().to_string();
                if !id.is_empty() && !name.is_empty() {
                    id_to_name.insert(id.clone(), name.clone());
                    name_to_id.insert(name.to_lowercase(), id);
                }
            }
        }

        let lookup = FieldLookup {
            id_to_name,
            name_to_id,
        };
        let mut cache = self.field_cache.lock().unwrap();
        *cache = Some(lookup.clone());
        Ok(lookup)
    }

    /// Get a single issue by key.
    pub async fn get_issue(&self, key: &str) -> Result<Value, String> {
        let path = format!("/issue/{}", key);
        self.request(AtlassianApi::Jira, reqwest::Method::GET, &path, None)
            .await
    }

    /// List available transitions for an issue.
    pub async fn get_transitions(&self, key: &str) -> Result<Value, String> {
        let path = format!("/issue/{}/transitions", key);
        self.request(AtlassianApi::Jira, reqwest::Method::GET, &path, None)
            .await
    }

    /// Perform a transition on an issue.
    pub async fn do_transition(&self, key: &str, transition_id: &str) -> Result<Value, String> {
        let path = format!("/issue/{}/transitions", key);
        let body = serde_json::json!({
            "transition": { "id": transition_id }
        });
        self.request(AtlassianApi::Jira, reqwest::Method::POST, &path, Some(body))
            .await
    }

    /// Add a comment to an issue.
    pub async fn add_comment(&self, key: &str, body_text: &str) -> Result<Value, String> {
        let path = format!("/issue/{}/comment", key);
        let body = serde_json::json!({
            "body": adf::from_plain_text(body_text)
        });
        self.request(AtlassianApi::Jira, reqwest::Method::POST, &path, Some(body))
            .await
    }

    /// Add a comment to an issue with an embedded external media object (e.g. an attachment).
    pub async fn add_comment_with_external_media(
        &self,
        key: &str,
        body_text: &str,
        url: &str,
    ) -> Result<Value, String> {
        let path = format!("/issue/{}/comment", key);
        let body = serde_json::json!({
            "body": adf::from_plain_text_with_external_media(body_text, url)
        });
        self.request(AtlassianApi::Jira, reqwest::Method::POST, &path, Some(body))
            .await
    }

    /// Add a comment to an issue with an embedded attachment using its Media ID.
    pub async fn add_comment_with_attachment(
        &self,
        key: &str,
        body_text: &str,
        media_id: &str,
    ) -> Result<Value, String> {
        let path = format!("/issue/{}/comment", key);
        let body = serde_json::json!({
            "body": adf::from_plain_text_with_attachment(body_text, media_id)
        });
        self.request(AtlassianApi::Jira, reqwest::Method::POST, &path, Some(body))
            .await
    }

    /// Retrieve the Media Services UUID for a given numeric attachment ID.
    /// This follows the redirect of the attachment content URL.
    pub async fn get_attachment_media_id(&self, attachment_id: &str) -> Result<String, String> {
        let url = format!("{}/rest/api/3/attachment/content/{}", self.config.site, attachment_id);

        let mut headers = HeaderMap::new();
        let auth = format!("{}:{}", self.config.user, self.config.token);
        let encoded_auth = general_purpose::STANDARD.encode(auth);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", encoded_auth)).unwrap(),
        );

        // We use a separate client that doesn't automatically follow redirects so we can see the Location header.
        let no_redirect_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| e.to_string())?;

        let response = no_redirect_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|l| l.to_str().ok())
            .ok_or_else(|| "No redirect location found for attachment content".to_string())?;

        // The URL typically looks like https://.../file/<UUID>/binary?...
        let parts: Vec<&str> = location.split("/file/").collect();
        if parts.len() < 2 {
            return Err(format!("Could not find Media UUID in redirect URL: {}", location));
        }
        let uuid_part = parts[1].split('/').next().ok_or("Malformed Media UUID path")?;
        
        Ok(uuid_part.to_string())
    }

    /// Add an attachment to a Jira issue.
    pub async fn attach_to_issue(&self, key: &str, file_path: &str) -> Result<Value, String> {
        let path = format!("/issue/{}/attachments", key);
        self.request_multipart(AtlassianApi::Jira, &path, file_path, None)
            .await
    }

    /// Create a new issue in the specified project.
    pub async fn create_issue(
        &self,
        project_key: &str,
        summary: &str,
        issue_type: &str,
        description: Option<&str>,
    ) -> Result<Value, String> {
        let mut fields = serde_json::json!({
            "project": { "key": project_key },
            "summary": summary,
            "issuetype": { "name": issue_type },
        });

        if let Some(desc) = description {
            fields
                .as_object_mut()
                .unwrap()
                .insert("description".to_string(), adf::from_plain_text(desc));
        }

        let body = serde_json::json!({ "fields": fields });
        self.request(
            AtlassianApi::Jira,
            reqwest::Method::POST,
            "/issue",
            Some(body),
        )
        .await
    }

    /// Search Jira users by query string.
    pub async fn search_users(&self, query: &str) -> Result<Value, String> {
        let path = format!("/user/search?query={}", urlencoding::encode(query));
        self.request(AtlassianApi::Jira, reqwest::Method::GET, &path, None)
            .await
    }

    /// Update fields on an existing issue.
    pub async fn update_issue(
        &self,
        key: &str,
        fields: serde_json::Value,
    ) -> Result<Value, String> {
        let path = format!("/issue/{}", key);
        let body = serde_json::json!({ "fields": fields });
        self.request(AtlassianApi::Jira, reqwest::Method::PUT, &path, Some(body))
            .await
    }

    // --- Confluence Methods ---

    /// Search for Confluence pages using CQL (v1 API).
    pub async fn search_pages(&self, cql: &str, limit: i64) -> Result<Value, String> {
        let path = format!("/search?cql={}&limit={}", urlencoding::encode(cql), limit);
        self.request(
            AtlassianApi::ConfluenceV1,
            reqwest::Method::GET,
            &path,
            None,
        )
        .await
    }

    /// Get a Confluence page by ID, including ADF body (v2 API).
    pub async fn get_page(&self, id: &str) -> Result<Value, String> {
        let path = format!("/pages/{}?body-format=atlas_doc_format", id);
        self.request(AtlassianApi::Confluence, reqwest::Method::GET, &path, None)
            .await
    }

    /// Resolve a Space Key to a Space ID (v2 API).
    pub async fn get_space_id(&self, key: &str) -> Result<String, String> {
        // If it's already numeric, return it
        if key.chars().all(|c| c.is_ascii_digit()) {
            return Ok(key.to_string());
        }

        let path = format!("/spaces?keys={}", key);
        let data = self
            .request(AtlassianApi::Confluence, reqwest::Method::GET, &path, None)
            .await?;
        let spaces = data["results"].as_array().ok_or("No spaces found")?;

        for s in spaces {
            if s["key"].as_str() == Some(key) {
                return s["id"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or("Space has no ID".to_string());
            }
        }
        Err(format!("Could not find space with key '{}'", key))
    }

    /// Create a new Confluence page (v2 API).
    pub async fn create_page(
        &self,
        space_id: &str,
        title: &str,
        parent_id: Option<&str>,
        adf_body: &Value,
    ) -> Result<Value, String> {
        let stringified_adf = serde_json::to_string(adf_body).map_err(|e| e.to_string())?;

        let mut body = serde_json::json!({
            "spaceId": space_id,
            "status": "current",
            "title": title,
            "body": {
                "representation": "atlas_doc_format",
                "value": stringified_adf
            }
        });

        if let Some(pid) = parent_id {
            body.as_object_mut()
                .unwrap()
                .insert("parentId".to_string(), serde_json::json!(pid));
        }

        self.request(
            AtlassianApi::Confluence,
            reqwest::Method::POST,
            "/pages",
            Some(body),
        )
        .await
    }

    /// Update a Confluence page (v2 API).
    pub async fn update_page(
        &self,
        id: &str,
        title: &str,
        space_id: &str,
        adf_body: &Value,
        version: i64,
        minor_edit: bool,
    ) -> Result<Value, String> {
        let path = format!("/pages/{}", id);

        // Confluence v2 requirement: body.value must be a stringified JSON string
        let stringified_adf = serde_json::to_string(adf_body).map_err(|e| e.to_string())?;

        let body = serde_json::json!({
            "id": id,
            "status": "current",
            "title": title,
            "spaceId": space_id,
            "body": {
                "representation": "atlas_doc_format",
                "value": stringified_adf
            },
            "version": {
                "number": version,
                "minorEdit": minor_edit
            }
        });

        self.request(
            AtlassianApi::Confluence,
            reqwest::Method::PUT,
            &path,
            Some(body),
        )
        .await
    }

    /// Add an attachment to a Confluence page (v2 API) with optional comment.
    pub async fn attach_to_page(
        &self,
        id: &str,
        file_path: &str,
        comment: Option<String>,
    ) -> Result<Value, String> {
        let path = format!("/pages/{}/attachments", id);
        self.request_multipart(AtlassianApi::Confluence, &path, file_path, comment)
            .await
    }
}
