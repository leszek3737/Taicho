use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchScope {
    Workspace,
    Peer(String),
    Session(String),
}
