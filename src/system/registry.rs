use anyhow::{Context, Result};
use winreg::enums::*;
use winreg::RegKey;

/// Get a value from the user environment registry
pub fn get_user_env(name: &str) -> Result<Option<String>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ)
        .context("Failed to open Environment registry key")?;

    match env.get_value::<String, _>(name) {
        Ok(value) => Ok(Some(value)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).context(format!("Failed to read {} from registry", name)),
    }
}

/// Set a value in the user environment registry
pub fn set_user_env(name: &str, value: &str) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Failed to open Environment registry key")?;

    env.set_value(name, &value)
        .context(format!("Failed to set {} in registry", name))?;

    Ok(())
}

/// Delete a value from the user environment registry
pub fn delete_user_env(name: &str) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Failed to open Environment registry key")?;

    match env.delete_value(name) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).context(format!("Failed to delete {} from registry", name)),
    }
}
