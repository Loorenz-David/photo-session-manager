---
plan: P49_capture_one_alongside
role: fix
round: 3
state: DONE
date: 2026-11-07
actor: codex
---

# P49 fix 2 handoff

## Finding → what changed → files

**F2 (blocking, B1; production).** The style-eligibility SQL comparison returned SQL NULL for an unmarked RAW, and reading that value as a Rust `bool` prevented preview issuance. Changed the predicate to `style_applied_at IS ?2 AND rendered_gen=0`; NULL now means ineligible, while a pending marker still matches. Added `tethered_capture_with_style_renders_without_applying_script_style`, which delivers a tethered file through the watcher with a configured style, checks that its variant render is issued and no script-style operation occurs, and verifies the persisted style marker is SQL NULL. The existing F1 retouch regression remains green.

Files: `crates/infra/src/sqlite/store.rs`; `crates/testkit/tests/p49_capture_one_alongside.rs`.

The requested pre-P49/unmarked case is represented by the watcher-created RAW with a NULL `style_applied_at`; the test reads that column directly so the Store API's sentinel masking cannot hide a pending marker.

## Divergences

None. SQLite `IS` is the authorized NULL-safe correction. No changes outside the finding's perimeter, other than the required tracker and Review log updates.

The red baseline was not captured before the production edit. The first targeted run happened after the SQL correction; it exposed a test timing issue (the watcher path needed one tick), which was fixed before the passing P49 run. No mutation ledger or named mutations were used, per `flow.md` §4.

## Files changed

- `crates/infra/src/sqlite/store.rs` — NULL-safe eligibility predicate.
- `crates/testkit/tests/p49_capture_one_alongside.rs` — tethered and unmarked RAW boundary case.
- `docs/implementation/master_plan.md` — P49 tracker row set to `IMPLEMENTED` with fix 2 note.
- `docs/implementation/plans/P49_capture_one_alongside.md` — fix 2 Review log entry.

## Checkpoint SHA

`3963477c14f0a7e3e47412157092a016d9b0d05d` — `CHECKPOINT (not approved): fix P49 style eligibility NULL read`.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace --quiet` | PASS; P49 21/21 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm run typecheck` | PASS |
| `cd apps/desktop && npm test -- --maxWorkers=1` | PASS, 369/369 |
| `git diff --check` | PASS before checkpoint |

No live Capture One or camera probe was run. The test uses `TestApp::with_fake_document` and fake Capture One state.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
