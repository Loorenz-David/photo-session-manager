---
plan: P35_composed_display_and_chrome
role: fix
round: 2
state: DONE
date: 2026-11-05
actor: codex
---

# Fix r2: P35 composed display and chrome

## Finding → what changed → files

**F1 (B1, production).** Added `right: auto; bottom: auto; margin: 0` to `.composed-frame img` in `apps/desktop/src/features/workspace/workspace.css`. The composed images no longer inherit the stage layer's right/bottom insets or auto margins, which had shifted the stage photo upward. As specified by the finding, no test was added because jsdom does not perform layout.

## Divergences

None.

## Files changed

- `apps/desktop/src/features/workspace/workspace.css`
- `docs/implementation/master_plan.md` (P35 tracker row)
- `docs/implementation/plans/P35_composed_display_and_chrome.md` (Review log)
- `docs/implementation/handoffs/implementer/handoff_P35_composed_display_and_chrome_fix_r2.md`

## Checkpoint SHA

HEAD after the checkpoint commit; resolve from `git log`.

## Stamp

- `cargo test --workspace`: passed, including `p35_split_any_photo`.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `npm test` in `apps/desktop`: 42 files, 320 tests passed.
- `npm run typecheck` in `apps/desktop`: passed.
- The checkpoint contains only the four paths listed above.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
