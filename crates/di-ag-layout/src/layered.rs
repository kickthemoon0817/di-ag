use std::collections::{HashMap, HashSet, VecDeque};

use di_ag_ir::{Document, Edge, LayoutDirection, Node, Position};

use crate::LayoutError;

#[derive(Clone, Copy)]
enum PrimaryAxis {
    Y,
    X,
}

fn axis_of(direction: &LayoutDirection) -> PrimaryAxis {
    match direction {
        LayoutDirection::TopToBottom | LayoutDirection::BottomToTop => PrimaryAxis::Y,
        LayoutDirection::LeftToRight | LayoutDirection::RightToLeft => PrimaryAxis::X,
    }
}

fn primary_extent(axis: PrimaryAxis, w: f64, h: f64) -> f64 {
    match axis {
        PrimaryAxis::Y => h,
        PrimaryAxis::X => w,
    }
}

fn cross_extent(axis: PrimaryAxis, w: f64, h: f64) -> f64 {
    match axis {
        PrimaryAxis::Y => w,
        PrimaryAxis::X => h,
    }
}

fn to_position(direction: &LayoutDirection, primary: f64, cross: f64) -> Position {
    match direction {
        LayoutDirection::TopToBottom => Position { x: cross, y: primary },
        LayoutDirection::BottomToTop => Position { x: cross, y: -primary },
        LayoutDirection::LeftToRight => Position { x: primary, y: cross },
        LayoutDirection::RightToLeft => Position { x: -primary, y: cross },
    }
}

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

    layout_level(&mut doc.nodes, &doc.edges, &direction, spacing)
}

