pub mod json_viewer;

pub mod pagination;

use dioxus::prelude::*;

#[component]
pub fn LoadingView(label: String) -> Element {
    rsx! {
        section { class: "state state-loading", aria_label: "{label}",
            div { class: "skeleton skeleton-title" }
            div { class: "skeleton skeleton-line" }
            div { class: "skeleton skeleton-line short" }
        }
    }
}

#[component]
pub fn EmptyView(title: String, message: String) -> Element {
    rsx! {
        section { class: "state state-empty",
            div { class: "state-icon", "∅" }
            h2 { "{title}" }
            p { "{message}" }
        }
    }
}

#[component]
pub fn ErrorView(
    code: String,
    message: String,
    retryable: bool,
    on_retry: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        section { class: "state state-error",
            div { class: "error-code", "{code}" }
            p { "{message}" }
            if retryable {
                if let Some(handler) = on_retry {
                    button {
                        class: "primary-button",
                        onclick: move |evt| handler.call(evt),
                        "Retry"
                    }
                }
            }
        }
    }
}
