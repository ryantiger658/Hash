//! Tantivy-backed full-text search index for the vault.
//!
//! The index lives at `<vault_root>/.mdkb/search-index/` and is persisted to
//! disk so cold starts are instant.  Incremental updates happen after every
//! successful file write/delete so the index is always up-to-date.
//!
//! Schema
//! ──────
//! • `path`  — STRING | STORED — exact vault-relative path (e.g. "notes/rust.md")
//! • `title` — TEXT   | STORED — first `# Heading` or filename stem; boosted 3×
//! • `tags`  — TEXT   | STORED — space-separated tags from YAML frontmatter; boosted 2×
//! • `body`  — TEXT           — document body (frontmatter stripped); not stored

use anyhow::{Context, Result};
use serde::Serialize;
use std::{path::Path, sync::Mutex};
use tantivy::{
    collector::{Count, TopDocs},
    doc,
    query::{FuzzyTermQuery, QueryParser, TermQuery},
    schema::Value as TantivyValue,
    schema::{Field, IndexRecordOption, Schema, STORED, STRING, TEXT},
    snippet::SnippetGenerator,
    Index, IndexReader, IndexWriter, ReloadPolicy, Term,
};

use crate::vault::Vault;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub score: f32,
    /// Up to 3 matching excerpts with the query term in context.
    pub snippets: Vec<String>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub total: usize,
    pub results: Vec<SearchResult>,
}

// ── Index ─────────────────────────────────────────────────────────────────────

pub struct SearchIndex {
    index: Index,
    /// Serialized: one writer at a time; kept open so segments are reused.
    writer: Mutex<IndexWriter>,
    reader: IndexReader,
    f_path: Field,
    f_title: Field,
    f_tags: Field,
    f_body: Field,
}

impl SearchIndex {
    /// Open an existing index or create a fresh one at `.mdkb/search-index/`.
    pub fn build_or_open(vault_path: &Path) -> Result<Self> {
        let index_dir = vault_path.join(".mdkb").join("search-index");
        std::fs::create_dir_all(&index_dir).context("create index dir")?;

        let mut sb = Schema::builder();
        let f_path = sb.add_text_field("path", STRING | STORED);
        let f_title = sb.add_text_field("title", TEXT | STORED);
        let f_tags = sb.add_text_field("tags", TEXT | STORED);
        let f_body = sb.add_text_field("body", TEXT | STORED);
        let schema = sb.build();

        let mmap =
            tantivy::directory::MmapDirectory::open(&index_dir).context("open index directory")?;

        // Open existing index; recreate on any schema mismatch or corruption.
        let index = if Index::exists(&mmap).unwrap_or(false) {
            match Index::open(mmap) {
                Ok(idx) => idx,
                Err(e) => {
                    tracing::warn!("Search index corrupt ({e}), recreating");
                    std::fs::remove_dir_all(&index_dir)?;
                    std::fs::create_dir_all(&index_dir)?;
                    let mmap2 = tantivy::directory::MmapDirectory::open(&index_dir)
                        .context("reopen index directory")?;
                    Index::open_or_create(mmap2, schema).context("create index")?
                }
            }
        } else {
            Index::open_or_create(mmap, schema).context("create index")?
        };

        // 50 MB write buffer — comfortable for homelab vault sizes.
        let writer = index.writer(50_000_000).context("create writer")?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .context("create reader")?;

        Ok(Self {
            index,
            writer: Mutex::new(writer),
            reader,
            f_path,
            f_title,
            f_tags,
            f_body,
        })
    }

    /// Full rebuild: index every `.md` file in the vault from scratch.
    /// Called once at startup after `build_or_open`.
    pub fn index_all(&self, vault: &Vault) -> Result<usize> {
        let files = vault.list_files(u64::MAX)?;
        let mut writer = self.writer.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        writer.delete_all_documents()?;

        let mut count = 0;
        for file in files {
            if file.is_dir || !file.path.ends_with(".md") {
                continue;
            }
            let Ok(bytes) = vault.read_file(&file.path) else {
                continue;
            };
            let Ok(content) = std::str::from_utf8(&bytes) else {
                continue;
            };
            let (title, tags, body) = extract_fields(&file.path, content);
            writer.add_document(doc!(
                self.f_path  => file.path.as_str(),
                self.f_title => title.as_str(),
                self.f_tags  => tags.as_str(),
                self.f_body  => body.as_str(),
            ))?;
            count += 1;
        }
        writer.commit()?;
        drop(writer);
        let _ = self.reader.reload();
        Ok(count)
    }

    /// Add or replace a single file's entry in the index.
    pub fn update_file(&self, path: &str, content: &str) {
        let Ok(mut writer) = self.writer.lock() else {
            return;
        };
        let (title, tags, body) = extract_fields(path, content);
        writer.delete_term(Term::from_field_text(self.f_path, path));
        let _ = writer.add_document(doc!(
            self.f_path  => path,
            self.f_title => title.as_str(),
            self.f_tags  => tags.as_str(),
            self.f_body  => body.as_str(),
        ));
        let _ = writer.commit();
        drop(writer);
        let _ = self.reader.reload();
    }

