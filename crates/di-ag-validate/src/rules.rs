use std::collections::{HashMap, HashSet};

use di_ag_ir::Document;

use crate::report::{Severity, Violation};

/// Simple substring/prefix-based closest-match finder. Returns up to `max`
/// names from `candidates` that share the longest common prefix or contain
/// the query as a substring.
fn close_matches(query: &str, candidates: &[&str], max: usize) -> Vec<String> {
    let q = query.to_ascii_lowercase();
    let mut scored: Vec<(&str, usize)> = candidates
        .iter()
        .map(|&c| {
            let score = if c == q {
                1000
            } else {
                // longest common prefix length
                c.chars()
                    .zip(q.chars())
                    .take_while(|(a, b)| a == b)
                    .count()
                    * 10
                    + if c.contains(q.as_str()) { 5 } else { 0 }
            };
            (c, score)
        })
        .filter(|(_, s)| *s > 0)
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.truncate(max);
    scored.into_iter().map(|(c, _)| c.to_string()).collect()
}

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
                    fix_hint: Some(format!(
                        "Rename one of the '{}' nodes to a unique id",
                        node.id
                    )),
                    suggestions: None,
                });
            }
            visit(&node.children, seen, violations);
        }
    }

    visit(&doc.nodes, &mut seen, &mut violations);
    violations
}

pub fn check_duplicate_edge_ids(doc: &Document) -> Vec<Violation> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut violations = vec![];
    for edge in &doc.edges {
        if !seen.insert(edge.id.clone()) {
            violations.push(Violation {
                violation_type: "duplicate_edge_id".into(),
                severity: Severity::Error,
                node: None,
                nodes: None,
                edge: Some(edge.id.clone()),
                message: Some(format!("Duplicate edge id: {}", edge.id)),
                fix_hint: Some("Edge ids must be unique across the document".into()),
                suggestions: None,
            });
        }
    }
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
                fix_hint: Some(format!(
                    "Connect '{}' to another node with an edge, or remove it",
                    node.id
                )),
                suggestions: None,
            });
        }
    }
    violations
}

