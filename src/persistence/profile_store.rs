use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub workspace_id: String,
    pub uses_api_key: bool,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_retries")]
    pub max_retries: u32,
}

fn default_timeout() -> u64 {
    120
}

fn default_retries() -> u32 {
    2
}

impl ConnectionProfile {
    pub fn new(name: String, base_url: String, workspace_id: String, uses_api_key: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            base_url,
            workspace_id,
            uses_api_key,
            timeout_secs: default_timeout(),
            max_retries: default_retries(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileStore {
    pub profiles: Vec<ConnectionProfile>,
    pub active_profile_id: Option<String>,
}

pub fn load_profiles(path: &Path) -> AppResult<ProfileStore> {
    match std::fs::read_to_string(path) {
        Ok(data) => Ok(serde_json::from_str(&data)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ProfileStore::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save_profiles(path: &Path, store: &ProfileStore) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(store)?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &json)?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.into());
    }
    Ok(())
}

pub fn validate_profile(profile: &mut ConnectionProfile) -> AppResult<()> {
    // Normalize: trim whitespace in place
    profile.name = profile.name.trim().to_string();
    profile.base_url = profile.base_url.trim().to_string();
    profile.workspace_id = profile.workspace_id.trim().to_string();

    if profile.name.is_empty() {
        return Err(AppError::Validation(
            "profile name must not be empty".to_string(),
        ));
    }

    if !(profile.base_url.starts_with("http://") || profile.base_url.starts_with("https://")) {
        return Err(AppError::Validation(
            "base URL must start with http:// or https://".to_string(),
        ));
    }

    let upper = profile.base_url.to_uppercase();
    if upper.contains("HONCHO_API_KEY") || upper.contains("API_KEY=") {
        return Err(AppError::Validation(
            "base URL must not contain an API key".to_string(),
        ));
    }

    validate_workspace_id(&profile.workspace_id)?;

    if profile.timeout_secs == 0 {
        return Err(AppError::Validation("timeout must be > 0".to_string()));
    }

    Ok(())
}

fn validate_workspace_id(id: &str) -> AppResult<()> {
    if id.is_empty() || id.len() > 512 {
        return Err(AppError::Validation(
            "workspace_id must be 1\u{2013}512 characters".to_string(),
        ));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::Validation(
            "workspace_id must contain only ASCII alphanumeric, underscore, or hyphen".to_string(),
        ));
    }
    Ok(())
}
