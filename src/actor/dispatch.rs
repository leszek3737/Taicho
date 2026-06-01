use std::collections::HashMap;

use futures_util::future::join_all;
use honcho_ai::Honcho;
use tracing::{debug, error};

use crate::actor::commands::Cmd;
use taicho::domain::raw_json::{JsonMap, RawJson};
use taicho::domain::{
    ConclusionRow, DomainPage, MessageRow, PageInfo, PeerContextView, PeerDetails, PeerRow,
    SessionContextView, SessionDetails, SessionPeerRow, SessionRow, SessionSummariesView,
    SummaryKind, SummaryView,
};
use taicho::error::AppResult;

/// Convert domain `JsonMap` to the `HashMap` expected by SDK methods.
fn json_map_to_hash_map(map: &JsonMap) -> HashMap<String, serde_json::Value> {
    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

pub async fn handle(client: &Honcho, cmd: Cmd) {
    match cmd {
        Cmd::Refresh { reply } => {
            let result: AppResult<()> = async {
                client.force_ensure().await?;
                Ok(())
            }
            .await;
            if reply.send(result).is_err() {
                debug!("refresh reply receiver dropped");
            }
        }
        cmd @ (Cmd::ListPeers { .. }
        | Cmd::GetPeer { .. }
        | Cmd::SetPeerMetadata { .. }
        | Cmd::SetPeerConfig { .. }
        | Cmd::GetPeerCard { .. }
        | Cmd::SetPeerCard { .. }
        | Cmd::GetPeerRepresentation { .. }
        | Cmd::GetPeerContext { .. }
        | Cmd::ListPeerSessions { .. }) => {
            handle_peer_cmd(client, cmd).await;
        }
        cmd @ (Cmd::ListSessions { .. }
        | Cmd::GetSession { .. }
        | Cmd::SetSessionMetadata { .. }
        | Cmd::SetSessionConfig { .. }
        | Cmd::GetSessionSummaries { .. }
        | Cmd::CloneSession { .. }
        | Cmd::DeleteSession { .. }
        | Cmd::GetSessionContext { .. }) => {
            handle_session_cmd(client, cmd).await;
        }
        cmd @ (Cmd::ListSessionPeers { .. }
        | Cmd::AddSessionPeer { .. }
        | Cmd::RemoveSessionPeer { .. }
        | Cmd::GetSessionPeerConfig { .. }
        | Cmd::SetSessionPeerConfig { .. }) => {
            handle_session_peer_cmd(client, cmd).await;
        }
        cmd @ (Cmd::ListMessages { .. }
        | Cmd::GetMessage { .. }
        | Cmd::UpdateMessageMetadata { .. }) => {
            handle_message_cmd(client, cmd).await;
        }
        cmd @ (Cmd::ListWorkspaces { .. }
        | Cmd::DeleteWorkspace { .. }
        | Cmd::GetWorkspaceMetadata { .. }
        | Cmd::SetWorkspaceMetadata { .. }
        | Cmd::GetWorkspaceConfig { .. }
        | Cmd::SetWorkspaceConfig { .. }) => {
            handle_workspace_cmd(client, cmd).await;
        }
        cmd @ (Cmd::ListConclusions { .. }
        | Cmd::QueryConclusions { .. }
        | Cmd::DeleteConclusion { .. }) => {
            handle_conclusion_cmd(client, cmd).await;
        }
        cmd @ (Cmd::Connect { .. } | Cmd::Disconnect { .. }) => {
            error!("connect/disconnect should not reach dispatch handle");
            cmd.reply_with_error(taicho::error::AppError::Validation(
                "unexpected command in dispatch".to_owned(),
            ));
        }
    }
}

async fn handle_peer_cmd(client: &Honcho, cmd: Cmd) {
    match cmd {
        Cmd::ListPeers { page, size, reply } => {
            let result = list_peers(client, page, size).await;
            if reply.send(result).is_err() {
                debug!("list_peers reply receiver dropped");
            }
        }
        Cmd::GetPeer { peer_id, reply } => {
            let result = get_peer_details(client, &peer_id).await;
            if reply.send(result).is_err() {
                debug!("get_peer reply receiver dropped");
            }
        }
        Cmd::SetPeerMetadata {
            peer_id,
            metadata,
            reply,
        } => {
            let result = set_peer_metadata(client, &peer_id, &metadata).await;
            if reply.send(result).is_err() {
                debug!("set_peer_metadata reply receiver dropped");
            }
        }
        Cmd::SetPeerConfig {
            peer_id,
            configuration,
            reply,
        } => {
            let result = set_peer_config(client, &peer_id, &configuration).await;
            if reply.send(result).is_err() {
                debug!("set_peer_config reply receiver dropped");
            }
        }
        Cmd::GetPeerCard { peer_id, reply } => {
            let result = get_peer_card(client, &peer_id).await;
            if reply.send(result).is_err() {
                debug!("get_peer_card reply receiver dropped");
            }
        }
        Cmd::SetPeerCard {
            peer_id,
            card,
            reply,
        } => {
            let result = set_peer_card(client, &peer_id, card).await;
            if reply.send(result).is_err() {
                debug!("set_peer_card reply receiver dropped");
            }
        }
        Cmd::GetPeerRepresentation { peer_id, reply } => {
            let result = get_peer_representation(client, &peer_id).await;
            if reply.send(result).is_err() {
                debug!("get_peer_representation reply receiver dropped");
            }
        }
        Cmd::GetPeerContext { peer_id, reply } => {
            let result = get_peer_context(client, &peer_id).await;
            if reply.send(result).is_err() {
                debug!("get_peer_context reply receiver dropped");
            }
        }
        Cmd::ListPeerSessions { peer_id, reply } => {
            let result = list_peer_sessions(client, &peer_id).await;
            if reply.send(result).is_err() {
                debug!("list_peer_sessions reply receiver dropped");
            }
        }
        other => {
            error!("unexpected command in handle_peer_cmd");
            drop(other);
        }
    }
}

async fn handle_session_cmd(client: &Honcho, cmd: Cmd) {
    match cmd {
        Cmd::ListSessions { page, size, reply } => {
            let result = list_sessions(client, page, size).await;
            if reply.send(result).is_err() {
                debug!("list_sessions reply receiver dropped");
            }
        }
        Cmd::GetSession { session_id, reply } => {
            let result = get_session_details(client, &session_id).await;
            if reply.send(result).is_err() {
                debug!("get_session reply receiver dropped");
            }
        }
        Cmd::SetSessionMetadata {
            session_id,
            metadata,
            reply,
        } => {
            let result = set_session_metadata(client, &session_id, &metadata).await;
            if reply.send(result).is_err() {
                debug!("set_session_metadata reply receiver dropped");
            }
        }
        Cmd::SetSessionConfig {
            session_id,
            configuration,
            reply,
        } => {
            let result = set_session_config(client, &session_id, &configuration).await;
            if reply.send(result).is_err() {
                debug!("set_session_config reply receiver dropped");
            }
        }
        Cmd::GetSessionSummaries { session_id, reply } => {
            let result = get_session_summaries(client, &session_id).await;
            if reply.send(result).is_err() {
                debug!("get_session_summaries reply receiver dropped");
            }
        }
        Cmd::CloneSession { session_id, reply } => {
            let result = clone_session(client, &session_id).await;
            if reply.send(result).is_err() {
                debug!("clone_session reply receiver dropped");
            }
        }
        Cmd::DeleteSession { session_id, reply } => {
            let result = delete_session(client, &session_id).await;
            if reply.send(result).is_err() {
                debug!("delete_session reply receiver dropped");
            }
        }
        Cmd::GetSessionContext { session_id, reply } => {
            let result = get_session_context(client, &session_id).await;
            if reply.send(result).is_err() {
                debug!("get_session_context reply receiver dropped");
            }
        }
        other => {
            // Routing in handle() prevents this; log if invariant is violated
            error!("unexpected command in handle_session_cmd");
            drop(other);
        }
    }
}

async fn handle_session_peer_cmd(client: &Honcho, cmd: Cmd) {
    match cmd {
        Cmd::ListSessionPeers { session_id, reply } => {
            let result = list_session_peers(client, &session_id).await;
            if reply.send(result).is_err() {
                debug!("list_session_peers reply receiver dropped");
            }
        }
        Cmd::AddSessionPeer {
            session_id,
            peer_id,
            reply,
        } => {
            let result = add_session_peer(client, &session_id, &peer_id).await;
            if reply.send(result).is_err() {
                debug!("add_session_peer reply receiver dropped");
            }
        }
        Cmd::RemoveSessionPeer {
            session_id,
            peer_id,
            reply,
        } => {
            let result = remove_session_peer(client, &session_id, &peer_id).await;
            if reply.send(result).is_err() {
                debug!("remove_session_peer reply receiver dropped");
            }
        }
        Cmd::GetSessionPeerConfig {
            session_id,
            peer_id,
            reply,
        } => {
            let result = get_session_peer_config(client, &session_id, &peer_id).await;
            if reply.send(result).is_err() {
                debug!("get_session_peer_config reply receiver dropped");
            }
        }
        Cmd::SetSessionPeerConfig {
            session_id,
            peer_id,
            observe_me,
            observe_others,
            reply,
        } => {
            let result =
                set_session_peer_config(client, &session_id, &peer_id, observe_me, observe_others)
                    .await;
            if reply.send(result).is_err() {
                debug!("set_session_peer_config reply receiver dropped");
            }
        }
        other => {
            error!("unexpected command in handle_session_peer_cmd");
            drop(other);
        }
    }
}

async fn handle_message_cmd(client: &Honcho, cmd: Cmd) {
    match cmd {
        Cmd::ListMessages {
            session_id,
            page,
            size,
            reply,
        } => {
            let result = list_messages(client, &session_id, page, size).await;
            if reply.send(result).is_err() {
                debug!("list_messages reply receiver dropped");
            }
        }
        Cmd::GetMessage {
            session_id,
            message_id,
            reply,
        } => {
            let result = get_message(client, &session_id, &message_id).await;
            if reply.send(result).is_err() {
                debug!("get_message reply receiver dropped");
            }
        }
        Cmd::UpdateMessageMetadata {
            session_id,
            message_id,
            metadata,
            reply,
        } => {
            let result = update_message_metadata(client, &session_id, &message_id, &metadata).await;
            if reply.send(result).is_err() {
                debug!("update_message_metadata reply receiver dropped");
            }
        }
        other => {
            // Routing in handle() prevents this; log if invariant is violated
            error!("unexpected command in handle_message_cmd");
            drop(other);
        }
    }
}

pub fn reply_not_connected(cmd: Cmd) {
    cmd.reply_with_error(taicho::error::AppError::NotConnected);
}

// ── Peers (M2) ────────────────────────────────────────────────────────────

async fn list_peers(client: &Honcho, page: u64, size: u64) -> AppResult<DomainPage<PeerRow>> {
    let sdk_page = client
        .peers_with_filters(HashMap::new(), page, size, false)
        .await?;
    let workspace_id = client.workspace_id().to_owned();
    Ok(DomainPage {
        items: sdk_page
            .raw_items()
            .iter()
            .map(|p| map_peer_row(p, &workspace_id))
            .collect(),
        info: PageInfo {
            page: sdk_page.page(),
            size: sdk_page.size(),
            total: sdk_page.total(),
            pages: sdk_page.pages(),
            has_next: sdk_page.has_next(),
        },
    })
}

async fn get_peer_details(client: &Honcho, peer_id: &str) -> AppResult<PeerDetails> {
    let peer = client.peer(peer_id, None, None).await?;
    let raw_config = peer.get_configuration_raw().await?;
    let card = peer.get_card().await?;
    let workspace_id = client.workspace_id().to_owned();
    Ok(PeerDetails {
        id: peer.id().to_owned(),
        workspace_id,
        metadata: RawJson::from_hash_map(peer.get_metadata().await?),
        configuration: RawJson::from_hash_map(raw_config),
        card,
        representation: None,
    })
}

async fn set_peer_metadata(client: &Honcho, peer_id: &str, metadata: &JsonMap) -> AppResult<()> {
    let peer = client.peer(peer_id, None, None).await?;
    peer.set_metadata(json_map_to_hash_map(metadata)).await?;
    Ok(())
}

async fn set_peer_config(client: &Honcho, peer_id: &str, configuration: &JsonMap) -> AppResult<()> {
    let peer = client.peer(peer_id, None, None).await?;
    peer.set_configuration_raw(json_map_to_hash_map(configuration))
        .await?;
    Ok(())
}

async fn get_peer_card(client: &Honcho, peer_id: &str) -> AppResult<Option<Vec<String>>> {
    let peer = client.peer(peer_id, None, None).await?;
    Ok(peer.get_card().await?)
}

async fn set_peer_card(
    client: &Honcho,
    peer_id: &str,
    card: Vec<String>,
) -> AppResult<Option<Vec<String>>> {
    let peer = client.peer(peer_id, None, None).await?;
    Ok(peer.set_card(card).await?)
}

async fn get_peer_representation(client: &Honcho, peer_id: &str) -> AppResult<String> {
    let peer = client.peer(peer_id, None, None).await?;
    Ok(peer.representation().await?)
}

async fn get_peer_context(client: &Honcho, peer_id: &str) -> AppResult<PeerContextView> {
    let peer = client.peer(peer_id, None, None).await?;
    let ctx = peer.context().await?;
    Ok(PeerContextView {
        peer_id: ctx.peer_id,
        target_id: Some(ctx.target_id),
        representation: ctx.representation,
        peer_card: ctx.peer_card,
    })
}

async fn list_peer_sessions(client: &Honcho, peer_id: &str) -> AppResult<Vec<SessionRow>> {
    let peer = client.peer(peer_id, None, None).await?;
    let sdk_page = peer.sessions().await?;
    let workspace_id = client.workspace_id().to_owned();
    sdk_page
        .raw_items()
        .iter()
        .map(|s| map_session_row(s, &workspace_id))
        .collect::<AppResult<Vec<_>>>()
}

fn map_peer_row(p: &honcho_ai::types::peer::Peer, workspace_id: &str) -> PeerRow {
    PeerRow {
        id: p.id.clone(),
        workspace_id: workspace_id.to_owned(),
        created_at: p.created_at.to_rfc3339(),
        metadata: RawJson::from_hash_map(p.metadata.clone()),
        configuration: RawJson::from_hash_map(p.configuration.clone()),
    }
}

// ── Sessions (M3) ─────────────────────────────────────────────────────────

async fn list_sessions(client: &Honcho, page: u64, size: u64) -> AppResult<DomainPage<SessionRow>> {
    let sdk_page = client
        .sessions_with_filters(HashMap::new(), page, size, false)
        .await?;
    let workspace_id = client.workspace_id().to_owned();
    Ok(DomainPage {
        items: sdk_page
            .raw_items()
            .iter()
            .map(|s| map_session_row(s, &workspace_id))
            .collect::<AppResult<Vec<_>>>()?,
        info: PageInfo {
            page: sdk_page.page(),
            size: sdk_page.size(),
            total: sdk_page.total(),
            pages: sdk_page.pages(),
            has_next: sdk_page.has_next(),
        },
    })
}

fn map_session_row(
    s: &honcho_ai::types::session::Session,
    workspace_id: &str,
) -> AppResult<SessionRow> {
    Ok(SessionRow {
        id: s.id.clone(),
        workspace_id: workspace_id.to_owned(),
        is_active: s.is_active,
        created_at: s.created_at.to_rfc3339(),
        metadata: RawJson::from_hash_map(s.metadata.clone()),
        configuration: RawJson::from_serialize(&s.configuration)?,
    })
}

async fn get_session_details(client: &Honcho, session_id: &str) -> AppResult<SessionDetails> {
    let session = client.session(session_id, None, None, None).await?;
    let raw_config = session.get_configuration_raw().await?;
    let summaries = session.summaries().await?;
    let workspace_id = client.workspace_id().to_owned();

    let summaries_view = if summaries.short_summary.is_some() || summaries.long_summary.is_some() {
        Some(map_summaries(&summaries))
    } else {
        None
    };

    Ok(SessionDetails {
        id: session.id().to_owned(),
        workspace_id,
        is_active: session.is_active(),
        created_at: session.created_at().to_rfc3339(),
        metadata: RawJson::from_hash_map(session.get_metadata().await?),
        configuration: RawJson::from_hash_map(raw_config),
        summaries: summaries_view,
    })
}

async fn set_session_metadata(
    client: &Honcho,
    session_id: &str,
    metadata: &JsonMap,
) -> AppResult<()> {
    let session = client.session(session_id, None, None, None).await?;
    session.set_metadata(json_map_to_hash_map(metadata)).await?;
    Ok(())
}

async fn set_session_config(
    client: &Honcho,
    session_id: &str,
    configuration: &JsonMap,
) -> AppResult<()> {
    let session = client.session(session_id, None, None, None).await?;
    session
        .set_configuration_raw(json_map_to_hash_map(configuration))
        .await?;
    Ok(())
}

async fn get_session_summaries(
    client: &Honcho,
    session_id: &str,
) -> AppResult<Option<SessionSummariesView>> {
    let session = client.session(session_id, None, None, None).await?;
    let summaries = session.summaries().await?;
    if summaries.short_summary.is_some() || summaries.long_summary.is_some() {
        Ok(Some(map_summaries(&summaries)))
    } else {
        Ok(None)
    }
}

async fn clone_session(client: &Honcho, session_id: &str) -> AppResult<SessionRow> {
    let session = client.session(session_id, None, None, None).await?;
    let cloned = session.clone_session().await?;
    let workspace_id = client.workspace_id().to_owned();
    Ok(SessionRow {
        id: cloned.id().to_owned(),
        workspace_id,
        is_active: cloned.is_active(),
        created_at: cloned.created_at().to_rfc3339(),
        metadata: RawJson::from_hash_map(cloned.get_metadata().await?),
        configuration: RawJson::from_hash_map(cloned.get_configuration_raw().await?),
    })
}

async fn delete_session(client: &Honcho, session_id: &str) -> AppResult<()> {
    let session = client.session(session_id, None, None, None).await?;
    session.delete().await?;
    Ok(())
}

async fn get_session_context(client: &Honcho, session_id: &str) -> AppResult<SessionContextView> {
    let session = client.session(session_id, None, None, None).await?;
    let ctx = session.context().await?;
    Ok(SessionContextView {
        id: ctx.id,
        messages_count: ctx.messages.len(),
        has_summary: ctx.summary.is_some(),
        peer_representation: ctx.peer_representation,
        peer_card: ctx.peer_card,
    })
}

async fn list_session_peers(client: &Honcho, session_id: &str) -> AppResult<Vec<SessionPeerRow>> {
    let session = client.session(session_id, None, None, None).await?;
    let peers = session.peers().await?;
    let configs = join_all(peers.iter().map(|p| {
        let session = &session;
        async move {
            let cfg = session.get_peer_configuration(p.id()).await.ok();
            (p.id().to_owned(), cfg)
        }
    }))
    .await;
    Ok(configs
        .into_iter()
        .map(|(id, cfg)| SessionPeerRow {
            id,
            observe_me: cfg.as_ref().and_then(|c| c.observe_me),
            observe_others: cfg.as_ref().and_then(|c| c.observe_others),
        })
        .collect())
}

async fn add_session_peer(client: &Honcho, session_id: &str, peer_id: &str) -> AppResult<()> {
    let session = client.session(session_id, None, None, None).await?;
    session.add_peer(peer_id).await?;
    Ok(())
}

async fn remove_session_peer(client: &Honcho, session_id: &str, peer_id: &str) -> AppResult<()> {
    let session = client.session(session_id, None, None, None).await?;
    session.remove_peers(&[peer_id.to_owned()]).await?;
    Ok(())
}

async fn get_session_peer_config(
    client: &Honcho,
    session_id: &str,
    peer_id: &str,
) -> AppResult<SessionPeerRow> {
    let session = client.session(session_id, None, None, None).await?;
    let cfg = session.get_peer_configuration(peer_id).await?;
    Ok(SessionPeerRow {
        id: peer_id.to_owned(),
        observe_me: cfg.observe_me,
        observe_others: cfg.observe_others,
    })
}

async fn set_session_peer_config(
    client: &Honcho,
    session_id: &str,
    peer_id: &str,
    observe_me: Option<bool>,
    observe_others: Option<bool>,
) -> AppResult<()> {
    let session = client.session(session_id, None, None, None).await?;
    // SessionPeerConfig is #[non_exhaustive] upstream, so direct struct
    // construction is not possible from outside the defining crate. Round-trip
    // through serde_json to build it.
    let mut cfg_json = serde_json::Map::new();
    if let Some(v) = observe_me {
        cfg_json.insert("observe_me".to_owned(), serde_json::Value::Bool(v));
    }
    if let Some(v) = observe_others {
        cfg_json.insert("observe_others".to_owned(), serde_json::Value::Bool(v));
    }
    let cfg: honcho_ai::types::session::SessionPeerConfig =
        serde_json::from_value(serde_json::Value::Object(cfg_json))?;
    session.set_peer_configuration(peer_id, &cfg).await?;
    Ok(())
}

fn map_summaries(summaries: &honcho_ai::types::session::SessionSummaries) -> SessionSummariesView {
    SessionSummariesView {
        id: summaries.id.clone(),
        short_summary: summaries.short_summary.as_ref().map(map_summary),
        long_summary: summaries.long_summary.as_ref().map(map_summary),
    }
}

fn map_summary(s: &honcho_ai::types::session::Summary) -> SummaryView {
    SummaryView {
        content: s.content.clone(),
        message_id: s.message_id.clone(),
        summary_type: SummaryKind::from_str_lossy(match s.summary_type {
            honcho_ai::types::session::SummaryType::Short => "short",
            honcho_ai::types::session::SummaryType::Long => "long",
            _ => {
                tracing::warn!("unknown summary_type encountered");
                "unknown"
            }
        }),
        created_at: s.created_at.to_rfc3339(),
        token_count: s.token_count,
    }
}

// ── Messages (M3) ─────────────────────────────────────────────────────────

async fn list_messages(
    client: &Honcho,
    session_id: &str,
    page: u64,
    size: u64,
) -> AppResult<DomainPage<MessageRow>> {
    let session = client.session(session_id, None, None, None).await?;
    let sdk_page = session
        .messages_with_options(None, page, size, false)
        .await?;
    Ok(DomainPage {
        items: sdk_page.items().iter().map(map_message_row).collect(),
        info: PageInfo {
            page: sdk_page.page(),
            size: sdk_page.size(),
            total: sdk_page.total(),
            pages: sdk_page.pages(),
            has_next: sdk_page.has_next(),
        },
    })
}

async fn get_message(client: &Honcho, session_id: &str, message_id: &str) -> AppResult<MessageRow> {
    let session = client.session(session_id, None, None, None).await?;
    let msg = session.get_message(message_id).await?;
    Ok(map_message_row(&msg))
}

async fn update_message_metadata(
    client: &Honcho,
    session_id: &str,
    message_id: &str,
    metadata: &JsonMap,
) -> AppResult<MessageRow> {
    let session = client.session(session_id, None, None, None).await?;
    let msg = session
        .update_message(message_id, json_map_to_hash_map(metadata))
        .await?;
    Ok(map_message_row(&msg))
}

fn map_message_row(m: &honcho_ai::Message) -> MessageRow {
    MessageRow {
        id: m.id().to_owned(),
        workspace_id: m.workspace_id().to_owned(),
        session_id: m.session_id().to_owned(),
        peer_id: m.peer_id().to_owned(),
        content: m.content().to_owned(),
        metadata: RawJson::from_hash_map(m.metadata().clone()),
        created_at: m.created_at().to_rfc3339(),
        token_count: m.token_count(),
    }
}

// ── Workspaces (M4) ──────────────────────────────────────────────────────

async fn handle_workspace_cmd(client: &Honcho, cmd: Cmd) {
    match cmd {
        Cmd::ListWorkspaces { reply } => {
            let result = list_workspaces(client).await;
            if reply.send(result).is_err() {
                debug!("list_workspaces reply receiver dropped");
            }
        }
        Cmd::DeleteWorkspace {
            workspace_id,
            reply,
        } => {
            let result = delete_workspace(client, &workspace_id).await;
            if reply.send(result).is_err() {
                debug!("delete_workspace reply receiver dropped");
            }
        }
        Cmd::GetWorkspaceMetadata { reply } => {
            let result = get_workspace_metadata(client).await;
            if reply.send(result).is_err() {
                debug!("get_workspace_metadata reply receiver dropped");
            }
        }
        Cmd::SetWorkspaceMetadata { metadata, reply } => {
            let result = set_workspace_metadata(client, &metadata).await;
            if reply.send(result).is_err() {
                debug!("set_workspace_metadata reply receiver dropped");
            }
        }
        Cmd::GetWorkspaceConfig { reply } => {
            let result = get_workspace_config(client).await;
            if reply.send(result).is_err() {
                debug!("get_workspace_config reply receiver dropped");
            }
        }
        Cmd::SetWorkspaceConfig {
            configuration,
            reply,
        } => {
            let result = set_workspace_config(client, &configuration).await;
            if reply.send(result).is_err() {
                debug!("set_workspace_config reply receiver dropped");
            }
        }
        other => {
            // Routing in handle() prevents this; log if invariant is violated
            error!("unexpected command in handle_workspace_cmd");
            drop(other);
        }
    }
}

async fn list_workspaces(client: &Honcho) -> AppResult<Vec<String>> {
    Ok(client.workspaces().await?.items())
}

async fn delete_workspace(client: &Honcho, workspace_id: &str) -> AppResult<()> {
    client.delete_workspace(workspace_id).await?;
    Ok(())
}

async fn get_workspace_metadata(client: &Honcho) -> AppResult<RawJson> {
    Ok(RawJson::from_hash_map(client.get_metadata().await?))
}

async fn set_workspace_metadata(client: &Honcho, metadata: &JsonMap) -> AppResult<()> {
    client.set_metadata(json_map_to_hash_map(metadata)).await?;
    Ok(())
}

async fn get_workspace_config(client: &Honcho) -> AppResult<RawJson> {
    Ok(RawJson::from_hash_map(
        client.get_configuration_raw().await?,
    ))
}

async fn set_workspace_config(client: &Honcho, configuration: &JsonMap) -> AppResult<()> {
    client
        .set_configuration_raw(json_map_to_hash_map(configuration))
        .await?;
    Ok(())
}

// ── Conclusions (M4) ─────────────────────────────────────────────────────

async fn handle_conclusion_cmd(client: &Honcho, cmd: Cmd) {
    match cmd {
        Cmd::ListConclusions {
            observer_id,
            observed_id,
            page,
            size,
            reply,
        } => {
            let result = list_conclusions(client, &observer_id, &observed_id, page, size).await;
            if reply.send(result).is_err() {
                debug!("list_conclusions reply receiver dropped");
            }
        }
        Cmd::QueryConclusions {
            observer_id,
            observed_id,
            query,
            top_k,
            reply,
        } => {
            let result = query_conclusions(client, &observer_id, &observed_id, &query, top_k).await;
            if reply.send(result).is_err() {
                debug!("query_conclusions reply receiver dropped");
            }
        }
        Cmd::DeleteConclusion {
            conclusion_id,
            observer_id,
            observed_id,
            reply,
        } => {
            let result =
                delete_conclusion(client, &conclusion_id, &observer_id, &observed_id).await;
            if reply.send(result).is_err() {
                debug!("delete_conclusion reply receiver dropped");
            }
        }
        other => {
            // Routing in handle() prevents this; log if invariant is violated
            error!("unexpected command in handle_conclusion_cmd");
            drop(other);
        }
    }
}

async fn list_conclusions(
    client: &Honcho,
    observer_id: &str,
    observed_id: &str,
    page: u64,
    size: u64,
) -> AppResult<DomainPage<ConclusionRow>> {
    let peer = client.peer(observer_id, None, None).await?;
    let scope = if observer_id == observed_id {
        peer.conclusions()
    } else {
        peer.conclusions_of(observed_id)
    };
    let sdk_page = scope
        .list()
        .page(page.try_into().unwrap_or(u32::MAX))
        .size(size.try_into().unwrap_or(u32::MAX))
        .send()
        .await?;
    Ok(DomainPage {
        items: sdk_page
            .raw_items()
            .iter()
            .map(map_conclusion_row)
            .collect(),
        info: PageInfo {
            page: sdk_page.page(),
            size: sdk_page.size(),
            total: sdk_page.total(),
            pages: sdk_page.pages(),
            has_next: sdk_page.has_next(),
        },
    })
}

fn map_conclusion_row(c: &honcho_ai::types::conclusion::Conclusion) -> ConclusionRow {
    ConclusionRow {
        id: c.id.clone(),
        content: c.content.clone(),
        observer_id: c.observer_id.clone(),
        observed_id: c.observed_id.clone(),
        session_id: c.session_id.clone(),
        created_at: c.created_at.to_rfc3339(),
    }
}

async fn query_conclusions(
    client: &Honcho,
    observer_id: &str,
    observed_id: &str,
    query: &str,
    top_k: u32,
) -> AppResult<Vec<ConclusionRow>> {
    let peer = client.peer(observer_id, None, None).await?;
    let scope = if observer_id == observed_id {
        peer.conclusions()
    } else {
        peer.conclusions_of(observed_id)
    };
    let results = scope.query(query).top_k(top_k).send().await?;
    Ok(results.iter().map(map_conclusion_from_wrapper).collect())
}

fn map_conclusion_from_wrapper(c: &honcho_ai::conclusion::Conclusion) -> ConclusionRow {
    ConclusionRow {
        id: c.id().to_owned(),
        content: c.content().to_owned(),
        observer_id: c.observer_id().to_owned(),
        observed_id: c.observed_id().to_owned(),
        session_id: c.session_id().map(|s| s.to_owned()),
        created_at: c.created_at().to_rfc3339(),
    }
}

async fn delete_conclusion(
    client: &Honcho,
    conclusion_id: &str,
    observer_id: &str,
    observed_id: &str,
) -> AppResult<()> {
    let peer = client.peer(observer_id, None, None).await?;
    let scope = if observer_id == observed_id {
        peer.conclusions()
    } else {
        peer.conclusions_of(observed_id)
    };
    scope.delete(conclusion_id).await?;
    Ok(())
}
