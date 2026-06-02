use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use taicho::domain::conclusion::ConclusionInput;
use taicho::domain::raw_json::JsonMap;
use taicho::error::AppError;

#[component]
pub fn CreateConclusionModal(
    peer_id: String,
    observed_id: Option<String>,
    on_close: EventHandler<()>,
    on_created: EventHandler<()>,
) -> Element {
    let coroutine = use_coroutine_handle::<Cmd>();
    let mut content = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let submit = move |_| {
        let c = content.read().trim().to_string();
        if c.is_empty() {
            return;
        }
        error.set(None);
        submitting.set(true);
        let (tx, rx) = tokio::sync::oneshot::channel();
        coroutine.send(Cmd::CreateConclusion {
            peer_id: peer_id.clone(),
            observed_id: observed_id.clone(),
            input: ConclusionInput {
                content: c,
                metadata: None::<JsonMap>,
            },
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("create_conclusion"))
                .and_then(|r| r);
            match result {
                Ok(_) => on_created.call(()),
                Err(e) => error.set(Some(e.user_message())),
            }
            submitting.set(false);
        });
    };

    rsx! {
        div {
            class: "modal-backdrop",
            onclick: move |_| {
                if !*submitting.read() {
                    on_close.call(());
                }
            },
        }
        div { class: "modal modal-create-conclusion",
            h2 { "New conclusion" }
            textarea {
                class: "conclusion-content",
                placeholder: "Conclusion content...",
                value: "{content.read()}",
                oninput: move |e| content.set(e.value()),
            }
            if let Some(err) = error.read().as_ref() {
                div { class: "error-banner", "{err}" }
            }
            div { class: "modal-actions",
                button {
                    class: "btn-secondary",
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
                button {
                    class: "btn-primary",
                    disabled: *submitting.read() || content.read().trim().is_empty(),
                    onclick: submit,
                    if *submitting.read() { "Creating..." } else { "Create" }
                }
            }
        }
    }
}
