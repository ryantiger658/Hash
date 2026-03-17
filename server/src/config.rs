use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

/// Top-level configuration loaded from config.toml.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub vault: VaultConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VaultConfig {
    pub path: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub api_key: String,
}

impl Config {
    /// Load configuration from a TOML file, falling back to environment variables.
    ///
    /// Resolution order:
    /// 1. The file at `HASH_CONFIG` (or `config.toml` if unset), if it exists.
    /// 2. Environment variables (useful for Docker / container deployments):
    ///    - `HASH_HOST`           (default: 0.0.0.0)
    ///    - `HASH_PORT`           (default: 3535)
    ///    - `HASH_VAULT_PATH`     (required if no config file)
    ///    - `HASH_API_KEY`        (required if no config file)
    ///    - `HASH_SECONDARY_COLOR`
    ///    - `HASH_DEFAULT_THEME`
    pub fn load() -> Result<Self> {
        let path = std::env::var("HASH_CONFIG").unwrap_or_else(|_| "config.toml".to_string());

        if let Ok(contents) = fs::read_to_string(&path) {
            return toml::from_str(&contents)
                .with_context(|| format!("Failed to parse config file: {path}"));
        }

        // No config file — build from environment variables.
        Ok(Config {
            server: ServerConfig {
                host: std::env::var("HASH_HOST").unwrap_or_else(|_| default_host()),
                port: std::env::var("HASH_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_port),
            },
            vault: VaultConfig {
                path: std::env::var("HASH_VAULT_PATH")
                    .context("HASH_VAULT_PATH must be set when no config.toml is present")?,
                poll_interval_secs: std::env::var("HASH_POLL_INTERVAL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_poll_interval),
            },
            auth: AuthConfig {
                api_key: std::env::var("HASH_API_KEY")
                    .context("HASH_API_KEY must be set when no config.toml is present")?,
            },
            ui: UiConfig {
                secondary_color: std::env::var("HASH_SECONDARY_COLOR")
                    .unwrap_or_else(|_| default_secondary_color()),
                default_theme: std::env::var("HASH_DEFAULT_THEME")
                    .unwrap_or_else(|_| default_theme()),
                show_hidden_files: std::env::var("HASH_SHOW_HIDDEN_FILES")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
                line_numbers: std::env::var("HASH_LINE_NUMBERS")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
                spell_check: std::env::var("HASH_SPELL_CHECK")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
                poll_interval_secs: std::env::var("HASH_POLL_INTERVAL_UI")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_poll_interval_ui),
                large_file_threshold_kb: std::env::var("HASH_LARGE_FILE_THRESHOLD_KB")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_large_file_threshold_kb),
            },
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct UiConfig {
    /// Accent/secondary color applied throughout the UI. Any valid CSS hex color.
    #[serde(default = "default_secondary_color")]
    pub secondary_color: String,
    /// Default theme. One of: "light", "dark", "system".
    #[serde(default = "default_theme")]
    pub default_theme: String,
    /// Show hidden files (names starting with '.') in the file tree.
    /// Defaults to false. Set to true to reveal dotfiles.
    #[serde(default)]
    pub show_hidden_files: bool,
    /// Show line numbers in the markdown editor pane.
    /// Defaults to false.
    #[serde(default)]
    pub line_numbers: bool,
    /// Enable browser spell-check in the editor pane.
    /// Defaults to false (markdown syntax causes many false positives).
    #[serde(default)]
    pub spell_check: bool,
    /// How often the browser polls the vault for changes (seconds).
    /// Lower values mean faster sync; higher values reduce server load.
    #[serde(default = "default_poll_interval_ui")]
    pub poll_interval_secs: u32,
    /// Files at or above this size (KiB) use mtime+size instead of SHA-256
    /// during vault listing. Reduces I/O for large attachments.
    /// Set to 0 to always use mtime, or a very large value to always use SHA-256.
    #[serde(default = "default_large_file_threshold_kb")]
    pub large_file_threshold_kb: u32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            secondary_color: default_secondary_color(),
            default_theme: default_theme(),
            show_hidden_files: false,
            line_numbers: false,
            spell_check: false,
            poll_interval_secs: default_poll_interval_ui(),
            large_file_threshold_kb: default_large_file_threshold_kb(),
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3535
}

fn default_poll_interval() -> u64 {
    30
}

fn default_poll_interval_ui() -> u32 {
    10
}

fn default_large_file_threshold_kb() -> u32 {
    512
}

fn default_secondary_color() -> String {
    "#aaff00".to_string() // chartreuse
}

fn default_theme() -> String {
    "system".to_string()
}

/// Runtime-mutable UI settings, stored in `.mdkb/ui-settings.toml`.
///
/// Values here take precedence over the base `UiConfig` loaded from `config.toml`/env.
/// `show_hidden_files` is intentionally excluded — it controls vault initialisation
/// and cannot be changed without a server restart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub secondary_color: String,
    pub default_theme: String,
    pub line_numbers: bool,
    pub spell_check: bool,
    #[serde(default = "default_poll_interval_ui")]
    pub poll_interval_secs: u32,
    #[serde(default = "default_large_file_threshold_kb")]
    pub large_file_threshold_kb: u32,
}

impl UiSettings {
    /// Load from `.mdkb/ui-settings.toml`, falling back to the base `UiConfig` for any
    /// missing fields.
    pub fn load_from_vault(vault: &crate::vault::Vault, base: &UiConfig) -> Self {
        let defaults = Self::from_base(base);
        match vault.read_file(".mdkb/ui-settings.toml") {
            Ok(bytes) => {
                let text = std::str::from_utf8(&bytes).unwrap_or("");
                toml::from_str::<Self>(text).unwrap_or(defaults)
            }
            Err(_) => defaults,
        }
    }

    /// Persist the current settings to `.mdkb/ui-settings.toml`.
    pub fn save_to_vault(&self, vault: &crate::vault::Vault) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize ui-settings")?;
        vault.write_file(".mdkb/ui-settings.toml", content.as_bytes())
    }

    fn from_base(base: &UiConfig) -> Self {
        Self {
            secondary_color: base.secondary_color.clone(),
            default_theme: base.default_theme.clone(),
            line_numbers: base.line_numbers,
            spell_check: base.spell_check,
            poll_interval_secs: base.poll_interval_secs,
            large_file_threshold_kb: base.large_file_threshold_kb,
        }
    }
}
