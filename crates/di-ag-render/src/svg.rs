use di_ag_ir::*;

use crate::shapes::{render_shape, ShapeAttrs};
use crate::theme::{get_theme, Theme};

pub fn build_svg(doc: &Document) -> String {
    build_svg_with_theme(doc, None)
}

pub fn build_svg_with_theme(doc: &Document, theme_name: Option<&str>) -> String {
    let preset_theme = doc
        .preset
        .as_ref()
        .and_then(|p| p.theme.as_deref());
    let theme = get_theme(theme_name.or(preset_theme).unwrap_or("light"));
    let (min_x, min_y, max_x, max_y) = compute_viewbox(doc);
    let padding = 40.0;
    let width = max_x - min_x + padding * 2.0;
    let height = max_y - min_y + padding * 2.0;
    let vx = min_x - padding;
    let vy = min_y - padding;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{}" height="{}">"#,
        vx, vy, width, height, width, height
    );
    svg.push('\n');

    // Background
    svg.push_str(&format!(
        r#"  <rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
        vx, vy, width, height, theme.background
    ));
    svg.push('\n');

    // Render edges first (behind nodes)
    for edge in &doc.edges {
        svg.push_str(&render_edge(edge, &theme));
    }

    // Render nodes
    for node in &doc.nodes {
        svg.push_str(&render_node(node, &theme));
    }

    svg.push_str("</svg>");
    svg
}

