pub mod config;
pub mod migrations;
pub mod okf;
pub mod routes;
pub mod search_index;
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

/// Pending browser authorization request. Stored briefly so callback state,
/// nonce, and PKCE verifier can be checked server-side.
pub struct OidcFlow {
    pub nonce: String,
    pub pkce_verifier: String,
    pub created: Instant,
}

/// Authenticated browser session. The cookie contains only the random map key.
pub struct WebSession {
    pub subject: String,
    pub created: Instant,
}

pub type OidcFlowStore = Arc<Mutex<HashMap<String, OidcFlow>>>;
pub type WebSessionStore = Arc<Mutex<HashMap<String, WebSession>>>;

/// Shared application state passed to all Axum route handlers.
pub struct AppState {
    pub config: config::Config,
    pub vault: vault::Vault,
    /// Mutable UI settings, initially loaded from .mdkb/ui-settings.toml and
    /// overrideable at runtime via POST /api/ui-config.
    pub ui_settings: Arc<RwLock<config::UiSettings>>,
    /// Session tokens for vault-asset image serving (see `POST /api/auth/session`).
    pub tokens: TokenStore,
    pub oidc_flows: OidcFlowStore,
    pub web_sessions: WebSessionStore,
    /// Tantivy full-text search index.  `None` if the index failed to initialize.
    pub search_index: Option<Arc<search_index::SearchIndex>>,
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

    // Build or open the Tantivy search index.
    let search_index =
        match search_index::SearchIndex::build_or_open(std::path::Path::new(&config.vault.path)) {
            Ok(idx) => {
                let idx = Arc::new(idx);
                let idx2 = idx.clone();
                let vault_ref = vault::Vault::new(&config.vault.path, config.ui.show_hidden_files);
                // Index all files in a background task so startup is non-blocking.
                tokio::spawn(async move {
                    match idx2.index_all(&vault_ref) {
                        Ok(n) => tracing::info!("Search index ready — indexed {n} files"),
                        Err(e) => tracing::warn!("Search index build error: {e}"),
                    }
                });
                Some(idx)
            }
            Err(e) => {
                tracing::warn!("Search index unavailable: {e}. Falling back to linear scan.");
                None
            }
        };

    let state = Arc::new(AppState {
        config,
        vault,
        ui_settings: Arc::new(RwLock::new(ui_settings)),
        tokens,
        oidc_flows: Arc::new(Mutex::new(HashMap::new())),
        web_sessions: Arc::new(Mutex::new(HashMap::new())),
        search_index,
    });
    let app = routes::build_router(state.clone());

    let bind_addr = format!("{}:{}", state.config.server.host, state.config.server.port);
    info!("#ash listening on http://{}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
