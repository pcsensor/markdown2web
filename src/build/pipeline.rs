use std::{collections::HashSet, sync::Arc};

use tokio::sync::{Mutex, RwLock};

use crate::{
    build::cache::BuildCache,
    config::AppConfig,
    content::{
        AssetRecord, Note, SiteData,
        assets::{AssetCandidate, materialize_assets},
        graph::build_site_data,
        links::{LinkLookup, rewrite_markdown},
        markdown::{render_markdown, word_count},
    },
    error::AppResult,
    store::{filesystem, sqlite::AppDatabase},
    time,
};

#[derive(Debug, Clone)]
pub struct BuildSummary {
    pub note_count: usize,
    pub asset_count: usize,
    pub changed_count: usize,
    pub reason: String,
}

pub struct BuildService {
    config: AppConfig,
    db: Arc<AppDatabase>,
    pub site: Arc<RwLock<SiteData>>,
    cache: Mutex<BuildCache>,
    build_lock: Mutex<()>,
}

impl BuildService {
    pub fn new(
        config: AppConfig,
        db: Arc<AppDatabase>,
        site: Arc<RwLock<SiteData>>,
    ) -> AppResult<Self> {
        let cache_path = cache_path(&config);
        let cache = BuildCache::load(&cache_path)?;
        Ok(Self {
            config,
            db,
            site,
            cache: Mutex::new(cache),
            build_lock: Mutex::new(()),
        })
    }

    pub async fn rebuild(&self, reason: impl Into<String>) -> AppResult<BuildSummary> {
        let reason = reason.into();
        let _guard = self.build_lock.lock().await;

        let discovered = filesystem::discover_notes(&self.config)?;
        let lookup = LinkLookup::new(&discovered);

        let mut notes = Vec::new();
        let mut broken_links = Vec::new();
        let mut all_assets = Vec::<AssetCandidate>::new();
        let mut new_hashes = std::collections::HashMap::new();

        for source in discovered {
            let rewritten = rewrite_markdown(&self.config, &source, &lookup)?;
            let (html, headings) = render_markdown(&rewritten.markdown)?;
            let note_assets: Vec<AssetRecord> = rewritten
                .assets
                .iter()
                .map(asset_record_from_candidate)
                .collect();
            all_assets.extend(rewritten.assets);
            broken_links.extend(rewritten.broken_links);
            new_hashes.insert(source.slug.clone(), source.hash.clone());
            notes.push(Note {
                title: source.title,
                slug: source.slug,
                summary: source.summary,
                tags: source.tags,
                status: source.status,
                aliases: source.aliases,
                source_path: source.source_path.to_string_lossy().to_string(),
                raw_markdown: source.body.clone(),
                html,
                headings,
                outbound_links: rewritten.outbound_links,
                asset_refs: note_assets,
                updated_at: time::now_cst_display(),
                word_count: word_count(&source.body),
            });
        }

        dedupe_assets(&mut all_assets);
        let changed_count = {
            let cache = self.cache.lock().await;
            cache.changed_count(
                new_hashes
                    .iter()
                    .map(|(slug, hash)| (slug.as_str(), hash.as_str())),
            )
        };
        let site_assets = materialize_assets(&self.config, &all_assets)?;
        let site_data = build_site_data(notes, broken_links, site_assets.clone(), reason.clone());
        {
            let mut site = self.site.write().await;
            *site = site_data;
        }
        {
            let mut cache = self.cache.lock().await;
            cache.note_hashes = new_hashes;
            cache.save(&cache_path(&self.config))?;
        }
        self.db.log_build(
            "info",
            &format!(
                "rebuild reason={} notes={} assets={} changed={}",
                reason,
                self.site.read().await.notes.len(),
                site_assets.len(),
                changed_count
            ),
        )?;

        Ok(BuildSummary {
            note_count: self.site.read().await.notes.len(),
            asset_count: site_assets.len(),
            changed_count,
            reason,
        })
    }
}

fn cache_path(config: &AppConfig) -> std::path::PathBuf {
    config.generated_dir.join("cache/build-cache.json")
}

fn asset_record_from_candidate(asset: &AssetCandidate) -> AssetRecord {
    AssetRecord {
        source_path: asset.source_path.to_string_lossy().to_string(),
        output_rel_path: asset.output_rel_path.to_string_lossy().to_string(),
        public_url: asset.public_url.clone(),
        content_type: mime_guess::from_path(&asset.source_path)
            .first_or_octet_stream()
            .to_string(),
    }
}

fn dedupe_assets(assets: &mut Vec<AssetCandidate>) {
    let mut seen = HashSet::new();
    assets.retain(|asset| seen.insert(asset.public_url.clone()));
}
