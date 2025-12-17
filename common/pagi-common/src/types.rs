use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TwinId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinState {
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Default for TwinState {
    fn default() -> Self {
        Self {
            status: "registered".to_string(),
            note: None,
        }
    }
}

