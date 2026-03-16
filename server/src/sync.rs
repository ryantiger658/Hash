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
#[allow(dead_code)] // `modified` will be used in M2 for conflict detection
pub struct PushItem {
    /// Vault-relative path.
    pub path: String,
    /// Base64-encoded file contents.
    pub content: String,
    /// Client-side last modified timestamp (Unix seconds).
    pub modified: i64,
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
}

#[derive(Debug, Serialize)]
pub struct RejectedItem {
    pub path: String,
    pub reason: String,
}
