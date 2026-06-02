use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::{AppState, ToastKind};
use taicho::domain::raw_json::JsonMap;
use taicho::domain::{SessionContextView, SessionDetails, SessionPeerRow};
use taicho::error::{AppError, AppResult};

use super::super::common::confirm_modal::ConfirmModal;
use super::super::common::json_editor::JsonEditor;
use super::super::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone, Copy, PartialEq, Eq)]
enum DetailTab {
    Overview,
    Metadata,
    Configuration,
    Summaries,
    Actions,
    Peers,
    Context,
}

#[component]
pub fn SessionDetail() -> Element {
    let state: AppState = use_context();
    let selected_session_id = state.selection.session_id.read().clone();

    let Some(session_id) = selected_session_id else {
        return rsx! {
            EmptyView {
                title: "Nothing selected".to_string(),
                message: "Select a session from the list.".to_string(),
            }
        };
    };

    rsx! { SessionDetailInner { session_id } }
}

#[component]
fn SessionDetailInner(session_id: String) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut details: Signal<Option<AppResult<SessionDetails>>> = use_signal(|| None);
    let active_tab: Signal<DetailTab> = use_signal(|| DetailTab::Overview);
    let mut confirm_delete: Signal<bool> = use_signal(|| false);
    let mut fetch_retry: Signal<u32> = use_signal(|| 0);
    let session_id_for_refresh = session_id.clone();

    // Dioxus 0.7: spawned futures are cancelled on component unmount (drop of the scope).
    // No manual cancellation token needed — navigating away drops the rx automatically.
    use_effect(move || {
        let _retry_token = *fetch_retry.read();
        let sid = session_id.clone();
        details.set(None);
        confirm_delete.set(false);
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::GetSession {
            session_id: sid,
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("get_session"))
                .and_then(|r| r);
            details.set(Some(result));
        });
    });

    let refresh_details = {
        let session_id = session_id_for_refresh.clone();
        let mut details = details;
        move || {
            let sid = session_id.clone();
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::GetSession {
                session_id: sid,
                reply: tx,
            });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("get_session"))
                    .and_then(|r| r);
                details.set(Some(result));
            });
        }
    };

    match &*details.read() {
        None => rsx! {
            LoadingView { label: "Loading session details...".to_string() }
        },
        Some(Err(e)) => rsx! {
            ErrorView {
                code: e.code().to_string(),
                message: e.user_message(),
                retryable: e.is_retryable(),
                on_retry: Some(EventHandler::new(move |_: MouseEvent| {
                    let next = *fetch_retry.read() + 1;
                    fetch_retry.set(next);
                })),
            }
        },
        Some(Ok(detail)) => {
            let detail = detail.clone();
            let tab = *active_tab.read();
            rsx! {
                div { class: "detail-content",
                    div { class: "detail-header",
                        h2 { "Session: {detail.id}" }
                        span { class: "muted", "Workspace: {detail.workspace_id}" }
                    }

                    div { class: "tab-bar",
                        TabButton { label: "Overview", tab: DetailTab::Overview, active_tab }
                        TabButton { label: "Metadata", tab: DetailTab::Metadata, active_tab }
                        TabButton { label: "Configuration", tab: DetailTab::Configuration, active_tab }
                        TabButton { label: "Summaries", tab: DetailTab::Summaries, active_tab }
                        TabButton { label: "Actions", tab: DetailTab::Actions, active_tab }
                        TabButton { label: "Peers", tab: DetailTab::Peers, active_tab }
                        TabButton { label: "Context", tab: DetailTab::Context, active_tab }
                    }

                    div { class: "tab-content",
                        {match tab {
                            DetailTab::Overview => rsx! {
                                OverviewTabContent { detail: detail.clone() }
                            },
                            DetailTab::Metadata => rsx! {
                                MetadataTabContent {
                                    session_id: detail.id.clone(),
                                    initial: detail.metadata.to_json_map().unwrap_or_default(),
                                    on_saved: EventHandler::new(move |_: ()| refresh_details()),
                                }
                            },
                            DetailTab::Configuration => rsx! {
                                ConfigurationTabContent {
                                    session_id: detail.id.clone(),
                                    initial: detail.configuration.to_json_map().unwrap_or_default(),
                                    on_saved: EventHandler::new(move |_: ()| refresh_details()),
                                }
                            },
                            DetailTab::Summaries => rsx! {
                                SummariesTabContent { detail: detail.clone() }
                            },
                            DetailTab::Actions => rsx! {
                                ActionsTabContent { detail, confirm_delete }
                            },
                            DetailTab::Peers => rsx! {
                                SessionPeersTab { session_id: detail.id.clone() }
                            },
                            DetailTab::Context => rsx! {
                                SessionContextTab { session_id: detail.id.clone() }
                            },
                        }}
                    }
                }
            }
        }
    }
}

