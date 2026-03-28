use std::collections::HashSet;

use di_ag_ir::Document;

use crate::report::{Severity, Violation};

const MAX_LABEL_LENGTH: usize = 60;

pub fn check_duplicate_ids(doc: &Document) -> Vec<Violation> {
    let mut seen = HashSet::new();
    let mut violations = vec![];

    fn visit(
        nodes: &[di_ag_ir::Node],
        seen: &mut HashSet<String>,
        violations: &mut Vec<Violation>,
    ) {
        for node in nodes {
            if !seen.insert(node.id.clone()) {
                violations.push(Violation {
                    violation_type: "duplicate_id".into(),
                    severity: Severity::Error,
                    node: Some(node.id.clone()),
                    nodes: None,
                    edge: None,
                    message: Some(format!("Duplicate node id: {}", node.id)),
                });
            }
            visit(&node.children, seen, violations);
        }
    }

    visit(&doc.nodes, &mut seen, &mut violations);
    violations
}

pub fn check_orphan_nodes(doc: &Document) -> Vec<Violation> {
    if doc.edges.is_empty() {
        return vec![];
    }

    let mut connected: HashSet<String> = HashSet::new();
    for edge in &doc.edges {
        connected.insert(edge.source.clone());
        connected.insert(edge.target.clone());
    }

    let mut violations = vec![];
    for node in &doc.nodes {
        if !connected.contains(&node.id) && node.children.is_empty() {
            violations.push(Violation {
                violation_type: "orphan_node".into(),
                severity: Severity::Info,
                node: Some(node.id.clone()),
                nodes: None,
                edge: None,
                message: Some(format!("Node '{}' has no connections", node.id)),
            });
        }
    }
    violations
}

pub fn check_label_lengths(doc: &Document) -> Vec<Violation> {
    let mut violations = vec![];

    fn visit(nodes: &[di_ag_ir::Node], violations: &mut Vec<Violation>) {
        for node in nodes {
            if node.label.len() > MAX_LABEL_LENGTH {
                violations.push(Violation {
                    violation_type: "label_too_long".into(),
                    severity: Severity::Warn,
                    node: Some(node.id.clone()),
                    nodes: None,
                    edge: None,
                    message: Some(format!(
                        "Label length {} exceeds max {}",
                        node.label.len(),
                        MAX_LABEL_LENGTH
                    )),
                });
            }
            visit(&node.children, violations);
        }
    }

    visit(&doc.nodes, &mut violations);
    violations
}

pub fn check_edge_references(doc: &Document) -> Vec<Violation> {
    let mut all_ids: HashSet<String> = HashSet::new();

    fn collect_ids(nodes: &[di_ag_ir::Node], ids: &mut HashSet<String>) {
        for node in nodes {
            ids.insert(node.id.clone());
            collect_ids(&node.children, ids);
        }
    }

    collect_ids(&doc.nodes, &mut all_ids);

    let mut violations = vec![];
    for edge in &doc.edges {
        let source_base = edge.source.split('.').next().unwrap_or(&edge.source);
        if !all_ids.contains(source_base) {
            violations.push(Violation {
                violation_type: "missing_node_ref".into(),
                severity: Severity::Error,
                edge: Some(edge.id.clone()),
                node: None,
                nodes: None,
                message: Some(format!(
                    "Edge '{}' references unknown source '{}'",
                    edge.id, edge.source
                )),
            });
        }
        let target_base = edge.target.split('.').next().unwrap_or(&edge.target);
        if !all_ids.contains(target_base) {
            violations.push(Violation {
                violation_type: "missing_node_ref".into(),
                severity: Severity::Error,
                edge: Some(edge.id.clone()),
                node: None,
                nodes: None,
                message: Some(format!(
                    "Edge '{}' references unknown target '{}'",
                    edge.id, edge.target
                )),
            });
        }
    }
    violations
}
