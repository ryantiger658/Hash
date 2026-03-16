use anyhow::Result;
use std::sync::Arc;
use tracing::info;

mod config;
mod routes;
mod sync;
mod vault;

pub use config::Config;

/// Shared application state passed to all route handlers.
pub struct AppState {
    pub config: Config,
    pub vault: vault::Vault,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging. Set RUST_LOG=debug for verbose output.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "notch_server=info,tower_http=info".into()),
        )
        .init();

    // Load configuration from config.toml (or NOTCH_CONFIG env var path).
    let config = Config::load()?;
    info!("Vault path: {}", config.vault.path);

    let vault = vault::Vault::new(&config.vault.path);
    let state = Arc::new(AppState { config, vault });

    let app = routes::build_router(state.clone());

    let bind_addr = format!("{}:{}", state.config.server.host, state.config.server.port);
    info!("#ash listening on http://{}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
