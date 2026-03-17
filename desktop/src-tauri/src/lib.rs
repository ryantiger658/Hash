mod config;
mod sync;

use std::sync::{Arc, Mutex};

/// Entry point called from main.rs.
pub fn run() {
    let sync_state = Arc::new(Mutex::new(sync::SyncState::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(sync_state)
        .setup(|app| {
            // Spawn the background sync loop as a detached tokio task.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                sync::sync_loop(handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sync::get_sync_status,
            sync::trigger_sync,
            sync::get_config,
            sync::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running #ash desktop app");
}
