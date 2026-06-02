use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use taicho::domain::MessageRow;
use taicho::error::AppError;

use super::super::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ListLoadingState {
    Initial,
    Loading,
    Loaded,
    Empty,
    Error,
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
    let mut loading_state: Signal<ListLoadingState> = use_signal(|| ListLoadingState::Initial);
    let mut all_items: Signal<Vec<MessageRow>> = use_signal(Vec::new);
    let mut current_page: Signal<u64> = use_signal(|| 1);
    let mut has_next: Signal<bool> = use_signal(|| false);
    let mut fetching_more: Signal<bool> = use_signal(|| false);
    let mut error_info: Signal<Option<(String, String, bool)>> = use_signal(|| None);

    let selected_msg_id = state.selection.message_id.read().clone();

    let fetch_messages = use_callback(move |(session_id, page, append): (String, u64, bool)| {
        if append {
            fetching_more.set(true);
        } else {
            loading_state.set(ListLoadingState::Loading);
            error_info.set(None);
        }

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

            match result {
                Ok(page_data) if page_data.is_empty() && !append => {
                    all_items.set(Vec::new());
                    loading_state.set(ListLoadingState::Empty);
                }
                Ok(page_data) => {
                    has_next.set(page_data.info.has_next);
                    current_page.set(page_data.info.page);
                    if append {
                        all_items.write().extend(page_data.items);
                    } else {
                        all_items.set(page_data.items);
                        loading_state.set(ListLoadingState::Loaded);
                    }
                }
                Err(e) => {
                    let retryable = e.is_retryable();
                    error_info.set(Some((e.code().to_string(), e.user_message(), retryable)));
                    if !append {
                        loading_state.set(ListLoadingState::Error);
                    }
                }
            }
            fetching_more.set(false);
        });
    });

    use_effect(move || {
        if let Some(ref session_id) = *state.selection.session_id.read() {
            all_items.set(Vec::new());
            current_page.set(1);
            has_next.set(false);
            fetch_messages.call((session_id.clone(), 1, false));
        } else {
            all_items.set(Vec::new());
            loading_state.set(ListLoadingState::Empty);
        }
    });

    let Some(session_id) = state.selection.session_id.read().clone() else {
        return rsx! {
            EmptyView {
                title: "No session selected".to_string(),
                message: "Select a session first to view its messages.".to_string(),
            }
        };
    };

    let refresh_sid = session_id.clone();
    let scroll_session_id = session_id.clone();
    let error_retry_sid = session_id.clone();
    let error_fallback_sid = session_id.clone();

    let on_scroll = move |evt: Event<ScrollData>| {
        let data = evt.data();
        #[allow(clippy::unnecessary_cast)]
        let scroll_top = data.scroll_top() as f64;
        #[allow(clippy::unnecessary_cast)]
        let scroll_height = data.scroll_height() as f64;
        #[allow(clippy::unnecessary_cast)]
        let client_height = data.client_height() as f64;

        let near_bottom = scroll_height - scroll_top - client_height < 200.0;

        if near_bottom && *has_next.read() && !*fetching_more.read() {
            let next = *current_page.read() + 1;
            fetch_messages.call((scroll_session_id.clone(), next, true));
        }
    };

    rsx! {
        div { class: "list-toolbar",
            h2 { "Messages" }
            button {
                class: "secondary-button",
                onclick: move |_| {
                    all_items.set(Vec::new());
                    current_page.set(1);
                    has_next.set(false);
                    fetch_messages.call((refresh_sid.clone(), 1, false));
                },
                "Refresh"
            }
        }

        match (*loading_state.read(), error_info.read().clone()) {
            (ListLoadingState::Initial | ListLoadingState::Loading, _) => rsx! {
                LoadingView { label: "Loading messages...".to_string() }
            },
            (ListLoadingState::Error, Some((code, message, retryable))) => {
                rsx! {
                    ErrorView {
                        code,
                        message,
                        retryable,
                        on_retry: Some(EventHandler::new(move |_: MouseEvent| {
                            all_items.set(Vec::new());
                            current_page.set(1);
                            has_next.set(false);
                            fetch_messages.call((error_retry_sid.clone(), 1, false));
                        })),
                    }
                }
            },
            (ListLoadingState::Error, _) => rsx! {
                ErrorView {
                    code: "unknown_error".to_string(),
                    message: "An unknown error occurred.".to_string(),
                    retryable: true,
                    on_retry: Some(EventHandler::new(move |_: MouseEvent| {
                        all_items.set(Vec::new());
                        current_page.set(1);
                        has_next.set(false);
                        fetch_messages.call((error_fallback_sid.clone(), 1, false));
                    })),
                }
            },
            (ListLoadingState::Empty, _) => rsx! {
                EmptyView {
                    title: "No messages".to_string(),
                    message: "This session has no messages.".to_string(),
                }
            },
            (ListLoadingState::Loaded, _) => {
                let items = all_items.read().clone();
                rsx! {
                    div {
                        class: "message-list-scroll",
                        onscroll: on_scroll,
                        overflow_y: "auto",
                        max_height: "calc(100vh - 160px)",

                        div { class: "list-items",
                            for msg in &items {
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

                        if *fetching_more.read() {
                            div { class: "infinite-scroll-loader",
                                LoadingView { label: "Loading more messages...".to_string() }
                            }
                        }
                    }
                }
            },
        }
    }
}
