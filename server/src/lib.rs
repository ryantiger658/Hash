pub mod config;
pub mod migrations;
pub mod routes;
pub mod sync;
pub mod vault;

use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    time::Instant,
};
use tracing::info;

/// TTL for image session tokens: 24 hours.
pub const SESSION_TOKEN_TTL_SECS: u64 = 86_400;

/// In-memory store of short-lived session tokens used for vault-asset image auth.
/// Keys are opaque UUIDs; values are the time the token was issued.
/// The actual API key never appears in image URLs — only the token does.
pub type TokenStore = Arc<Mutex<HashMap<String, Instant>>>;

/// Shared application state passed to all Axum route handlers.
pub struct AppState {
    pub config: config::Config,
    pub vault: vault::Vault,
    /// Mutable UI settings, initially loaded from .mdkb/ui-settings.toml and
    /// overrideable at runtime via POST /api/ui-config.
    pub ui_settings: Arc<RwLock<config::UiSettings>>,
    /// Session tokens for vault-asset image serving (see `POST /api/auth/session`).
    pub tokens: TokenStore,
}

/// Start the server. Called by main.rs.
pub async fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hash_server=info,tower_http=info".into()),
        )
        .init();

    let config = config::Config::load()?;
    info!("Vault path: {}", config.vault.path);

    let vault = vault::Vault::new(&config.vault.path, config.ui.show_hidden_files);

    // Apply any pending vault schema migrations before accepting requests.
    if let Err(e) = migrations::run(&vault) {
        tracing::warn!("Vault migration did not complete cleanly: {e}");
    }

    let ui_settings = config::UiSettings::load_from_vault(&vault, &config.ui);
    let tokens: TokenStore = Arc::new(Mutex::new(HashMap::new()));
    let state = Arc::new(AppState {
        config,
        vault,
        ui_settings: Arc::new(RwLock::new(ui_settings)),
        tokens,
    });
    let app = routes::build_router(state.clone());

    let bind_addr = format!("{}:{}", state.config.server.host, state.config.server.port);
    info!("#ash listening on http://{}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
