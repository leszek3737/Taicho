use dioxus::prelude::*;
use taicho::domain::QueueStatus;
use taicho::error::AppError;

use crate::actor::commands::Cmd;
use crate::ui::common::{EmptyView, ErrorView, LoadingView};

#[derive(Clone)]
enum QueuePanelState {
    Loaded(QueueStatus),
    Empty,
    Error(String, String, bool),
}

impl QueuePanelState {
    fn from_result(result: Result<QueueStatus, AppError>) -> Self {
        match result {
            Ok(q) if q.is_empty() => Self::Empty,
            Ok(q) => Self::Loaded(q),
            Err(e) => {
                let retryable = e.is_retryable();
                Self::Error(e.code().to_string(), e.user_message(), retryable)
            }
        }
    }
}

#[component]
pub fn QueuePanel() -> Element {
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut local: Signal<Option<QueuePanelState>> = use_signal(|| None);

    let fetch = use_callback(move |_: ()| {
        let (tx, rx) = tokio::sync::oneshot::channel();
        actor.send(Cmd::QueueStatus {
            observer_id: None,
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("queue_status"))
                .and_then(|r| r);
            local.set(Some(QueuePanelState::from_result(result)));
        });
    });

    use_effect(move || {
        fetch.call(());
    });

    let snapshot = local.read().clone();

    rsx! {
        div { class: "queue-panel",
            div { class: "list-toolbar",
                h3 { "Queue status" }
                button {
                    class: "secondary-button",
                    onclick: move |_| {
                        local.set(None);
                        fetch.call(());
                    },
                    "Refresh"
                }
            }
            match snapshot {
                None => rsx! { LoadingView { label: "Loading queue...".to_string() } },
                Some(QueuePanelState::Error(code, message, retryable)) => rsx! {
                    ErrorView {
                        code,
                        message,
                        retryable,
                        on_retry: Some(EventHandler::new(move |_: MouseEvent| {
                            local.set(None);
                            fetch.call(());
                        })),
                    }
                },
                Some(QueuePanelState::Empty) => rsx! {
                    EmptyView {
                        title: "No dreams yet".to_string(),
                        message: "Queue is empty. Schedule a dream to get started.".to_string(),
                    }
                },
                Some(QueuePanelState::Loaded(q)) => rsx! {
                    div { class: "queue-buckets",
                        div { class: "bucket bucket-pending",
                            span { class: "bucket-label", "Pending" }
                            span { class: "bucket-value", "{q.pending}" }
                        }
                        div { class: "bucket bucket-running",
                            span { class: "bucket-label", "Running" }
                            span { class: "bucket-value", "{q.running}" }
                        }
                        div { class: "bucket bucket-completed",
                            span { class: "bucket-label", "Done" }
                            span { class: "bucket-value", "{q.completed}" }
                        }
                        div { class: "bucket bucket-sessions",
                            span { class: "bucket-label", "Sessions" }
                            span { class: "bucket-value", "{q.sessions}" }
                        }
                    }
                },
            }
        }
    }
}
