use crate::client::JiraClient;
use crate::formatter::Formatter;
use serde_json::Value;

pub async fn run(
    client: &JiraClient,
    formatter: &Formatter,
    jql: String,
    fields: Option<String>,
    get_fields: bool,
    limit: i64,
) -> Result<(), String> {
    let lookup = client.field_lookup().await?;
    
    let requested_fields = fields
        .unwrap_or_else(|| "key,summary".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();

    let resolved = resolve_fields(&requested_fields, &lookup);

    if get_fields {
        let data = client.search(&jql, vec!["*all".to_string()], 1, None).await?;
        let issues = data["issues"].as_array().ok_or("No issues found")?;
        if let Some(issue) = issues.first() {
            let mut field_names: Vec<String> = issue["fields"].as_object().unwrap().keys().cloned().collect();
            field_names.sort();
            
            let mut rows = vec![vec!["FIELD".to_string()]];
            for f in field_names {
                let friendly = lookup.id_to_name.get(&f);
                let row = match friendly {
                    Some(name) => format!("\"{}\" ({})", name, f),
                    None => f,
                };
                rows.push(vec![row]);
            }
            println!("{}", formatter.render(rows));
        }
        return Ok(());
    }

    let (issues, more_available) = client.search_all(&jql, resolved.query_fields, limit).await?;

    let mut rows = vec![resolved.headers];
    for issue in issues {
        let mut row = Vec::new();
        for key in &resolved.keys {
            let val = format_field_value(&issue, key);
            row.push(val);
        }
        rows.push(row);
    }

    println!("{}", formatter.render(rows));

    if more_available && (issues.len() as i64) >= limit {
        eprintln!(
            "Warning: displayed {} issues (limit {}). More results are available; rerun with a higher --limit to see more.",
            issues.len(),
            limit
        );
    }

    Ok(())
}

struct ResolvedFields {
    query_fields: Vec<String>,
    headers: Vec<String>,
    keys: Vec<String>,
}

fn resolve_fields(requested: &Vec<String>, lookup: &crate::client::FieldLookup) -> ResolvedFields {
    let mut query_fields = Vec::new();
    let mut headers = Vec::new();
    let mut keys = Vec::new();

    for name in requested {
        let lower = name.toLowerCase();
        
        if lookup.id_to_name.contains_key(name) {
            headers.push(lookup.id_to_name.get(name).cloned().unwrap_or_else(|| name.clone()).to_uppercase());
            query_fields.push(name.clone());
            keys.push(name.clone());
            continue;
        }

        if let Some(id) = lookup.name_to_id.get(&lower) {
            headers.push(name.to_uppercase());
            query_fields.push(id.clone());
            keys.push(id.clone());
            continue;
        }

        // Default or unknown
        headers.push(name.to_uppercase());
        query_fields.push(name.clone());
        keys.push(name.clone());
    }

    if query_fields.is_empty() {
        query_fields.push("key".to_string());
        query_fields.push("summary".to_string());
        headers.push("KEY".to_string());
        headers.push("SUMMARY".to_string());
        keys.push("key".to_string());
        keys.push("summary".to_string());
    }

    ResolvedFields { query_fields, headers, keys }
}

fn format_field_value(issue: &Value, key: &str) -> String {
    // Special case for top-level fields like key
    if key == "key" {
        return issue["key"].as_str().unwrap_or_default().to_string();
    }

    let fields = &issue["fields"];
    let val = &fields[key];

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

    if let Some(obj) = val.as_object() {
        if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
            return name.to_string();
        }
        if let Some(value) = obj.get("value").and_then(|v| v.as_str()) {
            return value.to_string();
        }
        if let Some(display_name) = obj.get("displayName").and_then(|v| v.as_str()) {
            return display_name.to_string();
        }
    }

    if let Some(arr) = val.as_array() {
        return arr
            .iter()
            .map(|v| {
                if let Some(s) = v.as_str() {
                    s.to_string()
                } else if let Some(obj) = v.as_object() {
                    obj.get("name")
                        .and_then(|v| v.as_str())
                        .or_else(|| obj.get("value").and_then(|v| v.as_str()))
                        .unwrap_or_default()
                        .to_string()
                } else {
                    v.to_string()
                }
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
    }

    val.to_string()
}
