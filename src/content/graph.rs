use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;

use crate::content::{BrokenLink, Note, SiteData};

pub fn build_site_data(
    mut notes: Vec<Note>,
    broken_links: Vec<BrokenLink>,
    assets: Vec<crate::content::AssetRecord>,
    reason: String,
) -> SiteData {
    notes.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    let mut ordered_slugs = Vec::new();
    let mut note_map = BTreeMap::new();
    let mut tags = BTreeMap::<String, Vec<String>>::new();
    let mut backlinks = BTreeMap::<String, BTreeSet<String>>::new();

    for note in notes {
        let is_published = note.is_published();
        if is_published {
            ordered_slugs.push(note.slug.clone());
            for tag in &note.tags {
                tags.entry(tag.clone()).or_default().push(note.slug.clone());
            }
            for outbound in &note.outbound_links {
                backlinks
                    .entry(outbound.clone())
                    .or_default()
                    .insert(note.slug.clone());
            }
        }
        note_map.insert(note.slug.clone(), note);
    }

    SiteData {
        notes: note_map,
        ordered_slugs,
        tags,
        backlinks: backlinks
            .into_iter()
            .map(|(slug, refs)| (slug, refs.into_iter().collect()))
            .collect(),
        broken_links,
        assets,
        generated_at: Some(Utc::now()),
        last_reason: reason,
        build_message: "Build completed successfully".into(),
    }
}
