pub mod convert;
pub mod fmt;
pub mod init;
pub mod render;
pub mod validate;

use std::io::Read;

pub fn read_input(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("Failed to read stdin: {}", e))?;
        Ok(buf)
    } else {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", path, e))
    }
}

pub fn detect_format(path: &str) -> &str {
    if path.ends_with(".json") {
        "json"
    } else if path.ends_with(".yaml") || path.ends_with(".yml") {
        "yaml"
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
