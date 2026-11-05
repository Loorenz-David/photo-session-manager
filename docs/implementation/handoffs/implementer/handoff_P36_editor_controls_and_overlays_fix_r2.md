---
plan: P36_editor_controls_and_overlays
role: fix
round: 2
state: DONE
date: 2026-11-05
actor: codex
---

# P36 fix 1

## Finding → what changed → files

- **F1 (C7(a), C7(b), B2):** Focus the Shape select before changing it and the slider before pointer interaction. Send `x` to `document.activeElement` after each handoff, so the tests model keyboard input at the control that held focus. The tests still assert the editor receives the shortcut and the photo orientation changes. Files: `apps/desktop/src/features/compose/__tests__/p36_straighten.test.tsx`.
- **F2 (C1(b), B2):** From the initial photo 1 state, click tile 2 in the unfolded panel and assert the preview is photo 2. This makes the panel's photo selection observable. File: `apps/desktop/src/features/compose/__tests__/p36_strip_overlays.test.tsx`.

No production code changed. No other criterion or behavior was changed.

## Divergences

None. Both correction clauses were implemented as written.

## Files changed

- `apps/desktop/src/features/compose/__tests__/p36_straighten.test.tsx`
- `apps/desktop/src/features/compose/__tests__/p36_strip_overlays.test.tsx`
- `docs/implementation/master_plan.md` — P36 tracker row → `IMPLEMENTED`, note: fix 1.
- `docs/implementation/plans/P36_editor_controls_and_overlays.md` — Review log.
- `docs/implementation/handoffs/implementer/handoff_P36_editor_controls_and_overlays_fix_r2.md` — this handoff.

## Checkpoint SHA

This handoff is included in the checkpoint commit, so its SHA cannot be embedded without changing that SHA. The final session report gives the checkpoint SHA.

## Stamp

- `cargo test --workspace` — exit 0; all workspace tests and doctests passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo fmt --all -- --check` — exit 0.
- `cd apps/desktop && npm run typecheck` — exit 0.
- `cd apps/desktop && npm test` — exit 0; 46 files, 339 tests.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
