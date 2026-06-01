use serde::{Deserialize, Serialize};

use super::raw_json::RawJson;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerRow {
    pub id: String,
    pub workspace_id: String,
    pub created_at: String,
    pub metadata: RawJson,
    pub configuration: RawJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerDetails {
    pub id: String,
    pub workspace_id: String,
    pub metadata: RawJson,
    pub configuration: RawJson,
    pub card: Option<Vec<String>>,
    pub representation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerContextView {
    pub peer_id: String,
    pub target_id: Option<String>,
    pub representation: Option<String>,
    pub peer_card: Option<Vec<String>>,
}
