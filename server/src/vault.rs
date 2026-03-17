use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents the vault — the root directory of markdown files.
pub struct Vault {
    pub root: PathBuf,
    /// When false (default), entries whose name starts with '.' are hidden.
    /// The `.mdkb` metadata directory is always hidden regardless of this flag.
    pub show_hidden: bool,
}

/// Metadata about a single file (or directory) in the vault.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    /// Path relative to vault root, using forward slashes.
    pub path: String,
    /// SHA-256 checksum of the file contents. Empty string for directories.
    pub checksum: String,
    /// Last modified timestamp (Unix seconds). 0 for directories.
    pub modified: i64,
    /// Creation timestamp (Unix seconds). None on platforms that don't expose it.
    pub created: Option<i64>,
    /// File size in bytes. 0 for directories.
    pub size: u64,
    /// True when this entry represents a directory rather than a file.
    #[serde(rename = "isDir", default, skip_serializing_if = "std::ops::Not::not")]
    pub is_dir: bool,
}

impl Vault {
    pub fn new(root: &str, show_hidden: bool) -> Self {
        Self {
            root: PathBuf::from(root),
            show_hidden,
        }
    }

    /// List all markdown files and attachments in the vault.
    ///
    /// `large_file_threshold` controls the checksum strategy:
    /// - Files **smaller** than the threshold get a full SHA-256 hex string.
    /// - Files **at or above** the threshold get `"mtime:<secs>-size:<bytes>"` to
    ///   avoid reading large attachments on every poll.
    ///
    /// Pass `DEFAULT_LARGE_FILE_THRESHOLD` for the default behaviour.
    pub fn list_files(&self, large_file_threshold: u64) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();

        // Capture flag so the closure doesn't borrow `self`.
        let show_hidden = self.show_hidden;

        for entry in WalkDir::new(&self.root)
            // Keep follow_links(false) so entry.path() is always the path of the
            // entry within the vault tree (symlink path, not target path).
            // Symlinks are handled explicitly below via std::fs::metadata, which
            // follows symlinks for metadata and content.
            .follow_links(false)
            .into_iter()
            // filter_entry prunes entire subtrees so hidden dirs aren't descended into.
            .filter_entry(|e| {
                if e.depth() == 0 {
                    return true;
                } // always allow vault root
                let name = e.file_name().to_str().unwrap_or("");
                // Always prune .mdkb; prune other dotfiles when show_hidden is false.
                name != ".mdkb" && (show_hidden || !name.starts_with('.'))
            })
            .filter_map(|e| e.ok())
            // Skip vault root itself (depth 0)
            .filter(|e| e.depth() > 0)
        {
            let path = entry.path();
            let rel = match self.relative_path(path) {
                Ok(r) => r,
                Err(_) => continue,
            };

            if entry.file_type().is_dir() {
                entries.push(FileEntry {
                    path: rel,
                    checksum: String::new(),
                    modified: 0,
                    created: None,
                    size: 0,
                    is_dir: true,
                });
            } else {
                // Handles both regular files (is_file) and symlinks to files (is_symlink).
                // std::fs::metadata follows symlinks, so we get the target's metadata.
                // For symlinks to directories or broken symlinks, metadata.is_file()
                // will be false and we skip them.
                let meta = match std::fs::metadata(path) {
                    Ok(m) if m.is_file() => m,
                    _ => continue,
                };
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let created = meta
                    .created()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64);
                let checksum = match checksum_file(path, &meta, large_file_threshold) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                entries.push(FileEntry {
                    path: rel,
                    checksum,
                    modified,
                    created,
                    size: meta.len(),
                    is_dir: false,
                });
            }
        }

        Ok(entries)
    }

    /// Read a file's contents by vault-relative path.
    pub fn read_file(&self, rel_path: &str) -> Result<Vec<u8>> {
        let full = self.full_path(rel_path)?;
        Ok(std::fs::read(full)?)
    }

    /// Write a file's contents by vault-relative path, creating parent dirs as needed.
    pub fn write_file(&self, rel_path: &str, content: &[u8]) -> Result<()> {
        let full = self.full_path(rel_path)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::write(full, content)?)
    }

    /// Read the vault schema version from `.mdkb/vault.toml`.
    /// Returns `0` if the file is absent or unparseable (pre-v0.0.3 vault).
    pub fn read_schema_version(&self) -> Result<u32> {
        match self.read_file(".mdkb/vault.toml") {
            Ok(bytes) => {
                let content = std::str::from_utf8(&bytes).unwrap_or("");
                for line in content.lines() {
                    if let Some(rest) = line.trim().strip_prefix("schema_version") {
                        if let Some(val) = rest.trim().strip_prefix('=') {
                            if let Ok(v) = val.trim().parse::<u32>() {
                                return Ok(v);
                            }
                        }
                    }
                }
                Ok(0)
            }
            Err(_) => Ok(0),
        }
    }

    /// Write the vault schema version to `.mdkb/vault.toml`.
    pub fn write_schema_version(&self, version: u32) -> Result<()> {
        let content =
            format!("# #ash vault metadata — do not edit manually\nschema_version = {version}\n");
        self.write_file(".mdkb/vault.toml", content.as_bytes())
    }

    /// Delete a file by vault-relative path.
    pub fn delete_file(&self, rel_path: &str) -> Result<()> {
        let full = self.full_path(rel_path)?;
        Ok(std::fs::remove_file(full)?)
    }

    /// Move/rename a file or directory within the vault.
    /// Both `from` and `to` are vault-relative paths.
    pub fn rename(&self, from: &str, to: &str) -> Result<()> {
        let full_from = self.full_path(from)?;
        let full_to = self.full_path(to)?;
        if let Some(parent) = full_to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::rename(full_from, full_to)?)
    }

    /// Recursively delete a directory and all its contents by vault-relative path.
    pub fn delete_dir(&self, rel_path: &str) -> Result<()> {
        let full = self.full_path(rel_path)?;
        anyhow::ensure!(full.is_dir(), "Not a directory: {rel_path}");
        Ok(std::fs::remove_dir_all(full)?)
    }

    /// Resolve a vault-relative path to a full filesystem path.
    /// Rejects any path that escapes the vault root (path traversal protection).
    pub fn full_path(&self, rel_path: &str) -> Result<PathBuf> {
        let canonical_root = self.root.canonicalize()?;
        // Join against the canonical root so symlinks are resolved before normalization
        let joined = canonical_root.join(rel_path);
        let resolved = normalize_path(&joined);
        if !resolved.starts_with(&canonical_root) {
            anyhow::bail!("Path traversal attempt: {rel_path}");
        }
        Ok(resolved)
    }

    /// Return the checksum and last-modified timestamp (Unix seconds) for a single
    /// vault file. Uses the same threshold logic as `list_files`.
    /// Intended for the frequent open-file poll endpoint.
    pub fn file_checksum(
        &self,
        rel_path: &str,
        large_file_threshold: u64,
    ) -> Result<(String, i64)> {
        let full = self.full_path(rel_path)?;
        let meta = std::fs::metadata(&full)?;
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let checksum = checksum_file(&full, &meta, large_file_threshold)?;
        Ok((checksum, modified))
    }

    /// Convert an absolute path to a vault-relative string with forward slashes.
    fn relative_path(&self, abs: &Path) -> Result<String> {
        let rel = abs
            .strip_prefix(&self.root)
            .map_err(|_| anyhow::anyhow!("Path is not inside vault"))?;
        Ok(rel.to_string_lossy().replace('\\', "/"))
    }
}

