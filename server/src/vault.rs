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
            let checksum = checksum_file(path)?;

            entries.push(FileEntry {
                path: rel,
                checksum,
                modified,
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
        let joined = self.root.join(rel_path);
        let canonical_root = self.root.canonicalize()?;
        // Resolve without requiring the file to exist yet
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
