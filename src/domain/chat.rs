use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub peer_id: String,
    pub content: String,
    pub role: ChatRole,
    pub created_at: String,
    pub token_count: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn role_equality() {
        assert_eq!(ChatRole::User, ChatRole::User);
        assert_ne!(ChatRole::User, ChatRole::Assistant);
    }
}
