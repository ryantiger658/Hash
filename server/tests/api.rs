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
    let vault = Vault::new(&config.vault.path, false);
    let ui_settings = hash_server::config::UiSettings::load_from_vault(&vault, &config.ui);
    let state = Arc::new(AppState {
        config,
        vault,
        ui_settings: std::sync::Arc::new(std::sync::RwLock::new(ui_settings)),
        tokens: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
    });
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

/// Build an authenticated POST request with a JSON body.
fn auth_post_json(uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("Authorization", format!("Bearer {TEST_KEY}"))
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
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

// ── Rename ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn rename_file_moves_content() {
    let (app, _dir) = make_app();
    app.clone()
        .oneshot(auth_put("/api/files/old.md", "rename me"))
        .await
        .unwrap();

    let res = app
        .clone()
        .oneshot(auth_post_json(
            "/api/files/rename",
            serde_json::json!({ "from": "old.md", "to": "new.md" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Old path gone
    let old = app
        .clone()
        .oneshot(auth_get("/api/files/old.md"))
        .await
        .unwrap();
    assert_eq!(old.status(), StatusCode::NOT_FOUND);

    // New path has the content
    let new = app.oneshot(auth_get("/api/files/new.md")).await.unwrap();
    assert_eq!(new.status(), StatusCode::OK);
    assert_eq!(body_string(new).await, "rename me");
}

#[tokio::test]
async fn rename_into_subdirectory() {
    let (app, _dir) = make_app();
    app.clone()
        .oneshot(auth_put("/api/files/note.md", "moved"))
        .await
        .unwrap();

    let res = app
        .clone()
        .oneshot(auth_post_json(
            "/api/files/rename",
            serde_json::json!({ "from": "note.md", "to": "archive/note.md" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let moved = app
        .oneshot(auth_get("/api/files/archive/note.md"))
        .await
        .unwrap();
    assert_eq!(moved.status(), StatusCode::OK);
    assert_eq!(body_string(moved).await, "moved");
}

// ── Delete directory ──────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_dir_removes_all_contents() {
    let (app, _dir) = make_app();
    app.clone()
        .oneshot(auth_put("/api/files/notes/one.md", "one"))
        .await
        .unwrap();
    app.clone()
        .oneshot(auth_put("/api/files/notes/two.md", "two"))
        .await
        .unwrap();

    let res = app
        .clone()
        .oneshot(auth_delete("/api/dirs/notes"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Neither file should exist
    assert_eq!(
        app.clone()
            .oneshot(auth_get("/api/files/notes/one.md"))
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.oneshot(auth_get("/api/files/notes/two.md"))
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn delete_nonexistent_dir_returns_404() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(auth_delete("/api/dirs/ghost-folder"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ── Checksum endpoint ─────────────────────────────────────────────────────────

#[tokio::test]
async fn checksum_returns_checksum_and_modified_for_existing_file() {
    let (app, _dir) = make_app();
    app.clone()
        .oneshot(auth_put("/api/files/cksum.md", "checksum test"))
        .await
        .unwrap();

    let res = app
        .oneshot(auth_get("/api/checksum/cksum.md"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert!(json["checksum"].is_string());
    assert!(!json["checksum"].as_str().unwrap().is_empty());
    assert!(json["modified"].is_number());
}

#[tokio::test]
async fn checksum_returns_404_for_missing_file() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(auth_get("/api/checksum/doesnotexist.md"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn checksum_changes_after_file_update() {
    let (app, _dir) = make_app();
    app.clone()
        .oneshot(auth_put("/api/files/evolving.md", "version 1"))
        .await
        .unwrap();
    let r1 = body_json(
        app.clone()
            .oneshot(auth_get("/api/checksum/evolving.md"))
            .await
            .unwrap(),
    )
    .await;

    app.clone()
        .oneshot(auth_put("/api/files/evolving.md", "version 2"))
        .await
        .unwrap();
    let r2 = body_json(
        app.oneshot(auth_get("/api/checksum/evolving.md"))
            .await
            .unwrap(),
    )
    .await;

    assert_ne!(r1["checksum"], r2["checksum"]);
}

// ── UI config POST ────────────────────────────────────────────────────────────

#[tokio::test]
async fn post_ui_config_updates_secondary_color() {
    let (app, _dir) = make_app();
    let res = app
        .clone()
        .oneshot(auth_post_json(
            "/api/ui-config",
            serde_json::json!({ "secondary_color": "#ff0000" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json["secondary_color"], "#ff0000");

    // Verify GET reflects the update
    let get_res = app
        .oneshot(
            Request::builder()
                .uri("/api/ui-config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(body_json(get_res).await["secondary_color"], "#ff0000");
}

#[tokio::test]
async fn post_ui_config_partial_update_preserves_other_fields() {
    let (app, _dir) = make_app();
    // Only update line_numbers
    let res = app
        .clone()
        .oneshot(auth_post_json(
            "/api/ui-config",
            serde_json::json!({ "line_numbers": true }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json["line_numbers"], true);
    // Other fields still present
    assert!(json["secondary_color"].is_string());
    assert!(json["default_theme"].is_string());
}

#[tokio::test]
async fn post_ui_config_clamps_poll_interval_to_minimum_one() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(auth_post_json(
            "/api/ui-config",
            serde_json::json!({ "poll_interval_secs": 0 }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    // Server clamps to 1
    assert_eq!(body_json(res).await["poll_interval_secs"], 1);
}

// ── Session tokens / vault assets ─────────────────────────────────────────────

#[tokio::test]
async fn create_session_returns_token() {
    let (app, _dir) = make_app();
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/session")
                .header("Authorization", format!("Bearer {TEST_KEY}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    let token = json["token"].as_str().unwrap();
    assert!(!token.is_empty());
    // UUID format: 8-4-4-4-12
    assert_eq!(token.len(), 36);
}

#[tokio::test]
async fn vault_asset_serves_file_with_valid_token() {
    let (app, dir) = make_app();
    std::fs::write(dir.path().join("image.png"), b"\x89PNG").unwrap();

    // Obtain a session token
    let session_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/session")
                .header("Authorization", format!("Bearer {TEST_KEY}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let token = body_json(session_res).await["token"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/vault-asset/image.png?token={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get("content-type").unwrap().to_str().unwrap(),
        "image/png"
    );
}

#[tokio::test]
async fn vault_asset_rejects_missing_or_invalid_token() {
    let (app, dir) = make_app();
    std::fs::write(dir.path().join("img.png"), b"data").unwrap();

    // No token at all
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/vault-asset/img.png")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid (non-existent) token
    let res2 = app
        .oneshot(
            Request::builder()
                .uri("/api/vault-asset/img.png?token=not-a-real-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res2.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn vault_asset_returns_404_for_missing_file() {
    let (app, _dir) = make_app();

    let session_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/session")
                .header("Authorization", format!("Bearer {TEST_KEY}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let token = body_json(session_res).await["token"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/vault-asset/ghost.png?token={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ── Sync conflict detection ───────────────────────────────────────────────────

#[tokio::test]
async fn push_conflict_detected_when_server_changed_independently() {
    let (app, dir) = make_app();
    use base64::Engine;

    // Server has one version of the file
    std::fs::write(dir.path().join("shared.md"), "server version").unwrap();

    // Client pushes with a stale checksum (it last saw a different version)
    // and last_synced_timestamp = 0 so the server file is always "newer"
    let client_content = base64::engine::general_purpose::STANDARD.encode("client edits");
    let body = serde_json::json!({
        "upsert": [{
            "path": "shared.md",
            "content": client_content,
            "modified": 0,
            "last_synced_checksum": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "last_synced_timestamp": 0
        }],
        "delete": []
    });

    let res = app
        .oneshot(auth_post_json("/api/sync/push", body))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json["conflicts"].as_array().unwrap().len(), 1);
    assert_eq!(json["accepted"].as_array().unwrap().len(), 0);
    // Conflict item includes the server's content for resolution
    let conflict = &json["conflicts"][0];
    assert_eq!(conflict["path"], "shared.md");
    assert!(conflict["server_content"].is_string());
    assert!(conflict["server_checksum"].is_string());
}

#[tokio::test]
async fn push_accepted_when_last_synced_checksum_matches_server() {
    let (app, _dir) = make_app();
    use base64::Engine;
    use sha2::{Digest, Sha256};

    // Create a file via the API
    app.clone()
        .oneshot(auth_put("/api/files/synced.md", "agreed content"))
        .await
        .unwrap();

    // Compute the current server checksum
    let current_checksum = hex::encode(Sha256::digest(b"agreed content"));

    // Client pushes with matching last_synced_checksum — no conflict
    let new_content = base64::engine::general_purpose::STANDARD.encode("updated content");
    let body = serde_json::json!({
        "upsert": [{
            "path": "synced.md",
            "content": new_content,
            "modified": 0,
            "last_synced_checksum": current_checksum,
            "last_synced_timestamp": 0
        }],
        "delete": []
    });

    let res = app
        .oneshot(auth_post_json("/api/sync/push", body))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json = body_json(res).await;
    assert_eq!(json["conflicts"].as_array().unwrap().len(), 0);
    assert!(json["accepted"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("synced.md")));
}
