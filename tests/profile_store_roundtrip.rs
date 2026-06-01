#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr
)]

use std::path::PathBuf;

use taicho::persistence::profile_store::{self, ConnectionProfile, ProfileStore, validate_profile};

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("taicho-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn profile_store_round_trips_json() {
    let dir = temp_dir();
    let path = dir.join("profiles.json");

    let mut store = ProfileStore::default();
    let profile = ConnectionProfile::new(
        "Test".to_string(),
        "http://localhost:8000".to_string(),
        "default".to_string(),
        false,
    );
    store.profiles.push(profile.clone());
    store.active_profile_id = Some(profile.id.clone());

    profile_store::save_profiles(&path, &store).unwrap();
    let loaded = profile_store::load_profiles(&path).unwrap();

    assert_eq!(store, loaded);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn missing_profiles_file_returns_default() {
    let dir = temp_dir();
    let path = dir.join("nonexistent.json");

    let store = profile_store::load_profiles(&path).unwrap();
    assert!(store.profiles.is_empty());
    assert!(store.active_profile_id.is_none());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn save_profiles_creates_parent_dir() {
    let dir = temp_dir();
    let nested = dir.join("a").join("b").join("profiles.json");

    let store = ProfileStore::default();
    profile_store::save_profiles(&nested, &store).unwrap();

    assert!(nested.exists());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn validate_rejects_empty_name() {
    let mut profile = ConnectionProfile::new(
        "".to_string(),
        "http://localhost:8000".to_string(),
        "default".to_string(),
        false,
    );
    assert!(profile_store::validate_profile(&mut profile).is_err());
}

#[test]
fn validate_rejects_bad_base_url() {
    let mut profile = ConnectionProfile::new(
        "Test".to_string(),
        "localhost:8000".to_string(),
        "default".to_string(),
        false,
    );
    assert!(profile_store::validate_profile(&mut profile).is_err());
}

#[test]
fn validate_rejects_secret_in_url() {
    let mut profile = ConnectionProfile::new(
        "Test".to_string(),
        "https://api.honcho.dev?HONCHO_API_KEY=sk-123".to_string(),
        "default".to_string(),
        false,
    );
    assert!(profile_store::validate_profile(&mut profile).is_err());
}

#[test]
fn validate_rejects_bad_workspace_id() {
    let mut profile = ConnectionProfile::new(
        "Test".to_string(),
        "http://localhost:8000".to_string(),
        "has space".to_string(),
        false,
    );
    assert!(profile_store::validate_profile(&mut profile).is_err());
}

#[test]
fn validate_rejects_zero_timeout() {
    let mut profile = ConnectionProfile::new(
        "Test".to_string(),
        "http://localhost:8000".to_string(),
        "default".to_string(),
        false,
    );
    profile.timeout_secs = 0;
    assert!(profile_store::validate_profile(&mut profile).is_err());
}

#[test]
fn corrupt_json_returns_error() {
    let dir = temp_dir();
    let path = dir.join("profiles.json");
    std::fs::write(&path, "not valid json {{{").unwrap();

    let result = profile_store::load_profiles(&path);
    assert!(result.is_err());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn validate_accepts_valid_profile() {
    let mut profile = ConnectionProfile::new(
        "Production".to_string(),
        "https://api.honcho.dev".to_string(),
        "my_workspace-01".to_string(),
        true,
    );
    assert!(profile_store::validate_profile(&mut profile).is_ok());
}

#[test]
fn validate_workspace_id_boundary_512() {
    // Exactly 512 chars → ok
    let id_512: String = "a".repeat(512);
    let mut profile = ConnectionProfile::new(
        "Test".to_string(),
        "http://localhost:8000".to_string(),
        id_512,
        false,
    );
    assert!(profile_store::validate_profile(&mut profile).is_ok());

    // 513 chars → err
    let id_513: String = "a".repeat(513);
    profile.workspace_id = id_513;
    assert!(profile_store::validate_profile(&mut profile).is_err());
}

#[test]
fn validate_trims_fields_and_accepts() {
    let mut profile = ConnectionProfile::new(
        "  Test Profile  ".to_string(),
        "  http://localhost:8000  ".to_string(),
        "  my-workspace  ".to_string(),
        false,
    );
    assert!(validate_profile(&mut profile).is_ok());
    assert_eq!(profile.name, "Test Profile");
    assert_eq!(profile.base_url, "http://localhost:8000");
    assert_eq!(profile.workspace_id, "my-workspace");
}
