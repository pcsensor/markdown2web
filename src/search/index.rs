use crate::content::{Note, SiteData};

pub fn search_notes(site: &SiteData, query: &str) -> Vec<Note> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    site.published_notes()
        .into_iter()
        .filter(|note| {
            note.title.to_lowercase().contains(&query)
                || note.summary.to_lowercase().contains(&query)
                || note.raw_markdown.to_lowercase().contains(&query)
                || note
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&query))
        })
        .collect()
}
