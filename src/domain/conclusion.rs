use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConclusionRow {
    pub id: String,
    pub content: String,
    pub observer_id: String,
    pub observed_id: String,
    pub session_id: Option<String>,
    pub created_at: String,
}
