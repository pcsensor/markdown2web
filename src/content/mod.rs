pub mod assets;
pub mod front_matter;
pub mod graph;
pub mod links;
pub mod markdown;

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;

use front_matter::FrontMatter;

#[derive(Debug, Clone)]
pub struct NoteSource {
    pub source_path: std::path::PathBuf,
    pub relative_path: std::path::PathBuf,
    pub front_matter: FrontMatter,
    pub body: String,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub category: String,
    pub tags: Vec<String>,
    pub status: String,
    pub aliases: Vec<String>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Heading {
    pub level: u8,
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AssetRecord {
    pub source_path: String,
    pub output_rel_path: String,
    pub public_url: String,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BrokenLink {
    pub source_slug: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Note {
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub status: String,
    pub aliases: Vec<String>,
    pub category: String,
    pub source_path: String,
    pub raw_markdown: String,
    pub html: String,
    pub headings: Vec<Heading>,
    pub outbound_links: Vec<String>,
    pub asset_refs: Vec<AssetRecord>,
    pub updated_at: String,
    pub created_at: String,
    pub word_count: usize,
}

impl Note {
    pub fn is_published(&self) -> bool {
        self.status != "draft"
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SiteData {
    pub notes: BTreeMap<String, Note>,
    pub ordered_slugs: Vec<String>,
    pub tags: BTreeMap<String, Vec<String>>,
    pub backlinks: BTreeMap<String, Vec<String>>,
    pub broken_links: Vec<BrokenLink>,
    pub assets: Vec<AssetRecord>,
    pub generated_at: Option<DateTime<Utc>>,
    pub last_reason: String,
    pub build_message: String,
}

impl SiteData {
    pub fn all_notes(&self) -> Vec<Note> {
        let mut notes = self.notes.values().cloned().collect::<Vec<_>>();
        notes.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        notes
    }

    pub fn published_notes(&self) -> Vec<Note> {
        let mut notes = self
            .ordered_slugs
            .iter()
            .filter_map(|slug| self.notes.get(slug))
            .filter(|note| note.is_published())
            .cloned()
            .collect::<Vec<_>>();
        // Sort notes by updated_at in descending order (newest first)
        notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        notes
    }

    pub fn note(&self, slug: &str) -> Option<Note> {
        self.notes.get(slug).cloned()
    }
}
