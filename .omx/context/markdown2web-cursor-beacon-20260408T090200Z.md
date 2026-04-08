# markdown2web cursor beacon context snapshot

- Timestamp (UTC): 2026-04-08T09:02:00Z
- Task slug: markdown2web-cursor-beacon

## Task statement

Remove the existing pointer glow effect and replace it with a more elegant, modern, tech-feeling pointer locator interaction.

## Desired outcome

- No more card-local pointer glow
- A refined cursor-locator treatment that feels modern and less visually noisy
- Preserve strong interaction feedback on interactive elements
- Keep the approved background and palette direction

## Known facts / evidence

- Current system uses card-local radial glow in `static/css/app.css` and pointer-position tracking in `static/js/site.js`.
- The user explicitly dislikes the current glow and wants a more elegant replacement informed by more modern interaction patterns.

## Constraints

- Keep the result elegant and tech-flavored, not flashy
- Respect reduced-motion preferences
- Favor performant transform/opacity-based motion

## Likely touchpoints

- `templates/base.html`
- `static/js/site.js`
- `static/css/app.css`
- `tests/app_flow.rs`
