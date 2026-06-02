use dioxus::prelude::*;

pub mod connection;
pub mod selection;
pub mod toast;

use connection::ConnectionState;
use selection::{InspectorSection, SelectionState};
use taicho::domain::WorkspaceInfo;
pub use toast::{Toast, ToastKind};

#[derive(Clone, Copy)]
pub struct AppState {
    pub connection: Signal<ConnectionState>,
    pub selected_section: Signal<InspectorSection>,
    pub selection: SelectionState,
    pub status_message: Signal<String>,
    pub workspace_info: Signal<Option<WorkspaceInfo>>,
    // Dioxus use_context: clippy does not detect signal reads through rsx!
    #[allow(dead_code)]
    pub chat_streaming: Signal<bool>,
    // Dioxus use_context: clippy does not detect signal reads through rsx!
    #[allow(dead_code)]
    pub toasts: Signal<Vec<Toast>>,
    // Dioxus use_context: clippy does not detect signal reads through rsx!
    #[allow(dead_code)]
    pub search_open: Signal<bool>,
}

impl AppState {
    pub fn push_toast(&self, kind: ToastKind, msg: impl Into<String>) {
        let mut toasts = self.toasts;
        toasts.write().push(Toast::new(kind, msg));
    }

    pub fn dismiss_toast(&self, id: u32) {
        let mut toasts = self.toasts;
        toasts.write().retain(|t| t.id != id);
    }

    #[allow(dead_code)]
    pub fn clear_toasts(&self) {
        let mut toasts = self.toasts;
        toasts.write().clear();
    }
}
