# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

markdown2web 是一个 Rust 全栈应用，将 Markdown 笔记转换为 SSR 渲染的网站，附带管理后台。技术栈：Axum (Web)、Askama (模板)、SQLite/rusqlite (存储)、Comrak (Markdown 渲染)、Notify (文件监听)。

## 常用命令

```bash
cargo run          # 启动开发服务器 (默认 http://127.0.0.1:3000)
cargo check        # 类型检查
cargo test         # 运行全部测试
cargo test -- --test-name  # 运行单个测试
```

## 环境变量

所有配置通过 `M2W_` 前缀的环境变量设置（见 `src/config.rs`）。默认值可直接 `cargo run` 使用，无需任何环境变量。

## 架构

### 构建管线 (核心数据流)

```
content/notes/*.md → discover_notes → rewrite_markdown (链接/资源解析) → render_markdown (Comrak) → build_site_data (图关系) → SiteData (内存)
```

1. **`store::filesystem::discover_notes`** — 扫描 `content/notes/` 目录，解析 YAML front matter，生成 `NoteSource` 列表
2. **`content::links::rewrite_markdown`** — 处理 `[[Wiki Link]]`、相对 `.md` 链接、图片/资源引用，解析为站内 URL
3. **`content::markdown::render_markdown`** — 使用 Comrak 渲染 HTML（支持数学公式、语法高亮、表格等扩展）
4. **`content::graph::build_site_data`** — 构建标签索引、反向链接图、有序列表
5. **`build::pipeline::BuildService::rebuild`** — 串联以上步骤，更新 `SiteData`（`Arc<RwLock>`），持久化构建缓存

### 模块职责

- **`app`** — `AppState` 初始化与 Axum 路由定义
- **`config`** — `AppConfig`，从环境变量读取配置
- **`content`** — 数据模型 (`Note`, `SiteData`, `FrontMatter`) 及内容处理（markdown、links、assets、graph）
- **`build`** — 构建管线 (`pipeline`)、文件哈希缓存 (`cache`)、文件系统监听 (`watcher`)
- **`store`** — SQLite 数据库 (`sqlite`) 和文件系统操作 (`filesystem`)
- **`web`** — 路由处理：公开页面 (`public`)、管理后台 (`admin`)、认证 (`auth`)
- **`search`** — 搜索索引
- **`error`** — 统一错误类型 `AppError`，实现 `IntoResponse`

### 关键设计

- **SiteData 全量内存**：所有笔记数据在 `rebuild` 时全量加载到 `Arc<RwLock<SiteData>>`，Web 请求直接读取，无需查库
- **增量检测**：`BuildCache` 通过文件内容哈希判断变更，但 rebuild 仍全量执行
- **Watcher 防抖**：文件监听后 800ms 去抖，合并多次变更为一次 rebuild
- **链接解析**：`LinkLookup` 支持按 slug、标题、文件名、别名多路查找，解析失败标记为 broken link
- **认证**：Argon2 密码哈希，session token 存 SQLite，通过 cookie 验证

### 目录约定

- `content/notes/` — Markdown 笔记源文件
- `content/assets/` — 共享资源文件
- `generated/site/assets/` — 构建产出的公开资源（哈希前缀命名）
- `data/app.db` — SQLite 数据库
- `templates/` — Askama HTML 模板
- `static/` — 静态文件 (JS, favicon)

### Front Matter 格式

```yaml
---
title: 标题
slug: url-slug
summary: 摘要
tags: [tag1, tag2]
status: published  # 或 draft
aliases: [别名1]
---
```

### Rust 版本

使用 Rust edition 2024。
