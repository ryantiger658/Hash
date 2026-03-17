use crate::{AppState, SESSION_TOKEN_TTL_SECS};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::{sync::Arc, time::Instant};
use uuid::Uuid;

/// POST /api/auth/session — exchange a valid Bearer token for a short-lived session token.
///
/// The session token is an opaque UUID stored in memory. It is used as a `?token=` query
/// parameter on vault-asset image URLs so the real API key never appears in image URLs or logs.
pub async fn create_session(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let token = Uuid::new_v4().to_string();
    state
        .tokens
        .lock()
        .unwrap()
        .insert(token.clone(), Instant::now());
    axum::Json(serde_json::json!({ "token": token }))
}

/// GET /api/vault-asset/*path?token=<session_token> — serve a vault file as a raw asset.
///
/// Auth is via the `token` query parameter (a UUID session token, not the API key).
/// Tokens expire after SESSION_TOKEN_TTL_SECS (24 h). Path traversal is blocked by
/// the same vault canonicalization used everywhere else.
pub async fn get_vault_asset(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(params): Query<AssetQuery>,
) -> Response {
    // Validate session token
    let valid = {
        let store = state.tokens.lock().unwrap();
        store
            .get(&params.token)
            .is_some_and(|issued| issued.elapsed().as_secs() < SESSION_TOKEN_TTL_SECS)
    };
    if !valid {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // Read the file (path-traversal protected inside vault.read_file)
    let bytes = match state.vault.read_file(&path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("vault_asset({path:?}) error: {e}");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let content_type = content_type_for(&path);
    ([(header::CONTENT_TYPE, content_type)], Bytes::from(bytes)).into_response()
}

#[derive(serde::Deserialize)]
pub struct AssetQuery {
    #[serde(default)]
    pub token: String,
}

/// Infer a Content-Type string from a file extension.
fn content_type_for(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}
