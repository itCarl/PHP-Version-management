# PVM - PHP Version Management

A fast, lightweight CLI tool for switching between PHP versions on Windows. Inspired by [nvm](https://github.com/nvm-sh/nvm) (Node Version Manager).

## Features

- **Instant Switching** - Switch PHP versions in milliseconds using Windows junction points
- **XAMPP Integration** - Seamlessly link your PHP version to XAMPP
- **Auto-Download** - Download PHP versions directly from windows.php.net
- **PATH Management** - Automatically configures your system PATH
- **No Admin Required** - Works without administrator privileges (uses junctions, not symlinks)

## Installation

### Prerequisites

- Windows 10/11
- [Rust](https://rustup.rs/) (for building from source)

### Build from Source

```powershell
git clone https://github.com/itCarl/PHP-Version-management.git
cd PHP-Version-management
cargo install --path .
```

This installs `pvm.exe` to `%USERPROFILE%\.cargo\bin\` which is automatically in your PATH.

### Manual Installation

```powershell
cargo build --release
copy target\release\pvm.exe C:\Windows\
```

## Quick Start

```powershell
# 1. Initialize PVM (one-time setup)
pvm init

# 2. Install PHP versions
pvm install 8.3
pvm install 8.2
pvm install 8.1

# 3. Switch between versions
pvm use 8.3

# 4. Verify (open a new terminal)
php -v
```

## Commands

| Command | Description |
|---------|-------------|
| `pvm init` | Initialize PVM (create directories, configure PATH) |
| `pvm list` | List all installed PHP versions |
| `pvm current` | Show the currently active PHP version |
| `pvm use <version>` | Switch to a specific PHP version |
| `pvm install <version>` | Download and install a PHP version |
| `pvm uninstall <version>` | Remove an installed PHP version |
| `pvm which` | Show the path to the current php.exe |
| `pvm doctor` | Diagnose installation and PATH issues |

### XAMPP Integration

| Command | Description |
|---------|-------------|
| `pvm xampp link` | Link current PHP version to XAMPP |
| `pvm xampp unlink` | Restore XAMPP's original PHP |
| `pvm xampp status` | Show XAMPP linkage status |

## Usage Examples

### Install a Specific PHP Version

```powershell
# Install latest PHP 8.3.x (thread-safe, x64)
pvm install 8.3

# Install specific version
pvm install 8.3.15

# Install non-thread-safe version (for CLI/FastCGI)
pvm install 8.3 --nts

# Install 32-bit version
pvm install 8.3 --x86
```

### Switch PHP Versions

```powershell
# Switch to PHP 8.3
pvm use 8.3

# Switch to specific version
pvm use 8.2.20

# Check current version
pvm current
# Output: 8.3.15 (C:\php-versions\8.3.15)
```

### List Installed Versions

```powershell
pvm list
# Output:
#     8.1.27 [C:\php-versions\8.1.27]
#     8.2.20 [C:\php-versions\8.2.20]
#   * 8.3.15 (current) [C:\php-versions\8.3.15]
```

### XAMPP Integration

```powershell
# Link PVM's current PHP to XAMPP
pvm xampp link

# Check status
pvm xampp status

# Restore original XAMPP PHP
pvm xampp unlink
```

> **Note:** Restart Apache after linking/unlinking for changes to take effect.

## How It Works

### Directory Structure

```
C:\php-versions\           # PHP versions storage
├── 8.1.27\                # Installed PHP 8.1
├── 8.2.20\                # Installed PHP 8.2
├── 8.3.15\                # Installed PHP 8.3
└── current\               # Junction → active version (e.g., 8.3.15)
```

### Version Switching

PVM uses Windows **junction points** (similar to symlinks) for instant switching:

1. `C:\php-versions\current` is added to your PATH (one-time)
2. When you run `pvm use 8.3`, the `current` junction is updated to point to `8.3.15`
3. No PATH modification needed - switching is instant

### XAMPP Integration

When you run `pvm xampp link`:

1. Original XAMPP PHP is backed up to `C:\xampp\php.original`
2. A junction is created: `C:\xampp\php` → `C:\php-versions\8.3.15`
3. XAMPP now uses the same PHP version as PVM

## Configuration

Configuration is stored at `%APPDATA%\pvm\config.toml`:

```toml
[paths]
versions_dir = "C:\\php-versions"
xampp_path = "C:\\xampp"

[settings]
current_version = "8.3.15"
default_variant = "ts"      # "ts" (thread-safe) or "nts"
default_arch = "x64"        # "x64" or "x86"

[xampp]
linked = true
original_backup = "C:\\xampp\\php.original"
```

## Thread-Safe vs Non-Thread-Safe

| Variant | Use Case |
|---------|----------|
| **Thread-Safe (TS)** | Apache with mod_php, XAMPP (default) |
| **Non-Thread-Safe (NTS)** | CLI, IIS with FastCGI, nginx with PHP-FPM |

PVM defaults to **thread-safe** for XAMPP compatibility. Use `--nts` flag for non-thread-safe:

```powershell
pvm install 8.3 --nts
```

## Troubleshooting

### "php" is not recognized

1. Run `pvm init` to configure PATH
2. Open a **new terminal** (PATH changes require a new session)
3. Run `pvm doctor` to diagnose issues

### XAMPP still shows old PHP version

1. Ensure you ran `pvm xampp link`
2. **Restart Apache** from XAMPP Control Panel
3. Check `phpinfo()` in your browser

### Permission errors

PVM uses junction points which don't require admin privileges. If you still get errors:

1. Run `pvm doctor` to check for issues
2. Ensure antivirus isn't blocking junction creation
3. Try running as Administrator (shouldn't be needed)

### Download fails

1. Check your internet connection
2. PHP version might not exist - check [windows.php.net](https://windows.php.net/downloads/releases/)
3. Try a different version: `pvm install 8.3.14`

## Comparison with Other Tools

| Feature | PVM | Manual Switching | XAMPP Built-in |
|---------|-----|------------------|----------------|
| Instant switching | ✅ | ❌ | ❌ |
| Multiple versions | ✅ | ✅ | ❌ |
| Auto-download | ✅ | ❌ | ❌ |
| XAMPP integration | ✅ | ❌ | ✅ |
| No admin required | ✅ | ❌ | ✅ |
| PATH auto-config | ✅ | ❌ | ❌ |

## Requirements

- Windows 10 or later
- NTFS file system (for junction points)
- ~50-100 MB per PHP version

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request
