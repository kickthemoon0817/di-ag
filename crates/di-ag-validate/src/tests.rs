#[cfg(test)]
mod tests {
    use crate::validate;
    use di_ag_ir::*;

    #[test]
    fn test_valid_document_passes() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".into(),
            icon: None,
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let report = validate(&doc);
        assert!(report.valid);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_duplicate_node_ids() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A1".into(),
            icon: None,
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A2".into(),
            icon: None,
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let report = validate(&doc);
        assert!(!report.valid);
        assert!(report
            .violations
            .iter()
            .any(|v| v.violation_type == "duplicate_id"));
    }

    #[test]
    fn test_orphan_node_detection() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".into(),
            icon: None,
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        doc.nodes.push(Node {
            id: "b".into(),
            label: "B".into(),
            icon: None,
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        doc.nodes.push(Node {
            id: "c".into(),
            label: "C".into(),
            icon: None,
            shape: Shape::Rect,
            position: None,
            size: None,
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
            waypoints: vec![],
            style: EdgeStyle::default(),
        });
        let report = validate(&doc);
        assert!(report
            .violations
            .iter()
            .any(|v| v.violation_type == "orphan_node" && v.node == Some("c".into())));
    }

    #[test]
    fn test_label_too_long() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".repeat(100),
            icon: None,
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let report = validate(&doc);
        assert!(report
            .violations
            .iter()
            .any(|v| v.violation_type == "label_too_long"));
    }

    #[test]
    fn test_edge_references_missing_node() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".into(),
            icon: None,
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        doc.edges.push(Edge {
            id: "e0".into(),
            source: "a".into(),
            target: "nonexistent".into(),
            label: None,
            edge_type: EdgeType::Straight,
            waypoints: vec![],
            style: EdgeStyle::default(),
        });
        let report = validate(&doc);
        assert!(!report.valid);
        assert!(report
            .violations
            .iter()
            .any(|v| v.violation_type == "missing_node_ref"));
    }

    #[test]
    fn test_unknown_icon_produces_violation() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".into(),
            icon: Some("xyz".into()),
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let report = validate(&doc);
        assert!(report
            .violations
            .iter()
            .any(|v| v.violation_type == "unknown_icon"));
    }

    #[test]
    fn test_unknown_icon_includes_suggestions() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".into(),
            icon: Some("databse".into()), // intentional typo
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let report = validate(&doc);
        let v = report
            .violations
            .iter()
            .find(|v| v.violation_type == "unknown_icon")
            .expect("expected unknown_icon violation");
        let suggestions = v
            .suggestions
            .as_ref()
            .expect("expected non-empty suggestion list");
        assert!(
            suggestions.iter().any(|s| s == "database"),
            "expected 'database' in suggestions, got {:?}",
            suggestions
        );
    }

    #[test]
    fn test_known_icon_no_violation() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".into(),
            icon: Some("user".into()),
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let report = validate(&doc);
        assert!(!report
            .violations
            .iter()
            .any(|v| v.violation_type == "unknown_icon"));
    }

    #[test]
    fn test_serializes_to_json() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".repeat(100),
            icon: None,
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let report = validate(&doc);
        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("label_too_long"));
        assert!(json.contains("\"valid\""));
    }
}
