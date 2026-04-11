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

pub fn get_theme(name: &str) -> Theme {
    match name {
        "dark" => dark_theme(),
        "blueprint" => blueprint_theme(),
        "monochrome" => monochrome_theme(),
        _ => light_theme(),
    }
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

pub fn dark_theme() -> Theme {
    Theme {
        background: "#1e1e2e",
        node_fill: "#313244",
        node_stroke: "#89b4fa",
        node_stroke_width: 1.5,
        node_font_family: "Arial, Helvetica, sans-serif",
        node_font_size: 14.0,
        node_font_color: "#cdd6f4",
        edge_stroke: "#6c7086",
        edge_stroke_width: 1.5,
        edge_font_size: 12.0,
    }
}

pub fn blueprint_theme() -> Theme {
    Theme {
        background: "#1a3a5c",
        node_fill: "#1a3a5c",
        node_stroke: "#ffffff",
        node_stroke_width: 1.0,
        node_font_family: "Courier New, monospace",
        node_font_size: 13.0,
        node_font_color: "#ffffff",
        edge_stroke: "#7eb8da",
        edge_stroke_width: 1.0,
        edge_font_size: 11.0,
    }
}

pub fn monochrome_theme() -> Theme {
    Theme {
        background: "#ffffff",
        node_fill: "#ffffff",
        node_stroke: "#000000",
        node_stroke_width: 2.0,
        node_font_family: "Georgia, serif",
        node_font_size: 14.0,
        node_font_color: "#000000",
        edge_stroke: "#000000",
        edge_stroke_width: 1.5,
        edge_font_size: 12.0,
    }
}