#[component]
fn SessionPeersTab(session_id: String) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let state: AppState = use_context();
    let mut peers: Signal<Option<AppResult<Vec<SessionPeerRow>>>> = use_signal(|| None);
    let mut add_peer_id: Signal<String> = use_signal(String::new);
    let confirm_remove: Signal<Option<String>> = use_signal(|| None);

    let fetch_peers = {
        let session_id = session_id.clone();
        move || {
            let sid = session_id.clone();
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::ListSessionPeers {
                session_id: sid,
                reply: tx,
            });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("list_session_peers"))
                    .and_then(|r| r);
                peers.set(Some(result));
            });
        }
    };

    let fetch_for_effect = fetch_peers.clone();
    let fetch_for_retry = fetch_peers.clone();
    use_effect(move || {
        fetch_for_effect();
    });

    match &*peers.read() {
        None => rsx! {
            LoadingView { label: "Loading session peers...".to_string() }
        },
        Some(Err(e)) => rsx! {
            ErrorView {
                code: e.code().to_string(),
                message: e.user_message(),
                retryable: e.is_retryable(),
                on_retry: Some(EventHandler::new({
                    let fetch_for_retry = fetch_for_retry.clone();
                    move |_: MouseEvent| fetch_for_retry()
                })),
            }
        },
        Some(Ok(peer_list)) => {
            let peer_list = peer_list.clone();
            rsx! {
                div {
                    div { class: "field-group",
                        input {
                            r#type: "text",
                            placeholder: "Peer ID to add",
                            value: "{add_peer_id}",
                            oninput: move |e| add_peer_id.set(e.value()),
                        }
                        button {
                            class: "primary-button",
                            disabled: add_peer_id.read().is_empty(),
                            onclick: {
                                let session_id = session_id.clone();
                                let fetch = fetch_peers.clone();
                                move |_| {
                                    let pid = add_peer_id.read().clone();
                                    if pid.is_empty() { return; }
                                    let sid = session_id.clone();
                                    let fetch = fetch.clone();
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    actor.send(Cmd::AddSessionPeer {
                                        session_id: sid,
                                        peer_id: pid.clone(),
                                        reply: tx,
                                    });
                                    spawn(async move {
                                        match rx.await {
                                            Ok(Ok(())) => {
                                                fetch();
                                            }
                                            Ok(Err(e)) => {
                                                tracing::warn!("Add peer failed: {e}");
                                                state.push_toast(
                                                    ToastKind::Error,
                                                    format!(
                                                        "Failed to add peer: {}",
                                                        e.user_message()
                                                    ),
                                                );
                                            }
                                            Err(_) => {
                                                state.push_toast(
                                                    ToastKind::Error,
                                                    "Failed to add peer: channel closed",
                                                );
                                            }
                                        }
                                    });
                                    add_peer_id.set(String::new());
                                }
                            },
                            "Add Peer"
                        }
                    }

                    if peer_list.is_empty() {
                        p { class: "muted", "No peers in this session" }
                    } else {
                    for peer in peer_list {
                        {rsx! {
                            SessionPeerRowView {
                                key: "{peer.id}",
                                session_id: session_id.clone(),
                                peer,
                                confirm_remove,
                                on_refresh: fetch_peers.clone(),
                            }
                        }}
                    }
                    }
                }
            }
        }
    }
}

