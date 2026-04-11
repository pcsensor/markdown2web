use std::{
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

use mime_guess::from_path;
use regex::Regex;
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

#[derive(Debug, Clone, Default)]
pub struct MaterializedAssets {
    pub records: Vec<AssetRecord>,
    pub media: MediaManifest,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MediaManifest {
    pub images: HashMap<String, ResponsiveImage>,
    pub videos: HashMap<String, ProcessedVideo>,
}

#[derive(Debug, Clone)]
pub struct ResponsiveImage {
    pub fallback_url: String,
    pub srcset: String,
    pub source_type: String,
}

#[derive(Debug, Clone)]
pub struct ProcessedVideo {
    pub video_url: String,
    pub video_type: String,
    pub poster_url: Option<String>,
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
) -> AppResult<MaterializedAssets> {
    fs::create_dir_all(&config.generated_assets_dir)?;
    let mut records = Vec::new();
    let mut media = MediaManifest::default();
    let mut warnings = Vec::new();
    let ffmpeg_available = command_available("ffmpeg");

    for asset in assets {
        let content_type = from_path(&asset.source_path)
            .first_or_octet_stream()
            .to_string();

        if ffmpeg_available && is_image(&content_type) {
            match materialize_image_variants(config, asset) {
                Ok(Some(image)) => {
                    records.extend(image.records);
                    media.images.insert(asset.public_url.clone(), image.image);
                    continue;
                }
                Ok(None) => {}
                Err(err) => warnings.push(format!(
                    "image optimization skipped for {}: {}",
                    asset.source_path.display(),
                    err
                )),
            }
        }

        if ffmpeg_available && is_video(&content_type) {
            match materialize_video_variants(config, asset) {
                Ok(Some(video)) => {
                    records.extend(video.records);
                    media.videos.insert(asset.public_url.clone(), video.video);
                    continue;
                }
                Ok(None) => {}
                Err(err) => warnings.push(format!(
                    "video optimization skipped for {}: {}",
                    asset.source_path.display(),
                    err
                )),
            }
        } else if !ffmpeg_available && (is_image(&content_type) || is_video(&content_type)) {
            warnings.push(format!(
                "ffmpeg not found; publishing original asset {}",
                asset.source_path.display()
            ));
        }

        records.push(copy_original_asset(config, asset)?);
    }
    records.sort_by(|a, b| a.public_url.cmp(&b.public_url));
    records.dedup_by(|a, b| a.public_url == b.public_url);
    Ok(MaterializedAssets {
        records,
        media,
        warnings,
    })
}

pub fn apply_media_optimizations(html: &str, manifest: &MediaManifest) -> String {
    let mut output = html.to_string();
    for (original_url, image) in &manifest.images {
        output = replace_image_html(&output, original_url, image);
    }
    for (original_url, video) in &manifest.videos {
        output = replace_video_html(&output, original_url, video);
    }
    output
}

struct ImageMaterialization {
    records: Vec<AssetRecord>,
    image: ResponsiveImage,
}

struct VideoMaterialization {
    records: Vec<AssetRecord>,
    video: ProcessedVideo,
}

fn copy_original_asset(config: &AppConfig, asset: &AssetCandidate) -> AppResult<AssetRecord> {
    let destination = config.generated_assets_dir.join(&asset.output_rel_path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&asset.source_path, &destination)?;
    Ok(AssetRecord {
        source_path: asset.source_path.to_string_lossy().to_string(),
        output_rel_path: asset.output_rel_path.to_string_lossy().to_string(),
        public_url: asset.public_url.clone(),
        content_type: from_path(&asset.source_path)
            .first_or_octet_stream()
            .to_string(),
    })
}

fn materialize_image_variants(
    config: &AppConfig,
    asset: &AssetCandidate,
) -> AppResult<Option<ImageMaterialization>> {
    if let Some(image) =
        materialize_scaled_image_variants(config, asset, "webp", "image/webp", ["-quality", "78"])?
    {
        return Ok(Some(image));
    }
    materialize_scaled_image_variants(config, asset, "jpg", "image/jpeg", ["-q:v", "5"])
}

fn materialize_scaled_image_variants<const N: usize>(
    config: &AppConfig,
    asset: &AssetCandidate,
    extension: &str,
    content_type: &str,
    encoder_args: [&str; N],
) -> AppResult<Option<ImageMaterialization>> {
    let widths = [480_u32, 960, 1440];
    let mut records = Vec::new();
    let mut srcset_parts = Vec::new();
    let base = output_stem(asset)?;

    for width in widths {
        let rel_path = PathBuf::from(format!("{base}-{width}w.{extension}"));
        let destination = config.generated_assets_dir.join(&rel_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let scale = format!("scale='min({width},iw)':-2");
        let mut args = vec![
            "-y".into(),
            "-i".into(),
            asset.source_path.to_string_lossy().to_string(),
            "-vf".into(),
            scale,
        ];
        args.extend(encoder_args.iter().map(|arg| (*arg).to_string()));
        args.push(destination.to_string_lossy().to_string());
        let ok = generated_is_fresh(&asset.source_path, &destination) || run_ffmpeg(args);
        if !ok {
            return Ok(None);
        }
        let public_url = public_url_for(&rel_path);
        srcset_parts.push(format!("{public_url} {width}w"));
        records.push(record_for_generated(
            asset,
            &rel_path,
            &public_url,
            content_type,
        ));
    }

    let fallback_url = records
        .last()
        .map(|record| record.public_url.clone())
        .unwrap_or_else(|| asset.public_url.clone());
    Ok(Some(ImageMaterialization {
        records,
        image: ResponsiveImage {
            fallback_url,
            srcset: srcset_parts.join(", "),
            source_type: content_type.into(),
        },
    }))
}

fn materialize_video_variants(
    config: &AppConfig,
    asset: &AssetCandidate,
) -> AppResult<Option<VideoMaterialization>> {
    let base = output_stem(asset)?;
    let video_rel = PathBuf::from(format!("{base}-720p.mp4"));
    let video_destination = config.generated_assets_dir.join(&video_rel);
    if let Some(parent) = video_destination.parent() {
        fs::create_dir_all(parent)?;
    }

    let ok = generated_is_fresh(&asset.source_path, &video_destination)
        || run_ffmpeg(vec![
            "-y".into(),
            "-i".into(),
            asset.source_path.to_string_lossy().to_string(),
            "-vf".into(),
            "scale='min(1280,iw)':-2".into(),
            "-c:v".into(),
            "libx264".into(),
            "-preset".into(),
            "veryfast".into(),
            "-crf".into(),
            "28".into(),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            "96k".into(),
            "-movflags".into(),
            "+faststart".into(),
            video_destination.to_string_lossy().to_string(),
        ]);
    if !ok {
        return Ok(None);
    }

    let video_url = public_url_for(&video_rel);
    let mut records = vec![record_for_generated(
        asset,
        &video_rel,
        &video_url,
        "video/mp4",
    )];

    let poster = materialize_video_poster(config, asset, &base)?;
    let poster_url = poster.as_ref().map(|record| record.public_url.clone());
    if let Some(record) = poster {
        records.push(record);
    }

    Ok(Some(VideoMaterialization {
        records,
        video: ProcessedVideo {
            video_url,
            video_type: "video/mp4".into(),
            poster_url,
        },
    }))
}

fn replace_image_html(html: &str, original_url: &str, image: &ResponsiveImage) -> String {
    let re = Regex::new(&format!(
        r#"<img src="{}" alt="([^"]*)"\s*/?>"#,
        regex::escape(original_url)
    ))
    .expect("valid image replacement regex");
    re.replace_all(html, |caps: &regex::Captures| {
        let alt = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        format!(
            r#"<picture class="responsive-image"><source type="{}" srcset="{}" sizes="(max-width: 980px) calc(100vw - 48px), 900px"><img src="{}" alt="{}" loading="lazy" decoding="async"></picture>"#,
            image.source_type, image.srcset, image.fallback_url, alt
        )
    })
    .to_string()
}

fn materialize_video_poster(
    config: &AppConfig,
    asset: &AssetCandidate,
    base: &str,
) -> AppResult<Option<AssetRecord>> {
    if let Some(record) = materialize_video_poster_with_format(
        config,
        asset,
        base,
        "webp",
        "image/webp",
        ["-quality", "80"],
    )? {
        return Ok(Some(record));
    }
    materialize_video_poster_with_format(config, asset, base, "jpg", "image/jpeg", ["-q:v", "5"])
}

fn materialize_video_poster_with_format<const N: usize>(
    config: &AppConfig,
    asset: &AssetCandidate,
    base: &str,
    extension: &str,
    content_type: &str,
    encoder_args: [&str; N],
) -> AppResult<Option<AssetRecord>> {
    let poster_rel = PathBuf::from(format!("{base}-poster.{extension}"));
    let poster_destination = config.generated_assets_dir.join(&poster_rel);
    let mut args = vec![
        "-y".into(),
        "-ss".into(),
        "00:00:01".into(),
        "-i".into(),
        asset.source_path.to_string_lossy().to_string(),
        "-frames:v".into(),
        "1".into(),
        "-vf".into(),
        "scale='min(1280,iw)':-2".into(),
    ];
    args.extend(encoder_args.iter().map(|arg| (*arg).to_string()));
    args.push(poster_destination.to_string_lossy().to_string());

    let poster_ok = generated_is_fresh(&asset.source_path, &poster_destination) || run_ffmpeg(args);
    if !poster_ok {
        return Ok(None);
    }
    let url = public_url_for(&poster_rel);
    Ok(Some(record_for_generated(
        asset,
        &poster_rel,
        &url,
        content_type,
    )))
}

fn replace_video_html(html: &str, original_url: &str, video: &ProcessedVideo) -> String {
    let re = Regex::new(&format!(
        r#"(?s)<video class="video-player-media" preload="none" playsinline data-video-src="{}" data-video-type="[^"]*" data-video-key="{}">(?P<body>\s*)<source data-src="{}" type="[^"]+">"#,
        regex::escape(original_url),
        regex::escape(original_url),
        regex::escape(original_url)
    ))
    .expect("valid video replacement regex");
    let poster = video
        .poster_url
        .as_ref()
        .map(|url| format!(r#" poster="{url}""#))
        .unwrap_or_default();
    re.replace_all(html, |caps: &regex::Captures| {
        let body = caps.name("body").map(|m| m.as_str()).unwrap_or("\n");
        format!(
            r#"<video class="video-player-media" preload="none" playsinline{} data-video-src="{}" data-video-type="{}" data-video-key="{}">{}<source data-src="{}" type="{}">"#,
            poster, video.video_url, video.video_type, original_url, body, video.video_url, video.video_type
        )
    })
    .to_string()
}

fn output_stem(asset: &AssetCandidate) -> AppResult<String> {
    asset
        .output_rel_path
        .file_stem()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
        .ok_or_else(|| AppError::BadRequest("invalid generated asset filename".into()))
}

fn public_url_for(rel_path: &Path) -> String {
    format!("/assets/{}", rel_path.to_string_lossy())
}

fn record_for_generated(
    asset: &AssetCandidate,
    output_rel_path: &Path,
    public_url: &str,
    content_type: &str,
) -> AssetRecord {
    AssetRecord {
        source_path: asset.source_path.to_string_lossy().to_string(),
        output_rel_path: output_rel_path.to_string_lossy().to_string(),
        public_url: public_url.to_string(),
        content_type: content_type.to_string(),
    }
}

fn is_image(content_type: &str) -> bool {
    content_type.starts_with("image/") && !matches!(content_type, "image/svg+xml" | "image/gif")
}

fn is_video(content_type: &str) -> bool {
    content_type.starts_with("video/")
}

fn command_available(command: &str) -> bool {
    Command::new(command)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn generated_is_fresh(source: &Path, destination: &Path) -> bool {
    let Ok(source_modified) = fs::metadata(source).and_then(|metadata| metadata.modified()) else {
        return false;
    };
    let Ok(destination_modified) =
        fs::metadata(destination).and_then(|metadata| metadata.modified())
    else {
        return false;
    };
    destination_modified >= source_modified
}

fn run_ffmpeg(args: Vec<String>) -> bool {
    Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
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

    #[test]
    fn applies_responsive_image_manifest_to_html() {
        let mut manifest = MediaManifest::default();
        manifest.images.insert(
            "/assets/original.png".into(),
            ResponsiveImage {
                fallback_url: "/assets/original-1440w.jpg".into(),
                srcset: "/assets/original-480w.jpg 480w, /assets/original-960w.jpg 960w, /assets/original-1440w.jpg 1440w".into(),
                source_type: "image/jpeg".into(),
            },
        );

        let html = apply_media_optimizations(
            r#"<p><img src="/assets/original.png" alt="diagram"></p>"#,
            &manifest,
        );

        assert!(html.contains(r#"<picture class="responsive-image">"#));
        assert!(html.contains(r#"type="image/jpeg""#));
        assert!(html.contains(r#"srcset="/assets/original-480w.jpg 480w"#));
        assert!(html.contains(r#"loading="lazy" decoding="async""#));
        assert!(!html.contains(r#"<img src="/assets/original.png""#));
    }

    #[test]
    fn applies_lazy_processed_video_manifest_to_html() {
        let mut manifest = MediaManifest::default();
        manifest.videos.insert(
            "/assets/original.mp4".into(),
            ProcessedVideo {
                video_url: "/assets/original-720p.mp4".into(),
                video_type: "video/mp4".into(),
                poster_url: Some("/assets/original-poster.jpg".into()),
            },
        );

        let html = apply_media_optimizations(
            r#"<figure class="video-player" data-video-player>
                <div class="video-player-frame">
                    <video class="video-player-media" preload="none" playsinline data-video-src="/assets/original.mp4" data-video-type="video/mp4" data-video-key="/assets/original.mp4">
                        <source data-src="/assets/original.mp4" type="video/mp4">
                        fallback
                    </video>
                </div>
            </figure>"#,
            &manifest,
        );

        assert!(html.contains(r#"poster="/assets/original-poster.jpg""#));
        assert!(html.contains(r#"data-video-src="/assets/original-720p.mp4""#));
        assert!(html.contains(r#"data-video-key="/assets/original.mp4""#));
        assert!(html.contains(r#"<source data-src="/assets/original-720p.mp4" type="video/mp4">"#));
        assert!(!html.contains(r#"data-video-src="/assets/original.mp4""#));
    }
}
