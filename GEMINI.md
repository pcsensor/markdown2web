# Project Overview: markdown2web

`markdown2web` is a full-stack website engine written in Rust that transforms Markdown files into a feature-rich, SEO-friendly website. It features Server-Side Rendering (SSR), an administrative dashboard, real-time content watching, and advanced media optimization.

## Core Technologies
- **Language:** Rust (2024 edition)
- **Web Framework:** [Axum](https://github.com/tokio-rs/axum)
- **Templating:** [Askama](https://github.com/djc/askama) (type-safe Jinja-like templates)
- **Database:** SQLite via `rusqlite`
- **Markdown Processing:** [Comrak](https://github.com/kivikakk/comrak) (GFM compatible)
- **Media Processing:** FFmpeg (optional, for image/video optimization)
- **File Watching:** `notify` for automatic site rebuilding

## Key Features
- **Public Site:** SSR rendered notes with support for tags, categories, and full-text search.
- **Admin Panel:** Secure dashboard for managing notes, uploading assets, and user management.
- **Content Source:** `content/notes` for Markdown and `content/assets` for raw resources.
- **Media Optimization:** Automatic generation of responsive images and compressed video (requires FFmpeg).
- **Interactive Elements:** Support for LaTeX (math), syntax highlighting, reading progress, and video "danmaku" (floating comments).
- **Annotations:** Built-in support for user annotations on notes.

---

## Building and Running

### Prerequisites
- **Rust:** Latest stable version recommended.
- **FFmpeg:** (Optional) Install for media optimization capabilities.

### Commands
- **Start Development Server:**
  ```bash
  cargo run
  ```
  Defaults to `http://127.0.0.1:3000`. Admin panel at `/admin`.
- **Run Tests:**
  ```bash
  cargo test
  ```
- **Check Code:**
  ```bash
  cargo check
  ```
- **Lint Code:**
  ```bash
  # Standard Rust linting
  cargo clippy
  ```

---

## Directory Structure

- `src/`: Core logic and application code.
  - `app.rs`: Application state and router configuration.
  - `build/`: Site building pipeline and file watcher.
  - `content/`: Markdown parsing, front-matter handling, and asset management.
  - `store/`: Data persistence (SQLite and filesystem helpers).
  - `web/`: HTTP handlers grouped by functionality (public, admin, auth, account).
- `content/`: Source of truth for the site.
  - `notes/`: Markdown files.
  - `assets/`: Raw images, videos, and other files.
- `templates/`: Askama HTML templates.
- `static/`: Client-side assets (CSS, JS, favicons).
- `generated/site/`: Output directory for built assets and processed media.
- `data/`: Location for the SQLite database (`app.db`).

---

## Development Conventions

### Configuration
Configuration is managed via environment variables. See `.env.example` for available options.
- `M2W_WATCH_ENABLED`: Set to `true` (default) to enable automatic rebuilds on file changes.
- `M2W_ADMIN_USERNAME`/`M2W_ADMIN_PASSWORD`: Default credentials for the admin panel.

### Security
- **CSRF Protection:** State-changing requests (POST, PUT, DELETE) require a CSRF token.
- **Authentication:** Admin routes are protected; public users can register for annotations/danmaku features.
- **Input Validation:** Slugs and filenames are strictly validated to prevent path traversal.

### Content Format
Notes should include YAML front-matter:
```yaml
---
title: Note Title
slug: unique-slug
summary: Short description
tags: [tag1, tag2]
status: published # or draft
---
# Content starts here
```

### Media Usage
- **Images:** Standard Markdown `![alt](path)`.
- **Audio:** `#[label](path)` custom syntax.
- **Video:** `@[label](path)` custom syntax.
