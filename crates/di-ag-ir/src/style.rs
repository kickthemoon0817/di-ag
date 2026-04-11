use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Waypoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    #[default]
    Rect,
    RoundedRect,
    Diamond,
    Circle,
    Ellipse,
    Cylinder,
    Parallelogram,
    Hexagon,
    Triangle,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArrowHead {
    #[default]
    Normal,
    Open,
    Diamond,
    Circle,
    None,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EdgeStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dash: Option<String>,
    #[serde(default)]
    pub arrow_head: ArrowHead,
}
