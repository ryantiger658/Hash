use crate::AppState;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

mod assets;
mod auth;
mod files;
mod search;
mod sync;
mod ui;

/// Build the main Axum router with all routes and middleware.
pub fn build_router(state: Arc<AppState>) -> Router {
    // Public routes — no auth required.
    let public_api = Router::new()
        .route("/ui-config", get(ui::get_ui_config))
        // Vault assets use session-token auth (query param) so <img> tags work
        .route("/vault-asset/*path", get(assets::get_vault_asset));

    // Protected routes — API key required.
    let protected_api = Router::new()
        // Vault file operations
        .route("/files", get(files::list_files))
        .route("/checksum/*path", get(files::get_file_checksum))
        .route("/files/*path", get(files::get_file))
        .route("/files/*path", put(files::put_file))
        .route("/files/*path", delete(files::delete_file))
        .route("/files/rename", post(files::rename_file))
        .route("/dirs/*path", delete(files::delete_dir))
        // Full-text search
        .route("/search", get(search::search))
        // Session token for vault-asset image serving
        .route("/auth/session", post(assets::create_session))
        // Mutable UI settings
        .route("/ui-config", post(ui::post_ui_config))
        // Sync endpoints (used by desktop clients)
        .route("/sync/snapshot", get(sync::get_snapshot))
        .route("/sync/push", post(sync::push_changes))
        // Require API key on all protected routes
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    Router::new()
        .nest("/api", public_api.merge(protected_api))
        // Serve the compiled Svelte frontend from the static/ directory
        .fallback_service(tower_http::services::ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
