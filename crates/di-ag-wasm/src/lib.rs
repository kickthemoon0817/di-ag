use wasm_bindgen::prelude::*;

/// Parse DSL input and return the IR as JSON
#[wasm_bindgen]
pub fn parse_dsl(input: &str) -> Result<String, JsValue> {
    let doc = di_ag_dsl::parse(input)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Parse DSL, run layout, and return positioned IR as JSON
#[wasm_bindgen]
pub fn layout_dsl(input: &str) -> Result<String, JsValue> {
    let doc = di_ag_dsl::parse(input)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let doc = di_ag_layout::layout(doc)
        .map_err(|e| JsValue::from_str(&format!("Layout error: {}", e)))?;
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Parse IR from JSON, run layout, and return positioned IR as JSON
#[wasm_bindgen]
pub fn layout_json(json: &str) -> Result<String, JsValue> {
    let doc: di_ag_ir::Document = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;
    let doc = di_ag_layout::layout(doc)
        .map_err(|e| JsValue::from_str(&format!("Layout error: {}", e)))?;
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Parse DSL, layout, and render to SVG string
#[wasm_bindgen]
pub fn render_svg(input: &str) -> Result<String, JsValue> {
    let doc = di_ag_dsl::parse(input)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let doc = di_ag_layout::layout(doc)
        .map_err(|e| JsValue::from_str(&format!("Layout error: {}", e)))?;
    di_ag_render::render_svg(&doc)
        .map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))
}

/// Render SVG with a specific theme
#[wasm_bindgen]
pub fn render_svg_themed(input: &str, theme: &str) -> Result<String, JsValue> {
    let doc = di_ag_dsl::parse(input)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let doc = di_ag_layout::layout(doc)
        .map_err(|e| JsValue::from_str(&format!("Layout error: {}", e)))?;
    di_ag_render::render_svg_with_theme(&doc, Some(theme))
        .map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))
}

/// Render from IR JSON to SVG
#[wasm_bindgen]
pub fn render_json_to_svg(json: &str) -> Result<String, JsValue> {
    let doc: di_ag_ir::Document = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;
    let doc = di_ag_layout::layout(doc)
        .map_err(|e| JsValue::from_str(&format!("Layout error: {}", e)))?;
    di_ag_render::render_svg(&doc)
        .map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))
}

/// Validate DSL input, return JSON validation report
#[wasm_bindgen]
pub fn validate_dsl(input: &str) -> Result<String, JsValue> {
    let doc = di_ag_dsl::parse(input)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let report = di_ag_validate::validate(&doc);
    serde_json::to_string(&report)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Inspect a laid-out diagram from DSL, return JSON inspection report
#[wasm_bindgen]
pub fn inspect_dsl(input: &str) -> Result<String, JsValue> {
    let doc = di_ag_dsl::parse(input)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let doc = di_ag_layout::layout(doc)
        .map_err(|e| JsValue::from_str(&format!("Layout error: {}", e)))?;
    let report = di_ag_inspect::inspect(&doc);
    serde_json::to_string(&report)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Full pipeline: parse DSL, layout, render SVG, inspect — return JSON with svg + inspection
#[wasm_bindgen]
pub fn full_pipeline(input: &str) -> Result<String, JsValue> {
    let doc = di_ag_dsl::parse(input)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let doc = di_ag_layout::layout(doc)
        .map_err(|e| JsValue::from_str(&format!("Layout error: {}", e)))?;
    let svg = di_ag_render::render_svg(&doc)
        .map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))?;
    let inspection = di_ag_inspect::inspect(&doc);
    let result = serde_json::json!({
        "svg": svg,
        "inspection": inspection,
    });
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}
