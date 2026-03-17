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

    let files = state
        .vault
        .list_files(crate::vault::DEFAULT_LARGE_FILE_THRESHOLD)
        .map_err(|e| {
            tracing::error!("search list_files error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Collect filename matches first (higher relevance), then content-only matches.
    let mut filename_results: Vec<SearchResult> = Vec::new();
    let mut content_results: Vec<SearchResult> = Vec::new();

    for file in files {
        // Only search markdown files; skip directories and binary attachments.
        if file.is_dir || !file.path.ends_with(".md") {
            continue;
        }

        let filename_match = file.path.to_lowercase().contains(&query);

        let bytes = match state.vault.read_file(&file.path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let content = match std::str::from_utf8(&bytes) {
            Ok(s) => s,
            Err(_) => continue,
        };

        if filename_match {
            // Filename match takes priority — surface it first, no snippet needed.
            filename_results.push(SearchResult {
                path: file.path.clone(),
                snippet: String::new(),
                line: 0,
            });
        } else {
            // Content-only match: return first matching line as snippet.
            for (i, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&query) {
                    content_results.push(SearchResult {
                        path: file.path.clone(),
                        snippet: line.trim().to_string(),
                        line: i + 1,
                    });
                    break;
                }
            }
        }
    }

    filename_results.extend(content_results);
    Ok(Json(filename_results))
}
