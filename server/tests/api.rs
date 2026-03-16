//! Integration tests for the #ash REST API.
//!
//! Each test spins up a full Axum router backed by a temporary vault directory.
//! No network is involved — requests are dispatched via `tower::ServiceExt::oneshot`.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use hash_server::{
    config::{AuthConfig, Config, ServerConfig, UiConfig, VaultConfig},
    routes::build_router,
    vault::Vault,
    AppState,
};
use http_body_util::BodyExt;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

// ── Test helpers ──────────────────────────────────────────────────────────────

const TEST_KEY: &str = "test-api-key";

/// Create a router backed by a fresh temporary vault.
/// Returns the router and the TempDir (must be kept alive for the test).
fn make_app() -> (Router, TempDir) {
    make_app_with_key(TEST_KEY)
}

fn make_app_with_key(api_key: &str) -> (Router, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        server: ServerConfig {
            host: "127.0.0.1".into(),
            port: 3535,
        },
        vault: VaultConfig {
            path: dir.path().to_str().unwrap().into(),
            poll_interval_secs: 30,
        },
        auth: AuthConfig {
            api_key: api_key.into(),
        },
        ui: UiConfig::default(),
    };
    let vault = Vault::new(&config.vault.path);
    let state = Arc::new(AppState { config, vault });
    (build_router(state), dir)
}

/// Build an authenticated GET request.
fn auth_get(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("Authorization", format!("Bearer {TEST_KEY}"))
        .body(Body::empty())
        .unwrap()
}

/// Build an authenticated PUT request with a text body.
fn auth_put(uri: &str, body: &'static str) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header("Authorization", format!("Bearer {TEST_KEY}"))
        .header("Content-Type", "text/plain")
        .body(Body::from(body))
        .unwrap()
}

/// Build an authenticated DELETE request.
fn auth_delete(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("Authorization", format!("Bearer {TEST_KEY}"))
        .body(Body::empty())
        .unwrap()
}

/// Collect the response body as a String.
async fn body_string(res: axum::response::Response) -> String {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// Collect and parse the response body as JSON.
async fn body_json(res: axum::response::Response) -> serde_json::Value {
    let text = body_string(res).await;
    serde_json::from_str(&text).unwrap()
}

// ── Authentication ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn auth_missing_header_returns_401() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_wrong_key_returns_401() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/files")
                .header("Authorization", "Bearer wrong-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_correct_key_succeeds() {
    let (app, _dir) = make_app();
    let res = app.oneshot(auth_get("/api/files")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_bearer_prefix_required() {
    let (app, _dir) = make_app();
    // Key without "Bearer " prefix should be rejected
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/files")
                .header("Authorization", TEST_KEY)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

// ── Public routes (no auth) ───────────────────────────────────────────────────

#[tokio::test]
async fn ui_config_requires_no_auth() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/ui-config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn ui_config_returns_secondary_color() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/ui-config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(res).await;
    assert!(json["secondary_color"].is_string());
    assert!(json["default_theme"].is_string());
}

// ── File listing ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_files_empty_vault() {
    let (app, _dir) = make_app();
    let res = app.oneshot(auth_get("/api/files")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_files_after_create() {
    let (app, dir) = make_app();
    // Pre-populate vault directly
    std::fs::write(dir.path().join("note.md"), "# Note").unwrap();
    let res = app.oneshot(auth_get("/api/files")).await.unwrap();
    let json = body_json(res).await;
    let paths: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"note.md"));
}

// ── File CRUD ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn put_and_get_file() {
    let (app, _dir) = make_app();

    // Create the file
    let put_res = app
        .clone()
        .oneshot(auth_put("/api/files/hello.md", "# Hello World"))
        .await
        .unwrap();
    assert_eq!(put_res.status(), StatusCode::NO_CONTENT);

    // Read it back
    let get_res = app.oneshot(auth_get("/api/files/hello.md")).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);
    assert_eq!(body_string(get_res).await, "# Hello World");
}