    /// Remove a file from the index.
    pub fn delete_file(&self, path: &str) {
        let Ok(mut writer) = self.writer.lock() else {
            return;
        };
        writer.delete_term(Term::from_field_text(self.f_path, path));
        let _ = writer.commit();
        drop(writer);
        let _ = self.reader.reload();
    }

    /// Search the index.  Returns BM25-ranked results with snippets.
    pub fn search(
        &self,
        q: &str,
        limit: usize,
        offset: usize,
        path_prefix: Option<&str>,
    ) -> SearchResponse {
        let searcher = self.reader.searcher();
        let Some(query) = self.build_query(q.trim()) else {
            return SearchResponse {
                total: 0,
                results: vec![],
            };
        };

        // Fetch enough to cover offset + limit; we filter by path_prefix afterwards.
        let fetch = (offset + limit).max(100);
        let Ok(top_docs) = searcher.search(&*query, &TopDocs::with_limit(fetch)) else {
            return SearchResponse {
                total: 0,
                results: vec![],
            };
        };
        let total = searcher.search(&*query, &Count).unwrap_or(0);

        let snippet_gen = SnippetGenerator::create(&searcher, &*query, self.f_body).ok();

        let results: Vec<SearchResult> = top_docs
            .into_iter()
            .filter_map(|(score, addr)| {
                let doc: tantivy::TantivyDocument = searcher.doc(addr).ok()?;

                let path = doc
                    .get_first(self.f_path)
                    .map(|v| {
                        use TantivyValue;
                        v.as_str().unwrap_or_default().to_string()
                    })
                    .unwrap_or_default();

                // Apply path prefix filter if requested
                if let Some(prefix) = path_prefix {
                    if !path.starts_with(prefix) {
                        return None;
                    }
                }

                let title = doc
                    .get_first(self.f_title)
                    .map(|v| {
                        use TantivyValue;
                        v.as_str().unwrap_or_default().to_string()
                    })
                    .unwrap_or_default();

                // Try to get a highlighted snippet from the body.
                // If empty (e.g. the match was in the title/tags field), fall back
                // to the first non-empty line of the stored body as context.
                let body_text = doc
                    .get_first(self.f_body)
                    .map(|v| {
                        use TantivyValue;
                        v.as_str().unwrap_or_default().to_string()
                    })
                    .unwrap_or_default();

                let snippets = {
                    let highlighted = snippet_gen.as_ref().and_then(|gen| {
                        let s = gen.snippet_from_doc(&doc);
                        let text = strip_html(s.to_html());
                        if text.trim().is_empty() {
                            None
                        } else {
                            Some(text)
                        }
                    });
                    match highlighted {
                        Some(s) => vec![s],
                        None => {
                            // Fallback: scan body for a line containing any query term.
                            // Strip known prefixes so "tag:rust" searches for "rust".
                            let bare = q
                                .strip_prefix("tag:")
                                .or_else(|| q.strip_prefix("title:"))
                                .unwrap_or(q)
                                .trim();
                            let terms: Vec<&str> = bare.split_whitespace().collect();
                            let hit = body_text.lines().find(|l| {
                                let lower = l.to_lowercase();
                                !l.trim().is_empty()
                                    && !l.trim().starts_with('#')
                                    && terms.iter().any(|t| lower.contains(&t.to_lowercase()))
                            });
                            // If no term match in body, show first non-blank non-heading line.
                            let line = hit.or_else(|| {
                                body_text.lines().find(|l| {
                                    let t = l.trim();
                                    !t.is_empty() && !t.starts_with('#')
                                })
                            });
                            line.map(|l| l.trim().to_string()).into_iter().collect()
                        }
                    }
                };

                Some(SearchResult {
                    path,
                    title,
                    score,
                    snippets,
                })
            })
            .skip(offset)
            .take(limit)
            .collect();

        SearchResponse { total, results }
    }

    fn build_query(&self, q: &str) -> Option<Box<dyn tantivy::query::Query>> {
        if q.is_empty() {
            return None;
        }

        // tag:<term> — search only the tags field
        if let Some(term) = q.strip_prefix("tag:") {
            let t = Term::from_field_text(self.f_tags, &term.trim().to_lowercase());
            return Some(Box::new(TermQuery::new(t, IndexRecordOption::Basic)));
        }

        // title:<term> — search only the title field
        if let Some(term) = q.strip_prefix("title:") {
            let t = Term::from_field_text(self.f_title, &term.trim().to_lowercase());
            return Some(Box::new(TermQuery::new(t, IndexRecordOption::Basic)));
        }

        // Standard multi-field query with boosts.
        // QueryParser handles multi-word, phrase ("..."), AND/OR operators.
        let mut parser =
            QueryParser::for_index(&self.index, vec![self.f_body, self.f_title, self.f_tags]);
        parser.set_field_boost(self.f_title, 3.0);
        parser.set_field_boost(self.f_tags, 2.0);

        match parser.parse_query(q) {
            Ok(query) => Some(query),
            Err(_) => {
                // If the user typed something the parser rejects (e.g. bare `:`)
                // fall back to a fuzzy single-term search on the body.
                let term = Term::from_field_text(self.f_body, q);
                Some(Box::new(FuzzyTermQuery::new(term, 1, true)))
            }
        }
    }
}