fn layout_level(
    nodes: &mut [Node],
    edges: &[Edge],
    direction: &LayoutDirection,
    spacing: f64,
) -> Result<(), LayoutError> {
    let axis = axis_of(direction);

    let needs_layout: HashSet<String> = nodes
        .iter()
        .filter(|n| n.position.is_none())
        .map(|n| n.id.clone())
        .collect();
    if needs_layout.is_empty() {
        return Ok(());
    }

    // Only consider edges whose endpoints are at this level (ignore self-loops).
    let node_ids: HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let local_edges: Vec<&Edge> = edges
        .iter()
        .filter(|e| {
            node_ids.contains(&e.source)
                && node_ids.contains(&e.target)
                && e.source != e.target
        })
        .collect();

    // Build adjacency.
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut rev_adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    for n in nodes.iter() {
        adj.entry(n.id.clone()).or_default();
        rev_adj.entry(n.id.clone()).or_default();
        in_degree.entry(n.id.clone()).or_insert(0);
    }
    for e in &local_edges {
        adj.get_mut(&e.source).unwrap().push(e.target.clone());
        rev_adj.get_mut(&e.target).unwrap().push(e.source.clone());
        *in_degree.get_mut(&e.target).unwrap() += 1;
    }

    // Undirected connected components for cross-axis grouping.
    let components = compute_components(nodes, &adj, &rev_adj);

    // Kahn's algorithm (FIFO) for longest-path layering.
    let mut layers: HashMap<String, usize> = HashMap::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut initial_roots: Vec<String> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(id, _)| id.clone())
        .collect();
    initial_roots.sort();
    for r in initial_roots {
        queue.push_back(r);
    }
    let mut deg = in_degree.clone();
    while let Some(id) = queue.pop_front() {
        let layer = rev_adj
            .get(&id)
            .map(|preds| {
                preds
                    .iter()
                    .filter_map(|p| layers.get(p).copied())
                    .max()
                    .map_or(0, |m| m + 1)
            })
            .unwrap_or(0);
        layers.insert(id.clone(), layer);

        if let Some(succs) = adj.get(&id).cloned() {
            let mut newly_ready: Vec<String> = Vec::new();
            for s in &succs {
                if let Some(d) = deg.get_mut(s) {
                    if *d > 0 {
                        *d -= 1;
                        if *d == 0 {
                            newly_ready.push(s.clone());
                        }
                    }
                }
            }
            newly_ready.sort();
            for s in newly_ready {
                queue.push_back(s);
            }
        }
    }

    // Cycle handling: unplaced nodes go into one extra layer past the tail.
    let placed_max = layers.values().copied().max().unwrap_or(0);
    let mut unplaced: Vec<String> = nodes
        .iter()
        .map(|n| n.id.clone())
        .filter(|id| !layers.contains_key(id))
        .collect();
    unplaced.sort();
    if !unplaced.is_empty() {
        let cycle_layer = if layers.is_empty() { 0 } else { placed_max + 1 };
        for id in &unplaced {
            layers.insert(id.clone(), cycle_layer);
        }
    }

    // Group nodes by layer; sort within-layer by (component, id) for stable,
    // component-grouped ordering (poor man's crossing minimization).
    let max_layer = layers.values().copied().max().unwrap_or(0);
    let mut layer_groups: Vec<Vec<String>> = vec![Vec::new(); max_layer + 1];
    for (id, &layer) in &layers {
        layer_groups[layer].push(id.clone());
    }
    for group in &mut layer_groups {
        group.sort_by(|a, b| {
            let ca = components.get(a).copied().unwrap_or(0);
            let cb = components.get(b).copied().unwrap_or(0);
            (ca, a).cmp(&(cb, b))
        });
    }

    // Per-node size.
    let mut sizes: HashMap<String, (f64, f64)> = HashMap::new();
    for n in nodes.iter() {
        let s = n
            .size
            .as_ref()
            .map(|s| (s.width, s.height))
            .unwrap_or((80.0, 40.0));
        sizes.insert(n.id.clone(), s);
    }

    // Primary-axis extent per layer and running sum for layer start positions.
    let layer_gap = spacing;
    let mut layer_primary_extent: Vec<f64> = vec![0.0; layer_groups.len()];
    for (i, group) in layer_groups.iter().enumerate() {
        let mut max_p = 0.0f64;
        for id in group {
            if let Some(&(w, h)) = sizes.get(id) {
                max_p = max_p.max(primary_extent(axis, w, h));
            }
        }
        layer_primary_extent[i] = max_p;
    }
    let mut layer_primary_start: Vec<f64> = vec![0.0; layer_groups.len()];
    let mut cum = 0.0;
    for (i, ext) in layer_primary_extent.iter().enumerate() {
        layer_primary_start[i] = cum;
        cum += ext + layer_gap;
    }

    // Compute each node's (primary, cross) position, centering each layer.
    let node_spacing = spacing;
    let component_gap = spacing * 1.5;
    let mut computed: HashMap<String, (f64, f64)> = HashMap::new();
    for (layer_idx, group) in layer_groups.iter().enumerate() {
        let mut extents: Vec<(String, f64, usize)> = Vec::with_capacity(group.len());
        for id in group {
            let (w, h) = sizes.get(id).copied().unwrap_or((80.0, 40.0));
            let ce = cross_extent(axis, w, h);
            let comp = components.get(id).copied().unwrap_or(0);
            extents.push((id.clone(), ce, comp));
        }

        // Total cross length including gaps.
        let mut total = 0.0f64;
        let mut prev_comp: Option<usize> = None;
        for (i, (_, ce, comp)) in extents.iter().enumerate() {
            if i > 0 {
                total += if prev_comp.map_or(false, |pc| pc != *comp) {
                    component_gap
                } else {
                    node_spacing
                };
            }
            total += *ce;
            prev_comp = Some(*comp);
        }

        let mut cursor = -total / 2.0;
        let mut prev_comp: Option<usize> = None;
        for (i, (id, ce, comp)) in extents.iter().enumerate() {
            if i > 0 {
                cursor += if prev_comp.map_or(false, |pc| pc != *comp) {
                    component_gap
                } else {
                    node_spacing
                };
            }
            computed.insert(id.clone(), (layer_primary_start[layer_idx], cursor));
            cursor += *ce;
            prev_comp = Some(*comp);
        }
    }

    // Apply positions.
    for node in nodes.iter_mut() {
        if !needs_layout.contains(&node.id) {
            continue;
        }
        if let Some(&(primary, cross)) = computed.get(&node.id) {
            node.position = Some(to_position(direction, primary, cross));
        }
    }

    Ok(())
}

fn compute_components(
    nodes: &[Node],
    adj: &HashMap<String, Vec<String>>,
    rev_adj: &HashMap<String, Vec<String>>,
) -> HashMap<String, usize> {
    let mut comp: HashMap<String, usize> = HashMap::new();
    let mut next_id = 0usize;
    let mut ordered: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
    ordered.sort();
    for start in &ordered {
        if comp.contains_key(start) {
            continue;
        }
        let id = next_id;
        next_id += 1;
        let mut stack = vec![start.clone()];
        while let Some(cur) = stack.pop() {
            if comp.contains_key(&cur) {
                continue;
            }
            comp.insert(cur.clone(), id);
            if let Some(ns) = adj.get(&cur) {
                for n in ns {
                    if !comp.contains_key(n) {
                        stack.push(n.clone());
                    }
                }
            }
            if let Some(ns) = rev_adj.get(&cur) {
                for n in ns {
                    if !comp.contains_key(n) {
                        stack.push(n.clone());
                    }
                }
            }
        }
    }
    comp
}
