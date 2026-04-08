# markdown2web interaction + math/highlight context snapshot

- Timestamp (UTC): 2026-04-08T07:31:00Z
- Task slug: markdown2web-interaction-math-highlight

## Task statement

Enhance the existing site with richer modern interactions and fix Markdown rendering gaps:

1. Add more advanced, less dry interactions
2. Support LaTeX math formulas in notes
3. Add syntax highlighting for mainstream code blocks

## Desired outcome

- More polished and responsive interactions without changing the approved background/palette direction
- Notes render `$$...$$` formulas correctly
- Fenced code blocks render with syntax highlighting
- No regressions in site behavior

## Known facts / evidence

- The user is satisfied with the current background and palette.
- A test note now exists at `content/notes/测试文件.md` containing display math and a Rust code block.
- Current renderer uses `comrak` without math enabled and without code-fence syntax highlighting plugins.
- Current public JS is effectively absent.

## Constraints

- Keep the UI modern and content-first.
- Avoid heavy frontend frameworks.
- Preserve accessibility and respect reduced-motion preferences.

## Likely touchpoints

- `Cargo.toml`
- `src/content/markdown.rs`
- `templates/base.html`
- `templates/home.html`
- `templates/notes.html`
- `templates/search.html`
- `templates/tag.html`
- `templates/note.html`
- `templates/admin/*.html`
- `static/css/app.css`
- `static/js/site.js`
- `tests/app_flow.rs`
