use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use taicho::domain::{DomainPage, MessageRow};
use taicho::error::AppError;

use super::super::common::pagination::Pagination;
use super::super::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone)]
enum MessageListState {
    Loaded(DomainPage<MessageRow>),
    Empty,
    Error(String, String, bool),
}

impl MessageListState {
    fn from_result(result: Result<DomainPage<MessageRow>, AppError>) -> Self {
        match result {
            Ok(page) if page.is_empty() => Self::Empty,
            Ok(page) => Self::Loaded(page),
            Err(e) => {
                let retryable = e.is_retryable();
                Self::Error(e.code().to_string(), e.user_message(), retryable)
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        let mut end = max;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
}

#[component]
pub fn MessageList() -> Element {
    let mut state: AppState = use_context();
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut messages: Signal<Option<MessageListState>> = use_signal(|| None);
    let mut current_page: Signal<u64> = use_signal(|| 1);

    let selected_msg_id = state.selection.message_id.read().clone();

    let fetch_messages = use_callback(move |(session_id, page): (String, u64)| {
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::ListMessages {
            session_id,
            page,
            size: 50,
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("list_messages"))
                .and_then(|r| r);
            messages.set(Some(MessageListState::from_result(result)));
        });
    });

    use_effect(move || {
        if let Some(ref session_id) = *state.selection.session_id.read() {
            if messages.read().is_none() {
                fetch_messages.call((session_id.clone(), 1));
            }
        } else {
            messages.set(None);
        }
    });

    let snapshot = messages.read().clone();

    let Some(session_id) = state.selection.session_id.read().clone() else {
        return rsx! {
            EmptyView {
                title: "No session selected".to_string(),
                message: "Select a session first to view its messages.".to_string(),
            }
        };
    };

    let page_sid = session_id.clone();

    rsx! {
        div { class: "list-toolbar",
            h2 { "Messages" }
            button {
                class: "secondary-button",
                onclick: move |_| {
                    messages.set(None);
                    fetch_messages.call((session_id.clone(), *current_page.read()));
                },
                "Refresh"
            }
        }

        match snapshot {
            None => rsx! { LoadingView { label: "Loading messages...".to_string() } },
            Some(MessageListState::Error(code, message, retryable)) => rsx! {
                ErrorView {
                    code,
                    message,
                    retryable,
                    on_retry: Some(EventHandler::new(move |_: MouseEvent| {
                        messages.set(None);
                        fetch_messages.call((page_sid.clone(), *current_page.read()));
                    })),
                }
            },
            Some(MessageListState::Empty) => rsx! {
                EmptyView {
                    title: "No messages".to_string(),
                    message: "This session has no messages.".to_string(),
                }
            },
            Some(MessageListState::Loaded(page)) => rsx! {
                div { class: "list-items",
                    for msg in &page.items {
                        {
                            let msg_id = msg.id.clone();
                            let is_selected = selected_msg_id.as_deref() == Some(&msg.id);
                            let preview = truncate(&msg.content, 100);
                            rsx! {
                                button {
                                    key: "{msg_id}",
                                    class: if is_selected { "message-row selected" } else { "message-row" },
                                    onclick: move |_| {
                                        state.selection.message_id.set(Some(msg_id.clone()));
                                    },
                                    div { class: "message-row-header",
                                        span { class: "message-peer", "{msg.peer_id}" }
                                        span { class: "message-time", "{msg.created_at}" }
                                    }
                                    div { class: "message-preview", "{preview}" }
                                }
                            }
                        }
                    }
                }
                Pagination {
                    page: page.info.page,
                    pages: page.info.pages,
                        on_page_change: move |new_page: u64| {
                            current_page.set(new_page);
                            messages.set(None);
                            fetch_messages.call((page_sid.clone(), new_page));
                        },
                }
            },
        }
    }
}
