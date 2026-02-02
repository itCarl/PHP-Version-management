use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Create a directory junction (works without admin privileges)
pub fn create_junction(target: &Path, link: &Path) -> Result<()> {
    // Remove existing junction/directory if present
    if link.exists() || is_junction(link) {
        remove_junction(link)?;
    }

    // Use cmd /c mklink /J for junction creation
    // Junctions don't require admin privileges unlike symlinks
    let output = Command::new("cmd")
        .args([
            "/c",
            "mklink",
            "/J",
            &link.to_string_lossy(),
            &target.to_string_lossy(),
        ])
        .output()
        .context("Failed to execute mklink command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create junction: {}", stderr);
    }

    Ok(())
}

/// Remove a junction point
pub fn remove_junction(link: &Path) -> Result<()> {
    if !link.exists() && !is_junction(link) {
        return Ok(());
    }

    // For junctions, we use rmdir (not recursive delete)
    if is_junction(link) {
        let output = Command::new("cmd")
            .args(["/c", "rmdir", &link.to_string_lossy()])
            .output()
            .context("Failed to execute rmdir command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to remove junction: {}", stderr);
        }
    } else if link.is_dir() {
        fs::remove_dir_all(link).context("Failed to remove directory")?;
    }

    Ok(())
}

/// Check if a path is a junction point
pub fn is_junction(path: &Path) -> bool {
    if !path.exists() {
        // Check using dir command for junctions that "exist" but show as not existing
        let output = Command::new("cmd")
            .args(["/c", "dir", "/AL", path.parent().unwrap_or(path).to_string_lossy().as_ref()])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            return stdout.contains(&format!("<JUNCTION>")) && stdout.contains(name.as_ref());
        }
        return false;
    }

    // Use fsutil to check for reparse point (junction)
    let output = Command::new("fsutil")
        .args(["reparsepoint", "query", &path.to_string_lossy()])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

/// Get the target of a junction point
pub fn get_junction_target(link: &Path) -> Result<Option<String>> {
    if !is_junction(link) {
        return Ok(None);
    }

    // Use dir command to get junction target
    let output = Command::new("cmd")
        .args(["/c", "dir", "/AL", link.parent().unwrap_or(link).to_string_lossy().as_ref()])
        .output()
        .context("Failed to query junction")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let name = link.file_name().unwrap_or_default().to_string_lossy();

    // Parse output to find the target path
    for line in stdout.lines() {
        if line.contains("<JUNCTION>") && line.contains(name.as_ref()) {
            // Format: ... <JUNCTION> name [target]
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    return Ok(Some(line[start + 1..end].to_string()));
                }
            }
        }
    }

    Ok(None)
}
