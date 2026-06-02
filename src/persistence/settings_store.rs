use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::persistence::paths::AppPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_theme() -> String {
    "system".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
}

impl WindowState {
    /// Returns `true` if dimensions are positive, guarding against a 0×0 window
    /// that can occur when deserializing an empty or partially-written JSON file.
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

pub fn load_settings(paths: &AppPaths) -> AppResult<Settings> {
    match std::fs::read_to_string(&paths.settings_file) {
        Ok(data) => Ok(serde_json::from_str(&data)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save_settings(paths: &AppPaths, settings: &Settings) -> AppResult<()> {
    atomic_write(&paths.settings_file, settings)
}

pub fn load_window_state(paths: &AppPaths) -> AppResult<Option<WindowState>> {
    match std::fs::read_to_string(&paths.window_state_file) {
        Ok(data) => Ok(Some(serde_json::from_str(&data)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn save_window_state(paths: &AppPaths, state: &WindowState) -> AppResult<()> {
    atomic_write(&paths.window_state_file, state)
}

fn atomic_write(path: &std::path::Path, value: &impl Serialize) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)?;
    let tmp = path.with_extension("tmp");
    let write_result = std::fs::write(&tmp, &json).and_then(|_| std::fs::rename(&tmp, path));
    if write_result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    Ok(write_result?)
}
