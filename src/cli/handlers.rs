use crate::cli::commands::XamppCommands;
use crate::cli::output;
use crate::config::{Architecture, Config, PhpVariant};
use crate::core::manager;
use crate::download::{download_php, extractor};
use crate::system::{add_to_path, broadcast_environment_change, is_in_path};
use crate::xampp;
use anyhow::Result;
use std::fs;

/// Handle 'pvm init' command
pub fn handle_init() -> Result<()> {
    output::header("Initializing PVM");

    let config = Config::load()?;
    let versions_dir = &config.paths.versions_dir;
    let current_link = config.current_link_path();

    // Step 1: Create versions directory
    output::step(1, 3, "Creating PHP versions directory...");
    if !versions_dir.exists() {
        fs::create_dir_all(versions_dir)?;
        output::info(&format!("Created: {}", versions_dir.display()));
    } else {
        output::info(&format!("Already exists: {}", versions_dir.display()));
    }

    // Step 2: Add to PATH
    output::step(2, 3, "Configuring PATH...");
    let path_entry = current_link.to_string_lossy().to_string();
    if is_in_path(&path_entry)? {
        output::info("PATH already configured");
    } else {
        add_to_path(&path_entry)?;
        broadcast_environment_change()?;
        output::info(&format!("Added to PATH: {}", path_entry));
    }

    // Step 3: Save config
    output::step(3, 3, "Saving configuration...");
    config.save()?;

    output::success("PVM initialized successfully!");
    println!();
    output::info("Next steps:");
    println!("  1. Install a PHP version:  pvm install 8.3");
    println!("  2. Switch to that version: pvm use 8.3");
    println!("  3. Open a new terminal and run: php -v");

    Ok(())
}

/// Handle 'pvm list' command
pub fn handle_list() -> Result<()> {
    let config = Config::load()?;
    let versions = manager::list_versions(&config)?;
    let current = manager::get_current_version(&config)?;

    if versions.is_empty() {
        output::info("No PHP versions installed.");
        println!();
        println!("Install a version with: pvm install <version>");
        println!("Example: pvm install 8.3");
        return Ok(());
    }

    output::header("Installed PHP versions");
    for version in &versions {
        let is_current = current.as_ref().map(|c| c == version).unwrap_or(false);
        let path = config.version_path(version);
        output::version_line(version, is_current, &path.to_string_lossy());
    }

    Ok(())
}

/// Handle 'pvm current' command
pub fn handle_current() -> Result<()> {
    let config = Config::load()?;

    match manager::get_current_version(&config)? {
        Some(version) => {
            let path = config.version_path(&version);
            println!("{} ({})", version, path.display());
        }
        None => {
            output::info("No PHP version is currently active.");
            println!("Run 'pvm use <version>' to activate a version.");
        }
    }

    Ok(())
}

/// Handle 'pvm use <version>' command
pub fn handle_use(version: &str) -> Result<()> {
    let mut config = Config::load()?;

    output::info(&format!("Switching to PHP {}...", version));

    manager::switch_version(&mut config, version)?;
    broadcast_environment_change()?;

    let resolved = manager::get_current_version(&config)?.unwrap_or_else(|| version.to_string());
    output::success(&format!("Now using PHP {}", resolved));

    // Check if XAMPP is linked
    if config.xampp.linked {
        output::warning("XAMPP is linked. Restart Apache for changes to take effect.");
    }

    println!();
    output::info("Run 'php -v' in a new terminal to verify.");

    Ok(())
}

