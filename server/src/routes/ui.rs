use crate::AppState;
use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use std::sync::Arc;

/// GET /api/ui-config — public endpoint returning UI theming settings.
///
/// Called by the frontend on load (before auth) to apply the accent color
/// and default theme without a round-trip after login.
pub async fn get_ui_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(UiConfigResponse {
        secondary_color: state.config.ui.secondary_color.clone(),
        default_theme: state.config.ui.default_theme.clone(),
        editor_labels: state.config.ui.editor_labels,
        show_hidden_files: state.config.ui.show_hidden_files,
        line_numbers: state.config.ui.line_numbers,
        spell_check: state.config.ui.spell_check,
    })
}

#[derive(Serialize)]
pub struct UiConfigResponse {
    pub secondary_color: String,
    pub default_theme: String,
    pub editor_labels: bool,
    pub show_hidden_files: bool,
    pub line_numbers: bool,
    pub spell_check: bool,
}
