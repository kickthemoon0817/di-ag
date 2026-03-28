#[cfg(test)]
mod tests {
    use crate::inspect;
    use di_ag_ir::*;

    fn make_clean_layout() -> Document {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".into(),
            shape: Shape::Rect,
            position: Some(Position { x: 0.0, y: 0.0 }),
            size: Some(Size {
                width: 80.0,
                height: 40.0,
            }),
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        doc.nodes.push(Node {
            id: "b".into(),
            label: "B".into(),
            shape: Shape::Rect,
            position: Some(Position { x: 0.0, y: 100.0 }),
            size: Some(Size {
                width: 80.0,
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
                Waypoint { x: 40.0, y: 40.0 },
                Waypoint {
                    x: 40.0,
                    y: 100.0,
                },
            ],
            style: EdgeStyle::default(),
        });
        doc
    }

    #[test]
    fn test_clean_layout_high_score() {
        let doc = make_clean_layout();
        let report = inspect(&doc);
        assert!(
            report.score >= 80.0,
            "Clean layout should score >= 80, got {}",
            report.score
        );
    }

    #[test]
    fn test_clean_layout_no_issues() {
        let doc = make_clean_layout();
        let report = inspect(&doc);
        let errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.issue_type == "node_overlap" || i.issue_type == "edge_crossing")
            .collect();
        assert!(
            errors.is_empty(),
            "Clean layout should have no overlap/crossing issues"
        );
    }

    #[test]
    fn test_overlapping_nodes_detected() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".into(),
            shape: Shape::Rect,
            position: Some(Position { x: 0.0, y: 0.0 }),
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
            label: "B".into(),
            shape: Shape::Rect,
            position: Some(Position { x: 50.0, y: 20.0 }),
            size: Some(Size {
                width: 100.0,
                height: 40.0,
            }),
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let report = inspect(&doc);
        assert!(report
            .issues
            .iter()
            .any(|i| i.issue_type == "node_overlap"));
        assert!(report.metrics.node_overlaps > 0);
    }

    #[test]
    fn test_report_serializes_to_json() {
        let doc = make_clean_layout();
        let report = inspect(&doc);
        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("\"score\""));
        assert!(json.contains("\"metrics\""));
        assert!(json.contains("\"issues\""));
    }

    #[test]
    fn test_fix_hints_present() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".into(),
            shape: Shape::Rect,
            position: Some(Position { x: 0.0, y: 0.0 }),
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
            label: "B".into(),
            shape: Shape::Rect,
            position: Some(Position { x: 30.0, y: 10.0 }),
            size: Some(Size {
                width: 100.0,
                height: 40.0,
            }),
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let report = inspect(&doc);
        for issue in &report.issues {
            assert!(
                issue.fix_hint.is_some(),
                "Issue '{}' should have a fix_hint",
                issue.issue_type
            );
        }
    }
}
