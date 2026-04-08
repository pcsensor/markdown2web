# markdown2web note sidebar spacing + hover context snapshot

- Timestamp (UTC): 2026-04-08T08:44:00Z
- Task slug: markdown2web-note-sidebar-spacing-hover

## Task statement

Improve the article-page sidebar:

1. Increase the vertical gap between the "目录"/"Backlinks" module titles and their first list items
2. When hovering a TOC item, make that heading text slightly larger and bolder until pointer leave

## Desired outcome

- Sidebar sections breathe more
- TOC hover state feels clearer and more responsive
- No regressions to existing sidebar glow behavior

## Known facts / evidence

- Sidebar markup lives in `templates/note.html`
- Relevant styles live in `static/css/app.css`
- TOC and backlinks lists already use `.toc-list` and `.link-list`

## Constraints

- Keep the effect subtle
- Preserve readability and current visual style

## Likely touchpoints

- `static/css/app.css`
- `tests/app_flow.rs`
