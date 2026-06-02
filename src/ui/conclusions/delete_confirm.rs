use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use taicho::error::AppError;

#[component]
pub fn DeleteConclusionConfirm(
    peer_id: String,
    observed_id: Option<String>,
    conclusion_id: String,
    on_done: EventHandler<bool>,
) -> Element {
    let coroutine = use_coroutine_handle::<Cmd>();
    let mut deleting = use_signal(|| false);
    let mut typed = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let id_for_send = conclusion_id.clone();
    let obs_for_send = peer_id.clone();
    let obd_for_send = observed_id.clone().unwrap_or_else(|| peer_id.clone());
    let confirm = move |_| {
        error.set(None);
        deleting.set(true);
        let (tx, rx) = tokio::sync::oneshot::channel();
        coroutine.send(Cmd::DeleteConclusion {
            conclusion_id: id_for_send.clone(),
            observer_id: obs_for_send.clone(),
            observed_id: obd_for_send.clone(),
            reply: tx,
        });
        spawn(async move {
            let result = rx
                .await
                .map_err(|_| AppError::channel_closed("delete_conclusion"))
                .and_then(|r| r);
            match result {
                Ok(()) => on_done.call(true),
                Err(e) => {
                    error.set(Some(e.user_message()));
                    deleting.set(false);
                }
            }
        });
    };

    let id_for_check = conclusion_id.clone();
    let id_for_display = conclusion_id.clone();
    let can_delete = move || *typed.read() == id_for_check && !*deleting.read();

    rsx! {
        div {
            class: "modal-backdrop",
            onclick: move |_| on_done.call(false),
        }
        div { class: "modal modal-confirm",
            h2 { "Delete conclusion?" }
            p { "This will permanently remove the conclusion." }
            p { class: "monospace", "{id_for_display}" }
            p { "Type the conclusion ID to confirm deletion." }
            input {
                class: "confirm-input",
                placeholder: "Type conclusion ID to confirm",
                value: "{typed}",
                oninput: move |e| typed.set(e.value()),
            }
            if let Some(err) = error.read().as_ref() {
                div { class: "error-banner", "{err}" }
            }
            div { class: "modal-actions",
                button {
                    class: "btn-secondary",
                    disabled: *deleting.read(),
            onclick: move |_| {
                if !*deleting.read() {
                    on_done.call(false);
                }
            },
                    "Cancel"
                }
                button {
                    class: "btn-danger",
                    disabled: !can_delete(),
                    onclick: confirm,
                    if *deleting.read() { "Deleting..." } else { "Delete" }
                }
            }
        }
    }
}
