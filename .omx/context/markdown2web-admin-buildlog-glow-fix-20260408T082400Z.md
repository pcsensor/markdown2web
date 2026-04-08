# markdown2web admin build-log glow fix context snapshot

- Timestamp (UTC): 2026-04-08T08:24:00Z
- Task slug: markdown2web-admin-buildlog-glow-fix

## Task statement

Fix the Admin dashboard so the pointer glow does not obscure content inside the "最近构建日志" area.

## Desired outcome

- Build-log text remains clearly readable
- Any retained glow styling no longer sits above the log content
- No regressions to other glow-enabled components

## Known facts / evidence

- The build log items currently use `interactive-card interactive-card-subtle`
- Their content is mostly plain text nodes inside `<li>`
- Generic glow layering relies on positioned child content being above the pseudo-element

## Constraints

- Preserve the rest of the current admin/dashboard styling
- Keep the fix minimal and low-risk

## Likely touchpoints

- `templates/admin/dashboard.html`
- `static/css/app.css`
- `tests/app_flow.rs`
