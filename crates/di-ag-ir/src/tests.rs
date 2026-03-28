#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_empty_document_roundtrip_json() {
        let doc = Document::default();
        let json = serde_json::to_string(&doc).unwrap();
        let parsed: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes.len(), 0);
        assert_eq!(parsed.edges.len(), 0);
    }

    #[test]
    fn test_document_with_node_roundtrip_json() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "Node A".into(),
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![Port::Top, Port::Bottom],
            children: vec![],
        });
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let parsed: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.nodes[0].id, "a");
        assert_eq!(parsed.nodes[0].label, "Node A");
        assert_eq!(parsed.nodes[0].shape, Shape::Rect);
        assert!(parsed.nodes[0].position.is_none());
        assert_eq!(parsed.nodes[0].ports, vec![Port::Top, Port::Bottom]);
    }

    #[test]
    fn test_document_with_edge_roundtrip_json() {
        let mut doc = Document::default();
        doc.nodes.push(Node {
            id: "a".into(),
            label: "A".into(),
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
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        doc.edges.push(Edge {
            id: "e1".into(),
            source: "a".into(),
            target: "b".into(),
            label: Some("connects".into()),
            edge_type: EdgeType::Straight,
            waypoints: vec![],
            style: EdgeStyle::default(),
        });
        let json = serde_json::to_string(&doc).unwrap();
        let parsed: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.edges.len(), 1);
        assert_eq!(parsed.edges[0].source, "a");
        assert_eq!(parsed.edges[0].target, "b");
        assert_eq!(parsed.edges[0].label, Some("connects".into()));
    }

    #[test]
    fn test_document_roundtrip_yaml() {
        let mut doc = Document::default();
        doc.metadata.title = Some("Test".into());
        doc.nodes.push(Node {
            id: "x".into(),
            label: "X".into(),
            shape: Shape::Diamond,
            position: Some(Position { x: 100.0, y: 200.0 }),
            size: Some(Size { width: 80.0, height: 40.0 }),
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        });
        let yaml = serde_yaml::to_string(&doc).unwrap();
        let parsed: Document = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.metadata.title, Some("Test".into()));
        assert_eq!(parsed.nodes[0].position.as_ref().unwrap().x, 100.0);
        assert_eq!(parsed.nodes[0].size.as_ref().unwrap().width, 80.0);
    }

    #[test]
    fn test_nested_container() {
        let child = Node {
            id: "inner".into(),
            label: "Inner".into(),
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![],
        };
        let container = Node {
            id: "outer".into(),
            label: "Outer".into(),
            shape: Shape::Rect,
            position: None,
            size: None,
            style: NodeStyle::default(),
            ports: vec![],
            children: vec![child],
        };
        let mut doc = Document::default();
        doc.nodes.push(container);
        let json = serde_json::to_string(&doc).unwrap();
        let parsed: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes[0].children.len(), 1);
        assert_eq!(parsed.nodes[0].children[0].id, "inner");
    }

    #[test]
    fn test_preset_serialization() {
        let mut doc = Document::default();
        doc.preset = Some(Preset {
            diagram_type: DiagramType::Flowchart,
            theme: Some("dark".into()),
            layout: Some(LayoutConfig {
                direction: LayoutDirection::TopToBottom,
                spacing: 40.0,
            }),
        });
        let json = serde_json::to_string(&doc).unwrap();
        let parsed: Document = serde_json::from_str(&json).unwrap();
        let preset = parsed.preset.unwrap();
        assert_eq!(preset.diagram_type, DiagramType::Flowchart);
        assert_eq!(preset.theme, Some("dark".into()));
        assert_eq!(preset.layout.unwrap().spacing, 40.0);
    }
}
