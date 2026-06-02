use dioxus::prelude::*;

use crate::state::{AppState, Theme};

#[component]
pub fn SettingsPanel(on_close: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();
    let selected_theme = state.theme;

    rsx! {
        div {
            class: "settings-overlay",
            role: "presentation",
            onclick: move |_| on_close.call(()),
        }
        div {
            class: "settings-dialog",
            role: "dialog",
            aria_modal: "true",
            aria_label: "Settings",
            onclick: |e| e.stop_propagation(),
            div { class: "settings-header",
                h2 { "Settings" }
                button {
                    class: "settings-close",
                    aria_label: "Close settings",
                    onclick: move |_| on_close.call(()),
                    "\u{2715}"
                }
            }

            div { class: "settings-section",
                h3 { "Appearance" }
                p { class: "settings-section-desc",
                    "Choose how Taicho looks. System follows your OS setting."
                }
                div { class: "theme-selector",
                    {
                        let mut t = selected_theme;
                        let sys_label = "System";
                        let dark_label = "Dark";
                        let light_label = "Light";
                        let options: &[(Theme, &str)] = &[
                            (Theme::System, sys_label),
                            (Theme::Dark, dark_label),
                            (Theme::Light, light_label),
                        ];
                        rsx! {
                            for &(value, label) in options {
                                label {
                                    key: "{value:?}",
                                    class: "theme-option",
                                    input {
                                        r#type: "radio",
                                        name: "theme",
                                        checked: *t.read() == value,
                                        onchange: move |_| { t.set(value); }
                                    }
                                    span { "{label}" }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "settings-section",
                h3 { "About" }
                p { class: "settings-about",
                    "Taicho v{env!(\"CARGO_PKG_VERSION\")}"
                }
                p { class: "settings-about settings-about-muted",
                    "Honcho inspector \u{2014} native desktop client"
                }
            }
        }
    }
}
