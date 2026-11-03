---
plan: P24_capture_one_provisioning
role: fix
round: 2
state: DONE
date: 2026-11-03
actor: codex
---

# P24 fix cycle 1 handoff

## Finding → what changed → files

| Finding | What changed | Files |
|---|---|---|
| F1 (B3): p00 app-open tests could reach live Capture One through unconditional startup provisioning. | `c1b_second_open_preserves_settings_and_migration` and `c4d_open_drains_file_left_while_closed` now construct the app with `FakeCaptureOneScripting` and `run_startup = false`. | `crates/testkit/tests/p00_spike.rs` |
| F2 (B2): C4(b) could not observe the open-alongside throwaway `make` because Rust performed it outside the JXA harness. | `open_document.js` now makes `PSM Open Window` at the scratch parent before the single target `open`; null `currentDocument()` is safe in error cleanup. Rust passes the scratch parent as argv 2 and still removes the scratch directory. C4(b) asserts the ordered `make` then `open`. | `crates/integrations/src/editing/capture_one/jxa/open_document.js`, `crates/integrations/src/editing/capture_one/jxa.rs`, `crates/testkit/tests/p24_capture_one_provisioning.rs` |

## Divergences

None. Both corrections were implemented as written.

## Files changed

Checkpoint perimeter (four paths):
- `crates/integrations/src/editing/capture_one/jxa.rs`
- `crates/integrations/src/editing/capture_one/jxa/open_document.js`
- `crates/testkit/tests/p00_spike.rs`
- `crates/testkit/tests/p24_capture_one_provisioning.rs`

Process records updated for the runner: `docs/implementation/master_plan.md`, this plan's Review log, and this handoff.

## Checkpoint SHA

`79e5c44e38a9d4a1933b482d20cb37c744cb4f4d` — `CHECKPOINT (not approved): fix P24 Capture One provisioning findings` (four-path fix perimeter).

## Stamp

- Targeted `cargo test -p psm-testkit --test p00_spike --test p24_capture_one_provisioning`: 28 passed.
- `cargo test --workspace`: passed, 445 Rust tests.
- `npm test` (`apps/desktop`): passed, 129 UI tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `npm run typecheck` (`apps/desktop`): passed.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
