use std::time::Duration;

use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use crate::state::connection::ConnectionState;
use crate::state::selection::InspectorSection;

use super::common::EmptyView;
use super::common::toast::ToastContainer;
use super::search::palette::CommandPalette;

#[component]
pub fn RootShell() -> Element {
    let state: AppState = use_context();
    let connection = state.connection.read().clone();

    match connection {
        ConnectionState::Disconnected | ConnectionState::Failed { .. } => {
            rsx! { super::connection::ConnectionScreen {} }
        }
        ConnectionState::Connecting => {
            rsx! { super::common::LoadingView { label: "Connecting...".to_string() } }
        }
        ConnectionState::Connected => {
            rsx! { InspectorShell {} }
        }
    }
}

#[component]
fn InspectorShell() -> Element {
    let mut state: AppState = use_context();
    let selected_section = *state.selected_section.read();
    let status = state.status_message.read().clone();

    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();

    let search_open = *state.search_open.read();

    rsx! {
        super::shortcuts::KeyboardShortcuts {
            if search_open {
                CommandPalette {}
            }
            ToastContainer {}
            div { class: "app-shell",
            header { class: "top-bar",
                div {
                    strong { "Taicho" }
                    span { class: "muted", " — Honcho Inspector" }
                }
                button {
                    class: "secondary-button",
                    onclick: move |_| {
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        actor.send(Cmd::Disconnect { reply: tx });
                        let mut conn = state.connection;
                        let mut status_sig = state.status_message;
                        let mut workspace_info = state.workspace_info;
                        let mut selection = state.selection;
                        let mut section_sig = state.selected_section;
                        spawn(async move {
                            let mut reset_ui = |status_text: String| {
                                conn.set(ConnectionState::Disconnected);
                                workspace_info.set(None);
                                selection.clear();
                                section_sig.set(InspectorSection::default());
                                status_sig.set(status_text);
                            };
                            match tokio::time::timeout(Duration::from_secs(5), rx).await {
                                Ok(Ok(Ok(()))) => {
                                    reset_ui("Disconnected".to_string());
                                }
                                Ok(Ok(Err(e))) => {
                                    tracing::warn!("disconnect error: {}", e.user_message());
                                    reset_ui(format!("Disconnected ({})", e.user_message()));
                                }
                                Ok(Err(_)) => {
                                    reset_ui("Disconnected".to_string());
                                }
                                Err(_) => {
                                    tracing::warn!("disconnect reply timed out after 5s");
                                    reset_ui("Disconnected (timeout)".to_string());
                                }
                            }
                        });
                    },
                    "Disconnect"
                }
            }

            div { class: "three-pane",
                nav { class: "sidebar", aria_label: "Inspector sections",
                    for section in InspectorSection::ALL {
                        button {
                            key: "{section:?}",
                            class: if section == selected_section { "sidebar-item selected" } else { "sidebar-item" },
                            onclick: move |_| {
                                state.selected_section.set(section);
                            },
                            "{section.label()}"
                        }
                    }
                }

                section { class: "list-pane",
                    match selected_section {
                        InspectorSection::Peers => rsx! { super::peers::PeerList {} },
                        InspectorSection::Sessions => rsx! { super::sessions::SessionList {} },
                        InspectorSection::Messages => rsx! { super::messages::MessageList {} },
                        InspectorSection::Workspaces => rsx! { super::workspaces::WorkspaceView {} },
                        InspectorSection::Conclusions => rsx! { super::conclusions::ConclusionList {} },
                        InspectorSection::RawJson => rsx! {
                            div { class: "panel-content",
                                h3 { "Raw JSON" }
                                p { "Select an item to view its raw JSON metadata." }
                            }
                        },
                    }
                }

                section { class: "detail-pane",
                    match selected_section {
                        InspectorSection::Peers => rsx! { super::peers::PeerDetail {} },
                        InspectorSection::Sessions => rsx! { super::sessions::SessionDetail {} },
                        InspectorSection::Messages => rsx! {
                            super::messages::MessageDetail {}
                        },
                        InspectorSection::Workspaces => {
                            rsx! {
                                h2 { "Workspace" }
                                EmptyView {
                                    title: "Single view".to_string(),
                                    message: "Workspace details are shown in the left pane.".to_string(),
                                }
                            }
                        }
                        InspectorSection::Conclusions => {
                            let selected_conclusion = state.selection.conclusion_id.read().clone();
                            rsx! {
                                h2 { "Conclusion details" }
                                if let Some(item_id) = selected_conclusion {
                                    p { "Selected: {item_id}" }
                                } else {
                                    EmptyView {
                                        title: "Nothing selected".to_string(),
                                        message: "Select a conclusion from the middle pane.".to_string(),
                                    }
                                }
                            }
                        }
                        _ => {
                            rsx! {
                                h2 { "Details" }
                                EmptyView {
                                    title: "Nothing selected".to_string(),
                                    message: "Select a row from the middle pane.".to_string(),
                                }
                            }
                        }
                    }
                }
            }

            footer { class: "status-bar",
                span { "{status}" }
            }
        }
        }
    }
}
