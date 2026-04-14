use super::{detect_format, parse_input, read_input};
use di_ag_render::PngOptions;

pub fn run(
    input: &str,
    output: Option<&str>,
    format: &str,
    theme: Option<&str>,
    layout_override: Option<&str>,
    inspect: bool,
    score_threshold: Option<f64>,
    json: bool,
) -> Result<(), String> {
    let content = read_input(input)?;
    let fmt = if input == "-" {
        "dsl"
    } else {
        detect_format(input)
    };
    let mut doc = parse_input(&content, fmt)?;

    // --layout overrides the preset diagram type so users can force a specific
    // layout strategy regardless of the file's `@preset`.
    if let Some(lo) = layout_override {
        use di_ag_ir::{DiagramType, Preset};
        let dt = match lo {
            "layered" | "flowchart" => DiagramType::Flowchart,
            "force" | "freeform" => DiagramType::Freeform,
            "orthogonal" | "er" => DiagramType::Er,
            "tree" => DiagramType::Tree,
            "sequence" => DiagramType::Sequence,
            "class" => DiagramType::Class,
            other => return Err(format!("Unknown layout: {}", other)),
        };
        let preset = doc.preset.take().map(|mut p| {
            p.diagram_type = dt.clone();
            p
        });
        doc.preset = Some(preset.unwrap_or(Preset {
            diagram_type: dt,
            theme: None,
            layout: None,
        }));
    }

    let doc = di_ag_layout::layout(doc).map_err(|e| format!("Layout error: {}", e))?;

    let svg = di_ag_render::render_svg_with_theme(&doc, theme)
        .map_err(|e| format!("Render error: {}", e))?;

    // When the source is DSL, embed it into PNG output so the file becomes a
    // shareable round-trippable artifact (like `.drawio.png`).
    let embed_source = if fmt == "dsl" { Some(content.as_str()) } else { None };

    if inspect || json {
        let report = di_ag_inspect::inspect(&doc);

        if let Some(threshold) = score_threshold {
            if report.score < threshold {
                let report_json = serde_json::to_string_pretty(&report).unwrap();
                eprintln!(
                    "Score {:.1} is below threshold {:.1}",
                    report.score, threshold
                );
                eprintln!("{}", report_json);
                std::process::exit(2);
            }
        }

        if json {
            let output_json = serde_json::json!({
                "svg": svg,
                "inspection": report,
            });
            println!("{}", serde_json::to_string_pretty(&output_json).unwrap());
        } else if let Some(path) = output {
            write_output(path, format, &svg, &doc, theme, embed_source)?;
            let report_json = serde_json::to_string_pretty(&report).unwrap();
            println!("{}", report_json);
        } else {
            let report_json = serde_json::to_string_pretty(&report).unwrap();
            eprintln!("{}", report_json);
            print!("{}", svg);
        }
    } else {
        match output {
            Some(path) => {
                write_output(path, format, &svg, &doc, theme, embed_source)?;
                eprintln!("Wrote {}", path);
            }
            None => print!("{}", svg),
        }
    }

    Ok(())
}

fn write_output(
    path: &str,
    format: &str,
    svg: &str,
    doc: &di_ag_ir::Document,
    theme: Option<&str>,
    embed_source: Option<&str>,
) -> Result<(), String> {
    let actual_format = if format == "svg" && path.ends_with(".png") {
        "png"
    } else {
        format
    };

    match actual_format {
        "png" => {
            let png_data = if let Some(src) = embed_source {
                di_ag_render::render_png_with_source(doc, &PngOptions::default(), theme, src)
                    .map_err(|e| format!("PNG render error: {}", e))?
            } else {
                di_ag_render::render_png_with_theme(doc, &PngOptions::default(), theme)
                    .map_err(|e| format!("PNG render error: {}", e))?
            };
            std::fs::write(path, &png_data)
                .map_err(|e| format!("Failed to write '{}': {}", path, e))?;
        }
        _ => {
            std::fs::write(path, svg)
                .map_err(|e| format!("Failed to write '{}': {}", path, e))?;
        }
    }
    Ok(())
}
