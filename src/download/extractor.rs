use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipArchive;

/// Extract a ZIP file to the specified directory
pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = File::open(zip_path).context("Failed to open ZIP file")?;
    let mut archive = ZipArchive::new(file).context("Failed to read ZIP archive")?;

    // Create destination directory
    fs::create_dir_all(dest_dir).context("Failed to create destination directory")?;

    let total_files = archive.len();
    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files")
            .unwrap()
            .progress_chars("#>-"),
    );

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("Failed to read file from archive")?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            // Directory
            fs::create_dir_all(&outpath).context("Failed to create directory")?;
        } else {
            // File
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).context("Failed to create parent directory")?;
                }
            }

            let mut outfile = File::create(&outpath).context("Failed to create output file")?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).context("Failed to read from archive")?;
            outfile.write_all(&buffer).context("Failed to write to file")?;
        }

        // Set permissions (not critical on Windows, but good practice)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).ok();
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Extracted");
    Ok(())
}
