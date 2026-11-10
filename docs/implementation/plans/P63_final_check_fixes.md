---
phase: P63_final_check_fixes
title: Fixes from the final hands-on check — Capture One is never launched by a background script during a take-back, a missing Capture One session is reported instead of silently recreated, changing the session moves the camera helper's staging too, Save settings cannot be missed, the style is picked from a drop-down, and the unplug-to-Wi-Fi time is back to ~6 s
difficulty: high
projection: no
depends_on: [P64_publish_speed_and_visibility]
---
# P63 — Final-check fixes (#22 remainder, #27, #28, Save settings, unplug recovery)

Written by the orchestrator (Opus) on 2026-11-10 from the owner's hands-on check on the dev Mac (build of P53–P62; evidence
`docs/intention/evidence/sony_camera_lab.md` §Final hands-on check). Everything passed; these are the remaining defects. The owner installs on the work Mac
meanwhile (option A).

## Goal
1. **#22 remainder — no launch by background scripts.** During the automatic take-back, Capture One was relaunched 0.35 s after the quit
   (stopwatch 14:36:01.386 → 14:36:01.739), before our USB session (14:36:07.5), and the take-back logged a false
   `stopped at step 3: Capture One did not quit`. P60 held render requests, but another path still talks to Capture One: likely the variant follower
   (`crates/integrations/src/editing/capture_one/follow.rs`, `find_image` about once a second) — on macOS any JXA/AppleScript that addresses
   `Application('Capture One')` launches it. After this phase **every** script call to Capture One goes through one gate that, while Capture One must
   stay closed (take-back between quit and end; the P54 wait after the photographer quits it with the cable waiting), does not run the script (callers
   see a "not now" result that is neither a failure nor an attempt). No script ever launches Capture One except the explicit launch step.
2. **#27 — missing session document.** The configured document `~/Pictures/session test 1` had been deleted; the app silently recreated
   `session test 1/Capture/<item>/` and filed a photo there that Capture One never saw (stuck "adjusting"). After this phase, if the configured document
   (its folder or `.cosessiondb`) does not exist, the app creates **no** folders under it, the receiver answers uploads `451` (MC-32: the camera keeps
   them queued), and the header + Health say `The Capture One session in Settings is missing — choose one in Settings`.
3. **#28 — helper staging follows the session.** After the photographer changed the document in Settings, the helper kept `--save-dir` under the old
   `session test 1/Capture/.incoming/` until the app restarted (photos were still filed correctly). After this phase a document change restarts the
   helper with the new capture root's staging folder (outside a protected operation; deferred until it ends).
4. **Save settings cannot be missed.** The owner did not find "Save settings" (it sits below all sections). Make the Save button always visible
   (sticky footer of the Settings page) and, when leaving Settings with unsaved changes, ask `Save your changes?` with Save / Discard / Cancel.
5. **Unplug recovery.** Unplug → Wi-Fi ready took 11.8 s (`recovery_ms=11838`, 13:40:24Z) where run 2 measured 6.1 s. Find why (P57's cached-endpoint
   direct reconnect timing out before discovery? the P56 holder watch?) from the code and the helper events, fix it so the unplug path is ≤ ~6 s with a
   reachable camera, and write the cause in the handoff.
6. **Style drop-down (owner 2026-11-10).** Settings → Style becomes a drop-down of the styles Capture One lists (user/custom styles first, then built-in),
   plus "No style"; the stored value stays the style's exact name. When Capture One is not running or the list cannot be read, the field falls back to
   the current text (editable) with `Couldn't read Capture One's styles`. (The owner typed `Test-style` for `test-style` and saw nothing applied.)
7. **Test time (backlog P62 r3).** `p48_camera_ownership` takes ~12 min since P60's real-time waits; make those waits use the injected clock or test-sized
   timeouts so the file runs in under a minute, without changing production values.

