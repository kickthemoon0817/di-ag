use di_ag_render::{icon_description, ICON_NAMES};

pub fn run(json: bool) -> Result<(), String> {
    let mut names: Vec<&str> = ICON_NAMES.to_vec();
    names.sort_unstable();

    if json {
        let entries: Vec<serde_json::Value> = names
            .iter()
            .map(|&name| {
                serde_json::json!({
                    "name": name,
                    "description": icon_description(name).unwrap_or(""),
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
