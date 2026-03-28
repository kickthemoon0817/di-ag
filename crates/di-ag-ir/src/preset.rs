use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagramType {
    Flowchart,
    Sequence,
    Er,
    Class,
    Tree,
    Freeform,
}

impl Default for DiagramType {
    fn default() -> Self {
        DiagramType::Flowchart
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

impl Default for LayoutDirection {
    fn default() -> Self {
        LayoutDirection::TopToBottom
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutConfig {
    #[serde(default)]
    pub direction: LayoutDirection,
    #[serde(default = "default_spacing")]
    pub spacing: f64,
}

fn default_spacing() -> f64 {
    60.0
}

impl Default for LayoutConfig {
    fn default() -> Self {
        LayoutConfig {
            direction: LayoutDirection::default(),
            spacing: default_spacing(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub diagram_type: DiagramType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutConfig>,
}
