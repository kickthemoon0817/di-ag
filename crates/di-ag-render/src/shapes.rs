use di_ag_ir::{Position, Shape, Size};

pub struct ShapeAttrs {
    pub fill: String,
    pub stroke: String,
    pub stroke_width: f64,
}

pub fn render_shape(shape: &Shape, pos: &Position, size: &Size, attrs: &ShapeAttrs) -> String {
    match shape {
        Shape::Rect => render_rect(pos, size, attrs, 0.0),
        Shape::RoundedRect => render_rect(pos, size, attrs, 8.0),
        Shape::Diamond => render_diamond(pos, size, attrs),
        Shape::Circle => render_circle(pos, size, attrs),
        Shape::Ellipse => render_ellipse(pos, size, attrs),
        Shape::Cylinder => render_cylinder(pos, size, attrs),
        Shape::Parallelogram => render_parallelogram(pos, size, attrs),
        Shape::Hexagon => render_hexagon(pos, size, attrs),
        Shape::Triangle => render_triangle(pos, size, attrs),
    }
}

fn render_rect(pos: &Position, size: &Size, attrs: &ShapeAttrs, rx: f64) -> String {
    format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        pos.x, pos.y, size.width, size.height, rx, attrs.fill, attrs.stroke, attrs.stroke_width
    )
}

fn render_diamond(pos: &Position, size: &Size, attrs: &ShapeAttrs) -> String {
    let cx = pos.x + size.width / 2.0;
    let cy = pos.y + size.height / 2.0;
    let points = format!(
        "{},{} {},{} {},{} {},{}",
        cx,
        pos.y,
        pos.x + size.width,
        cy,
        cx,
        pos.y + size.height,
        pos.x,
        cy
    );
    format!(
        r#"<polygon points="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        points, attrs.fill, attrs.stroke, attrs.stroke_width
    )
}

fn render_circle(pos: &Position, size: &Size, attrs: &ShapeAttrs) -> String {
    let r = size.width.min(size.height) / 2.0;
    let cx = pos.x + size.width / 2.0;
    let cy = pos.y + size.height / 2.0;
    format!(
        r#"<circle cx="{}" cy="{}" r="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        cx, cy, r, attrs.fill, attrs.stroke, attrs.stroke_width
    )
}

fn render_ellipse(pos: &Position, size: &Size, attrs: &ShapeAttrs) -> String {
    let cx = pos.x + size.width / 2.0;
    let cy = pos.y + size.height / 2.0;
    format!(
        r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        cx,
        cy,
        size.width / 2.0,
        size.height / 2.0,
        attrs.fill,
        attrs.stroke,
        attrs.stroke_width
    )
}

fn render_cylinder(pos: &Position, size: &Size, attrs: &ShapeAttrs) -> String {
    let ry = 10.0;
    let x = pos.x;
    let y1 = pos.y + ry;
    let y2 = pos.y + size.height - ry;
    let x2 = pos.x + size.width;
    let rx = size.width / 2.0;
    let cx = pos.x + size.width / 2.0;
    format!(
        r#"<path d="M {} {} L {} {} A {} {} 0 0 0 {} {} L {} {} A {} {} 0 0 0 {} {} Z" fill="{}" stroke="{}" stroke-width="{}"/>
<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        x,
        y1,
        x,
        y2,
        rx,
        ry,
        x2,
        y2,
        x2,
        y1,
        rx,
        ry,
        x,
        y1,
        attrs.fill,
        attrs.stroke,
        attrs.stroke_width,
        cx,
        y1,
        rx,
        ry,
        attrs.fill,
        attrs.stroke,
        attrs.stroke_width
    )
}

fn render_parallelogram(pos: &Position, size: &Size, attrs: &ShapeAttrs) -> String {
    let skew = size.width * 0.15;
    let points = format!(
        "{},{} {},{} {},{} {},{}",
        pos.x + skew,
        pos.y,
        pos.x + size.width,
        pos.y,
        pos.x + size.width - skew,
        pos.y + size.height,
        pos.x,
        pos.y + size.height
    );
    format!(
        r#"<polygon points="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        points, attrs.fill, attrs.stroke, attrs.stroke_width
    )
}

fn render_hexagon(pos: &Position, size: &Size, attrs: &ShapeAttrs) -> String {
    let inset = size.width * 0.2;
    let cy = pos.y + size.height / 2.0;
    let points = format!(
        "{},{} {},{} {},{} {},{} {},{} {},{}",
        pos.x + inset,
        pos.y,
        pos.x + size.width - inset,
        pos.y,
        pos.x + size.width,
        cy,
        pos.x + size.width - inset,
        pos.y + size.height,
        pos.x + inset,
        pos.y + size.height,
        pos.x,
        cy
    );
    format!(
        r#"<polygon points="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        points, attrs.fill, attrs.stroke, attrs.stroke_width
    )
}

fn render_triangle(pos: &Position, size: &Size, attrs: &ShapeAttrs) -> String {
    let cx = pos.x + size.width / 2.0;
    let points = format!(
        "{},{} {},{} {},{}",
        cx,
        pos.y,
        pos.x + size.width,
        pos.y + size.height,
        pos.x,
        pos.y + size.height
    );
    format!(
        r#"<polygon points="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        points, attrs.fill, attrs.stroke, attrs.stroke_width
    )
}
