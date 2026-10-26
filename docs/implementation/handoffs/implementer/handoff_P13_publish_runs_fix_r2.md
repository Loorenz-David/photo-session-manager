---
plan: P13_publish_runs
role: fix
round: 2
state: DONE
date: 2026-10-26
actor: codex
---
# P13 publish runs — fix r2 handoff

Resolved both production findings routed by review r1. The tracker is `IMPLEMENTED` with the fix 1 note.

## Finding → what changed → files

### F1 — ready nameless media stalled upload

`PublishService::upload` now waits for nameless media only while its status is `Processing` or `Uploaded`. Nameless
`Ready` and `Failed` media are ignored by upload adoption and handled by gallery reconciliation. Added the requested
boundary case: a linked product's READY nameless media is announced for removal, publishing reaches `done`, and the
resulting gallery equals the desired list D.

Files: `crates/core/src/services/publish.rs`, `crates/testkit/tests/p13_publish_runs.rs`.

### F2 — publish work blocked capture and previews

The shared runner now excludes `publish_step` jobs. A dedicated filtered runner handles only those jobs on its own
worker thread, and the worker is stopped and joined with `App`. The SQLite job query supports inclusion and exclusion
filters. TestApp ticks both runners. The requested boundary test holds the fake shop's media poll, records a newly
scanned capture photo row before releasing the poll, and then joins the publish worker.

Files: `crates/integrations/src/lib.rs`, `crates/infra/src/jobs.rs`, `crates/core/src/ports/store.rs`,
`crates/infra/src/sqlite/store.rs`, `crates/testkit/src/test_app.rs`, `crates/testkit/src/fake_commerce.rs`,
`crates/testkit/tests/p13_publish_runs.rs`.

## Divergences

None. The corrections and test boundaries follow the reviewer clauses.

## Files changed

- `crates/core/src/ports/store.rs`
- `crates/core/src/services/publish.rs`
- `crates/infra/src/jobs.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/src/lib.rs`
- `crates/testkit/src/fake_commerce.rs`
- `crates/testkit/src/test_app.rs`
- `crates/testkit/tests/p13_publish_runs.rs`
- `docs/implementation/master_plan.md` (P13 tracker row)
- `docs/implementation/plans/P13_publish_runs.md` (Review log)
- `docs/implementation/handoffs/implementer/handoff_P13_publish_runs_fix_r2.md`

## Checkpoint SHA

`5e9531e9b88b3fb4ed190b927c6700f9c5e45a5b` — `CHECKPOINT (not approved): fix P13 publish runs`.

## Stamp

Final tree: checkpoint `5e9531e9b88b3fb4ed190b927c6700f9c5e45a5b`; no test failure baseline was recorded before the
fix. No mutation probes were run, per flow §4.

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS: 345 Rust tests; 36 P13 boundary tests; doc tests pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm run typecheck` | PASS |
| `cd apps/desktop && npm test` | PASS: 79 tests across 5 files |
| `git diff --check` | PASS |

One earlier full-suite attempt had a transient failure in P05 `c4d_every_delivery_recorded`; its isolated rerun passed,
and the final full-suite run passed.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
