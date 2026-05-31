use serde_json::Value;
use crate::client::AtlassianClient;

/// Normalize a Jira field value into a human-readable string.
/// Handles strings, numbers, booleans, arrays, and complex objects (e.g., users, status).
pub fn normalize_value(val: &Value) -> String {
    if val.is_null() {
        return String::new();
    }

    if let Some(s) = val.as_str() {
        return s.to_string();
    }

    if let Some(n) = val.as_f64() {
        return n.to_string();
    }

    if let Some(b) = val.as_bool() {
        return b.to_string();
    }

    if let Some(arr) = val.as_array() {
        return arr
            .iter()
            .map(normalize_value)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
    }

    if let Some(obj) = val.as_object() {
        // Try various common display fields in Jira JSON objects
        let priority = ["displayName", "name", "value", "title", "label", "key"];
        for field in priority {
            if let Some(v) = obj.get(field).and_then(|v| v.as_str()) {
                return v.to_string();
            }
        }

        // Handle nested options or parent/child relationships
        if let Some(child) = obj.get("child") {
            return normalize_value(child);
        }
        if let Some(parent) = obj.get("parent") {
            return normalize_value(parent);
        }
    }

    val.to_string()
}

/// Helper to get a field value from an issue JSON and normalize it.
pub fn get_field_value(issue: &Value, key: &str) -> String {
    let key_lower = key.to_lowercase();

    // Top-level fields like "key" or "id" are not under "fields"
    if key_lower == "key" || key_lower == "issuekey" {
        return issue["key"]
            .as_str()
            .or_else(|| issue["fields"]["key"].as_str())
            .unwrap_or_default()
            .to_string();
    }

    if key_lower == "id" {
        return issue["id"].as_str().unwrap_or_default().to_string();
    }

    let val = &issue["fields"][key];
    normalize_value(val)
}

/// Parse a string in KEY=VALUE format.
fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let mut parts = s.splitn(2, '=');
    let key = parts.next().ok_or_else(|| "Empty key-value pair".to_string())?.trim().to_string();
    let val = parts.next().ok_or_else(|| format!("Missing '=' in field assignment: {}", s))?.trim().to_string();
    if key.is_empty() {
        return Err(format!("Empty key in field assignment: {}", s));
    }
    Ok((key, val))
}

/// Resolve a human-readable field name or ID to the exact Jira field ID.
pub fn resolve_field_id(key: &str, lookup: &crate::client::FieldLookup) -> Result<String, String> {
    // If it's a known ID directly, return it.
    if lookup.id_to_name.contains_key(key) {
        return Ok(key.to_string());
    }

    let key_lower = key.to_lowercase();
    if let Some(ids) = lookup.name_to_ids.get(&key_lower) {
        if ids.is_empty() {
            return Err(format!("Field '{}' not found", key));
        }
        if ids.len() > 1 {
            return Err(format!(
                "Field name '{}' is ambiguous. Multiple field IDs found: {}. Please use the exact field ID instead.",
                key,
                ids.join(", ")
            ));
        }
        return Ok(ids[0].clone());
    }

    // Default: try lowercase name_to_id fallback
    if let Some(id) = lookup.name_to_id.get(&key_lower) {
        return Ok(id.clone());
    }

    // If not found in cache, it might be a valid field ID that is not returned or custom.
    // If it looks like a custom field ID (customfield_XXXX), allow it.
    if key.starts_with("customfield_") {
        return Ok(key.to_string());
    }

    Err(format!("Unknown Jira field name or ID: '{}'", key))
}

/// Format a field value based on its Jira schema type.
pub async fn format_field_value(
    client: &AtlassianClient,
    field_id: &str,
    value_str: &str,
    schema: Option<&crate::client::FieldSchema>,
) -> Result<Value, String> {
    let field_type = schema.and_then(|s| s.field_type.as_deref()).unwrap_or("string");
    let items = schema.and_then(|s| s.items.as_deref());

    match field_type {
        "string" => {
            if field_id == "description" {
                Ok(crate::adf::from_plain_text(value_str))
            } else {
                Ok(Value::String(value_str.to_string()))
            }
        }
        "number" => {
            if let Ok(n) = value_str.parse::<f64>() {
                if let Ok(i) = value_str.parse::<i64>() {
                    Ok(Value::Number(i.into()))
                } else if let Some(f) = serde_json::Number::from_f64(n) {
                    Ok(Value::Number(f))
                } else {
                    Err(format!("Invalid float value: {}", value_str))
                }
            } else {
                Err(format!("Field '{}' expects a number, got '{}'", field_id, value_str))
            }
        }
        "boolean" => {
            match value_str.to_lowercase().as_str() {
                "true" | "yes" | "1" => Ok(Value::Bool(true)),
                "false" | "no" | "0" => Ok(Value::Bool(false)),
                _ => Err(format!("Field '{}' expects a boolean (true/false), got '{}'", field_id, value_str)),
            }
        }
        "user" => {
            let account_id = client.resolve_account_id(value_str).await?;
            Ok(serde_json::json!({ "accountId": account_id }))
        }
        "priority" | "resolution" | "project" => {
            if value_str.chars().all(|c| c.is_ascii_digit()) {
                Ok(serde_json::json!({ "id": value_str }))
            } else {
                Ok(serde_json::json!({ "name": value_str }))
            }
        }
        "option" | "option-with-child" => {
            Ok(serde_json::json!({ "value": value_str }))
        }
        "array" => {
            let item_type = items.unwrap_or("string");
            let parts: Vec<&str> = value_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            let mut arr = Vec::new();
            for part in parts {
                match item_type {
                    "string" => {
                        arr.push(Value::String(part.to_string()));
                    }
                    "user" => {
                        let account_id = client.resolve_account_id(part).await?;
                        arr.push(serde_json::json!({ "accountId": account_id }));
                    }
                    "component" | "version" | "option" => {
                        if item_type == "option" {
                            arr.push(serde_json::json!({ "value": part }));
                        } else if part.chars().all(|c| c.is_ascii_digit()) && item_type != "component" {
                            arr.push(serde_json::json!({ "id": part }));
                        } else {
                            arr.push(serde_json::json!({ "name": part }));
                        }
                    }
                    _ => {
                        arr.push(serde_json::json!({ "name": part }));
                    }
                }
            }
            Ok(Value::Array(arr))
        }
        _ => {
            let trimmed = value_str.trim();
            if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                || (trimmed.starts_with('[') && trimmed.ends_with(']'))
            {
                serde_json::from_str(trimmed)
                    .map_err(|e| format!("Failed to parse field '{}' value as JSON: {}", field_id, e))
            } else {
                Ok(Value::String(value_str.to_string()))
            }
        }
    }
}

