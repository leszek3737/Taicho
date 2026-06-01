use dioxus::prelude::*;

#[component]
pub fn JsonViewer(value: String) -> Element {
    rsx! {
        pre { class: "json-view", "{value}" }
    }
}
