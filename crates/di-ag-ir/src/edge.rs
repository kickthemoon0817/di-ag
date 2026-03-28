use serde::{Deserialize, Serialize};
use crate::style::{EdgeStyle, Waypoint};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Straight,
    Curved,
    Orthogonal,
}

impl Default for EdgeType {
    fn default() -> Self {
        EdgeType::Straight
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub edge_type: EdgeType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waypoints: Vec<Waypoint>,
    #[serde(default)]
    pub style: EdgeStyle,
}
