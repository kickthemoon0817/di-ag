use std::collections::{HashMap, HashSet};

use di_ag_ir::{Document, LayoutDirection, Position};

use crate::LayoutError;

pub fn layout_layered(doc: &mut Document) -> Result<(), LayoutError> {
    let direction = doc
        .preset
        .as_ref()
        .and_then(|p| p.layout.as_ref())
        .map(|l| l.direction.clone())
        .unwrap_or(LayoutDirection::TopToBottom);

    let spacing = doc
        .preset
        .as_ref()
        .and_then(|p| p.layout.as_ref())
        .map(|l| l.spacing)
        .unwrap_or(60.0);

    // Collect nodes that need positioning (no position set)
    let needs_layout: HashSet<String> = doc
        .nodes
        .iter()
        .filter(|n| n.position.is_none())
        .map(|n| n.id.clone())
        .collect();

    if needs_layout.is_empty() {
        return Ok(());
    }

    // Build adjacency from edges
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    for node in &doc.nodes {
        adj.entry(node.id.clone()).or_default();
        in_degree.entry(node.id.clone()).or_insert(0);
    }
    for edge in &doc.edges {
        adj.entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
        *in_degree.entry(edge.target.clone()).or_insert(0) += 1;
    }

    // Assign layers via topological sort (Kahn's algorithm)
    let mut layers: HashMap<String, usize> = HashMap::new();
    let mut queue: Vec<String> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| id.clone())
        .collect();
    queue.sort(); // deterministic ordering

    while let Some(node_id) = queue.pop() {
        let layer = {
            let mut max_pred_layer: Option<usize> = None;
            for edge in &doc.edges {
                if edge.target == node_id {
                    if let Some(&pred_layer) = layers.get(&edge.source) {
                        max_pred_layer = Some(
                            max_pred_layer.map_or(pred_layer, |m: usize| m.max(pred_layer)),
                        );
                    }
                }
            }
            max_pred_layer.map_or(0, |l| l + 1)
        };
        layers.insert(node_id.clone(), layer);

        if let Some(neighbors) = adj.get(&node_id) {
            for neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(neighbor.clone());
                    queue.sort();
                }
            }
        }
    }

    // Handle cycles: any unvisited nodes get assigned to layer 0
    for node in &doc.nodes {
        layers.entry(node.id.clone()).or_insert(0);
    }

    // Group nodes by layer
    let max_layer = layers.values().copied().max().unwrap_or(0);
    let mut layer_groups: Vec<Vec<String>> = vec![vec![]; max_layer + 1];
    for (id, &layer) in &layers {
        layer_groups[layer].push(id.clone());
    }
    for group in &mut layer_groups {
        group.sort();
    }

    // Assign positions
    let node_sizes: HashMap<String, (f64, f64)> = doc
        .nodes
        .iter()
        .filter_map(|n| n.size.as_ref().map(|s| (n.id.clone(), (s.width, s.height))))
        .collect();

    let layer_spacing = spacing + 40.0;
    let node_spacing = spacing;

    for (layer_idx, group) in layer_groups.iter().enumerate() {
        let total_width: f64 = group
            .iter()
            .filter_map(|id| node_sizes.get(id).map(|(w, _)| *w))
            .sum::<f64>()
            + (group.len().saturating_sub(1) as f64) * node_spacing;

        let start_x = -total_width / 2.0;
        let mut current_x = start_x;

        for id in group {
            if !needs_layout.contains(id) {
                continue;
            }
            let (w, _h) = node_sizes.get(id).copied().unwrap_or((80.0, 40.0));

            let (x, y) = match direction {
                LayoutDirection::TopToBottom => (current_x, layer_idx as f64 * layer_spacing),
                LayoutDirection::BottomToTop => (current_x, -(layer_idx as f64) * layer_spacing),
                LayoutDirection::LeftToRight => (layer_idx as f64 * layer_spacing, current_x),
                LayoutDirection::RightToLeft => (-(layer_idx as f64) * layer_spacing, current_x),
            };

            if let Some(node) = doc.nodes.iter_mut().find(|n| n.id == *id) {
                node.position = Some(Position { x, y });
            }

            current_x += w + node_spacing;
        }
    }

    Ok(())
}
