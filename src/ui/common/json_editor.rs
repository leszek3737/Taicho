use dioxus::prelude::*;

use taicho::domain::raw_json::JsonMap;

#[component]
pub fn JsonEditor(
    initial: Option<JsonMap>,
    label: String,
    on_change: EventHandler<JsonMap>,
    on_cancel: EventHandler<()>,
    #[props(default = false)] saving: bool,
) -> Element {
    let initial_signal: Memo<Option<JsonMap>> = use_memo(move || initial.clone());
    let mut editing_text: Signal<String> = use_signal(String::new);
    let mut editing: Signal<bool> = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);
    let mut last_initial: Signal<Option<JsonMap>> = use_signal(|| None);
    use_effect(move || {
        let current_initial = (*initial_signal.read()).clone();
        if current_initial != *last_initial.read() {
            last_initial.set(current_initial.clone());
            editing.set(false);
        }
        if !*editing.read() {
            if let Some(initial) = current_initial.as_ref() {
                let s = serde_json::to_string_pretty(&serde_json::Value::Object(initial.clone()))
                    .unwrap_or_else(|_| "{}".to_string());
                editing_text.set(s);
            }
            error_msg.set(None);
        }
    });

    let is_editing = *editing.read();
    let text_val = editing_text.read().clone();

    if !is_editing {
        rsx! {
            div { class: "json-editor",
                div { class: "json-editor-toolbar",
                    span { class: "json-editor-label", "{label}" }
                    button {
                        class: "secondary-button",
                        onclick: move |_| editing.set(true),
                        "Edit"
                    }
                }
                pre {
                    class: "json-view",
                    {
                        let map = (*initial_signal.read()).clone().unwrap_or_default();
                        serde_json::to_string_pretty(&serde_json::Value::Object(map))
                            .unwrap_or_else(|_| "{}".to_string())
                    }
                }
            }
        }
    } else {
        rsx! {
            div { class: "json-editor",
                div { class: "json-editor-toolbar",
                    span { class: "json-editor-label", "{label} (editing)" }
                }
                if let Some(err) = &*error_msg.read() {
                    p { class: "json-editor-error", "{err}" }
                }
                textarea {
                    class: "json-editor-textarea",
                    value: text_val,
                    oninput: move |evt| {
                        editing_text.set(evt.value());
                        error_msg.set(None);
                    },
                    rows: 16,
                }
                div { class: "json-editor-actions",
                    button {
                        class: "primary-button",
                        disabled: saving,
                        onclick: move |_| {
                            if saving {
                                return;
                            }
                            let text = editing_text.read().clone();
                            match serde_json::from_str::<serde_json::Value>(&text) {
                                Ok(serde_json::Value::Object(map)) => {
                                    on_change.call(map);
                                    error_msg.set(None);
                                }
                                Ok(_) => {
                                    error_msg.set(Some("Expected a JSON object (key-value pairs)".to_string()));
                                }
                                Err(e) => {
                                    error_msg.set(Some(format!("Invalid JSON: {e}")));
                                }
                            }
                        },
                        if saving { "Saving..." } else { "Save" }
                    }
                    button {
                        class: "secondary-button",
                        disabled: saving,
                        onclick: move |_| {
                            if saving {
                                return;
                            }
                            let map = (*initial_signal.read()).clone().unwrap_or_default();
                            let s = serde_json::to_string_pretty(&serde_json::Value::Object(map))
                                .unwrap_or_else(|_| "{}".to_string());
                            editing_text.set(s);
                            on_cancel.call(());
                            editing.set(false);
                            error_msg.set(None);
                        },
                        "Cancel"
                    }
                }
            }
        }
    }
}
