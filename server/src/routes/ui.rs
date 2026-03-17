use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// GET /api/ui-config — public endpoint returning UI theming settings.
///
/// Reads from the runtime-mutable `ui_settings` (which merges config.toml defaults
/// with any overrides saved via POST /api/ui-config).
pub async fn get_ui_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let s = state.ui_settings.read().unwrap();
    Json(UiConfigResponse {
        secondary_color: s.secondary_color.clone(),
        default_theme: s.default_theme.clone(),
        show_hidden_files: state.config.ui.show_hidden_files,
        line_numbers: s.line_numbers,
        spell_check: s.spell_check,
        poll_interval_secs: s.poll_interval_secs,
        large_file_threshold_kb: s.large_file_threshold_kb,
    })
}

/// POST /api/ui-config — update mutable UI settings (protected route).
///
/// All fields are optional; omitted fields keep their current values.
/// Changes are persisted to `.mdkb/ui-settings.toml` in the vault.
pub async fn post_ui_config(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UiConfigPatch>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut s = state.ui_settings.write().unwrap();

    if let Some(v) = payload.secondary_color {
        s.secondary_color = v
    }
    if let Some(v) = payload.default_theme {
        s.default_theme = v
    }
    if let Some(v) = payload.line_numbers {
        s.line_numbers = v
    }
    if let Some(v) = payload.spell_check {
        s.spell_check = v
    }
    if let Some(v) = payload.poll_interval_secs {
        s.poll_interval_secs = v.max(1) // minimum 1 second
    }
    if let Some(v) = payload.large_file_threshold_kb {
        s.large_file_threshold_kb = v
    }

    let updated = s.clone();
    drop(s); // release write lock before doing I/O

    if let Err(e) = updated.save_to_vault(&state.vault) {
        tracing::warn!("Failed to persist ui-settings: {e}");
    }

    Ok(Json(UiConfigResponse {
        secondary_color: updated.secondary_color,
        default_theme: updated.default_theme,
        show_hidden_files: state.config.ui.show_hidden_files,
        line_numbers: updated.line_numbers,
        spell_check: updated.spell_check,
        poll_interval_secs: updated.poll_interval_secs,
        large_file_threshold_kb: updated.large_file_threshold_kb,
    }))
}

#[derive(Serialize)]
pub struct UiConfigResponse {
    pub secondary_color: String,
    pub default_theme: String,
    pub show_hidden_files: bool,
    pub line_numbers: bool,
    pub spell_check: bool,
    pub poll_interval_secs: u32,
    pub large_file_threshold_kb: u32,
}

#[derive(Deserialize)]
pub struct UiConfigPatch {
    pub secondary_color: Option<String>,
    pub default_theme: Option<String>,
    pub line_numbers: Option<bool>,
    pub spell_check: Option<bool>,
    pub poll_interval_secs: Option<u32>,
    pub large_file_threshold_kb: Option<u32>,
}
