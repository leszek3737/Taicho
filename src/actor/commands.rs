use tokio::sync::mpsc;
use tokio::sync::oneshot;

use taicho::domain::raw_json::{JsonMap, RawJson};
use taicho::domain::{
    ConclusionInput, ConclusionRow, DomainPage, FileSource, MessageRow, PeerContextView,
    PeerDetails, PeerRow, QueueStatus, SearchScope, SessionContextView, SessionDetails,
    SessionPeerRow, SessionRow, SessionSummariesView, WorkspaceInfo,
};
use taicho::error::{AppError, AppResult};
use taicho::persistence::ConnectionProfile;

// Some Cmd variants are not yet wired to UI but are part of the actor protocol
#[allow(dead_code, clippy::too_many_lines)]
pub enum Cmd {
    // --- Connection (M1) ---
    Connect {
        profile: ConnectionProfile,
        api_key: Option<String>,
        reply: oneshot::Sender<AppResult<WorkspaceInfo>>,
    },
    Disconnect {
        reply: oneshot::Sender<AppResult<()>>,
    },
    Refresh {
        reply: oneshot::Sender<AppResult<()>>,
    },

    // --- Peers (M2) ---
    ListPeers {
        page: u64,
        size: u64,
        reply: oneshot::Sender<AppResult<DomainPage<PeerRow>>>,
    },
    GetPeer {
        peer_id: String,
        reply: oneshot::Sender<AppResult<PeerDetails>>,
    },
    SetPeerMetadata {
        peer_id: String,
        metadata: JsonMap,
        reply: oneshot::Sender<AppResult<()>>,
    },
    SetPeerConfig {
        peer_id: String,
        configuration: JsonMap,
        reply: oneshot::Sender<AppResult<()>>,
    },
    GetPeerCard {
        peer_id: String,
        reply: oneshot::Sender<AppResult<Option<Vec<String>>>>,
    },
    SetPeerCard {
        peer_id: String,
        card: Vec<String>,
        reply: oneshot::Sender<AppResult<Option<Vec<String>>>>,
    },
    GetPeerRepresentation {
        peer_id: String,
        reply: oneshot::Sender<AppResult<String>>,
    },

    // --- Peers completion (M2) ---
    GetPeerContext {
        peer_id: String,
        reply: oneshot::Sender<AppResult<PeerContextView>>,
    },
    ListPeerSessions {
        peer_id: String,
        reply: oneshot::Sender<AppResult<Vec<SessionRow>>>,
    },

    // --- Sessions (M3) ---
    ListSessions {
        page: u64,
        size: u64,
        reply: oneshot::Sender<AppResult<DomainPage<SessionRow>>>,
    },
    GetSession {
        session_id: String,
        reply: oneshot::Sender<AppResult<SessionDetails>>,
    },
    SetSessionMetadata {
        session_id: String,
        metadata: JsonMap,
        reply: oneshot::Sender<AppResult<()>>,
    },
    SetSessionConfig {
        session_id: String,
        configuration: JsonMap,
        reply: oneshot::Sender<AppResult<()>>,
    },
    GetSessionSummaries {
        session_id: String,
        reply: oneshot::Sender<AppResult<Option<SessionSummariesView>>>,
    },
    CloneSession {
        session_id: String,
        reply: oneshot::Sender<AppResult<SessionRow>>,
    },
    DeleteSession {
        session_id: String,
        reply: oneshot::Sender<AppResult<()>>,
    },

    // --- Sessions completion (M3) ---
    GetSessionContext {
        session_id: String,
        reply: oneshot::Sender<AppResult<SessionContextView>>,
    },

    // --- Session Peers (M4) ---
    ListSessionPeers {
        session_id: String,
        reply: oneshot::Sender<AppResult<Vec<SessionPeerRow>>>,
    },
    AddSessionPeer {
        session_id: String,
        peer_id: String,
        reply: oneshot::Sender<AppResult<()>>,
    },
    RemoveSessionPeer {
        session_id: String,
        peer_id: String,
        reply: oneshot::Sender<AppResult<()>>,
    },
    GetSessionPeerConfig {
        session_id: String,
        peer_id: String,
        reply: oneshot::Sender<AppResult<SessionPeerRow>>,
    },
    SetSessionPeerConfig {
        session_id: String,
        peer_id: String,
        observe_me: Option<bool>,
        observe_others: Option<bool>,
        reply: oneshot::Sender<AppResult<()>>,
    },

