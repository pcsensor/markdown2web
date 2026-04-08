# markdown2web autopilot context snapshot

- Timestamp (UTC): 2026-04-08T06:34:30Z
- Task slug: markdown2web-autopilot

## Task statement

Implement the planned markdown2web project as a Rust full-stack application based on the approved PRD and test spec.

## Desired outcome

- Working Rust web app
- Public Markdown-powered website
- Admin authentication and note management
- Filesystem-backed Markdown/content source of truth
- Upload/update note flows
- Automatic rebuilds when notes/assets change
- Working note-to-note links and asset embedding
- Tests and verification evidence

## Known facts / evidence

- Repository started essentially empty except `.omx/` runtime files.
- Existing plan artifacts:
  - `.omx/plans/prd-markdown2web-2026-04-08.md`
  - `.omx/plans/test-spec-markdown2web-2026-04-08.md`
- Requested stack direction: Rust backend/admin, clean modern site, dynamic upload/update, auto rebuild on note changes.

## Constraints

- Must follow workspace AGENTS.md rules.
- Prefer single Rust monolith.
- No new dependencies without clear implementation need.
- Must verify before claiming completion.

## Unknowns / open questions

- Final dependency resolution may require network access for cargo if crates are not cached locally.
- Scope may need practical MVP interpretation when plan items exceed first-pass implementation capacity.

## Likely codebase touchpoints

- `Cargo.toml`
- `src/**`
- `templates/**`
- `static/**`
- `content/**`
- `tests/**`
