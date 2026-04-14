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
        "dsl" | "diag" => super::dsl_emit::emit(&doc),
        "svg" => {
            let doc =
                di_ag_layout::layout(doc).map_err(|e| format!("Layout error: {}", e))?;
            di_ag_render::render_svg(&doc).map_err(|e| format!("Render error: {}", e))?
        }
        "png" => {
            let doc_laid =
                di_ag_layout::layout(doc).map_err(|e| format!("Layout error: {}", e))?;
            // If the original source was DSL, embed it so the .png round-trips.
            let png_data = if fmt == "dsl" {
                di_ag_render::render_png_with_source(
                    &doc_laid,
                    &di_ag_render::PngOptions::default(),
                    None,
                    &content,
                )
                .map_err(|e| format!("PNG render error: {}", e))?
            } else {
                di_ag_render::render_png(&doc_laid, &di_ag_render::PngOptions::default())
                    .map_err(|e| format!("PNG render error: {}", e))?
            };
            let path = output.ok_or("PNG output requires --output flag")?;
            std::fs::write(path, &png_data)
                .map_err(|e| format!("Failed to write '{}': {}", path, e))?;
            eprintln!("Wrote {}", path);
            return Ok(());
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
