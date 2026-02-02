use anyhow::{Context, Result};
use winreg::enums::*;
use winreg::RegKey;

#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG};
#[cfg(windows)]
use windows::Win32::Foundation::{WPARAM, LPARAM};

const WM_SETTINGCHANGE: u32 = 0x001A;

/// Add a path to the user's PATH environment variable
pub fn add_to_path(new_entry: &str) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Failed to open Environment registry key")?;

    // Read current PATH
    let current_path: String = env.get_value("Path").unwrap_or_default();

    // Check if already present
    if is_path_entry_present(&current_path, new_entry) {
        return Ok(());
    }

    // Append new entry
    let new_path = if current_path.is_empty() {
        new_entry.to_string()
    } else {
        format!("{};{}", new_entry, current_path)
    };

    // Write back
    env.set_value("Path", &new_path)
        .context("Failed to update PATH in registry")?;

    Ok(())
}

/// Remove a path from the user's PATH environment variable
pub fn remove_from_path(entry: &str) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Failed to open Environment registry key")?;

    let current_path: String = env.get_value("Path").unwrap_or_default();

    // Filter out the entry
    let new_path: String = current_path
        .split(';')
        .filter(|p| !paths_equal(p, entry))
        .collect::<Vec<_>>()
        .join(";");

    env.set_value("Path", &new_path)
        .context("Failed to update PATH in registry")?;

    Ok(())
}

/// Check if a path entry is in the PATH
pub fn is_in_path(entry: &str) -> Result<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ)
        .context("Failed to open Environment registry key")?;

    let current_path: String = env.get_value("Path").unwrap_or_default();
    Ok(is_path_entry_present(&current_path, entry))
}

fn is_path_entry_present(path: &str, entry: &str) -> bool {
    path.split(';').any(|p| paths_equal(p, entry))
}

fn paths_equal(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Broadcast WM_SETTINGCHANGE to notify applications of environment change
#[cfg(windows)]
pub fn broadcast_environment_change() -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let env_wide: Vec<u16> = OsStr::new("Environment")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut _result: usize = 0;
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(env_wide.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            5000,
            Some(&mut _result),
        );
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn broadcast_environment_change() -> Result<()> {
    Ok(())
}
