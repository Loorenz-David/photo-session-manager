---
plan: P50_camera_settings_and_support
role: fix
round: 3
state: DONE
date: 2026-11-08
actor: codex
---

# Finding → what changed → files

**F3 (MUST; launch read without installed camera support).** `CameraRuntime::bind` now reads the cached FTP password or starts the asynchronous Keychain read only when `support::installed(&self.paths).is_some()`. The read remains off the startup path. Added a `FakeSecrets` get counter and C5(e): a support-free app is bound and ticked, then the counter is asserted to remain zero after an async-read window. This protects all five `App::open_with` paths named in the finding because they have no support folder, so they do not query the real Keychain. No desktop test edit was needed.

Files: `crates/integrations/src/camera/runtime.rs`, `crates/testkit/src/fake_secrets.rs`, `crates/testkit/tests/p50_camera_settings_and_support.rs`, and the phase plan's C5(e) criterion row.

# Divergences

None.

# Files changed

- `crates/integrations/src/camera/runtime.rs`
- `crates/testkit/src/fake_secrets.rs`
- `crates/testkit/tests/p50_camera_settings_and_support.rs`
- `docs/implementation/master_plan.md` (P50 tracker row only)
- `docs/implementation/plans/P50_camera_settings_and_support.md` (C5(e) criterion and Review log)
- `docs/implementation/handoffs/implementer/handoff_P50_camera_settings_and_support_fix_r3.md`

# Checkpoint SHA

This handoff is included in the checkpoint commit. Resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P50_camera_settings_and_support_fix_r3.md` after the commit; a commit cannot contain its own SHA in its contents.

# Stamp

- `cargo test --workspace`: passed on the final tree, 816 passed, 0 failed, including the five formerly skipped Keychain-reaching `open_with` tests and P50 22/22. The first full run hit the previously classified C5(a) random-temp-path flake; an isolated C5(a) rerun passed, and the subsequent full workspace run passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `npm run typecheck` in `apps/desktop`: passed.
- `npm test` in `apps/desktop`: passed, 50 files / 369 tests.
- `git diff --check`: passed before handoff and tracker updates.

# ⚠ OWNER DECISIONS REQUIRED (0)

None.
