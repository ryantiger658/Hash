use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use tauri::Manager;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

// ── Shared state ──────────────────────────────────────────────────────────────

/// Sync state shared between the background loop and Tauri commands.
#[derive(Default)]
pub struct SyncState {
    /// True once a valid config file has been found at least once.
    pub configured: bool,
    pub connected: bool,
    pub last_synced: Option<i64>,
    pub pending_changes: u32,
    pub last_error: Option<String>,
}

// ── Tauri-facing types ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SyncStatus {
    pub configured: bool,
    pub connected: bool,
    pub last_synced: Option<i64>,
    pub pending_changes: u32,
}

#[derive(Serialize, Debug, Default)]
pub struct SyncResult {
    pub pushed: u32,
    pub pulled: u32,
    pub conflicts: u32,
    pub errors: Vec<String>,
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Return the current sync status for the UI status indicator.
#[tauri::command]
pub async fn get_sync_status(
    state: tauri::State<'_, Arc<Mutex<SyncState>>>,
) -> Result<SyncStatus, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(SyncStatus {
        configured: s.configured,
        connected: s.connected,
        last_synced: s.last_synced,
        pending_changes: s.pending_changes,
    })
}

/// Manually trigger an immediate sync cycle.
#[tauri::command]
pub async fn trigger_sync(
    state: tauri::State<'_, Arc<Mutex<SyncState>>>,
) -> Result<SyncResult, String> {
    let config = crate::config::ClientConfig::load().map_err(|e| e.to_string())?;
    let state_arc = state.inner().clone();
    Ok(run_sync(&config, &state_arc).await)
}

/// Return the current client config (or null if not yet configured).
#[tauri::command]
pub fn get_config() -> Option<crate::config::ClientConfig> {
    crate::config::ClientConfig::load().ok()
}

/// Save a new client config and immediately trigger a sync.
#[tauri::command]
pub async fn save_config(
    config: crate::config::ClientConfig,
    state: tauri::State<'_, Arc<Mutex<SyncState>>>,
) -> Result<SyncResult, String> {
    config.save().map_err(|e| e.to_string())?;
    // Mark as configured right away so UI updates
    if let Ok(mut s) = state.lock() {
        s.configured = true;
    }
    let state_arc = state.inner().clone();
    Ok(run_sync(&config, &state_arc).await)
}

// ── Background loop ───────────────────────────────────────────────────────────

