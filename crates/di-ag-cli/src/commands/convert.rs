use super::{detect_format, parse_input, read_input};

pub fn run(
    input: &str,
    to: &str,
    from: Option<&str>,
    output: Option<&str>,
) -> Result<(), String> {
    let content = read_input(input)?;
    let fmt = from.unwrap_or_else(|| {
        if input == "-" {
            "dsl"
        } else {
            detect_format(input)
        }
    });
    let doc = parse_input(&content, fmt)?;

    let result = match to {
        "json" => serde_json::to_string_pretty(&doc)
            .map_err(|e| format!("JSON serialization error: {}", e))?,
        "yaml" => {
            serde_yaml::to_string(&doc).map_err(|e| format!("YAML serialization error: {}", e))?
        }
        "svg" => {
            let doc =
                di_ag_layout::layout(doc).map_err(|e| format!("Layout error: {}", e))?;
            di_ag_render::render_svg(&doc).map_err(|e| format!("Render error: {}", e))?
        }
        other => return Err(format!("Unsupported target format: {}", other)),
    };

    match output {
        Some(path) => {
            std::fs::write(path, &result)
                .map_err(|e| format!("Failed to write '{}': {}", path, e))?;
            eprintln!("Wrote {}", path);
        }
        None => print!("{}", result),
    }

    Ok(())
}
