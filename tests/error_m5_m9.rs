#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr
)]

use taicho::error::AppError;

#[test]
fn validation_display() {
    let e = AppError::Validation("bad input".into());
    assert!(e.to_string().contains("bad input"));
}

#[test]
fn not_connected_display() {
    let e = AppError::NotConnected;
    assert!(e.to_string().contains("not connected"));
}

#[test]
fn channel_closed_display() {
    let e = AppError::channel_closed("list_peers");
    assert!(e.to_string().contains("list_peers"));
}

#[test]
fn validation_is_not_retryable() {
    let e = AppError::Validation("x".into());
    assert!(!e.is_retryable());
}

#[test]
fn not_connected_is_not_retryable() {
    let e = AppError::NotConnected;
    assert!(!e.is_retryable());
}

#[test]
fn validation_code() {
    let e = AppError::Validation("x".into());
    assert_eq!(e.code(), "validation_error");
}

#[test]
fn not_connected_code() {
    let e = AppError::NotConnected;
    assert_eq!(e.code(), "not_connected");
}

#[test]
fn channel_closed_code() {
    let e = AppError::channel_closed("op");
    assert_eq!(e.code(), "channel_closed");
}

#[test]
fn code_returns_distinct_codes() {
    let codes = [
        AppError::NotConnected.code(),
        AppError::channel_closed("a").code(),
        AppError::Validation("a".into()).code(),
    ];
    let unique: std::collections::HashSet<_> = codes.iter().collect();
    assert_eq!(unique.len(), codes.len(), "all codes should be distinct");
}
