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

    let candidate = if target.starts_with('/') {
        resolve_site_relative_asset(config, target)?
    } else {
        let note_dir = note
            .source_path
            .parent()
            .unwrap_or(config.notes_dir.as_path());
        let candidate = normalize_join(note_dir, target);
        if candidate.exists() {
            candidate
        } else {
            let shared = normalize_join(&config.assets_dir, target);
            if shared.exists() {
                shared
            } else {
                return None;
            }
        }
    };

    let filename = hashed_name(&candidate).ok()?;
    let output_rel_path = PathBuf::from(filename);
    let public_url = format!("/assets/{}", output_rel_path.to_string_lossy());
    Some(AssetCandidate {
        source_path: candidate,
        public_url,
        output_rel_path,
    })
}

fn resolve_site_relative_asset(config: &AppConfig, target: &str) -> Option<PathBuf> {
    let trimmed = target.trim_start_matches('/');
    let mut candidates = vec![normalize_join(&config.content_dir, trimmed)];

    if let Some(stripped) = trimmed.strip_prefix("assets/") {
        candidates.push(normalize_join(&config.assets_dir, stripped));
    }
    candidates.push(normalize_join(&config.assets_dir, trimmed));

    candidates.into_iter().find(|candidate| candidate.exists())
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;
    use crate::content::front_matter::FrontMatter;

    #[test]
    fn resolves_site_relative_assets_from_content_assets() {
        let temp = TempDir::new().unwrap();
        let content_dir = temp.path().join("content");
        let notes_dir = content_dir.join("notes");
        let assets_dir = content_dir.join("assets");
        let generated_dir = temp.path().join("generated/site");
        let data_dir = temp.path().join("data");
        std::fs::create_dir_all(&notes_dir).unwrap();
        std::fs::create_dir_all(&assets_dir).unwrap();
        std::fs::write(assets_dir.join("voice.mp3"), b"fake-mp3").unwrap();

        let config = AppConfig {
            host: "127.0.0.1".into(),
            port: 3000,
            base_url: "http://127.0.0.1:3000".into(),
            site_name: "Test".into(),
            content_dir: content_dir.clone(),
            notes_dir: notes_dir.clone(),
            assets_dir: assets_dir.clone(),
            generated_dir: generated_dir.clone(),
            generated_assets_dir: generated_dir.join("assets"),
            data_dir,
            admin_username: "admin".into(),
            admin_password: "admin123456".into(),
            watch_enabled: false,
            upload_limit_mb: 10,
            turnstile_site_key: String::new(),
            turnstile_secret_key: String::new(),
        };
        let note = NoteSource {
            source_path: notes_dir.join("demo.md"),
            relative_path: PathBuf::from("demo.md"),
            front_matter: FrontMatter::default(),
            body: String::new(),
            title: "Demo".into(),
            slug: "demo".into(),
            summary: String::new(),
            category: vec![],
            tags: vec![],
            status: "published".into(),
            aliases: vec![],
            hash: "hash".into(),
        };

        let asset = resolve_asset_reference(&config, &note, "/assets/voice.mp3").unwrap();
        assert!(asset.public_url.starts_with("/assets/"));
        assert!(
            asset
                .output_rel_path
                .to_string_lossy()
                .ends_with("voice.mp3")
        );
    }
}
