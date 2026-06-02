use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use taicho::domain::{ConclusionRow, DomainPage};
use taicho::error::AppError;

use super::super::common::pagination::Pagination;
use super::super::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone)]
enum ConclusionListState {
    Loaded(DomainPage<ConclusionRow>),
    Queried(Vec<ConclusionRow>),
    Empty,
    Error(String, String, bool),
}

impl ConclusionListState {
    fn from_page_result(result: Result<DomainPage<ConclusionRow>, AppError>) -> Self {
        match result {
            Ok(page) if page.is_empty() => Self::Empty,
            Ok(page) => Self::Loaded(page),
            Err(e) => Self::Error(e.code().to_string(), e.user_message(), e.is_retryable()),
        }
    }

    fn from_query_result(result: Result<Vec<ConclusionRow>, AppError>) -> Self {
        match result {
            Ok(items) if items.is_empty() => Self::Empty,
            Ok(items) => Self::Queried(items),
            Err(e) => Self::Error(e.code().to_string(), e.user_message(), e.is_retryable()),
        }
    }
}

#[component]
pub fn ConclusionList() -> Element {
    let state: AppState = use_context();
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();

    let mut observer_id: Signal<String> = use_signal(String::new);
    let mut observed_id: Signal<String> = use_signal(String::new);
    let mut search_query: Signal<String> = use_signal(String::new);
    let mut conclusions: Signal<Option<ConclusionListState>> = use_signal(|| None);
    let mut current_page: Signal<u64> = use_signal(|| 1);
    let confirm_delete_id: Signal<Option<String>> = use_signal(|| None);
    let mut repr_text: Signal<Option<String>> = use_signal(|| None);
    let mut repr_error: Signal<Option<String>> = use_signal(|| None);
    let mut repr_loading: Signal<bool> = use_signal(|| false);

    let fetch_list = use_callback(move |(obs_id, obd_id, page): (String, String, u64)| {
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::ListConclusions {
            observer_id: obs_id,
            observed_id: obd_id,
            page,
            size: 50,
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("list_conclusions"))
                .and_then(|r| r);
            conclusions.set(Some(ConclusionListState::from_page_result(result)));
        });
    });

    let do_query = use_callback(move |(obs_id, obd_id, query): (String, String, String)| {
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::QueryConclusions {
            observer_id: obs_id,
            observed_id: obd_id,
            query,
            top_k: 20,
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("query_conclusions"))
                .and_then(|r| r);
            conclusions.set(Some(ConclusionListState::from_query_result(result)));
        });
    });

    let snapshot = conclusions.read().clone();

    let items: Vec<ConclusionRow> = match &snapshot {
        Some(ConclusionListState::Loaded(page)) => page.items.clone(),
        Some(ConclusionListState::Queried(vec)) => vec.clone(),
        _ => Vec::new(),
    };

    let page_info = match &snapshot {
        Some(ConclusionListState::Loaded(page)) => Some(page.info.clone()),
        _ => None,
    };

    // If both IDs are empty, show informational message instead of loading
    let ids_empty = observer_id.read().is_empty() && observed_id.read().is_empty();

    rsx! {
        div { class: "list-toolbar",
            h2 { "Conclusions" }
            button {
                class: "secondary-button",
                onclick: move |_| {
                    let obs = observer_id.read().clone();
                    let obd = observed_id.read().clone();
                    if !obs.is_empty() && !obd.is_empty() {
                        conclusions.set(None);
                        search_query.set(String::new());
                        fetch_list.call((obs, obd, *current_page.read()));
                    }
                },
                "Refresh"
            }
        }

        div { class: "field-group",
            label { class: "field",
                span { "Observer ID" }
                input {
                    value: "{observer_id}",
                    oninput: move |e| observer_id.set(e.value()),
                }
            }
            label { class: "field",
                span { "Observed ID" }
                input {
                    value: "{observed_id}",
                    oninput: move |e| observed_id.set(e.value()),
                }
            }
            button {
                class: "primary-button",
                disabled: observer_id.read().is_empty() || observed_id.read().is_empty(),
                onclick: move |_| {
                    let obs = observer_id.read().clone();
                    let obd = observed_id.read().clone();
                    conclusions.set(None);
                    current_page.set(1);
                    fetch_list.call((obs, obd, 1));
                },
                "Load"
            }
        }

        div { class: "search-bar",
            input {
                r#type: "text",
                placeholder: "Semantic search...",
                value: "{search_query}",
                oninput: move |e| search_query.set(e.value()),
                disabled: observer_id.read().is_empty() || observed_id.read().is_empty(),
            }
            button {
                class: "secondary-button",
                disabled: search_query.read().is_empty()
                    || observer_id.read().is_empty()
                    || observed_id.read().is_empty(),
                onclick: move |_| {
                    let obs = observer_id.read().clone();
                    let obd = observed_id.read().clone();
                    let q = search_query.read().clone();
                    if !q.is_empty() {
                        conclusions.set(None);
                        do_query.call((obs, obd, q));
                    }
                },
                "Search"
            }
        }

        if !ids_empty {
            div { class: "conclusion-representation",
                div { class: "conclusion-repr-header",
                    h3 { "Representation" }
                    button {
                        class: "secondary-button",
                        disabled: *repr_loading.read()
                            || observer_id.read().is_empty()
                            || observed_id.read().is_empty(),
                        onclick: move |_| {
                            let obs = observer_id.read().clone();
                            let obd = observed_id.read().clone();
                            if !obs.is_empty() && !obd.is_empty() {
                                repr_loading.set(true);
                                repr_error.set(None);
                                repr_text.set(None);
                                let (tx, rx) = tokio::sync::oneshot::channel();
                                actor.send(Cmd::GetConclusionRepresentation {
                                    observer_id: obs,
                                    observed_id: obd,
                                    reply: tx,
                                });
                                spawn(async move {
                                    let result = rx
                                        .await
                                        .map_err(|_| AppError::channel_closed("get_conclusion_representation"))
                                        .and_then(|r| r);
                                    repr_loading.set(false);
                                    match result {
                                        Ok(text) => repr_text.set(Some(text)),
                                        Err(e) => repr_error.set(Some(e.user_message())),
                                    }
                                });
                            }
                        },
                        if *repr_loading.read() { "Generating..." } else { "Generate" }
                    }
                }
                if let Some(text) = repr_text.read().as_ref() {
                    pre { class: "representation-view", "{text}" }
                }
                if let Some(err) = repr_error.read().as_ref() {
                    p { class: "error-text", "{err}" }
                }
            }
        }

        if ids_empty && snapshot.is_none() {
            EmptyView {
                title: "Conclusions".to_string(),
                message: "Enter observer and observed peer IDs to load conclusions.".to_string(),
            }
        } else {
            match snapshot {
                None => rsx! { LoadingView { label: "Loading conclusions...".to_string() } },
                Some(ConclusionListState::Error(code, message, retryable)) => rsx! {
                    ErrorView {
                        code,
                        message,
                        retryable,
                        on_retry: Some(EventHandler::new(move |_: MouseEvent| {
                            let obs = observer_id.read().clone();
                            let obd = observed_id.read().clone();
                            if !obs.is_empty() && !obd.is_empty() {
                                conclusions.set(None);
                                fetch_list.call((obs, obd, *current_page.read()));
                            }
                        })),
                    }
                },
                Some(ConclusionListState::Empty) => rsx! {
                    EmptyView {
                        title: "No conclusions".to_string(),
                        message: "No conclusions found for this peer pair.".to_string(),
                    }
                },
                Some(ConclusionListState::Loaded(_) | ConclusionListState::Queried(_)) => rsx! {
                    div { class: "list-items",
                        for conc in items {
                            { rsx! {
                                ConclusionItem {
                                    key: "{conc.id}",
                                    conc,
                                    actor,
                                    confirm_delete_id,
                                    conclusion_id: state.selection.conclusion_id,
                                    on_delete_success: EventHandler::new(move |_: ()| {
                                        let obs = observer_id.read().clone();
                                        let obd = observed_id.read().clone();
                                        if !obs.is_empty() && !obd.is_empty() {
                                            conclusions.set(None);
                                            fetch_list.call((obs, obd, *current_page.read()));
                                        }
                                    }),
                                }
                            }}
                        }
                    }
                    if let Some(info) = page_info {
                        { rsx! {
                            Pagination {
                                page: info.page,
                                pages: info.pages,
                                on_page_change: move |new_page: u64| {
                                    current_page.set(new_page);
                                    let obs = observer_id.read().clone();
                                    let obd = observed_id.read().clone();
                                    conclusions.set(None);
                                    fetch_list.call((obs, obd, new_page));
                                },
                            }
                        }}
                    }
                },
            }
        }
    }
}

