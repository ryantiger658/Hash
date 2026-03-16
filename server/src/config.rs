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
    /// Load configuration from config.toml, or the path in NOTCH_CONFIG env var.
    pub fn load() -> Result<Self> {
        let path = std::env::var("HASH_CONFIG").unwrap_or_else(|_| "config.toml".to_string());
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {path}"))?;
        let config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config.toml")?;
        Ok(config)
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
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            secondary_color: default_secondary_color(),
            default_theme: default_theme(),
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
    "#6366f1".to_string() // indigo
}

fn default_theme() -> String {
    "system".to_string()
}
