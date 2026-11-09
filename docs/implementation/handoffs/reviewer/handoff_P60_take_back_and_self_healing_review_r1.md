---
plan: P60_take_back_and_self_healing
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-09
actor: opus (pipeline-reviewer)
---
# Review — P60 take-back fixes and self-healing for deleted files (round 1)

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. Nine notes went to the backlog. The narrative is written. Nothing needs an owner answer to proceed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. One product choice (a photo whose file was deleted still blocks its item's export/publish) is put to the owner as a "Choices made for you" line in the narrative, with the sentence to say to change it. The current behaviour is safe: nothing wrong is published.

## Gate
- Tracker row read `IMPLEMENTED`. Checkpoint `6ed79bfaf14736d75167ff8c8e3572b1e1c5af19` is in `git log`.
- `git show --stat 6ed79bf`: 23 paths. They equal the handoff's declared list exactly: 18 production, the fake Capture One, 2 tests, the tracker row and the plan Review log. Nothing extra, nothing missing.

## Stamp (reviewer run, code tree `6ed79bf`; HEAD `2808183` adds only pipeline docs; porcelain clean before this review's own files)
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | 874 passed, 0 failed (59 binaries; the `p53` C3(a) intermittent did not recur) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` (apps/desktop) | clean |
| `npm test` (apps/desktop) | 60 files, 424/424 |

## Verified correct
- **C1(a) no launch during the take-back.** `LaunchGate::hold_quiet` + `quiet_here` refuse every lane script except `editor_info`/`quit` off the permitted thread. `GatedScripting::lane` increments `in_flight`, then re-checks quiet inside the lane. The take-back stores quiet, then `wait_idle` reads `in_flight`. Both are SeqCst, so a script either sees quiet and is refused, or is counted and drained (≤ 30 s) before save/quit. `PreviewService::held()` stops `request` and `tick` while the gate is held. A deferred `send_next` puts the render back at the head of `pending`. Test: the fake relaunches on any Apple event to a quit Capture One, with a script in flight at the start. It asserts no launch before the cable session, exactly one launch, after the quit, and every render's first `process` after the launch. An early relaunch would also make step 3's `wait_quit` fail, which C1(b) catches.
- **C1(b) no false failure.** `TakeBackReport { completed_steps: 5, camera_held: true }`, status on Cable, and "did not quit" in neither status nor Health.
- **C2(a) dialog named, same plug-in retried.** A quit error or a `wait_quit` timeout sets `CAPTURE_ONE_WINDOW_OPEN` on the adapter. The runtime turns `TakeBackFailed{step:3}` into a retry 2 s later for the same plug-in (`auto_taken_this_plug` reset), keeping the message. The test asserts the exact sentence, no launch while ignored, then on cable, one launch, ≥ 2 quits and exactly one `usb.arrived`. The UI test shows the sentence in the header card.
- **C3(a) stuck camera service.** Named only when ≥ 10 s have passed since the quit, a claim has arrived since the quit, the cable is still held and `known_running` is empty. The test asserts the exact sentence, ≥ 9.9 s after the observed quit, and still present after step 5.
- **C4(a) deleted file heals.** `check_missing_files` → `missing_decision` → `RawMissing`, or `IntegrityCheck::verify` when the file is back (same bytes → `Ok`). `sessions.view` and the session-list count exclude `raw_missing`. Checks run at `on_startup`, `on_focus`, the follow tick's reopen edge, `collection_switched`, `request`/`request_initial`, `defer` and `schedule_due` (disk change since the last check). The test bites without the view filter (3 in view) and without the pre-check, because the startup catch-up would ask Capture One for `DSC06101`. Health detail asserted exactly. The restore brings back 3 photos with integrity `Ok`.
- **C4(b) missing files never block.** `renders_in_flight` calls `discard_missing`, which drops gone RAWs from `jobs` and `pending`, so the take-back precondition never reports "a preview is rendering". The test fails otherwise, because the fake never finishes those renders.
- **C5(a) no storm.** `render_eligibility` returns `Current` for non-stale states, and `on_editor_opened` runs only `schedule_due(false)`. Asserted `sent == ["process_started:v-new:PSM Preview"]`.
- **C6(a) recipe self-repair.** A refused `process` with `recipe_exists == false` → one `ensure_recipe` + one retry. A non-JPEG preview export → `recipe_fault` → `recipes_checked` cleared → repair + one re-send (`recipe_retried`); a second wrong export fails the preview with code `recipe`. Asserted on the fake's ops (`make_recipe`, `set_recipe_props …outputFormat`, `process_started` counts), the stored bytes of the retry, and no further `process` and no `recipe_missing` over 30 s.
- **Core mechanisms.** `mechanisms::missing::{file_missing, missing_decision, render_eligibility}` match the plan's Core mechanisms list exactly. Input → output unit tests, no orphans.
- **Over-engineering test:** passed. Tests observe stored state, the service view, Health, camera status and the calls the fake received. None asserts call counts of internal collaborators or private helpers.
- **Assumptions 1–12:** none contradicts the book. Assumption 5 (export/publish unchanged) is MC-3's rule ("a missing file marks `raw_missing` … fails the publish item at prepare").
- **H1–H8:** H2 holds: no RAW is opened for writing or deleted, and the hash is re-read read-only. M39 / the plan's hard rule holds: only Capture One's `quit`, no kill. H5 holds: AppleScript only. H7 holds: core gained `missing.rs`, a `Hold` eligibility and `recipe_fault` on the editing port, with no device or app names outside comments. H8 holds: no Tauri in services. No live services in the suite.

## Findings
None blocking (B1–B4).

## Notes written to backlog (`docs/implementation/backlog.md`, "P60 reviewer r1")
1. A missing photo is hidden but still verified by export/publish (MC-3), so its item cannot be exported or published, and the photographer cannot see the cause. Owner choice: offered in the narrative.
2. C4(a)'s "variant_not_found" half is reached by the preview pass's disk check. The refusal path (`defer` → check, adapter `discard_missing` → `file_missing` refusal) is untested; today it is redundant.
3. `check_missing_files` stats every RAW and runs one query per RAW on every focus and reopen: a cost on large libraries.
4. `recipes_checked` clears when the queue empties, so on a quiet shoot every arrival's preview pays an `ensure_recipe` first.
5. The dialog retry loop: save + quit + a quiet gate held for up to 20 s, every ~22 s, while a Capture One window stays open.
6. A failed manual take-back can leave the dialog sentence up after the window is closed.
7. → P62: the retention clean-up must not be counted as "deleted outside Session Manager".
8. Registry and 10E MC-34 drift (new names and rules listed in the note).
9. Environment: `/tmp/p60head-target` (2.1 GB) left by the implementer; the `p53` intermittent.

## Lessons for the plans
- C4(a)'s input "Capture One answers `variant_not_found`" and its outcome "no render request for it ever" can both hold only if the lookup happens without a render request. An implementation that checks the disk first never reaches the input. A row naming a trigger should name an observable that proves the trigger fired, or name only the outcome.
- Changelog 43 ("leaves the session views") and MC-3 ("a missing file fails the publish item") together create a dead end the plan did not address. Plans that hide something from the photographer should say what happens to it downstream (export/publish).

## Write perimeter (this session)
- `docs/implementation/master_plan.md`: P60 tracker row only (→ APPROVED).
- `docs/implementation/plans/P60_take_back_and_self_healing.md`: Review log entry appended.
- `docs/implementation/backlog.md`: 9 bullets appended.
- `docs/implementation/narratives/P60_take_back_and_self_healing.md`: new.
- This handoff.
- No production or test file touched; no probes; no commits. The full suite, clippy and fmt rebuilt only `target/`.
