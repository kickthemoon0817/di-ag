pub mod png;
pub mod shapes;
pub mod svg;
pub mod theme;

#[cfg(test)]
mod tests;

use di_ag_ir::Document;
use thiserror::Error;

pub use png::PngOptions;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Render failed: {0}")]
    Failed(String),
}

pub fn render_svg(doc: &Document) -> Result<String, RenderError> {
    Ok(svg::build_svg(doc))
}

pub fn render_svg_with_theme(doc: &Document, theme: Option<&str>) -> Result<String, RenderError> {
    Ok(svg::build_svg_with_theme(doc, theme))
}

pub fn render_png(doc: &Document, options: &PngOptions) -> Result<Vec<u8>, RenderError> {
    let svg_str = render_svg(doc)?;
    png::svg_to_png(&svg_str, options)
}

pub fn render_png_with_theme(
    doc: &Document,
    options: &PngOptions,
    theme: Option<&str>,
) -> Result<Vec<u8>, RenderError> {
    let svg_str = render_svg_with_theme(doc, theme)?;
    png::svg_to_png(&svg_str, options)
}
