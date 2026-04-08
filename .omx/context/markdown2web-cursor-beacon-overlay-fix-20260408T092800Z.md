# markdown2web cursor beacon overlay fix context snapshot

- Timestamp (UTC): 2026-04-08T09:28:00Z
- Task slug: markdown2web-cursor-beacon-overlay-fix

## Task statement

Refine the new cursor beacon so it no longer obscures text within its area.

## Desired outcome

- Cursor beacon remains elegant and visible
- Text beneath it stays readable
- The beacon behaves more like a ring/locator than a translucent blocking disc

## Known facts / evidence

- Current cursor beacon lives in `templates/base.html`, `static/js/site.js`, and `static/css/app.css`
- The current CSS uses a semi-opaque filled background plus backdrop blur, which can visually block text

## Constraints

- Keep the overall modern tech feel
- Preserve reduced-motion and fine-pointer safeguards
- Minimize change scope

## Likely touchpoints

- `static/css/app.css`
- `tests/app_flow.rs`
