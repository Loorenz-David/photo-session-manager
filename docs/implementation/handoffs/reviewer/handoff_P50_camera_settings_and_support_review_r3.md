---
plan: P50_camera_settings_and_support
role: review
round: 3
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-08
actor: opus
---
# Re-review P50_camera_settings_and_support, round 3 (after authorised fix 2): APPROVED_WITH_NOTES

Fix 2 resolves F3 in production. `CameraRuntime::bind` now looks up the FTP password, from the cache or through the background
Keychain read, only when a Sony support package is installed. A launch without camera support therefore never touches the Keychain,
and the five suite tests that open the app on the real Keychain no longer reach it. I ran them unskipped and they were green. The
perimeter is exactly what the ruling allowed. One blocking-class item remains and is triaged **DROP**: R3, the new C5(e) test cannot
fail on the F3 defect (B2), because its fixture's store answers the password from the cache, so the old code never called `get`
either. No MUST remains, so the verdict is APPROVED_WITH_NOTES and R3 goes to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
- r1 (`06f5e61`) settled the installer, manifest, `installed`/`damaged` and health, secrets to the store only, the Wi-Fi restart and
  USB defer in `secrets_changed`, the FTP password, the fingerprint paths, commands, error mapping, events, P47 (a)(b)(e)(f), H1–H8,
  and 22 of 23 rows.
- r2 (`3fd76cf`) settled F1 (startup credentials reach the respawn line; restart on `session.lost usb` with a stale line) and F2
  (C5(a) redaction observes the runtime). It triaged F3 MUST, and R1 (`c5d` cannot fail) and R2 (C5(a) temp-path flake) DROP.
- r3 (this round) covers only the F3 seam: the support gate in `bind`, the `FakeSecrets` counter, and C5(e). Settled areas were not
  re-verified.

## Gate
Tracker row: `IMPLEMENTED`, actor codex, note "fix 2: F3 resolved …". Fix checkpoint `00d112e` (`CHECKPOINT (not approved): P50
no-support Keychain gate`) exists. HEAD `a390c6c` differs from it only by the runner's re-review prompt
(`git diff 00d112e HEAD --name-only` → `docs/implementation/prompts/reviewer/P50_camera_settings_and_support_review_r3.md`), so the
code tree is `00d112e`. `git status --porcelain` was clean at the start.

## Perimeter check
Authority: the orchestrator's ruling in the plan Review log (`75fd4d5`, FIX2_AUTHORISED). `git show --stat 00d112e` (6 paths):
- `crates/integrations/src/camera/runtime.rs` (+11/−9, the gate only), `crates/testkit/src/fake_secrets.rs` (+7, a `get_count`
  counter), `crates/testkit/tests/p50_camera_settings_and_support.rs` (+11, `c5e_no_support_does_not_read_keychain_secrets`): allowed.
- `apps/desktop/src-tauri/src/lib.rs`: not touched (allowed only if needed; it was not needed).
- `docs/implementation/master_plan.md` (P50 row only), the plan (C5(e) row, header count 23 → 24 rows, Review log entry), the fix
  handoff: closing artefacts. The C5(e) row is the "one p50 row" the ruling asked for.

Nothing outside the perimeter. Commits between `3fd76cf` and `00d112e` that are not fix work: `0f5c96e`/`a2b5208`/`5eeec0c` (runner),
`75fd4d5` (orchestrator ruling, plan Review log +6 lines).

## Suite, lint, typecheck (reviewer run, code tree `00d112e`)
Run line, written before the run: the re-review prompt mandates one run, and the orchestrator's ruling requires this re-review to run
the whole suite unskipped. The point is to confirm that the five tests r2 skipped no longer reach the real Keychain. By reading,
the only `SecretStore::get` calls in the workspace are `bind`'s FTP read (now gated), the credentials thread (behind
`installed_sidecar`), `secrets_changed` and `ftp_password_changed` (user actions). The suite's temporary homes have no support
folder, so no prompt was expected.

