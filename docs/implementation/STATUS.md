# STATUS — read first (updated by the orchestrator at every stop)

**2026-11-11 08:40: P65 launched (Codex first, Claude if out; time-boxed review).** First work-site publish (owner at work, SDK build on the dev Mac): failed `variant_not_found` because Capture One took 17–80 s to list new photos; retry succeeded in 13 s (P64 grouped calls verified on the real store). Owner: style slow, jobs should queue, inventory tracking + location + quantity with a remembered default (changelog 45). P65 plan written and pushed.

**2026-11-10 22:00: SDK app built from HEAD (P53–P64) and installed in /Applications on the dev Mac** (old P62 build moved to the session scratchpad; repo back on the stub). Waiting for the owner: publish on the test store, style drop-down, cable plug/unplug; then the work-Mac install.

**2026-11-10 21:55: P64 APPROVED_WITH_NOTES, then P63 APPROVED_WITH_NOTES (both on Claude Opus high effort after Codex limits; time-boxed reviews).** Narratives relayed. Owed: one full `cargo test --workspace` run on a quiet Mac (both stamps partial); P64 batched Shopify calls not yet run against a real store; P63 unplug ~6 s only with a valid cached endpoint (stale ≈ 11.5 s). Next: build the SDK app from HEAD, install on the dev Mac, try publish on the test store + the P63 checks, then the work-Mac install (retention days high until P62 is verified on the test store).

**2026-11-10 15:35: owner: go, fast (within an hour).** Order changed: P64 (publish) first, then P63 (camera/settings). Both `difficulty: high`; plans carry a time-boxed review mode (implementer runs the full gate once; reviewer runs touched tests only). Codex first, Claude when out.

**2026-11-10 15:20: owner: fix before the work-Mac install.** First real publish: unreviewed photos skipped; republish slow (615+ `FilesAll` pages per condition-photo lookup; one-at-a-time uploads); no progress detail; no cancel. Style typed by hand (case mismatch). P63 += style drop-down; new P64 (publish speed/visibility, cancel, unreviewed published; changelog 44). Waiting for the owner's go to start P63 → P64 (owner asked not to start until their corrections are in).

**2026-11-10 14:50: P63 paused by the owner before any code** ("do not start that fix yet, as I might correct some things also before this"). Codex was out; the Claude implementer was stopped within seconds; tree clean. Restart P63 (fresh) only after the owner adds their corrections to the plan.

**2026-11-10 14:50: final hands-on check on the dev Mac passed (evidence §Final hands-on check).** Owner chose option A: installs the current build on the work Mac now; P63 (small fixes) runs in the background (Codex, Claude if out). App in /Applications on the dev Mac = build of P53–P62.

**2026-11-10 14:00: dev Mac cleaned by the owner (fresh start).** Deleted: Session Manager DB, previews, backups, logs; Capture One session `~/Pictures/session test 1` and other test folders/catalogues; the Capture One settings backup. Kept: `settings.json` (still points at the deleted document — owner creates a new Capture One session and picks it), camera support + Keychain + fingerprint. Free space 64 → 96 GB. The evidence above refers to files that no longer exist.

**2026-11-10 13:30: P62 APPROVED_WITH_NOTES (re-review r3 after the authorised fix 2). P60–P62 done; storage (10F) implemented.** SDK app built from HEAD at `apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app` (not yet installed). Next: owner installs on the work Mac today; first run P62 steps 3–8 on the test store before relying on retention. Backlog: p48 tests take ~12 min since P60's real-time take-back waits.

**2026-11-10 08:50: overnight run: P60 APPROVED, P61 APPROVED (both on Claude after Codex limits); P62 re-review r2 CHANGES_REQUESTED with one MUST (F2: a RAW captured/imported into a cleared item after clearing would be deleted).** Orchestrator wrote FIX2_AUTHORISED (created_at <= cleared_at guard + post-clear arrivals go to Unassigned) and relaunched (Codex, Claude if out). Then: build the app for the work-Mac install (owner, today).

