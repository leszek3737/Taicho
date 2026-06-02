use dioxus::prelude::*;
use taicho::domain::raw_json::RawJson;

#[component]
pub fn MetadataEditor(
    value: Signal<Option<RawJson>>,
    on_change: EventHandler<Option<RawJson>>,
) -> Element {
    let text = use_memo(move || {
        value
            .read()
            .as_ref()
            .and_then(|v| serde_json::to_string_pretty(v).ok())
            .unwrap_or_default()
    });
    let mut error: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div { class: "metadata-editor",
            label { "Metadata (JSON object, optional)" }
            textarea {
                class: "metadata-textarea",
                placeholder: "{{ }}",
                value: "{text.read()}",
                oninput: move |e| {
                    let s = e.value();
                    if s.trim().is_empty() {
                        error.set(None);
                        on_change.call(None);
                        return;
                    }
                    match serde_json::from_str::<RawJson>(&s) {
                        Ok(v) => {
                            error.set(None);
                            on_change.call(Some(v));
                        }
                        Err(err) => {
                            error.set(Some(format!("Invalid JSON: {err}")));
                        }
                    }
                },
            }
            if let Some(err) = error.read().as_ref() {
                div { class: "error-banner", "{err}" }
            }
        }
    }
}
