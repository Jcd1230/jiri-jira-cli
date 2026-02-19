use crate::client::JiraClient;

pub async fn run(client: &JiraClient, key: String, status: Option<String>) -> Result<(), String> {
    match status {
        None => list_transitions(client, &key).await,
        Some(target) => do_transition(client, &key, &target).await,
    }
}

async fn list_transitions(client: &JiraClient, key: &str) -> Result<(), String> {
    let data = client.get_transitions(key).await?;
    let transitions = data["transitions"]
        .as_array()
        .ok_or("No transitions found")?;

    println!("Available transitions for {}:", key);
    for t in transitions {
        let id = t["id"].as_str().unwrap_or("?");
        let name = t["name"].as_str().unwrap_or("?");
        println!("  [{}] {}", id, name);
    }
    Ok(())
}

async fn do_transition(client: &JiraClient, key: &str, target: &str) -> Result<(), String> {
    let data = client.get_transitions(key).await?;
    let transitions = data["transitions"]
        .as_array()
        .ok_or("No transitions found")?;

    // Find matching transition (case-insensitive, prefix match)
    let target_lower = target.to_lowercase();
    let matched = transitions.iter().find(|t| {
        let name = t["name"].as_str().unwrap_or("").to_lowercase();
        name == target_lower || name.starts_with(&target_lower)
    });

    let transition = matched.ok_or_else(|| {
        let available: Vec<String> = transitions
            .iter()
            .filter_map(|t| t["name"].as_str().map(|s| s.to_string()))
            .collect();
        format!(
            "No transition matching '{}'. Available: {}",
            target,
            available.join(", ")
        )
    })?;

    let id = transition["id"].as_str().unwrap_or("?");
    let name = transition["name"].as_str().unwrap_or("?");

    client.do_transition(key, id).await?;
    println!("Transitioned {} → {}", key, name);
    Ok(())
}
