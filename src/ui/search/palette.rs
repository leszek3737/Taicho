use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use crate::ui::common::{EmptyView, ErrorView, LoadingView};
use taicho::domain::message::MessageRow;
use taicho::domain::search::SearchScope;

#[derive(Clone, Debug, PartialEq)]
enum SearchState {
    Idle,
    Loading,
    Loaded(Vec<MessageRow>),
    Empty,
    Error { message: String, retryable: bool },
}

#[component]
pub fn CommandPalette() -> Element {
    let mut state: AppState = use_context();
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();

    let open = *state.search_open.read();
    let mut query = use_signal(String::new);
    let mut scope = use_signal(|| SearchScope::Workspace);
    let mut scope_input = use_signal(String::new);
    let mut limit = use_signal(|| 25u32);
    let mut search_state = use_signal(|| SearchState::Idle);

    let mut do_search = move |_| {
        let q = query.read().clone();
        if q.trim().is_empty() {
            return;
        }
        let scope_value = match scope.read().clone() {
            SearchScope::Workspace => SearchScope::Workspace,
            SearchScope::Peer(_) => {
                let id = scope_input.read().clone();
                if id.is_empty() {
                    state.push_toast(
                        crate::state::ToastKind::Error,
                        "Peer ID required for peer scope",
                    );
                    return;
                }
                SearchScope::Peer(id)
            }
            SearchScope::Session(_) => {
                let id = scope_input.read().clone();
                if id.is_empty() {
                    state.push_toast(
                        crate::state::ToastKind::Error,
                        "Session ID required for session scope",
                    );
                    return;
                }
                SearchScope::Session(id)
            }
        };
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::Search {
            scope: scope_value,
            query: q,
            limit: Some(*limit.read()),
            reply: tx,
        });
        search_state.set(SearchState::Loading);
        spawn(async move {
            match rx.await {
                Ok(Ok(rows)) => {
                    if rows.is_empty() {
                        search_state.set(SearchState::Empty);
                    } else {
                        search_state.set(SearchState::Loaded(rows));
                    }
                }
                Ok(Err(e)) => {
                    search_state.set(SearchState::Error {
                        message: e.user_message(),
                        retryable: e.is_retryable(),
                    });
                }
                Err(_) => {
                    search_state.set(SearchState::Error {
                        message: "Search cancelled".to_string(),
                        retryable: false,
                    });
                }
            }
        });
    };

    if !open {
        return rsx! { div {} };
    }

    let is_loading = matches!(*search_state.read(), SearchState::Loading);

    let search_results: Vec<MessageRow> = match &*search_state.read() {
        SearchState::Loaded(rows) => rows.clone(),
        _ => Vec::new(),
    };

    rsx! {
        div {
            class: "palette-backdrop",
            onclick: move |_| state.search_open.set(false),
        }
        div { class: "palette",
            div { class: "palette-header",
                input {
                    class: "palette-input",
                    placeholder: "Search messages...",
                    value: "{query.read()}",
                    autofocus: true,
                    oninput: move |e| query.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            do_search(());
                        }
                        if e.key() == Key::Escape {
                            state.search_open.set(false);
                        }
                    },
                }
                button {
                    class: "btn-icon",
                    aria_label: "Close search",
                    onclick: move |_| state.search_open.set(false),
                    "X"
                }
            }
            div { class: "palette-controls",
                select {
                    onchange: move |e| {
                        scope.set(match e.value().as_str() {
                            "workspace" => SearchScope::Workspace,
                            "peer" => SearchScope::Peer(String::new()),
                            "session" => SearchScope::Session(String::new()),
                            _ => SearchScope::Workspace,
                        });
                        scope_input.set(String::new());
                    },
                    option { value: "workspace", "Workspace" }
                    option { value: "peer", "Peer" }
                    option { value: "session", "Session" }
                }
                if !matches!(*scope.read(), SearchScope::Workspace) {
                    input {
                        class: "palette-scope-id",
                        placeholder: match *scope.read() {
                            SearchScope::Peer(_) => "Peer ID",
                            SearchScope::Session(_) => "Session ID",
                            _ => "",
                        },
                        value: "{scope_input.read()}",
                        oninput: move |e| scope_input.set(e.value()),
                    }
                }
                input {
                    class: "palette-limit",
                    r#type: "number",
                    min: "1",
                    max: "100",
                    placeholder: "limit",
                    value: "{limit.read()}",
                    oninput: move |e| limit.set(e.value().parse().unwrap_or(25)),
                }
                button {
                    class: "btn-primary",
                    disabled: is_loading,
                    onclick: move |_| do_search(()),
                    "Search"
                }
            }
            div { class: "palette-results",
                match &*search_state.read() {
                    SearchState::Idle => rsx! {},
                    SearchState::Loading => rsx! {
                        LoadingView { label: "Searching...".to_string() }
                    },
                    SearchState::Loaded(_) => rsx! {
                        for row in search_results.into_iter() {
                            div {
                                class: "result-row",
                                onclick: move |_| {
                                    state.selection.message_id.set(Some(row.id.clone()));
                                    state.search_open.set(false);
                                },
                                div { class: "result-peer", "{row.peer_id}" }
                                div { class: "result-content", "{row.content}" }
                                div { class: "result-meta", "{row.created_at}" }
                            }
                        }
                    },
                    SearchState::Empty => rsx! {
                        EmptyView {
                            title: "No results".to_string(),
                            message: "Try a different query.".to_string(),
                        }
                    },
                    SearchState::Error { message, retryable } => rsx! {
                        ErrorView {
                            code: "search_error".to_string(),
                            message: message.clone(),
                            retryable: *retryable,
                            on_retry: Some(EventHandler::new(move |_: MouseEvent| {
                                do_search(());
                            })),
                        }
                    },
                }
            }
        }
    }
}
