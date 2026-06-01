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
