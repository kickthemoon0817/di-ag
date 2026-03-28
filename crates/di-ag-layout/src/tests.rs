#[cfg(test)]
mod tests {
    use crate::{layout, score};
    use di_ag_ir::*;

    fn make_simple_chain() -> Document {
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
        doc.nodes.push(Node {
            id: "c".into(),
            label: "C".into(),
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
        doc.edges.push(Edge {
            id: "e1".into(),
            source: "b".into(),
            target: "c".into(),
            label: None,
            edge_type: EdgeType::Straight,
            waypoints: vec![],
            style: EdgeStyle::default(),
        });
        doc
    }

    #[test]
    fn test_layout_assigns_positions() {
        let doc = make_simple_chain();
        let result = layout(doc).unwrap();
        for node in &result.nodes {
            assert!(
                node.position.is_some(),
                "Node {} should have position",
                node.id
            );
        }
    }

    #[test]
    fn test_layout_assigns_sizes() {
        let doc = make_simple_chain();
        let result = layout(doc).unwrap();
        for node in &result.nodes {
            assert!(node.size.is_some(), "Node {} should have size", node.id);
        }
    }

    #[test]
    fn test_layout_top_to_bottom_ordering() {
        let doc = make_simple_chain();
        let result = layout(doc).unwrap();
        let pos_a = result
            .nodes
            .iter()
            .find(|n| n.id == "a")
            .unwrap()
            .position
            .as_ref()
            .unwrap();
        let pos_b = result
            .nodes
            .iter()
            .find(|n| n.id == "b")
            .unwrap()
            .position
            .as_ref()
            .unwrap();
        let pos_c = result
            .nodes
            .iter()
            .find(|n| n.id == "c")
            .unwrap()
            .position
            .as_ref()
            .unwrap();
        assert!(pos_a.y < pos_b.y, "A should be above B");
        assert!(pos_b.y < pos_c.y, "B should be above C");
    }

    #[test]
    fn test_layout_no_overlaps() {
        let doc = make_simple_chain();
        let result = layout(doc).unwrap();
        let nodes: Vec<_> = result.nodes.iter().collect();
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let a_pos = nodes[i].position.as_ref().unwrap();
                let a_size = nodes[i].size.as_ref().unwrap();
                let b_pos = nodes[j].position.as_ref().unwrap();
                let b_size = nodes[j].size.as_ref().unwrap();
                let overlap_x =
                    a_pos.x < b_pos.x + b_size.width && a_pos.x + a_size.width > b_pos.x;
                let overlap_y =
                    a_pos.y < b_pos.y + b_size.height && a_pos.y + a_size.height > b_pos.y;
                assert!(
                    !(overlap_x && overlap_y),
                    "Nodes {} and {} overlap",
                    nodes[i].id,
                    nodes[j].id
                );
            }
        }
    }

    #[test]
    fn test_layout_respects_existing_positions() {
        let mut doc = make_simple_chain();
        doc.nodes[0].position = Some(Position { x: 500.0, y: 500.0 });
        let result = layout(doc).unwrap();
        let pos = result.nodes[0].position.as_ref().unwrap();
        assert_eq!(pos.x, 500.0);
        assert_eq!(pos.y, 500.0);
    }

    #[test]
    fn test_layout_scoring() {
        let doc = make_simple_chain();
        let result = layout(doc).unwrap();
        let s = score(&result);
        assert!(
            s >= 0.0 && s <= 100.0,
            "Score should be 0-100, got {}",
            s
        );
        assert!(s > 50.0, "Simple chain should score well, got {}", s);
    }
}
