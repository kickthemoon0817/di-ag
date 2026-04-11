pub mod force;
pub mod layered;
pub mod orthogonal;
pub mod post_pass;
pub mod scoring;

#[cfg(test)]
mod tests;

use di_ag_ir::{DiagramType, Document};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("Layout failed: {0}")]
    Failed(String),
}

pub fn layout(mut doc: Document) -> Result<Document, LayoutError> {
    assign_default_sizes(&mut doc.nodes);

    // Select layout strategy based on preset
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

    post_pass::run_post_passes(&mut doc);
    Ok(doc)
}

pub fn score(doc: &Document) -> f64 {
    scoring::compute_score(doc)
}

fn assign_default_sizes(nodes: &mut [di_ag_ir::Node]) {
    for node in nodes.iter_mut() {
        if node.size.is_none() {
            let char_width = 9.0;
            let padding = 40.0;
            let width = (node.label.len() as f64 * char_width + padding).max(80.0);
            let height = 40.0;
            node.size = Some(di_ag_ir::Size { width, height });
        }
        if !node.children.is_empty() {
            assign_default_sizes(&mut node.children);
        }
    }
}
