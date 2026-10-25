---
plan: P05_capture_arrival
role: fix
round: 2
state: DONE
date: 2026-10-25
actor: codex
---
# P05 capture arrival — fix 1 handoff

Implemented the routed F1 correction. Capture arrivals now retain a known RAW's existing photo row when its session changes, including during startup reconciliation. The C7(b) test moves a known capture out of Unassigned before reopening; import deduplication remains session-scoped.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Finding → what changed → files

- **F1 (B1, production).** `arrival_insert` now checks for an existing canonical RAW path across all sessions for `PhotoSource::Capture`, while `PhotoSource::Import` retains the `(session_id, raw_path)` check. C7(b) now captures a known RAW in Unassigned, assigns it to an item's session, and verifies reopening leaves exactly one row in that session while ignoring the CaptureOne cache file. Files: `crates/infra/src/sqlite/store.rs`, `crates/testkit/tests/p05_capture_arrival.rs`.

## Divergences

None. The correction was implemented as written; the store signature did not need to change.

## Files changed

- `crates/infra/src/sqlite/store.rs`
- `crates/testkit/tests/p05_capture_arrival.rs`
- `docs/implementation/master_plan.md` — P05 tracker row set to `IMPLEMENTED` (fix 1)
- `docs/implementation/plans/P05_capture_arrival.md` — fix 1 Review log entry
- `docs/implementation/handoffs/implementer/handoff_P05_capture_arrival_fix_r2.md` — this handoff

## Checkpoint SHA

`806ecaa` — `CHECKPOINT (not approved): fix P05 capture arrival dedupe`.

## Stamp

- `cargo test --workspace`: passed, 156 tests; P05 boundary suite 33/33.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `cd apps/desktop && npm run typecheck`: passed.
- `cd apps/desktop && npm test`: passed (no frontend test files).
- `git diff --check`: passed.
