use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use taicho::domain::SessionDetails;
use taicho::error::{AppError, AppResult};

use super::super::common::json_viewer::JsonViewer;
use super::super::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone, Copy, PartialEq, Eq)]
enum DetailTab {
    Overview,
    Metadata,
    Configuration,
    Summaries,
    Actions,
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

    // Dioxus 0.7: spawned futures are cancelled on component unmount (drop of the scope).
    // No manual cancellation token needed — navigating away drops the rx automatically.
    use_effect(move || {
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

    match &*details.read() {
        None => rsx! {
            LoadingView { label: "Loading session details...".to_string() }
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
                        h2 { "Session: {detail.id}" }
                        span { class: "muted", "Workspace: {detail.workspace_id}" }
                    }

                    div { class: "tab-bar",
                        TabButton { label: "Overview", tab: DetailTab::Overview, active_tab }
                        TabButton { label: "Metadata", tab: DetailTab::Metadata, active_tab }
                        TabButton { label: "Configuration", tab: DetailTab::Configuration, active_tab }
                        TabButton { label: "Summaries", tab: DetailTab::Summaries, active_tab }
                        TabButton { label: "Actions", tab: DetailTab::Actions, active_tab }
                    }

                    div { class: "tab-content",
                        {match tab {
                            DetailTab::Overview => rsx! {
                                OverviewTabContent { detail: detail.clone() }
                            },
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
                            DetailTab::Summaries => rsx! {
                                SummariesTabContent { detail: detail.clone() }
                            },
                            DetailTab::Actions => rsx! {
                                ActionsTabContent { detail, confirm_delete }
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
                                    state.status_message.set(
                                        format!("Session cloned \u{2192} {}", row.id),
                                    );
                                    state.selection.session_id.set(Some(row.id));
                                }
                                Ok(Err(e)) => {
                                    state.status_message.set(
                                        format!("Clone failed: {}", e.user_message()),
                                    );
                                }
                                Err(_) => {
                                    state.status_message.set(
                                        "Clone cancelled (channel closed)".to_string(),
                                    );
                                }
                            }
                        });
                    }
                },
                "Clone Session"
            }

            if *confirm_delete.read() {
                div { class: "confirm-bar",
                    p { "Are you sure? This cannot be undone." }
                    button {
                        class: "danger-button",
                        onclick: {
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
                                            state.status_message.set(
                                                "Session deleted".to_string(),
                                            );
                                        }
                                        Ok(Err(e)) => {
                                            state.status_message.set(
                                                format!("Delete failed: {}", e.user_message()),
                                            );
                                        }
                                        Err(_) => {
                                            state.status_message.set(
                                                "Delete cancelled (channel closed)".to_string(),
                                            );
                                        }
                                    }
                                });
                                confirm_delete.set(false);
                            }
                        },
                        "Confirm"
                    }
                    button {
                        class: "secondary-button",
                        onclick: move |_| confirm_delete.set(false),
                        "Cancel"
                    }
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