fn render_node(node: &Node, theme: &Theme) -> String {
    let pos = match &node.position {
        Some(p) => p,
        None => return String::new(),
    };
    let size = match &node.size {
        Some(s) => s,
        None => return String::new(),
    };

    let mut result = format!(r#"  <g data-id="{}">"#, node.id);
    result.push('\n');

    let attrs = ShapeAttrs {
        fill: node
            .style
            .fill
            .clone()
            .unwrap_or_else(|| theme.node_fill.into()),
        stroke: node
            .style
            .stroke
            .clone()
            .unwrap_or_else(|| theme.node_stroke.into()),
        stroke_width: node.style.stroke_width.unwrap_or(theme.node_stroke_width),
    };

    result.push_str("    ");
    result.push_str(&render_shape(&node.shape, pos, size, &attrs));
    result.push('\n');

    // Label
    let font_size = node.style.font_size.unwrap_or(theme.node_font_size);
    let font_color = node
        .style
        .font_color
        .clone()
        .unwrap_or_else(|| theme.node_font_color.into());
    let font_family = node
        .style
        .font_family
        .clone()
        .unwrap_or_else(|| theme.node_font_family.into());
    let is_container = !node.children.is_empty();
    let cx = pos.x + size.width / 2.0;
    let cy = if is_container {
        // Container: anchor label near the top with a small inset.
        pos.y + font_size + 6.0
    } else {
        pos.y + size.height / 2.0 + font_size * 0.35
    };

    result.push_str(&format!(
        r#"    <text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
        cx,
        cy,
        font_family,
        font_size,
        font_color,
        escape_xml_label(&node.label)
    ));
    result.push('\n');

    // Render children if container (children are already in absolute coordinates)
    for child in &node.children {
        result.push_str(&render_node(child, theme));
    }

    result.push_str("  </g>\n");
    result
}

fn render_edge(edge: &Edge, theme: &Theme) -> String {
    if edge.waypoints.len() < 2 {
        return String::new();
    }

    let stroke = edge
        .style
        .color
        .clone()
        .or_else(|| edge.style.stroke.clone())
        .unwrap_or_else(|| theme.edge_stroke.into());
    let stroke_width = edge.style.stroke_width.unwrap_or(theme.edge_stroke_width);

    let mut result = format!(r#"  <g data-id="{}">"#, edge.id);
    result.push('\n');

    // Path
    let mut d = format!("M {} {}", edge.waypoints[0].x, edge.waypoints[0].y);
    for wp in &edge.waypoints[1..] {
        d.push_str(&format!(" L {} {}", wp.x, wp.y));
    }

    let dash = edge
        .style
        .dash
        .as_deref()
        .map(|d| match d {
            "dashed" => r#" stroke-dasharray="8,4""#,
            "dotted" => r#" stroke-dasharray="2,4""#,
            _ => "",
        })
        .unwrap_or("");

    result.push_str(&format!(
        r#"    <path d="{}" fill="none" stroke="{}" stroke-width="{}"{}/>"#,
        d, stroke, stroke_width, dash
    ));
    result.push('\n');

    // Arrowhead
    let last = &edge.waypoints[edge.waypoints.len() - 1];
    let prev = &edge.waypoints[edge.waypoints.len() - 2];
    result.push_str(&render_arrowhead(prev.x, prev.y, last.x, last.y, &stroke));

    // Label
    if let Some(ref label) = edge.label {
        let wps = &edge.waypoints;
        let (mx, my) = if wps.len() >= 2 {
            let mid_idx = wps.len() / 2;
            let a = &wps[mid_idx - 1];
            let b = &wps[mid_idx];
            ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
        } else {
            (wps[0].x, wps[0].y)
        };
        result.push_str(&format!(
            r#"    <text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
            mx,
            my - 6.0,
            theme.node_font_family,
            theme.edge_font_size,
            theme.node_font_color,
            escape_xml_label(label)
        ));
        result.push('\n');
    }

    result.push_str("  </g>\n");
    result
}

fn render_arrowhead(from_x: f64, from_y: f64, to_x: f64, to_y: f64, color: &str) -> String {
    let angle = (to_y - from_y).atan2(to_x - from_x);
    let arrow_len = 10.0;
    let arrow_angle = 0.4;

    let x1 = to_x - arrow_len * (angle - arrow_angle).cos();
    let y1 = to_y - arrow_len * (angle - arrow_angle).sin();
    let x2 = to_x - arrow_len * (angle + arrow_angle).cos();
    let y2 = to_y - arrow_len * (angle + arrow_angle).sin();

    format!(
        "    <polygon points=\"{},{} {},{} {},{}\" fill=\"{}\"/>\n",
        to_x, to_y, x1, y1, x2, y2, color
    )
}

fn compute_viewbox(doc: &Document) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    fn visit_nodes(
        nodes: &[Node],
        min_x: &mut f64,
        min_y: &mut f64,
        max_x: &mut f64,
        max_y: &mut f64,
    ) {
        for node in nodes {
            if let (Some(pos), Some(size)) = (&node.position, &node.size) {
                *min_x = min_x.min(pos.x);
                *min_y = min_y.min(pos.y);
                *max_x = max_x.max(pos.x + size.width);
                *max_y = max_y.max(pos.y + size.height);
            }
            visit_nodes(&node.children, min_x, min_y, max_x, max_y);
        }
    }

    visit_nodes(&doc.nodes, &mut min_x, &mut min_y, &mut max_x, &mut max_y);

    // Include edge waypoints so labels and long routes aren't clipped.
    for edge in &doc.edges {
        for wp in &edge.waypoints {
            min_x = min_x.min(wp.x);
            min_y = min_y.min(wp.y);
            max_x = max_x.max(wp.x);
            max_y = max_y.max(wp.y);
        }
    }

    if min_x == f64::MAX {
        (0.0, 0.0, 200.0, 200.0)
    } else {
        (min_x, min_y, max_x, max_y)
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Escape a label for SVG text content. Renders literal newlines as multiple
/// <tspan> lines so DSL `\n` escape sequences produce multi-line labels.
fn escape_xml_label(s: &str) -> String {
    if !s.contains('\n') {
        return escape_xml(s);
    }
    // Multi-line: wrap lines in tspans. This gets embedded in the <text>
    // element already, so we produce only the inner content.
    let lines: Vec<&str> = s.split('\n').collect();
    let n = lines.len();
    let start = -(((n as f64) - 1.0) * 0.6);
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        let dy = if i == 0 {
            format!("{}em", start)
        } else {
            "1.2em".to_string()
        };
        out.push_str(&format!(
            "<tspan x=\"0\" dy=\"{}\">{}</tspan>",
            dy,
            escape_xml(line)
        ));
    }
    out
}
