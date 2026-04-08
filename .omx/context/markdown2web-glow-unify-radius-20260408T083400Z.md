# markdown2web glow unify + radius fix context snapshot

- Timestamp (UTC): 2026-04-08T08:34:00Z
- Task slug: markdown2web-glow-unify-radius

## Task statement

Fix the pointer glow so it no longer shows a gray center with cyan-blue outer banding. Use one unified soft color and reduce the glow radius to about half of the current size.

## Desired outcome

- Single unified glow tone
- Softer visual result with no obvious color layering
- Glow radius reduced by roughly 50%

## Known facts / evidence

- Glow is rendered in `static/css/app.css` via `.interactive-card::after`
- Current implementation uses two radial gradients with different colors/sizes
- The user explicitly wants a single-color feel and a smaller radius

## Constraints

- Preserve current interaction model and pointer tracking
- Keep content readable
- Keep change minimal and low-risk

## Likely touchpoints

- `static/css/app.css`
- `tests/app_flow.rs`
