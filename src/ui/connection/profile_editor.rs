use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;
use crate::state::connection::ConnectionState;
use taicho::persistence::ConnectionProfile;
use taicho::persistence::profile_store::validate_profile;

#[component]
pub fn ConnectionScreen() -> Element {
    let mut base_url = use_signal(|| "http://localhost:8000".to_string());
    let mut workspace_id = use_signal(|| "default".to_string());
    let mut api_key = use_signal(String::new);
    let mut uses_api_key = use_signal(|| false);
    let mut connecting = use_signal(|| false);
    let mut error_msg = use_signal(|| None::<String>);

    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();
    let mut state: AppState = use_context();

    let conn_error: Option<String> = match &*state.connection.read() {
        ConnectionState::Failed { message, .. } => Some(message.clone()),
        _ => None,
    };

    rsx! {
        main { class: "connection-page",
            section { class: "connection-card",
                div { class: "connection-kicker", "Taicho" }
                h1 { "Connect to Honcho" }
                p { "Enter your Honcho server details to connect." }

                label { class: "field",
                    span { "Base URL" }
                    input {
                        value: "{base_url}",
                        oninput: move |e| base_url.set(e.value()),
                        disabled: *connecting.read(),
                    }
                }

                label { class: "field",
                    span { "Workspace ID" }
                    input {
                        value: "{workspace_id}",
                        oninput: move |e| workspace_id.set(e.value()),
                        disabled: *connecting.read(),
                    }
                }

                label { class: "check-field",
                    input {
                        r#type: "checkbox",
                        checked: *uses_api_key.read(),
                        onchange: move |e| uses_api_key.set(e.checked()),
                        disabled: *connecting.read(),
                    }
                    span { "Use API key" }
                }

                if *uses_api_key.read() {
                    label { class: "field",
                        span { "API Key" }
                        input {
                            r#type: "password",
                            value: "{api_key}",
                            oninput: move |e| api_key.set(e.value()),
                            disabled: *connecting.read(),
                        }
                    }
                }

                if let Some(msg) = error_msg.read().as_ref() {
                    p { class: "error-text", "{msg}" }
                }

                if let Some(msg) = &conn_error {
                    p { class: "error-text", "{msg}" }
                }

                // Double-click guard: the `connecting` signal disables this button,
                // preventing duplicate submissions at the UI level.
                button {
                    class: "primary-button",
                    onclick: move |_| {
                        let base = base_url.read().trim().to_string();
                        let ws = workspace_id.read().trim().to_string();
                        let key = api_key.read().trim().to_string();
                        let use_key = *uses_api_key.read();

                        if !(base.starts_with("http://") || base.starts_with("https://")) {
                            error_msg.set(Some("Base URL must start with http:// or https://".to_string()));
                            return;
                        }

                        let mut profile = ConnectionProfile::new(
                            "Default".to_string(),
                            base,
                            ws,
                            use_key,
                        );

                        // Validate (also normalizes/trims fields in place)
                        if let Err(e) = validate_profile(&mut profile) {
                            error_msg.set(Some(e.user_message()));
                            return;
                        }

                        // Keyring integration: for NEW (unsaved) profiles we don't have
                        // a profile.id to look up a stored key. Keyring load for saved
                        // profiles is planned for M1-D (future milestone).
                        let actual_key = if use_key {
                            if key.is_empty() {
                                // No key entered and no saved profile to load from keyring.
                                // Proceed without a key (server may not require one).
                                None
                            } else {
                                Some(key)
                            }
                        } else {
                            None
                        };

                        connecting.set(true);
                        error_msg.set(None);
                        state.connection.set(ConnectionState::Connecting);

                        let (tx, rx) = tokio::sync::oneshot::channel();
                        actor.send(Cmd::Connect {
                            profile,
                            api_key: actual_key,
                            reply: tx,
                        });

                        let mut conn = state.connection;
                        let mut status = state.status_message;
                        let mut connecting_sig = connecting;
                        let mut ws_info = state.workspace_info;

                        spawn(async move {
                            match rx.await {
                                Ok(Ok(info)) => {
                                    conn.set(ConnectionState::Connected);
                                    status.set(format!("Connected to {}", info.base_url));
                                    ws_info.set(Some(info));
                                }
                                Ok(Err(e)) => {
                                    conn.set(ConnectionState::Failed {
                                        code: e.code().to_string(),
                                        message: e.user_message(),
                                        retryable: e.is_retryable(),
                                    });
                                    status.set(format!("Error: {}", e.user_message()));
                                }
                                Err(_) => {
                                    conn.set(ConnectionState::Failed {
                                        code: "channel_closed".to_string(),
                                        message: "Request was canceled".to_string(),
                                        retryable: false,
                                    });
                                    status.set("Error: Request was canceled".to_string());
                                }
                            }
                            connecting_sig.set(false);
                        });
                    },
                    disabled: *connecting.read(),
                    if *connecting.read() {
                        "Connecting..."
                    } else {
                        "Connect"
                    }
                }
            }
        }
    }
}
