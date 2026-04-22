use di_ag_render::ICON_NAMES;

/// Descriptions for each built-in icon, parallel to ICON_NAMES order.
fn description(name: &str) -> &'static str {
    match name {
        "user" => "Person / actor / human user",
        "database" => "SQL or NoSQL database",
        "server" => "Physical or virtual server / rack",
        "cloud" => "Cloud service / SaaS",
        "api" => "API endpoint / service interface",
        "web" => "Web browser / web app",
        "mobile" => "Mobile phone / native app",
        "cache" => "In-memory cache (Redis, Memcached)",
        "queue" => "Message queue / job queue",
        "auth" => "Authentication service",
        "lock" => "Security / access control (alias of auth)",
        "storage" => "Block storage / object storage",
        "gear" => "Configuration / settings / service",
        "settings" => "Configuration (alias of gear)",
        "file" => "Document / file",
        "chart" => "Analytics / reporting",
        "mail" => "Email / notification service",
        _ => "",
    }
}

pub fn run(json: bool) -> Result<(), String> {
    let mut names: Vec<&str> = ICON_NAMES.to_vec();
    names.sort_unstable();

    if json {
        let entries: Vec<serde_json::Value> = names
            .iter()
            .map(|&name| {
                serde_json::json!({
                    "name": name,
                    "description": description(name),
                })
            })
            .collect();
        let out = serde_json::to_string_pretty(&entries)
            .map_err(|e| format!("JSON error: {}", e))?;
        println!("{}", out);
    } else {
        for name in names {
            println!("{}", name);
        }
    }

    Ok(())
}
