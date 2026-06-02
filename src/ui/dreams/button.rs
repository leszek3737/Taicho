use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::{AppState, ToastKind};

#[component]
pub fn DreamButton(observer_id: String) -> Element {
    let state = use_context::<AppState>();
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut dialog_open = use_signal(|| false);
    let mut session = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let mut reset_form = move || {
        session.set(String::new());
        error.set(None);
    };

    let mut close_dialog = move || {
        dialog_open.set(false);
        reset_form();
    };

    let submit = {
        let observer_id = observer_id.clone();
        move |_| {
            let session_trimmed = session.read().trim().to_string();

            submitting.set(true);
            error.set(None);
            let (tx, rx) = tokio::sync::oneshot::channel();
            actor.send(Cmd::ScheduleDream {
                session_id: session_trimmed,
                observer_id: Some(observer_id.clone()),
                reply: tx,
            });
            spawn(async move {
                let outcome = rx.await;
                if !*dialog_open.read() {
                    submitting.set(false);
                    return;
                }
                match outcome {
                    Ok(Ok(())) => {
                        state.push_toast(ToastKind::Info, "Dream scheduled");
                        close_dialog();
                    }
                    Ok(Err(e)) => error.set(Some(e.user_message())),
                    Err(_) => error.set(Some("Channel closed".to_string())),
                }
                submitting.set(false);
            });
        }
    };

    let on_key = move |e: KeyboardEvent| {
        if e.key() == Key::Escape {
            close_dialog();
        }
    };

    rsx! {
        button {
            class: "btn-primary dream-button",
            onclick: move |_| {
                reset_form();
                dialog_open.set(true);
            },
            "Schedule dream"
        }
        if *dialog_open.read() {
            div {
                class: "modal-backdrop",
                onclick: move |_| close_dialog(),
            }
            div {
                class: "modal modal-dream",
                tabindex: "0",
                onkeydown: on_key,
                h2 { "Schedule dream" }
                p {
                    "Observer: "
                    span { class: "monospace", "{observer_id}" }
                }
                input {
                    class: "dream-session",
                    placeholder: "Session ID (optional)",
                    value: "{session.read()}",
                    oninput: move |e| session.set(e.value()),
                    onkeydown: on_key,
                }
                if let Some(err) = error.read().as_ref() {
                    div { class: "error-banner", "{err}" }
                }
                div { class: "modal-actions",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| close_dialog(),
                        "Cancel"
                    }
                    button {
                        class: "btn-primary",
                        disabled: *submitting.read(),
                        onclick: submit,
                        if *submitting.read() { "Scheduling..." } else { "Schedule" }
                    }
                }
            }
        }
    }
}