## Read first
Evidence §Final hands-on check; book `10E` MC-31/MC-32/MC-34, `10C` (document), changelog 38, 43. P54, P57, P60 narratives. Code: `crates/infra/src/osascript.rs`,
`crates/integrations/src/editing/capture_one/{gate.rs, launch_gate.rs, follow.rs, adapter.rs, jxa/*.js}`, `crates/integrations/src/camera/{runtime.rs, direct/take_back.rs,
sidecar/supervisor.rs}`, `crates/infra/src/ftp/*`, `crates/core/src/services/{camera_ledger.rs, sessions.rs, settings.rs}`, `apps/desktop/src/features/settings/SettingsPage.tsx`,
`native/camera-sidecar/src/transport_manager.cpp`. Tests: p48, p54, p57, p60 testkit files; settings UI tests.

## Files expected to change (write perimeter)
The files above as needed; `crates/testkit/tests/p63_final_check_fixes.rs` (new); a settings UI test; `crates/testkit/tests/p48_camera_ownership.rs` (timing only,
same assertions); C++ tests if the unplug fix is in the helper. ≈ 16 paths, ≈ 500 lines.

## Criteria (7 criteria, 8 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | no script launches Capture One during a take-back | take-back past its quit step; the follower and a render both due | the fake Capture One receives no script and no launch until our USB session; then one launch; no "did not quit" | `[rule]` MC-34; #22 |
| C2(a) | missing document | configured document folder absent; an FTP upload and a session activation | no folder created under it; upload answered `451`; header/Health sentence as above | `[rule]` MC-32; 10C; #27 |
| C3(a) | staging follows the document | document changed in Settings while connected | the helper is restarted with `--save-dir <new capture root>/.incoming/sdk-…`; not during a protected operation | `[rule]` MC-31; #28 |
| C4(a) | Save always visible / unsaved changes | Settings scrolled to the top with a change; navigate away | the Save button is in view; leaving asks `Save your changes?` (Save saves, Discard drops, Cancel stays) | `[rule]` owner 2026-11-10 |
| C5(a) | unplug recovery | helper test: USB session, unplug, Wi-Fi reachable (cached endpoint valid / invalid) | `CONNECTED_WIFI` within the policy's bound (≤ ~6 s with the fake's timings); the cause in the handoff | `[rule]` MC-31; P52 A7 |
| C6(b) | style drop-down | fake Capture One lists `test-style` (user) and `Vivid` (built-in) | Settings shows a drop-down with `No style`, `test-style`, `Vivid`; choosing `test-style` and saving stores `style_name: "test-style"`; list unreadable → editable text + the sentence | `[rule]` 10C step 5; owner 2026-11-10 |
| C6(a) | p48 runs fast | `cargo test -p psm-testkit --test p48_camera_ownership` | under 60 s, same assertions | `[rule]` backlog P62 r3 |
| C7(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck, sidecar stub tests green | `[rule]` master plan §7.2 |

## Notes
- **Hard rules:** never kill or quit any process except Capture One through the take-back; never delete photos; the suite never needs the camera or
  Capture One. Default implementer (Codex; Claude when Codex is exhausted). If the SDK is present locally, prove the SDK build compiles, then restore the stub.

## Review mode (owner 2026-11-10: "I need this within an hour")
- **Implementer:** high effort; run the full closing gate once (`cargo test --workspace`, clippy, fmt, typecheck, `npm test`) and put the stamp in the handoff.
- **Reviewer (time-boxed, still correct):** do not re-run `cargo test --workspace`. Run this phase's new test file(s), the test files of the crates and features the diff touches, clippy, fmt, typecheck and the touched UI test files; trust the implementer's full-suite stamp unless it is missing or contradicts what you see. Read every changed production path against the rows; only B1–B4 findings block. Aim to finish in about 15 minutes.

## Review log

### 2026-11-10 — implement r1 (Claude Opus, `pipeline-implementer`)
**Built.** (#22) `CaptureOneScripting::is_running` (the process list, never an Apple event; JXA side: `ps` with a 200 ms cache) and one gate
check in `GatedScripting`: off the take-back's permitted thread, while the take-back holds the gate or a relaunch waits for the camera
helper with Capture One not running (P54), every script but `launch`/`is_running` answers `EditorLaunchDeferred` without running
(`editor_info` and `quit` included while the take-back holds it; `launch` keeps its start-order wait). The follower's and the take-back's
"is it running?" polls use `is_running` (the follower polled `editor_info.js` on every 50 ms tick, and the take-back's step 3/4 polls did
too — `running()` true, then `captureDoneScript()`/`version()` reaching a Capture One that finished quitting relaunches it: the 0.35 s
relaunch and the false "did not quit"). The ensure treats a refused `editor_info` as deferred. (#27) `document_present` (folder + a
`.cosessiondb`) and `DOCUMENT_MISSING`; a session chosen in Settings that is missing is reported by the ensure (health `document_missing`,
never `make_document`), the receiver's capture root is none (uploads → 451), the helper is not spawned (no staging folder) until the
session is there, and the runtime tick names it in Health and in the header (`NeedsActionReason::DocumentMissing`). (#28) the runtime tick
restarts the helper with the current session's staging (`--save-dir <new>/Capture/.incoming/sdk-<run>`, cached `--ip/--mac` kept) when the
session changed, never during a take-back, an upload or a Save-to-PC transfer. (#4) sticky Save footer; leaving Settings with unsaved changes
(Back, Escape/⌘[, Health) asks `Save your changes?` Save/Discard/Cancel through a leave guard in `lib/routes.ts`. (#5) helper: the SDK adapter
caches the endpoint of its own Wi-Fi connect; after a lost session a failed cached connect is retried at once for 5 s, then discovery starts
at once; an empty Wi-Fi discovery while recovering is retried at once (no backoff). (#6) `EditingProvider::styles` + Tauri `editor_styles`
(null when unreadable) + Settings → Style drop-down (No style, your styles, built-in; a stored name not listed stays selectable). (#7) see
Deviations: the p48 time was App shutdown joining the spool thread stuck in a 11–22 s FSEvents watcher start; shutdown now waits ≤ 1 s for it.
**Cause of the 11.8 s unplug recovery** (app log 2026-11-10 13:40:12–24Z): lost +0; Wi-Fi discovery +1.06 s → empty after 3.68 s; backoff
1.0 s; discovery +5.8 s → found after 3.84 s; connect 2.2 s → 11.84 s. No cached endpoint existed; the camera's Wi-Fi was already up (FTP
uploads arrived at +3.7 s). With the fix (fake at those timings): cached endpoint ≤ 6.1 s; stale endpoint ≤ 5 + 3.8 + 2.2 s; none ≤ 10.8 s.
**Assumptions.** The "missing" rule applies to a session chosen in Settings (`document_id` set); the app's own session is still created
when absent (10C step 2). The header sentence shows on the direct camera's status (the header's camera line); with the Capture One
camera adapter only Health shows it. Protected operations for the helper restart = take-back running, an FTP upload open, a Save-to-PC
transfer expected. `editor_info` is refused only while the take-back holds the gate; in the P54 window it still runs (the ensure needs it to
reach `launch`, whose wait is the explicit launch step). The style drop-down lists only names, split by the `.costyle` files under
`~/Library/Application Support/Capture One/Styles`.
**Deviations.** Files outside the listed perimeter: `crates/core/src/{domain/camera.rs, domain/editing.rs, mechanisms/camera_status.rs,
ports/editing.rs, services/editing_startup.rs}`, `crates/integrations/src/lib.rs`, `crates/integrations/src/editing/capture_one/{jxa.rs,
scripting.rs, provision.rs, jxa/list_styles.js (new)}`, `crates/testkit/src/fake_capture_one.rs`, `apps/desktop/src-tauri/src/{lib.rs,
commands/settings.rs, commands/camera.rs}`, `apps/desktop/src/{services/bindings.ts (generated), services/styles.ts (new), lib/routes.ts,
lib/keyboard.ts, features/shell/PageHeader.tsx}`. `p48_camera_ownership.rs`: besides no timing edit being needed, `c3c` now runs on the
app's own (not yet created) session — a deleted *configured* session is no longer recreated (#27), so the make-document path it guards only
exists for the app's own session; its assertions are unchanged. R-I20 "staging never moved for this process" is superseded by #28.

### 2026-11-10 — review r1 (Opus, `pipeline-reviewer`): APPROVED_WITH_NOTES
**Gate.** Tracker `IMPLEMENTED`; checkpoint `0247c78` in `git log`. **Perimeter:** `git show --stat 0247c78` = 35 paths = the handoff's
declared list (27 production, 5 tests/harness, tracker, plan log); 23 sit outside the plan's named files, all disclosed and needed by the rows.
**Evidence run by the reviewer** (time-boxed per Review mode; tree = `0247c78`): `p63_final_check_fixes` 4/4, `p48_camera_ownership` 32/32 in
10.3 s (C6(a)), `p54` 6/6, `p57` 8/8, `p60` 8/8, `p62_retention_after_publish` 12/12 (the implementer's full-run failure does not reproduce),
`cargo test -p psm-core -p psm-integrations -p psm-desktop` green, clippy `-D warnings` clean, fmt clean, typecheck clean, `npm test` 64 files
442/442, `tools/sidecar/build.sh --stub --test` 31/0. The full workspace run is trusted from the implementer's stamp (partial, see notes).
**Verified correct.** C1(a): `GatedScripting::call` refuses every script off the permitted thread while the take-back holds the gate or the P54
wait applies with Capture One not running; `launch` keeps its start-order wait; follower and take-back polls use `is_running` (`ps -axo comm=`,
the same process name the ownership check already uses); `quit`/`editor_info` refused while the take-back holds the gate; the test's fake
relaunches on any Apple event and the ops between `quit` and `launch` must be exactly step 5's `editor_info`, so a follower or render script
fails it. C2(a): `document_present` (folder + `.cosessiondb`) gates the receiver root (451), the helper spawn (no staging folder) and the
ensure (`document_missing`, no `make_document`) for a session chosen in Settings; the test asserts sentence, header reason, 451 and the path's
absence after an activation. C3(a): `check_document` restarts the helper when the staging differs, never during a take-back, an open upload or
an expected transfer; old staging removed only if empty (H2); test holds an upload open, sees no restart, then the new `--save-dir`.
C4(a): sticky footer; `goTo` leave guard with Save/Discard/Cancel; tests cover Cancel stays, Discard leaves unsaved, Save saves then leaves,
no change no prompt. C5(a): cached-endpoint retry window and no-backoff recovery in `transport_manager.cpp`; the valid-endpoint case asserts
`recovery_ms ≤ 6100` at the log's SDK costs; cause recorded in the handoff. C6(b): options `No style`, user, built-in in order; exact name saved;
unreadable → text + sentence; backend split and no op on a closed Capture One. C7(a): green as above. Assumptions (chosen-session-only rule,
header on the direct camera only, protected-operation set, `editor_info` in the P54 wait, recovery windows, user styles by `.costyle` files,
unlisted stored name kept) are consistent with 10C/10E and the plan. H1–H8 hold; H7 placement of `document_present` in core is a note.
**Notes (backlog):** H7 placement of the `.cosessiondb` check; only the valid cached endpoint meets ~6 s (stale ≈ 11.5 s, none ≈ 10.8 s, real
SDK failure time unmeasured); the restarted helper spawns without `--ip/--mac`; leave guard covers Back/Escape/⌘[/Health only; `availableStyles`
includes presets; shutdown leaves a stuck spool thread behind; partial closing stamp; book/registry drift (R-I20, new names, p48 c3c fixture).
