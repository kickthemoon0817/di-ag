use std::collections::HashSet;

use di_ag_ir::{Document, Position};

use crate::LayoutError;

/// Grid-based orthogonal layout. Places nodes on a grid with clean right-angle alignment.
pub fn layout_orthogonal(doc: &mut Document) -> Result<(), LayoutError> {
    let spacing = doc
        .preset
        .as_ref()
        .and_then(|p| p.layout.as_ref())
        .map(|l| l.spacing)
        .unwrap_or(80.0);

    let needs_layout: HashSet<String> = doc
        .nodes
        .iter()
        .filter(|n| n.position.is_none())
        .map(|n| n.id.clone())
        .collect();

    if needs_layout.is_empty() {
        return Ok(());
    }

    // Determine grid dimensions
    let count = needs_layout.len();
    let cols = (count as f64).sqrt().ceil() as usize;

    let node_sizes: Vec<(String, f64, f64)> = doc
        .nodes
        .iter()
        .filter(|n| needs_layout.contains(&n.id))
        .map(|n| {
            let size = n.size.as_ref().map(|s| (s.width, s.height)).unwrap_or((80.0, 40.0));
            (n.id.clone(), size.0, size.1)
        })
        .collect();

    // Find max cell dimensions
    let max_w = node_sizes.iter().map(|(_, w, _)| *w).fold(0.0f64, f64::max);
    let max_h = node_sizes.iter().map(|(_, _, h)| *h).fold(0.0f64, f64::max);

    let cell_w = max_w + spacing;
    let cell_h = max_h + spacing;

    // Place nodes on grid
    for (idx, (id, w, h)) in node_sizes.iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;

        // Center node within cell
        let x = (col as f64) * cell_w + (cell_w - w) / 2.0;
        let y = (row as f64) * cell_h + (cell_h - h) / 2.0;

        if let Some(node) = doc.nodes.iter_mut().find(|n| n.id == *id) {
            node.position = Some(Position { x, y });
        }
    }

    Ok(())
}
