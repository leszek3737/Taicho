use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_TOAST_ID: AtomicU32 = AtomicU32::new(1);

// Used through AppState::push_toast and read in ui/common/toast.rs RSX.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: u32,
    pub kind: ToastKind,
    pub message: String,
}

impl Toast {
    // Used by AppState::push_toast (state/mod.rs).
    pub fn new(kind: ToastKind, message: impl Into<String>) -> Self {
        Self {
            id: NEXT_TOAST_ID.fetch_add(1, Ordering::Relaxed),
            kind,
            message: message.into(),
        }
    }
}
