use dioxus::prelude::*;

use crate::state::AppState;
use crate::state::toast::ToastKind;

#[component]
pub fn ToastContainer() -> Element {
    let state: AppState = use_context();
    let toasts = state.toasts.read().clone();

    rsx! {
        div { class: "toast-container",
            for t in toasts.iter() {
                div {
                    key: "{t.id}",
                    class: match t.kind {
                        ToastKind::Info => "toast toast-info",
                        ToastKind::Warning => "toast toast-warning",
                        ToastKind::Error => "toast toast-error",
                    },
                    role: match t.kind {
                        ToastKind::Info | ToastKind::Warning => "status",
                        ToastKind::Error => "alert",
                    },
                    aria_live: match t.kind {
                        ToastKind::Info | ToastKind::Warning => "polite",
                        ToastKind::Error => "assertive",
                    },
                    span { class: "toast-message", "{t.message}" }
                    button {
                        class: "toast-close",
                        onclick: {
                            let tid = t.id;
                            move |_| {
                                state.dismiss_toast(tid);
                            }
                        },
                        "x"
                    }
                }
            }
        }
    }
}
