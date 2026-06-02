#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr
)]

use taicho::domain::chat::{ChatMessage, ChatRole};
use taicho::domain::search::SearchScope;

#[test]
fn chat_message_construction() {
    let m = ChatMessage {
        peer_id: "alice".into(),
        content: "hi".into(),
        role: ChatRole::User,
        created_at: chrono::Utc::now().to_string(),
        token_count: 1,
    };
    assert_eq!(m.peer_id, "alice");
    assert_eq!(m.role, ChatRole::User);
}

#[test]
fn chat_role_distinct() {
    assert_ne!(ChatRole::User, ChatRole::Assistant);
    assert_ne!(ChatRole::Assistant, ChatRole::System);
    assert_ne!(ChatRole::User, ChatRole::System);
}

#[test]
fn chat_role_unknown_fallback() {
    let role: ChatRole = serde_json::from_str("\"unknown_variant\"").unwrap();
    assert_eq!(role, ChatRole::Unknown);
}

#[test]
fn search_scope_workspace() {
    let s = SearchScope::Workspace;
    assert!(matches!(s, SearchScope::Workspace));
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("Workspace"));
    let deserialized: SearchScope = serde_json::from_str(&json).unwrap();
    assert_eq!(s, deserialized);
}
