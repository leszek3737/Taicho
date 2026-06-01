use dioxus::prelude::*;

pub mod connection;
pub mod selection;

use connection::ConnectionState;
use selection::{InspectorSection, SelectionState};
use taicho::domain::WorkspaceInfo;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct AppState {
    pub connection: Signal<ConnectionState>,
    pub selected_section: Signal<InspectorSection>,
    pub selection: SelectionState,
    pub status_message: Signal<String>,
    pub workspace_info: Signal<Option<WorkspaceInfo>>,
}
