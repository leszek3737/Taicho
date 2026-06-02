use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::{AppState, ToastKind};
use taicho::domain::MessageRow;
use taicho::domain::raw_json::JsonMap;
use taicho::error::{AppError, AppResult};

use crate::ui::common::json_editor::JsonEditor;
use crate::ui::common::json_viewer::JsonViewer;
use crate::ui::common::{EmptyView, ErrorView, LoadingView};

#[component]
pub fn MessageDetail() -> Element {
    let state: AppState = use_context();
    let msg_id = state.selection.message_id.read().clone();
    let session_id = state.selection.session_id.read().clone();

    match (session_id, msg_id) {
        (Some(sid), Some(mid)) => rsx! {
            MessageDetailInner { key: "{sid}/{mid}", session_id: sid, message_id: mid }
        },
        _ => rsx! {
            EmptyView {
                title: "No message selected".to_string(),
                message: "Select a message from the list to view details.".to_string(),
            }
        },
    }
}

#[component]
fn MessageDetailInner(session_id: String, message_id: String) -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let state: AppState = use_context();
    let mut message: Signal<Option<AppResult<MessageRow>>> = use_signal(|| None);
    let mut editing_metadata: Signal<bool> = use_signal(|| false);
    let mut saving_metadata: Signal<bool> = use_signal(|| false);
    let fetch_version: Signal<u32> = use_signal(|| 0);

    let session_id_for_effect = session_id.clone();
    let message_id_for_effect = message_id.clone();
    use_effect(move || {
        // Depend on fetch_version so retry handler can re-trigger the fetch.
        let _v = *fetch_version.read();
        editing_metadata.set(false);
        saving_metadata.set(false);
        message.set(None);
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::GetMessage {
            session_id: session_id_for_effect.clone(),
            message_id: message_id_for_effect.clone(),
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("get_message"))
                .and_then(|r| r);
            message.set(Some(result));
        });
    });

    match &*message.read() {
        None => rsx! { LoadingView { label: "Loading message...".to_string() } },
        Some(Err(e)) => {
            let fetch_for_retry = fetch_version;
            let on_retry: Option<EventHandler<MouseEvent>> =
                Some(EventHandler::new(move |_: MouseEvent| {
                    let mut v = fetch_for_retry;
                    let current = *v.read();
                    v.set(current + 1);
                }));
            rsx! {
                ErrorView {
                    code: e.code().to_string(),
                    message: e.user_message(),
                    retryable: e.is_retryable(),
                    on_retry,
                }
            }
        }
        Some(Ok(msg)) => {
            let msg = msg.clone();
            let metadata_json = msg.metadata.to_json_map();
            rsx! {
                div { class: "detail-content",
                    div { class: "detail-header",
                        h2 { "Message: {msg.id}" }
                    }

                    div { class: "detail-section",
                        h3 { "Message ID" }
                        code { "{msg.id}" }
                    }
                    div { class: "detail-section",
                        h3 { "Workspace ID" }
                        code { "{msg.workspace_id}" }
                    }
                    div { class: "detail-section",
                        h3 { "Session ID" }
                        code { "{msg.session_id}" }
                    }
                    div { class: "detail-section",
                        h3 { "Peer ID" }
                        code { "{msg.peer_id}" }
                    }
                    div { class: "detail-section",
                        h3 { "Created At" }
                        p { "{msg.created_at}" }
                    }
                    div { class: "detail-section",
                        h3 { "Token Count" }
                        p { "{msg.token_count}" }
                    }

                    div { class: "detail-section",
                        h3 { "Content" }
                        pre { class: "message-content-view", "{msg.content}" }
                    }

                    div { class: "detail-section",
                        h3 { "Metadata" }
                        if metadata_json.is_none() && !*editing_metadata.read() {
                            p { class: "empty-text", "No metadata" }
                        } else if *editing_metadata.read() {
                            JsonEditor {
                                initial: metadata_json,
                                label: "Metadata".to_string(),
                                saving: *saving_metadata.read(),
                                on_change: move |new_json: JsonMap| {
                                    if *saving_metadata.read() {
                                        return;
                                    }
                                    saving_metadata.set(true);
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    actor.send(Cmd::UpdateMessageMetadata {
                                        session_id: session_id.clone(),
                                        message_id: message_id.clone(),
                                        metadata: new_json,
                                        reply: tx,
                                    });
                                    let mut msg_signal = message;
                                    let mut editing = editing_metadata;
                                    let mut saving = saving_metadata;
                                    spawn(async move {
                                        match rx.await {
                                            Ok(Ok(updated)) => {
                                                msg_signal.set(Some(Ok(updated)));
                                                state.push_toast(ToastKind::Info, "Metadata saved");
                                                editing.set(false);
                                                saving.set(false);
                                            }
                                            Ok(Err(e)) => {
                                                state.push_toast(ToastKind::Error, format!(
                                                    "Save failed: {}",
                                                    e.user_message()
                                                ));
                                                saving.set(false);
                                            }
                                            Err(_) => {
                                                state.push_toast(ToastKind::Warning, "Save cancelled");
                                                saving.set(false);
                                            }
                                        }
                                    });
                                },
                                on_cancel: move |_| {
                                    editing_metadata.set(false);
                                },
                            }
                        } else {
                            div {
                                JsonViewer {
                                    value: serde_json::to_string_pretty(msg.metadata.value())
                                        .unwrap_or_else(|e| format!("JSON error: {e}")),
                                }
                                button {
                                    class: "secondary-button",
                                    onclick: move |_| editing_metadata.set(true),
                                    "Edit metadata"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
