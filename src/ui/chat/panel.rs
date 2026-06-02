use dioxus::prelude::*;

use crate::actor::commands::{ChatOpts, Cmd, StreamEvent};
use crate::state::{AppState, ToastKind};
use taicho::domain::chat::{ChatMessage, ChatRole};

use super::markdown::render_markdown;

#[component]
pub fn ChatPanel(peer_id: String) -> Element {
    let mut state: AppState = use_context();
    let coroutine: Coroutine<Cmd> = use_coroutine_handle();
    let mut input = use_signal(String::new);
    let mut history: Signal<Vec<ChatMessage>> = use_signal(Vec::new);
    let mut target: Signal<Option<String>> = use_signal(|| None);
    let mut non_stream = use_signal(|| false);

    let mut chat_streaming = state.chat_streaming;

    // Reset streaming flag if the component unmounts mid-stream.
    use_drop(move || {
        chat_streaming.set(false);
    });

    let streaming = move || *state.chat_streaming.read();

    let send = move |_| {
        let q = input.read().clone();
        if q.trim().is_empty() || *state.chat_streaming.read() {
            return;
        }
        history.write().push(ChatMessage {
            peer_id: peer_id.clone(),
            content: q.clone(),
            role: ChatRole::User,
            created_at: chrono::Utc::now().to_rfc3339(),
            token_count: 0,
        });
        let opts = ChatOpts {
            session_id: None,
            peer_target: target.read().clone(),
        };
        if *non_stream.read() {
            let (tx, rx) = tokio::sync::oneshot::channel();
            coroutine.send(Cmd::Chat {
                peer_id: peer_id.clone(),
                query: q,
                opts,
                reply: tx,
            });
            let state_for_spawn = state;
            let mut history_for_spawn = history;
            let peer_id_for_spawn = peer_id.clone();
            spawn(async move {
                match rx.await {
                    Ok(Ok(Some(text))) => {
                        history_for_spawn.write().push(ChatMessage {
                            peer_id: peer_id_for_spawn,
                            content: text,
                            role: ChatRole::Assistant,
                            created_at: chrono::Utc::now().to_rfc3339(),
                            token_count: 0,
                        });
                    }
                    Ok(Err(e)) => {
                        state_for_spawn.push_toast(ToastKind::Error, e.user_message());
                    }
                    Err(_) => {
                        state_for_spawn.push_toast(ToastKind::Error, "Chat reply channel closed");
                    }
                    Ok(Ok(None)) => {}
                }
            });
        } else {
            state.chat_streaming.set(true);
            let (tx, mut rx) = tokio::sync::mpsc::channel(64);
            coroutine.send(Cmd::StreamChat {
                peer_id: peer_id.clone(),
                query: q,
                opts,
                tx,
            });
            let mut state_for_spawn = state;
            let mut history_for_spawn = history;
            let peer_id_for_spawn = peer_id.clone();
            spawn(async move {
                let mut acc = String::new();
                let mut last_is_assistant = history_for_spawn
                    .read()
                    .last()
                    .is_some_and(|m| matches!(m.role, ChatRole::Assistant));
                while let Some(ev) = rx.recv().await {
                    match ev {
                        StreamEvent::Chunk(s) => {
                            acc.push_str(&s);
                            if last_is_assistant {
                                if let Some(last) = history_for_spawn.write().last_mut() {
                                    last.content = acc.clone();
                                }
                            } else {
                                history_for_spawn.write().push(ChatMessage {
                                    peer_id: peer_id_for_spawn.clone(),
                                    content: acc.clone(),
                                    role: ChatRole::Assistant,
                                    created_at: chrono::Utc::now().to_rfc3339(),
                                    token_count: 0,
                                });
                                last_is_assistant = true;
                            }
                        }
                        StreamEvent::Done(final_text) => {
                            if !final_text.is_empty()
                                && let Some(last) = history_for_spawn.write().last_mut()
                            {
                                last.content = final_text;
                            }
                            break;
                        }
                        StreamEvent::Err(e) => {
                            state_for_spawn.push_toast(ToastKind::Error, e.user_message());
                            break;
                        }
                    }
                }
                state_for_spawn.chat_streaming.set(false);
            });
        }
        input.set(String::new());
    };

    let mut send_for_key = send.clone();
    let mut send_for_click = send;

    rsx! {
        div { class: "chat-panel",
            div { class: "chat-messages",
                if history.read().is_empty() {
                    div { class: "chat-empty",
                        "No messages yet. Ask the peer something."
                    }
                } else {
                    for (idx, msg) in history.read().iter().enumerate() {
                        div {
                            key: "{idx}",
                            class: match msg.role {
                                ChatRole::User => "msg msg-user",
                                ChatRole::Assistant => "msg msg-assistant",
                                ChatRole::System | ChatRole::Unknown => "msg msg-system",
                            },
                            div {
                                class: "msg-content",
                                role: "article",
                                dangerous_inner_html: "{render_markdown(&msg.content)}"
                            }
                        }
                    }
                }
            }
            div { class: "chat-controls",
                input {
                    class: "chat-target",
                    placeholder: "Target peer (optional)",
                    value: "{target.read().clone().unwrap_or_default()}",
                    oninput: move |e| target.set(if e.value().is_empty() { None } else { Some(e.value()) }),
                }
                label {
                    input {
                        r#type: "checkbox",
                        checked: *non_stream.read(),
                        onchange: move |e| non_stream.set(e.checked()),
                    }
                    " Non-stream"
                }
            }
            div { class: "chat-input-row",
                textarea {
                    class: "chat-input",
                    placeholder: if streaming() { "Streaming..." } else { "Ask the peer..." },
                    value: "{input.read()}",
                    disabled: streaming(),
                    oninput: move |e| input.set(e.value()),
                    onkeydown: move |e| {
                        if e.modifiers().contains(Modifiers::META) && e.key() == Key::Enter {
                            send_for_key(());
                        }
                    },
                }
                button {
                    class: "chat-send",
                    disabled: streaming() || input.read().trim().is_empty(),
                    onclick: move |_| send_for_click(()),
                    "Send"
                }
            }
        }
    }
}
