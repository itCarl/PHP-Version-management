use crate::cli::output;
use crate::config::Config;
use crate::system::junction::{create_junction, get_junction_target, is_junction, remove_junction};
use anyhow::{Context, Result};
use std::fs;

pub struct XamppStatus {
    pub xampp_found: bool,
    pub xampp_path: String,
    pub is_linked: bool,
    pub linked_version: Option<String>,
    pub original_backed_up: bool,
}

/// Get XAMPP PHP linkage status
pub fn get_xampp_status(config: &Config) -> Result<XamppStatus> {
    let xampp_path = config
        .paths
        .xampp_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "C:\\xampp".to_string());

    let xampp_dir = std::path::Path::new(&xampp_path);
    let xampp_php = xampp_dir.join("php");
    let backup_path = xampp_dir.join("php.original");

    let xampp_found = xampp_dir.exists();
    let original_backed_up = backup_path.exists();

    let (is_linked, linked_version) = if is_junction(&xampp_php) {
        let target = get_junction_target(&xampp_php)?;
        let version = target.as_ref().and_then(|t| {
            std::path::Path::new(t)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
        });
        (true, version)
    } else {
        (false, None)
    };

    Ok(XamppStatus {
        xampp_found,
        xampp_path,
        is_linked,
        linked_version,
        original_backed_up,
    })
}

/// Link current PHP version to XAMPP
pub fn link_xampp(config: &mut Config) -> Result<()> {
    let xampp_path = config
        .paths
        .xampp_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("XAMPP path not configured. Set it in the config file."))?
        .clone();

    // Check XAMPP exists
    if !xampp_path.exists() {
        anyhow::bail!(
            "XAMPP not found at '{}'. Please verify the path or update your config.",
            xampp_path.display()
        );
    }

    // Check current version is set
    let current_version = config
        .settings
        .current_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No PHP version selected. Run 'pvm use <version>' first."))?
        .clone();

    let target_path = config.version_path(&current_version);
    if !target_path.exists() {
        anyhow::bail!(
            "PHP version '{}' not found at expected path.",
            current_version
        );
    }

    let xampp_php = xampp_path.join("php");
    let backup_path = xampp_path.join("php.original");

    // Step 1: Backup original PHP if not already backed up
    if !backup_path.exists() {
        if xampp_php.exists() && !is_junction(&xampp_php) {
            output::step(1, 4, "Backing up original XAMPP PHP...");
            fs::rename(&xampp_php, &backup_path).context("Failed to backup original PHP")?;
        } else if !xampp_php.exists() {
            anyhow::bail!("XAMPP PHP directory not found at {}", xampp_php.display());
        } else {
            output::step(1, 4, "Original already backed up, removing existing link...");
            remove_junction(&xampp_php)?;
        }
    } else {
        output::step(1, 4, "Using existing backup...");
        // Remove existing junction if present
        if xampp_php.exists() || is_junction(&xampp_php) {
            remove_junction(&xampp_php)?;
        }
    }

    // Step 2: Create junction to current PHP version
    output::step(2, 4, &format!("Linking XAMPP to PHP {}...", current_version));
    create_junction(&target_path, &xampp_php).context("Failed to create junction to PHP version")?;

    // Step 3: Update config
    output::step(3, 4, "Updating configuration...");
    config.xampp.linked = true;
    config.xampp.original_backup = Some(backup_path);
    config.save()?;

    // Step 4: Done
    output::step(4, 4, "Complete!");

    Ok(())
}

/// Restore XAMPP's original PHP
pub fn unlink_xampp(config: &mut Config) -> Result<()> {
    let xampp_path = config
        .paths
        .xampp_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("XAMPP path not configured."))?
        .clone();

    let xampp_php = xampp_path.join("php");
    let backup_path = config
        .xampp
        .original_backup
        .as_ref()
        .cloned()
        .unwrap_or_else(|| xampp_path.join("php.original"));

    if !backup_path.exists() {
        anyhow::bail!(
            "Original PHP backup not found at '{}'. Cannot restore.",
            backup_path.display()
        );
    }

    // Step 1: Remove junction
    output::step(1, 3, "Removing junction...");
    if xampp_php.exists() || is_junction(&xampp_php) {
        remove_junction(&xampp_php)?;
    }

    // Step 2: Restore original
    output::step(2, 3, "Restoring original PHP...");
    fs::rename(&backup_path, &xampp_php).context("Failed to restore original PHP")?;

    // Step 3: Update config
    output::step(3, 3, "Updating configuration...");
    config.xampp.linked = false;
    config.xampp.original_backup = None;
    config.save()?;

    Ok(())
}
