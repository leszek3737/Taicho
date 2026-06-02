use dioxus::events::KeyboardEvent;
use dioxus::prelude::*;

use crate::actor::commands::Cmd;
use crate::state::AppState;

#[component]
pub fn KeyboardShortcuts(children: Element) -> Element {
    let mut state: AppState = use_context();
    let coroutine: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();

    rsx! {
        div {
            class: "shortcuts-capture",
            onkeydown: move |e: KeyboardEvent| {
                let modifiers = e.modifiers();
                let meta = modifiers.contains(Modifiers::META) || modifiers.contains(Modifiers::SUPER);
                let shift = modifiers.contains(Modifiers::SHIFT);
                let key = e.key();

                if meta && !shift {
                    match &key {
                        Key::Character(k) if k.eq_ignore_ascii_case("k") => {
                            e.prevent_default();
                            let cur = *state.search_open.read();
                            state.search_open.set(!cur);
                        }
                        Key::Character(k) if k.eq_ignore_ascii_case("r") => {
                            e.prevent_default();
                            let (tx, rx) = tokio::sync::oneshot::channel();
                            coroutine.send(Cmd::Refresh { reply: tx });
                            spawn(async move { let _ = rx.await; });
                        }
                        _ => {}
                    }
                }

                if meta
                    && shift
                    && let Key::Character(k) = &key
                    && k.eq_ignore_ascii_case("d")
                {
                    e.prevent_default();
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    coroutine.send(Cmd::Disconnect { reply: tx });
                    spawn(async move { let _ = rx.await; });
                }

                if key == Key::Escape && *state.search_open.read() {
                    state.search_open.set(false);
                }
            },
            {children}
        }
    }
}
