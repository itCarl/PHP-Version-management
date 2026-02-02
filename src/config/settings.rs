use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub paths: PathsConfig,
    pub settings: SettingsConfig,
    pub xampp: XamppConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathsConfig {
    pub versions_dir: PathBuf,
    pub xampp_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub current_version: Option<String>,
    pub default_variant: PhpVariant,
    pub default_arch: Architecture,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XamppConfig {
    pub linked: bool,
    pub original_backup: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PhpVariant {
    Ts,
    Nts,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    X64,
    X86,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            paths: PathsConfig {
                versions_dir: PathBuf::from("C:\\php-versions"),
                xampp_path: Some(PathBuf::from("C:\\xampp")),
            },
            settings: SettingsConfig {
                current_version: None,
                default_variant: PhpVariant::Ts,
                default_arch: Architecture::X64,
            },
            xampp: XamppConfig {
                linked: false,
                original_backup: None,
            },
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let base = directories::BaseDirs::new()
            .context("Failed to get base directories")?;
        Ok(base.config_dir().join("pvm"))
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .context("Failed to read config file")?;
            let config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&path, content)
            .context("Failed to write config file")?;
        Ok(())
    }

    pub fn current_link_path(&self) -> PathBuf {
        self.paths.versions_dir.join("current")
    }

    pub fn version_path(&self, version: &str) -> PathBuf {
        self.paths.versions_dir.join(version)
    }
}
