#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
compile_error!("Taicho requires macOS, Windows, or Linux for secret storage");

const KEYRING_SERVICE: &str = "dev.taicho";

pub fn init_keyring() {
    #[cfg(target_os = "macos")]
    {
        let store = apple_native_keyring_store::keychain::Store::new()
            .expect("failed to init macOS Keychain store");
        keyring_core::set_default_store(store);
    }

    #[cfg(target_os = "windows")]
    {
        let store = windows_native_keyring_store::Store::new()
            .expect("failed to init Windows Credential store");
        keyring_core::set_default_store(store);
    }

    #[cfg(target_os = "linux")]
    {
        let store = dbus_secret_service_keyring_store::Store::new()
            .expect("failed to init D-Bus Secret Service store");
        keyring_core::set_default_store(store);
    }
}

pub fn get_api_key(profile_id: &str) -> crate::error::AppResult<Option<String>> {
    let entry = keyring_core::Entry::new(KEYRING_SERVICE, &api_key_user(profile_id))?;
    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(e) => Err(crate::error::AppError::from(e)),
    }
}

pub fn set_api_key(profile_id: &str, api_key: &str) -> crate::error::AppResult<()> {
    let entry = keyring_core::Entry::new(KEYRING_SERVICE, &api_key_user(profile_id))?;
    entry.set_password(api_key)?;
    Ok(())
}

pub fn delete_api_key(profile_id: &str) -> crate::error::AppResult<()> {
    let entry = keyring_core::Entry::new(KEYRING_SERVICE, &api_key_user(profile_id))?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
        Err(e) => Err(crate::error::AppError::from(e)),
    }
}

fn api_key_user(profile_id: &str) -> String {
    format!("{profile_id}/api_key")
}
