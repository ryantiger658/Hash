use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct SearchParams {
    q: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    /// Vault-relative file path.
    pub path: String,
    /// The matching line, trimmed. Empty if the match was on the filename.
    pub snippet: String,
    /// 1-based line number of the match, 0 if filename-only match.
    pub line: usize,
}

/// GET /api/search?q=<query>
///
/// Case-insensitive full-text search across all markdown files in the vault.
/// Returns the first matching line per file plus any filename matches.
pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let query = params.q.trim().to_lowercase();
    if query.is_empty() {
        return Ok(Json(vec![]));
    }

    let files = state.vault.list_files().map_err(|e| {
        tracing::error!("search list_files error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut results: Vec<SearchResult> = Vec::new();

    for file in files {
        // Only search markdown files; skip binary attachments.
        if !file.path.ends_with(".md") {
            continue;
        }

        let bytes = match state.vault.read_file(&file.path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let content = match std::str::from_utf8(&bytes) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Search file contents line by line.
        let mut matched_content = false;
        for (i, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query) {
                results.push(SearchResult {
                    path: file.path.clone(),
                    snippet: line.trim().to_string(),
                    line: i + 1,
                });
                matched_content = true;
                break; // one snippet per file keeps results clean
            }
        }

        // Also match on filename if not already matched.
        if !matched_content && file.path.to_lowercase().contains(&query) {
            results.push(SearchResult {
                path: file.path.clone(),
                snippet: String::new(),
                line: 0,
            });
        }
    }

    Ok(Json(results))
}
