use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use taicho::domain::{DomainPage, SessionRow};
use taicho::error::AppError;

use super::super::common::pagination::Pagination;
use super::super::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone)]
enum SessionListState {
    Loaded(DomainPage<SessionRow>),
    Empty,
    Error(String, String, bool),
}

impl SessionListState {
    fn from_result(result: Result<DomainPage<SessionRow>, AppError>) -> Self {
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

#[component]
pub fn SessionList() -> Element {
    let mut state: AppState = use_context();
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut sessions: Signal<Option<SessionListState>> = use_signal(|| None);
    let mut current_page: Signal<u64> = use_signal(|| 1);

    let selected_id = state.selection.session_id.read().clone();

    let fetch_sessions = use_callback(move |page: u64| {
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::ListSessions {
            page,
            size: 50,
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("list_sessions"))
                .and_then(|r| r);
            sessions.set(Some(SessionListState::from_result(result)));
        });
    });

    use_effect(move || {
        if sessions.read().is_none() {
            fetch_sessions.call(1);
        }
    });

    let snapshot = sessions.read().clone();

    rsx! {
        div { class: "list-toolbar",
            h2 { "Sessions" }
            button {
                class: "secondary-button",
                onclick: move |_| {
                    sessions.set(None);
                    fetch_sessions.call(*current_page.read());
                },
                "Refresh"
            }
        }

        match snapshot {
            None => rsx! { LoadingView { label: "Loading sessions...".to_string() } },
            Some(SessionListState::Error(code, message, retryable)) => rsx! {
                ErrorView {
                    code,
                    message,
                    retryable,
                    on_retry: Some(EventHandler::new(move |_: MouseEvent| {
                        sessions.set(None);
                        fetch_sessions.call(*current_page.read());
                    })),
                }
            },
            Some(SessionListState::Empty) => rsx! {
                EmptyView {
                    title: "No sessions".to_string(),
                    message: "No sessions found on this server.".to_string(),
                }
            },
            Some(SessionListState::Loaded(page)) => rsx! {
                div { class: "list-items",
                    for session in &page.items {
                        {
                            let session_id = session.id.clone();
                            let is_selected = selected_id.as_deref() == Some(&session.id);
                            let is_active = session.is_active;
                            rsx! {
                                button {
                                    key: "{session_id}",
                                    class: if is_selected { "list-item selected" } else { "list-item" },
                                    onclick: move |_| {
                                        state.selection.session_id.set(Some(session_id.clone()));
                                    },
                                    span { class: "list-item-id", "{session.id}" }
                                    span {
                                        class: if is_active { "badge badge-active" } else { "badge badge-inactive" },
                                        if is_active { "Active" } else { "Inactive" }
                                    }
                                    span { class: "list-item-meta", "{session.created_at}" }
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
                            sessions.set(None);
                            fetch_sessions.call(new_page);
                        },
                }
            },
        }
    }
}
