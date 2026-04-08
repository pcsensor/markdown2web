# Repository Guidelines

## Project Structure & Module Organization
`src/` holds the Rust application: `main.rs` boots the Axum server, `app.rs` wires routes/state, and feature folders such as `build/`, `content/`, `search/`, `store/`, and `web/` contain pipeline, parsing, persistence, and HTTP logic. Askama templates live in `templates/` (`admin/` for dashboard views). Static assets are in `static/css`, `static/js`, and `static/favicon.svg`. Markdown source-of-truth content lives in `content/notes` and `content/assets`; generated public output is written to `generated/site`. Integration tests live in `tests/`, and SQLite state is stored under `data/`.

## Build, Test, and Development Commands
- `cargo run` — start the local site and admin server on `127.0.0.1:3000`.
- `cargo check` — fast compile verification for Rust code.
- `cargo test` — run unit and integration tests (`tests/app_flow.rs`).
- `cargo fmt --all` — apply standard Rust formatting.
- `cargo clippy --all-targets --all-features` — optional lint pass before opening a PR.
- `node --check static/js/site.js` — quick syntax check for the hand-written frontend script.

## Coding Style & Naming Conventions
Follow `rustfmt` defaults (4-space indentation, trailing commas where formatted). Use `snake_case` for functions/modules, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep Axum handlers thin; push parsing, storage, and build logic into their feature modules. Reuse existing template and CSS patterns before adding new abstractions. Name templates and static files with lowercase kebab-case where practical.

## Testing Guidelines
Prefer focused unit tests beside the module they cover and integration tests in `tests/*.rs` for end-to-end flows. Name tests for observable behavior, e.g. `rebuild_updates_generated_site`. Cover content ingestion, link rewriting, admin auth, and public rendering whenever behavior changes. Run `cargo test` before every commit; use `cargo check` for faster iteration between edits.

## Commit & Pull Request Guidelines
Recent history uses short, imperative subjects (`粒子`, `update password`), but contributors should follow the repo’s Lore commit protocol: an intent-first subject, brief rationale, then trailers such as `Constraint:`, `Confidence:`, and `Tested:`. PRs should include a concise summary, linked issue or task, verification commands run, and screenshots/GIFs for template or styling changes. Call out config or data-shape changes explicitly.

## Security & Configuration Tips
Do not commit real credentials or production `data/app.db` contents. Configure secrets and runtime values through `M2W_*` environment variables in `README.md`, especially admin credentials, host/port, content paths, and upload limits.
