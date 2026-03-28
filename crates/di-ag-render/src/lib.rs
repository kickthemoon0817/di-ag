pub mod shapes;
pub mod svg;
pub mod theme;

#[cfg(test)]
mod tests;

use di_ag_ir::Document;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Render failed: {0}")]
    Failed(String),
}

pub fn render_svg(doc: &Document) -> Result<String, RenderError> {
    Ok(svg::build_svg(doc))
}
