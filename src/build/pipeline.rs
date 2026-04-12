use std::{collections::HashSet, sync::Arc};

use chrono::{DateTime, FixedOffset, Utc};

use tokio::sync::{Mutex, RwLock};

use crate::{
    build::cache::BuildCache,
    config::AppConfig,
    content::{
        AssetRecord, Note, SiteData,
        assets::{AssetCandidate, apply_media_optimizations, materialize_assets},
        graph::build_site_data,
        links::{LinkLookup, rewrite_markdown},
        markdown::{render_markdown, word_count},
    },
    error::AppResult,
    store::{filesystem, sqlite::AppDatabase},
    time,
};

use crate::content::assets::MediaJob;

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct BuildProgress {
    pub session_id: u64,
    pub is_running: bool,
    pub total_jobs: usize,
    pub completed_jobs: usize,
    pub current_job: Option<MediaJob>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildSummary {
    pub note_count: usize,
    pub asset_count: usize,
    pub changed_count: usize,
    pub reason: String,
    pub media_jobs: Vec<MediaJob>,
}

pub struct BuildService {
    config: AppConfig,
    db: Arc<AppDatabase>,
    pub site: Arc<RwLock<SiteData>>,
    pub progress: Arc<RwLock<BuildProgress>>,
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
            progress: Arc::new(RwLock::new(BuildProgress::default())),
            cache: Mutex::new(cache),
            build_lock: Mutex::new(()),
        })
    }

    pub async fn spawn_media_worker(self: Arc<Self>, jobs: Vec<MediaJob>) {
        if jobs.is_empty() {
            // 如果没有新任务，保持静默，不更新 session_id
            let mut p = self.progress.write().await;
            p.is_running = false;
            return;
        }

        let progress_ptr = self.progress.clone();
        let new_session_id = rand::random::<u64>();

        tokio::spawn(async move {
            {
                let mut p = progress_ptr.write().await;
                p.session_id = new_session_id;
                p.is_running = true;
                p.total_jobs = jobs.len();
                p.completed_jobs = 0;
                p.last_error = None;
            }

            for job in jobs {
                {
                    let mut p = progress_ptr.write().await;
                    p.current_job = Some(job.clone());
                }

                println!("Background processing: {}", job.destination);
                let success = crate::content::assets::run_ffmpeg(job.args);

                {
                    let mut p = progress_ptr.write().await;
                    if success {
                        p.completed_jobs += 1;
                    } else {
                        p.last_error = Some(format!("Failed to process {}", job.destination));
                    }
                }
            }

            {
                let mut p = progress_ptr.write().await;
                p.is_running = false;
                p.current_job = None;
            }
            println!("Background media processing complete.");
        });
    }

    pub async fn rebuild(&self, reason: impl Into<String>) -> AppResult<BuildSummary> {
        let reason = reason.into();
        let _guard = self.build_lock.lock().await;

        println!("Starting rebuild (reason: {})...", reason);
        let discovered = filesystem::discover_notes(&self.config)?;
        println!("Discovered {} notes.", discovered.len());
        let lookup = LinkLookup::new(&discovered);

        let mut notes = Vec::new();
        let mut broken_links = Vec::new();
        let mut all_assets = Vec::<AssetCandidate>::new();
        let mut new_hashes = std::collections::HashMap::new();

        for (i, source) in discovered.into_iter().enumerate() {
            if i % 20 == 0 && i > 0 {
                println!("Processing notes: {}/...", i);
            }
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
            // Compute last modification time: FrontMatter.updated > filesystem mtime
            let updated_at = if let Some(fm_updated) = &source.front_matter.updated {
                fm_updated.clone()
            } else {
                match std::fs::metadata(&source.source_path) {
                    Ok(meta) => match meta.modified() {
                        Ok(mtime) => {
                            let dt_utc: DateTime<Utc> = DateTime::<Utc>::from(mtime);
                            dt_utc
                                .with_timezone(
                                    &FixedOffset::east_opt(8 * 3600).expect("UTC+8 is valid"),
                                )
                                .format("%Y-%m-%d %H:%M")
                                .to_string()
                        }
                        Err(_) => source
                            .front_matter
                            .updated
                            .clone()
                            .unwrap_or_else(time::now_cst_display),
                    },
                    Err(_) => source
                        .front_matter
                        .updated
                        .clone()
                        .unwrap_or_else(time::now_cst_display),
                }
            };
            // Prepare a clone for created_at before moving updated_at into Note
            let updated_at_clone = updated_at.clone();
            notes.push(Note {
                title: source.title,
                slug: source.slug,
                summary: source.summary,
                tags: source.tags,
                status: source.status,
                aliases: source.aliases,
                category: source.category,
                source_path: source.source_path.to_string_lossy().to_string(),
                raw_markdown: source.body.clone(),
                html,
                headings,
                outbound_links: rewritten.outbound_links,
                asset_refs: note_assets,
                updated_at,
                created_at: updated_at_clone,
                word_count: word_count(&source.body),
            });
        }

        println!("Deduplicating assets...");
        dedupe_assets(&mut all_assets);
        let changed_count = {
            let cache = self.cache.lock().await;
            cache.changed_count(
                new_hashes
                    .iter()
                    .map(|(slug, hash)| (slug.as_str(), hash.as_str())),
            )
        };
        println!("Materializing {} assets...", all_assets.len());
        let materialized_assets = materialize_assets(&self.config, &all_assets)?;
        println!("Applying media optimizations to notes...");
        for note in &mut notes {
            note.html = apply_media_optimizations(&note.html, &materialized_assets.media);
        }
        for warning in &materialized_assets.warnings {
            self.db.log_build("warn", warning)?;
        }
        let site_assets = materialized_assets.records;
        let mut media_jobs = materialized_assets.jobs;
        println!("Building site graph...");
        let site_data = build_site_data(notes, broken_links, site_assets.clone(), reason.clone());
        {
            let mut site = self.site.write().await;
            *site = site_data;
        }
        {
            let mut cache = self.cache.lock().await;
            media_jobs = filter_media_jobs(&self.config, &reason, media_jobs, &mut cache);
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

        println!("Rebuild complete.");
        Ok(BuildSummary {
            note_count: self.site.read().await.notes.len(),
            asset_count: site_assets.len(),
            changed_count,
            reason,
            media_jobs,
        })
    }
}

fn cache_path(config: &AppConfig) -> std::path::PathBuf {
    config.generated_dir.join("cache/build-cache.json")
}

fn filter_media_jobs(
    config: &AppConfig,
    reason: &str,
    jobs: Vec<MediaJob>,
    cache: &mut BuildCache,
) -> Vec<MediaJob> {
    let suppress_for_note_save = reason.starts_with("admin save ");
    let force_retry = reason == "manual rebuild";

    jobs.into_iter()
        .filter(|job| {
            let key = media_job_cache_key(config, job);
            if suppress_for_note_save {
                return false;
            }
            let is_new = cache.media_job_destinations.insert(key);
            force_retry || is_new
        })
        .collect()
}

fn media_job_cache_key(config: &AppConfig, job: &MediaJob) -> String {
    std::path::Path::new(&job.destination)
        .strip_prefix(&config.generated_assets_dir)
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| job.destination.clone())
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
