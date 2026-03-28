use crate::report::Metrics;

pub fn compute_score(metrics: &Metrics) -> f64 {
    let overlap_penalty = (metrics.node_overlaps as f64) * 15.0;
    let crossing_penalty = (metrics.edge_crossings as f64) * 5.0;
    let whitespace_bonus = metrics.whitespace_efficiency * 20.0;
    let readability_bonus = metrics.label_readability * 15.0;
    let symmetry_bonus = metrics.symmetry * 10.0;

    let score =
        100.0 - overlap_penalty - crossing_penalty + whitespace_bonus + readability_bonus
            + symmetry_bonus;
    score.clamp(0.0, 100.0)
}
