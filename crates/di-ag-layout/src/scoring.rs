use di_ag_ir::Document;

pub fn compute_score(doc: &Document) -> f64 {
    let overlap_score = score_overlaps(doc);
    let crossing_score = score_edge_crossings(doc);
    let spacing_score = score_spacing(doc);

    let score = overlap_score * 0.4 + crossing_score * 0.3 + spacing_score * 0.3;
    (score * 100.0).clamp(0.0, 100.0)
}

fn score_overlaps(doc: &Document) -> f64 {
    let nodes = &doc.nodes;
    let mut overlaps = 0;
    let total_pairs = nodes.len() * (nodes.len().saturating_sub(1)) / 2;
    if total_pairs == 0 {
        return 1.0;
    }

    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            if let (Some(a_pos), Some(a_size), Some(b_pos), Some(b_size)) = (
                nodes[i].position.as_ref(),
                nodes[i].size.as_ref(),
                nodes[j].position.as_ref(),
                nodes[j].size.as_ref(),
            ) {
                let ox = a_pos.x < b_pos.x + b_size.width && a_pos.x + a_size.width > b_pos.x;
                let oy =
                    a_pos.y < b_pos.y + b_size.height && a_pos.y + a_size.height > b_pos.y;
                if ox && oy {
                    overlaps += 1;
                }
            }
        }
    }
    1.0 - (overlaps as f64 / total_pairs as f64)
}

fn score_edge_crossings(doc: &Document) -> f64 {
    let edges = &doc.edges;
    if edges.len() < 2 {
        return 1.0;
    }
    let total_pairs = edges.len() * (edges.len() - 1) / 2;
    let mut crossings = 0;

    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            if edges_cross(&edges[i], &edges[j], doc) {
                crossings += 1;
            }
        }
    }
    1.0 - (crossings as f64 / total_pairs as f64).min(1.0)
}

fn edges_cross(e1: &di_ag_ir::Edge, e2: &di_ag_ir::Edge, doc: &Document) -> bool {
    let get_endpoints = |e: &di_ag_ir::Edge| -> Option<((f64, f64), (f64, f64))> {
        let s = doc.nodes.iter().find(|n| n.id == e.source)?;
        let t = doc.nodes.iter().find(|n| n.id == e.target)?;
        let sp = s.position.as_ref()?;
        let tp = t.position.as_ref()?;
        Some(((sp.x, sp.y), (tp.x, tp.y)))
    };

    let (a1, a2) = match get_endpoints(e1) {
        Some(v) => v,
        None => return false,
    };
    let (b1, b2) = match get_endpoints(e2) {
        Some(v) => v,
        None => return false,
    };

    segments_intersect(a1, a2, b1, b2)
}

fn segments_intersect(
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    p4: (f64, f64),
) -> bool {
    let d1 = direction(p3, p4, p1);
    let d2 = direction(p3, p4, p2);
    let d3 = direction(p1, p2, p3);
    let d4 = direction(p1, p2, p4);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }
    false
}

fn direction(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
    (c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)
}

fn score_spacing(doc: &Document) -> f64 {
    let nodes = &doc.nodes;
    if nodes.len() < 2 {
        return 1.0;
    }

    let min_gap = 20.0;
    let mut good = 0;
    let mut total = 0;

    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            if let (Some(a_pos), Some(a_size), Some(b_pos), Some(b_size)) = (
                nodes[i].position.as_ref(),
                nodes[i].size.as_ref(),
                nodes[j].position.as_ref(),
                nodes[j].size.as_ref(),
            ) {
                total += 1;
                let gap_x = (b_pos.x - (a_pos.x + a_size.width))
                    .max(a_pos.x - (b_pos.x + b_size.width));
                let gap_y = (b_pos.y - (a_pos.y + a_size.height))
                    .max(a_pos.y - (b_pos.y + b_size.height));
                if gap_x >= min_gap || gap_y >= min_gap {
                    good += 1;
                }
            }
        }
    }
    if total == 0 {
        1.0
    } else {
        good as f64 / total as f64
    }
}