/// Handle 'pvm install <version>' command
pub fn handle_install(version: &str, nts: bool, x86: bool) -> Result<()> {
    let mut config = Config::load()?;

    let variant = if nts {
        PhpVariant::Nts
    } else {
        config.settings.default_variant
    };

    let arch = if x86 {
        Architecture::X86
    } else {
        config.settings.default_arch
    };

    // Ensure versions directory exists
    if !config.paths.versions_dir.exists() {
        fs::create_dir_all(&config.paths.versions_dir)?;
    }

    let dest_dir = config.version_path(version);

    // Check if already installed
    if dest_dir.exists() && dest_dir.join("php.exe").exists() {
        output::warning(&format!("PHP {} is already installed.", version));
        println!("Use 'pvm uninstall {}' to remove it first.", version);
        return Ok(());
    }

    output::header(&format!("Installing PHP {}", version));

    let variant_str = match variant {
        PhpVariant::Ts => "thread-safe",
        PhpVariant::Nts => "non-thread-safe",
    };
    let arch_str = match arch {
        Architecture::X64 => "x64",
        Architecture::X86 => "x86",
    };
    output::info(&format!("Variant: {} ({})", variant_str, arch_str));

    // Download
    output::step(1, 3, "Downloading PHP...");
    let temp_dir = std::env::temp_dir().join("pvm-downloads");
    fs::create_dir_all(&temp_dir)?;

    let zip_path = download_php(version, variant, arch, &temp_dir)?;

    // Extract
    output::step(2, 3, "Extracting...");
    fs::create_dir_all(&dest_dir)?;
    extractor::extract_zip(&zip_path, &dest_dir)?;

    // Cleanup
    output::step(3, 3, "Cleaning up...");
    fs::remove_file(&zip_path).ok();

    // Copy php.ini-development to php.ini if no php.ini exists
    let ini_path = dest_dir.join("php.ini");
    let ini_dev_path = dest_dir.join("php.ini-development");
    if !ini_path.exists() && ini_dev_path.exists() {
        fs::copy(&ini_dev_path, &ini_path)?;
        output::info("Created php.ini from php.ini-development");
    }

    output::success(&format!("PHP {} installed successfully!", version));
    println!();
    println!("To use this version: pvm use {}", version);

    // Auto-switch if no version is currently active
    if config.settings.current_version.is_none() {
        output::info("Auto-switching to installed version...");
        manager::switch_version(&mut config, version)?;
        broadcast_environment_change()?;
        output::success(&format!("Now using PHP {}", version));
    }

    Ok(())
}

/// Handle 'pvm uninstall <version>' command
pub fn handle_uninstall(version: &str) -> Result<()> {
    let mut config = Config::load()?;

    let version = manager::resolve_version(&config, version)?;
    let version_path = config.version_path(&version);

    if !version_path.exists() {
        anyhow::bail!("PHP version '{}' is not installed.", version);
    }

    // Check if this is the current version
    let current = manager::get_current_version(&config)?;
    if current.as_ref().map(|c| c == &version).unwrap_or(false) {
        output::warning(&format!(
            "PHP {} is currently active. Switch to another version first.",
            version
        ));
        return Ok(());
    }

    output::info(&format!("Removing PHP {}...", version));

    fs::remove_dir_all(&version_path)?;

    output::success(&format!("PHP {} has been uninstalled.", version));

    Ok(())
}

/// Handle 'pvm xampp <command>' command
pub fn handle_xampp(command: XamppCommands) -> Result<()> {
    match command {
        XamppCommands::Link => {
            let mut config = Config::load()?;
            output::header("Linking XAMPP to current PHP version");
            xampp::link_xampp(&mut config)?;
            output::success("XAMPP is now linked to PVM!");
            output::warning("Restart Apache for changes to take effect.");
        }
        XamppCommands::Unlink => {
            let mut config = Config::load()?;
            output::header("Restoring XAMPP original PHP");
            xampp::unlink_xampp(&mut config)?;
            output::success("XAMPP PHP restored to original.");
            output::warning("Restart Apache for changes to take effect.");
        }
        XamppCommands::Status => {
            let config = Config::load()?;
            let status = xampp::get_xampp_status(&config)?;

            output::header("XAMPP Status");

            println!("  XAMPP Path:     {}", status.xampp_path);
            println!(
                "  XAMPP Found:    {}",
                if status.xampp_found { "Yes" } else { "No" }
            );
            println!(
                "  PHP Linked:     {}",
                if status.is_linked { "Yes" } else { "No" }
            );

            if let Some(version) = &status.linked_version {
                println!("  Linked Version: {}", version);
            }

            println!(
                "  Original Backup: {}",
                if status.original_backed_up {
                    "Yes"
                } else {
                    "No"
                }
            );
        }
    }

    Ok(())
}