#[component]
fn SessionPeerRowView(
    session_id: String,
    peer: SessionPeerRow,
    confirm_remove: Signal<Option<String>>,
    on_refresh: EventHandler<()>,
) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut observe_me: Signal<Option<bool>> = use_signal(|| peer.observe_me);
    let mut observe_others: Signal<Option<bool>> = use_signal(|| peer.observe_others);
    let mut is_pending: Signal<bool> = use_signal(|| false);
    let is_confirming = confirm_remove.read().as_deref() == Some(&peer.id);
    let state: AppState = use_context();

    rsx! {
        div { class: "session-peer-row",
            div { class: "session-peer-row-header",
                span { class: "list-item-id", "{peer.id}" }
                div { class: "session-peer-toggles",
                    label { class: "toggle-label",
                        input {
                            r#type: "checkbox",
                            checked: observe_me.read().unwrap_or(false),
                            disabled: *is_pending.read(),
                            onchange: {
                                let session_id = session_id.clone();
                                let peer_id = peer.id.clone();
                                move |_| {
                                    if *is_pending.read() { return; }
                                    let current = *observe_me.read();
                                    let new_val = Some(!current.unwrap_or(false));
                                    let old_val = current;
                                    observe_me.set(new_val);
                                    is_pending.set(true);
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    actor.send(Cmd::SetSessionPeerConfig {
                                        session_id: session_id.clone(),
                                        peer_id: peer_id.clone(),
                                        observe_me: new_val,
                                        observe_others: *observe_others.read(),
                                        reply: tx,
                                    });
                                    spawn(async move {
                                        match rx.await {
                                            Ok(Ok(())) => {}
                                            Ok(Err(e)) => {
                                                tracing::warn!("SetSessionPeerConfig failed: {e}");
                                                observe_me.set(old_val);
                                                state.push_toast(
                                                    ToastKind::Error,
                                                    format!("Failed to update config: {}", e.user_message()),
                                                );
                                            }
                                            Err(_) => {
                                                observe_me.set(old_val);
                                                state.push_toast(
                                                    ToastKind::Error,
                                                    "Failed to update config: channel closed",
                                                );
                                            }
                                        }
                                        is_pending.set(false);
                                    });
                                }
                            },
                        }
                        "Observe me"
                    }
                    label { class: "toggle-label",
                        input {
                            r#type: "checkbox",
                            checked: observe_others.read().unwrap_or(false),
                            disabled: *is_pending.read(),
                            onchange: {
                                let session_id = session_id.clone();
                                let peer_id = peer.id.clone();
                                move |_| {
                                    if *is_pending.read() { return; }
                                    let current = *observe_others.read();
                                    let new_val = Some(!current.unwrap_or(false));
                                    let old_val = current;
                                    observe_others.set(new_val);
                                    is_pending.set(true);
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    actor.send(Cmd::SetSessionPeerConfig {
                                        session_id: session_id.clone(),
                                        peer_id: peer_id.clone(),
                                        observe_me: *observe_me.read(),
                                        observe_others: new_val,
                                        reply: tx,
                                    });
                                    spawn(async move {
                                        match rx.await {
                                            Ok(Ok(())) => {}
                                            Ok(Err(e)) => {
                                                tracing::warn!("SetSessionPeerConfig failed: {e}");
                                                observe_others.set(old_val);
                                                state.push_toast(
                                                    ToastKind::Error,
                                                    format!("Failed to update config: {}", e.user_message()),
                                                );
                                            }
                                            Err(_) => {
                                                observe_others.set(old_val);
                                                state.push_toast(
                                                    ToastKind::Error,
                                                    "Failed to update config: channel closed",
                                                );
                                            }
                                        }
                                        is_pending.set(false);
                                    });
                                }
                            },
                        }
                        "Observe others"
                    }
                }
                if is_confirming {
                    span { class: "confirm-bar",
                        "Remove?"
                        button {
                            class: "danger-button",
                            onclick: {
                                let session_id = session_id.clone();
                                let peer_id = peer.id.clone();
                                move |_| {
                                    let sid = session_id.clone();
                                    let pid = peer_id.clone();
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    actor.send(Cmd::RemoveSessionPeer {
                                        session_id: sid,
                                        peer_id: pid.clone(),
                                        reply: tx,
                                    });
                                    let on_refresh = on_refresh;
                                    spawn(async move {
                                        match rx.await {
                                            Ok(Ok(())) => {
                                                on_refresh.call(());
                                            }
                                            Ok(Err(e)) => {
                                                state.push_toast(
                                                    ToastKind::Error,
                                                    format!("Failed to remove peer: {}", e.user_message()),
                                                );
                                            }
                                            Err(_) => {
                                                state.push_toast(
                                                    ToastKind::Error,
                                                    "Failed to remove peer: channel closed",
                                                );
                                            }
                                        }
                                    });
                                    confirm_remove.set(None);
                                }
                            },
                            "Yes"
                        }
                        button {
                            class: "secondary-button",
                            onclick: move |_| confirm_remove.set(None),
                            "No"
                        }
                    }
                } else {
                    button {
                        class: "danger-button",
                        onclick: {
                            let peer_id = peer.id.clone();
                            move |_| confirm_remove.set(Some(peer_id.clone()))
                        },
                        "Remove"
                    }
                }
            }
        }
    }
}