| Command | Result |
|---|---|
| `cargo test --workspace` (unskipped) | 44 of 45 test binaries finished, **794 passed, 0 failed**. The run was stopped by the session's tool time limit (about 20 minutes) inside the last binary, the p50 testkit file. It was not a hang: that binary takes about 85 s. All five Keychain-reaching tests passed: desktop `p25_tests::c6a_startup_marker_is_written_before_app_open`, p00 `c1b_second_open_preserves_settings_and_migration`, `c4d_open_drains_file_left_while_closed`, p05 `c7a_startup_scans_root_and_children`, `c7b_startup_ignores_cache_and_known_raw`. |
| `cargo test -p psm-testkit --test p50_camera_settings_and_support` (the cut-off remainder) | exit 0; **22 passed**, 0 failed (85.0 s) |
| `cargo test --workspace --doc` (the remainder) | exit 0; 0 doc tests |
| Total | **816 passed, 0 failed**, nothing skipped (= the fix stamp's 816) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` (apps/desktop) | exit 0 |
| `npm test` (apps/desktop) | exit 0; 50 files / 369 tests |

The remainder run covers exactly the targets the cut-off run had not finished. `crates/testkit/tests/` has no file after `p50`, and the
doc tests come last. No test was run twice.

## Findings resolved
### F3: resolved
Read against the r2 correction, R-I18 and P47 (d):
- `bind` (`runtime.rs` 398–409): the `cached` lookup and the thread calling `secrets.get(FTP_PASSWORD)` now sit inside
  `if super::support::installed(&self.paths).is_some()`. `installed` is exactly `installed_sidecar(..).is_some()`
  (`support/mod.rs`: `installed_sidecar = installed(..).map(sidecar_path)`). The new gate is therefore the same condition that
  `bind_receiver_with_password` used to check only *after* the read. No case that used to bind a receiver is lost: without support,
  the old code read the password and then returned at `installed_sidecar(..).is_none()`.
- The read stays off the start path: with support installed and no cache (the Keychain), it still runs on a spawned thread (P47 (d)).
- Install after launch: `camera_support_install` → `support_changed()` → `app.reopen()` → `bind` again, now with `installed` =
  `Some`, so the receiver gets its password exactly as before the fix. A damaged folder makes `installed` `None`: no read, no receiver,
  and health says "damaged" (unchanged).
- Remaining reads without support are user-initiated only: `ftp_password_changed` (Generate) and `secrets_changed` (entering the
  Access Authentication values). Both are writes the user just made, so they are not a launch prompt. P47 S6 concerns launches.
- The five `App::open_with` tests (p00 `c1b`, `c4d`; p05 `c7a`, `c7b`; desktop `c6a`) use temporary homes with no
  `camera/sony/` folder, so `installed` is `None` and `KeychainSecrets::get` is never called. They passed unskipped in the run above.

## Remaining findings
### R3 (B2), triage DROP. C5(e) `c5e_no_support_does_not_read_keychain_secrets` cannot fail on the F3 defect
**Evidence, by reading** (flow §4: no production edit, not even in scratch; the reading is conclusive).
`TestApp::with_fake_document()` → `create(.., sidecar = None)` → `secret_cache_enabled` stays `true` (`test_app.rs` 302, 384), and
the fixture seeds `ftp-password = "ftp-pw-unique-1"` (390). In the pre-fix `bind`, `self.secrets.cached(FTP_PASSWORD)` therefore
returns `Some(..)`, the synchronous branch runs `bind_receiver_with_password` (which never calls `get`), and no thread is spawned. With no
support, nothing else in `bind` or `tick` calls `get` (the credentials thread sits behind `installed_sidecar`). So on the `3fd76cf`
runtime, `get_count()` is also `0` and the test passes. The production defect only shows with a store whose `cached` is `None`, which
is the Keychain's behaviour. The fixture does not model it.
**Why DROP.** The production fix is correct by reading (above). The five real-Keychain tests now run green, and they are the
observable proof on this Mac. What is missing is a test that can fail: "more tests", flow §5. This is the same shape as r2's R1.
**Remedy (backlog).** Call `a.secrets.disable_cache()` before the bind under test. Since the fixture binds during `create`, the
simplest form is a `TestApp` builder flag, or `with_camera_script`-style `secret_cache_enabled: false` without a sidecar, so that the
old code would take the thread path and `get_count() == 1`.

### Passing observation (note, backlog)
The plan now has rows C5(a), (b), (c), (e) and no C5(d). The F1 boundary test `c5d_startup_keychain_read_restarts_after_usb_session_lost`
(fix r2) has no criterion row, so it is untraced. It is the F1 correction's test, already DROP-noted as non-discriminating (R1).
Fold it into a C5(d) row when R1's remedy is done, or remove it.

## Triage (flow §5)
| Item | Class | Reason | Destination |
|---|---|---|---|
| R3 C5(e) cannot fail on F3 | DROP | production correct by reading, and the real-Keychain tests run green unskipped; a non-discriminating test is "more tests" | `backlog.md` |
| C5(d) row missing for `c5d` | DROP (note) | trace hygiene, no behaviour | `backlog.md` |
| r2 R1, R2 | DROP (unchanged) | already in the backlog | — |

No MUST, so the verdict is **APPROVED_WITH_NOTES**.

## Assumptions (flow §6)
The fix handoff declares no assumptions or divergences. None of the r1 assumptions contradicts the intention (settled in r1).

## Carry-forward dispositions
| Note | Destination |
|---|---|
| R3 C5(e) fixture uses the cache | backlog → next touch of the p50 tests (P51 or the backlog tidy-up) |
| `c5d` untraced / C5(d) row | backlog → same |
| r1/r2 notes already in the backlog (SHA refresh-always, fingerprint repeats, fingerprint pinning, test seams, archive names, H8 commands, suite time, c5d, C5(a) flake) | unchanged: P51 / P52 / tidy-up as recorded |

## Lessons for the plans
- A "never called" row must name the fixture state that makes the old code call. Here that was a cache-less store. "The counter stays
  0" was true of the defect too, because the default fake answers from its cache. Corrections that add a counter should say which
  branch the counter must observe.
- Second time in this phase (R1, R3) that a fake's convenience (an instant `get`, a warm cache) hides the production path the row
  exists for. The `FakeSecrets` default should model the Keychain (`cached` → `None`) and let tests opt into a cache, not the reverse.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P50_camera_settings_and_support_review_r3.md` (this file, new)
- `docs/implementation/master_plan.md` (P50 tracker row only → `APPROVED`)
- `docs/implementation/plans/P50_camera_settings_and_support.md` (Review log entry appended)
- `docs/implementation/backlog.md` (two P50 reviewer r3 entries appended)
- `docs/implementation/narratives/P50_camera_settings_and_support.md` (new)
- No production or test file edited, and no mutation probe. Scratch outside the repo: `/tmp/p50rev3/` (suite logs only). Build
  artefacts under the repo's `target/` and `apps/desktop/src-tauri/target/` (untracked). No Keychain, database or settings state
  touched by me. Nothing committed.
