use std::{fs, path::PathBuf};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::{
    config::AppConfig,
    content::{
        front_matter::{parse_front_matter, FrontMatter},
        markdown::slugify,
        NoteSource,
    },
    error::AppResult,
};

fn hash_string(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn discover_notes(config: &AppConfig) -> AppResult<Vec<NoteSource>> {
    let mut notes = Vec::new();
    for entry in WalkDir::new(&config.notes_dir)
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.into_path();
        let is_markdown = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "md" | "markdown"));
        if !is_markdown {
            continue;
        }
        let raw = fs::read_to_string(&path)?;
        let (front_matter, body) = parse_front_matter(&raw)?;
        let relative_path = path
            .strip_prefix(&config.notes_dir)
            .unwrap_or(&path)
            .to_path_buf();
        let title = front_matter.title.clone().unwrap_or_else(|| {
            relative_path
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or("Untitled")
                .replace('-', " ")
        });
        let slug = front_matter.slug.clone().unwrap_or_else(|| slugify(&title));
        let summary = front_matter.summary.clone().unwrap_or_else(|| {
            body.lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or_default()
                .chars()
                .take(160)
                .collect()
        });
        notes.push(NoteSource {
            source_path: path,
            relative_path,
            front_matter: FrontMatter {
                title: Some(title.clone()),
                slug: Some(slug.clone()),
                summary: Some(summary.clone()),
                tags: front_matter.tags.clone(),
                status: front_matter.status.clone(),
                aliases: front_matter.aliases.clone(),
                category: front_matter.category.clone(),
            },
            body,
            title,
            slug,
            summary,
            category: front_matter.category.clone(),
            tags: front_matter.tags,
            status: front_matter.status.unwrap_or_else(|| "published".into()),
            aliases: front_matter.aliases,
            hash: hash_string(&raw),
        });
    }
    notes.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(notes)
}

pub fn write_note(config: &AppConfig, slug: &str, contents: &str) -> AppResult<PathBuf> {
    let path = config.notes_dir.join(format!("{}.md", slug));
    fs::create_dir_all(&config.notes_dir)?;
    fs::write(&path, contents)?;
    Ok(path)
}

pub fn write_asset(config: &AppConfig, filename: &str, bytes: &[u8]) -> AppResult<PathBuf> {
    let path = config.assets_dir.join(filename);
    fs::create_dir_all(&config.assets_dir)?;
    fs::write(&path, bytes)?;
    Ok(path)
}

pub fn ensure_sample_content(config: &AppConfig) -> AppResult<()> {
    fs::create_dir_all(&config.notes_dir)?;
    let has_markdown = WalkDir::new(&config.notes_dir)
        .into_iter()
        .flatten()
        .any(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| matches!(ext, "md" | "markdown"))
        });
    if has_markdown {
        return Ok(());
    }

    let welcome = r#"---
title: Welcome to markdown2web
slug: welcome
summary: 项目启动后的欢迎页与说明。
tags: [intro, rust]
status: published
---
# Welcome to markdown2web

这是示例笔记。你现在可以：

- 浏览 [架构说明](architecture.md)
- 登录后台 `/admin`
- 上传或编辑 Markdown 文件

支持 Wiki Link：[[Architecture]]
"#;

    let architecture = r#"---
title: Architecture
slug: architecture
summary: markdown2web 的核心架构说明。
tags: [architecture, notes]
status: published
---
# Architecture

markdown2web 使用 **Rust + Axum + Askama + SQLite**。

## Content flow

`Markdown -> parse -> link rewrite -> HTML render -> site refresh`

返回到 [[Welcome to markdown2web]]。
"#;

    write_note(config, "welcome", welcome)?;
    write_note(config, "architecture", architecture)?;
    Ok(())
}
