use serde::Serialize;

/// Tauri command: return the current sync status for display in the UI.
#[tauri::command]
pub async fn get_sync_status() -> Result<SyncStatus, String> {
    // TODO M3: check connectivity, return last sync time, pending changes count
    Ok(SyncStatus {
        connected: false,
        last_synced: None,
        pending_changes: 0,
    })
}

/// Tauri command: manually trigger an immediate sync.
#[tauri::command]
pub async fn trigger_sync() -> Result<SyncResult, String> {
    // TODO M3: implement delta sync against server snapshot
    Ok(SyncResult {
        pushed: 0,
        pulled: 0,
        conflicts: 0,
        errors: vec![],
    })
}

#[derive(Serialize)]
pub struct SyncStatus {
    pub connected: bool,
    pub last_synced: Option<i64>,
    pub pending_changes: u32,
}

#[derive(Serialize)]
pub struct SyncResult {
    pub pushed: u32,
    pub pulled: u32,
    pub conflicts: u32,
    pub errors: Vec<String>,
}
