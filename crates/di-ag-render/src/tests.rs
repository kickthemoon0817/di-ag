#[cfg(test)]
mod tests {
    use crate::render_svg;
    use di_ag_ir::*;

    fn make_two_node_doc() -> Document {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "Hello".into(),
            icon: None,
            shape: Shape::Rect,
            position: Some(Position { x: 10.0, y: 10.0 }),
            size: Some(Size {
                width: 100.0,
                height: 40.0,
            }),
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        doc.nodes.push(Node {
            id: "b".into(),
            label: "World".into(),
            icon: None,
            shape: Shape::Rect,
            position: Some(Position {
                x: 10.0,
                y: 110.0,
            }),
            size: Some(Size {
                width: 100.0,
                height: 40.0,
            }),
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        doc.edges.push(Edge {
            id: "e0".into(),
            source: "a".into(),
            target: "b".into(),
            label: None,
            edge_type: EdgeType::Straight,
            waypoints: vec![
                Waypoint { x: 60.0, y: 50.0 },
                Waypoint {
                    x: 60.0,
                    y: 110.0,
                },
            ],
            style: EdgeStyle::default(),
        });
        doc
    }

    #[test]
    fn test_render_produces_valid_svg() {
        let doc = make_two_node_doc();
        let svg = render_svg(&doc).unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_render_includes_data_ids() {
        let doc = make_two_node_doc();
        let svg = render_svg(&doc).unwrap();
        assert!(svg.contains(r#"data-id="a""#));
        assert!(svg.contains(r#"data-id="b""#));
        assert!(svg.contains(r#"data-id="e0""#));
    }

    #[test]
    fn test_render_includes_labels() {
        let doc = make_two_node_doc();
        let svg = render_svg(&doc).unwrap();
        assert!(svg.contains("Hello"));
        assert!(svg.contains("World"));
    }

    #[test]
    fn test_render_diamond_shape() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "d".into(),
            label: "Decision".into(),
            icon: None,
            shape: Shape::Diamond,
            position: Some(Position { x: 50.0, y: 50.0 }),
            size: Some(Size {
                width: 100.0,
                height: 80.0,
            }),
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let svg = render_svg(&doc).unwrap();
        assert!(svg.contains("polygon"));
        assert!(svg.contains("Decision"));
    }

    #[test]
    fn test_render_edge_with_label() {
        let mut doc = make_two_node_doc();
        doc.edges[0].label = Some("connects".into());
        let svg = render_svg(&doc).unwrap();
        assert!(svg.contains("connects"));
    }

    #[test]
    fn icon_names_match_icons_table() {
        use crate::icons::{icon_description, icon_svg, ICONS, ICON_NAMES};
        assert_eq!(
            ICON_NAMES.len(),
            ICONS.len(),
            "ICON_NAMES ({}) and ICONS ({}) have diverged",
            ICON_NAMES.len(),
            ICONS.len()
        );
        for name in ICON_NAMES {
            assert!(
                icon_svg(name).is_some(),
                "ICON_NAMES has '{}' but icon_svg returns None",
                name
            );
            assert!(
                icon_description(name).is_some(),
                "ICON_NAMES has '{}' but icon_description returns None",
                name
            );
        }
        for entry in ICONS {
            assert!(
                ICON_NAMES.contains(&entry.name),
                "ICONS has '{}' but ICON_NAMES does not",
                entry.name
            );
        }
    }

    #[test]
    fn test_render_custom_node_style() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "s".into(),
            label: "Styled".into(),
            icon: None,
            shape: Shape::Rect,
            position: Some(Position { x: 0.0, y: 0.0 }),
            size: Some(Size {
                width: 100.0,
                height: 40.0,
            }),
            style: NodeStyle {
                fill: Some("#ff0000".into()),
                stroke: Some("#000000".into()),
                stroke_width: Some(2.0),
                ..NodeStyle::default()
            },
            ports: vec![],
            children: vec![],
        });
        let svg = render_svg(&doc).unwrap();
        assert!(svg.contains("#ff0000"));
        assert!(svg.contains("#000000"));
    }
}
