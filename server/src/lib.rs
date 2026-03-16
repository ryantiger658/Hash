pub mod config;
pub mod migrations;
pub mod routes;
pub mod sync;
pub mod vault;

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

/// Shared application state passed to all Axum route handlers.
pub struct AppState {
    pub config: config::Config,
    pub vault: vault::Vault,
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

    let state = Arc::new(AppState { config, vault });
    let app = routes::build_router(state.clone());

    let bind_addr = format!("{}:{}", state.config.server.host, state.config.server.port);
    info!("#ash listening on http://{}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
