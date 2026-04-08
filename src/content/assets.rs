use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use mime_guess::from_path;
use sha2::{Digest, Sha256};

use crate::{
    config::AppConfig,
    content::{AssetRecord, NoteSource},
    error::{AppError, AppResult},
};

#[derive(Debug, Clone)]
pub struct AssetCandidate {
    pub source_path: PathBuf,
    pub public_url: String,
    pub output_rel_path: PathBuf,
}

fn normalize_join(base: &Path, relative: &str) -> PathBuf {
    let joined = base.join(relative);
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn hashed_name(path: &Path) -> AppResult<String> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hex::encode(hasher.finalize());
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::BadRequest("invalid asset filename".into()))?;
    Ok(format!("{}-{}", &digest[..12], filename))
}

pub fn resolve_asset_reference(
    config: &AppConfig,
    note: &NoteSource,
    target: &str,
) -> Option<AssetCandidate> {
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with('#')
        || target.starts_with("mailto:")
    {
        return None;
    }

    let note_dir = note
        .source_path
        .parent()
        .unwrap_or(config.notes_dir.as_path());
    let mut candidate = normalize_join(note_dir, target);
    if !candidate.exists() {
        let shared = normalize_join(&config.assets_dir, target);
        if shared.exists() {
            candidate = shared;
        } else {
            return None;
        }
    }

    let filename = hashed_name(&candidate).ok()?;
    let output_rel_path = PathBuf::from(filename);
    let public_url = format!("/assets/{}", output_rel_path.to_string_lossy());
    Some(AssetCandidate {
        source_path: candidate,
        public_url,
        output_rel_path,
    })
}

pub fn materialize_assets(
    config: &AppConfig,
    assets: &[AssetCandidate],
) -> AppResult<Vec<AssetRecord>> {
    fs::create_dir_all(&config.generated_assets_dir)?;
    let mut records = Vec::new();
    for asset in assets {
        let destination = config.generated_assets_dir.join(&asset.output_rel_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&asset.source_path, &destination)?;
        records.push(AssetRecord {
            source_path: asset.source_path.to_string_lossy().to_string(),
            output_rel_path: asset.output_rel_path.to_string_lossy().to_string(),
            public_url: asset.public_url.clone(),
            content_type: from_path(&asset.source_path)
                .first_or_octet_stream()
                .to_string(),
        });
    }
    records.sort_by(|a, b| a.public_url.cmp(&b.public_url));
    records.dedup_by(|a, b| a.public_url == b.public_url);
    Ok(records)
}
