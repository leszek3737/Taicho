use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ReprOpts {
    pub session_id: Option<String>,
    pub target: Option<String>,
    pub search_query: Option<String>,
    pub search_top_k: Option<u32>,
    pub search_max_distance: Option<f64>,
    pub include_most_frequent: Option<bool>,
    pub max_conclusions: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerContextView {
    pub peer_id: String,
    pub target_id: Option<String>,
    pub representation: Option<String>,
    pub peer_card: Option<Vec<String>>,
}

impl PeerContextView {
    fn build_system_content(&self) -> String {
        let mut parts = Vec::new();
        if let Some(repr) = &self.representation {
            parts.push(repr.clone());
        }
        if let Some(card) = &self.peer_card {
            let card_text = card.join(", ");
            parts.push(format!("Peer card: [{card_text}]"));
        }
        if parts.is_empty() {
            String::new()
        } else {
            parts.join("\n\n")
        }
    }

    pub fn to_openai_preview(&self) -> String {
        let system_content = self.build_system_content();
        let messages = if system_content.is_empty() {
            json!([])
        } else {
            json!([
                { "role": "system", "content": system_content }
            ])
        };
        serde_json::to_string_pretty(&messages).unwrap_or_else(|e| format!("JSON error: {e}"))
    }

    pub fn to_anthropic_preview(&self) -> String {
        let system_content = self.build_system_content();
        let obj = if system_content.is_empty() {
            json!({ "system": "", "messages": [] })
        } else {
            json!({
                "system": system_content,
                "messages": [
                    { "role": "user", "content": "..." }
                ]
            })
        };
        serde_json::to_string_pretty(&obj).unwrap_or_else(|e| format!("JSON error: {e}"))
    }
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

    #[test]
    fn repr_opts_default_all_none() {
        let opts = ReprOpts::default();
        assert!(opts.session_id.is_none());
        assert!(opts.target.is_none());
        assert!(opts.search_query.is_none());
        assert!(opts.search_top_k.is_none());
        assert!(opts.search_max_distance.is_none());
        assert!(opts.include_most_frequent.is_none());
        assert!(opts.max_conclusions.is_none());
    }

    #[test]
    fn openai_preview_with_data() {
        let view = PeerContextView {
            peer_id: "p1".into(),
            target_id: Some("t1".into()),
            representation: Some("I am a helpful assistant.".into()),
            peer_card: Some(vec!["friendly".into(), "concise".into()]),
        };
        let json = view.to_openai_preview();
        assert!(json.contains("\"role\": \"system\""));
        assert!(json.contains("I am a helpful assistant."));
        assert!(json.contains("Peer card: [friendly, concise]"));
    }

    #[test]
    fn anthropic_preview_with_data() {
        let view = PeerContextView {
            peer_id: "p1".into(),
            target_id: None,
            representation: Some("I am a helper.".into()),
            peer_card: None,
        };
        let json = view.to_anthropic_preview();
        assert!(json.contains("\"system\": \"I am a helper.\""));
        assert!(json.contains("\"role\": \"user\""));
    }

    #[test]
    fn preview_empty_context() {
        let view = PeerContextView {
            peer_id: "p1".into(),
            target_id: None,
            representation: None,
            peer_card: None,
        };
        assert!(view.to_openai_preview().contains("[]"));
        assert!(view.to_anthropic_preview().contains("\"system\": \"\""));
    }
}
