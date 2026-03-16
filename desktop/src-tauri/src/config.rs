use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Desktop client configuration — stored in the OS config directory.
/// e.g. ~/.config/notch/config.toml (Linux) or ~/Library/Application Support/notch/config.toml (macOS)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientConfig {
    /// URL of the #ash server, e.g. "http://192.168.1.100:3535"
    pub server_url: String,
    /// API key matching the server's config.toml [auth] api_key
    pub api_key: String,
    /// Local directory to sync the vault into
    pub local_vault_path: String,
    /// How often to auto-sync when connected (seconds)
    #[serde(default = "default_sync_interval")]
    pub sync_interval_secs: u64,
}

impl ClientConfig {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        let contents = std::fs::read_to_string(&path)
            .map_err(|_| anyhow::anyhow!("Config not found at {:?}. Run setup first.", path))?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

fn config_path() -> Result<std::path::PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    Ok(base.join("hash").join("config.toml"))
}

fn default_sync_interval() -> u64 {
    60
}
