//! Vault schema migrations.
//!
//! On every server startup, `run()` checks the vault's recorded schema version
//! against `CURRENT_SCHEMA_VERSION`. If the vault is behind, each migration
//! function is applied in order and the recorded version is updated.
//!
//! # Adding a new migration
//!
//! 1. Increment `CURRENT_SCHEMA_VERSION`.
//! 2. Add a `migrate_vN(vault: &Vault) -> Result<()>` function below.
//! 3. Add a `N => migrate_vN(vault)?` arm to the `match` in `run()`.
//!
//! Migrations MUST be idempotent — they may be re-run if the server crashes
//! after applying a migration but before the version file is updated.
//!
//! Migrations MUST preserve note content. Transformations to note files must
//! produce output that older clients can still read (forwards-compatible writes)
//! or must bump the schema version high enough that older clients refuse to open
//! the vault (incompatible writes — avoid these).

use crate::vault::Vault;
use anyhow::Result;
use tracing::{info, warn};

/// The schema version this build of the server understands.
/// Bump this when adding a new migration.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Apply any pending migrations to the vault, then return.
/// Errors are logged as warnings — a failed migration never prevents startup,
/// but the vault version will not be advanced past the point of failure.
pub fn run(vault: &Vault) -> Result<()> {
    let recorded = vault.read_schema_version().unwrap_or_else(|e| {
        warn!("Could not read vault schema version: {e} — assuming v0");
        0
    });

    if recorded >= CURRENT_SCHEMA_VERSION {
        return Ok(());
    }

    info!(
        "Vault schema v{recorded} → v{CURRENT_SCHEMA_VERSION}: running {} migration(s)",
        CURRENT_SCHEMA_VERSION - recorded
    );

    for target in (recorded + 1)..=CURRENT_SCHEMA_VERSION {
        info!("Applying vault migration to schema v{target}…");
        let result = match target {
            1 => migrate_v1(vault),
            _ => Ok(()), // future migrations added here
        };
        match result {
            Ok(()) => {
                vault.write_schema_version(target)?;
                info!("Migration to schema v{target} complete.");
            }
            Err(e) => {
                warn!("Migration to schema v{target} failed: {e}");
                return Err(e);
            }
        }
    }

    Ok(())
}

// ── Migration functions ────────────────────────────────────────────────────

/// v1 — initial schema record.
///
/// Nothing to transform; this migration simply stamps the version file for
/// vaults that pre-date schema tracking (all vaults created before v0.0.3).
fn migrate_v1(_vault: &Vault) -> Result<()> {
    // No structural changes — plain markdown files are always v1-compatible.
    Ok(())
}
