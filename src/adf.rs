use pulldown_cmark::{Event, Parser, Tag, TagEnd};
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
        let parts: Vec<String> = content
            .iter()
            .map(|child| to_plain_text(child))
            .collect::<Vec<_>>();
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

/// Recursively extract all text content from a node for matching purposes (no formatting).
pub fn get_node_text(node: &Value) -> String {
    if node.is_null() {
        return String::new();
    }

    if let Some(text) = node.get("text").and_then(|t| t.as_str()) {
        return text.to_string();
    }

    if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
        return content
            .iter()
            .map(|child| get_node_text(child))
            .collect::<Vec<_>>()
            .join("");
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

/// Convert Markdown to ADF nodes.
pub fn from_markdown(markdown: &str) -> Vec<Value> {
    let parser = Parser::new(markdown);
    let mut doc_nodes = Vec::new();
    let mut current_block: Option<Value> = None;
    let mut current_block_content = Vec::new();
    let mut list_stack: Vec<(Value, Vec<Value>)> = Vec::new();
    let mut marks: Vec<Value> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    current_block = Some(json!({ "type": "paragraph" }));
                }
                Tag::Heading { level, .. } => {
                    current_block =
                        Some(json!({ "type": "heading", "attrs": { "level": level as u8 } }));
                }
                Tag::List(ordered) => {
                    let list_type = if ordered.is_some() {
                        "orderedList"
                    } else {
                        "bulletList"
                    };
                    list_stack.push((json!({ "type": list_type }), Vec::new()));
                }
                Tag::Item => {
                    current_block = Some(json!({ "type": "listItem" }));
                }
                Tag::Strong => marks.push(json!({ "type": "strong" })),
                Tag::Emphasis => marks.push(json!({ "type": "em" })),
                Tag::Link { dest_url, .. } => {
                    marks.push(json!({ "type": "link", "attrs": { "href": dest_url.to_string() } }))
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph | TagEnd::Heading(_) => {
                    if let Some(mut block) = current_block.take() {
                        block["content"] = json!(current_block_content);
                        current_block_content = Vec::new();
                        if list_stack.is_empty() {
                            doc_nodes.push(block);
                        } else {
                            list_stack.last_mut().unwrap().1.push(block);
                        }
                    }
                }
                TagEnd::List(_) => {
                    if let Some((mut list, content)) = list_stack.pop() {
                        list["content"] = json!(content);
                        if list_stack.is_empty() {
                            doc_nodes.push(list);
                        } else {
                            list_stack.last_mut().unwrap().1.push(list);
                        }
                    }
                }
                TagEnd::Item => {
                    if let Some(mut item) = current_block.take() {
                        item["content"] = json!(current_block_content);
                        current_block_content = Vec::new();
                        list_stack.last_mut().unwrap().1.push(item);
                    }
                }
                TagEnd::Strong | TagEnd::Emphasis | TagEnd::Link => {
                    marks.pop();
                }
                _ => {}
            },
            Event::Text(text) => {
                let mut node = json!({
                    "type": "text",
                    "text": text.to_string()
                });
                if !marks.is_empty() {
                    node["marks"] = json!(marks);
                }
                current_block_content.push(node);
            }
            Event::SoftBreak => {
                current_block_content.push(json!({ "type": "text", "text": " " }));
            }
            Event::HardBreak => {
                current_block_content.push(json!({ "type": "hardBreak" }));
            }
            _ => {}
        }
    }

    doc_nodes
}

/// Append nodes to the end of the ADF document.
pub fn append_nodes(doc: &mut Value, new_nodes: Vec<Value>) {
    if let Some(content) = doc.get_mut("content").and_then(|c| c.as_array_mut()) {
        content.extend(new_nodes);
    }
}

/// Prepend nodes to the beginning of the ADF document.
pub fn prepend_nodes(doc: &mut Value, new_nodes: Vec<Value>) {
    if let Some(content) = doc.get_mut("content").and_then(|c| c.as_array_mut()) {
        for node in new_nodes.into_iter().rev() {
            content.insert(0, node);
        }
    }
}

/// Recursively replace text in the ADF tree.
pub fn replace_text(node: &mut Value, old: &str, new: &str) {
    if let Some(text_val) = node.get_mut("text") {
        if let Some(text) = text_val.as_str() {
            if text.contains(old) {
                let replaced = text.replace(old, new);
                *text_val = json!(replaced);
            }
        }
    }

    if let Some(content) = node.get_mut("content").and_then(|c| c.as_array_mut()) {
        for child in content {
            replace_text(child, old, new);
        }
    }
}

