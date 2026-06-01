use serde::{Deserialize, Serialize};

use super::raw_json::RawJson;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageRow {
    pub id: String,
    pub workspace_id: String,
    pub session_id: String,
    pub peer_id: String,
    pub content: String,
    pub metadata: RawJson,
    pub created_at: String,
    pub token_count: u64,
}
