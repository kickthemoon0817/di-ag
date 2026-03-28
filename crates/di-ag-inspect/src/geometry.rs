use di_ag_ir::{Document, Edge, Node};

pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

pub fn node_bbox(node: &Node) -> Option<BBox> {
    let pos = node.position.as_ref()?;
    let size = node.size.as_ref()?;
    Some(BBox {
        x: pos.x,
        y: pos.y,
        w: size.width,
        h: size.height,
    })
}

pub fn bboxes_overlap(a: &BBox, b: &BBox) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

pub fn find_overlapping_pairs(doc: &Document) -> Vec<(String, String)> {
    let mut pairs = vec![];
    let nodes = &doc.nodes;
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            if let (Some(a), Some(b)) = (node_bbox(&nodes[i]), node_bbox(&nodes[j])) {
                if bboxes_overlap(&a, &b) {
                    pairs.push((nodes[i].id.clone(), nodes[j].id.clone()));
                }
            }
        }
    }
    pairs
}

pub fn find_edge_crossings(doc: &Document) -> Vec<(String, String, f64, f64)> {
    let mut crossings = vec![];
    let edges = &doc.edges;

    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            if let Some((x, y)) = edges_cross_at(&edges[i], &edges[j], doc) {
                crossings.push((edges[i].id.clone(), edges[j].id.clone(), x, y));
            }
        }
    }
    crossings
}

fn edges_cross_at(e1: &Edge, e2: &Edge, doc: &Document) -> Option<(f64, f64)> {
    let (a1, a2) = edge_endpoints(e1, doc)?;
    let (b1, b2) = edge_endpoints(e2, doc)?;
    segment_intersection(a1, a2, b1, b2)
}

fn edge_endpoints(e: &Edge, doc: &Document) -> Option<((f64, f64), (f64, f64))> {
    if e.waypoints.len() >= 2 {
        let first = &e.waypoints[0];
        let last = &e.waypoints[e.waypoints.len() - 1];
        return Some(((first.x, first.y), (last.x, last.y)));
    }
    let s = doc.nodes.iter().find(|n| n.id == e.source)?;
    let t = doc.nodes.iter().find(|n| n.id == e.target)?;
    let sp = s.position.as_ref()?;
    let tp = t.position.as_ref()?;
    Some(((sp.x, sp.y), (tp.x, tp.y)))
}

fn segment_intersection(
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    p4: (f64, f64),
) -> Option<(f64, f64)> {
    let d1x = p2.0 - p1.0;
    let d1y = p2.1 - p1.1;
    let d2x = p4.0 - p3.0;
    let d2y = p4.1 - p3.1;

    let denom = d1x * d2y - d1y * d2x;
    if denom.abs() < 1e-10 {
        return None;
    }

    let t = ((p3.0 - p1.0) * d2y - (p3.1 - p1.1) * d2x) / denom;
    let u = ((p3.0 - p1.0) * d1y - (p3.1 - p1.1) * d1x) / denom;

    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        let x = p1.0 + t * d1x;
        let y = p1.1 + t * d1y;
        Some((x, y))
    } else {
        None
    }
}

pub fn compute_whitespace_efficiency(doc: &Document) -> f64 {
    let nodes = &doc.nodes;
    if nodes.is_empty() {
        return 1.0;
    }

    let mut total_node_area = 0.0;
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for node in nodes {
        if let Some(bbox) = node_bbox(node) {
            total_node_area += bbox.w * bbox.h;
            min_x = min_x.min(bbox.x);
            min_y = min_y.min(bbox.y);
            max_x = max_x.max(bbox.x + bbox.w);
            max_y = max_y.max(bbox.y + bbox.h);
        }
    }

    let canvas_area = (max_x - min_x) * (max_y - min_y);
    if canvas_area <= 0.0 {
        return 1.0;
    }

    let ratio = total_node_area / canvas_area;
    let ideal = 0.35;
    1.0 - (ratio - ideal).abs().min(1.0)
}
