use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct SearchParams {
    q: String,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

fn default_limit() -> usize {
    20
}

/// GET /api/search?q=<query>[&limit=20][&offset=0]
///
/// Returns BM25-ranked results with snippets when the Tantivy index is ready.
/// Falls back to a linear scan if the index is unavailable.
///
/// Supports query prefixes:
///   `tag:<term>`   — search only tags
///   `title:<term>` — search only title
///   `path:<prefix>` — filter results to files under the given path prefix
pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let q = params.q.trim();
    if q.is_empty() {
        return Ok(Json(crate::search_index::SearchResponse {
            total: 0,
            results: vec![],
        }));
    }

    // Pull out `path:<prefix>` as a post-filter; pass remaining query to the index.
    let (path_prefix, query_str) = extract_path_prefix(q);

    if let Some(idx) = &state.search_index {
        let response = idx.search(
            query_str,
            params.limit,
            params.offset,
            path_prefix.as_deref(),
        );
        return Ok(Json(response));
    }

    // ── Fallback: linear scan (no Tantivy) ───────────────────────────────────
    Ok(Json(linear_search(
        &state,
        query_str,
        params.limit,
        params.offset,
        path_prefix.as_deref(),
    )))
}

/// Shared search entry point for local integrations such as MCP.
pub(crate) fn search_for_mcp(
    state: &AppState,
    query: &str,
    limit: usize,
) -> crate::search_index::SearchResponse {
    let (path_prefix, query_str) = extract_path_prefix(query);
    if let Some(index) = &state.search_index {
        index.search(query_str, limit, 0, path_prefix.as_deref())
    } else {
        linear_search(state, query_str, limit, 0, path_prefix.as_deref())
    }
}

/// Strip a leading `path:<prefix>` token and return (prefix, remaining_query).
fn extract_path_prefix(q: &str) -> (Option<String>, &str) {
    if let Some(rest) = q.strip_prefix("path:") {
        // `path:journal/` with optional trailing query terms
        let mut parts = rest.splitn(2, char::is_whitespace);
        let prefix = parts.next().unwrap_or("").to_string();
        let remaining = parts.next().unwrap_or("").trim();
        (Some(prefix), remaining)
    } else {
        (None, q)
    }
}

/// Simple O(n) fallback when the Tantivy index is not available.
fn linear_search(
    state: &AppState,
    query: &str,
    limit: usize,
    offset: usize,
    path_prefix: Option<&str>,
) -> crate::search_index::SearchResponse {
    use crate::search_index::{SearchResponse, SearchResult};

    let q_lower = query.to_lowercase();
    let Ok(files) = state
        .vault
        .list_files(crate::vault::DEFAULT_LARGE_FILE_THRESHOLD)
    else {
        return SearchResponse {
            total: 0,
            results: vec![],
        };
    };

    let mut filename_hits: Vec<SearchResult> = Vec::new();
    let mut content_hits: Vec<SearchResult> = Vec::new();

    for file in files {
        if file.is_dir || !file.path.ends_with(".md") {
            continue;
        }
        if let Some(pfx) = path_prefix {
            if !file.path.starts_with(pfx) {
                continue;
            }
        }

        let fname_match = file.path.to_lowercase().contains(&q_lower);
        let Ok(bytes) = state.vault.read_file(&file.path) else {
            continue;
        };
        let Ok(content) = std::str::from_utf8(&bytes) else {
            continue;
        };

        let title = content
            .lines()
            .find(|l| l.starts_with("# "))
            .map(|l| l[2..].trim().to_string())
            .unwrap_or_else(|| {
                std::path::Path::new(&file.path)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default()
            });

        if fname_match {
            filename_hits.push(SearchResult {
                path: file.path,
                title,
                score: 1.0,
                snippets: vec![],
            });
        } else {
            for line in content.lines() {
                if line.to_lowercase().contains(&q_lower) {
                    content_hits.push(SearchResult {
                        path: file.path,
                        title,
                        score: 0.5,
                        snippets: vec![line.trim().to_string()],
                    });
                    break;
                }
            }
        }
    }

    filename_hits.extend(content_hits);
    let total = filename_hits.len();
    let results = filename_hits.into_iter().skip(offset).take(limit).collect();
    SearchResponse { total, results }
}
