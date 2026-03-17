use serde::{Deserialize, Serialize};

/// A snapshot of the vault sent to clients so they can compute a delta.
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultSnapshot {
    /// Server's current Unix timestamp.
    pub server_time: i64,
    /// All files currently in the vault.
    pub files: Vec<crate::vault::FileEntry>,
}

/// A single file change the client wants to push to the server.
#[derive(Debug, Deserialize)]
pub struct PushItem {
    /// Vault-relative path.
    pub path: String,
    /// Base64-encoded file contents.
    pub content: String,
    /// SHA-256 hex checksum of the content being pushed (used for conflict detection).
    #[serde(default)]
    pub checksum: String,
    /// Client-side last modified timestamp (Unix seconds).
    #[serde(default)]
    pub modified: i64,
    /// Checksum of the file at the time of the client's last successful sync.
    /// Empty for files the client is pushing for the first time.
    #[serde(default)]
    pub last_synced_checksum: String,
    /// Unix timestamp of the client's last successful sync for this file. 0 if new.
    #[serde(default)]
    pub last_synced_timestamp: i64,
}

/// A file path the client wants to delete on the server.
#[derive(Debug, Deserialize)]
pub struct DeleteItem {
    pub path: String,
}

/// Response to a sync push.
#[derive(Debug, Serialize)]
pub struct PushResult {
    pub accepted: Vec<String>,
    pub rejected: Vec<RejectedItem>,
    /// Files where both sides changed since the last sync.
    /// The client must present a resolution UI and re-push the chosen version.
    pub conflicts: Vec<ConflictItem>,
}

#[derive(Debug, Serialize)]
pub struct RejectedItem {
    pub path: String,
    pub reason: String,
}

/// A conflict: the server copy changed independently since the client's last sync.
/// The server returns its current content so the client can diff and resolve.
#[derive(Debug, Serialize)]
pub struct ConflictItem {
    pub path: String,
    pub server_checksum: String,
    pub server_modified: i64,
    /// Base64-encoded current server content.
    pub server_content: String,
}

// ── Per-file sync metadata ────────────────────────────────────────────────────

/// Metadata stored at `.mdkb/sync/<path>.toml` tracking the last successful sync.
pub struct SyncMeta {
    pub last_synced_checksum: String,
    pub last_synced_timestamp: i64,
}

impl SyncMeta {
    pub fn read(vault: &crate::vault::Vault, file_path: &str) -> Option<Self> {
        let meta_path = format!(".mdkb/sync/{file_path}.toml");
        let bytes = vault.read_file(&meta_path).ok()?;
        let text = std::str::from_utf8(&bytes).ok()?;

        let mut checksum = String::new();
        let mut timestamp: i64 = 0;

        for line in text.lines() {
            if let Some(v) = line.trim().strip_prefix("last_synced_checksum") {
                if let Some(v) = v.trim().strip_prefix('=') {
                    checksum = v.trim().trim_matches('"').to_string();
                }
            }
            if let Some(v) = line.trim().strip_prefix("last_synced_timestamp") {
                if let Some(v) = v.trim().strip_prefix('=') {
                    timestamp = v.trim().parse().unwrap_or(0);
                }
            }
        }

        if checksum.is_empty() {
            None
        } else {
            Some(Self {
                last_synced_checksum: checksum,
                last_synced_timestamp: timestamp,
            })
        }
    }

    pub fn write(vault: &crate::vault::Vault, file_path: &str, checksum: &str, timestamp: i64) {
        let meta_path = format!(".mdkb/sync/{file_path}.toml");
        let content = format!(
            "# #ash sync metadata — do not edit manually\nlast_synced_checksum = \"{checksum}\"\nlast_synced_timestamp = {timestamp}\n"
        );
        if let Err(e) = vault.write_file(&meta_path, content.as_bytes()) {
            tracing::warn!("Failed to write sync metadata for {file_path}: {e}");
        }
    }

    pub fn delete(vault: &crate::vault::Vault, file_path: &str) {
        let meta_path = format!(".mdkb/sync/{file_path}.toml");
        let _ = vault.delete_file(&meta_path);
    }
}
