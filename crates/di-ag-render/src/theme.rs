pub struct Theme {
    pub background: &'static str,
    pub node_fill: &'static str,
    pub node_stroke: &'static str,
    pub node_stroke_width: f64,
    pub node_font_family: &'static str,
    pub node_font_size: f64,
    pub node_font_color: &'static str,
    pub edge_stroke: &'static str,
    pub edge_stroke_width: f64,
    pub edge_font_size: f64,
}

pub fn light_theme() -> Theme {
    Theme {
        background: "#ffffff",
        node_fill: "#f5f5f5",
        node_stroke: "#333333",
        node_stroke_width: 1.5,
        node_font_family: "Arial, Helvetica, sans-serif",
        node_font_size: 14.0,
        node_font_color: "#333333",
        edge_stroke: "#666666",
        edge_stroke_width: 1.5,
        edge_font_size: 12.0,
    }
}
