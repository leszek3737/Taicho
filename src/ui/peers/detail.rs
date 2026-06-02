use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use taicho::domain::{PeerContextView, PeerDetails, ReprOpts, SessionRow};
use taicho::error::{AppError, AppResult};

use super::super::common::json_viewer::JsonViewer;
use super::super::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ContextPreviewMode {
    Raw,
    OpenAi,
    Anthropic,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DetailTab {
    Metadata,
    Configuration,
    Card,
    Representation,
    Context,
    Sessions,
}

#[component]
pub fn PeerDetail() -> Element {
    let state: AppState = use_context();
    let selected_peer_id = state.selection.peer_id.read().clone();

    let Some(peer_id) = selected_peer_id else {
        return rsx! {
            EmptyView {
                title: "Nothing selected".to_string(),
                message: "Select a peer from the list.".to_string(),
            }
        };
    };

    rsx! { PeerDetailInner { peer_id } }
}

#[component]
fn PeerDetailInner(peer_id: String) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut details: Signal<Option<AppResult<PeerDetails>>> = use_signal(|| None);
    let active_tab: Signal<DetailTab> = use_signal(|| DetailTab::Metadata);
    let fetch_retry: Signal<u32> = use_signal(|| 0);

    let peer_id_for_effect = peer_id.clone();
    use_effect(move || {
        let pid = peer_id_for_effect.clone();
        let _v = *fetch_retry.read();
        details.set(None);
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::GetPeer {
            peer_id: pid,
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("get_peer"))
                .and_then(|r| r);
            details.set(Some(result));
        });
    });

    match &*details.read() {
        None => rsx! {
            LoadingView { label: "Loading peer details...".to_string() }
        },
        Some(Err(e)) => rsx! {
            ErrorView {
                code: e.code().to_string(),
                message: e.user_message(),
                retryable: e.is_retryable(),
                on_retry: Some(EventHandler::new({
                    let mut r = fetch_retry;
                    move |_: MouseEvent| {
                        let v = *r.read();
                        r.set(v + 1);
                    }
                })),
            }
        },
        Some(Ok(detail)) => {
            let detail = detail.clone();
            let tab = *active_tab.read();
            rsx! {
                div { class: "detail-content",
                    div { class: "detail-header",
                        h2 { "Peer: {detail.id}" }
                        span { class: "muted", "Workspace: {detail.workspace_id}" }
                    }

                    div { class: "tab-bar",
                        TabButton { label: "Metadata", tab: DetailTab::Metadata, active_tab }
                        TabButton { label: "Configuration", tab: DetailTab::Configuration, active_tab }
                        TabButton { label: "Card", tab: DetailTab::Card, active_tab }
                        TabButton { label: "Representation", tab: DetailTab::Representation, active_tab }
                        TabButton { label: "Context", tab: DetailTab::Context, active_tab }
                        TabButton { label: "Sessions", tab: DetailTab::Sessions, active_tab }
                    }

                    div { class: "tab-content",
                        {match tab {
                            DetailTab::Metadata => rsx! {
                                JsonViewer {
                                    value: serde_json::to_string_pretty(detail.metadata.value())
                                        .unwrap_or_else(|e| format!("JSON error: {e}")),
                                }
                            },
                            DetailTab::Configuration => rsx! {
                                JsonViewer {
                                    value: serde_json::to_string_pretty(detail.configuration.value())
                                        .unwrap_or_else(|e| format!("JSON error: {e}")),
                                }
                            },
                            DetailTab::Card => rsx! {
                                CardTabContent { detail }
                            },
                            DetailTab::Representation => rsx! {
                                RepresentationTabContent {
                                    peer_id: detail.id.clone(),
                                    representation: detail.representation.clone(),
                                }
                            },
                            DetailTab::Context => rsx! {
                                PeerContextTab { peer_id: detail.id.clone() }
                            },
                            DetailTab::Sessions => rsx! {
                                PeerSessionsTab { peer_id: detail.id.clone() }
                            },
                        }}
                    }
                }
            }
        }
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
fn CardTabContent(detail: PeerDetails) -> Element {
    match &detail.card {
        Some(items) if !items.is_empty() => rsx! {
            div { class: "card-tags",
                for tag in items {
                    span { class: "card-tag", "{tag}" }
                }
            }
        },
        _ => rsx! {
            p { class: "muted", "No card set" }
        },
    }
}

#[component]
fn RepresentationTabContent(peer_id: String, representation: Option<String>) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut generated: Signal<Option<String>> = use_signal(|| None);
    let mut gen_error: Signal<Option<(String, String, bool)>> = use_signal(|| None);
    let mut generating: Signal<bool> = use_signal(|| false);
    let mut show_opts: Signal<bool> = use_signal(|| false);
    let mut search_query: Signal<String> = use_signal(String::new);
    let mut search_top_k: Signal<String> = use_signal(String::new);
    let mut search_max_distance: Signal<String> = use_signal(String::new);
    let mut max_conclusions: Signal<String> = use_signal(String::new);
    let mut include_most_frequent: Signal<bool> = use_signal(|| false);

    let build_opts = move || -> ReprOpts {
        ReprOpts {
            session_id: None,
            target: None,
            search_query: {
                let v = search_query.read().trim().to_string();
                if v.is_empty() { None } else { Some(v) }
            },
            search_top_k: search_top_k.read().trim().parse::<u32>().ok(),
            search_max_distance: search_max_distance.read().trim().parse::<f64>().ok(),
            include_most_frequent: {
                let v = *include_most_frequent.read();
                if v { Some(true) } else { None }
            },
            max_conclusions: max_conclusions.read().trim().parse::<u32>().ok(),
        }
    };

    let trigger = {
        move || {
            generating.set(true);
            gen_error.set(None);
            let opts = build_opts();
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::GetPeerRepresentation {
                peer_id: peer_id.clone(),
                opts,
                reply: tx,
            });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("get_peer_representation"))
                    .and_then(|r| r);
                match result {
                    Ok(text) => {
                        generated.set(Some(text));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to generate representation: {e}");
                        gen_error.set(Some((
                            e.code().to_string(),
                            e.user_message(),
                            e.is_retryable(),
                        )));
                    }
                }
                generating.set(false);
            });
        }
    };

    let mut trigger_for_onclick = trigger.clone();
    let trigger_for_retry = trigger.clone();

    match &representation {
        Some(text) => rsx! {
            pre { class: "representation-view", "{text}" }
        },
        None => {
            let is_generating = *generating.read();
            let gen_err = gen_error.read().clone();
            let gen_text = generated.read().clone();
            rsx! {
                div {
                    p { class: "muted", "No representation available" }

                    button {
                        class: "tab-button",
                        onclick: move |_| {
                            let current = *show_opts.read();
                            show_opts.set(!current);
                        },
                        if *show_opts.read() { "Hide Options" } else { "Show Options" }
                    }

                    if *show_opts.read() {
                        div { class: "repr-opts",
                            div { class: "repr-opts-row",
                                label { "Search query" }
                                input {
                                    r#type: "text",
                                    placeholder: "e.g. hobbies",
                                    value: "{search_query.read()}",
                                    oninput: move |e| search_query.set(e.value()),
                                }
                            }
                            div { class: "repr-opts-row",
                                label { "Search top-k" }
                                input {
                                    r#type: "text",
                                    placeholder: "1–100",
                                    value: "{search_top_k.read()}",
                                    oninput: move |e| search_top_k.set(e.value()),
                                }
                            }
                            div { class: "repr-opts-row",
                                label { "Max distance" }
                                input {
                                    r#type: "text",
                                    placeholder: "0.0–1.0",
                                    value: "{search_max_distance.read()}",
                                    oninput: move |e| search_max_distance.set(e.value()),
                                }
                            }
                            div { class: "repr-opts-row",
                                label { "Max conclusions" }
                                input {
                                    r#type: "text",
                                    placeholder: "1–100",
                                    value: "{max_conclusions.read()}",
                                    oninput: move |e| max_conclusions.set(e.value()),
                                }
                            }
                            div { class: "repr-opts-row",
                                label { "Include most frequent" }
                                input {
                                    r#type: "checkbox",
                                    checked: "{include_most_frequent.read()}",
                                    onchange: move |e| include_most_frequent.set(e.checked()),
                                }
                            }
                        }
                    }

                    if is_generating {
                        p { "Generating..." }
                    } else {
                        button {
                            class: "primary-button",
                            onclick: move |_| trigger_for_onclick(),
                            "Generate"
                        }
                    }

                    if let Some(text) = &gen_text {
                        pre { class: "representation-view", "{text}" }
                    }

                    if let Some((code, message, retryable)) = &gen_err {
                        ErrorView {
                            code: code.clone(),
                            message: message.clone(),
                            retryable: *retryable,
                            on_retry: Some(EventHandler::new({
                                let mut trigger = trigger_for_retry.clone();
                                move |_: MouseEvent| trigger()
                            })),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PeerContextTab(peer_id: String) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut context: Signal<Option<AppResult<PeerContextView>>> = use_signal(|| None);
    let mut preview_mode: Signal<ContextPreviewMode> = use_signal(|| ContextPreviewMode::Raw);

    let mut fetch = {
        move || {
            let pid = peer_id.clone();
            context.set(None);
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::GetPeerContext {
                peer_id: pid,
                reply: tx,
            });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("get_peer_context"))
                    .and_then(|r| r);
                context.set(Some(result));
            });
        }
    };

    let fetch_for_retry = fetch.clone();
    use_effect(move || {
        fetch();
    });

    match &*context.read() {
        None => rsx! {
            LoadingView { label: "Loading peer context...".to_string() }
        },
        Some(Err(e)) => rsx! {
            ErrorView {
                code: e.code().to_string(),
                message: e.user_message(),
                retryable: e.is_retryable(),
                on_retry: Some(EventHandler::new({
                    let mut fetch_for_retry = fetch_for_retry.clone();
                    move |_: MouseEvent| fetch_for_retry()
                })),
            }
        },
        Some(Ok(ctx)) => {
            let ctx = ctx.clone();
            let mode = *preview_mode.read();
            rsx! {
                div {
                    div { class: "context-preview-tabs",
                        button {
                            class: if mode == ContextPreviewMode::Raw { "tab-button active" } else { "tab-button" },
                            onclick: move |_| preview_mode.set(ContextPreviewMode::Raw),
                            "Raw"
                        }
                        button {
                            class: if mode == ContextPreviewMode::OpenAi { "tab-button active" } else { "tab-button" },
                            onclick: move |_| preview_mode.set(ContextPreviewMode::OpenAi),
                            "OpenAI"
                        }
                        button {
                            class: if mode == ContextPreviewMode::Anthropic { "tab-button active" } else { "tab-button" },
                            onclick: move |_| preview_mode.set(ContextPreviewMode::Anthropic),
                            "Anthropic"
                        }
                    }

                    {match mode {
                        ContextPreviewMode::Raw => rsx! {
                            div {
                                div { class: "detail-section",
                                    h3 { "Peer ID" }
                                    p { class: "list-item-id", "{ctx.peer_id}" }
                                }
                                if let Some(target) = &ctx.target_id {
                                    div { class: "detail-section",
                                        h3 { "Target ID" }
                                        p { class: "list-item-id", "{target}" }
                                    }
                                }
                                if let Some(repr) = &ctx.representation {
                                    div { class: "detail-section",
                                        h3 { "Representation" }
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
                                if ctx.representation.is_none() && ctx.peer_card.is_none() {
                                    p { class: "muted", "No context data available" }
                                }
                            }
                        },
                        ContextPreviewMode::OpenAi => rsx! {
                            div {
                                div { class: "detail-section",
                                    h3 { "OpenAI Messages Format" }
                                    p { class: "muted", "How context would appear as OpenAI chat messages" }
                                }
                                JsonViewer {
                                    value: ctx.to_openai_preview(),
                                }
                            }
                        },
                        ContextPreviewMode::Anthropic => rsx! {
                            div {
                                div { class: "detail-section",
                                    h3 { "Anthropic Messages Format" }
                                    p { class: "muted", "How context would appear as Anthropic messages with separate system prompt" }
                                }
                                JsonViewer {
                                    value: ctx.to_anthropic_preview(),
                                }
                            }
                        },
                    }}
                }
            }
        }
    }
}

#[component]
fn PeerSessionsTab(peer_id: String) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut sessions: Signal<Option<AppResult<Vec<SessionRow>>>> = use_signal(|| None);

    let fetch = {
        let peer_id = peer_id.clone();
        move || {
            let pid = peer_id.clone();
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::ListPeerSessions {
                peer_id: pid,
                reply: tx,
            });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("list_peer_sessions"))
                    .and_then(|r| r);
                sessions.set(Some(result));
            });
        }
    };

    let fetch_for_retry = fetch.clone();
    use_effect(move || {
        fetch();
    });

    match &*sessions.read() {
        None => rsx! {
            LoadingView { label: "Loading peer sessions...".to_string() }
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
        Some(Ok(list)) => {
            let list = list.clone();
            rsx! {
                div {
                    if list.is_empty() {
                        p { class: "muted", "No sessions found for this peer" }
                    } else {
                        for session in list {
                            {rsx! {
                                div {
                                    key: "{session.id}",
                                    class: "session-peer-row",
                                    div { class: "session-peer-row-header",
                                        span { class: "list-item-id", "{session.id}" }
                                        span {
                                            class: if session.is_active { "badge badge-active" } else { "badge badge-inactive" },
                                            if session.is_active { "Active" } else { "Inactive" }
                                        }
                                    }
                                    p { class: "muted", "{session.created_at}" }
                                }
                            }}
                        }
                    }
                }
            }
        }
    }
}
