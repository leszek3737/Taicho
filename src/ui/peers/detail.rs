use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use taicho::domain::PeerDetails;
use taicho::error::{AppError, AppResult};

use super::super::common::json_viewer::JsonViewer;
use super::super::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone, Copy, PartialEq, Eq)]
enum DetailTab {
    Metadata,
    Configuration,
    Card,
    Representation,
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

    use_effect(move || {
        let pid = peer_id.clone();
        details.set(None);
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::GetPeer {
            peer_id: pid,
            reply: tx,
        });
        // NOTE: In Dioxus 0.7, setting a signal on an unmounted component emits a
        // runtime warning but does not crash. True cancellation would require
        // `use_resource`, which re-fires on signal deps but cannot accept external
        // params like `peer_id` prop without extra wrapping. Keeping spawn is the
        // pragmatic choice here.
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
                on_retry: None,
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
        Some(_) | None => rsx! {
            p { class: "muted", "No card set" }
        },
    }
}

#[component]
fn RepresentationTabContent(peer_id: String, representation: Option<String>) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut generated: Signal<Option<String>> = use_signal(|| None);
    let mut gen_error: Signal<Option<(String, String)>> = use_signal(|| None);
    let mut generating: Signal<bool> = use_signal(|| false);

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

                    if is_generating {
                        p { "Generating..." }
                    } else {
                        button {
                            class: "primary-button",
                            onclick: {
                                let peer_id = peer_id.clone();
                                move |_| {
                                    generating.set(true);
                                    gen_error.set(None);
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    actor.send(Cmd::GetPeerRepresentation {
                                        peer_id: peer_id.clone(),
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
                                                )));
                                            }
                                        }
                                        generating.set(false);
                                    });
                                }
                            },
                            "Generate"
                        }
                    }

                    if let Some(text) = &gen_text {
                        pre { class: "representation-view", "{text}" }
                    }

                    if let Some((code, message)) = &gen_err {
                        ErrorView {
                            code: code.clone(),
                            message: message.clone(),
                            retryable: false,
                            on_retry: None,
                        }
                    }
                }
            }
        }
    }
}
