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

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr
)]
mod tests {
    use super::*;

    #[test]
    fn peer_context_view_default_empty() {
        let view = PeerContextView {
            peer_id: String::new(),
            target_id: None,
            representation: None,
            peer_card: None,
        };
        assert_eq!(view.peer_id, "");
        assert_eq!(view.target_id, None);
        assert_eq!(view.representation, None);
        assert_eq!(view.peer_card, None);
    }
}
