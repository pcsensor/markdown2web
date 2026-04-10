use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
};

use regex::Regex;

use crate::{
    config::AppConfig,
    content::{
        BrokenLink, NoteSource,
        assets::{AssetCandidate, resolve_asset_reference},
        markdown::slugify,
    },
    error::AppResult,
};

#[derive(Debug, Clone)]
pub struct LinkLookup {
    by_slug_like: HashMap<String, String>,
    by_path: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct RewriteOutcome {
    pub markdown: String,
    pub outbound_links: Vec<String>,
    pub broken_links: Vec<BrokenLink>,
    pub assets: Vec<AssetCandidate>,
}

fn normalize_relative_path(path: &Path) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized.to_string_lossy().replace('\\', "/")
}

impl LinkLookup {
    pub fn new(notes: &[NoteSource]) -> Self {
        let mut by_slug_like = HashMap::new();
        let mut by_path = HashMap::new();
        for note in notes {
            by_slug_like.insert(note.slug.to_lowercase(), note.slug.clone());
            by_slug_like.insert(slugify(&note.title), note.slug.clone());
            by_slug_like.insert(
                slugify(
                    note.relative_path
                        .file_stem()
                        .and_then(|v| v.to_str())
                        .unwrap_or_default(),
                ),
                note.slug.clone(),
            );
            for alias in &note.aliases {
                by_slug_like.insert(alias.to_lowercase(), note.slug.clone());
                by_slug_like.insert(slugify(alias), note.slug.clone());
            }
            by_path.insert(
                normalize_relative_path(&note.relative_path),
                note.slug.clone(),
            );
        }
        Self {
            by_slug_like,
            by_path,
        }
    }

    fn resolve_relative_markdown(&self, note: &NoteSource, destination: &str) -> Option<String> {
        let note_dir = note.relative_path.parent().unwrap_or_else(|| Path::new(""));
        let mut joined = note_dir.join(destination);
        if joined.extension().and_then(|ext| ext.to_str()) != Some("md") {
            joined.set_extension("md");
        }
        self.by_path.get(&normalize_relative_path(&joined)).cloned()
    }

    fn resolve_slug_like(&self, raw: &str) -> Option<String> {
        let key = slugify(raw);
        self.by_slug_like
            .get(&key)
            .cloned()
            .or_else(|| self.by_slug_like.get(&raw.to_lowercase()).cloned())
    }
}

pub fn rewrite_markdown(
    config: &AppConfig,
    note: &NoteSource,
    lookup: &LinkLookup,
) -> AppResult<RewriteOutcome> {
    let image_re = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").expect("valid regex");
    let link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").expect("valid regex");
    let wiki_re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").expect("valid regex");

    let mut outcome = RewriteOutcome {
        markdown: note.body.clone(),
        ..Default::default()
    };

    outcome.markdown = image_re
        .replace_all(&outcome.markdown, |caps: &regex::Captures| {
            let alt = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let target = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
            if let Some(asset) = resolve_asset_reference(config, note, target) {
                outcome.assets.push(asset.clone());
                format!("![{}]({})", alt, asset.public_url)
            } else {
                outcome.broken_links.push(BrokenLink {
                    source_slug: note.slug.clone(),
                    target: target.to_string(),
                });
                format!("![{}]({})", alt, target)
            }
        })
        .into_owned();

    outcome.markdown = wiki_re
        .replace_all(&outcome.markdown, |caps: &regex::Captures| {
            let target = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let label = caps.get(2).map(|m| m.as_str()).unwrap_or(target);
            if let Some(slug) = lookup.resolve_slug_like(target) {
                outcome.outbound_links.push(slug.clone());
                format!("[{}](/notes/{})", label, slug)
            } else {
                outcome.broken_links.push(BrokenLink {
                    source_slug: note.slug.clone(),
                    target: target.to_string(),
                });
                format!("<span class=\"broken-link\">{}</span>", label)
            }
        })
        .into_owned();

    outcome.markdown = link_re
        .replace_all(&outcome.markdown, |caps: &regex::Captures| {
            let label = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let target = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
            if target.ends_with(".md") {
                if let Some(slug) = lookup.resolve_relative_markdown(note, target) {
                    outcome.outbound_links.push(slug.clone());
                    format!("[{}](/notes/{})", label, slug)
                } else {
                    outcome.broken_links.push(BrokenLink {
                        source_slug: note.slug.clone(),
                        target: target.to_string(),
                    });
                    format!("<span class=\"broken-link\">{}</span>", label)
                }
            } else if let Some(asset) = resolve_asset_reference(config, note, target) {
                outcome.assets.push(asset.clone());
                format!("[{}]({})", label, asset.public_url)
            } else {
                caps.get(0)
                    .map(|m| m.as_str())
                    .unwrap_or_default()
                    .to_string()
            }
        })
        .into_owned();

    outcome.outbound_links.sort();
    outcome.outbound_links.dedup();
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{NoteSource, front_matter::FrontMatter};

    #[test]
    fn resolves_wiki_links_by_title() {
        let note = NoteSource {
            source_path: PathBuf::from("content/notes/source.md"),
            relative_path: PathBuf::from("source.md"),
            front_matter: FrontMatter::default(),
            body: "See [[Target Note]]".into(),
            title: "Source".into(),
            slug: "source".into(),
            summary: String::new(),
            category: vec![],
            tags: vec![],
            status: "published".into(),
            aliases: vec![],
            hash: "x".into(),
        };
        let target = NoteSource {
            source_path: PathBuf::from("content/notes/target.md"),
            relative_path: PathBuf::from("target.md"),
            front_matter: FrontMatter::default(),
            body: String::new(),
            title: "Target Note".into(),
            slug: "target-note".into(),
            summary: String::new(),
            category: vec![],
            tags: vec![],
            status: "published".into(),
            aliases: vec![],
            hash: "y".into(),
        };
        let lookup = LinkLookup::new(&[note.clone(), target]);
        let config = AppConfig::from_env().unwrap();
        let rewritten = rewrite_markdown(&config, &note, &lookup).unwrap();
        assert!(rewritten.markdown.contains("/notes/target-note"));
    }
}