**2026-11-09 19:30: owner: run overnight, storage tonight, work-Mac install tomorrow.** Driver `overnight.sh` (session scratchpad) runs P60 → P61 → P62: each phase starts on Codex; a phase stopped by a Codex limit/outage is resumed once with `PSM_IMPLEMENTER=claude`; any other stop waits for the orchestrator. Owner standing permission: Codex first, Claude when Codex is exhausted, Codex again for the next phase.

**2026-11-09 19:15: evening hands-on check of P57–P59 done (evidence §Hands-on check).** Automatic take-back works after a Mac restart (12.5 s, no click); fast compose works. Owner: fix the take-back issues and self-healing for files deleted by hand, then storage, before the work Mac. P60 (take-back + self-healing + recipe self-repair, changelog 43) running on Codex; P61 (storage clean-up + budget) and P62 (retention after publish) written, to run after P60. Then the work-Mac install.

**2026-11-09 17:50: P59 APPROVED_WITH_NOTES (re-review after fix 1).** Narrative relayed. SDK app built from HEAD (P53–P59) at `apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app`; the install-and-zip command was declined, so it is NOT installed in /Applications yet. Next: owner tries it here, then installs on the work Mac (steps given in chat: support from Sony's zip, new FTP password + camera Server host = work Mac address, fingerprint, Capture One document, shop credentials). Storage (10F) ratified, not planned; item type later moves to `beyo.item_type`.

**2026-11-09 15:20: P58 APPROVED_WITH_NOTES (re-review after fix 1; Opus implement, Sonnet fix).** Narrative relayed. P59 (item type from Shopify product Type, changelog 42) launched on Codex. Then: build the SDK app, owner tries it, work-Mac install. Storage chapter 10F ratified, not planned.

**2026-11-09 11:05: P58 implement stopped: Codex Sol worked ~15 min (30 files changed, uncommitted, backup patch in the session scratchpad) then hit its limit; Terra also limited until 13:09.** A background wait reruns P58 at 13:11 on Codex, continuing on the partial tree. Owner's Claude allowance is nearly used up, so no Claude implementer fallback unless the owner says so.

**2026-11-09 10:45: P57 APPROVED_WITH_NOTES (re-review r2 after fix 1, tests only).** Narrative relayed. P58 launched (Codex). After P58: build the SDK app for the work Mac, install steps for the owner.

**2026-11-09 09:20: owner: implement the fast preview before the work-Mac install.** P58_fast_compose_on_camera_preview written (Opus; changelog 39): compose on the A7R V's full-size embedded JPEG at once, swap to Capture One's preview in place, render queue focused → newest → rest. Runs after P57 (Codex). Work-Mac install waits for P58.

**2026-11-09 08:30: P52 run 2 done with the owner (evidence §Acceptance run 2).** Most rows pass after P53–P56. Hard facts: ptpcamerad (started with Capture One) claims the cable at enumeration, so the SDK cannot win; the camera has no Wi-Fi remote while the cable is in. Owner decisions (changelog 38): automatic take-back on plug-in (setting, default on); direct grab / own PTP deferred (backlog). P57 (automatic take-back + F15–F21) written on Opus and launched on Codex. Then the first install on the work Mac for real sessions; storage + faster previews by 2026-11-12. The scratchpad was cleared between sessions; results now live only in the evidence file. Output JPEG quality was left at 91 (was 90) → reset after P57.

**2026-11-08 20:55: P56 APPROVED_WITH_NOTES (first review; implemented on Claude Opus after Codex hit its usage limit). All P52 fix phases done (P53–P56).** Narrative relayed. Owner's schedule: **2026-11-09 P52 run 2 with the owner** (camera; remaining rows A2, A8, A11, A13, A16, A18 quit + reruns of A0, A1, A3, A6, A10, A12, A15, A17, plus plug-in claim, holder release, 1 h never-idle, no export loop); then the first install on the work Mac for a real workflow session; **by 2026-11-12 storage policy + faster previews (F11)** (owner's Claude allowance ~96 % used: keep orchestrator token use minimal; Codex for implementation). The SDK app build + install for run 2 is the first step tomorrow (not pre-built: the build command was declined tonight).