#[component]
fn ConclusionItem(
    conc: ConclusionRow,
    actor: Coroutine<Cmd>,
    confirm_delete_id: Signal<Option<String>>,
    conclusion_id: Signal<Option<String>>,
    on_delete_success: EventHandler<()>,
) -> Element {
    let state: AppState = use_context();

    let is_confirming = confirm_delete_id
        .read()
        .as_deref()
        .is_some_and(|id| id == conc.id);

    rsx! {
        div {
            class: "list-item conclusion-item",

            div {
                class: "conclusion-item-content",
                div { class: "message-row-header",
                    span { class: "list-item-id", "{conc.id}" }
                    span { class: "list-item-meta", "{conc.created_at}" }
                }
                div { class: "message-preview", "{conc.content}" }
            }

            if is_confirming {
                button {
                    class: "danger-button",
                    onclick: {
                        let conc = conc.clone();
                        move |_| {
                            let (tx, rx) = tokio::sync::oneshot::channel();
                            actor.send(Cmd::DeleteConclusion {
                                conclusion_id: conc.id.clone(),
                                observer_id: conc.observer_id.clone(),
                                observed_id: conc.observed_id.clone(),
                                reply: tx,
                            });
                            spawn(async move {
                                match rx.await {
                                    Ok(Ok(())) => {
                                        confirm_delete_id.set(None);
                                        on_delete_success.call(());
                                    }
                                    Ok(Err(e)) => {
                                        let mut status = state.status_message;
                                        status.set(format!("Delete failed: {}", e.user_message()));
                                        confirm_delete_id.set(None);
                                    }
                                    Err(_) => {
                                        let mut status = state.status_message;
                                        status.set("Delete failed: channel closed".to_string());
                                        confirm_delete_id.set(None);
                                    }
                                }
                            });
                        }
                    },
                    "Confirm"
                }
                button {
                    class: "secondary-button",
                    onclick: move |_| confirm_delete_id.set(None),
                    "Cancel"
                }
            } else {
                button {
                    class: "danger-button",
                    onclick: {
                        let id = conc.id.clone();
                        move |_| {
                            confirm_delete_id.set(Some(id.clone()));
                        }
                    },
                    "Delete"
                }
            }
        }
    }
}
