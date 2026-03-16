use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents the vault — the root directory of markdown files.
pub struct Vault {
    pub root: PathBuf,
}

/// Metadata about a single file in the vault.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    /// Path relative to vault root, using forward slashes.
    pub path: String,
    /// SHA-256 checksum of the file contents.
    pub checksum: String,
    /// Last modified timestamp (Unix seconds).
    pub modified: i64,
    /// Creation timestamp (Unix seconds). None on platforms that don't expose it.
    pub created: Option<i64>,
    /// File size in bytes.
    pub size: u64,
}

impl Vault {
    pub fn new(root: &str) -> Self {
        Self {
            root: PathBuf::from(root),
        }
    }

    /// List all markdown files and attachments in the vault.
    pub fn list_files(&self) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();

        for entry in WalkDir::new(&self.root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            // Skip the .mdkb metadata directory
            .filter(|e| !e.path().components().any(|c| c.as_os_str() == ".mdkb"))
        {
            let path = entry.path();
            let rel = self.relative_path(path)?;
            let meta = std::fs::metadata(path)?;
            let modified = meta
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64;
            let created = meta.created()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);
            let checksum = checksum_file(path)?;

            entries.push(FileEntry {
                path: rel,
                checksum,
                modified,
                created,
                size: meta.len(),
            });
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

    /// Delete a file by vault-relative path.
    pub fn delete_file(&self, rel_path: &str) -> Result<()> {
        let full = self.full_path(rel_path)?;
        Ok(std::fs::remove_file(full)?)
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

    /// Convert an absolute path to a vault-relative string with forward slashes.
    fn relative_path(&self, abs: &Path) -> Result<String> {
        let rel = abs
            .strip_prefix(&self.root)
            .map_err(|_| anyhow::anyhow!("Path is not inside vault"))?;
        Ok(rel.to_string_lossy().replace('\\', "/"))
    }
}

/// SHA-256 checksum of a file's contents, returned as a hex string.
fn checksum_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    let hash = Sha256::digest(&bytes);
    Ok(hex::encode(hash))
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
        let vault = Vault::new(dir.path().to_str().unwrap());
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
        let files = vault.list_files().unwrap();
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"one.md"));
        assert!(paths.contains(&"two.md"));
        assert!(paths.contains(&"sub/three.md"));
    }

    #[test]
    fn list_files_skips_mdkb_metadata() {
        let (vault, _dir) = setup();
        vault.write_file("note.md", b"content").unwrap();
        vault.write_file(".mdkb/sync/note.toml", b"metadata").unwrap();
        let files = vault.list_files().unwrap();
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
        let files = vault.list_files().unwrap();
        let entry = files.iter().find(|f| f.path == "sized.md").unwrap();
        assert_eq!(entry.size, 11);
        assert!(!entry.checksum.is_empty());
        assert_eq!(entry.checksum.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn checksum_is_deterministic() {
        let (vault, _dir) = setup();
        vault.write_file("det.md", b"same content").unwrap();
        let f1 = vault.list_files().unwrap();
        let c1 = &f1[0].checksum;
        // Overwrite with identical content
        vault.write_file("det.md", b"same content").unwrap();
        let f2 = vault.list_files().unwrap();
        let c2 = &f2[0].checksum;
        assert_eq!(c1, c2);
    }

    #[test]
    fn checksum_differs_for_different_content() {
        let (vault, _dir) = setup();
        vault.write_file("change.md", b"version 1").unwrap();
        let c1 = vault.list_files().unwrap()[0].checksum.clone();
        vault.write_file("change.md", b"version 2").unwrap();
        let c2 = vault.list_files().unwrap()[0].checksum.clone();
        assert_ne!(c1, c2);
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
