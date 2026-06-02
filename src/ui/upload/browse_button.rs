use dioxus::prelude::*;
use rfd::AsyncFileDialog;
use std::path::PathBuf;

#[component]
pub fn BrowseButton(on_picked: EventHandler<Vec<PathBuf>>) -> Element {
    let pick = move |_| {
        let dialog = AsyncFileDialog::new();
        let on_picked = on_picked;
        spawn(async move {
            if let Some(paths) = dialog.pick_files().await {
                let paths: Vec<PathBuf> = paths.into_iter().map(Into::into).collect();
                if !paths.is_empty() {
                    on_picked.call(paths);
                }
            }
        });
    };

    rsx! {
        button { class: "btn-secondary", onclick: pick, "Browse..." }
    }
}