#[component]
fn SessionContextTab(session_id: String) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut context: Signal<Option<AppResult<SessionContextView>>> = use_signal(|| None);
    let mut fetch_retry: Signal<u32> = use_signal(|| 0);

    use_effect(move || {
        let _retry_token = *fetch_retry.read();
        let sid = session_id.clone();
        context.set(None);
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::GetSessionContext {
            session_id: sid,
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("get_session_context"))
                .and_then(|r| r);
            context.set(Some(result));
        });
    });

    match &*context.read() {
        None => rsx! {
            LoadingView { label: "Loading session context...".to_string() }
        },
        Some(Err(e)) => rsx! {
            ErrorView {
                code: e.code().to_string(),
                message: e.user_message(),
                retryable: e.is_retryable(),
                on_retry: Some(EventHandler::new(move |_: MouseEvent| {
                    let next = *fetch_retry.read() + 1;
                    fetch_retry.set(next);
                })),
            }
        },
        Some(Ok(ctx)) => rsx! {
            div {
                div { class: "detail-section",
                    h3 { "Session ID" }
                    p { class: "list-item-id", "{ctx.id}" }
                }
                div { class: "detail-section",
                    h3 { "Messages Count" }
                    p { "{ctx.messages_count}" }
                }
                div { class: "detail-section",
                    h3 { "Has Summary" }
                    span {
                        class: if ctx.has_summary { "badge badge-active" } else { "badge badge-inactive" },
                        if ctx.has_summary { "Yes" } else { "No" }
                    }
                }
                if let Some(repr) = &ctx.peer_representation {
                    div { class: "detail-section",
                        h3 { "Peer Representation" }
                        pre { class: "representation-view", "{repr}" }
                    }
                }
                if let Some(card) = &ctx.peer_card {
                    div { class: "detail-section",
                        h3 { "Peer Card" }
                        div { class: "card-tags",
                            for tag in card {
                                span { class: "card-tag", "{tag}" }
                            }
                        }
                    }
                }
            }
        },
    }
}

#[component]
fn TabButton(label: String, tab: DetailTab, active_tab: Signal<DetailTab>) -> Element {
    let is_active = *active_tab.read() == tab;
    rsx! {
        button {
            class: if is_active { "tab-button active" } else { "tab-button" },
            onclick: move |_| active_tab.set(tab),
            "{label}"
        }
    }
}

#[component]
fn OverviewTabContent(detail: SessionDetails) -> Element {
    rsx! {
        div {
            div { class: "detail-section",
                h3 { "ID" }
                p { class: "list-item-id", "{detail.id}" }
            }
            div { class: "detail-section",
                h3 { "Status" }
                span {
                    class: if detail.is_active { "badge badge-active" } else { "badge badge-inactive" },
                    if detail.is_active { "Active" } else { "Inactive" }
                }
            }
            div { class: "detail-section",
                h3 { "Created" }
                p { "{detail.created_at}" }
            }
            div { class: "detail-section",
                h3 { "Workspace" }
                p { class: "list-item-id", "{detail.workspace_id}" }
            }
        }
    }
}

#[component]
fn SummariesTabContent(detail: SessionDetails) -> Element {
    match &detail.summaries {
        Some(summaries) => rsx! {
            div {
                if let Some(short) = &summaries.short_summary {
                    div { class: "detail-section",
                        h3 { "Short Summary" }
                        p { "{short.content}" }
                        p { class: "muted", "Tokens: {short.token_count}" }
                    }
                }
                if let Some(long) = &summaries.long_summary {
                    div { class: "detail-section",
                        h3 { "Long Summary" }
                        p { "{long.content}" }
                        p { class: "muted", "Tokens: {long.token_count}" }
                    }
                }
                if summaries.short_summary.is_none() && summaries.long_summary.is_none() {
                    p { class: "muted", "No summary content available yet" }
                }
            }
        },
        None => rsx! {
            p { class: "muted", "No summaries available yet" }
        },
    }
}