/// Handle 'pvm doctor' command
pub fn handle_doctor() -> Result<()> {
    output::header("PVM Doctor");

    let config = Config::load()?;
    let mut issues = 0;

    // Check versions directory
    println!("\nVersions Directory:");
    if config.paths.versions_dir.exists() {
        println!("  {} {}", "OK".green(), config.paths.versions_dir.display());
    } else {
        println!(
            "  {} {} (not found)",
            "!!".red(),
            config.paths.versions_dir.display()
        );
        println!("     Run 'pvm init' to create it.");
        issues += 1;
    }

    // Check PATH
    println!("\nPATH Configuration:");
    let current_link = config.current_link_path();
    let path_entry = current_link.to_string_lossy().to_string();
    match is_in_path(&path_entry) {
        Ok(true) => {
            println!("  {} {} is in PATH", "OK".green(), path_entry);
        }
        Ok(false) => {
            println!("  {} {} is NOT in PATH", "!!".red(), path_entry);
            println!("     Run 'pvm init' to add it.");
            issues += 1;
        }
        Err(e) => {
            println!("  {} Failed to check PATH: {}", "!!".red(), e);
            issues += 1;
        }
    }

    // Check current version
    println!("\nCurrent Version:");
    match manager::get_current_version(&config)? {
        Some(version) => {
            let path = config.version_path(&version);
            if path.join("php.exe").exists() {
                println!("  {} PHP {} at {}", "OK".green(), version, path.display());
            } else {
                println!(
                    "  {} PHP {} linked but php.exe not found",
                    "!!".red(),
                    version
                );
                issues += 1;
            }
        }
        None => {
            println!("  {} No PHP version active", "..".yellow());
            println!("     Run 'pvm use <version>' to activate one.");
        }
    }

    // Check XAMPP
    println!("\nXAMPP Integration:");
    let xampp_status = xampp::get_xampp_status(&config)?;
    if xampp_status.xampp_found {
        println!("  {} XAMPP found at {}", "OK".green(), xampp_status.xampp_path);
        if xampp_status.is_linked {
            if let Some(version) = &xampp_status.linked_version {
                println!("  {} Linked to PHP {}", "OK".green(), version);
            }
        } else {
            println!("  {} Not linked to PVM", "..".yellow());
        }
    } else {
        println!(
            "  {} XAMPP not found at {}",
            "..".yellow(),
            xampp_status.xampp_path
        );
    }

    // Summary
    println!();
    if issues == 0 {
        output::success("No issues found!");
    } else {
        output::warning(&format!("{} issue(s) found. See above for details.", issues));
    }

    Ok(())
}

/// Handle 'pvm which' command
pub fn handle_which() -> Result<()> {
    let config = Config::load()?;

    match manager::get_current_version(&config)? {
        Some(version) => {
            let php_exe = config.version_path(&version).join("php.exe");
            println!("{}", php_exe.display());
        }
        None => {
            output::info("No PHP version is currently active.");
        }
    }

    Ok(())
}

// Helper trait for colored output in doctor command
trait ColoredStr {
    fn green(&self) -> colored::ColoredString;
    fn red(&self) -> colored::ColoredString;
    fn yellow(&self) -> colored::ColoredString;
}

impl ColoredStr for &str {
    fn green(&self) -> colored::ColoredString {
        use colored::Colorize;
        (*self).green()
    }

    fn red(&self) -> colored::ColoredString {
        use colored::Colorize;
        (*self).red()
    }

    fn yellow(&self) -> colored::ColoredString {
        use colored::Colorize;
        (*self).yellow()
    }
}
