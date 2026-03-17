use crate::{
    sync::{ConflictItem, DeleteItem, PushItem, PushResult, RejectedItem, SyncMeta, VaultSnapshot},
    AppState,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// GET /api/sync/snapshot — return the full vault file listing with checksums and timestamps.
///
/// Desktop clients call this to compute which files need to be pushed or pulled.
pub async fn get_snapshot(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let files = state
        .vault
        .list_files(crate::vault::DEFAULT_LARGE_FILE_THRESHOLD)
        .map_err(|e| {
            tracing::error!("snapshot error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(VaultSnapshot {
        server_time: Utc::now().timestamp(),
        files,
    }))
}

/// POST /api/sync/push — accept file changes from a desktop client.
///
/// Each upsert item may carry `last_synced_checksum` and `last_synced_timestamp`
/// for conflict detection. A conflict is declared when:
///   - The server file was modified since the client's last sync (server checksum ≠
///     last_synced_checksum), AND
///   - The server file's mtime is newer than the client's last_synced_timestamp.
///
/// Accepted upserts update `.mdkb/sync/<path>.toml`. Conflicting items are returned
/// to the client for resolution; the client re-pushes the resolved content.
pub async fn push_changes(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PushPayload>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    let mut conflicts = Vec::new();

    let now = Utc::now().timestamp();

    for item in payload.upsert {
        // Decode content first — reject early if invalid
        let bytes = match BASE64.decode(&item.content) {
            Ok(b) => b,
            Err(_) => {
                rejected.push(RejectedItem {
                    path: item.path,
                    reason: "Invalid base64 encoding".to_string(),
                });
                continue;
            }
        };

        // Check for conflicts if the client supplied sync metadata
        if !item.last_synced_checksum.is_empty() {
            if let Ok(server_bytes) = state.vault.read_file(&item.path) {
                let server_checksum = hex::encode(Sha256::digest(&server_bytes));
                let server_modified = state
                    .vault
                    .list_files(crate::vault::DEFAULT_LARGE_FILE_THRESHOLD)
                    .ok()
                    .and_then(|fs| fs.into_iter().find(|f| f.path == item.path))
                    .map(|f| f.modified)
                    .unwrap_or(0);

                let server_changed = server_checksum != item.last_synced_checksum;
                let server_newer = server_modified > item.last_synced_timestamp;

                if server_changed && server_newer {
                    conflicts.push(ConflictItem {
                        path: item.path,
                        server_checksum,
                        server_modified,
                        server_content: BASE64.encode(&server_bytes),
                    });
                    continue;
                }
            }
        }

        // Accept the upsert
        match state.vault.write_file(&item.path, &bytes) {
            Ok(_) => {
                let written_checksum = hex::encode(Sha256::digest(&bytes));
                SyncMeta::write(&state.vault, &item.path, &written_checksum, now);
                accepted.push(item.path);
            }
            Err(e) => {
                rejected.push(RejectedItem {
                    path: item.path,
                    reason: e.to_string(),
                });
            }
        }
    }

    for item in payload.delete {
        match state.vault.delete_file(&item.path) {
            Ok(_) => {
                SyncMeta::delete(&state.vault, &item.path);
                accepted.push(item.path);
            }
            Err(e) => {
                rejected.push(RejectedItem {
                    path: item.path,
                    reason: e.to_string(),
                });
            }
        }
    }

    Ok(Json(PushResult {
        accepted,
        rejected,
        conflicts,
    }))
}

#[derive(serde::Deserialize)]
pub struct PushPayload {
    #[serde(default)]
    pub upsert: Vec<PushItem>,
    #[serde(default)]
    pub delete: Vec<DeleteItem>,
}
