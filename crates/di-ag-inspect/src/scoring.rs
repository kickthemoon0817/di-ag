use crate::report::Metrics;

/// Computes a 0..=100 quality score where defects genuinely depress the result.
///
/// Model:
///   base quality  = weighted mix of whitespace, readability, symmetry (0..=1)
///   overlap_factor  = 0.5 ^ node_overlaps  (every overlap halves the score)
///   crossing_factor = 0.92 ^ edge_crossings
///   score = 100 * base * overlap_factor * crossing_factor
///
/// A perfect layout with zero defects scores ~100; a layout with overlaps
/// collapses fast; a layout with a handful of crossings loses ~8% each.
pub fn compute_score(metrics: &Metrics) -> f64 {
    let base = (metrics.whitespace_efficiency * 0.5
        + metrics.label_readability * 0.3
        + metrics.symmetry * 0.2)
        .clamp(0.0, 1.0);

    // Saturate defect counts before the exponent to prevent a theoretical
    // overflow when u32 → i32 casts a count larger than i32::MAX. At even
    // 64 overlaps the score has already collapsed to effectively zero, so
    // we cap at a comfortably-in-range value.
    const DEFECT_CAP: u32 = 256;
    let overlaps = metrics.node_overlaps.min(DEFECT_CAP) as i32;
    let crossings = metrics.edge_crossings.min(DEFECT_CAP) as i32;

    let overlap_factor = 0.5f64.powi(overlaps);
    let crossing_factor = 0.92f64.powi(crossings);

    let score = 100.0 * base * overlap_factor * crossing_factor;
    score.clamp(0.0, 100.0)
}
