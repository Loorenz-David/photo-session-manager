---
plan: P23_shape_on_arrival
role: fix
round: 2
state: DONE
date: 2026-10-29
actor: codex
---

# P23 fix 1 handoff

## Finding → what changed → files

- **F1 — Untouched Capture One crop ratio is `null`**: the untouched check now accepts `null` and `undefined` alongside `""` and `"Unconstrained"`. The C1 harness now explicitly returns `null` for the untouched variant and retains explicit `""` and `"Unconstrained"` cases. Files: `crates/integrations/src/editing/capture_one/jxa/apply_crop.js`; `crates/testkit/tests/p23_shape_on_arrival.rs`.
- **F2 — Capture One's decoded maximum rectangle reports half-dimensions in `width`/`height`**: the size check now reads the true dimensions as `x + width` and `y + height`; malformed or non-numeric dimensions still throw. The C1 harness uses the observed object `{x:3000,y:1993,width:2981,height:1994}` against the 5981×3987 crop. Files: `crates/integrations/src/editing/capture_one/jxa/apply_crop.js`; `crates/testkit/tests/p23_shape_on_arrival.rs`.

## Divergences

None. Both corrections were implemented as written. No owner decisions are needed.

## Files changed

- `crates/integrations/src/editing/capture_one/jxa/apply_crop.js`
- `crates/testkit/tests/p23_shape_on_arrival.rs`
- `docs/implementation/master_plan.md` (P23 tracker row)
- `docs/implementation/plans/P23_shape_on_arrival.md` (Review log)
- `docs/implementation/handoffs/implementer/handoff_P23_shape_on_arrival_fix_r2.md`

## Checkpoint SHA

`5d2210d18ea7597d0948e4f140548bfc070c0542` — `CHECKPOINT (not approved): fix P23 crop shape handling` (implementation, harness, tracker and Review log; handoff remains for the pipeline runner to commit).

## Stamp

- `cargo test --workspace` — passed; P23 suite 11/11. Includes updated C1(a) null and empty-string cases, C1(b) fixed-ratio and `Unconstrained` cases, and C1(c) malformed maximum checks.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `cd apps/desktop && npm run typecheck` — passed.
- `cd apps/desktop && npm test` — passed; 17 files, 127 tests.
- `git diff --check` — passed before commit.

The C6(a) live Capture One probe was not repeated in this fix session; the fix is driven by the reviewer's recorded live observations. Re-review should run it as specified in the review handoff.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
