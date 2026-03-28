#[cfg(test)]
mod tests {
    use crate::parse;
    use di_ag_ir::*;

    #[test]
    fn test_parse_single_node() {
        let input = r#"node a "Node A""#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(doc.nodes[0].id, "a");
        assert_eq!(doc.nodes[0].label, "Node A");
    }

    #[test]
    fn test_parse_node_with_block() {
        let input = r#"
node api "API Server" {
    shape: rect
    size: 200x80
}
"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes[0].id, "api");
        assert_eq!(doc.nodes[0].label, "API Server");
        assert_eq!(doc.nodes[0].shape, Shape::Rect);
        assert_eq!(
            doc.nodes[0].size,
            Some(Size {
                width: 200.0,
                height: 80.0
            })
        );
    }

    #[test]
    fn test_parse_edge() {
        let input = r#"
node a "A"
node b "B"
edge a -> b
"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.edges.len(), 1);
        assert_eq!(doc.edges[0].source, "a");
        assert_eq!(doc.edges[0].target, "b");
    }

    #[test]
    fn test_parse_edge_with_label_and_block() {
        let input = r#"
node a "A"
node b "B"
edge a -> b {
    label: "connects"
    route: orthogonal
}
"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.edges[0].label, Some("connects".into()));
        assert_eq!(doc.edges[0].edge_type, EdgeType::Orthogonal);
    }

    #[test]
    fn test_parse_directives() {
        let input = r#"
@preset flowchart
@theme dark
@layout direction=TB spacing=40

node a "A"
"#;
        let doc = parse(input).unwrap();
        let preset = doc.preset.unwrap();
        assert_eq!(preset.diagram_type, DiagramType::Flowchart);
        assert_eq!(preset.theme, Some("dark".into()));
        let layout = preset.layout.unwrap();
        assert_eq!(layout.direction, LayoutDirection::TopToBottom);
        assert_eq!(layout.spacing, 40.0);
    }

    #[test]
    fn test_parse_container() {
        let input = r#"
container backend "Backend" {
    node api "API"
    node db "Database"
    edge api -> db
}
"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(doc.nodes[0].id, "backend");
        assert_eq!(doc.nodes[0].children.len(), 2);
        assert_eq!(doc.edges.len(), 1);
    }

    #[test]
    fn test_parse_variable_and_spread() {
        let input = r##"
let primary = "#2196F3"

node a "A" {
    style: { fill: $primary }
}
"##;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes[0].style.fill, Some("#2196F3".into()));
    }

    #[test]
    fn test_parse_node_with_ports() {
        let input = r#"
node api "API" {
    port: [top, bottom, left, right]
}
"#;
        let doc = parse(input).unwrap();
        assert_eq!(
            doc.nodes[0].ports,
            vec![Port::Top, Port::Bottom, Port::Left, Port::Right]
        );
    }

    #[test]
    fn test_parse_comments_ignored() {
        let input = r#"
# This is a comment
node a "A"
# Another comment
node b "B"
"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes.len(), 2);
    }

    #[test]
    fn test_parse_node_with_style_block() {
        let input = r##"
node a "A" {
    shape: diamond
    style: { fill: "#ff0000", stroke: "#000000", border_radius: 8 }
}
"##;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes[0].shape, Shape::Diamond);
        assert_eq!(doc.nodes[0].style.fill, Some("#ff0000".into()));
        assert_eq!(doc.nodes[0].style.stroke, Some("#000000".into()));
        assert_eq!(doc.nodes[0].style.border_radius, Some(8.0));
    }

    #[test]
    fn test_parse_layout_directives() {
        let input = r#"
node a "A"
node b "B"
node c "C"
align horizontal [a, b, c]
"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes.len(), 3);
    }

    #[test]
    fn test_parse_repeat_block() {
        let input = r#"
node lb "Load Balancer"

repeat 3 as i {
    node worker_$i "Worker $i"
    edge lb -> worker_$i
}
"#;
        let doc = parse(input).unwrap();
        // lb + worker_0, worker_1, worker_2
        assert_eq!(doc.nodes.len(), 4);
        assert_eq!(doc.nodes[1].id, "worker_0");
        assert_eq!(doc.nodes[1].label, "Worker 0");
        assert_eq!(doc.nodes[2].id, "worker_1");
        assert_eq!(doc.nodes[3].id, "worker_2");
        assert_eq!(doc.edges.len(), 3);
        assert_eq!(doc.edges[0].target, "worker_0");
        assert_eq!(doc.edges[1].target, "worker_1");
        assert_eq!(doc.edges[2].target, "worker_2");
    }

    #[test]
    fn test_parse_chain_shorthand() {
        let input = r#"[Start] --> [Process] --> [End]"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes.len(), 3);
        assert_eq!(doc.nodes[0].id, "start");
        assert_eq!(doc.nodes[0].label, "Start");
        assert_eq!(doc.nodes[0].shape, Shape::Rect);
        assert_eq!(doc.nodes[1].id, "process");
        assert_eq!(doc.nodes[2].id, "end");
        assert_eq!(doc.edges.len(), 2);
        assert_eq!(doc.edges[0].source, "start");
        assert_eq!(doc.edges[0].target, "process");
        assert_eq!(doc.edges[1].source, "process");
        assert_eq!(doc.edges[1].target, "end");
    }

    #[test]
    fn test_parse_chain_with_shapes() {
        let input = r#"[Input] --> {Decision} --> (Done)"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes[0].shape, Shape::Rect);
        assert_eq!(doc.nodes[1].shape, Shape::Diamond);
        assert_eq!(doc.nodes[1].label, "Decision");
        assert_eq!(doc.nodes[2].shape, Shape::RoundedRect);
        assert_eq!(doc.nodes[2].label, "Done");
    }

    #[test]
    fn test_parse_chain_with_labels() {
        let input = r#"[A] --yes--> [B] --no--> [C]"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.edges[0].label, Some("yes".into()));
        assert_eq!(doc.edges[1].label, Some("no".into()));
    }

    #[test]
    fn test_parse_repeat_with_braces() {
        let input = r#"
repeat 2 as idx {
    node svc_${idx} "Service ${idx}"
}
"#;
        let doc = parse(input).unwrap();
        assert_eq!(doc.nodes.len(), 2);
        assert_eq!(doc.nodes[0].id, "svc_0");
        assert_eq!(doc.nodes[0].label, "Service 0");
        assert_eq!(doc.nodes[1].id, "svc_1");
    }
}
