use thiserror::Error;

#[derive(Error, Debug)]
pub enum PvmError {
    #[error("PHP version '{0}' is not installed. Run 'pvm install {0}' first.")]
    VersionNotInstalled(String),

    #[error("PHP version '{0}' is already installed.")]
    VersionAlreadyInstalled(String),

    #[error("Invalid PHP version format: '{0}'. Expected format: 8.3 or 8.3.15")]
    InvalidVersionFormat(String),

    #[error("PHP version '{0}' not found on windows.php.net")]
    VersionNotAvailable(String),

    #[error("Failed to create symlink/junction: {0}. Try running as Administrator or enable Developer Mode.")]
    SymlinkFailed(String),

    #[error("XAMPP not found at '{0}'. Set path with 'pvm config xampp_path <path>'")]
    XamppNotFound(String),

    #[error("XAMPP PHP is not linked. Run 'pvm xampp link' first.")]
    XamppNotLinked,

    #[error("No PHP version is currently active. Run 'pvm use <version>' first.")]
    NoActiveVersion,

    #[error("Failed to modify PATH: {0}")]
    PathModificationFailed(String),

    #[error("Failed to download PHP: {0}")]
    DownloadFailed(String),

    #[error("Failed to extract archive: {0}")]
    ExtractionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
