# STATUS — live preview (book 10J)

Running log of the live preview track on branch `feature/live-preview`, worktree
`~/Desktop/Developer/Photo-Session-Manager-live`. The main tracker `STATUS.md` and other tracks' files are not edited
by this track. Wake prompt: `docs/process/prompts/wake_orchestrator_live.md`.

## Where things stand
- **2026-11-26:** 10J RATIFIED (D18). Orchestrator started on Opus 5.5.
  - Planning prompt written: `docs/implementation/prompts/reviewer/live_plan_r1.md` (owner constraints: ~3 build
    milestones + 1 owner-attended hardware check, units with CHECKPOINT commits, one review per milestone; no
    inventory session, contracts section in the master plan; `implementer: fable` on phone-UI milestones only;
    projection off; phone app only, plain HTTP; sidecar live view/focus/capture-at-point/clean quit ported from the
    lab; Rust hub; no desktop delivery now; fake sidecar; guide updates in the same change).
  - First planner launch failed at once (prompt starting with `---` passed as an argument was read as a CLI option);
    relaunched with the prompt on stdin, as the runner does.
  - **Plan written** (planner handoff `handoffs/reviewer/handoff_live_plan_r1.md`, OWNER_DECISIONS_PENDING, 1 card):
    LV1 camera live view to the hub (default implementer), LV2 current shot and phone server (default; the riskiest:
    placement rewrite under MC-LV3), LV3 phone app (`implementer: fable`), LV4 owner-attended hardware check (never
    dispatched by the runner). 15 units, 91 criteria, 261 rows; projection off. Planner installed `node_modules` in
    this worktree (`npm ci`; git-ignored).
  - Planner finding for later: `cargo test ... --test suite a:: b::` with two filters before `--` is refused; flow §4A
    wording could be corrected (not done; main's flow file).
  - **Card 1 answered (owner 2026-11-26):** "the article number is the product barcode in shopify" → branch (a):
    `article_number` = the item's barcode (already read from the Shopify variant barcode); LV2·C9(b), LV3·C24, master
    plan PH6/§9 amended.
  - Plan relayed to the owner. **Owner go (2026-11-26):** "we can go! … I expect you complete the three steps
    autonomously"; resolve blocked sessions from the intention and the plan; if Codex runs out entirely, continue on
    Opus (runner change, flow log v1.7).

  - **LV1 run 1** (10:15–11:08): implemented (Codex Sol high, 3 unit checkpoints), review r1 CHANGES_REQUESTED (3 findings:
    frame listener across helper restarts, frame-channel failure stopping focus, a test expecting a derived value),
    fix r2 done and green but BLOCKED on its commit (Codex sandbox could not write the worktree's git lock under the main
    repo's `.git`). Orchestrator committed the fix (`f7d03aa1`), set the handoff DONE, added the git common dir to the
    runner's Codex sandbox, resumed for the re-review.
  - **LV1 re-review r2:** CHANGES_REQUESTED, one MUST (R2-B1: helper restart while live view is off leaves a dead frame
    connection; next camera session gets no frames; reproduced 3/3). FIX2_AUTHORISED in the plan's Review log; resumed.
  - **LV1 fix 2** (Codex Luna): R2-B1 fixed (`7bba1e49`); BLOCKED only on a P50 test failing inside the Codex sandbox
    (re-signing the bundled helper); passes outside it (orchestrator run). Handoff set DONE; runner gained a resume guard
    for fix r3; resumed for re-review r3.
  - **LV1 APPROVED** (`0f56b860`, re-review r3, APPROVED_WITH_NOTES; narrative relayed). Notes in
    `backlog_live/LV1_camera_live_view.md`. Branch pushed. **LV2 started.**
  - **LV2 APPROVED** (`APPROVED_WITH_NOTES`, review r1, no fix; narrative relayed). Codex hit its usage limit at
    11:58 (resets 15:15); the runner fell back to Claude Opus by itself (implement 11:58–13:05). Reviewer notes routed to
    LV3 folded into the LV3 plan as Unit 0 (phone Delete events, session-delete announce, rebuild observe, generic error
    bodies; rows C26–C29); master plan §8 gate runs `npm run build` first. Pushed. **LV3 started** (Fable).
  - **LV3 APPROVED** (review r1, APPROVED_WITH_NOTES, no fix; Fable implement 13:14–13:56, 5 units incl. U0; narrative
    relayed). Six notes in `backlog_live/LV3_phone_app.md` (capture toast matching and the look comparison with the
    prototype go to LV4). Plan lesson folded (jsdom has no PointerEvent, master plan §3). **All three build milestones
    approved**; pushed; pull request #33 to `main` opened; waiting for the owner.
  - **LV4 preparation (2026-11-26 14:14):** SDK sidecar built against the lab's SDK 2.02 (`tools/sidecar/build.sh --sdk`,
    both ctest programs pass, 17 live-view tests); release bundle built
    (`apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app`, 50 MB, ad-hoc signed): `phone-web/phone.html`
    inside; the bundled helper links `@rpath/libCr_Core.dylib` (supplied at run time by the installed camera support
    package). **Installed** with the owner's OK (build `2523595b`; previous kept as "2026-11-25 labels L4" in
    `~/Desktop/Session Manager old (2026-11-12)/`). Owner preparing the template; phones ready.
  - **LV4 hardware check in progress (owner-attended, 2026-11-26 from 14:35), running record:**
    - C7 iPhone: first QR opened (`.local` page), Add to Home Screen → full screen, still paired. PASS.
    - C8 Android: shortcut only (Chrome), Full screen button shown once a session showed; one tap hid the bars. PASS.
    - Phones follow the Mac's active session (session opened → both showed shot 1 at once). Settings → Phone list showed
      Android "Never connected" while it was connected (stale list; note).
    - C4 both on template: `sent_10s=0` for both, many lines. PASS.
    - C2/C5 Wi-Fi: source 17–21 fps; iPhone locked 153–217 per 10 s, dip during a 42 MB RAW upload (2.4 s) to ~15–18
      fps, no stall text; both phones locked together ~20 fps each, skipped 0–9 per 10 s. PASS (edge-of-Wi-Fi part not run).
    - Glass→Android over Wi-Fi (iPhone photo of Mac clock + Android): 264600−264436 ≈ 165 ms, 262258−262067 ≈ 190 ms,
      260383−260235 ≈ 150 ms → **≈ 150–190 ms** (lab ≈ 145 ms USB, Wi-Fi camera→Mac ≈ 122 ms). Camera's own screen lags
      the Mac ≈ 20–30 ms.
    - C9/C10 partial (Wi-Fi): 5 captures (camera button, iPhone ×2, Android ×2) → 5 photos, bound 7→10→8→9 (shooting
      order) then none (Condition); request→exposed 0.50–0.63 s; AF 305 ms. PASS so far.
    - Finding F1: the real helper's `liveview.exposure` line (shutter/f_number/iso as objects with `display`) is
      "unparsable" in Rust → the phone's exposure readout never shows.
    - C3 no stale frame on a peek (Android): PASS (owner). C6 reconnect after a 1-min lock (iPhone): PASS; Android also,
      with the template shown for a moment before live (designed: template until the first fresh frame).
    - Finding F2: the Android's socket was closed `write_timeout` (one frame write > 2 s, PH5) 5 times in 2 min while on
      live over Wi-Fi (13:51–13:53), reconnecting 1.4–8 s later (one 43 s gap); later 8 min without a drop. A Wi-Fi stall
      closes the whole socket instead of skipping frames.
    - Finding F3: Deny/delete on the **Mac** (`photos_delete` → `ReviewService::delete`) emits no event, so phones update
      only on their next request (owner: "the phone interface didn't react that fast"). LV3·U0 announced only the
      phone's own delete (`delete_announced`); every Mac review action needs the same.
    - C11 tap a shot then camera button (container 9): PASS. C12 Retake → red on the Mac at once, Cmd+Z restores, retake +
      phone capture fills it and advances: PASS. C13 two after the last → Condition: PASS. Phone request→exposed
      0.55–0.73 s.
    - **Finding F4 (MC-LV3, important):** the advance after a bound exposure treats a container whose exposure is bound
      but whose file has not arrived yet as empty. Log 14:07:02.224 exposure 61 → 8, `set container=10 by=advance` while
      exposure 60's file for 10 was still on the way (arrived 14:07:02.8, then `set none`); 14:07:07.375 exposure 63 →
      none, then `set container=8` while exposure 61's file for 8 was on the way (corrected to none at 07.582 on
      arrival). A shutter in those windows (≈ 0.2–0.6 s with ~4 s Wi-Fi files) would bind a second exposure to a taken
      container. Nothing went wrong in this run. MC-LV3: "first empty container … after any reported shutters whose
      files are still on the way".
    - C15 tap-to-focus over Wi-Fi from the phone: **20/20 focused**, tap → focused 0.39–0.97 s (median ≈ 0.63 s; point set
      0.14–0.31 s, AF 0.32–0.62 s); camera screen followed. PASS. C16 sharp where tapped (owner at 100 %): PASS.
    - C17 ×: retested — `/focus/clear` → `focus x=320 y=240 af=false`, point set in ≈ 0.21 s; chip gone, camera square back
      at the centre (owner). PASS (the first round simply had no × press).
    - **C18 FAIL → finding F5:** Focus Area set to Wide on the camera, taps on the phone: no message; results
      `point_ms=-1` (point not confirmed by a focus frame) and `outcome=Focused` 1.3–1.4 s, so the phone shows AF LOCKED
      while the camera chose its own spot. The app changed nothing (owner set it back; still works): the safety half holds,
      the "set a movable focus area" report does not fire on the a7R V's Wide.
    - C14 revoke while on live: `phone.viewer closed … reason=revoked`, Android showed the removed text; iPhone unaffected.
      PASS. (Also a `write_timeout` on the iPhone at 14:15:43, reconnected in 0.7 s: F2 on the iPhone too.)
    - **Finding F6 (owner card):** after Remove + re-pair, the **installed** app (iPhone home-screen app, Android
      shortcut) stays dead: it starts from the manifest `start_url` `/phone/?pair=<old token>` (`wire.rs:132`) → 401; the
      new QR opens a browser tab with the new token, so the owner had to install the app again. On iPhone the installed
      app's storage is separate from Safari, so a new QR can never reach it. **Owner: accept reinstalling** (LV-D19 (2)).
    - Finding F7: dock tiles show broken images ("?") — `/photos/{id}/preview` answered 404 (31×) before the preview
      existed and the phone never retried; files exist now.
    - **Paused by the owner at 15:25 local** before the USB rows (C1, C9), C19 (M-LV5), C20 (clean stop, M-LV6 targets),
      C5's edge-of-Wi-Fi part. Owner: "for now we can correct these issues". Owner decision LV-D19 (book, changelog D19):
      dock without hold gestures (bigger tiles, scroll, single tap); reinstall after Remove.
  - **Fix planning launched:** planner (Opus) with `prompts/reviewer/live_fix_plan_r1.md` → LV5_live_fixes (default
    implementer: F1–F5, F7 server half, Settings list) and LV6_phone_dock (Fable: LV-D19 dock, F7 phone half, removed text).
  - **Fix plan written** (handoff `handoffs/reviewer/handoff_live_fix_plan_r1.md`, 0 owner cards): LV5_live_fixes
    (default implementer; F1–F5, F7 server half, Settings list; F4 root cause: ~1 s between a file reaching the Mac and
    its photo row) and LV6_phone_dock (Fable; LV-D19 dock, F7 phone half, removed text). Planner choices: tiles 56 px;
    a still touch is a tap; only Flexible Spot (incl. tracking) counts as movable; sockets survive stalls ≤ 15 s;
    Settings list refresh 5 s; missing exposure value "–". LV4 now depends on LV6. **LV5 started.**
  - **LV5 implement r1 BLOCKED** (16:31–17:01, Codex Sol): all 4 units checkpointed; the gate failed on P40's
    `c7a_attribute_events_only_history` (forbids `SessionChanged` after attribute undo; PH8 requires it). Answered from
    the master plan: PH8 supersedes P40 C7(a) (no rule, only "emits nothing today"); perimeter amended in the LV5 Review
    log; relaunched with RESUME=1 (implement re-runs to finish).
  - **LV5 APPROVED** (17:28, re-review r2, APPROVED_WITH_NOTES; one fix: stricter C3 assertions, committed by the
    orchestrator — Codex fix sandbox again refused the git lock). **LV6 started** (Fable).
  - Owner during LV6: lock swipe halved (LV-D19 (3), row LV6·C3(b)). **LV6 APPROVED** (17:52, review r1, no fix; lock 55 px
    made it in). SDK sidecar rebuilt; release bundle built from `3c403e05` and **installed** (previous kept as
    "2026-11-26 live LV1-LV3"). Pushed.
  - **LV4 resumed (2026-11-26 ~18:05 local) on the fixed build.** Re-check: LV-D19 dock (bigger tiles, scroll, single
    tap) PASS; lock swipe 55 px PASS; tile picture appears by itself PASS; Mac Deny/undo and reorder reach both phones
    PASS (F3 fixed); Focus Area Wide message PASS (F5 fixed). Exposure readout: Mac parses it now (no "unparsable"),
    but the owner saw no readout on live.
  - **Owner change requests (to fix after the check, with the other findings):**
    - **R1 exposure readout off the live view:** "i don't really need it in the live view mode this can be placed on a
      drawer … the bottom drawer that pops out" → show it in the phone's drawer, not over live (and make it actually
      show: it did not appear).
    - **R2 a quiet loading cue on a tile whose picture is arriving:** "while the picture is arriving i see the gray
      picture i would like to see some small loading animation that doesn't disturb (like gradient loading … slightly,
      so it doesn't look like i have to wait to take the next picture)".
  - **C1/C9 USB (2026-11-27 ~14:24 local):** link `Cable`; source 23–25 fps (`frames_10s` 231–251); iPhone locked
    `sent_10s` 216–250, skipped 0–33 around captures; request → exposed 0.44–0.47 s; each capture one photo via
    Save-to-PC ~1.4 s after exposure, bound to the current shot (11, 9, 12 …); the same file also arrived by Auto FTP
    over Wi-Fi ~5 s later and was filed `outcome=duplicate` (no second photo). Owner: "seems fast"; retake onto a taken
    shot works. PASS. Glass→iPhone over USB (4 photos, two during captures): 150, 107, 143, 132 ms → **≈ 110–150 ms**.
  - **R1 withdrawn** (owner: the readout shows on live, "well positioned … we can leave it").
  - C19 (M-LV5): owner "same as always" (file format, exposure mode M, Focus Area); helper log over both days shows only
    `sdk.store_destination.set` (×4) and the focus point written. PASS.
  - C20 clean stop: quit during live → phone live froze, phone actions refused while the Mac was gone; reopened →
    first USB connect `connect failed` once (13:31:02), connected 7 s later on its own, live back (13:31:09). PASS
    (camera needs a few seconds after a clean disconnect, as Experiment 10). Unplug during live: Wi-Fi live back in
    10 s (13:32:28→38); re-plug → USB only after ~40 s, needing a click on Capture One's window (known Capture One
    hand-back limitation, owner aware). PASS with that limitation.
  - C5 edge of Wi-Fi: lag at the far end, Wi-Fi off froze live, back on while walking → live resumed by itself (no
    template fallback). PASS.
  - **LV4 DONE** (20/20 PASS; evidence `live_preview_lab.md` §Hardware check (LV4), `sony_camera_lab.md` §Live view in
    the app). **M-LV6 targets (owner): USB ≥ 23 fps, ≤ 0.20 s; Wi-Fi ≥ 15 fps (upload dips allowed), ≤ 0.25 s.**
  - R2 planned by the orchestrator as `LV7_tile_loading_cue` (Fable, 1 unit, 4 rows); started.
  - **LV7 APPROVED** (review r1). Release bundle built (not installed yet: the app was open).
  - **Owner request R3 → book LV-D20 (changelog D20):** swipe to classify in the Preview (left Deny, right Main with a
    confirm for template-shot photos, up Condition), the Preview gets the strip, press-hold-lift drag and drop (swap
    between template shots, insert-and-shift among extra Main and among Condition); built from the description (no
    design prototype). Planner launched (`prompts/reviewer/live_sort_plan_r1.md`).
  - **LV8 planned** (handoff `handoff_live_sort_plan_r1.md`, 0 cards): one Fable milestone, 4 units, 70 rows (server
    classify/place route; Preview swipes + strip; lift-and-drag; reliability). Planner choices: cross-group drop = end
    of the group (book 06), onto a template shot = swap; template photo to Main empties the shot and makes it current;
    lift after 0.4 s still; swipe ≥ 96 px; optimistic drop with rollback, 8 s timeout. Owner: shimmer installs with LV8.
    **LV8 started.**
  - **LV8 APPROVED** (15:51, review r1, no fix; Fable 15:08–15:46, 4 unit checkpoints). SDK sidecar rebuilt (the gate
    had put the stub back), release bundle with LV7 + LV8 **installed** (previous kept as "2026-11-26 live fixes
    LV5-LV6"). Pull request #33 carries everything; waiting for the owner's try-out.
  - **Owner try-out of LV8 → LV-D21 (changelog D21):** corrections (loading cue missing in the strips, flicker on
    moves, lifted template photo should leave its template, Preview without header/buttons: ⋯ drawer + close) and new
    tools (pinch zoom, reframe on the phone as 10D MC-27 with guides and grid, guides on live). Planner launched
    (`prompts/reviewer/live_tools_plan_r1.md`) → LV9_preview_corrections, LV10_zoom_reframe_guides (both Fable).
  - **LV9/LV10 planned** (after the usage-limit reset; handoff `handoff_live_tools_plan_r1.md`, 0 cards). Causes found:
    no "on its way" signal between shutter and photo row (2.5 s USB / 5–6 s Wi-Fi) → LV9 sends it from the shutter;
    four flicker causes; phone shows uncropped pictures → LV10 shows framed previews. Planner choices: Move image leaves
    the Preview (drag/swipe cover it); in reframe the photo follows the fingers (as the Mac). **Chain LV9 → LV10 started.**

## Next
The owner reviews the pull request and the three narratives. Then LV4, the owner-attended hardware check (never
dispatched by the runner): packaged build with the SDK sidecar, both phones, the a7R V; the plan is
`plans/LV4_hardware_check.md`. Open notes for LV4: capture toast on real captures, look vs. the prototype, revoke while
connected, LV1 carry-overs.

## Run in flight
- None. LV9 and LV10 APPROVED 2026-11-27; build installed 2026-11-28 (previous kept as "Session Manager (2026-11-27 live LV7-LV8).app" in the Desktop old folder); PR #33 description updated. Owner trying LV9–LV10 on the phone.

## Needs the owner
- Review / merge the pull request to `main`; schedule the hardware check (LV4).
- Possible gap to raise: the phone does not draw the camera's own focus frame (10J LV-D14 "the live view shows where
  the camera is focusing"); only the tap marker and "AF LOCKED".

