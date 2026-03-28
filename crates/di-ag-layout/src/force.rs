use std::collections::HashSet;

use di_ag_ir::{Document, Position};

use crate::LayoutError;

const REPULSION: f64 = 5000.0;
const ATTRACTION: f64 = 0.01;
const DAMPING: f64 = 0.9;
const MIN_DISTANCE: f64 = 1.0;
const ITERATIONS: usize = 200;

pub fn layout_force_directed(doc: &mut Document) -> Result<(), LayoutError> {
    let needs_layout: HashSet<String> = doc
        .nodes
        .iter()
        .filter(|n| n.position.is_none())
        .map(|n| n.id.clone())
        .collect();

    if needs_layout.is_empty() {
        return Ok(());
    }

    // Deterministic initial placement in a circle
    let node_count = doc.nodes.len();
    let radius = (node_count as f64) * 40.0;
    for (i, node) in doc.nodes.iter_mut().enumerate() {
        if node.position.is_none() {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (node_count as f64);
            node.position = Some(Position {
                x: radius * angle.cos(),
                y: radius * angle.sin(),
            });
        }
    }

    // Build edge index
    let edges: Vec<(usize, usize)> = doc
        .edges
        .iter()
        .filter_map(|e| {
            let si = doc.nodes.iter().position(|n| n.id == e.source)?;
            let ti = doc.nodes.iter().position(|n| n.id == e.target)?;
            Some((si, ti))
        })
        .collect();

    // Velocity storage
    let mut vx = vec![0.0f64; node_count];
    let mut vy = vec![0.0f64; node_count];

    for _ in 0..ITERATIONS {
        // Repulsive forces between all pairs
        for i in 0..node_count {
            for j in (i + 1)..node_count {
                let (pi, pj) = get_positions(&doc.nodes, i, j);
                let dx = pi.0 - pj.0;
                let dy = pi.1 - pj.1;
                let dist = (dx * dx + dy * dy).sqrt().max(MIN_DISTANCE);
                let force = REPULSION / (dist * dist);
                let fx = (dx / dist) * force;
                let fy = (dy / dist) * force;

                if needs_layout.contains(&doc.nodes[i].id) {
                    vx[i] += fx;
                    vy[i] += fy;
                }
                if needs_layout.contains(&doc.nodes[j].id) {
                    vx[j] -= fx;
                    vy[j] -= fy;
                }
            }
        }

        // Attractive forces along edges
        for &(si, ti) in &edges {
            let (ps, pt) = get_positions(&doc.nodes, si, ti);
            let dx = ps.0 - pt.0;
            let dy = ps.1 - pt.1;
            let dist = (dx * dx + dy * dy).sqrt().max(MIN_DISTANCE);
            let force = ATTRACTION * dist;
            let fx = (dx / dist) * force;
            let fy = (dy / dist) * force;

            if needs_layout.contains(&doc.nodes[si].id) {
                vx[si] -= fx;
                vy[si] -= fy;
            }
            if needs_layout.contains(&doc.nodes[ti].id) {
                vx[ti] += fx;
                vy[ti] += fy;
            }
        }

        // Apply velocities with damping
        for i in 0..node_count {
            if !needs_layout.contains(&doc.nodes[i].id) {
                continue;
            }
            vx[i] *= DAMPING;
            vy[i] *= DAMPING;
            if let Some(ref mut pos) = doc.nodes[i].position {
                pos.x += vx[i];
                pos.y += vy[i];
            }
        }
    }

    Ok(())
}

fn get_positions(
    nodes: &[di_ag_ir::Node],
    i: usize,
    j: usize,
) -> ((f64, f64), (f64, f64)) {
    let pi = nodes[i]
        .position
        .as_ref()
        .map(|p| (p.x, p.y))
        .unwrap_or((0.0, 0.0));
    let pj = nodes[j]
        .position
        .as_ref()
        .map(|p| (p.x, p.y))
        .unwrap_or((0.0, 0.0));
    (pi, pj)
}
