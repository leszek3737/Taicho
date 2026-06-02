use dioxus::html::HasFileData;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn DragDropZone(on_files: EventHandler<Vec<PathBuf>>, accept: Option<String>) -> Element {
    let mut hover = use_signal(|| false);
    let accept_attr = accept.as_deref().unwrap_or("*/*");

    rsx! {
        div {
            class: if *hover.read() { "drop-zone drop-zone-active" } else { "drop-zone" },
            ondragover: move |e: DragEvent| {
                e.prevent_default();
                hover.set(true);
            },
            ondragleave: move |_| hover.set(false),
            ondrop: move |e: DragEvent| {
                e.prevent_default();
                hover.set(false);
                let paths: Vec<PathBuf> = e.files().into_iter().map(|f| f.path()).collect();
                if !paths.is_empty() {
                    on_files.call(paths);
                }
            },
            div { class: "drop-zone-icon", "[drag here]" }
            div { class: "drop-zone-text", "Drop files here, or use Browse button" }
            div { class: "drop-zone-accept", "Accepts: {accept_attr}" }
        }
    }
}
