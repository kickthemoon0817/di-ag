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

    // Label readability: graduated penalty on char count
    let total_labels = doc.nodes.len();
    let label_readability = if total_labels == 0 {
        1.0
    } else {
        let sum: f64 = doc
            .nodes
            .iter()
            .map(|n| {
                let len = n.label.chars().count();
                if n.label.is_empty() {
                    0.6
                } else if len <= 40 {
                    1.0
                } else if len <= 80 {
                    // Linear decay from 1.0 to 0.3 over [40, 80]
                    1.0 - (len - 40) as f64 / 40.0 * 0.7
                } else {
                    0.3
                }
            })
            .sum();
        sum / total_labels as f64
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

    // Count left/right of the centroid. Nodes exactly on the axis split
    // evenly so perfectly column-aligned layouts score 1.0.
    let mut left: f64 = 0.0;
    let mut right: f64 = 0.0;
    for (x, _) in &positions {
        if *x < cx - 1e-9 {
            left += 1.0;
        } else if *x > cx + 1e-9 {
            right += 1.0;
        } else {
            left += 0.5;
            right += 0.5;
        }
    }

    let balance = 1.0 - ((left - right).abs() / positions.len() as f64);
    balance.clamp(0.0, 1.0)
}
