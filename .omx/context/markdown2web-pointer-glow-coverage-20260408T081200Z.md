# markdown2web pointer glow coverage context snapshot

- Timestamp (UTC): 2026-04-08T08:12:00Z
- Task slug: markdown2web-pointer-glow-coverage

## Task statement

Expand pointer-glow coverage so that:

1. Home page recent-content cards and the Build Signal card clearly participate
2. Article page TOC and Backlinks cards clearly participate
3. Other suitable interactive cards/components also receive the glow treatment

## Desired outcome

- Consistent pointer-glow behavior across major card-like interactive surfaces
- No overuse on tiny/decorative-only elements
- No regressions in current motion or rendering features

## Known facts / evidence

- Current pointer glow is driven by `.interactive-card` in `static/js/site.js` and `static/css/app.css`
- Core requested cards already use card/panel/sidebar structures and can be explicitly reinforced in templates/tests
- Some sub-card surfaces (metrics/stats) can reasonably benefit from the same treatment

## Constraints

- Preserve the approved palette/background
- Keep the glow on appropriate components only
- Maintain readability and reduced-motion behavior

## Likely touchpoints

- `templates/home.html`
- `templates/note.html`
- `templates/admin/dashboard.html`
- `static/css/app.css`
- `tests/app_flow.rs`
