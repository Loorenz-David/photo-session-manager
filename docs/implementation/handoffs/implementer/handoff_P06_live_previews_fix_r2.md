---
plan: P06_live_previews
role: fix
round: 2
state: DONE
date: 2026-10-25
actor: codex
---
# P06 live previews — fix r2

## Finding → what changed → files

- **F1:** A later row for an already-rendered RAW now receives the current preview path when `request_initial` finds a clean rendered state. The store updates only rows whose path is missing, and the service emits `photo_changed` for each updated row. Added a boundary case that renders in S1, adds the same RAW to S2, and verifies the path and event. Files: `crates/core/src/services/previews.rs`, `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`, `crates/testkit/tests/p06_live_previews.rs`.
- **F2:** Added provider-neutral `EditingProvider::settings_changed_at`; the Capture One adapter scans its `CaptureOne/Settings*/*.cos` layout and returns the newest mtime. Core retains the existing comparison and once-per-mtime behavior. C4 tests remain unchanged. Files: `crates/core/src/ports/editing.rs`, `crates/core/src/services/previews.rs`, `crates/integrations/src/editing/capture_one/adapter.rs`.

## Divergences

None.

## Files changed

- `crates/core/src/ports/editing.rs`
- `crates/core/src/ports/store.rs`
- `crates/core/src/services/previews.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/src/editing/capture_one/adapter.rs`
- `crates/testkit/tests/p06_live_previews.rs`
- `docs/implementation/master_plan.md` (P06 tracker row only)
- `docs/implementation/plans/P06_live_previews.md` (Review log entry)
- This handoff is separate from the checkpoint commit so its SHA field can identify the commit.

## Checkpoint SHA

`135c676` — `CHECKPOINT (not approved): fix P06 live previews findings`.

## Stamp

- `cargo test -p psm-testkit --test p06_live_previews` — passed, 19/19.
- `cargo test --workspace` — passed; all unit, integration and doc tests green.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run typecheck` — passed.
- `cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm test` — passed (no frontend test files).

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
