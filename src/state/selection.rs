use dioxus::prelude::*;

/// Per-section selection state. Each section has its own selected item.
#[derive(Clone, Copy)]
pub struct SelectionState {
    pub peer_id: Signal<Option<String>>,
    pub session_id: Signal<Option<String>>,
    pub message_id: Signal<Option<String>>,
    pub conclusion_id: Signal<Option<String>>,
}

impl SelectionState {
    pub fn new(
        peer_id: Signal<Option<String>>,
        session_id: Signal<Option<String>>,
        message_id: Signal<Option<String>>,
        conclusion_id: Signal<Option<String>>,
    ) -> Self {
        Self {
            peer_id,
            session_id,
            message_id,
            conclusion_id,
        }
    }

    #[allow(dead_code)]
    pub fn select_peer(&mut self, id: Option<String>) {
        self.peer_id.set(id);
        self.session_id.set(None);
        self.message_id.set(None);
        self.conclusion_id.set(None);
    }

    #[allow(dead_code)]
    pub fn select_session(&mut self, id: Option<String>) {
        self.session_id.set(id);
        self.message_id.set(None);
        self.conclusion_id.set(None);
    }

    #[allow(dead_code)]
    pub fn select_message(&mut self, id: Option<String>) {
        self.message_id.set(id);
    }

    #[allow(dead_code)]
    pub fn select_conclusion(&mut self, id: Option<String>) {
        self.conclusion_id.set(id);
    }

    pub fn clear(&mut self) {
        self.peer_id.set(None);
        self.session_id.set(None);
        self.message_id.set(None);
        self.conclusion_id.set(None);
    }
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            peer_id: Signal::new(None),
            session_id: Signal::new(None),
            message_id: Signal::new(None),
            conclusion_id: Signal::new(None),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InspectorSection {
    #[default]
    Peers,
    Sessions,
    Messages,
    Conclusions,
    Workspaces,
    RawJson,
}

impl InspectorSection {
    pub const ALL: [Self; 6] = [
        Self::Peers,
        Self::Sessions,
        Self::Messages,
        Self::Conclusions,
        Self::Workspaces,
        Self::RawJson,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Peers => "Peers",
            Self::Sessions => "Sessions",
            Self::Messages => "Messages",
            Self::Conclusions => "Conclusions",
            Self::Workspaces => "Workspaces",
            Self::RawJson => "Raw JSON",
        }
    }
}
