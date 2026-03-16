use crate::AppState;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

mod auth;
mod files;
mod sync;
mod ui;

/// Build the main Axum router with all routes and middleware.
pub fn build_router(state: Arc<AppState>) -> Router {
    // Public routes — no auth required.
    let public_api = Router::new()
        .route("/ui-config", get(ui::get_ui_config));

    // Protected routes — API key required.
    let protected_api = Router::new()
        // Vault file operations
        .route("/files", get(files::list_files))
        .route("/files/*path", get(files::get_file))
        .route("/files/*path", put(files::put_file))
        .route("/files/*path", delete(files::delete_file))
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
