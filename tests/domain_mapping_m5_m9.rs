#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr
)]

use taicho::domain::queue::QueueStatus;
use taicho::domain::search::SearchScope;

#[test]
fn queue_status_serializes() {
    let q = QueueStatus {
        pending: 1,
        running: 0,
        completed: 0,
        sessions: 0,
        last_updated: None,
    };
    let json = serde_json::to_string(&q).unwrap();
    assert!(json.contains("\"pending\":1"));
    let deserialized: QueueStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(q, deserialized);
}

#[test]
fn search_scope_workspace_serializes() {
    let s = SearchScope::Workspace;
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("Workspace"));
}

#[test]
fn search_scope_peer_serializes() {
    let s = SearchScope::Peer("alice".into());
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("alice"));
}

#[test]
fn search_scope_session_serializes() {
    let s = SearchScope::Session("sess-1".into());
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("sess-1"));
}
