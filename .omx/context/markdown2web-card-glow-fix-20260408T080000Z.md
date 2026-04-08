# markdown2web card glow fix context snapshot

- Timestamp (UTC): 2026-04-08T08:00:00Z
- Task slug: markdown2web-card-glow-fix

## Task statement

Fix the pointer glow behavior on cards:

1. While scrolling without moving the mouse, the glow must remain around the actual cursor
2. Leaving card bounds should not abruptly kill the glow; it should stay clipped within the card
3. Glow color should stay soft, background-adaptive, and not wash out content

## Desired outcome

- Pointer glow tracks the real cursor during pointermove, scroll, and resize
- Glow fades naturally by clipping, not hover on/off snapping
- Content remains readable

## Known facts / evidence

- Existing interaction system lives in `static/js/site.js` and `static/css/app.css`
- Current project verification baseline is green

## Constraints

- Preserve the user-approved background and palette direction
- Keep interactions smooth and modern
- Maintain reduced-motion safety

## Likely touchpoints

- `static/js/site.js`
- `static/css/app.css`
- `tests/app_flow.rs`
