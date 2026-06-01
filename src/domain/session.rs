use serde::{Deserialize, Serialize};

use super::raw_json::RawJson;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryKind {
    Short,
    Long,
    Unknown,
}

impl SummaryKind {
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "short" => Self::Short,
            "long" => Self::Long,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::Long => "long",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for SummaryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRow {
    pub id: String,
    pub workspace_id: String,
    pub is_active: bool,
    pub created_at: String,
    pub metadata: RawJson,
    pub configuration: RawJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDetails {
    pub id: String,
    pub workspace_id: String,
    pub is_active: bool,
    pub created_at: String,
    pub metadata: RawJson,
    pub configuration: RawJson,
    pub summaries: Option<SessionSummariesView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummariesView {
    pub id: String,
    pub short_summary: Option<SummaryView>,
    pub long_summary: Option<SummaryView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryView {
    pub content: String,
    pub message_id: String,
    pub summary_type: SummaryKind,
    pub created_at: String,
    pub token_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPeerRow {
    pub id: String,
    pub observe_me: Option<bool>,
    pub observe_others: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionContextView {
    pub id: String,
    pub messages_count: usize,
    pub has_summary: bool,
    pub peer_representation: Option<String>,
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
    fn session_peer_row_constructs_with_some_flags() {
        let row = SessionPeerRow {
            id: "p1".to_string(),
            observe_me: Some(true),
            observe_others: Some(false),
        };
        assert_eq!(row.id, "p1");
        assert_eq!(row.observe_me, Some(true));
        assert_eq!(row.observe_others, Some(false));
    }

    #[test]
    fn session_peer_row_constructs_with_none_flags() {
        let row = SessionPeerRow {
            id: "p2".to_string(),
            observe_me: None,
            observe_others: None,
        };
        assert_eq!(row.id, "p2");
        assert_eq!(row.observe_me, None);
        assert_eq!(row.observe_others, None);
    }

    #[test]
    fn session_peer_row_id_preserved() {
        let row = SessionPeerRow {
            id: "peer-abc-123".to_string(),
            observe_me: Some(true),
            observe_others: Some(true),
        };
        assert_eq!(row.id, "peer-abc-123");
    }

    #[test]
    fn session_context_view_default_empty() {
        let view = SessionContextView {
            id: String::new(),
            messages_count: 0,
            has_summary: false,
            peer_representation: None,
            peer_card: None,
        };
        assert_eq!(view.id, "");
        assert_eq!(view.messages_count, 0);
        assert!(!view.has_summary);
        assert_eq!(view.peer_representation, None);
        assert_eq!(view.peer_card, None);
    }
}
