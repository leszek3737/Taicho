use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_TOAST_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn new(kind: ToastKind, message: impl Into<String>) -> Self {
        Self {
            id: NEXT_TOAST_ID.fetch_add(1, Ordering::Relaxed),
            kind,
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn info(msg: impl Into<String>) -> Self {
        Self::new(ToastKind::Info, msg)
    }

    #[allow(dead_code)]
    pub fn warning(msg: impl Into<String>) -> Self {
        Self::new(ToastKind::Warning, msg)
    }

    #[allow(dead_code)]
    pub fn error(msg: impl Into<String>) -> Self {
        Self::new(ToastKind::Error, msg)
    }
}
