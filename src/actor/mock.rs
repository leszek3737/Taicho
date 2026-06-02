#![cfg(feature = "mock-honcho")]
// Mock types are consumed by integration tests via `#[path = "..."] include!`,
// which keeps them out of the crate's normal use graph → suppress dead_code.
#![allow(dead_code, deprecated)]
#![deprecated(note = "Mock module — real mock implementation deferred to v0.2")]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use honcho_ai::Honcho;

pub trait HonchoLike: Send + Sync {
    fn raw(&self) -> &Honcho;
}

pub struct MockHoncho {
    inner: Honcho,
    pub peers: Arc<Mutex<HashMap<String, MockPeer>>>,
}

pub struct MockPeer {
    pub id: String,
    pub metadata: serde_json::Value,
    pub configuration: serde_json::Value,
}

impl MockHoncho {
    pub fn new(inner: Honcho) -> Self {
        Self {
            inner,
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl HonchoLike for MockHoncho {
    fn raw(&self) -> &Honcho {
        &self.inner
    }
}
