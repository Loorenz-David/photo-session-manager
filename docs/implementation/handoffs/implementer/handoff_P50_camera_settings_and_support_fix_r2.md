---
plan: P50_camera_settings_and_support
role: fix
round: 2
state: DONE
date: 2026-11-08
actor: codex
---

# Finding → what changed → files

- **F1 (startup Keychain credentials during USB):** When the asynchronous read returns, the runtime now stores the credentials in the supervisor's respawn spec. It records the credentials used for the live process and, after a USB `session.lost` has been applied to the adapter, restarts only if that process still has a stale line. Added a Keychain-like fake mode with `cached() == None` and a boundary test that observes the second process start with the stored credentials after USB loss. Files: `crates/integrations/src/camera/runtime.rs`, `crates/testkit/src/fake_secrets.rs`, `crates/testkit/src/test_app.rs`, `crates/testkit/tests/p50_camera_settings_and_support.rs`.
- **F2 (C5(a) redaction fixture):** Removed both fixture-side `add_secret` calls. The test now observes the redaction registration performed by the runtime. File: `crates/testkit/tests/p50_camera_settings_and_support.rs`.

# Divergences

None from the F1 and F2 correction clauses. The separate F3 entry at the end of the plan's Review log is explicitly routed to the re-reviewer and is outside this fix prompt's r1 finding list; it remains for that review.

# Files changed

- `crates/integrations/src/camera/runtime.rs`
- `crates/testkit/src/fake_secrets.rs`
- `crates/testkit/src/test_app.rs`
- `crates/testkit/tests/p50_camera_settings_and_support.rs`
- `docs/implementation/master_plan.md` (P50 tracker row → `IMPLEMENTED`)
- `docs/implementation/plans/P50_camera_settings_and_support.md` (fix r2 Review log)
- `docs/implementation/handoffs/implementer/handoff_P50_camera_settings_and_support_fix_r2.md`

# Checkpoint SHA

The checkpoint contains this handoff, so it cannot embed its own commit SHA. Resolve it with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P50_camera_settings_and_support_fix_r2.md` after the commit.

# Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | Passed on final run; all workspace tests, including P50 (21/21). An earlier run hit a random-temp-path substring false positive in C5(a); the focused rerun and final workspace run passed. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed. |
| `cargo fmt --all -- --check` | Passed. |
| `cd apps/desktop && npm run typecheck` | Passed. |
| `cd apps/desktop && npm test` | Passed on final run, 50 files / 369 tests. The first run hit the previously recorded intermittent P33 C4(d) timing failure; the rerun passed. |
| `git diff --check` | Passed before checkpoint. |

# ⚠ OWNER DECISIONS REQUIRED (0)

None.
