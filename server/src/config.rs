use anyhow::{Context, Result};
use serde::Deserialize;
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
                editor_labels: std::env::var("HASH_EDITOR_LABELS")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
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
    /// Show text labels on editor mode buttons (Edit/Split/Preview).
    /// Defaults to false (icon-only). Set to true to restore text labels.
    #[serde(default)]
    pub editor_labels: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            secondary_color: default_secondary_color(),
            default_theme: default_theme(),
            editor_labels: false,
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

fn default_secondary_color() -> String {
    "#aaff00".to_string() // chartreuse
}

fn default_theme() -> String {
    "system".to_string()
}
