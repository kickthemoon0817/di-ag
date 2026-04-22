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

    let mut result = format!(r#"  <g data-id="{}">"#, escape_xml(&node.id));
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

    // Icon: if present and recognized, render it above the label and shift
    // the label down to the lower half of the node. Icon uses the font color
    // as its stroke so it reads as part of the label.
    let icon_fragment = node
        .icon
        .as_deref()
        .and_then(|name| crate::icons::icon_svg_colored(name, &font_color));

    let cy = if is_container {
        // Container: anchor label near the top with a small inset.
        pos.y + font_size + 6.0
    } else if icon_fragment.is_some() {
        // With icon: label in the lower 2/3 of the node so the icon has
        // room in the upper 1/3.
        pos.y + size.height * 0.68 + font_size * 0.35
    } else {
        pos.y + size.height / 2.0 + font_size * 0.35
    };

    if let Some(fragment) = icon_fragment {
        // Icon occupies a 20x20 box by design. Place its top-left so the
        // icon's center lands on (cx, pos.y + size.height * 0.3).
        let icon_cx = cx;
        let icon_cy = pos.y + size.height * 0.30;
        let icon_tx = icon_cx - 10.0;
        let icon_ty = icon_cy - 10.0;
        result.push_str(&format!(
            "    <g transform=\"translate({} {})\">\n      {}\n    </g>\n",
            icon_tx, icon_ty, fragment
        ));
    }

    result.push_str(&render_text_block(
        cx,
        cy,
        &node.label,
        &font_family,
        font_size,
        &font_color,
    ));

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

    let mut result = format!(r#"  <g data-id="{}">"#, escape_xml(&edge.id));
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
        d,
        escape_xml(&stroke),
        stroke_width,
        dash
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
        result.push_str(&render_text_block(
            mx,
            my - 6.0,
            label,
            theme.node_font_family,
            theme.edge_font_size,
            theme.node_font_color,
        ));
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
        to_x,
        to_y,
        x1,
        y1,
        x2,
        y2,
        escape_xml(color)
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

/// XML-escape a string for safe interpolation into an SVG attribute value or
/// text node. Exposed to sibling modules (shapes.rs) so every user-controlled
/// value is escaped at the emit site.
pub(crate) fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Render a possibly-multi-line label as one or more absolutely-positioned
/// `<text>` elements centered at (cx, cy). Using separate text elements
/// instead of tspans avoids the SVG tspan-x inheritance quirk that would
/// left-align subsequent lines.
fn render_text_block(
    cx: f64,
    cy: f64,
    label: &str,
    font_family: &str,
    font_size: f64,
    font_color: &str,
) -> String {
    let lines: Vec<&str> = label.split('\n').collect();
    let ff = escape_xml(font_family);
    let fc = escape_xml(font_color);
    if lines.len() <= 1 {
        return format!(
            r#"    <text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}">{}</text>
"#,
            cx,
            cy,
            ff,
            font_size,
            fc,
            escape_xml(label)
        );
    }
    // Center the block vertically: shift start by half the block height.
    let line_height = font_size * 1.2;
    let block_height = (lines.len() as f64 - 1.0) * line_height;
    let start_y = cy - block_height / 2.0;
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        let y = start_y + (i as f64) * line_height;
        out.push_str(&format!(
            r#"    <text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="{}">{}</text>
"#,
            cx,
            y,
            ff,
            font_size,
            fc,
            escape_xml(line)
        ));
    }
    out
}
