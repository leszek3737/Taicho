use std::collections::HashMap;

use futures_util::future::join_all;
use honcho_ai::Honcho;
use honcho_ai::types::message::MessageSearchOptions;
use tracing::{debug, error};

use super::stream;
use crate::actor::commands::{Cmd, StreamEvent};
use taicho::domain::raw_json::{JsonMap, RawJson};
use taicho::domain::{
    ConclusionRow, DomainPage, FileSource, MessageRow, PageInfo, PeerContextView, PeerDetails,
    PeerRow, QueueStatus, ReprOpts, SearchScope, SessionContextView, SessionDetails,
    SessionPeerRow, SessionRow, SessionSummariesView, SummaryKind, SummaryView,
};
use taicho::error::{AppError, AppResult};

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
        | Cmd::ListPeerSessions { .. }
        | Cmd::Chat { .. }
        | Cmd::StreamChat { .. }) => {
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
        | Cmd::SetWorkspaceConfig { .. }
        | Cmd::ScheduleDream { .. }
        | Cmd::QueueStatus { .. }
        | Cmd::Search { .. }) => {
            handle_workspace_cmd(client, cmd).await;
        }
        cmd @ (Cmd::ListConclusions { .. }
        | Cmd::QueryConclusions { .. }
        | Cmd::DeleteConclusion { .. }
        | Cmd::CreateConclusion { .. }
        | Cmd::GetConclusionRepresentation { .. }) => {
            handle_conclusion_cmd(client, cmd).await;
        }
        Cmd::UploadFile { .. } | Cmd::UploadFileToMultiplePeers { .. } => {
            handle_upload_cmd(client, cmd).await;
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
        Cmd::GetPeerRepresentation {
            peer_id,
            opts,
            reply,
        } => {
            let result = get_peer_representation(client, &peer_id, &opts).await;
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
        Cmd::Chat {
            peer_id,
            query,
            opts,
            reply,
        } => {
            let result = peer_chat(client, &peer_id, &query, &opts).await;
            if reply.send(result).is_err() {
                debug!("chat reply receiver dropped");
            }
        }
        Cmd::StreamChat {
            peer_id,
            query,
            opts,
            tx,
        } => {
            handle_stream_chat(client, &peer_id, &query, &opts, tx).await;
        }
        other => {
            error!("unexpected command in handle_peer_cmd");
            drop(other);
        }
    }
}