pub fn check_label_lengths(doc: &Document) -> Vec<Violation> {
    let mut violations = vec![];

    fn visit(nodes: &[di_ag_ir::Node], violations: &mut Vec<Violation>) {
        for node in nodes {
            let len = node.label.chars().count();
            if len > MAX_LABEL_LENGTH {
                violations.push(Violation {
                    violation_type: "label_too_long".into(),
                    severity: Severity::Warn,
                    node: Some(node.id.clone()),
                    nodes: None,
                    edge: None,
                    message: Some(format!(
                        "Label length {} exceeds max {}",
                        len, MAX_LABEL_LENGTH
                    )),
                    fix_hint: Some("Shorten the label or use an edge label for detail".into()),
                    suggestions: None,
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
                fix_hint: Some(format!(
                    "Define a node with id '{}' or fix the edge source",
                    edge.source
                )),
                suggestions: None,
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
                fix_hint: Some(format!(
                    "Define a node with id '{}' or fix the edge target",
                    edge.target
                )),
                suggestions: None,
            });
        }
    }
    violations
}

pub fn check_self_loops(doc: &Document) -> Vec<Violation> {
    let mut out = vec![];
    for edge in &doc.edges {
        if edge.source == edge.target {
            out.push(Violation {
                violation_type: "self_loop".into(),
                severity: Severity::Warn,
                edge: Some(edge.id.clone()),
                node: Some(edge.source.clone()),
                nodes: None,
                message: Some(format!(
                    "Edge '{}' is a self-loop on '{}'",
                    edge.id, edge.source
                )),
                fix_hint: Some(
                    "Self-loops are rarely intentional in flowcharts; split into two nodes if needed"
                        .into(),
                ),
                suggestions: None,
            });
        }
    }
    out
}

/// Detect directed cycles via DFS with gray/black coloring. Reports one violation
/// per strongly-connected cycle member set encountered.
pub fn check_cycles(doc: &Document) -> Vec<Violation> {
    // Build adjacency at flat level (top-level node ids + container child ids).
    let mut all_ids: HashSet<String> = HashSet::new();
    fn collect(nodes: &[di_ag_ir::Node], ids: &mut HashSet<String>) {
        for n in nodes {
            ids.insert(n.id.clone());
            collect(&n.children, ids);
        }
    }
    collect(&doc.nodes, &mut all_ids);

    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for id in &all_ids {
        adj.insert(id.clone(), Vec::new());
    }
    for edge in &doc.edges {
        if edge.source == edge.target {
            continue; // self-loops handled separately
        }
        let src = edge.source.split('.').next().unwrap_or(&edge.source).to_string();
        let tgt = edge.target.split('.').next().unwrap_or(&edge.target).to_string();
        if all_ids.contains(&src) && all_ids.contains(&tgt) {
            adj.entry(src).or_default().push(tgt);
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }
    let mut color: HashMap<String, Color> = all_ids.iter().map(|id| (id.clone(), Color::White)).collect();
    let mut cycle_nodes: HashSet<String> = HashSet::new();

    // Iterative DFS to keep stack shallow.
    let mut ordered: Vec<String> = all_ids.iter().cloned().collect();
    ordered.sort();
    for start in ordered {
        if color.get(&start).copied() != Some(Color::White) {
            continue;
        }
        let mut stack: Vec<(String, usize)> = vec![(start.clone(), 0)];
        color.insert(start.clone(), Color::Gray);
        while let Some((node, idx)) = stack.pop() {
            let succs = adj.get(&node).cloned().unwrap_or_default();
            if idx < succs.len() {
                let next = succs[idx].clone();
                stack.push((node.clone(), idx + 1));
                match color.get(&next).copied() {
                    Some(Color::White) => {
                        color.insert(next.clone(), Color::Gray);
                        stack.push((next, 0));
                    }
                    Some(Color::Gray) => {
                        // Back edge — every gray node on the stack is part of a cycle.
                        cycle_nodes.insert(next.clone());
                        for (n, _) in &stack {
                            if color.get(n).copied() == Some(Color::Gray) {
                                cycle_nodes.insert(n.clone());
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                color.insert(node, Color::Black);
            }
        }
    }

    if cycle_nodes.is_empty() {
        return vec![];
    }

    let mut nodes: Vec<String> = cycle_nodes.into_iter().collect();
    nodes.sort();
    vec![Violation {
        violation_type: "cycle".into(),
        severity: Severity::Warn,
        node: None,
        nodes: Some(nodes),
        edge: None,
        message: Some("Directed cycle detected in graph".into()),
        fix_hint: Some(
            "Flowcharts usually should not contain cycles; consider a loopback label or break the cycle"
                .into(),
        ),
        suggestions: None,
    }]
}

pub fn check_unknown_icons(doc: &Document) -> Vec<Violation> {
    let known: Vec<&str> = di_ag_render::ICON_NAMES.to_vec();
    let available_list = known.join(", ");
    let mut violations = vec![];

    fn visit(
        nodes: &[di_ag_ir::Node],
        known: &[&str],
        available_list: &str,
        violations: &mut Vec<Violation>,
    ) {
        for node in nodes {
            if let Some(icon_name) = &node.icon {
                if di_ag_render::icon_svg(icon_name).is_none() {
                    let suggestions = close_matches(icon_name, known, 3);
                    violations.push(Violation {
                        violation_type: "unknown_icon".into(),
                        severity: Severity::Warn,
                        node: Some(node.id.clone()),
                        nodes: None,
                        edge: None,
                        message: Some(format!(
                            "Unknown icon name: '{}'. Available: {}",
                            icon_name, available_list
                        )),
                        fix_hint: Some(format!(
                            "Use one of the built-in icon names: {}",
                            available_list
                        )),
                        suggestions: if suggestions.is_empty() {
                            None
                        } else {
                            Some(suggestions)
                        },
                    });
                }
            }
            visit(&node.children, known, available_list, violations);
        }
    }

    visit(&doc.nodes, &known, &available_list, &mut violations);
    violations
}
