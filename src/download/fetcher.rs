use crate::config::{Architecture, PhpVariant};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const RELEASES_URL: &str = "https://windows.php.net/downloads/releases/";
const ARCHIVES_URL: &str = "https://windows.php.net/downloads/releases/archives/";

pub struct PhpRelease {
    pub version: String,
    pub variant: PhpVariant,
    pub arch: Architecture,
    pub url: String,
    pub filename: String,
}

impl PhpRelease {
    pub fn new(version: &str, variant: PhpVariant, arch: Architecture) -> Self {
        let vc = determine_vc_version(version);
        let arch_str = match arch {
            Architecture::X64 => "x64",
            Architecture::X86 => "x86",
        };
        let variant_suffix = match variant {
            PhpVariant::Ts => "",
            PhpVariant::Nts => "-nts",
        };

        let filename = format!(
            "php-{}{}-Win32-{}-{}.zip",
            version, variant_suffix, vc, arch_str
        );

        let url = format!("{}{}", RELEASES_URL, filename);

        Self {
            version: version.to_string(),
            variant,
            arch,
            url,
            filename,
        }
    }

    pub fn archive_url(&self) -> String {
        format!("{}{}", ARCHIVES_URL, self.filename)
    }
}

fn determine_vc_version(version: &str) -> &'static str {
    let major: u32 = version
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);

    let minor: u32 = version
        .split('.')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    match (major, minor) {
        (8, 4..) => "vs17",  // PHP 8.4+ uses VS17
        (8, _) => "vs16",    // PHP 8.0-8.3 uses VS16
        (7, 4) => "vc15",    // PHP 7.4 uses VC15
        (7, _) => "vc15",    // PHP 7.x uses VC15
        _ => "vs16",
    }
}

/// Get the download URL for a PHP version
pub fn get_download_url(version: &str, variant: PhpVariant, arch: Architecture) -> String {
    PhpRelease::new(version, variant, arch).url
}

/// Download PHP to the specified directory
pub fn download_php(
    version: &str,
    variant: PhpVariant,
    arch: Architecture,
    dest_dir: &Path,
) -> Result<std::path::PathBuf> {
    let release = PhpRelease::new(version, variant, arch);
    let dest_file = dest_dir.join(&release.filename);

    // Try main releases URL first, then archives
    let urls = [release.url.clone(), release.archive_url()];
    let mut last_error = None;

    for url in &urls {
        match download_file(url, &dest_file) {
            Ok(()) => return Ok(dest_file),
            Err(e) => {
                last_error = Some(e);
                continue;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Failed to download PHP")))
}

fn download_file(url: &str, dest: &Path) -> Result<()> {
    println!("Downloading from: {}", url);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(url)
        .send()
        .context("Failed to connect to download server")?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed with status: {}", response.status());
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    let mut file = File::create(dest).context("Failed to create destination file")?;

    let mut downloaded: u64 = 0;
    let mut response = response;
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = response.read(&mut buffer).context("Failed to read from response")?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])
            .context("Failed to write to file")?;
        downloaded += bytes_read as u64;

        if let Some(ref pb) = pb {
            pb.set_position(downloaded);
        }
    }

    if let Some(pb) = pb {
        pb.finish_with_message("Downloaded");
    }

    Ok(())
}