// ── Field extraction helpers ──────────────────────────────────────────────────

/// Extract (title, tags, body) from raw file content.
///
/// - `title`: first `# Heading` found in the body, or the filename stem.
/// - `tags`:  space-joined tags from YAML frontmatter `tags:` field.
/// - `body`:  everything after the frontmatter (what gets indexed for BM25).
fn extract_fields(path: &str, content: &str) -> (String, String, String) {
    let content = content.trim_start_matches('\u{FEFF}'); // strip BOM
    let (frontmatter, body) = split_frontmatter(content);

    let tags = extract_tags(frontmatter);

    // First `# Heading` in the body, or filename stem as fallback.
    let title = body
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l[2..].trim().to_string())
        .unwrap_or_else(|| {
            Path::new(path)
                .file_stem()
                .map(|s| s.to_string_lossy().replace(['-', '_'], " "))
                .unwrap_or_default()
        });

    (title, tags, body.to_string())
}

/// Split content into (frontmatter, body).  Both halves may be empty strings.
fn split_frontmatter(content: &str) -> (&str, &str) {
    if !content.starts_with("---") {
        return ("", content);
    }
    // Find the closing `---` on its own line, after the opening delimiter.
    let rest = &content[3..];
    // Skip the first newline after `---`
    let after_nl = match rest.find('\n') {
        Some(i) => &rest[i + 1..],
        None => return ("", content),
    };
    // Find `\n---` (possibly followed by `\n` or end-of-string)
    if let Some(close) = after_nl.find("\n---") {
        let fm = &after_nl[..close];
        let body_start = close + 4; // skip \n---
        let body = after_nl
            .get(body_start..)
            .map(|s| s.trim_start_matches(['\n', '\r']))
            .unwrap_or("");
        (fm, body)
    } else {
        ("", content)
    }
}

/// Extract tags from the frontmatter block as a space-separated string.
///
/// Handles both inline (`tags: [a, b]`) and block (`tags:\n  - a`) forms.
fn extract_tags(frontmatter: &str) -> String {
    let mut tags: Vec<&str> = Vec::new();
    let mut in_tags = false;

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed
            .strip_prefix("tags:")
            .or_else(|| trimmed.strip_prefix("tag:"))
        {
            let rest = rest.trim();
            if rest.starts_with('[') {
                // Inline list: [tag1, "tag 2", ...]
                let inner = rest.trim_matches(|c| c == '[' || c == ']');
                for tag in inner.split(',') {
                    let t = tag.trim().trim_matches('"').trim_matches('\'').trim();
                    if !t.is_empty() {
                        tags.push(t);
                    }
                }
                in_tags = false;
            } else if rest.is_empty() {
                in_tags = true;
            } else {
                // Single inline value: tags: rust
                tags.push(rest.trim_matches('"').trim_matches('\''));
                in_tags = false;
            }
        } else if in_tags {
            if let Some(tag) = trimmed.strip_prefix("- ") {
                tags.push(tag.trim().trim_matches('"').trim_matches('\''));
            } else if !trimmed.is_empty() {
                in_tags = false;
            }
        }
    }

    tags.join(" ")
}

/// Remove HTML tags from a Tantivy snippet (which uses `<b>` for highlights).
fn strip_html(s: String) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Collapse excess whitespace
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter_present() {
        let content = "---\ntitle: Foo\ntags: [rust]\n---\n# Foo\nHello world";
        let (fm, body) = split_frontmatter(content);
        assert!(fm.contains("title: Foo"));
        assert!(body.starts_with("# Foo"));
    }

    #[test]
    fn test_split_frontmatter_absent() {
        let content = "# Just a heading\nNo frontmatter";
        let (fm, body) = split_frontmatter(content);
        assert!(fm.is_empty());
        assert!(body.contains("heading"));
    }

    #[test]
    fn test_extract_tags_inline() {
        let tags = extract_tags("tags: [rust, async, tokio]");
        assert_eq!(tags, "rust async tokio");
    }

    #[test]
    fn test_extract_tags_block() {
        let tags = extract_tags("tags:\n  - rust\n  - async");
        assert_eq!(tags, "rust async");
    }

    #[test]
    fn test_strip_html() {
        let s = "foo <b>bar</b> baz".to_string();
        assert_eq!(strip_html(s), "foo bar baz");
    }
}