**2026-11-08 19:10: P55 APPROVED_WITH_NOTES (first review). P56 implement r1 BLOCKED on two contract cards (fast USB claim drops Wi-Fi; USB retry after the three-failure fallback).** Orchestrator ruled both from the owner's instruction ("fast, flow not disturbed"): (1) yes, close Wi-Fi and try USB ≤ 300 ms at plug-in, Wi-Fi back at once on failure; (2) retry USB only on the holder's release (`UsbExclusiveOwner` cleared), no blind probing. Plan §Orchestrator rulings, book changelog 37; export-loop diagnosis (Capture One's own `.cos` touch counted as an adjustment) folded in. P56 relaunched (Codex). Narratives P53–P55 relayed. Owner may override the rulings.

**2026-11-08 18:10: P53 APPROVED_WITH_NOTES (first review), P54 APPROVED_WITH_NOTES (re-review r2 after fix 1).** Narratives relayed. P55 implement stopped at once (Codex: "workspace routing discovery failed", no changes); rerun launched with P56 chained (Codex). If Codex fails again → `PSM_IMPLEMENTER=claude` (owner standing permission). Session Manager stays closed until P56 (export loop). Then P52 run 2 with the owner.

**2026-11-08 16:45: disk emergency.** The Capture One export loop (F12) wrote 860 `PSM Master` TIFF copies (64 GB) into `~/Pictures/session test 1/Output/`; the disk reached 167 MB free. The orchestrator quit Session Manager (keep it closed until P56) and, with the owner's OK, deleted exactly those 860 TIFFs (mtime 2026-11-08, Output root only; the 178 older TIFFs, previews and all RAWs kept). Disk now 60 GB free. Added to P56: C3(d) master renders do not pile up + a 5 GB free-space guard. P56 (claim at plug-in, never idle, export loop) runs after the P53–P55 chain. Storage policy (none exists in the book) → owner card after the fixes.

**2026-11-08 15:10: P52 run 1 (dev Mac, owner) stopped after A15 at the owner's request: "better to fix what we have gathered and later when fixed we finish the tests".** Pass: A4, A5, A7, A14, A18 (USB, Wi-Fi), A9 so far, A12 (restart ~5 s). Fail: install from Sony's download (F1), Wi-Fi cold start shows no Connecting (F3), no capture button (F4), no Wi-Fi while ptpcamerad holds the cable (F6), take-back blocked by deferred renders (F7), camera FTP stops after a receiver outage and does not re-send (F8, supersedes 10E A10), double uploads from the late 226 (F9), Capture One relaunch hands the cable to ptpcamerad (F10). Evidence §Acceptance (P52, 2026-11-08); P52 Review log. The orchestrator wrote P53 (upload + log), P54 (cable + take-back), P55 (setup polish) and launched them in a chain (Codex). F11 (preview speed) → backlog, owner card after P55. Then P52 run 2 (remaining rows + reruns), then the work Mac. Outside the repo: the SDK build of the app is in `/Applications`; the support folder was installed by the app; the repo has the stub sidecar.

**2026-11-08 11:31: P51b APPROVED_WITH_NOTES (first review). Confirm now pins exactly the shown fingerprint; N5 title fixed. Camera chapter: only P52 left.** Narrative relayed. P52 needs the owner (camera, cable, Capture One; the SDK zip is in `~/Downloads/CrSDK_v2.02.00_20260610a_Mac.zip`). Before step 1 the orchestrator builds the SDK app and removes the P47 hand-installed support folder.

**2026-11-08 10:40: owner: fix the fingerprint confirm before P52.** The orchestrator wrote `P51b_fingerprint_exact_confirm` (Confirm carries the shown value; the helper pins only that value; N5 title) and launched it (Codex). Then P52 with the owner.

**2026-11-08 10:20: P51 APPROVED_WITH_NOTES (re-review r2, after fix 1). Camera chapter 10 of 11.** Implement stopped BLOCKED once on old P07/P19 camera-dot assertions (ruled `d64eeec`); review r1 F1: fingerprint Confirm was offered without the value (fixed `6145db7`). Narrative relayed. Next: P52 hardware acceptance (owner + camera, ~1 h, on the work Mac, incl. S1(c)/S3). Before P52, consider the P50 note (confirm trusts the next fingerprint, not the displayed one) and P51 backlog N5/N6. Waiting for the owner.

**2026-11-08 09:05: P50 APPROVED_WITH_NOTES (re-review r3, after fix 2; F3 Keychain read gated on camera support). Camera chapter 9 of 11.** Note to watch: confirming a fingerprint trusts the next one the camera shows, not exactly the one displayed (backlog; consider before P52). P51 launched (Codex).

**2026-11-08 06:50: P47 spike DONE with the owner (camera chapter 8 of 11).** Cable and Wi-Fi both connect from the packaged app; Wi-Fi shot → app ~10 s. Main finding: Keychain prompts (first use, and 3–4 after every ad-hoc update) block the sidecar spawn, so Capture One won the cable twice → folded into P50 Notes (d) with S2/S3/S7 answers. Owner set camera FTP Server 2 for Session Manager (`sessionmanager`, port 2121). Support-folder sidecar parked again. Next: P50 (Codex), then P51, then P52 (owner + camera, ~1 h, on the work Mac incl. S1(c)/S3).

**2026-11-07 21:40: P47 prep done (steps 1–2, no camera).** Packaged app built; Sony support folder hand-installed under `~/Library/Application Support/com.loorenz.sessionmanager/camera/sony/2.02.00/` (outside the repo). **S2 answered offline: hardened runtime + ad hoc refuses Sony's dylibs; `disable-library-validation` on the sidecar fixes it** (→ P50). The sidecar copy is **parked** (`camera-sidecar.parked`) so the owner's everyday app keeps using the Capture One adapter until the spike. Stub restored, repo clean. Nothing else runs without the owner: P47 (≈15 min with the camera) gates P50–P52.

**2026-11-07 21:05: P49 APPROVED_WITH_NOTES (re-review r3, after the authorised fix 2). Camera chapter: 7 of 11 done (P42–P46, P48, P49).** Next: P47 (owner + camera, ~15 min) gates P50 → P51 → P52. Tonight the orchestrator prepares P47 steps 1–2 (packaged build with the SDK sidecar, hand-installed support folder under `~/Library/Application Support/com.loorenz.sessionmanager/camera/sony/<ver>/`, ad-hoc signed; nothing in the repo; stub sidecar restored afterwards). Keychain items and steps 3–10 wait for the owner.

**2026-11-07 20:25: P49 re-review r2 CHANGES_REQUESTED (1 MUST, F2): fix 1 made the style-eligibility read error on NULL, so tethered Capture One previews would never render while a style is set. Orchestrator authorised fix 2 (one SQL line + one test, `766178d`); running on Codex (gpt-6-luna).** Owner (evening): continue overnight independently; answer from the intention; switch to `PSM_IMPLEMENTER=claude` if Codex runs out. After P49 the camera chain waits on P47 (owner + camera); tonight the orchestrator prepares P47 steps 1–2 (packaged build, hand-installed support folder outside the repo; no Keychain items, no camera).

**2026-11-07 18:55: P48 APPROVED_WITH_NOTES. Owner: "we can continue with codex as the session has renewed".** P49's Claude implement session was stopped after 49 s (no changes) and relaunched with Codex (runner default). P47 waits for the owner + camera; P50 needs P47 and P49.

**2026-11-07 18:30: P48 implement r1 finished (handoff DONE) but lost its network before the checkpoint commit; the review stopped BLOCKED at the gate.** The orchestrator verified the tree (fmt, clippy, workspace tests, typecheck, UI 369) and committed checkpoint `4da7fe9`, then filled the handoff's SHA. Rerunning `RESUME=1` P48 (review r1) → P49. P47 waits for the owner + camera.

**2026-11-07 12:30: P48 projection r0 AMENDMENTS_REQUIRED (L1–L25, 0 owner cards).** Main issues: six rows would hang, wait 45 s of real time, or pass on wrong code; a deferred ensure would raise a false health warning; take-back step 5 would wait an extra 15 s. The planner is folding them. Next: `PSM_IMPLEMENTER=claude RESUME=1` P48 → P49; P47 waits for the owner + camera.

**2026-11-07 12:15: P46 APPROVED_WITH_NOTES (first review, Claude Opus). P42–P46 done.** P47 (support package spike) needs the owner and the camera (~15 min); the owner is away. P48 and P49 depend only on P46, so the orchestrator is running `PSM_IMPLEMENTER=claude` P48 → P49 now (no camera, no Capture One in tests). Then P47 with the owner; P50 needs P47 and P49.

**2026-11-07 11:00: P46 projection r0 AMENDMENTS_REQUIRED (L1–L22, 0 owner cards; main issue: the test clock starts in 1970 while the scripts are stamped 2026).** The planner is folding them (L13: the camera's FTP report goes into the "not received" health detail, in P46). Next: commit, `PSM_IMPLEMENTER=claude RESUME=1 tools/pipeline/run_phase.sh phase P46_direct_camera_adapter`; stop before P47 (owner + camera). P45 narrative relayed.

**2026-11-07: P45 APPROVED_WITH_NOTES (first review, Claude Opus implementer). P46 projection r0 cut off: connection dropped (the owner's Mac was closed), no handoff written.** Rerunning `PSM_IMPLEMENTER=claude` P46 from the projection; stop before P47 (owner + camera). The owner is away about 2 h; answer from the book.

**2026-11-06 19:15: P45 plan amended (L1–L23 folded: 21 rows, 42 paths, `projection: done`; P46 consistency edits for the `ConnectFailed.error` field and the staging path).** The projection's scratch build `/tmp/p45proj` (Sony dylib copies, outside the repo, 53 MB) was deleted by the owner. Running `PSM_IMPLEMENTER=claude` P45 → P46; stop before P47 (owner + camera).

**2026-11-06 18:55: P44 APPROVED_WITH_NOTES (first review, Claude Opus implementer); narrative relayed. P45 projection r0 AMENDMENTS_REQUIRED (L1–L23, 0 owner cards).** The planner is folding them (plus §6.11 and any P46 consistency edits). Next: commit, `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase P45_sidecar_and_protocol`, then P46; stop before P47 (owner + camera).

**2026-11-06 17:45: owner: "we can continue implementation using claude agents".** Running `PSM_IMPLEMENTER=claude RESUME=1` P44 → P46 (Opus implements, Sonnet fixes; no Fable). Partial Codex work remains in `stash@{0}`, not used. Stop before P47 (owner + camera).

**2026-11-06 17:45: P44 plan amended (L1–L24 folded, 36 rows, `projection: done`; L18 moved to P46 as C8(c)).** Planner reading (5)–(6) accepted: rules (1)/(2) carry the prior's session even when it had no exposure. Running P44 → P46; stop before P47 (owner + camera). Known stale anchors ("code at 41bef14") in P45/P46: the projection and implementer check at HEAD.

**2026-11-06 17:15: P43 APPROVED_WITH_NOTES (after fix 2); narrative relayed. P44 projection r0 AMENDMENTS_REQUIRED (L1–L24, 1 card).** Card 1 (counter wrap matched new photos to old sessions) answered from the book: R-I26 clarification, changelog 36. The planner is folding L1–L24 (L18 → P46). Next: `projection: done`, commit, `tools/pipeline/run_phase.sh phase P44_capture_ledger`, then P45–P46; stop before P47 (owner + camera).

**2026-11-06 16:35: P43 re-review r2 CHANGES_REQUESTED with one MUST (F3: test `c7b` waits only 3 s for the fake sidecar pid; suite red ~25 % under parallel load; receiver correct).** Orchestrator: FIX2_AUTHORISED (test file only, 15 s pid wait). The runner now skips a DONE fix r2 and review r2 on RESUME. Running: `RESUME=1 … P43` (fix 2 + re-review 2), then P44–P46; stop before P47 (owner + camera). P42 narrative relayed.

**2026-11-06 15:02: direct camera adapter — P42 APPROVED; P43 implement r1 DONE, review cut off by the usage limit (resets 15:30).** Next: `RESUME=1 tools/pipeline/run_phase.sh phase P43_ftp_receiver`, then P44–P46; stop before P47 (owner + camera). R-I18–R-I38 approved by the owner (changelog 35). Still owed from P41: owner checks of steps 4 and 5, and the re-frame-on-undo choice.

**2026-11-05 21:01: P38–P41 APPROVED** — editor polish (R40) and undo/redo history MC-30 (R41). P39 and P40 projections amended (0 owner decisions); P39 C4(h) ruled by the orchestrator from MC-30. Owner checks owed: P41 step 5 (⌘Z inside a text field must undo typing only, not a history step); the open choice "undoing a re-frame should ask me to re-frame again". App build for the owner to install with ditto.

**2026-11-05 13:10: P37_editor_chrome APPROVED_WITH_NOTES** (first review) — the owner's second editor corrections (10D, R39): icon toolbar with name/key/description hover cards, Tools panel hide button in its own header with a left rail, Rotate and flip drawer, Turn and grid icons, folded nudge pad, frame meter without a button. Plan e04393f; implementer Codex. Leftovers: the stage ⓘ and ⋯ buttons still use the old tooltip; the flaky P33 stage-bar test.

Updated: 2026-11-04 by the orchestrator.

**Composition corrections done (2026-11-05 09:57): P35 and P36 APPROVED (with notes, 0 owner cards); narratives relayed; app rebuilt into /Applications.** P27–P36 APPROVED. Owner 2026-11-05: no Fable for implementation until the owner says so. Owed by the owner: on-screen try-out of P35/P36, P34 step 8 timings; rotate the Shopify client secret (displayed in chat by mistake, never committed). Known leftovers: a flaky P33 stage-bar place-count test under the full suite; typed straighten dropped if a drag starts before Enter; one shared save queue for all editor actions. Next: the owner's new intention `docs/intention/improved-session-manager/` (untracked, excluded locally). Owner's camera FTP/SDK tests live outside the repo in ../Camera-Connection-Tests.

## Where things stand
- Intention: **RATIFIED** 2026-10-24 (`docs/intention/intention.md`).
- Mechanism inventory: **done** 2026-10-25 (exit 10, state OWNER_DECISIONS_PENDING, one card I1).
  20 contracts MC-1…MC-20 in 05A/07A/10A; 17 resolutions R-I1…R-I17 listed for ratification in
  `handoffs/reviewer/handoff_intention_inventory_r1.md`. Scripted capture now WORKS: autofocus first,
  then `capture` (evidence VERIFY-12, 2026-10-25); the inventory's capture-dependent probes are
  still to be run by the first phase that needs them. Colour-tag mapping confirmed by
  the owner 2026-10-25.
- Master plan and 17 phase plans: written 2026-10-25 (exit 10, OWNER_DECISIONS_PENDING, 2 cards).
  Order in `phases.txt` (P00…P16); Shopify phase P12 needs the token.
- Phases: P00–P11 APPROVED 2026-10-25/26 (most with notes; narratives relayed). P04 projection r0 →
  amendments applied by the orchestrator (plan Review log). P00 Automation prompt check PASSED with
  the owner 19:25 (evidence packaging_spike.md).
- P12 (Shopify): projection r0 (2026-10-26 00:18) → AMENDMENTS_REQUIRED, ledger L1–L25 in
  `handoffs/reviewer/handoff_P12_shopify_adapter_projection_r0.md`; applied 2026-10-26 (commit a1cf9b4: P12 59 rows,
  P13 task 0 wiring + C9, master plan, 07A/09 grounding, changelog 17). Owner answers folded in
  (book 09A, R32, changelog 18; P12 68 rows / 9 criteria, P13, P14 21 rows, P16, master plan). P12 APPROVED (with notes) 2026-10-26 06:02 on Claude; narrative relayed.
  P13 APPROVED (with notes) 06:52 on Codex. P14 fix r2 → OWNER_DECISIONS_PENDING (tag merge base);
  answered from 07A MC-13's intent (option A, R33, changelog 19), C2(c) amended + C2(d) (22 rows), attempt archived.
  P14 APPROVED (with notes) 07:29 on Codex. P15 implement r1 (07:37) BLOCKED: failure/upload-state/notification
  data not exposed by commands → plan amended (task 0 backend reads, C7(a)–(c), 14 rows; ch04 S-G aligned with N2,
  changelog 20); attempt to be archived by the orchestrator, next implementer continues from `e3848bd`.
  P15 attempt archived 07:41; relaunch → re-review r2 one MUST (F4 batch panel inert) → FIX2_AUTHORISED in the
  P15 Review log 08:20. 08:09 Codex at capacity → stale-handoff BLOCK; archived 08:11.
  P15 APPROVED (with notes) 08:21 on Claude. P16 implement r1 (Codex) hit the usage limit 08:41
  (resets 12:09), archived. P16 APPROVED (with notes) 09:05 on Claude. **All 17 phases P00–P16 APPROVED.**
- 2026-10-26 14:00–14:30 first real run with the owner → findings R1–R6 (`real_run_2026-10-26.md`); owner wants
  the lag fixed now. Planner wrote P17_real_run_fixes (backend) + P18_pending_state_ui (frontend); 05A MC-2 capture
  root corrected (R34, changelog 21). P17 APPROVED 15:18, P18 APPROVED 15:48 (Codex). Second real run 2026-10-29 PASS end to end (capture → previews → classify/reorder → form → publish to the test
  store, read back). Owner findings R7 (back button top left), R8 (drop animation) → small fix phase P19 (planned 2026-10-29, 9 rows).
  P19 APPROVED 2026-10-29 08:51 (Codex, no fix round). Owner: back button good, drop lands right but shrinks then pops → P20_drop_continuity (orchestrator plan).
  P20 APPROVED 09:50 (Codex). No run in flight. Next: owner eyeballs the glide; then the work-Mac move with the real store.
- Open after the MVP build: one tethered session to measure shot→original/adjusted preview (P16 Not yet);
  first end-to-end real use with the owner (shoot, sort, form, publish to the test store); backlog.md notes.
- Implementer and fix sessions: Codex again from P13 (owner 2026-10-26); P07–P12 were built on Claude
  (Opus/Sonnet) while Codex was rate-limited; `PSM_IMPLEMENTER=claude` switches.

## Needs the owner
Nothing. Files scope granted 2026-10-26 (re-check: scope `write_products,write_metaobject_definitions,
write_metaobjects,write_files`; `files` + `products` queries readable; token never printed).
Earlier cards (I1, plan cards 1–2, P12 card 1, SKU suggestion) answered; see Facts and changelog 18.

## Next command
When the owner says the credentials file is filled: check the token request (never print it), then
`tools/pipeline/run_phase.sh all` in the background (resumes at P12; projection is marked done). On exit: per orchestrator.md table.

## Facts the next orchestrator needs
- 2026-10-25 ~15:55: owner took the camera away (not tethered). Capture One stays open on
  `session test 1` on this Mac; the owner will not use Capture One here. Capture-dependent probes →
  UNPROBED; rerun 2026-10-26 08:10–08:20 with the camera: VERIFY-11 (fallback script broken → P16 task 0) and VERIFY-8 capture latency recorded; VERIFY-6 at N≥50 still unprobed.
- Codex trust for this repo is set in `~/.codex/config.toml`; `claude -p --agent pipeline-reviewer --model opus` verified working.
- 2026-10-26 owner answers: (B) the P12 probe store is the owner's **test store**; the real store comes later by
  changing shop + credentials in settings only, so nothing store-specific is hard-coded. (A) Shopify access is a Dev
  Dashboard app: client ID + secret, client credentials grant, token in memory only (09A MC-23); the owner keeps them
  in `~/.psm_shopify.env` (outside the repo). (C) Condition photos go to one metafield named by the setting
  `integration.condition_photos_key`, initial `custom.damage_report`, set by hand (09A MC-24). (D) No answer on the
  next-free-SKU suggestion → kept as written in 05B MC-22: the proposal advances to the next free number
  (P13 row C9(g)). The orchestrator first misdescribed this to the owner; corrected 2026-10-26.
