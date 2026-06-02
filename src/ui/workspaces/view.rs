use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use crate::state::connection::ConnectionState;
use crate::state::selection::InspectorSection;
use taicho::domain::raw_json::RawJson;
use taicho::error::AppError;

use super::super::common::json_viewer::JsonViewer;
use super::super::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone, Copy, PartialEq, Eq)]
enum WsTab {
    Metadata,
    Configuration,
    AllWorkspaces,
}

#[component]
pub fn WorkspaceView() -> Element {
    let mut state: AppState = use_context();
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let active_tab: Signal<WsTab> = use_signal(|| WsTab::Metadata);

    let mut metadata: Signal<Option<Result<RawJson, AppError>>> = use_signal(|| None);
    let mut config: Signal<Option<Result<RawJson, AppError>>> = use_signal(|| None);
    let mut workspace_ids: Signal<Option<Result<Vec<String>, AppError>>> = use_signal(|| None);
    let mut confirm_input: Signal<String> = use_signal(String::new);
    let mut show_delete_confirm: Signal<bool> = use_signal(|| false);
    let mut delete_status: Signal<Option<String>> = use_signal(|| None);

    let fetch_metadata = {
        move || {
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::GetWorkspaceMetadata { reply: tx });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("get_workspace_metadata"))
                    .and_then(|r| r);
                metadata.set(Some(result));
            });
        }
    };

    let fetch_config = {
        move || {
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::GetWorkspaceConfig { reply: tx });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("get_workspace_config"))
                    .and_then(|r| r);
                config.set(Some(result));
            });
        }
    };

    let fetch_ids = {
        move || {
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::ListWorkspaces { reply: tx });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("list_workspaces"))
                    .and_then(|r| r);
                workspace_ids.set(Some(result));
            });
        }
    };

    let retry_metadata = {
        let mut metadata = metadata;
        move |_: MouseEvent| {
            metadata.set(None);
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::GetWorkspaceMetadata { reply: tx });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("get_workspace_metadata"))
                    .and_then(|r| r);
                metadata.set(Some(result));
            });
        }
    };

    let retry_config = {
        let mut config = config;
        move |_: MouseEvent| {
            config.set(None);
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::GetWorkspaceConfig { reply: tx });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("get_workspace_config"))
                    .and_then(|r| r);
                config.set(Some(result));
            });
        }
    };

    let retry_ids = {
        let mut workspace_ids = workspace_ids;
        move |_: MouseEvent| {
            workspace_ids.set(None);
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::ListWorkspaces { reply: tx });
            spawn(async move {
                let result = rx
                    .await
                    .map_err(|_| AppError::channel_closed("list_workspaces"))
                    .and_then(|r| r);
                workspace_ids.set(Some(result));
            });
        }
    };

    use_effect(move || {
        fetch_metadata();
        fetch_config();
        fetch_ids();
    });

    let ws_info = state.workspace_info.read().clone();
    let tab = *active_tab.read();
    let is_confirming = *show_delete_confirm.read();

    rsx! {
            div { class: "workspace-view",
                div { class: "list-toolbar",
                    h2 { "Workspace" }
                    button {
                        class: "secondary-button",
                        onclick: move |_| {
                            metadata.set(None);
                            config.set(None);
                            workspace_ids.set(None);
                            fetch_metadata();
                            fetch_config();
                            fetch_ids();
                        },
                        "Refresh"
                    }
                }

                if let Some(info) = &ws_info {
                    div { class: "workspace-card",
                        div { class: "workspace-card-field",
                            span { class: "workspace-card-label", "ID" }
                            code { "{info.id}" }
                        }
                        div { class: "workspace-card-field",
                            span { class: "workspace-card-label", "Base URL" }
                            code { "{info.base_url}" }
                        }
                    }
                }

                div { class: "detail-tabs",
                    TabBtn { label: "Metadata", tab: WsTab::Metadata, active_tab }
                    TabBtn { label: "Configuration", tab: WsTab::Configuration, active_tab }
                    TabBtn { label: "All Workspaces", tab: WsTab::AllWorkspaces, active_tab }
                }

                div { class: "tab-content",
                    {match tab {
                        WsTab::Metadata => match &*metadata.read() {
                            None => rsx! { LoadingView { label: "Loading metadata...".to_string() } },
                            Some(Err(e)) => rsx! {
                                ErrorView {
                                    code: e.code().to_string(),
                                    message: e.user_message(),
                                    retryable: e.is_retryable(),
                                    on_retry: Some(EventHandler::new(retry_metadata)),
                                }
                            },
                            Some(Ok(raw)) => rsx! {
                                JsonViewer {
    value: serde_json::to_string_pretty(raw.value())
                                        .unwrap_or_else(|e| format!("JSON error: {e}")),
                                }
                            },
                        },
                        WsTab::Configuration => match &*config.read() {
                            None => rsx! { LoadingView { label: "Loading configuration...".to_string() } },
                            Some(Err(e)) => rsx! {
                                ErrorView {
                                    code: e.code().to_string(),
                                    message: e.user_message(),
                                    retryable: e.is_retryable(),
                                    on_retry: Some(EventHandler::new(retry_config)),
                                }
                            },
                            Some(Ok(raw)) => rsx! {
                                JsonViewer {
                                    value: serde_json::to_string_pretty(raw.value())
                                        .unwrap_or_else(|e| format!("JSON error: {e}")),
                                }
                            },
                        },
                        WsTab::AllWorkspaces => match &*workspace_ids.read() {
                            None => rsx! { LoadingView { label: "Loading workspaces...".to_string() } },
                            Some(Err(e)) => rsx! {
                                ErrorView {
                                    code: e.code().to_string(),
                                    message: e.user_message(),
                                    retryable: e.is_retryable(),
                                    on_retry: Some(EventHandler::new(retry_ids)),
                                }
                            },
                            Some(Ok(ids)) if ids.is_empty() => rsx! {
                                EmptyView {
                                    title: "No workspaces".to_string(),
                                    message: "No workspaces found.".to_string(),
                                }
                            },
                            Some(Ok(ids)) => rsx! {
                                div { class: "list-items",
                                    for id in ids {
                                        div { key: "{id}", class: "list-item",
                                            span { class: "list-item-id", "{id}" }
                                        }
                                    }
                                }
                            },
                        },
                    }}
                }

            div { class: "danger-section",
                h3 { "Danger Zone" }
                if let Some(status) = &*delete_status.read() {
                    p { class: "status-message", "{status}" }
                }
                if is_confirming {
                    div { class: "confirm-bar",
                        p { "Type the workspace ID to confirm deletion. This cannot be undone." }
                        input {
                            class: "confirm-input",
                            placeholder: "Type workspace ID to confirm",
                            value: "{confirm_input}",
                            oninput: move |evt| confirm_input.set(evt.value()),
                        }
                        button {
                            class: "danger-button",
                            disabled: ws_info.as_ref().is_none_or(|info| *confirm_input.read() != info.id),
                            onclick: {
                                let ws_info = ws_info.clone();
                                move |_| {
                                    let Some(info) = ws_info.as_ref() else { return };
                                    show_delete_confirm.set(false);
                                    confirm_input.set(String::new());
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    actor.send(Cmd::DeleteWorkspace {
                                        workspace_id: info.id.clone(),
                                        reply: tx,
                                    });
                                    spawn(async move {
                                        match rx.await {
                                            Ok(Ok(())) => {
                                                state.connection.set(ConnectionState::Disconnected);
                                                state.workspace_info.set(None);
                                                state.selection.clear();
                                                state.selected_section.set(InspectorSection::default());
                                                state.status_message.set("Workspace deleted".to_string());
                                                delete_status.set(None);
                                            }
                                            Ok(Err(e)) => {
                                                delete_status.set(Some(format!("Delete failed: {}", e.user_message())));
                                            }
                                            Err(_) => {
                                                delete_status.set(Some("Delete failed: channel closed".to_string()));
                                            }
                                        }
                                    });
                                }
                            },
                            "Delete Workspace"
                        }
                        button {
                            class: "secondary-button",
                            onclick: move |_| {
                                show_delete_confirm.set(false);
                                confirm_input.set(String::new());
                                delete_status.set(None);
                            },
                            "Cancel"
                        }
                    }
                } else {
                    button {
                        class: "danger-button",
                        onclick: move |_| show_delete_confirm.set(true),
                        "Delete Workspace"
                    }
                }
            }
            }
        }
}

#[component]
fn TabBtn(label: String, tab: WsTab, active_tab: Signal<WsTab>) -> Element {
    let is_active = *active_tab.read() == tab;
    rsx! {
        button {
            class: if is_active { "detail-tab active" } else { "detail-tab" },
            onclick: move |_| active_tab.set(tab),
            "{label}"
        }
    }
}
