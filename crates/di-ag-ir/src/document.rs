use serde::{Deserialize, Serialize};
use crate::edge::Edge;
use crate::node::Node;
use crate::preset::Preset;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            title: None,
            version: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    #[serde(default)]
    pub metadata: Metadata,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<Preset>,
}

impl Default for Document {
    fn default() -> Self {
        Document {
            metadata: Metadata::default(),
            nodes: vec![],
            edges: vec![],
            preset: None,
        }
    }
}
