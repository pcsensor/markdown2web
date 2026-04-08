# markdown2web visual polish context snapshot

- Timestamp (UTC): 2026-04-08T07:12:00Z
- Task slug: markdown2web-visual-polish

## Task statement

Add a stronger visual design pass to the existing markdown2web site:

1. Card hover scale with subtle edge highlight
2. More styling throughout the site
3. Further beautify the UI with icons, decorative backgrounds, and improved polish

## Desired outcome

- Cleaner, more modern, more premium-looking UI
- Improved motion and visual hierarchy
- Decorative details without losing readability
- No functional regressions

## Known facts / evidence

- The project is already implemented and passes `cargo check` / `cargo test`.
- Current styling is minimal and content-first.
- Current UI files mainly live in:
  - `static/css/app.css`
  - `templates/base.html`
  - `templates/home.html`
  - `templates/note.html`
  - `templates/notes.html`
  - `templates/search.html`
  - `templates/tag.html`
  - `templates/admin/*`

## Constraints

- Keep the site clean and modern.
- Preserve SSR/templates; do not introduce a heavy frontend stack.
- Verify after changes.

## Unknowns / open questions

- No user-supplied visual reference exists, so polish is guided by the current product intent rather than strict visual matching.

## Likely codebase touchpoints

- `static/css/app.css`
- `templates/base.html`
- `templates/home.html`
- `templates/note.html`
- `templates/notes.html`
- `templates/search.html`
- `templates/tag.html`
- `templates/admin/dashboard.html`
- `templates/admin/login.html`
- `templates/admin/note_edit.html`
- `static/*`
