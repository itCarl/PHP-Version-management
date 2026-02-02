use crate::config::Config;
use crate::system::{create_junction, is_junction};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Get list of installed PHP versions
pub fn list_versions(config: &Config) -> Result<Vec<String>> {
    let versions_dir = &config.paths.versions_dir;

    if !versions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut versions = Vec::new();

    for entry in fs::read_dir(versions_dir).context("Failed to read versions directory")? {
        let entry = entry?;
        let path = entry.path();

        // Skip the 'current' symlink
        if path.file_name().map(|n| n == "current").unwrap_or(false) {
            continue;
        }

        // Only include directories that contain php.exe
        if path.is_dir() && path.join("php.exe").exists() {
            if let Some(name) = path.file_name() {
                versions.push(name.to_string_lossy().to_string());
            }
        }
    }

    // Sort versions
    versions.sort_by(|a, b| compare_versions(a, b));

    Ok(versions)
}

/// Get the currently active PHP version
pub fn get_current_version(config: &Config) -> Result<Option<String>> {
    // First check config
    if let Some(ref version) = config.settings.current_version {
        // Verify the junction still points to this version
        let current_link = config.current_link_path();
        if current_link.exists() || is_junction(&current_link) {
            return Ok(Some(version.clone()));
        }
    }

    // Try to detect from junction target
    let current_link = config.current_link_path();
    if let Ok(Some(target)) = crate::system::junction::get_junction_target(&current_link) {
        let target_path = PathBuf::from(&target);
        if let Some(name) = target_path.file_name() {
            return Ok(Some(name.to_string_lossy().to_string()));
        }
    }

    Ok(None)
}

/// Switch to a specific PHP version
pub fn switch_version(config: &mut Config, version: &str) -> Result<()> {
    let version = resolve_version(config, version)?;
    let version_path = config.version_path(&version);

    // Validate version exists
    if !version_path.exists() {
        anyhow::bail!("PHP version '{}' is not installed. Run 'pvm install {}' first.", version, version);
    }

    // Validate php.exe exists
    if !version_path.join("php.exe").exists() {
        anyhow::bail!("PHP version '{}' appears corrupted (php.exe not found).", version);
    }

    // Create/update the 'current' junction
    let current_link = config.current_link_path();
    create_junction(&version_path, &current_link)
        .context("Failed to create junction to PHP version")?;

    // Update config
    config.settings.current_version = Some(version.clone());
    config.save()?;

    Ok(())
}

/// Resolve a partial version (e.g., "8.3") to a full version (e.g., "8.3.15")
pub fn resolve_version(config: &Config, version: &str) -> Result<String> {
    let versions = list_versions(config)?;

    // Exact match
    if versions.contains(&version.to_string()) {
        return Ok(version.to_string());
    }

    // Partial match (e.g., "8.3" matches "8.3.15")
    let matching: Vec<_> = versions
        .iter()
        .filter(|v| v.starts_with(&format!("{}.", version)) || v.starts_with(&format!("{}-", version)))
        .collect();

    match matching.len() {
        0 => {
            // Check if the exact version directory exists (might be newly installed)
            let version_path = config.version_path(version);
            if version_path.exists() && version_path.join("php.exe").exists() {
                return Ok(version.to_string());
            }
            anyhow::bail!("No installed version matches '{}'. Run 'pvm list' to see available versions.", version)
        }
        1 => Ok(matching[0].clone()),
        _ => {
            // Return the latest matching version
            Ok(matching.last().unwrap().to_string())
        }
    }
}

/// Check if a version is installed
pub fn is_version_installed(config: &Config, version: &str) -> bool {
    let version_path = config.version_path(version);
    version_path.exists() && version_path.join("php.exe").exists()
}

/// Compare two version strings for sorting
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_version = |s: &str| -> Vec<u32> {
        s.split(|c: char| c == '.' || c == '-')
            .filter_map(|p| p.parse().ok())
            .collect()
    };

    let va = parse_version(a);
    let vb = parse_version(b);

    va.cmp(&vb)
}