    // --- Messages (M3) ---
    ListMessages {
        session_id: String,
        page: u64,
        size: u64,
        reply: oneshot::Sender<AppResult<DomainPage<MessageRow>>>,
    },
    GetMessage {
        session_id: String,
        message_id: String,
        reply: oneshot::Sender<AppResult<MessageRow>>,
    },
    UpdateMessageMetadata {
        session_id: String,
        message_id: String,
        metadata: JsonMap,
        reply: oneshot::Sender<AppResult<MessageRow>>,
    },

    // --- Workspaces (M4) ---
    ListWorkspaces {
        reply: oneshot::Sender<AppResult<Vec<String>>>,
    },
    DeleteWorkspace {
        workspace_id: String,
        reply: oneshot::Sender<AppResult<()>>,
    },
    GetWorkspaceMetadata {
        reply: oneshot::Sender<AppResult<RawJson>>,
    },
    SetWorkspaceMetadata {
        metadata: JsonMap,
        reply: oneshot::Sender<AppResult<()>>,
    },
    GetWorkspaceConfig {
        reply: oneshot::Sender<AppResult<RawJson>>,
    },
    SetWorkspaceConfig {
        configuration: JsonMap,
        reply: oneshot::Sender<AppResult<()>>,
    },

    // --- Conclusions (M4) ---
    ListConclusions {
        observer_id: String,
        observed_id: String,
        page: u64,
        size: u64,
        reply: oneshot::Sender<AppResult<DomainPage<ConclusionRow>>>,
    },
    QueryConclusions {
        observer_id: String,
        observed_id: String,
        query: String,
        top_k: u32,
        reply: oneshot::Sender<AppResult<Vec<ConclusionRow>>>,
    },
    DeleteConclusion {
        conclusion_id: String,
        observer_id: String,
        observed_id: String,
        reply: oneshot::Sender<AppResult<()>>,
    },

    // --- Chat (M5) ---
    Chat {
        peer_id: String,
        query: String,
        opts: ChatOpts,
        reply: oneshot::Sender<AppResult<Option<String>>>,
    },
    StreamChat {
        peer_id: String,
        query: String,
        opts: ChatOpts,
        tx: mpsc::Sender<StreamEvent>,
    },

    // --- Conclusions (M6) ---
    CreateConclusion {
        peer_id: String,
        observed_id: Option<String>,
        input: ConclusionInput,
        reply: oneshot::Sender<AppResult<ConclusionRow>>,
    },

    // --- Upload (M7) ---
    UploadFile {
        session_id: String,
        peer_id: String,
        source: FileSource,
        metadata: Option<JsonMap>,
        reply: oneshot::Sender<AppResult<MessageRow>>,
    },

    // --- Dreams (M8) ---
    ScheduleDream {
        session_id: String,
        observer_id: Option<String>,
        reply: oneshot::Sender<AppResult<()>>,
    },
    QueueStatus {
        observer_id: Option<String>,
        reply: oneshot::Sender<AppResult<QueueStatus>>,
    },

    // --- Search (M9) ---
    Search {
        scope: SearchScope,
        query: String,
        limit: Option<u32>,
        reply: oneshot::Sender<AppResult<Vec<MessageRow>>>,
    },
}

#[derive(Clone, Debug, Default)]
pub struct ChatOpts {
    pub session_id: Option<String>,
    pub peer_target: Option<String>,
}

#[derive(Debug)]
pub enum StreamEvent {
    Chunk(String),
    Done(String),
    Err(AppError),
}

