#![cfg(feature = "mock-honcho")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr,
    deprecated
)]

#[path = "../src/actor/mock.rs"]
mod mock;

use honcho_ai::Honcho;
use mock::{HonchoLike, MockHoncho, MockPeer};
use serde_json::json;

#[test]
fn mock_honcho_can_be_constructed() {
    let inner = Honcho::new("http://localhost:0", "test").expect("constructor ok");
    let mock = MockHoncho::new(inner);
    assert!(mock.peers.lock().unwrap().is_empty());
}

#[test]
fn mock_honcho_constructed_with_inner() {
    let inner = Honcho::new("http://localhost:0", "ws").expect("constructor ok");
    let mock = MockHoncho::new(inner);
    let raw: &Honcho = mock.raw();
    let _: &Honcho = raw;
    assert!(mock.peers.lock().unwrap().is_empty());
    mock.peers.lock().unwrap().insert(
        "alice".to_owned(),
        MockPeer {
            id: "alice".to_owned(),
            metadata: serde_json::Value::Null,
            configuration: serde_json::Value::Null,
        },
    );
    let peers = mock.peers.lock().unwrap();
    let alice = peers.get("alice").expect("alice present");
    assert_eq!(alice.id, "alice");
    assert!(alice.metadata.is_null());
    assert!(alice.configuration.is_null());
}

#[test]
fn mock_peer_round_trip() {
    let inner = Honcho::new("http://localhost:0", "test").expect("constructor ok");
    let mock = MockHoncho::new(inner);
    {
        let mut peers = mock.peers.lock().unwrap();
        peers.insert(
            "alice".to_owned(),
            MockPeer {
                id: "alice".to_owned(),
                metadata: json!({ "name": "Alice" }),
                configuration: json!({ "tone": "warm" }),
            },
        );
    }
    let peers = mock.peers.lock().unwrap();
    let alice = peers.get("alice").expect("alice present");
    assert_eq!(alice.metadata["name"], "Alice");
    assert_eq!(alice.configuration["tone"], "warm");
}
