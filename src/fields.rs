use serde_json::Value;

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
