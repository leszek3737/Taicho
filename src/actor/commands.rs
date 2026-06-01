use tokio::sync::oneshot;

use taicho::domain::raw_json::{JsonMap, RawJson};
use taicho::domain::{
    ConclusionRow, DomainPage, MessageRow, PeerDetails, PeerRow, SessionDetails, SessionRow,
    SessionSummariesView, WorkspaceInfo,
};
use taicho::error::{AppError, AppResult};
use taicho::persistence::ConnectionProfile;

// Some Cmd variants are not yet wired to UI but are part of the actor protocol
#[allow(dead_code)]
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
}

impl Cmd {
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
        }
    }
}
