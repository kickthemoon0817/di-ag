use di_ag_ir::{Document, LayoutDirection, Node, Shape};

/// Emit a `Document` back to di-ag DSL syntax so `convert --to dsl` round-trips.
/// The output is faithful to IR state — layout-computed positions are omitted
/// so re-parsing produces a clean document that can be laid out again.
pub fn emit(doc: &Document) -> String {
    let mut out = String::new();

    if let Some(preset) = &doc.preset {
        out.push_str(&format!("@preset {}\n", diagram_type_str(&preset.diagram_type)));
        if let Some(t) = &preset.theme {
            out.push_str(&format!("@theme {}\n", t));
        }
        if let Some(layout) = &preset.layout {
            out.push_str(&format!(
                "@layout direction={} spacing={}\n",
                direction_str(&layout.direction),
                format_num(layout.spacing)
            ));
        }
        out.push('\n');
    }

    let top_level_edges = &doc.edges;

    for node in &doc.nodes {
        emit_node(node, 0, &mut out, top_level_edges);
    }

    if !doc.edges.is_empty() && doc.nodes.iter().all(|n| n.children.is_empty()) {
        // For flat docs, edges come after nodes.
        out.push('\n');
    }
    for edge in &doc.edges {
        // Skip edges that were emitted inside a container.
        if is_inside_container(edge, &doc.nodes) {
            continue;
        }
        emit_edge(edge, 0, &mut out);
    }

    out
}

fn is_inside_container(edge: &di_ag_ir::Edge, nodes: &[Node]) -> bool {
    for n in nodes {
        if !n.children.is_empty() {
            let child_ids: std::collections::HashSet<String> =
                n.children.iter().map(|c| c.id.clone()).collect();
            if child_ids.contains(&edge.source) && child_ids.contains(&edge.target) {
                return true;
            }
        }
    }
    false
}

fn emit_node(node: &Node, depth: usize, out: &mut String, all_edges: &[di_ag_ir::Edge]) {
    let indent = "    ".repeat(depth);
    let is_container = !node.children.is_empty();
    let keyword = if is_container { "container" } else { "node" };
    out.push_str(&format!(
        "{}{} {} \"{}\"",
        indent,
        keyword,
        node.id,
        escape_dsl_string(&node.label)
    ));

    let has_props = !is_container
        && (!matches!(node.shape, Shape::Rect)
            || node.size.is_some()
            || node.position.is_some()
            || node.icon.is_some()
            || has_any_style(&node.style));

    if is_container {
        out.push_str(" {\n");
        for child in &node.children {
            emit_node(child, depth + 1, out, all_edges);
        }
        let child_ids: std::collections::HashSet<String> =
            node.children.iter().map(|c| c.id.clone()).collect();
        for edge in all_edges {
            if child_ids.contains(&edge.source) && child_ids.contains(&edge.target) {
                emit_edge(edge, depth + 1, out);
            }
        }
        out.push_str(&format!("{}}}\n", indent));
    } else if has_props {
        out.push_str(" {\n");
        if !matches!(node.shape, Shape::Rect) {
            out.push_str(&format!("{}    shape: {}\n", indent, shape_str(&node.shape)));
        }
        if let Some(icon) = &node.icon {
            out.push_str(&format!("{}    icon: \"{}\"\n", indent, escape_dsl_string(icon)));
        }
        if let Some(pos) = &node.position {
            out.push_str(&format!(
                "{}    position: {},{}\n",
                indent,
                format_num(pos.x),
                format_num(pos.y)
            ));
        }
        if let Some(size) = &node.size {
            // Only emit if it looks user-specified (round width or non-default height).
            if size.height != 40.0 {
                out.push_str(&format!(
                    "{}    size: {}x{}\n",
                    indent,
                    format_num(size.width),
                    format_num(size.height)
                ));
            }
        }
        if has_any_style(&node.style) {
            out.push_str(&format!("{}    style: {{ ", indent));
            let mut parts: Vec<String> = Vec::new();
            if let Some(v) = &node.style.fill {
                parts.push(format!("fill: \"{}\"", v));
            }
            if let Some(v) = &node.style.stroke {
                parts.push(format!("stroke: \"{}\"", v));
            }
            if let Some(v) = node.style.stroke_width {
                parts.push(format!("stroke_width: {}", format_num(v)));
            }
            if let Some(v) = &node.style.font_color {
                parts.push(format!("font_color: \"{}\"", v));
            }
            if let Some(v) = node.style.font_size {
                parts.push(format!("font_size: {}", format_num(v)));
            }
            out.push_str(&parts.join(", "));
            out.push_str(" }\n");
        }
        out.push_str(&format!("{}}}\n", indent));
    } else {
        out.push('\n');
    }
}

