---
plan: P50_camera_settings_and_support
role: review
round: 2
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-08
actor: opus
---
# Re-review P50_camera_settings_and_support, round 2 (after fix 1): CHANGES_REQUESTED, one MUST

Fix 1 resolves both round-1 findings in production code. F1: Keychain credentials read at startup during a USB session now reach the
sidecar after the session ends. F2: the redaction cell now observes the runtime. One blocking item remains, and it is classified
**MUST**: F3, the orchestrator's finding that `bind` reads the Keychain's FTP password before checking for a support package. I
verified it by reading. It is routed here and was not in the fix prompt. Two further items are DROP and went to the backlog: the new F1
test cannot fail on the F1 defect, and C5(a) can flake on a random temp path. Suite, lint and typecheck are green.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. Whether F3 gets the second fix is the orchestrator's call (flow §5: `FIX2_AUTHORISED`).

## Review history
r1 (`06f5e61`) verified the installer, manifest, `installed`/`damaged` and health, secrets to the store only, Wi-Fi restart and USB
defer in `secrets_changed`, the FTP password, fingerprint C++/Rust paths, commands, error mapping, event, P47 (a)(b)(e)(f), H1–H8,
22 of 23 rows. Those areas are settled and not re-verified here. This round covers the F1/F2 seam in `runtime.rs` and the testkit, plus
the orchestrator's F3 entry (commit `638dc38`, plan Review log).

## Gate
The tracker row read `IMPLEMENTED` with the fix note. The fix checkpoint `3fd76cf` (`CHECKPOINT (not approved): P50 startup credentials
and redaction`) exists. HEAD `486762f` differs from it only by the runner's re-review prompt (`git diff 3fd76cf HEAD --name-only`), so
the code tree is `3fd76cf`. `git status --porcelain` was clean at start and again after all runs.

## Perimeter check
`git show --stat 3fd76cf` (7 paths):
- `crates/integrations/src/camera/runtime.rs`, `crates/testkit/src/fake_secrets.rs`, `crates/testkit/src/test_app.rs`,
  `crates/testkit/tests/p50_camera_settings_and_support.rs`: exactly F1's and F2's allowed files.
- `docs/implementation/master_plan.md` (P50 row only), the plan's Review log (+2 lines), the fix handoff: the fix role's closing artefacts.

Nothing outside the perimeter. Commits `638dc38` (orchestrator, plan Review log +1 line) and `9aeea8b`/`486762f` (runner prompts) are
not fix work. `FakeSecrets` changed from a tuple struct to `{ values, cache_enabled }`, and `.0` has no remaining users
(workspace builds; clippy is green).

## Suite, lint, typecheck (reviewer run, code tree `3fd76cf`)
Run line, written before the run: the re-review prompt mandates one run, and the fix stamp reports two first-run flakes, so this run
also checks whether those flakes recur. Five tests are skipped because they read the owner's real Keychain (F3). Running them would
repeat the prompt the owner already got.