impl Cmd {
    #[allow(clippy::too_many_lines)]
    pub fn reply_with_error(self, err: AppError) {
        match self {
            Self::Connect { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::Disconnect { reply } => {
                let _ = reply.send(Err(err));
            }
            Self::Refresh { reply } => {
                let _ = reply.send(Err(err));
            }
            Self::ListPeers { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetPeer { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::SetPeerMetadata { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::SetPeerConfig { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetPeerCard { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::SetPeerCard { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetPeerRepresentation { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetPeerContext { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::ListPeerSessions { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::ListSessions { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetSession { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::SetSessionMetadata { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::SetSessionConfig { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetSessionSummaries { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::CloneSession { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::DeleteSession { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetSessionContext { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::ListSessionPeers { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::AddSessionPeer { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::RemoveSessionPeer { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetSessionPeerConfig { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::SetSessionPeerConfig { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::ListMessages { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetMessage { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::UpdateMessageMetadata { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::ListWorkspaces { reply } => {
                let _ = reply.send(Err(err));
            }
            Self::DeleteWorkspace { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetWorkspaceMetadata { reply } => {
                let _ = reply.send(Err(err));
            }
            Self::SetWorkspaceMetadata { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::GetWorkspaceConfig { reply } => {
                let _ = reply.send(Err(err));
            }
            Self::SetWorkspaceConfig { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::ListConclusions { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::QueryConclusions { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::DeleteConclusion { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::Chat { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::StreamChat { tx, .. } => {
                let _ = tx.blocking_send(StreamEvent::Err(err));
            }
            Self::CreateConclusion { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::UploadFile { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::ScheduleDream { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::QueueStatus { reply, .. } => {
                let _ = reply.send(Err(err));
            }
            Self::Search { reply, .. } => {
                let _ = reply.send(Err(err));
            }
        }
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

    macro_rules! assert_reply_err {
        ($rx:expr, $msg:expr) => {{
            let result = $rx.blocking_recv().expect("rx should receive");
            assert!(result.is_err(), "expected Err, got Ok");
            assert_eq!(result.unwrap_err().user_message(), $msg);
        }};
    }

    #[test]
    fn get_peer_context_reply_with_error() {
        let (tx, rx) = oneshot::channel::<AppResult<PeerContextView>>();
        Cmd::GetPeerContext {
            peer_id: String::new(),
            reply: tx,
        }
        .reply_with_error(AppError::Validation("v".to_string()));
        assert_reply_err!(rx, "v");
    }

    #[test]
    fn list_peer_sessions_reply_with_error() {
        let (tx, rx) = oneshot::channel::<AppResult<Vec<SessionRow>>>();
        Cmd::ListPeerSessions {
            peer_id: String::new(),
            reply: tx,
        }
        .reply_with_error(AppError::Validation("v".to_string()));
        assert_reply_err!(rx, "v");
    }

    #[test]
    fn get_session_context_reply_with_error() {
        let (tx, rx) = oneshot::channel::<AppResult<SessionContextView>>();
        Cmd::GetSessionContext {
            session_id: String::new(),
            reply: tx,
        }
        .reply_with_error(AppError::Validation("v".to_string()));
        assert_reply_err!(rx, "v");
    }

    #[test]
    fn list_session_peers_reply_with_error() {
        let (tx, rx) = oneshot::channel::<AppResult<Vec<SessionPeerRow>>>();
        Cmd::ListSessionPeers {
            session_id: String::new(),
            reply: tx,
        }
        .reply_with_error(AppError::Validation("v".to_string()));
        assert_reply_err!(rx, "v");
    }

    #[test]
    fn add_session_peer_reply_with_error() {
        let (tx, rx) = oneshot::channel::<AppResult<()>>();
        Cmd::AddSessionPeer {
            session_id: String::new(),
            peer_id: String::new(),
            reply: tx,
        }
        .reply_with_error(AppError::Validation("v".to_string()));
        assert_reply_err!(rx, "v");
    }

    #[test]
    fn remove_session_peer_reply_with_error() {
        let (tx, rx) = oneshot::channel::<AppResult<()>>();
        Cmd::RemoveSessionPeer {
            session_id: String::new(),
            peer_id: String::new(),
            reply: tx,
        }
        .reply_with_error(AppError::Validation("v".to_string()));
        assert_reply_err!(rx, "v");
    }

    #[test]
    fn get_session_peer_config_reply_with_error() {
        let (tx, rx) = oneshot::channel::<AppResult<SessionPeerRow>>();
        Cmd::GetSessionPeerConfig {
            session_id: String::new(),
            peer_id: String::new(),
            reply: tx,
        }
        .reply_with_error(AppError::Validation("v".to_string()));
        assert_reply_err!(rx, "v");
    }

    #[test]
    fn set_session_peer_config_reply_with_error() {
        let (tx, rx) = oneshot::channel::<AppResult<()>>();
        Cmd::SetSessionPeerConfig {
            session_id: String::new(),
            peer_id: String::new(),
            observe_me: None,
            observe_others: None,
            reply: tx,
        }
        .reply_with_error(AppError::Validation("v".to_string()));
        assert_reply_err!(rx, "v");
    }
}