fn emit_edge(edge: &di_ag_ir::Edge, depth: usize, out: &mut String) {
    let indent = "    ".repeat(depth);
    out.push_str(&format!(
        "{}edge {} -> {}",
        indent, edge.source, edge.target
    ));
    if let Some(label) = &edge.label {
        out.push_str(&format!(
            " {{ label: \"{}\" }}",
            escape_dsl_string(label)
        ));
    }
    out.push('\n');
}

fn has_any_style(s: &di_ag_ir::NodeStyle) -> bool {
    s.fill.is_some()
        || s.stroke.is_some()
        || s.stroke_width.is_some()
        || s.font_color.is_some()
        || s.font_size.is_some()
        || s.font_family.is_some()
        || s.border_radius.is_some()
        || s.opacity.is_some()
}

fn diagram_type_str(dt: &di_ag_ir::DiagramType) -> &'static str {
    match dt {
        di_ag_ir::DiagramType::Flowchart => "flowchart",
        di_ag_ir::DiagramType::Sequence => "sequence",
        di_ag_ir::DiagramType::Er => "er",
        di_ag_ir::DiagramType::Class => "class",
        di_ag_ir::DiagramType::Tree => "tree",
        di_ag_ir::DiagramType::Freeform => "freeform",
    }
}

fn direction_str(d: &LayoutDirection) -> &'static str {
    match d {
        LayoutDirection::TopToBottom => "TB",
        LayoutDirection::BottomToTop => "BT",
        LayoutDirection::LeftToRight => "LR",
        LayoutDirection::RightToLeft => "RL",
    }
}

fn shape_str(s: &Shape) -> &'static str {
    match s {
        Shape::Rect => "rect",
        Shape::RoundedRect => "rounded_rect",
        Shape::Diamond => "diamond",
        Shape::Circle => "circle",
        Shape::Ellipse => "ellipse",
        Shape::Cylinder => "cylinder",
        Shape::Parallelogram => "parallelogram",
        Shape::Hexagon => "hexagon",
        Shape::Triangle => "triangle",
    }
}

fn format_num(n: f64) -> String {
    if n.fract().abs() < 1e-9 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

fn escape_dsl_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::emit;

    #[test]
    fn test_roundtrip_icon_and_position() {
        let src = r#"node db "Database" {
    icon: "database"
    position: 150, 75
}
"#;
        let parsed = di_ag_dsl::parse(src).expect("initial parse");
        assert_eq!(parsed.nodes[0].icon.as_deref(), Some("database"));
        let pos = parsed.nodes[0]
            .position
            .as_ref()
            .expect("expected position");
        assert_eq!(pos.x, 150.0);
        assert_eq!(pos.y, 75.0);

        let emitted = emit(&parsed);
        // Re-parse the emitted DSL and confirm both fields survived.
        let reparsed = di_ag_dsl::parse(&emitted).expect("reparse");
        assert_eq!(reparsed.nodes[0].icon.as_deref(), Some("database"));
        let pos2 = reparsed.nodes[0]
            .position
            .as_ref()
            .expect("expected position after roundtrip");
        assert_eq!(pos2.x, 150.0);
        assert_eq!(pos2.y, 75.0);
    }
}