/// Locate the index of an anchor node in the top-level content array.
pub fn find_anchor_index(doc: &Value, selector: &str) -> Result<usize, String> {
    let parts: Vec<&str> = selector.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("Anchor selector must be in 'type:query' format".to_string());
    }

    let anchor_type = parts[0];
    let query = parts[1].to_lowercase();

    let content = doc
        .get("content")
        .and_then(|c| c.as_array())
        .ok_or("Invalid ADF: missing top-level content array")?;

    let mut matches = Vec::new();

    for (index, node) in content.iter().enumerate() {
        let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let mut is_match = false;

        match anchor_type {
            "heading" => {
                if node_type == "heading" && get_node_text(node).to_lowercase().contains(&query) {
                    is_match = true;
                }
            }
            "panel" => {
                if node_type == "panel" && get_node_text(node).to_lowercase().contains(&query) {
                    is_match = true;
                }
            }
            "list" => {
                if (node_type == "bulletList" || node_type == "orderedList")
                    && node
                        .get("content")
                        .and_then(|c| c.as_array())
                        .and_then(|a| a.first())
                        .map(|item| get_node_text(item).to_lowercase().contains(&query))
                        .unwrap_or(false)
                {
                    is_match = true;
                }
            }
            "id" => {
                if node
                    .get("attrs")
                    .and_then(|a| a.get("localId"))
                    .and_then(|id| id.as_str())
                    == Some(parts[1])
                {
                    is_match = true;
                }
            }
            _ => return Err(format!("Unsupported anchor type: {}", anchor_type)),
        }

        if is_match {
            matches.push(index);
        }
    }

    match matches.len() {
        0 => Err(format!("Anchor not found: {}", selector)),
        1 => Ok(matches[0]),
        n => Err(format!(
            "Ambiguous anchor: found {} matches for '{}'",
            n, selector
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_from_markdown_simple() {
        let md = "# Title\n\nThis is a paragraph with **bold** and [link](https://google.com).";
        let nodes = from_markdown(md);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0]["type"], "heading");
        assert_eq!(nodes[0]["attrs"]["level"], 1);
        assert_eq!(nodes[1]["type"], "paragraph");

        let p_content = nodes[1]["content"].as_array().unwrap();
        assert_eq!(p_content[0]["text"], "This is a paragraph with ");
        assert_eq!(p_content[1]["marks"][0]["type"], "strong");
        assert_eq!(p_content[2]["text"], " and ");
        assert_eq!(p_content[3]["marks"][0]["type"], "link");
    }

    #[test]
    fn test_append_prepend() {
        let mut doc = json!({
            "type": "doc",
            "version": 1,
            "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Middle"}]}]
        });

        let new_nodes =
            vec![json!({"type": "paragraph", "content": [{"type": "text", "text": "New"}]})];

        append_nodes(&mut doc, new_nodes.clone());
        assert_eq!(doc["content"].as_array().unwrap().len(), 2);
        assert_eq!(doc["content"][1]["content"][0]["text"], "New");

        prepend_nodes(&mut doc, new_nodes);
        assert_eq!(doc["content"].as_array().unwrap().len(), 3);
        assert_eq!(doc["content"][0]["content"][0]["text"], "New");
    }

    #[test]
    fn test_replace_text() {
        let mut doc = json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "Hello world"}]
                }
            ]
        });

        replace_text(&mut doc, "world", "Atlassian");
        assert_eq!(doc["content"][0]["content"][0]["text"], "Hello Atlassian");
    }

    #[test]
    fn test_find_anchor_index() {
        let doc = json!({
            "type": "doc",
            "content": [
                { "type": "heading", "attrs": { "level": 1 }, "content": [{ "type": "text", "text": "Introduction" }] },
                { "type": "panel", "attrs": { "panelType": "info" }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Some info" }] }] },
                { "type": "bulletList", "content": [
                    { "type": "listItem", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Item one" }] }] }
                ]},
                { "type": "heading", "attrs": { "level": 2, "localId": "uuid-123" }, "content": [{ "type": "text", "text": "Deep Dive" }] }
            ]
        });

        assert_eq!(find_anchor_index(&doc, "heading:intro").unwrap(), 0);
        assert_eq!(find_anchor_index(&doc, "panel:info").unwrap(), 1);
        assert_eq!(find_anchor_index(&doc, "list:one").unwrap(), 2);
        assert_eq!(find_anchor_index(&doc, "id:uuid-123").unwrap(), 3);

        assert!(find_anchor_index(&doc, "heading:notfound").is_err());
        assert!(find_anchor_index(&doc, "heading:").is_err()); // Ambiguous if query is empty and multiple headings exist
    }
}
