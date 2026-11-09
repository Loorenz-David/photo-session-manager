---
plan: P60_take_back_and_self_healing
role: implement
round: 1
state: DONE
date: 2026-11-09
actor: claude-opus (PSM_IMPLEMENTER=claude; continued the uncommitted partial tree Codex left)
---
# Handoff — P60 take-back fixes and self-healing for deleted files (implement r1)

## What was built
- **Capture One is not reopened during the take-back (#22).** While "Take the camera back" (automatic or by click) holds Capture One, no
  script of the app reaches it: every script except "is it running?" and "quit" is refused, preview requests wait in place (they neither
  fail nor count as attempts) and go out after the take-back ends. Before it saves and quits, the take-back now waits for scripts that
  were already talking to Capture One to finish (≤ 30 s). This was the likely cause of the 0.34 s relaunch: a script under way when
  Capture One quit woke it with its next call, and the take-back then wrongly said "did not quit". Render scripts also never wait or
  launch while Capture One is closed and the cable is waiting for the camera helper (P54's wait); they are tried again later.
- **The open dialog is named (#23).** If Capture One does not answer the quit or is still running after it, the header says
  `Capture One has a window open — close it so Session Manager can take the camera`, and the automatic take-back tries again for the
  same plug-in every 2 s until Capture One quits; then it runs to the end.
- **A stuck camera service is named (#25).** If, 10 s after Capture One quit, the cable is still being claimed and no app that uses
  cameras is running, the header says `macOS's camera service is holding the cable with no app using it. Restart the Mac to free it.`
  instead of blaming Capture One (kept until the camera is unplugged or our USB session starts).
- **Files deleted by hand heal themselves (#24/#26, changelog 43).** A photo whose RAW is gone is marked missing (the existing
  `photos.integrity = raw_missing`; classification and history unchanged). It leaves the session view and the session list's
  "missing preview" count, is never asked of Capture One, never counts as "a preview is rendering" (queued or issued renders of a gone
  RAW are dropped), and Health shows `N photos' files were deleted outside Session Manager` (code `file_missing`, label "Deleted
  files"). Put the file back at the same path and the photo returns (hash re-checked). Checked at startup, on window focus, when
  Capture One (re)opens, when a render for it is refused, and whenever a preview pass finds a RAW deleted or returned since the last check.
- **No re-render storm (#26).** When Capture One reopens, only photos that need a preview are asked for; the 100-photo check shows zero
  requests for current previews and the new arrival's request goes first.
- **Recipes repair themselves (10F §5).** Before each render batch the render's own recipe is read back and repaired (as at startup);
  if Capture One refuses a render because the recipe vanished mid-batch, one repair and one retry; a preview export that comes out in
  the wrong format (e.g. TIFF) is never taken as a preview — the recipe is repaired and the render sent once more.

## Tests → criterion rows
| Row | Test (boundary) | Asserts |
|---|---|---|
| C1(a) | `p60_take_back_and_self_healing::c1a_nothing_launches_capture_one_during_the_take_back` (fake Capture One ops while a manual take-back runs; a script in flight at its start; 5 previews asked for after the quit; every Apple event to a quit Capture One relaunches it) | no `launch` while our USB session is not reported; exactly one `launch`, after `quit`; each of the 5 renders' first `process` after the launch |
| C1(b) | `…::c1b_take_back_reports_success_never_did_not_quit` (same scenario, take-back result + status + Health) | `TakeBackReport { completed_steps: 5, camera_held: true }`; status on cable; "did not quit" in neither status nor Health |
| C2(a) | `…::c2a_open_dialog_is_named_and_the_same_plug_in_is_retried` (camera status = header source; automatic take-back) + `p60_header_sentences.test.tsx` C2(a) (header card shows the sentence) | header = exact sentence while the quit is ignored, no launch; after the fake quits again: on cable, one launch, ≥ 2 quits, one `usb.arrived` (same plug-in) |
| C3(a) | `…::c3a_stuck_camera_service_is_named` + `p60_header_sentences.test.tsx` C3(a) | after the quit, with `usb.claimed_by` again and no camera app, header = exact sentence ≥ 10 s after the quit; still there after step 5 relaunches Capture One |
| C4(a) | `…::c4a_deleted_file_heals_and_returns` (session view, Health, fake ops) | 3 photos, one RAW deleted, app start: view has 2 (not the deleted one), Health `1 photos' files were deleted outside Session Manager`, no op ever names it; file restored + focus: view 3, integrity `Ok`, Health item gone; separately (running app, Capture One no longer knowing the photo, RAW deleted): missing, Health sentence, no op names it |
| C4(b) | `…::c4b_missing_files_never_block_the_take_back` (automatic take-back, camera status) | 50 photos with renders queued/issued, all 50 RAWs deleted: the take-back runs (on cable, one launch, one quit) and the status never says "a preview is rendering" |
| C5(a) | `…::c5a_no_render_storm_when_capture_one_reopens` (fake ops) | 100 current previews, Capture One quits and reopens, focus, a new arrival: the only `process` is the arrival's |
| C6(a) | `…::c6a_recipes_repair_themselves_once_without_a_deferral_loop` (fake ops, stored preview) | recipe deleted mid-batch: one `make_recipe:PSM Preview`, two `process` for the queued render, preview stored; recipe switched to TIFF: the TIFF is not stored, one `set_recipe_props` with `outputFormat`, one re-render whose JPEG is the stored preview; 30 s later no further `process` (no loop), no `recipe_missing` |
| C7(a) | full suite, clippy, fmt, typecheck, UI, helper stub tests | see Stamp |
| Core mechanisms | `psm-core mechanisms::missing::tests::{missing_file_decision, missing_decision_transitions, render_eligibility_in_check_order}` | "missing file" decision; render-request eligibility (missing, current preview, take-back hold) |

Every test in the two new test files maps to a row above. An earlier version refused every script under any hold; P48 C3(e) (a held
gate still reports the open document) went red, so the full refusal was narrowed to the take-back's quiet mode. A sanity check (not a ledger): with the take-back's drain removed, C1(a) goes
red (Capture One woken after the quit, the take-back never sees it gone — the evening's #22 and its false "did not quit").

## Assumptions
1. **Where the hold lives (delegated):** in the launch gate and in the preview service. A take-back holds the gate in a "quiet" mode
   (`LaunchGate::hold_quiet`): every lane script but `editor_info`/`quit` is refused, and scripts already running are drained before
   save/quit (≤ 30 s, then it quits anyway with a log line). A plain hold keeps P48's meaning (launch-capable scripts refused, reads such as
   the document check still run — P48 C3(e)); render scripts are refused under either. The preview service skips requests while held, so
   no tracking rows churn.
2. **Render scripts never block:** with Capture One closed and a relaunch waiting for the camera helper, `process`/`find_image`/
   `find_image_in_folder`/`recipe_exists` answer "deferred" at once (not counted) instead of waiting up to 15 s on the app's background thread.
3. **How "missing" is stored (delegated):** the existing `photos.integrity` value `raw_missing` — no migration, no new column;
   classification and history untouched; the reason is the Health code `file_missing`.
4. **Missing means** nothing at the path or something that is not a file; an unreadable path (permissions) counts as present and is logged.
5. **"Leaves the session views"** = the session view (`sessions.view`, also the Unassigned view) and the session list's missing-preview
   count. Export and publish are unchanged: a missing main/condition photo still stops them with the existing `raw_missing` error.
6. **When the check runs:** startup, window focus, Capture One (re)opening (seen by the follow tick when it goes from not running to
   running, and after every ensure), any refused render for a gone RAW, and any preview pass that finds a stale RAW deleted or returned
   since the last check. So a deleted file is normally found **before** Capture One is asked; a `variant_not_found` for a gone RAW also
   triggers it. No periodic background scan (the partial tree's 5 s scan was removed).
7. **A file that comes back** is re-hashed: same bytes → `ok`; different bytes → `raw_changed` (existing rule), visible either way.
8. **Recipe batch:** renders sent while the render queue has work; only the render's own recipe is checked (a broken `PSM Master`
   does not block previews — P04 C3(b) kept). A refused `process` whose recipe no longer exists → one repair, one retry. A preview
   export not named `.jpg`/`.jpeg` → repair + one re-render; a second wrong one fails that preview ("Capture One produced no preview").
   Masters are not format-checked (TIFF by design).
9. **Dialog retry cadence:** 2 s after a failed quit; each try asks Capture One to quit again and waits ≤ 20 s; a click take-back
   that fails at step 3 shows the same sentence.
10. **Stuck-service rule and cadence (delegated):** checked every 500 ms during step 4; named once ≥ 10 s after the quit with no USB
    session, a new `usb.claimed_by` since the quit, and none of the known camera programs running; kept for that plug-in.
11. The header dot shows the first 48 characters (existing UI); the full sentence is in its card, as the UI test shows.
12. Health label for the new item: "Deleted files"; detail exactly the plan's sentence (including "1 photos'").

## Deviations
- **Actor:** run with `PSM_IMPLEMENTER=claude` (Opus). The tree held Codex's uncommitted partial work (6 files); I kept its direction
  (gate refusal, render hold, missing-file marking, retry state) and rewrote what did not hold: it checked both recipes before every
  render (broke P04 C3(b) and P17 C5(a)), dropped a new arrival's first request during the hold, made render scripts block on the
  P54 wait, scanned every 5 s, and counted deferrals as refusals.
- **Small additions outside the listed files** (all "as needed"): `EditingProvider::recipe_fault` (default `false`) and
  `RenderPurpose: Hash` in core; one SQL line in `crates/infra/src/sqlite/store.rs` (session list count); the Health label in
  `HealthPage.tsx`; `DirectCameraAdapter::cable_held`.
- No migration (assumption 3), so the P01 schema test is untouched.

## Files changed
Production: `crates/core/src/mechanisms/missing.rs` (new), `crates/core/src/mechanisms/mod.rs`, `crates/core/src/services/previews.rs`,
`crates/core/src/services/sessions.rs`, `crates/core/src/ports/editing.rs`, `crates/core/src/domain/editing.rs`,
`crates/infra/src/sqlite/store.rs`, `crates/integrations/src/camera/direct/{adapter.rs, mod.rs, take_back.rs}`,
`crates/integrations/src/camera/runtime.rs`, `crates/integrations/src/editing/capture_one/{adapter.rs, follow.rs, gate.rs,
launch_gate.rs, renders.rs}`, `crates/integrations/src/lib.rs`, `apps/desktop/src/features/health/HealthPage.tsx`.
Test support: `crates/testkit/src/fake_capture_one.rs` (`apple_events_launch`; `process` refuses an unknown recipe).
Tests: `crates/testkit/tests/p60_take_back_and_self_healing.rs` (new), `apps/desktop/src/features/shell/__tests__/p60_header_sentences.test.tsx` (new).
Docs: `docs/implementation/master_plan.md` (own tracker row), `docs/implementation/plans/P60_take_back_and_self_healing.md` (Review log),
this handoff.
Probes: none applied to tracked files except one sanity check on `take_back.rs` (drain disabled, C1(a) run, file restored byte-for-byte from a copy).

## Checkpoint SHA
`6ed79bfaf14736d75167ff8c8e3572b1e1c5af19` — 23 paths (code, tests, own tracker row, plan Review log). This handoff is left for the runner to commit.

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` (final code) | 873 passed, 1 failed: `p53_camera_upload_and_log::c3a_camera_events_and_uploads_are_logged_without_secrets` — intermittent, **pre-existing and environmental** (see below); rerun: file 2/3 then 1/1 green, test alone 6/6 green |
| `cargo test -p psm-testkit --test p60_take_back_and_self_healing` | 8/8 |
| `cargo test -p psm-core` (incl. `mechanisms::missing::tests`) | 58/58 |
| `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` | clean · clean |
| `npm run typecheck` · `npm test` (apps/desktop, Node 22) | clean · 60 files, 424/424 (incl. `p60_header_sentences` 2/2; the known `p33_split_grid` intermittent did not occur) |
| `tools/sidecar/build.sh --sdk …/Camera-Connection-Tests/vendor/sony-sdk` then `tools/sidecar/build.sh --stub --test` | SDK helper builds; stub restored (no `Cr_` libraries linked), ctest 1/1 |

**The P53 intermittent.** That test waits 3 s for log lines while other camera tests in the same binary start and stop apps. In this
checkout every camera `TestApp` takes ~4–12 s to shut down (the app's background thread sits in the file watcher's FSEvents setup). I
measured it with a scratch timing test (deleted): HEAD code built into the main build dir `apps/desktop/src-tauri/target` → 4.5 s;
the same HEAD code or the P60 code in a fresh build dir → 0.5–0.7 s. So it depends on the build dir, not on P60, and it existed before.
Worth a backlog note: on a 76 GB build dir, FSEvents setup is slow.

**Left outside the repo:** `/tmp/p60head-target` (2.1 GB scratch build of that measurement). `rm -rf /tmp/p60head-target` was refused by
the permission rules (the command was denied as part of a cleanup line), so it stays; safe to delete. The temporary git worktree was
removed (`git worktree list` shows only the main checkout). Building HEAD into the main build dir left stale `psm-core` artifacts; a
`touch` of the crate roots rebuilt them, and P04/P53/P60, the core units and clippy were re-run green afterwards.

## Manual try-out
1. Plug the camera in with Capture One open: Capture One closes, the dot reads Cable, and only then Capture One reopens (≈ 7–8 s).
   No "did not quit" in the log.
2. Delete one RAW in Finder, switch back to Session Manager: the photo disappears from the session; Health shows "1 photos' files were
   deleted outside Session Manager"; Capture One's preview work does not loop. Put the file back (Finder's Undo) and switch back: the photo returns.
3. Open a window in Capture One (e.g. its sign-in or preferences) and plug in: the camera card says "Capture One has a window open —
   close it so Session Manager can take the camera"; close the window and the take-back runs on its own.
4. In Capture One, change `PSM Preview`'s format to TIFF and take a photo: the preview still arrives as a JPEG and the recipe is back to JPEG.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
