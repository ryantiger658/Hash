use crate::{
    sync::{DeleteItem, PushItem, PushResult, RejectedItem, VaultSnapshot},
    AppState,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use std::sync::Arc;

/// GET /api/sync/snapshot — return the full vault file listing.
///
/// Desktop clients call this to compute which files need to be pushed or pulled.
pub async fn get_snapshot(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let files = state.vault.list_files().map_err(|e| {
        tracing::error!("snapshot error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let snapshot = VaultSnapshot {
        server_time: Utc::now().timestamp(),
        files,
    };

    Ok(Json(snapshot))
}

/// POST /api/sync/push — accept file changes from a desktop client.
///
/// The client sends a list of files to upsert and a list of paths to delete.
/// Uses last-write-wins: the incoming content always overwrites the server copy.
pub async fn push_changes(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PushPayload>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();

    for item in payload.upsert {
        match BASE64.decode(&item.content) {
            Ok(bytes) => match state.vault.write_file(&item.path, &bytes) {
                Ok(_) => accepted.push(item.path),
                Err(e) => rejected.push(RejectedItem {
                    path: item.path,
                    reason: e.to_string(),
                }),
            },
            Err(_) => rejected.push(RejectedItem {
                path: item.path,
                reason: "Invalid base64 encoding".to_string(),
            }),
        }
    }

    for item in payload.delete {
        match state.vault.delete_file(&item.path) {
            Ok(_) => accepted.push(item.path),
            Err(e) => rejected.push(RejectedItem {
                path: item.path,
                reason: e.to_string(),
            }),
        }
    }

    Ok(Json(PushResult { accepted, rejected }))
}

#[derive(serde::Deserialize)]
pub struct PushPayload {
    #[serde(default)]
    pub upsert: Vec<PushItem>,
    #[serde(default)]
    pub delete: Vec<DeleteItem>,
}
