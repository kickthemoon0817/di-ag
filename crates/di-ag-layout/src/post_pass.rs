use di_ag_ir::{Document, Waypoint};

pub fn run_post_passes(doc: &mut Document) {
    fix_overlaps(doc);
    route_edges(doc);
}

fn fix_overlaps(doc: &mut Document) {
    let node_count = doc.nodes.len();
    for _ in 0..10 {
        let mut moved = false;
        for i in 0..node_count {
            for j in (i + 1)..node_count {
                let (a_x, a_y, a_w, a_h) = get_bounds(&doc.nodes[i]);
                let (b_x, b_y, b_w, b_h) = get_bounds(&doc.nodes[j]);

                let overlap_x = a_x < b_x + b_w && a_x + a_w > b_x;
                let overlap_y = a_y < b_y + b_h && a_y + a_h > b_y;

                if overlap_x && overlap_y {
                    let push = (a_x + a_w + 20.0) - b_x;
                    if let Some(ref mut pos) = doc.nodes[j].position {
                        pos.x += push;
                        moved = true;
                    }
                }
            }
        }
        if !moved {
            break;
        }
    }
}

fn route_edges(doc: &mut Document) {
    // Collect node centers as (center_bottom, center_top) for edge routing
    let node_info: Vec<(String, Option<(f64, f64)>, Option<(f64, f64)>)> = doc
        .nodes
        .iter()
        .map(|n| {
            let bottom = n.position.as_ref().and_then(|pos| {
                n.size
                    .as_ref()
                    .map(|size| (pos.x + size.width / 2.0, pos.y + size.height))
            });
            let top = n.position.as_ref().and_then(|pos| {
                n.size
                    .as_ref()
                    .map(|size| (pos.x + size.width / 2.0, pos.y))
            });
            (n.id.clone(), bottom, top)
        })
        .collect();

    for edge in &mut doc.edges {
        if !edge.waypoints.is_empty() {
            continue;
        }
        let source = node_info.iter().find(|(id, _, _)| *id == edge.source);
        let target = node_info.iter().find(|(id, _, _)| *id == edge.target);

        if let (Some((_, Some((sx, sy)), _)), Some((_, _, Some((tx, ty))))) = (source, target) {
            edge.waypoints = vec![Waypoint { x: *sx, y: *sy }, Waypoint { x: *tx, y: *ty }];
        }
    }
}

fn get_bounds(node: &di_ag_ir::Node) -> (f64, f64, f64, f64) {
    let pos = node
        .position
        .as_ref()
        .map(|p| (p.x, p.y))
        .unwrap_or((0.0, 0.0));
    let size = node
        .size
        .as_ref()
        .map(|s| (s.width, s.height))
        .unwrap_or((80.0, 40.0));
    (pos.0, pos.1, size.0, size.1)
}
