use dioxus::prelude::*;

#[component]
pub fn ConfirmModal(
    title: String,
    message: String,
    expected: String,
    confirm_label: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut typed = use_signal(String::new);

    let expected_for_check = expected.clone();
    let can_confirm = use_memo(move || *typed.read() == expected_for_check);

    rsx! {
        div {
            class: "modal-backdrop",
            onclick: move |_| on_cancel.call(()),
        }
        div { class: "modal modal-confirm",
            h2 { "{title}" }
            p { "{message}" }
            p { class: "monospace", "{expected}" }
            input {
                class: "confirm-input",
                placeholder: "Type to confirm",
                value: "{typed}",
                oninput: move |e| typed.set(e.value()),
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter && *can_confirm.read() {
                        on_confirm.call(());
                    }
                },
            }
            div { class: "modal-actions",
                button {
                    class: "btn-secondary",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                button {
                    class: "btn-danger",
                    disabled: !*can_confirm.read(),
                    onclick: move |_| on_confirm.call(()),
                    "{confirm_label}"
                }
            }
        }
    }
}
