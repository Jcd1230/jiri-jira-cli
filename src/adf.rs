use serde_json::{json, Value};

/// Extract plain text from Atlassian Document Format (ADF) JSON.
pub fn to_plain_text(node: &Value) -> String {
    if node.is_null() {
        return String::new();
    }

    if let Some(text) = node.get("text").and_then(|t| t.as_str()) {
        return text.to_string();
    }

    if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
        let parts: Vec<String> = content.iter().map(|child| to_plain_text(child)).collect::<Vec<_>>();
        let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
        return match node_type {
            "paragraph" | "heading" => format!("{}\n", parts.join("")),
            "bulletList" | "orderedList" => parts.join(""),
            "listItem" => format!("• {}\n", parts.join("").trim()),
            _ => parts.join(""),
        };
    }

    String::new()
}

/// Create a simple ADF JSON structure (single paragraph) from a string.
pub fn from_plain_text(text: &str) -> Value {
    json!({
        "type": "doc",
        "version": 1,
        "content": [{
            "type": "paragraph",
            "content": [{
                "type": "text",
                "text": text
            }]
        }]
    })
}