/// Spawn in Tauri's setup hook — loops forever, syncing on the configured interval.
pub async fn sync_loop(app_handle: tauri::AppHandle) {
    loop {
        let config = match crate::config::ClientConfig::load() {
            Ok(c) => {
                // Mark as configured so the UI shows the right dot state
                let state = app_handle.state::<Arc<Mutex<SyncState>>>();
                if let Ok(mut s) = state.lock() {
                    s.configured = true;
                }
                c
            }
            Err(e) => {
                tracing::warn!("Sync loop: config not found — {e}. Retrying in 30 s.");
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                continue;
            }
        };

        let interval = config.sync_interval_secs.max(5);
        let state = app_handle.state::<Arc<Mutex<SyncState>>>();
        let result = run_sync(&config, state.inner()).await;

        if !result.errors.is_empty() {
            tracing::warn!("Sync errors: {:?}", result.errors);
        } else {
            tracing::info!(
                "Sync complete — pushed {}, pulled {}, conflicts {}",
                result.pushed,
                result.pulled,
                result.conflicts
            );
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

// ── Core sync engine ──────────────────────────────────────────────────────────

/// Run one full sync cycle: diff local vault against server snapshot, then
/// push/pull as needed.  Never holds the state lock across await points.
pub async fn run_sync(
    config: &crate::config::ClientConfig,
    state: &Arc<Mutex<SyncState>>,
) -> SyncResult {
    let vault_path = Path::new(&config.local_vault_path);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    // ── 1. Fetch server snapshot ──────────────────────────────────────────────
    let snapshot = match fetch_snapshot(&client, config).await {
        Ok(s) => {
            set_connected(state, true);
            s
        }
        Err(e) => {
            set_connected(state, false);
            set_error(state, &e);
            return SyncResult {
                errors: vec![e],
                ..Default::default()
            };
        }
    };

    // ── 2. Scan local vault ───────────────────────────────────────────────────
    let local_map = scan_local_vault(vault_path);

    // ── 3. Compute pending_changes for status display ─────────────────────────
    let pending = count_pending(&local_map, vault_path);
    {
        if let Ok(mut s) = state.lock() {
            s.pending_changes = pending;
        }
    }

    // ── 4. Build server file map (dirs excluded) ──────────────────────────────
    let server_map: HashMap<String, &ServerFileEntry> = snapshot
        .files
        .iter()
        .filter(|f| !f.is_dir)
        .map(|f| (f.path.clone(), f))
        .collect();

    // ── 5. Diff ───────────────────────────────────────────────────────────────
    let all_paths: HashSet<&str> = server_map
        .keys()
        .chain(local_map.keys())
        .map(String::as_str)
        .collect();

    let mut to_push: Vec<String> = Vec::new();
    let mut to_pull: Vec<String> = Vec::new();
    let mut to_delete_local: Vec<String> = Vec::new();
    let mut to_delete_server: Vec<String> = Vec::new();

    for path in &all_paths {
        let local = local_map.get(*path);
        let server = server_map.get(*path);
        let meta = read_sync_meta(vault_path, path);

        match (local, server) {
            // File exists locally but not on server
            (Some(_), None) => {
                if meta.is_some() {
                    // Previously synced → deleted on server → remove locally
                    to_delete_local.push(path.to_string());
                } else {
                    // Never synced → new local file → push to server
                    to_push.push(path.to_string());
                }
            }
            // File exists on server but not locally
            (None, Some(_)) => {
                if meta.is_some() {
                    // Previously synced → deleted locally → propagate to server
                    to_delete_server.push(path.to_string());
                } else {
                    // Never seen → new server file → pull down
                    to_pull.push(path.to_string());
                }
            }
            // File exists on both sides
            (Some(l), Some(s)) => {
                let local_changed = meta
                    .as_ref()
                    .map(|m| l.checksum != m.last_synced_checksum)
                    .unwrap_or(true); // never synced → treat as changed
                let server_changed = meta
                    .as_ref()
                    .map(|m| s.checksum != m.last_synced_checksum)
                    .unwrap_or(true);

                match (local_changed, server_changed) {
                    (false, false) => {} // in sync, nothing to do
                    (true, false) => to_push.push(path.to_string()),
                    (false, true) => to_pull.push(path.to_string()),
                    // Both changed — push with conflict metadata; server decides
                    (true, true) => to_push.push(path.to_string()),
                }
            }
            (None, None) => unreachable!(),
        }
    }

    let mut result = SyncResult::default();

    // ── 6. Pull files from server ─────────────────────────────────────────────
    for path in &to_pull {
        match pull_file(&client, config, path, vault_path).await {
            Ok(checksum) => {
                let now = unix_now();
                write_sync_meta(vault_path, path, &checksum, now);
                result.pulled += 1;
            }
            Err(e) => result.errors.push(format!("pull {path}: {e}")),
        }
    }

    // ── 7. Build push payload ─────────────────────────────────────────────────
    let mut upserts: Vec<PushItem> = Vec::new();
    for path in &to_push {
        let Some(local) = local_map.get(path.as_str()) else {
            continue;
        };
        let bytes = match std::fs::read(&local.full_path) {
            Ok(b) => b,
            Err(e) => {
                result.errors.push(format!("read {path}: {e}"));
                continue;
            }
        };
        let checksum = hex::encode(Sha256::digest(&bytes));
        let content = BASE64.encode(&bytes);
        let meta = read_sync_meta(vault_path, path);

        upserts.push(PushItem {
            path: path.clone(),
            content,
            checksum,
            modified: local.modified,
            last_synced_checksum: meta.as_ref().map(|m| m.last_synced_checksum.clone()).unwrap_or_default(),
            last_synced_timestamp: meta.as_ref().map(|m| m.last_synced_timestamp).unwrap_or(0),
        });
    }

    let deletes: Vec<DeleteItem> = to_delete_server
        .iter()
        .map(|p| DeleteItem { path: p.clone() })
        .collect();

    // ── 8. Execute push ───────────────────────────────────────────────────────
    if !upserts.is_empty() || !deletes.is_empty() {
        match push_changes(&client, config, upserts, deletes).await {
            Ok(push_result) => {
                let now = unix_now();
                for path in &push_result.accepted {
                    if let Some(local) = local_map.get(path.as_str()) {
                        write_sync_meta(vault_path, path, &local.checksum, now);
                    } else {
                        // Accepted delete
                        delete_sync_meta(vault_path, path);
                    }
                    result.pushed += 1;
                }
                for conflict in &push_result.conflicts {
                    result.conflicts += 1;
                    // Log conflict — full resolution UI comes in M4
                    // For now, server keeps its version; client will pull on next cycle
                    tracing::warn!(
                        "Conflict on '{}': server and local both changed. \
                         Server version preserved; will pull on next sync.",
                        conflict.path
                    );
                    result
                        .errors
                        .push(format!("conflict: {}", conflict.path));
                }
                for rejected in &push_result.rejected {
                    result
                        .errors
                        .push(format!("rejected {}: {}", rejected.path, rejected.reason));
                }
            }
            Err(e) => result.errors.push(format!("push: {e}")),
        }
    }

    // ── 9. Delete files deleted on server ─────────────────────────────────────
    for path in &to_delete_local {
        let full = vault_path.join(path);
        if std::fs::remove_file(&full).is_ok() {
            delete_sync_meta(vault_path, path);
        }
    }

    // ── 10. Update last_synced ────────────────────────────────────────────────
    let now = unix_now();
    if let Ok(mut s) = state.lock() {
        s.last_synced = Some(now);
        s.pending_changes = 0;
        s.last_error = result.errors.first().cloned();
    }

    result
}

// ── HTTP helpers ──────────────────────────────────────────────────────────────

async fn fetch_snapshot(
    client: &reqwest::Client,
    config: &crate::config::ClientConfig,
) -> Result<VaultSnapshot, String> {
    let url = format!("{}/api/sync/snapshot", config.server_url);
    client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .send()
        .await
        .map_err(|e| format!("network: {e}"))?
        .error_for_status()
        .map_err(|e| format!("server: {e}"))?
        .json::<VaultSnapshot>()
        .await
        .map_err(|e| format!("parse: {e}"))
}

async fn pull_file(
    client: &reqwest::Client,
    config: &crate::config::ClientConfig,
    path: &str,
    vault_path: &Path,
) -> Result<String, String> {
    let encoded = encode_path(path);
    let url = format!("{}/api/files/{}", config.server_url, encoded);

    let bytes = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    let full_path = vault_path.join(path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&full_path, &bytes).map_err(|e| e.to_string())?;

    Ok(hex::encode(Sha256::digest(&bytes)))
}

async fn push_changes(
    client: &reqwest::Client,
    config: &crate::config::ClientConfig,
    upsert: Vec<PushItem>,
    delete: Vec<DeleteItem>,
) -> Result<PushResult, String> {
    let url = format!("{}/api/sync/push", config.server_url);
    client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&PushPayload { upsert, delete })
        .send()
        .await
        .map_err(|e| format!("network: {e}"))?
        .error_for_status()
        .map_err(|e| format!("server: {e}"))?
        .json::<PushResult>()
        .await
        .map_err(|e| format!("parse: {e}"))
}

// ── Local vault scanning ──────────────────────────────────────────────────────

struct LocalEntry {
    checksum: String,
    modified: i64,
    full_path: PathBuf,
}

fn scan_local_vault(vault_path: &Path) -> HashMap<String, LocalEntry> {
    let mut map = HashMap::new();

    for entry in WalkDir::new(vault_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_str().unwrap_or("");
            // Skip .mdkb (metadata) and other dotfiles
            name != ".mdkb" && !name.starts_with('.')
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.depth() > 0 && e.file_type().is_file())
    {
        let path = entry.path();
        let rel = match path.strip_prefix(vault_path) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };

        let meta = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        map.insert(
            rel,
            LocalEntry {
                checksum: hex::encode(Sha256::digest(&bytes)),
                modified,
                full_path: path.to_path_buf(),
            },
        );
    }

    map
}

/// Count files that differ from their last-synced state (for the status dot).
fn count_pending(local_map: &HashMap<String, LocalEntry>, vault_path: &Path) -> u32 {
    local_map
        .iter()
        .filter(|(path, entry)| {
            read_sync_meta(vault_path, path)
                .map(|m| entry.checksum != m.last_synced_checksum)
                .unwrap_or(true) // never synced → pending
        })
        .count() as u32
}

// ── Local sync metadata ───────────────────────────────────────────────────────

struct SyncMeta {
    last_synced_checksum: String,
    last_synced_timestamp: i64,
}

fn meta_path_for(vault_path: &Path, file_path: &str) -> PathBuf {
    vault_path
        .join(".mdkb")
        .join("sync")
        .join(format!("{file_path}.toml"))
}

fn read_sync_meta(vault_path: &Path, file_path: &str) -> Option<SyncMeta> {
    let text = std::fs::read_to_string(meta_path_for(vault_path, file_path)).ok()?;

    let mut checksum = String::new();
    let mut timestamp: i64 = 0;

    for line in text.lines() {
        if let Some(v) = line.trim().strip_prefix("last_synced_checksum") {
            if let Some(v) = v.trim().strip_prefix('=') {
                checksum = v.trim().trim_matches('"').to_string();
            }
        }
        if let Some(v) = line.trim().strip_prefix("last_synced_timestamp") {
            if let Some(v) = v.trim().strip_prefix('=') {
                timestamp = v.trim().parse().unwrap_or(0);
            }
        }
    }

    if checksum.is_empty() {
        None
    } else {
        Some(SyncMeta {
            last_synced_checksum: checksum,
            last_synced_timestamp: timestamp,
        })
    }
}

fn write_sync_meta(vault_path: &Path, file_path: &str, checksum: &str, timestamp: i64) {
    let path = meta_path_for(vault_path, file_path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = format!(
        "# #ash sync metadata — do not edit manually\n\
         last_synced_checksum = \"{checksum}\"\n\
         last_synced_timestamp = {timestamp}\n"
    );
    let _ = std::fs::write(path, content);
}

fn delete_sync_meta(vault_path: &Path, file_path: &str) {
    let _ = std::fs::remove_file(meta_path_for(vault_path, file_path));
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Percent-encode each path segment so file names with spaces/dots are safe in URLs.
fn encode_path(path: &str) -> String {
    path.split('/')
        .map(urlencoding::encode)
        .collect::<Vec<_>>()
        .join("/")
}

fn set_connected(state: &Arc<Mutex<SyncState>>, v: bool) {
    if let Ok(mut s) = state.lock() {
        s.connected = v;
    }
}

fn set_error(state: &Arc<Mutex<SyncState>>, msg: &str) {
    if let Ok(mut s) = state.lock() {
        s.last_error = Some(msg.to_string());
    }
}

// ── Server API mirror types ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct VaultSnapshot {
    #[allow(dead_code)]
    server_time: i64,
    files: Vec<ServerFileEntry>,
}

#[derive(Deserialize)]
struct ServerFileEntry {
    path: String,
    checksum: String,
    #[allow(dead_code)]
    modified: i64,
    #[serde(rename = "isDir", default)]
    is_dir: bool,
}

#[derive(Serialize)]
struct PushPayload {
    upsert: Vec<PushItem>,
    delete: Vec<DeleteItem>,
}

#[derive(Serialize)]
struct PushItem {
    path: String,
    content: String,
    checksum: String,
    modified: i64,
    last_synced_checksum: String,
    last_synced_timestamp: i64,
}

#[derive(Serialize)]
struct DeleteItem {
    path: String,
}

#[derive(Deserialize)]
struct PushResult {
    accepted: Vec<String>,
    #[serde(default)]
    rejected: Vec<RejectedItem>,
    #[serde(default)]
    conflicts: Vec<ConflictItem>,
}

#[derive(Deserialize)]
struct RejectedItem {
    path: String,
    reason: String,
}

#[derive(Deserialize)]
struct ConflictItem {
    path: String,
    #[allow(dead_code)]
    server_checksum: String,
    #[allow(dead_code)]
    server_modified: i64,
    #[allow(dead_code)]
    server_content: String,
}
