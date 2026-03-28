use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Port {
    Top,
    Bottom,
    Left,
    Right,
    Custom(String),
}
