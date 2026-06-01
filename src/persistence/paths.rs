use std::path::PathBuf;

use crate::error::{AppError, AppResult};

pub struct AppPaths {
    pub config_dir: PathBuf,
    pub profiles_file: PathBuf,
}

pub fn app_paths() -> AppResult<AppPaths> {
    let dirs = directories::ProjectDirs::from("dev", "Taicho", "Taicho").ok_or_else(|| {
        AppError::Validation("application data directory unavailable".to_string())
    })?;

    let config_dir = dirs.config_dir().to_path_buf();

    Ok(AppPaths {
        profiles_file: config_dir.join("profiles.json"),
        config_dir,
    })
}
