---
plan: live-fixes
role: plan
round: 1
date: 2026-11-26
---
# Plan the fixes found at the live preview hardware check (book 10J, LV-D19)

Repository root: `/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager-live` (git worktree, branch
`feature/live-preview`; never touch the `main` checkout, `docs/implementation/STATUS.md` or other tracks' files).

## Context
LV1–LV3 are APPROVED (see their plans and narratives under `docs/implementation/`). The owner-attended hardware check
(`plans/LV4_hardware_check.md`) is **paused part-way**; its running record is in `docs/implementation/STATUS_live.md`
(section "LV4 hardware check in progress"). The owner asked to fix what was found before resuming. The app log of the
check is `~/Library/Application Support/com.loorenz.sessionmanager/logs/psm.2026-11-26.log` (read-only; quote lines as
evidence). Book: `docs/intention/book/10J_live_preview.md`, new decision **LV-D19** (changelog D19 in
`16_changelog.md`). Process: `docs/process/flow.md` (§10 milestones of units apply, as for LV1–LV3) and
`docs/implementation/master_plan_live.md` (contracts §5, names §4, environment §8). No inventory session; projection off
unless you see a reason.

## Write
- Two milestone plans, in the same format as `plans/LV1…LV3` (frontmatter `phase`, `title`, `difficulty`,
  `projection`, `depends_on`, optional `implementer`; units ≤ 8 criteria; criteria tables with row id, boundary, input,
  exact outcome, trace; core mechanisms; delegated choices; manual try-out; guide files; empty Review log):
  - **`plans/LV5_live_fixes.md`** — Rust/C++/desktop fixes, **default implementer** (never `implementer: fable`;
    camera code). `depends_on: [LV3_phone_app]`.
  - **`plans/LV6_phone_dock.md`** — phone UI changes, **`implementer: fable`**. `depends_on: [LV5_live_fixes]` (if it
    needs LV5's wire changes) or `[LV3_phone_app]`.
- Amend `master_plan_live.md` where a contract changes (state the amendment with date and LV-D19 / finding id; never
  silently rewrite a contract), its tracker/sequencing, and `phases_live.txt` (insert LV5, LV6 before LV4; LV4 stays
  last and owner-attended).
- Handoff `docs/implementation/handoffs/reviewer/handoff_live_fix_plan_r1.md` (frontmatter `plan: live-fixes`,
  `role: plan`, `round: 1`, `state`, `date`, `actor`): files written, counts, the two milestones in plain words (what
  each fixes, what the owner can try after it), `⚠ OWNER DECISIONS REQUIRED (n)` with cards verbatim.
Commit nothing. Change no code. Do not edit the book.

## What to fix (each is a finding with evidence; verify the cause in the code before writing the rows)
**LV5 (default implementer):**
- **F4 — MC-LV3, the advance ignores exposures whose file is still on the way.** After an exposure is bound, the
  current shot moves to a container whose own exposure is bound but whose file has not arrived; it is corrected only
  when that file arrives. Log: `14:07:02.224 current_shot.bound … exposure=61 container=8` then `current_shot.set …
  container=10 by=advance` while exposure 60's file for 10 arrived at `14:07:02.8`; `14:07:07.375 … container=none` then
  `set container=8 by=advance` while exposure 61's file for 8 was on the way. A shutter in that window binds a second
  exposure to a taken container. MC-LV3: "the first empty container in shooting order, after any reported shutters
  whose files are still on the way"; "after an exposure is bound, the current shot moves to the next empty container
  after it". A container with a bound, pending exposure is **not empty** for the advance and the default. Rows must
  include fast Wi-Fi sequences with files arriving late and out of order (fake sidecar), and the arrival of the pending
  file must not move the current shot backwards.
- **F3 — Mac review actions do not reach the phones.** `photos_delete` → `ReviewService::delete` emits no event;
  LV3·U0 added `delete_announced` only for the phone's own delete. Every Mac action that changes what the phone shows
  (deny/delete, classify Main/Condition, move/reorder, restore, undo/redo, drop into a container, template hand changes)
  must lead to the phone's `session` message (the same events the phone sockets already forward). Emit from core /
  application services, never in Tauri handlers (H8). One row per action family at the socket boundary.
- **F2 — a Wi-Fi stall closes the phone's socket.** PH5 closes the socket when one write exceeds `write_timeout` (2 s),
  including a **frame** write; the Android was dropped 5 times in 2 min (`phone.viewer closed … reason=write_timeout`,
  13:51–13:53; the iPhone once at 14:15:43), each reconnect showing the template for a moment. Amend PH5 so a slow frame
  write makes the viewer **skip frames** (latest-only, as MC-LV1/M-LV1 intend) and the socket closes only when the phone
  is really gone (ping/pong or text writes failing for a longer, named time). Rows at the socket boundary with a slow
  reader.
- **F5 — Focus Area not movable is not reported.** With the camera's Focus Area set to Wide, taps gave
  `focus.result … point_ms=-1 … outcome=Focused` (1.3–1.4 s) and the phone showed AF LOCKED; FC4/FC6 and LV-D14 say the
  app reports a non-movable focus area and changes nothing. Read the sidecar (`native/camera-sidecar/src/live_runner.cpp`,
  the focus worker) and the lab (`~/Desktop/Developer/Camera-Connection-Tests/sdk/src/camera_liveview.cpp`,
  `experiments/10-focus-from-live.md`) to decide the detection (the Focus Area property value, and/or the point never
  confirmed by a focus frame) and the outcome the phone already maps to its "Set the camera's Focus Area to a movable
  spot" text. Never write the Focus Area.
- **F1 — exposure readout never shown.** The real helper emits
  `{"event":"liveview.exposure","shutter_speed":{…,"display":"1/80",…},"f_number":{…,"display":"F8.0",…},"iso":{…,"display":"ISO 2000",…}}`
  (full line in the log at `13:29:31.609243Z`, "unparsable sidecar line"); the Rust protocol expects another shape. Make
  the two agree (protocol test with that exact line), so the phone's readout shows (LV3 already draws it).
- **F7 (server half) — tile previews stuck as broken images.** The phone asked `/photos/{id}/preview` before the preview
  existed (404 ×31 in the log) and never retried; the files exist now. Make sure a `session`/photo message reaches the
  phone when a photo's preview becomes available (or changes), so the phone can reload it (LV6 does the phone half).
- **Settings → Phone list stale:** it showed the Android "Never connected" while it was connected; the list should
  reflect connection and last seen while the page is open (desktop UI, small).

**LV6 (`implementer: fable`), owner decision LV-D19 and the phone halves:**
- **Dock without hold gestures (LV-D19 (1)):** remove hold-to-magnify and hold-and-drag-up; larger tiles; horizontal
  scroll; single tap on a filled tile opens its Preview; single tap on an empty template tile makes it current; Free
  shot and Condition tiles as before. Keep the design's look (README) otherwise; update the tests that encoded the old
  gestures (LV3·U3 rows) and say which.
- **F7 (phone half):** a tile's image reloads when its preview becomes available or changes (versioned URL from the
  session view, and/or retry on error); never a permanent broken image.
- **Removed screen (LV-D19 (2)):** the text tells the person to delete the app from the home screen and install it again
  from the new QR code.
- Anything else the phone needs for LV5's wire changes (exposure readout, focus-area message).

## Owner's standing constraints
Hard rules as in `master_plan_live.md` and `docs/guide/03_rules_and_principles.md` (camera: never write exposure,
network or FTP settings; Focus Area never written; H7, H8, H3; the suite never needs the camera or a phone — fake
sidecar). Few units, one review per milestone, full gate once at the end of each milestone (`npm run build` first).
Guide updates in the same change (flow §7A).

Final message, first line exactly:
`HANDOFF: docs/implementation/handoffs/reviewer/handoff_live_fix_plan_r1.md | STATE: <DONE|OWNER_DECISIONS_PENDING|BLOCKED> | OWNER_CARDS: <n>`
