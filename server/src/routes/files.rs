use crate::{vault::DEFAULT_LARGE_FILE_THRESHOLD, AppState};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;

/// GET /api/files — list all files in the vault.
pub async fn list_files(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let threshold = {
        let s = state.ui_settings.read().unwrap();
        s.large_file_threshold_kb as u64 * 1024
    };
    state.vault.list_files(threshold).map(Json).map_err(|e| {
        tracing::error!("list_files error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

/// GET /api/checksum/*path — lightweight single-file change detection.
///
/// Returns `{"checksum": "...", "modified": <unix_secs>}`.
/// Used by the browser's fast open-file poll (every ~2 s) so only one file
/// is checked rather than the whole vault.
pub async fn get_file_checksum(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let threshold = {
        let s = state.ui_settings.read().unwrap();
        s.large_file_threshold_kb as u64 * 1024
    };
    // Fall back to the compile-time default if ui_settings somehow returns 0.
    let threshold = if threshold == 0 {
        DEFAULT_LARGE_FILE_THRESHOLD
    } else {
        threshold
    };
    state
        .vault
        .file_checksum(&path, threshold)
        .map(|(checksum, modified)| {
            Json(serde_json::json!({ "checksum": checksum, "modified": modified }))
        })
        .map_err(|e| {
            tracing::warn!("get_file_checksum({path}) error: {e}");
            StatusCode::NOT_FOUND
        })
}

/// GET /api/files/*path — download a single file's raw contents.
pub async fn get_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    state.vault.read_file(&path).map(Bytes::from).map_err(|e| {
        tracing::warn!("get_file({path}) error: {e}");
        StatusCode::NOT_FOUND
    })
}

/// PUT /api/files/*path — create or overwrite a file.
pub async fn put_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    state
        .vault
        .write_file(&path, &body)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| {
            tracing::error!("put_file({path}) error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// DELETE /api/files/*path — delete a file.
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state
        .vault
        .delete_file(&path)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| {
            tracing::warn!("delete_file({path}) error: {e}");
            StatusCode::NOT_FOUND
        })
}

/// POST /api/files/rename — rename or move a file or directory within the vault.
pub async fn rename_file(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RenamePayload>,
) -> Result<StatusCode, StatusCode> {
    state
        .vault
        .rename(&payload.from, &payload.to)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| {
            tracing::warn!("rename({} -> {}) error: {e}", payload.from, payload.to);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[derive(serde::Deserialize)]
pub struct RenamePayload {
    pub from: String,
    pub to: String,
}

/// DELETE /api/dirs/*path — recursively delete a directory and all its contents.
pub async fn delete_dir(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state
        .vault
        .delete_dir(&path)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| {
            tracing::warn!("delete_dir({path}) error: {e}");
            StatusCode::NOT_FOUND
        })
}
