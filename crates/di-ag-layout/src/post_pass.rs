use std::collections::HashMap;

use di_ag_ir::{Document, LayoutDirection, Node, Waypoint};

pub fn run_post_passes(doc: &mut Document) {
    let direction = doc
        .preset
        .as_ref()
        .and_then(|p| p.layout.as_ref())
        .map(|l| l.direction.clone())
        .unwrap_or(LayoutDirection::TopToBottom);
    fix_overlaps(doc, &direction);
    route_edges(doc, &direction);
}

fn fix_overlaps(doc: &mut Document, direction: &LayoutDirection) {
    // Only resolve overlaps at the top level. Container children were already
    // laid out in a local subgraph and are positioned with enough spacing there.
    let is_horizontal = matches!(
        direction,
        LayoutDirection::LeftToRight | LayoutDirection::RightToLeft
    );
    let node_count = doc.nodes.len();
    for _ in 0..20 {
        let mut moved = false;
        for i in 0..node_count {
            for j in (i + 1)..node_count {
                let a = get_bounds(&doc.nodes[i]);
                let b = get_bounds(&doc.nodes[j]);
                if a.is_none() || b.is_none() {
                    continue;
                }
                let (ax, ay, aw, ah) = a.unwrap();
                let (bx, by, bw, bh) = b.unwrap();
                let overlap_x = ax < bx + bw && ax + aw > bx;
                let overlap_y = ay < by + bh && ay + ah > by;
                if overlap_x && overlap_y {
                    if is_horizontal {
                        // Push the lower of the two down along Y (cross axis for LR/RL).
                        let push = (ay + ah + 20.0) - by;
                        if let Some(ref mut p) = doc.nodes[j].position {
                            p.y += push;
                            moved = true;
                        }
                    } else {
                        // Push the right of the two further right along X (cross axis for TB/BT).
                        let push = (ax + aw + 20.0) - bx;
                        if let Some(ref mut p) = doc.nodes[j].position {
                            p.x += push;
                            moved = true;
                        }
                    }
                }
            }
        }
        if !moved {
            break;
        }
    }
}

fn route_edges(doc: &mut Document, direction: &LayoutDirection) {
    // Build a flat id -> (x, y, w, h) map that sees into container children.
    let mut nodemap: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();
    fn visit(nodes: &[Node], map: &mut HashMap<String, (f64, f64, f64, f64)>) {
        for n in nodes {
            if let (Some(pos), Some(size)) = (&n.position, &n.size) {
                map.insert(n.id.clone(), (pos.x, pos.y, size.width, size.height));
            }
            visit(&n.children, map);
        }
    }
    visit(&doc.nodes, &mut nodemap);

    for edge in &mut doc.edges {
        if !edge.waypoints.is_empty() {
            continue;
        }
        let src = match nodemap.get(&edge.source) {
            Some(v) => *v,
            None => continue,
        };
        let tgt = match nodemap.get(&edge.target) {
            Some(v) => *v,
            None => continue,
        };
        let (s_pt, t_pt) = anchor_points(direction, src, tgt);
        edge.waypoints = vec![
            Waypoint { x: s_pt.0, y: s_pt.1 },
            Waypoint { x: t_pt.0, y: t_pt.1 },
        ];
    }
}

fn anchor_points(
    direction: &LayoutDirection,
    s: (f64, f64, f64, f64),
    t: (f64, f64, f64, f64),
) -> ((f64, f64), (f64, f64)) {
    let (sx, sy, sw, sh) = s;
    let (tx, ty, tw, th) = t;
    let s_cx = sx + sw / 2.0;
    let s_cy = sy + sh / 2.0;
    let t_cx = tx + tw / 2.0;
    let t_cy = ty + th / 2.0;

    // Choose exit/enter sides based on the layout direction and the relative
    // position of source vs target, so reverse edges do not look broken.
    match direction {
        LayoutDirection::TopToBottom | LayoutDirection::BottomToTop => {
            let source_below_target = s_cy > t_cy;
            let (s_pt, t_pt) = if source_below_target {
                ((s_cx, sy), (t_cx, ty + th))
            } else {
                ((s_cx, sy + sh), (t_cx, ty))
            };
            (s_pt, t_pt)
        }
        LayoutDirection::LeftToRight | LayoutDirection::RightToLeft => {
            let source_right_of_target = s_cx > t_cx;
            let (s_pt, t_pt) = if source_right_of_target {
                ((sx, s_cy), (tx + tw, t_cy))
            } else {
                ((sx + sw, s_cy), (tx, t_cy))
            };
            (s_pt, t_pt)
        }
    }
}

fn get_bounds(node: &Node) -> Option<(f64, f64, f64, f64)> {
    let pos = node.position.as_ref()?;
    let size = node.size.as_ref()?;
    Some((pos.x, pos.y, size.width, size.height))
}
