//! Non-destructive validation for Open Knowledge Format (OKF) v0.1 bundles.
//!
//! #ash keeps its general-purpose Markdown model. This module reports the
//! minimum OKF conformance gaps without rewriting or rejecting existing notes.

use serde::Serialize;

use crate::vault::{Vault, DEFAULT_LARGE_FILE_THRESHOLD};

#[derive(Debug, Serialize)]
pub struct OkfIssue {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct OkfValidation {
    pub valid: bool,
    pub concepts_checked: usize,
    pub reserved_files: usize,
    pub issues: Vec<OkfIssue>,
}

/// Validate the OKF requirements that apply to a #ash vault.
/// `index.md` and `log.md` are reserved at every directory level; all other
/// Markdown files are concepts and need a leading frontmatter block with type.
pub fn validate(vault: &Vault) -> OkfValidation {
    let mut concepts_checked = 0;
    let mut reserved_files = 0;
    let mut issues = Vec::new();

    let Ok(files) = vault.list_files(DEFAULT_LARGE_FILE_THRESHOLD) else {
        return OkfValidation {
            valid: false,
            concepts_checked,
            reserved_files,
            issues: vec![OkfIssue {
                path: String::new(),
                message: "Could not list vault files".into(),
            }],
        };
    };

    for file in files
        .into_iter()
        .filter(|file| !file.is_dir && file.path.ends_with(".md"))
    {
        let name = file.path.rsplit('/').next().unwrap_or(&file.path);
        if matches!(name, "index.md" | "log.md") {
            reserved_files += 1;
            continue;
        }
        concepts_checked += 1;

        let Ok(bytes) = vault.read_file(&file.path) else {
            issues.push(OkfIssue {
                path: file.path,
                message: "Could not read note".into(),
            });
            continue;
        };
        let Ok(content) = std::str::from_utf8(&bytes) else {
            issues.push(OkfIssue {
                path: file.path,
                message: "Note is not valid UTF-8".into(),
            });
            continue;
        };

        match okf_type(content) {
            Some(_) => {}
            None => issues.push(OkfIssue {
                path: file.path,
                message:
                    "OKF concepts require leading YAML frontmatter with a non-empty `type` field"
                        .into(),
            }),
        }
    }

    OkfValidation {
        valid: issues.is_empty(),
        concepts_checked,
        reserved_files,
        issues,
    }
}

fn okf_type(content: &str) -> Option<&str> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }
    let mut found = None;
    for line in lines {
        if line == "---" {
            return found;
        }
        if let Some(value) = line.strip_prefix("type:") {
            let value = value.trim().trim_matches('"').trim_matches('\'').trim();
            if !value.is_empty() {
                found = Some(value);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::okf_type;

    #[test]
    fn type_must_be_in_leading_frontmatter() {
        assert_eq!(
            okf_type("---\ntype: Playbook\n---\n# Runbook"),
            Some("Playbook")
        );
        assert_eq!(okf_type("# Note\ntype: Playbook"), None);
        assert_eq!(okf_type("---\ntitle: Untyped\n---\n"), None);
    }
}
