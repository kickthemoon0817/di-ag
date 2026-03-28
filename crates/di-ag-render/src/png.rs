use crate::RenderError;

pub struct PngOptions {
    pub dpi_scale: f64,
    pub transparent: bool,
}

impl Default for PngOptions {
    fn default() -> Self {
        PngOptions {
            dpi_scale: 2.0,
            transparent: false,
        }
    }
}

pub fn svg_to_png(svg_str: &str, options: &PngOptions) -> Result<Vec<u8>, RenderError> {
    let tree = resvg::usvg::Tree::from_str(svg_str, &resvg::usvg::Options::default())
        .map_err(|e| RenderError::Failed(format!("SVG parse error: {}", e)))?;

    let size = tree.size();
    let scale = options.dpi_scale as f32;
    let width = (size.width() * scale) as u32;
    let height = (size.height() * scale) as u32;

    if width == 0 || height == 0 {
        return Err(RenderError::Failed("SVG has zero dimensions".into()));
    }

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| RenderError::Failed("Failed to create pixmap".into()))?;

    if !options.transparent {
        pixmap.fill(tiny_skia::Color::WHITE);
    }

    let transform =
        tiny_skia::Transform::from_scale(options.dpi_scale as f32, options.dpi_scale as f32);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| RenderError::Failed(format!("PNG encode error: {}", e)))
}
