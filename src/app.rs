use dioxus::prelude::*;

use crate::actor;
use crate::state::AppState;
use crate::state::connection::ConnectionState;
use crate::state::selection::{InspectorSection, SelectionState};
use crate::ui::RootShell;

#[component]
pub fn App() -> Element {
    let connection = use_signal(ConnectionState::default);
    let selected_section = use_signal(InspectorSection::default);
    let peer_id = use_signal(|| None);
    let session_id = use_signal(|| None);
    let message_id = use_signal(|| None);
    let conclusion_id = use_signal(|| None);
    let selection = SelectionState::new(peer_id, session_id, message_id, conclusion_id);
    let status_message = use_signal(|| "Not connected".to_string());
    let workspace_info = use_signal(|| None::<taicho::domain::WorkspaceInfo>);

    let _actor = use_coroutine(actor::run_honcho_actor);

    use_context_provider(|| AppState {
        connection,
        selected_section,
        selection,
        status_message,
        workspace_info,
    });

    rsx! {
        link { rel: "stylesheet", href: "/assets/styles.css" }
        RootShell {}
    }
}