#[component]
fn MetadataTabContent(session_id: String, initial: JsonMap, on_saved: EventHandler<()>) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let state: AppState = use_context();
    let session_id_for_save = session_id.clone();
    let saving_metadata: Signal<bool> = use_signal(|| false);

    rsx! {
        JsonEditor {
            initial: Some(initial),
            label: "Metadata".to_string(),
            saving: *saving_metadata.read(),
            on_change: move |new_metadata: JsonMap| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                actor.send(Cmd::SetSessionMetadata {
                    session_id: session_id_for_save.clone(),
                    metadata: new_metadata,
                    reply: tx,
                });
                spawn(async move {
                    match rx.await {
                        Ok(Ok(())) => {
                            state.push_toast(ToastKind::Info, "Metadata saved");
                            on_saved.call(());
                        }
                        Ok(Err(e)) => {
                            state.push_toast(
                                ToastKind::Error,
                                format!("Save failed: {}", e.user_message()),
                            );
                        }
                        Err(_) => {
                            state.push_toast(ToastKind::Error, "Save cancelled");
                        }
                    }
                });
            },
            on_cancel: move |_| {},
        }
    }
}

#[component]
fn ConfigurationTabContent(
    session_id: String,
    initial: JsonMap,
    on_saved: EventHandler<()>,
) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let state: AppState = use_context();
    let session_id_for_save = session_id.clone();
    let saving_config: Signal<bool> = use_signal(|| false);

    rsx! {
        JsonEditor {
            initial: Some(initial),
            label: "Configuration".to_string(),
            saving: *saving_config.read(),
            on_change: move |new_config: JsonMap| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                actor.send(Cmd::SetSessionConfig {
                    session_id: session_id_for_save.clone(),
                    configuration: new_config,
                    reply: tx,
                });
                spawn(async move {
                    match rx.await {
                        Ok(Ok(())) => {
                            state.push_toast(ToastKind::Info, "Configuration saved");
                            on_saved.call(());
                        }
                        Ok(Err(e)) => {
                            state.push_toast(
                                ToastKind::Error,
                                format!("Save failed: {}", e.user_message()),
                            );
                        }
                        Err(_) => {
                            state.push_toast(ToastKind::Error, "Save cancelled");
                        }
                    }
                });
            },
            on_cancel: move |_| {},
        }
    }
}

#[component]
fn ActionsTabContent(detail: SessionDetails, confirm_delete: Signal<bool>) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut state: AppState = use_context();

    rsx! {
        div { class: "actions-bar",
            button {
                class: "secondary-button",
                onclick: {
                    let session_id = detail.id.clone();
                    move |_| {
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        actor.send(Cmd::CloneSession {
                            session_id: session_id.clone(),
                            reply: tx,
                        });
                        spawn(async move {
                            match rx.await {
                                Ok(Ok(row)) => {
                                    state.push_toast(
                                        ToastKind::Info,
                                        format!("Session cloned \u{2192} {}", row.id),
                                    );
                                    state.selection.session_id.set(Some(row.id));
                                }
                                Ok(Err(e)) => {
                                    state.push_toast(
                                        ToastKind::Error,
                                        format!("Clone failed: {}", e.user_message()),
                                    );
                                }
                                Err(_) => {
                                    state.push_toast(
                                        ToastKind::Error,
                                        "Clone cancelled (channel closed)",
                                    );
                                }
                            }
                        });
                    }
                },
                "Clone Session"
            }

            if *confirm_delete.read() {
                ConfirmModal {
                    title: "Delete Session".to_string(),
                    message: "Are you sure? This cannot be undone.".to_string(),
                    expected: detail.id.clone(),
                    confirm_label: "Delete Session".to_string(),
                    on_confirm: {
                        let session_id = detail.id.clone();
                        move |_| {
                            let (tx, rx) = tokio::sync::oneshot::channel();
                            actor.send(Cmd::DeleteSession {
                                session_id: session_id.clone(),
                                reply: tx,
                            });
                            spawn(async move {
                                match rx.await {
                                    Ok(Ok(())) => {
                                        state.selection.session_id.set(None);
                                        state.push_toast(
                                            ToastKind::Info,
                                            "Session deleted",
                                        );
                                    }
                                    Ok(Err(e)) => {
                                        state.push_toast(
                                            ToastKind::Error,
                                            format!("Delete failed: {}", e.user_message()),
                                        );
                                    }
                                    Err(_) => {
                                        state.push_toast(
                                            ToastKind::Error,
                                            "Delete cancelled (channel closed)",
                                        );
                                    }
                                }
                            });
                            confirm_delete.set(false);
                        }
                    },
                    on_cancel: move |_| confirm_delete.set(false),
                }
            } else {
                button {
                    class: "danger-button",
                    onclick: move |_| confirm_delete.set(true),
                    "Delete Session"
                }
            }
        }
    }
}
