mod config;
mod sync;

/// Entry point called from main.rs.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            sync::get_sync_status,
            sync::trigger_sync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running #ash desktop app");
}
