use crate::config::Config;
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub struct JiraClient {
    client: reqwest::Client,
    pub config: Config,
    field_cache: std::sync::Mutex<Option<FieldLookup>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLookup {
    pub id_to_name: HashMap<String, String>,
    pub name_to_id: HashMap<String, String>,
}

impl JiraClient {
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

    async fn request(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> Result<Value, String> {
        let url = format!("{}{}", self.config.site, path);
        let mut headers = HeaderMap::new();

        let auth = format!("{}:{}", self.config.user, self.config.token);
        let encoded_auth = general_purpose::STANDARD.encode(auth);
        
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", encoded_auth)).unwrap());
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        
        if body.is_some() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }

        let mut request_builder = self.client.request(method, &url).headers(headers);
        if let Some(b) = body {
            request_builder = request_builder.json(&b);
        }

        let response = request_builder.send().await.map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Jira request failed {}: {}", status, text));
        }

        response.json().await.map_err(|e| e.to_string())
    }

    pub async fn projects(&self) -> Result<Value, String> {
        self.request(reqwest::Method::GET, "/rest/api/3/project/search", None).await
    }

    pub async fn search(&self, jql: &str, fields: Vec<String>, max_results: i64, next_page_token: Option<String>) -> Result<Value, String> {
        let mut body = serde_json::json!({
            "jql": jql,
            "fields": fields,
            "maxResults": max_results,
        });

        if let Some(token) = next_page_token {
            body.as_object_mut().unwrap().insert("nextPageToken".to_string(), Value::String(token));
        }

        self.request(reqwest::Method::POST, "/rest/api/3/search/jql", Some(body)).await
    }

    pub async fn search_all(&self, jql: &str, fields: Vec<String>, limit: i64) -> Result<(Vec<Value>, bool), String> {
        let page_size = 100;
        let mut issues = Vec::new();
        let mut next_page_token: Option<String> = None;
        let mut more_available = false;

        while (issues.len() as i64) < limit {
            let remaining = limit - (issues.len() as i64);
            let page = self.search(jql, fields.clone(), remaining.min(page_size), next_page_token).await?;
            
            let page_issues = page["issues"].as_array().cloned().unwrap_or_default();
            issues.extend(page_issues.clone());
            
            next_page_token = page["nextPageToken"].as_str().map(|s| s.to_string());
            more_available = next_page_token.is_some();

            if next_page_token.is_none() || page_issues.is_empty() {
                break;
            }
        }

        Ok((issues, more_available))
    }

    pub async fn field_lookup(&self) -> Result<FieldLookup, String> {
        {
            let cache = self.field_cache.lock().unwrap();
            if let Some(ref lookup) = *cache {
                return Ok(lookup.clone());
            }
        }

        let data = self.request(reqwest::Method::GET, "/rest/api/3/field", None).await?;
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

        let lookup = FieldLookup { id_to_name, name_to_id };
        let mut cache = self.field_cache.lock().unwrap();
        *cache = Some(lookup.clone());
        Ok(lookup)
    }

    pub async fn get_issue(&self, key: &str) -> Result<Value, String> {
        let path = format!("/rest/api/3/issue/{}", key);
        self.request(reqwest::Method::GET, &path, None).await
    }

    pub async fn get_transitions(&self, key: &str) -> Result<Value, String> {
        let path = format!("/rest/api/3/issue/{}/transitions", key);
        self.request(reqwest::Method::GET, &path, None).await
    }

    pub async fn do_transition(&self, key: &str, transition_id: &str) -> Result<Value, String> {
        let path = format!("/rest/api/3/issue/{}/transitions", key);
        let body = serde_json::json!({
            "transition": { "id": transition_id }
        });
        self.request(reqwest::Method::POST, &path, Some(body)).await
    }

    pub async fn add_comment(&self, key: &str, body_text: &str) -> Result<Value, String> {
        let path = format!("/rest/api/3/issue/{}/comment", key);
        let body = serde_json::json!({
            "body": {
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": body_text
                    }]
                }]
            }
        });
        self.request(reqwest::Method::POST, &path, Some(body)).await
    }

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
            fields.as_object_mut().unwrap().insert(
                "description".to_string(),
                serde_json::json!({
                    "type": "doc",
                    "version": 1,
                    "content": [{
                        "type": "paragraph",
                        "content": [{
                            "type": "text",
                            "text": desc
                        }]
                    }]
                }),
            );
        }

        let body = serde_json::json!({ "fields": fields });
        self.request(reqwest::Method::POST, "/rest/api/3/issue", Some(body)).await
    }
}
