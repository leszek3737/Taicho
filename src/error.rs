pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Honcho(#[from] honcho_ai::error::HonchoError),
    #[error(transparent)]
    Keyring(#[from] keyring_core::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("not connected")]
    NotConnected,
    #[error("channel closed during {operation}")]
    ChannelClosed { operation: &'static str },
    #[error("{0}")]
    Validation(String),
}

impl AppError {
    pub fn channel_closed(operation: &'static str) -> Self {
        Self::ChannelClosed { operation }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::Honcho(e) => e.message().to_owned(),
            Self::NotConnected => "Not connected to Honcho".to_owned(),
            Self::ChannelClosed { .. } => "Request was canceled".to_owned(),
            Self::Validation(msg) => msg.clone(),
            Self::Keyring(e) => format!("Keychain error: {e}"),
            Self::Io(e) => format!("I/O error: {e}"),
            Self::Json(e) => format!("JSON error: {e}"),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Honcho(e) => e.code(),
            Self::Keyring(_) => "keyring_error",
            Self::Io(_) => "io_error",
            Self::Json(_) => "json_error",
            Self::NotConnected => "not_connected",
            Self::ChannelClosed { .. } => "channel_closed",
            Self::Validation(_) => "validation_error",
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Honcho(e) if e.is_retryable())
    }

    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            Self::Honcho(e) => e.retry_after(),
            _ => None,
        }
    }
}
