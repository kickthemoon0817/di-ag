pub mod force;
pub mod layered;
pub mod orthogonal;
pub mod post_pass;
pub mod scoring;

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use di_ag_ir::{DiagramType, Document, Edge, Node, Preset, Size};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("Layout failed: {0}")]
    Failed(String),
}

pub fn layout(mut doc: Document) -> Result<Document, LayoutError> {
    assign_default_sizes(&mut doc.nodes);

    // Pre-pass: lay out container children in their own local coordinate
    // system and size each container to fit. This runs for every strategy
    // so containers work with layered / force / orthogonal alike.
    let preset_clone = doc.preset.clone();
    let edges_snapshot = doc.edges.clone();
    layout_container_tree(&mut doc.nodes, &edges_snapshot, preset_clone.as_ref())?;

    // Main pass: pick a strategy for the top-level nodes. Containers are
    // treated as opaque sized boxes here; their children are already placed.
    let diagram_type = doc
        .preset
        .as_ref()
        .map(|p| p.diagram_type.clone())
        .unwrap_or(DiagramType::Flowchart);

    match diagram_type {
        DiagramType::Freeform => force::layout_force_directed(&mut doc)?,
        DiagramType::Er | DiagramType::Class => orthogonal::layout_orthogonal(&mut doc)?,
        _ => layered::layout_layered(&mut doc)?,
    }

    // Post-pass: translate container children from local to absolute coords
    // now that each container has its top-level position.
    for node in doc.nodes.iter_mut() {
        if !node.children.is_empty() {
            if let Some(pos) = node.position.clone() {
                translate_descendants_to_absolute((pos.x, pos.y), &mut node.children);
            }
        }
    }

    post_pass::run_post_passes(&mut doc);
    Ok(doc)
}

pub fn score(doc: &Document) -> f64 {
    scoring::compute_score(doc)
}

const CONTAINER_PADDING: f64 = 20.0;
const CONTAINER_LABEL_SPACE: f64 = 28.0;

fn assign_default_sizes(nodes: &mut [Node]) {
    for node in nodes.iter_mut() {
        if node.size.is_none() && node.children.is_empty() {
            let char_width = 9.0;
            let padding = 40.0;
            let char_count = node.label.chars().count();
            let width = (char_count as f64 * char_width + padding).max(80.0);
            let height = 40.0;
            node.size = Some(Size { width, height });
        }
        if !node.children.is_empty() {
            assign_default_sizes(&mut node.children);
        }
    }
}

/// For every node that is a container (has children), lay out its children in
/// their own local coordinate system using the same layout strategy that the
/// document requests. After all descendants have been placed, size the
/// container to fit its content and normalize child positions so that the
/// child bounding box begins at `(CONTAINER_PADDING, CONTAINER_PADDING + label_space)`.
fn layout_container_tree(
    nodes: &mut [Node],
    all_edges: &[Edge],
    preset: Option<&Preset>,
) -> Result<(), LayoutError> {
    for node in nodes.iter_mut() {
        if node.children.is_empty() {
            continue;
        }

        // Recurse grand-children first so the current container's direct
        // children already have correct sizes when we lay them out.
        layout_container_tree(&mut node.children, all_edges, preset)?;

        // Build a sub-document with only the children and edges that live
        // entirely inside this container.
        let child_ids: HashSet<String> = collect_all_ids(&node.children);
        let local_edges: Vec<Edge> = all_edges
            .iter()
            .filter(|e| child_ids.contains(&e.source) && child_ids.contains(&e.target))
            .cloned()
            .collect();

        let mut sub_doc = Document {
            metadata: Default::default(),
            nodes: std::mem::take(&mut node.children),
            edges: local_edges,
            preset: preset.cloned(),
        };

        let sub_dt = sub_doc
            .preset
            .as_ref()
            .map(|p| p.diagram_type.clone())
            .unwrap_or(DiagramType::Flowchart);
        match sub_dt {
            DiagramType::Freeform => force::layout_force_directed(&mut sub_doc)?,
            DiagramType::Er | DiagramType::Class => {
                orthogonal::layout_orthogonal(&mut sub_doc)?
            }
            _ => layered::layout_layered(&mut sub_doc)?,
        }

        node.children = sub_doc.nodes;

        // Compute child bounding box and normalize so it starts at the
        // padded/labelled origin. Grand-children positions are left in local
        // coordinates of their own parent; the final absolute translation
        // happens in `translate_descendants_to_absolute`.
        let (minx, miny, maxx, maxy) = children_bounds(&node.children);
        if minx.is_finite() {
            let dx = CONTAINER_PADDING - minx;
            let dy = CONTAINER_PADDING + CONTAINER_LABEL_SPACE - miny;
            for c in node.children.iter_mut() {
                if let Some(ref mut p) = c.position {
                    p.x += dx;
                    p.y += dy;
                }
            }
            let content_w = maxx - minx;
            let content_h = maxy - miny;
            let w = content_w + CONTAINER_PADDING * 2.0;
            let h = content_h + CONTAINER_PADDING * 2.0 + CONTAINER_LABEL_SPACE;
            node.size = Some(Size {
                width: w.max(80.0),
                height: h.max(60.0),
            });
        } else {
            // Empty container: give it a minimal size.
            node.size = Some(Size { width: 120.0, height: 80.0 });
        }
    }
    Ok(())
}

fn collect_all_ids(nodes: &[Node]) -> HashSet<String> {
    let mut ids = HashSet::new();
    fn visit(nodes: &[Node], ids: &mut HashSet<String>) {
        for n in nodes {
            ids.insert(n.id.clone());
            visit(&n.children, ids);
        }
    }
    visit(nodes, &mut ids);
    ids
}

fn children_bounds(children: &[Node]) -> (f64, f64, f64, f64) {
    let mut minx = f64::INFINITY;
    let mut miny = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut maxy = f64::NEG_INFINITY;
    for c in children {
        if let (Some(pos), Some(size)) = (&c.position, &c.size) {
            minx = minx.min(pos.x);
            miny = miny.min(pos.y);
            maxx = maxx.max(pos.x + size.width);
            maxy = maxy.max(pos.y + size.height);
        }
    }
    (minx, miny, maxx, maxy)
}

/// Recursively translate a subtree from local coordinates (relative to the
/// parent) into absolute coordinates, carrying the accumulated ancestor offset.
fn translate_descendants_to_absolute(parent_abs: (f64, f64), children: &mut [Node]) {
    for c in children {
        if let Some(ref mut pos) = c.position {
            pos.x += parent_abs.0;
            pos.y += parent_abs.1;
            let new_abs = (pos.x, pos.y);
            if !c.children.is_empty() {
                translate_descendants_to_absolute(new_abs, &mut c.children);
            }
        }
    }
}
