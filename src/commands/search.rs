use crate::client::AtlassianClient;
use crate::fields;
use crate::formatter::{Formatter, OutputFormat};
use owo_colors::OwoColorize;
use serde_json::Value;

/// Execute the search command.
pub async fn run(
    client: &AtlassianClient,
    formatter: &Formatter,
    jql: String,
    fields: Option<String>,
    get_fields: bool,
    limit: i64,
    all_projects: bool,
) -> Result<(), String> {
    let original_jql = jql.clone();
    let mut final_jql = jql;

    // If not searching all projects and a default project exists, prepend it.
    if !all_projects {
        if let Some(default_project) = &client.config().default_project {
            // Simple check if project context is already provided.
            if !query_mentions_project(&final_jql) {
                let (filter, order_by) = split_order_by_clause(&final_jql);
                final_jql = if filter.is_empty() {
                    format!("project = \"{}\"{}", default_project, order_by)
                } else {
                    format!(
                        "project = \"{}\" AND ({}){}",
                        default_project, filter, order_by
                    )
                };
            }
        }
    }

    let lookup = client.field_lookup().await?;

    let requested_fields = fields
        .unwrap_or_else(|| "key,summary".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();

    let resolved = resolve_fields(&requested_fields, &lookup);

    if get_fields {
        let data = client
            .search(&final_jql, vec!["*all".to_string()], 1, None)
            .await
            .map_err(|err| search_error_with_context(&original_jql, &final_jql, err))?;
        let issues = data["issues"].as_array().ok_or("No issues found")?;
        if let Some(issue) = issues.first() {
            let fields_obj = issue["fields"]
                .as_object()
                .ok_or("Search result did not include a fields object")?;
            let mut field_names: Vec<String> = fields_obj.keys().cloned().collect();
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

    let (issues, more_available) = client
        .search_all(&final_jql, resolved.query_fields, limit)
        .await
        .map_err(|err| search_error_with_context(&original_jql, &final_jql, err))?;

    if matches!(formatter.format, OutputFormat::Json) {
        println!(
            "{}",
            serde_json::to_string_pretty(&Value::Array(issues.clone())).unwrap_or_default()
        );
        return Ok(());
    }

    let mut rows = vec![resolved.headers];
    for issue in &issues {
        let mut row = Vec::new();
        for key in &resolved.keys {
            let val = fields::get_field_value(issue, key);
            row.push(val);
        }
        rows.push(row);
    }

    println!("{}", formatter.render(rows));

    if more_available && (issues.len() as i64) >= limit {
        eprintln!(
            "{} displayed {} issues (limit {}). More results are available; rerun with a higher --limit to see more.",
            "warning:".yellow().bold(),
            issues.len(),
            limit
        );
    }

    Ok(())
}

fn query_mentions_project(jql: &str) -> bool {
    let lower = jql.to_lowercase();
    matches_project_clause(&lower, "project =")
        || matches_project_clause(&lower, "project in")
        || matches_project_clause(&lower, "project not in")
        || matches_project_clause(&lower, "project !=")
        || matches_project_clause(&lower, "project is")
}

fn split_order_by_clause(jql: &str) -> (String, String) {
    let lower = jql.to_lowercase();

    if let Some(idx) = lower.rfind(" order by ") {
        let filter = jql[..idx].trim_end().to_string();
        let order_by = jql[idx..].to_string();
        return (filter, order_by);
    }

    if lower.starts_with("order by ") {
        return (String::new(), jql.to_string());
    }

    (jql.to_string(), String::new())
}

fn matches_project_clause(jql_lower: &str, clause: &str) -> bool {
    jql_lower.contains(clause)
}

fn search_error_with_context(original_jql: &str, final_jql: &str, err: String) -> String {
    eprintln!("{} JQL search failed", "error:".red().bold());
    eprintln!("{} {}", "  input:".cyan(), original_jql);
    eprintln!("{} {}", "  sent:".cyan(), final_jql);
    eprintln!(
        "{} if the query is too complex for Jiri's pre-parser, rerun with {} to skip default project injection",
        "hint:".yellow(),
        "-a".bold()
    );
    err
}

struct ResolvedFields {
    query_fields: Vec<String>,
    headers: Vec<String>,
    keys: Vec<String>,
}

fn resolve_fields(requested: &[String], lookup: &crate::client::FieldLookup) -> ResolvedFields {
    let mut query_fields = Vec::new();
    let mut headers = Vec::new();
    let mut keys = Vec::new();

    for name in requested {
        let lower = name.to_lowercase();

        if lookup.id_to_name.contains_key(name) {
            headers.push(
                lookup
                    .id_to_name
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone())
                    .to_uppercase(),
            );
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

    ResolvedFields {
        query_fields,
        headers,
        keys,
    }
}
