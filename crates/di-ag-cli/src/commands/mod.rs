pub mod convert;
pub mod dsl_emit;
pub mod extract_cmd;
pub mod fmt;
pub mod init;
pub mod render;
pub mod serve;
pub mod validate;

use std::io::Read;

/// Read text input (DSL/JSON/YAML) from a path or from stdin ("-").
///
/// For `.diag.png` (or any PNG that carries an embedded di-ag source chunk)
/// this extracts the embedded DSL and returns that — letting any command that
/// currently operates on a DSL path also operate on a shared `.diag.png` file.
pub fn read_input(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("Failed to read stdin: {}", e))?;
        return Ok(buf);
    }

    if path.ends_with(".png") {
        let bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read '{}': {}", path, e))?;
        return di_ag_render::extract_png_source(&bytes).ok_or_else(|| {
            format!(
                "'{}' does not contain an embedded di-ag source chunk",
                path
            )
        });
    }

    std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", path, e))
}

pub fn detect_format(path: &str) -> &str {
    if path.ends_with(".json") {
        "json"
    } else if path.ends_with(".yaml") || path.ends_with(".yml") {
        "yaml"
    } else if path.ends_with(".png") {
        // Embedded PNG source round-trips as DSL.
        "dsl"
    } else {
        "dsl"
    }
}

pub fn parse_input(content: &str, format: &str) -> Result<di_ag_ir::Document, String> {
    match format {
        "json" => {
            serde_json::from_str(content).map_err(|e| format!("JSON parse error: {}", e))
        }
        "yaml" => {
            serde_yaml::from_str(content).map_err(|e| format!("YAML parse error: {}", e))
        }
        _ => di_ag_dsl::parse(content).map_err(|e| format!("DSL parse error: {}", e)),
    }
}
