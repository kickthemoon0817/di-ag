pub mod geometry;
pub mod report;
pub mod scoring;

#[cfg(test)]
mod tests;

use di_ag_ir::Document;
pub use report::{InspectionReport, Issue, Metrics};

pub fn inspect(doc: &Document) -> InspectionReport {
    let mut issues = vec![];

    // Check overlaps
    let overlaps = geometry::find_overlapping_pairs(doc);
    let node_overlaps = overlaps.len() as u32;
    for (a, b) in &overlaps {
        issues.push(Issue {
            issue_type: "node_overlap".into(),
            nodes: Some(vec![a.clone(), b.clone()]),
            edges: None,
            at: None,
            bounds: None,
            fix_hint: Some(format!(
                "Move '{}' and '{}' apart to eliminate overlap",
                a, b
            )),
        });
    }

    // Check edge crossings
    let crossings = geometry::find_edge_crossings(doc);
    let edge_crossings = crossings.len() as u32;
    for (e1, e2, x, y) in &crossings {
        issues.push(Issue {
            issue_type: "edge_crossing".into(),
            nodes: None,
            edges: Some(vec![e1.clone(), e2.clone()]),
            at: Some((*x, *y)),
            bounds: None,
            fix_hint: Some("Reorder nodes to reduce edge crossings".into()),
        });
    }

    let whitespace_efficiency = geometry::compute_whitespace_efficiency(doc);

    // Label readability
    let total_labels = doc.nodes.len();
    let good_labels = doc
        .nodes
        .iter()
        .filter(|n| !n.label.is_empty() && n.label.len() <= 40)
        .count();
    let label_readability = if total_labels == 0 {
        1.0
    } else {
        good_labels as f64 / total_labels as f64
    };

    let symmetry = compute_symmetry(doc);

    let metrics = Metrics {
        edge_crossings,
        node_overlaps,
        whitespace_efficiency,
        label_readability,
        symmetry,
    };

    let score = scoring::compute_score(&metrics);

    InspectionReport {
        score,
        issues,
        metrics,
    }
}

fn compute_symmetry(doc: &Document) -> f64 {
    let positions: Vec<(f64, f64)> = doc
        .nodes
        .iter()
        .filter_map(|n| n.position.as_ref().map(|p| (p.x, p.y)))
        .collect();

    if positions.len() < 2 {
        return 1.0;
    }

    let cx: f64 = positions.iter().map(|p| p.0).sum::<f64>() / positions.len() as f64;

    let mut left = 0;
    let mut right = 0;
    for (x, _) in &positions {
        if *x < cx {
            left += 1;
        } else {
            right += 1;
        }
    }

    let balance = 1.0 - ((left as f64 - right as f64).abs() / positions.len() as f64);
    balance.clamp(0.0, 1.0)
}
