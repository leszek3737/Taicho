use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use taicho::domain::{DomainPage, PeerRow};
use taicho::error::AppError;

use super::super::common::pagination::Pagination;
use super::super::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone)]
enum PeerListState {
    Loaded(DomainPage<PeerRow>),
    Empty,
    Error(String, String, bool),
}

impl PeerListState {
    fn from_result(result: Result<DomainPage<PeerRow>, AppError>) -> Self {
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
pub fn PeerList() -> Element {
    let mut state: AppState = use_context();
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut peers: Signal<Option<PeerListState>> = use_signal(|| None);
    let mut current_page: Signal<u64> = use_signal(|| 1);

    let selected_id = state.selection.peer_id.read().clone();

    let fetch_peers = use_callback(move |page: u64| {
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::ListPeers {
            page,
            size: 50,
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("list_peers"))
                .and_then(|r| r);
            peers.set(Some(PeerListState::from_result(result)));
        });
    });

    use_effect(move || {
        if peers.read().is_none() {
            fetch_peers.call(1);
        }
    });

    let snapshot = peers.read().clone();

    rsx! {
        div { class: "list-toolbar",
            h2 { "Peers" }
            button {
                class: "secondary-button",
                onclick: move |_| {
                    peers.set(None);
                    fetch_peers.call(*current_page.read());
                },
                "Refresh"
            }
        }

        match snapshot {
            None => rsx! { LoadingView { label: "Loading peers...".to_string() } },
            Some(PeerListState::Error(code, message, retryable)) => rsx! {
                ErrorView {
                    code,
                    message,
                    retryable,
                    on_retry: Some(EventHandler::new(move |_: MouseEvent| {
                        peers.set(None);
                        fetch_peers.call(*current_page.read());
                    })),
                }
            },
            Some(PeerListState::Empty) => rsx! {
                EmptyView {
                    title: "No peers".to_string(),
                    message: "No peers found on this server.".to_string(),
                }
            },
            Some(PeerListState::Loaded(page)) => rsx! {
                div { class: "list-items",
                    for peer in &page.items {
                        {
                            let peer_id = peer.id.clone();
                            let is_selected = selected_id.as_deref() == Some(&peer.id);
                            rsx! {
                                button {
                                    key: "{peer_id}",
                                    class: if is_selected { "list-item selected" } else { "list-item" },
                                    onclick: move |_| {
                                        state.selection.peer_id.set(Some(peer_id.clone()));
                                    },
                                    span { class: "list-item-id", "{peer.id}" }
                                    span { class: "list-item-meta", "{peer.created_at}" }
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
                            peers.set(None);
                            fetch_peers.call(new_page);
                        },
                }
            },
        }
    }
}
