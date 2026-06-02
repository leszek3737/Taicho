use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use taicho::domain::conclusion::ConclusionRow;
use taicho::error::AppError;

#[component]
pub fn ConclusionsQueryBar(
    peer_id: String,
    observed_id: Option<String>,
    on_results: EventHandler<Vec<ConclusionRow>>,
) -> Element {
    let coroutine = use_coroutine_handle::<Cmd>();
    let mut query = use_signal(String::new);
    let mut top_k = use_signal(|| 10u32);
    let mut searching = use_signal(|| false);
    let mut error_msg = use_signal(|| None::<String>);

    let search = move |_| {
        let q = query.read().trim().to_string();
        if q.is_empty() {
            return;
        }
        let k = *top_k.read();
        if !(1..=100).contains(&k) {
            error_msg.set(Some(format!("top_k must be between 1 and 100, got {k}")));
            return;
        }
        error_msg.set(None);
        searching.set(true);
        let (tx, rx) = tokio::sync::oneshot::channel();
        coroutine.send(Cmd::QueryConclusions {
            observer_id: peer_id.clone(),
            observed_id: observed_id.clone().unwrap_or_else(|| peer_id.clone()),
            query: q,
            top_k: *top_k.read(),
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("query_conclusions"))
                .and_then(|r| r);
            match result {
                Ok(rows) => {
                    error_msg.set(None);
                    on_results.call(rows);
                }
                Err(e) => {
                    error_msg.set(Some(e.user_message()));
                }
            }
            searching.set(false);
        });
    };

    rsx! {
        div { class: "query-bar",
            input {
                class: "query-input",
                placeholder: "Semantic search...",
                value: "{query.read()}",
                oninput: move |e| query.set(e.value()),
            }
            input {
                class: "query-topk",
                r#type: "number",
                placeholder: "top_k",
                value: "{top_k.read()}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<u32>() {
                        top_k.set(v);
                    }
                },
            }
            button {
                class: "btn-primary",
                disabled: *searching.read() || query.read().trim().is_empty(),
                onclick: search,
                if *searching.read() { "Searching..." } else { "Search" }
            }
        }
        if let Some(err) = error_msg.read().as_ref() {
            div { class: "error-banner", "{err}" }
        }
    }
}