| Command | Result |
|---|---|
| `cargo test --workspace -- --skip c1b_second_open_preserves_settings_and_migration --skip c4d_open_drains_file_left_while_closed --skip c7a_startup_scans_root_and_children --skip c7b_startup_ignores_cache_and_known_raw --skip c6a_startup_marker_is_written_before_app_open` | exit 0; 810 passed, 0 failed, 5 filtered (r1: 814 + new `c5d` = 815); p50 21/21 in 51.9 s |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` (Node 22.22.3) | exit 0 |
| `npm test` | exit 0; 50 files / 369 tests |

Variation run (scratch, outside the repo): `git archive 3fd76cf` into `/tmp/p50rev2/r1runtime`, `runtime.rs` replaced by the
`06f5e61` version (the round-1 code with the F1 defect), separate target dir, `cargo test -p psm-testkit --test
p50_camera_settings_and_support c5d` → **1 passed**. See R1 below.

## Findings resolved
### F1: resolved in production. Its new test cannot fail (R1, DROP)
Read against R-I18, P47 (d) and the r1 correction:
- When the startup read returns, `update_spawn_credentials()` writes the line into the running supervisor's `spec.first_line`. Every
  later backoff respawn therefore sends it. Then `restart_if_not_usb()` runs, as before.
- `spawned_credentials` records the line each live process received. All three spawn sites set it: `bind`, `restart_if_not_usb`, and
  the supervisor's `Started` event in `tick`. The `Started` handler re-locks `supervisor`, which is safe: the poll block releases that
  lock before `side` is handled.
- In `tick`, after `perform` has applied the events (so `usb_session()` is false), a `SessionLost { transport: Usb }` combined with a
  stale line (`spawned != Credentials{credentials}`) leads to `restart_if_not_usb()`. No restart happens when the credentials are still
  unread (`None`); the read's own `restart_if_not_usb` covers that case later. No restart happens when the line already matches.
- This also covers `secrets_changed` during USB (spec updated, credentials stored, process stale): the sidecar now restarts at session
  loss instead of at the next backoff spawn. That is within R-I18 (a restart is allowed while no USB session is held) and consistent
  with C5(c), which stays green.

The production behaviour matches the correction. The new boundary test does not discriminate. See R1.

### F2: resolved
`setting_secrets` now calls only `CameraService::set_secrets(…, || secrets_changed())`. `secrets_changed` registers both values
(`logging::add_secret`). So C5(a)'s `redact_registered("x p2 y") == "x [REDACTED] y"` now fails if the runtime stops registering.

## Remaining findings
### F3 (blocking, B1 + secrets/no-live-services rule), route: production. The Keychain is read at every start, even with no camera support
Source: the orchestrator's entry in the plan Review log (`638dc38`), verified here by reading.
**What is wrong.** In `CameraRuntime::bind` (`runtime.rs`, the block after `adapter.publish`), `self.secrets.cached(FTP_PASSWORD)` is
checked first. For the Keychain, `cached` is always `None` (the `SecretStore` default), so a thread calls `secrets.get(FTP_PASSWORD)`.
This happens **before** any support check: `bind_receiver_with_password` checks `installed_sidecar` only after the value has been read.
Before P50, `bind_receiver` returned on `installed_sidecar(..).is_none()` before reading anything. `App::open_*` calls `bind`
unconditionally (`crates/integrations/src/lib.rs`, `camera_runtime.bind(AppBinding{…})`). Two consequences:
1. **Production.** Every launch reads the Keychain item, including installs with no Sony support package that run on Capture One alone.
   When the item exists (any Mac where an FTP password was ever generated), a launch after each ad-hoc update shows a Keychain access
   prompt (P47 S6) for a camera feature that is not in use.
2. **Suite.** `App::open_with` wires the real `KeychainSecrets`. Five tests reach it: p00 `c1b_second_open_preserves_settings_and_migration`
   (two opens), p00 `c4d_open_drains_file_left_while_closed`, p05 `c7a_startup_scans_root_and_children` and
   `c7b_startup_ignores_cache_and_known_raw` (through `reopen`), and desktop `c6a_startup_marker_is_written_before_app_open`. On the
   owner's Mac they read the real item and prompt. The orchestrator observed exactly this during the phase's checks.
   The orchestrator's entry names only the desktop test. The other four have the same path.
**Authority.** Plan Notes → Hazards ("`KeychainSecrets` is still never constructed in tests") and the plan Review log L23 ("a
`KeychainSecrets` is never called from a test"); master plan standing rule B3 (secrets / live services). R-I18 and P47 (d) require the
read to stay **off** the start path, not to happen without support. P47 S6 (prompts after every update).
**Triage: MUST.** The outcome is user-visible: a Keychain prompt at launch with no camera support, and the automated suite prompting on
the owner's Mac. It also breaks the secrets/live-services rule.
**Correction** (as the orchestrator's entry): read `ftp-password`, and the access credentials (already behind `installed_sidecar`),
only when a support package is installed (`installed(&paths).is_some()`). Keep the read off the start path (thread). The runtime check
alone keeps all five `open_with` tests away from the Keychain, because their temp homes have no support folder. Routing the desktop
test through a fake store is optional. One boundary row: no support installed → the store's `get` is never called (a counting
`FakeSecrets`).
**Files the fix may touch.** `crates/integrations/src/camera/runtime.rs`; `crates/testkit/src/fake_secrets.rs` (a get counter);
`crates/testkit/tests/p50_camera_settings_and_support.rs`; `apps/desktop/src-tauri/src/lib.rs` (the `c6a` test only, if routed
through a fake).

### R1 (B2), triage DROP. `c5d_startup_keychain_read_restarts_after_usb_session_lost` cannot fail on the F1 defect
**Evidence.** It passes on the round-1 runtime (scratch run above). With `cache_enabled = false`, `FakeSecrets::get` still answers at
once. The startup thread therefore finishes before the fake's `ready_usb` script reaches `CONNECTED_USB` (it sleeps 300 ms first), and
the thread's `restart_if_not_usb()` respawns the helper with the credentials before USB. By the time the test injects `session.lost
usb`, `pids.len() == 2` already holds, so the session-lost path is never exercised.
**Why DROP.** The production code is correct by reading (above). What is missing is a test that can fail ("more tests", flow §5).
The remedy is in the backlog: a no-cache store whose `get` blocks until the test releases it after `Connected { link: Cable }`.
My round-1 correction asked only for "a secret store whose `cached` returns `None`". That did not force the slow read, so this is
partly a defect of the r1 correction (see lessons).

### R2 (B4-adjacent), triage DROP. C5(a) flakes on random temp paths
`assert!(!settings.contains("u2"))` / `"p2"` against a `settings.json` that holds random `.tmpXXXXXX` paths. The fix implementer hit
it once, and it did not recur in this run. It is intermittent and the change is test-only, so it is not a reliable red suite. Backlog:
use values that cannot occur in a path.

## Triage (flow §5)
| Item | Class | Reason | Destination |
|---|---|---|---|
| F3 Keychain read without support | **MUST** | user-visible prompt at launch with no camera support; the suite reads the owner's real Keychain (secrets rule) | second fix, if the orchestrator authorises it |
| R1 `c5d` cannot fail on the F1 defect | DROP | production correct; more tests | `backlog.md` |
| R2 C5(a) temp-path flake | DROP | intermittent, test-only | `backlog.md` |

## Assumptions (flow §6)
The fix handoff declares no new assumptions and no divergences. It correctly left F3 to this review.

## Lessons for the plans
- A correction that asks for a test of a *slow* read must name the slowness. "`cached` returns `None`" was not enough, because the fake's
  `get` is instant. Name the gate instead: "`get` blocks until the test releases it after the USB session opens".
- When a phase moves a read earlier in a function, the plan should require the reviewer to compare the guard order with the old code.
  F3 is a guard that was lost when `bind_receiver` was split into the `cached`/thread path.
- The plan's Hazard "`KeychainSecrets` is never constructed in tests" was already false (`App::open_with` constructs it). It held only
  because nothing read it without a support folder. The planner should state it as "never *called*" and name the guard that keeps it so.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P50_camera_settings_and_support_review_r2.md` (this file, new)
- `docs/implementation/master_plan.md` (P50 tracker row only → `CHANGES_REQUESTED`)
- `docs/implementation/plans/P50_camera_settings_and_support.md` (Review log entry appended)
- `docs/implementation/backlog.md` (two P50 reviewer r2 entries appended)
- No production or test file in the repo edited, and no mutation probe. Scratch outside the repo: `/tmp/p50rev2/` (suite logs; the
  `r1runtime/` export and its `target/` remain there, since the session was not permitted to delete them). Build artefacts under the
  repo's `target/` (untracked). No narrative written (not approved). No Keychain, database or settings state touched; the five
  Keychain-reaching tests were skipped for that reason. Nothing committed.