/// Default large-file threshold (512 KiB). Files at or above this size use
/// mtime+size instead of SHA-256 during vault listing. Users can adjust this
/// via the settings panel (stored in `.mdkb/ui-settings.toml`).
pub const DEFAULT_LARGE_FILE_THRESHOLD: u64 = 512 * 1024;

/// Return a change-detection string for a file.
/// - Small files  (< threshold): full SHA-256 hex string.
/// - Large files (>= threshold): `"mtime:<secs>-size:<bytes>"` — reads no
///   file data, so poll overhead for large attachments is near-zero.
fn checksum_file(path: &Path, meta: &std::fs::Metadata, threshold: u64) -> Result<String> {
    if meta.len() < threshold {
        let bytes = std::fs::read(path)?;
        let hash = Sha256::digest(&bytes);
        Ok(hex::encode(hash))
    } else {
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Ok(format!("mtime:{}-size:{}", mtime, meta.len()))
    }
}

/// Normalize a path without requiring it to exist on disk (no canonicalize).
fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (Vault, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::new(dir.path().to_str().unwrap(), true);
        (vault, dir)
    }

    #[test]
    fn write_and_read_roundtrip() {
        let (vault, _dir) = setup();
        vault.write_file("note.md", b"# Hello").unwrap();
        let bytes = vault.read_file("note.md").unwrap();
        assert_eq!(bytes, b"# Hello");
    }

    #[test]
    fn write_creates_parent_directories() {
        let (vault, _dir) = setup();
        vault.write_file("a/b/c/note.md", b"nested").unwrap();
        let bytes = vault.read_file("a/b/c/note.md").unwrap();
        assert_eq!(bytes, b"nested");
    }

    #[test]
    fn list_files_returns_written_files() {
        let (vault, _dir) = setup();
        vault.write_file("one.md", b"one").unwrap();
        vault.write_file("two.md", b"two").unwrap();
        vault.write_file("sub/three.md", b"three").unwrap();
        let files = vault.list_files(DEFAULT_LARGE_FILE_THRESHOLD).unwrap();
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"one.md"));
        assert!(paths.contains(&"two.md"));
        assert!(paths.contains(&"sub/three.md"));
    }

    #[test]
    fn list_files_skips_mdkb_metadata() {
        let (vault, _dir) = setup();
        vault.write_file("note.md", b"content").unwrap();
        vault
            .write_file(".mdkb/sync/note.toml", b"metadata")
            .unwrap();
        let files = vault.list_files(DEFAULT_LARGE_FILE_THRESHOLD).unwrap();
        assert!(files.iter().all(|f| !f.path.starts_with(".mdkb")));
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn delete_removes_file() {
        let (vault, _dir) = setup();
        vault.write_file("temp.md", b"delete me").unwrap();
        assert!(vault.read_file("temp.md").is_ok());
        vault.delete_file("temp.md").unwrap();
        assert!(vault.read_file("temp.md").is_err());
    }

    #[test]
    fn delete_nonexistent_returns_error() {
        let (vault, _dir) = setup();
        assert!(vault.delete_file("ghost.md").is_err());
    }

    #[test]
    fn read_nonexistent_returns_error() {
        let (vault, _dir) = setup();
        assert!(vault.read_file("missing.md").is_err());
    }

    #[test]
    fn file_entry_includes_checksum_and_size() {
        let (vault, _dir) = setup();
        vault.write_file("sized.md", b"hello world").unwrap();
        let files = vault.list_files(DEFAULT_LARGE_FILE_THRESHOLD).unwrap();
        let entry = files.iter().find(|f| f.path == "sized.md").unwrap();
        assert_eq!(entry.size, 11);
        assert!(!entry.checksum.is_empty());
        assert_eq!(entry.checksum.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn checksum_is_deterministic() {
        let (vault, _dir) = setup();
        vault.write_file("det.md", b"same content").unwrap();
        let f1 = vault.list_files(DEFAULT_LARGE_FILE_THRESHOLD).unwrap();
        let c1 = &f1[0].checksum;
        // Overwrite with identical content
        vault.write_file("det.md", b"same content").unwrap();
        let f2 = vault.list_files(DEFAULT_LARGE_FILE_THRESHOLD).unwrap();
        let c2 = &f2[0].checksum;
        assert_eq!(c1, c2);
    }

    #[test]
    fn checksum_differs_for_different_content() {
        let (vault, _dir) = setup();
        vault.write_file("change.md", b"version 1").unwrap();
        let c1 = vault.list_files(DEFAULT_LARGE_FILE_THRESHOLD).unwrap()[0]
            .checksum
            .clone();
        vault.write_file("change.md", b"version 2").unwrap();
        let c2 = vault.list_files(DEFAULT_LARGE_FILE_THRESHOLD).unwrap()[0]
            .checksum
            .clone();
        assert_ne!(c1, c2);
    }

    #[test]
    fn list_files_hides_dotfiles_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::new(dir.path().to_str().unwrap(), false);
        vault.write_file("visible.md", b"hello").unwrap();
        vault.write_file(".hidden.md", b"secret").unwrap();
        vault.write_file(".attachments/image.png", b"img").unwrap();
        let files = vault.list_files(DEFAULT_LARGE_FILE_THRESHOLD).unwrap();
        assert!(files.iter().any(|f| f.path == "visible.md"));
        assert!(files.iter().all(|f| !f.path.starts_with('.')));
    }

    #[test]
    fn list_files_shows_dotfiles_when_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::new(dir.path().to_str().unwrap(), true);
        vault.write_file("visible.md", b"hello").unwrap();
        vault.write_file(".hidden.md", b"secret").unwrap();
        let files = vault.list_files(DEFAULT_LARGE_FILE_THRESHOLD).unwrap();
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"visible.md"));
        assert!(paths.contains(&".hidden.md"));
    }

    #[test]
    fn path_traversal_single_dotdot_blocked() {
        let (vault, _dir) = setup();
        let result = vault.full_path("../escape.txt");
        assert!(result.is_err(), "path traversal should be rejected");
    }

    #[test]
    fn path_traversal_deep_blocked() {
        let (vault, _dir) = setup();
        let result = vault.full_path("sub/../../etc/passwd");
        assert!(result.is_err(), "deep path traversal should be rejected");
    }

    #[test]
    fn valid_nested_path_allowed() {
        let (vault, _dir) = setup();
        // Writing the file first ensures the path resolves correctly
        vault.write_file("a/b.md", b"ok").unwrap();
        assert!(vault.full_path("a/b.md").is_ok());
    }
}