async fn handle_stream_chat(
    client: &Honcho,
    peer_id: &str,
    query: &str,
    opts: &crate::actor::commands::ChatOpts,
    tx: tokio::sync::mpsc::Sender<StreamEvent>,
) {
    let peer_result = client.peer(peer_id, None, None).await;
    match peer_result {
        Ok(peer) => {
            let mut stream_builder = peer.chat_stream(query);
            if let Some(ref sid) = opts.session_id {
                stream_builder = stream_builder.session(sid.clone());
            }
            if let Some(ref target) = opts.peer_target {
                stream_builder = stream_builder.target(target.clone());
            }
            match stream_builder.send().await {
                Ok(stream) => {
                    tokio::spawn(async move {
                        stream::run_stream(stream, tx).await;
                    });
                }
                Err(e) => {
                    let _ = tx.send(StreamEvent::Err(e.into())).await;
                }
            }
        }
        Err(e) => {
            let _ = tx.send(StreamEvent::Err(e.into())).await;
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

async fn get_peer_representation(
    client: &Honcho,
    peer_id: &str,
    opts: &ReprOpts,
) -> AppResult<String> {
    let peer = client.peer(peer_id, None, None).await?;
    let mut builder = peer.representation_builder();
    if let Some(ref v) = opts.session_id {
        builder = builder.session_id(v);
    }
    if let Some(ref v) = opts.target {
        builder = builder.target(v);
    }
    if let Some(ref v) = opts.search_query {
        builder = builder.search_query(v);
    }
    if let Some(v) = opts.search_top_k {
        builder = builder.search_top_k(v);
    }
    if let Some(v) = opts.search_max_distance {
        builder = builder.search_max_distance(v);
    }
    if let Some(v) = opts.include_most_frequent {
        builder = builder.include_most_frequent(v);
    }
    if let Some(v) = opts.max_conclusions {
        builder = builder.max_conclusions(v);
    }
    Ok(builder.send().await?)
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

async fn peer_chat(
    client: &Honcho,
    peer_id: &str,
    query: &str,
    opts: &crate::actor::commands::ChatOpts,
) -> AppResult<Option<String>> {
    let peer = client.peer(peer_id, None, None).await?;
    let d_opts = honcho_ai::types::dialectic::DialecticOptions::builder()
        .query(query.to_owned())
        .stream(false)
        .maybe_target(opts.peer_target.clone())
        .maybe_session_id(opts.session_id.clone())
        .build();
    d_opts.validate().map_err(AppError::from)?;
    Ok(peer.chat_with_options(&d_opts).await?)
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
        Cmd::ScheduleDream {
            session_id,
            observer_id,
            reply,
        } => {
            let result = async {
                let observer = observer_id.as_deref().ok_or_else(|| {
                    AppError::Validation("observer_id required for schedule_dream".to_owned())
                })?;
                let session_opt = if session_id.is_empty() {
                    None
                } else {
                    Some(session_id.as_str())
                };
                client.schedule_dream(observer, session_opt, None).await?;
                Ok(())
            }
            .await;
            if reply.send(result).is_err() {
                debug!("schedule_dream reply receiver dropped");
            }
        }
        Cmd::QueueStatus { observer_id, reply } => {
            handle_queue_status(client, observer_id, reply).await;
        }
        Cmd::Search {
            scope,
            query,
            limit,
            reply,
        } => {
            handle_search(client, scope, &query, limit, reply).await;
        }
        other => {
            // Routing in handle() prevents this; log if invariant is violated
            error!("unexpected command in handle_workspace_cmd");
            drop(other);
        }
    }
}

async fn handle_search(
    client: &Honcho,
    scope: SearchScope,
    query: &str,
    limit: Option<u32>,
    reply: tokio::sync::oneshot::Sender<AppResult<Vec<MessageRow>>>,
) {
    let result = run_search(client, &scope, query, limit).await;
    if reply.send(result).is_err() {
        debug!("search reply receiver dropped");
    }
}

async fn handle_queue_status(
    client: &Honcho,
    observer_id: Option<String>,
    reply: tokio::sync::oneshot::Sender<AppResult<QueueStatus>>,
) {
    let result = client
        .queue_status(observer_id.as_deref(), None, None)
        .await;
    match result {
        Ok(status) => {
            let session_count = status.sessions.as_ref().map_or(0, |s| s.len() as u64);
            let qs = QueueStatus {
                pending: status.pending_work_units,
                running: status.in_progress_work_units,
                completed: status.completed_work_units,
                sessions: session_count,
                last_updated: Some(chrono::Utc::now()),
            };
            if reply.send(Ok(qs)).is_err() {
                debug!("queue_status reply receiver dropped");
            }
        }
        Err(e) => {
            if reply.send(Err(e.into())).is_err() {
                debug!("queue_status reply receiver dropped");
            }
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

async fn run_search(
    client: &Honcho,
    scope: &SearchScope,
    query: &str,
    limit: Option<u32>,
) -> AppResult<Vec<MessageRow>> {
    let limit_val = limit.unwrap_or(25);
    let msgs = match scope {
        SearchScope::Workspace => client.search(query, Some(limit_val), None).await?,
        SearchScope::Peer(pid) => {
            let peer = client.peer(pid, None, None).await?;
            let opts = MessageSearchOptions {
                query: query.to_owned(),
                filters: None,
                limit: limit_val,
            };
            peer.search_with_options(&opts).await?
        }
        SearchScope::Session(sid) => {
            let session = client.session(sid, None, None, None).await?;
            let opts = MessageSearchOptions {
                query: query.to_owned(),
                filters: None,
                limit: limit_val,
            };
            session.search_with_options(&opts).await?
        }
    };
    Ok(msgs.iter().map(map_message_row).collect())
}

async fn handle_upload_cmd(client: &Honcho, cmd: Cmd) {
    match cmd {
        Cmd::UploadFile {
            session_id,
            peer_id,
            source,
            metadata,
            reply,
        } => {
            let result = async {
                let session = client.session(&session_id, None, None, None).await?;
                let mut builder = session.upload_file(honcho_sdk_file_source(&source));
                builder = builder.peer(&peer_id);
                if let Some(md) = metadata {
                    let val = serde_json::Value::Object(md);
                    builder = builder.metadata(val);
                }
                let msgs = builder.send().await?;
                if let Some(m) = msgs.into_iter().next() {
                    Ok(map_message_row(&m))
                } else {
                    Err(AppError::Validation("no message returned".to_owned()))
                }
            }
            .await;
            let _ = reply.send(result);
        }
        Cmd::UploadFileToMultiplePeers {
            session_id,
            peer_ids,
            source,
            metadata,
            reply,
        } => {
            let result =
                upload_to_multiple_peers(client, &session_id, &peer_ids, &source, metadata).await;
            let _ = reply.send(result);
        }
        other => {
            error!("unexpected command in handle_upload_cmd");
            drop(other);
        }
    }
}

async fn upload_to_multiple_peers(
    client: &Honcho,
    session_id: &str,
    peer_ids: &[String],
    source: &FileSource,
    metadata: Option<JsonMap>,
) -> AppResult<Vec<(String, AppResult<MessageRow>)>> {
    let session = client.session(session_id, None, None, None).await?;
    let futures = peer_ids.iter().map(|pid| {
        let pid = pid.clone();
        let md = metadata.clone();
        let session = session.clone();
        async move {
            let mut builder = session.upload_file(honcho_sdk_file_source(source));
            builder = builder.peer(&pid);
            if let Some(ref m) = md {
                let val = serde_json::Value::Object(m.clone());
                builder = builder.metadata(val);
            }
            let upload_result = async {
                let msgs = builder.send().await?;
                if let Some(m) = msgs.into_iter().next() {
                    Ok(map_message_row(&m))
                } else {
                    Err(AppError::Validation("no message returned".to_owned()))
                }
            }
            .await;
            (pid, upload_result)
        }
    });
    Ok(join_all(futures).await)
}

fn honcho_sdk_file_source(src: &FileSource) -> honcho_ai::FileSource {
    match src {
        FileSource::Path(p) => honcho_ai::FileSource::path(p.clone()),
        FileSource::Bytes { name, mime, data } => {
            honcho_ai::FileSource::bytes(name.clone(), data.clone(), mime.clone())
        }
    }
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
        Cmd::CreateConclusion {
            peer_id,
            observed_id,
            input,
            reply,
        } => {
            let result = create_conclusion(client, &peer_id, observed_id.as_deref(), &input).await;
            if reply.send(result).is_err() {
                debug!("create_conclusion reply receiver dropped");
            }
        }
        Cmd::GetConclusionRepresentation {
            observer_id,
            observed_id,
            reply,
        } => {
            let result = get_conclusion_representation(client, &observer_id, &observed_id).await;
            if reply.send(result).is_err() {
                debug!("get_conclusion_representation reply receiver dropped");
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

async fn create_conclusion(
    client: &Honcho,
    peer_id: &str,
    observed_id: Option<&str>,
    input: &taicho::domain::ConclusionInput,
) -> AppResult<ConclusionRow> {
    let peer = client.peer(peer_id, None, None).await?;
    let scope = match observed_id {
        Some(observed) if observed != peer_id => peer.conclusions_of(observed),
        _ => peer.conclusions(),
    };
    let params = honcho_ai::ConclusionCreateParams::new(&input.content);
    let created = scope.create([params]).await?;
    if let Some(c) = created.into_iter().next() {
        Ok(map_conclusion_from_wrapper(&c))
    } else {
        Err(AppError::Validation("no conclusion returned".to_owned()))
    }
}

async fn get_conclusion_representation(
    client: &Honcho,
    observer_id: &str,
    observed_id: &str,
) -> AppResult<String> {
    let peer = client.peer(observer_id, None, None).await?;
    let scope = if observer_id == observed_id {
        peer.conclusions()
    } else {
        peer.conclusions_of(observed_id)
    };
    Ok(scope.representation().send().await?)
}
