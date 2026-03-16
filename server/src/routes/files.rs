use crate::AppState;
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
    state
        .vault
        .list_files()
        .map(|files| Json(files))
        .map_err(|e| {
            tracing::error!("list_files error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
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