/// Construct the final update payload Map from all input options.
pub async fn build_update_payload(
    client: &AtlassianClient,
    summary: Option<String>,
    description: Option<String>,
    labels: Option<String>,
    assignee: Option<String>,
    custom_fields: &[String],
    custom_json_fields: &[String],
) -> Result<serde_json::Map<String, Value>, String> {
    let mut fields = serde_json::Map::new();

    if let Some(s) = summary {
        fields.insert("summary".to_string(), Value::String(s));
    }

    if let Some(d) = description {
        fields.insert("description".to_string(), crate::adf::from_plain_text(&d));
    }

    if let Some(l) = labels {
        let label_list = l
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| Value::String(s.to_string()))
            .collect::<Vec<_>>();
        fields.insert("labels".to_string(), Value::Array(label_list));
    }

    if let Some(a) = assignee {
        let account_id = client.resolve_account_id(&a).await?;
        fields.insert(
            "assignee".to_string(),
            serde_json::json!({ "accountId": account_id }),
        );
    }

    if !custom_fields.is_empty() || !custom_json_fields.is_empty() {
        let lookup = client.field_lookup().await?;

        for item in custom_fields {
            let (key, val) = parse_key_value(item)?;
            let resolved_id = resolve_field_id(&key, &lookup)?;
            let schema = lookup.id_to_schema.get(&resolved_id);
            let formatted_val = format_field_value(client, &resolved_id, &val, schema).await?;
            fields.insert(resolved_id, formatted_val);
        }

        for item in custom_json_fields {
            let (key, val) = parse_key_value(item)?;
            let resolved_id = resolve_field_id(&key, &lookup)?;
            let parsed_json: Value = serde_json::from_str(&val)
                .map_err(|e| format!("Invalid JSON for field '{}': {}", key, e))?;
            fields.insert(resolved_id, parsed_json);
        }
    }

    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::FieldLookup;
    use std::collections::HashMap;

    #[test]
    fn test_parse_key_value() {
        assert_eq!(parse_key_value("foo=bar").unwrap(), ("foo".to_string(), "bar".to_string()));
        assert_eq!(parse_key_value("foo = bar ").unwrap(), ("foo".to_string(), "bar".to_string()));
        assert_eq!(parse_key_value("foo=bar=baz").unwrap(), ("foo".to_string(), "bar=baz".to_string()));
        assert!(parse_key_value("foo").is_err());
        assert!(parse_key_value("=bar").is_err());
    }

    #[test]
    fn test_resolve_field_id() {
        let mut id_to_name = HashMap::new();
        id_to_name.insert("customfield_10010".to_string(), "Story Points".to_string());
        id_to_name.insert("summary".to_string(), "Summary".to_string());

        let mut name_to_id = HashMap::new();
        name_to_id.insert("story points".to_string(), "customfield_10010".to_string());
        name_to_id.insert("summary".to_string(), "summary".to_string());

        let mut name_to_ids = HashMap::new();
        name_to_ids.insert("story points".to_string(), vec!["customfield_10010".to_string()]);
        name_to_ids.insert("summary".to_string(), vec!["summary".to_string()]);

        // Duplicate name entry
        name_to_ids.insert("duplicate field".to_string(), vec!["customfield_10011".to_string(), "customfield_10012".to_string()]);

        let lookup = FieldLookup {
            id_to_name,
            name_to_id,
            id_to_schema: HashMap::new(),
            name_to_ids,
        };

        // Check exact ID resolve
        assert_eq!(resolve_field_id("customfield_10010", &lookup).unwrap(), "customfield_10010");
        // Check lookup by display name (case-insensitive)
        assert_eq!(resolve_field_id("Story Points", &lookup).unwrap(), "customfield_10010");
        // Check duplicate name error
        assert!(resolve_field_id("Duplicate Field", &lookup).is_err());
        // Check fallback to unknown custom field ID
        assert_eq!(resolve_field_id("customfield_12345", &lookup).unwrap(), "customfield_12345");
        // Check unknown display name error
        assert!(resolve_field_id("Nonexistent Field", &lookup).is_err());
    }
}
