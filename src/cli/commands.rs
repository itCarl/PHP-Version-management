use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pvm")]
#[command(author, version, about = "PHP Version Management - Fast PHP version switching for Windows")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize PVM (create directories and add to PATH)
    Init,

    /// List all installed PHP versions
    List,

    /// Show currently active PHP version
    Current,

    /// Switch to a specific PHP version
    #[command(name = "use")]
    Use {
        /// PHP version to switch to (e.g., 8.3, 8.3.15)
        version: String,
    },

    /// Download and install a PHP version
    Install {
        /// PHP version to install (e.g., 8.3, 8.3.15)
        version: String,

        /// Install non-thread-safe version (default is thread-safe for XAMPP)
        #[arg(long)]
        nts: bool,

        /// Install 32-bit version (default is 64-bit)
        #[arg(long)]
        x86: bool,
    },

    /// Remove an installed PHP version
    Uninstall {
        /// PHP version to remove
        version: String,
    },

    /// XAMPP integration commands
    Xampp {
        #[command(subcommand)]
        command: XamppCommands,
    },

    /// Diagnose installation and PATH issues
    Doctor,

    /// Show path to current php.exe
    Which,
}

#[derive(Subcommand)]
pub enum XamppCommands {
    /// Link current PHP version to XAMPP
    Link,

    /// Restore XAMPP's original PHP
    Unlink,

    /// Show XAMPP PHP linkage status
    Status,
}
