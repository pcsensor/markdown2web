# markdown2web autopilot implementation plan

- Generated: 2026-04-08

## Implementation steps

1. Remove card-local pointer glow CSS and any obsolete glow assumptions/tests
2. Add a global cursor beacon layer driven by lightweight JS for fine pointers only
3. Preserve modern hover feedback on cards/buttons without pointer-local floodlighting
4. Verify with fresh tests/build and add coverage for the new cursor-beacon path