#[tokio::test]
async fn put_creates_parent_directories() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(auth_put(
            "/api/files/projects/rust/notes.md",
            "nested content",
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn put_overwrites_existing_file() {
    let (app, _dir) = make_app();
    app.clone()
        .oneshot(auth_put("/api/files/edit.md", "original"))
        .await
        .unwrap();
    app.clone()
        .oneshot(auth_put("/api/files/edit.md", "updated"))
        .await
        .unwrap();
    let res = app.oneshot(auth_get("/api/files/edit.md")).await.unwrap();
    assert_eq!(body_string(res).await, "updated");
}

#[tokio::test]
async fn get_nonexistent_file_returns_404() {
    let (app, _dir) = make_app();
    let res = app.oneshot(auth_get("/api/files/ghost.md")).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_file() {
    let (app, _dir) = make_app();
    app.clone()
        .oneshot(auth_put("/api/files/bye.md", "goodbye"))
        .await
        .unwrap();

    let del_res = app
        .clone()
        .oneshot(auth_delete("/api/files/bye.md"))
        .await
        .unwrap();
    assert_eq!(del_res.status(), StatusCode::NO_CONTENT);

    // Confirm it's gone
    let get_res = app.oneshot(auth_get("/api/files/bye.md")).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nonexistent_returns_404() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(auth_delete("/api/files/nobody.md"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn path_with_spaces_encoded() {
    let (app, _dir) = make_app();
    let res = app
        .clone()
        .oneshot(auth_put(
            "/api/files/Getting%20Started.md",
            "# Getting Started",
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let get_res = app
        .oneshot(auth_get("/api/files/Getting%20Started.md"))
        .await
        .unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);
    assert_eq!(body_string(get_res).await, "# Getting Started");
}

// ── Path traversal protection ─────────────────────────────────────────────────

#[tokio::test]
async fn path_traversal_get_blocked() {
    let (app, _dir) = make_app();
    // %2F is /, %2E%2E is ..
    let res = app
        .oneshot(auth_get("/api/files/..%2Fetc%2Fpasswd"))
        .await
        .unwrap();
    assert_ne!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn path_traversal_put_blocked() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(auth_put("/api/files/..%2F..%2Fevil.txt", "evil"))
        .await
        .unwrap();
    assert_ne!(res.status(), StatusCode::NO_CONTENT);
}

// ── Search ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn search_finds_content_match() {
    let (app, dir) = make_app();
    std::fs::write(dir.path().join("rust.md"), "# Rust\nOwnership is great").unwrap();
    std::fs::write(dir.path().join("python.md"), "# Python\nDynamic typing").unwrap();

    let res = app
        .oneshot(auth_get("/api/search?q=Ownership"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    let results = json.as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["path"], "rust.md");
    assert!(results[0]["snippet"]
        .as_str()
        .unwrap()
        .contains("Ownership"));
}

#[tokio::test]
async fn search_is_case_insensitive() {
    let (app, dir) = make_app();
    std::fs::write(
        dir.path().join("note.md"),
        "# Cargo is the Rust package manager",
    )
    .unwrap();
    let res = app.oneshot(auth_get("/api/search?q=cargo")).await.unwrap();
    let json = body_json(res).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn search_returns_empty_for_no_match() {
    let (app, dir) = make_app();
    std::fs::write(dir.path().join("note.md"), "hello world").unwrap();
    let res = app
        .oneshot(auth_get("/api/search?q=zylophone"))
        .await
        .unwrap();
    let json = body_json(res).await;
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn search_matches_filename() {
    let (app, dir) = make_app();
    std::fs::write(dir.path().join("quarterly-review.md"), "some content").unwrap();
    let res = app
        .oneshot(auth_get("/api/search?q=quarterly"))
        .await
        .unwrap();
    let json = body_json(res).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["path"], "quarterly-review.md");
}

#[tokio::test]
async fn search_empty_query_returns_empty() {
    let (app, dir) = make_app();
    std::fs::write(dir.path().join("note.md"), "content").unwrap();
    let res = app.oneshot(auth_get("/api/search?q=")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body_json(res).await.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn search_skips_non_markdown_files() {
    let (app, dir) = make_app();
    // A binary/non-md file that matches the query
    std::fs::write(dir.path().join("image.png"), "searchterm-in-binary").unwrap();
    std::fs::write(dir.path().join("note.md"), "no match here").unwrap();
    let res = app
        .oneshot(auth_get("/api/search?q=searchterm"))
        .await
        .unwrap();
    // Should not find the .png file
    assert_eq!(body_json(res).await.as_array().unwrap().len(), 0);
}

// ── Sync API ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn snapshot_returns_file_list() {
    let (app, dir) = make_app();
    std::fs::write(dir.path().join("snap.md"), "snapshot test").unwrap();
    let res = app.oneshot(auth_get("/api/sync/snapshot")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert!(json["server_time"].is_number());
    let files = json["files"].as_array().unwrap();
    assert!(files.iter().any(|f| f["path"] == "snap.md"));
}

#[tokio::test]
async fn push_upserts_file() {
    let (app, dir) = make_app();
    use base64::Engine;
    let content = base64::engine::general_purpose::STANDARD.encode("# Pushed note");

    let body = serde_json::json!({
        "upsert": [{ "path": "pushed.md", "content": content, "modified": 0 }],
        "delete": []
    });

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/sync/push")
                .header("Authorization", format!("Bearer {TEST_KEY}"))
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert!(json["accepted"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("pushed.md")));
    assert!(dir.path().join("pushed.md").exists());
}

#[tokio::test]
async fn push_deletes_file() {
    let (app, dir) = make_app();
    std::fs::write(dir.path().join("remove.md"), "to delete").unwrap();

    let body = serde_json::json!({
        "upsert": [],
        "delete": [{ "path": "remove.md" }]
    });

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/sync/push")
                .header("Authorization", format!("Bearer {TEST_KEY}"))
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert!(!dir.path().join("remove.md").exists());
}

#[tokio::test]
async fn push_invalid_base64_reports_rejection() {
    let (app, _dir) = make_app();

    let body = serde_json::json!({
        "upsert": [{ "path": "bad.md", "content": "not-valid-base64!!!", "modified": 0 }],
        "delete": []
    });

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/sync/push")
                .header("Authorization", format!("Bearer {TEST_KEY}"))
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json["accepted"].as_array().unwrap().len(), 0);
    assert_eq!(json["rejected"].as_array().unwrap().len(), 1);
}
