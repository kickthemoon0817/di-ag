use super::{detect_format, parse_input, read_input};
use di_ag_render::PngOptions;

pub fn run(
    input: &str,
    output: Option<&str>,
    format: &str,
    theme: Option<&str>,
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
    let doc = parse_input(&content, fmt)?;

    let doc = di_ag_layout::layout(doc).map_err(|e| format!("Layout error: {}", e))?;

    let svg = di_ag_render::render_svg_with_theme(&doc, theme)
        .map_err(|e| format!("Render error: {}", e))?;

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
            write_output(path, format, &svg, &doc, theme)?;
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
                write_output(path, format, &svg, &doc, theme)?;
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
) -> Result<(), String> {
    let actual_format = if format == "svg" && path.ends_with(".png") {
        "png"
    } else {
        format
    };

    match actual_format {
        "png" => {
            let png_data =
                di_ag_render::render_png_with_theme(doc, &PngOptions::default(), theme)
                    .map_err(|e| format!("PNG render error: {}", e))?;
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
