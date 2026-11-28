---
plan: LV3_phone_app
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-26
actor: opus
---
# Handoff — LV3 the phone app (Shoot Companion), review r1

**Verdict: APPROVED_WITH_NOTES.** All 85 rows of the five units are made true by the production code, and each has a
test at its boundary that would fail on a wrong outcome. The plan's Review-mode checks are green and H1–H8 hold. There
is no B1–B4 finding. Six notes go to `docs/implementation/backlog_live/LV3_phone_app.md`. Tracker → `APPROVED`;
narrative written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker line read `IMPLEMENTED (2026-11-26, fable)`.
- Checkpoints U0 `0ee711f8`, U1 `fefafb8f`, U2 `ce02e69f`, U3 `34fa2a4f`, U4 `a43b75d5`, close `9ba8180a`: all in
  `git log`. Tree at review: `1c00d149`, `git status --porcelain` clean.

## Perimeter (step 1)
`git diff --stat 0ee711f8^ 9ba8180a` = 46 paths, matching the handoff's 45 code and guide files plus the plan file.
Outside the plan's per-unit lists, and both disclosed and needed: `crates/integrations/src/lib.rs` (the event hub, the
plan's "code path that rebuilds the App") and `crates/testkit/src/phone.rs` (a `TestPhone` that follows the App across a
rebuild, for C28). No extra or missing path.

## Checks (step 2: the plan's Review mode, once each, one at a time)
| Command | Result |
|---|---|
| `cd apps/desktop && npm run build` | built |
| `cargo test -p psm-testkit --test suite -- lv2_phone_ops:: lv2_phone_ws:: lv3_server_carry::` | 29 passed, 0 failed |
| `cd apps/desktop && npx vitest run src/phone` | 7 files, 94 passed |
| `npm run typecheck` | clean |
The implementer's closing-gate stamp (Rust 2723 passed, clippy, fmt, sidecar stub, UI 1047) stands: it was taken on
`a43b75d5` with a clean tree, and the close commit changes only the plan file.

## Verified correct
- **U0 (C26–C29).** `ReviewService::delete_announced` (core) emits `PhotoChanged` + `SessionChanged` per affected session
  after the same Deny; the route only calls it (H8). C26(b): one undo step restores the photo's place and the undo
  label. `session_delete` builds `SessionsService::with_events`, and announces `ActiveSessionChanged {None}` itself only
  when Capture One could not be left (`still_active` check, so no double emission). `EventHub::observe_carried` is
  handed to the new hub in **both** rebuild paths (`apply_settings` Runtime, `reopen`) and deduplicated by pointer.
  `ensure_fed` now registers one `OnceLock` bridge as carried. `error_body` returns fixed texts for every 500 and 404;
  the C29(b) test checks that neither the missing folder nor the temp root is in the body.
- **Test boundary.** The UI tests fake the Mac at `fetch` and `WebSocket` (`helpers.ts`), so `api.ts` and `socket.ts`
  run for real. The asserted request bodies, the `Authorization` header and the demand messages are exactly what the
  phone sends. Assertions are exact (`toEqual` on bodies and demand lists, exact texts, exact opacities and styles).
  No test accepts several outcomes or asserts its own fixture.
- **U1.** Token kept, removed from the address, manifest link set, `Bearer` on every request, 401 → token removed and
  the §6 text. `decodeFrame` follows PH5's header. `reconnectDelay` gives 500/1000/2000/4000/4000 ms, no attempt while
  hidden, one at once on visible, and a new-`gen` demand on open. Frames are accepted only for the current `gen` with a
  `seq` above the last. `liveShown` keeps the template at 1 until a frame of the current `gen` has been drawn. Full
  screen is shown iff `fullscreenEnabled && !fullscreenElement && !standalone`. The camera status names match
  `ws::status_name` (`connected`/`connecting`/`busy`/`needs_action`/`disconnected`).
- **U2.** `tapToFrame` re-derived by hand: C12(b) 98.65/587.3 = 0.16797; C12(c) (39 + 98.65)/587.3 = 0.23438; C12(d)
  local (225, 150) → (0.5, 0.5). The touch mode is decided once per touch; lock at 110 px with `[25,40,70]`. The double
  tap clears the pending 260 ms focus and unlocks. × calls `POST /focus/clear` (card LV9). Capture: flash and haptic at
  once, toast only after `exposed` and a re-read listing the photo. 409 `session_changed` → text + re-read. The phone
  never advances the shot itself (C14(f)).
- **U3.** Dock order: shots in shooting order with `final_no` tags, extras with Main position tags (inverted), Free
  shot, divider, Condition C1…. `tileScale` matches design §3. Tap/scroll/hold/drag-up thresholds match. Tile centres
  are snapshotted at pointer down. An empty template opened by drag-up selects it.
- **U4.** Move targets: the extras' Main position (`shots.length + index + 1`) matches the server's
  container-then-extras model in `template_placement::move_to`; the `container` target carries `container_id`, which the
  route converts to `final_no`. The preview follows its photo by id after the re-read (C23(b) second test). The Item
  drawer hides "Article number" when `article_number` is null. The properties page is empty with a working back.
- **Review-mode reads.** No `serviceWorker`, `wakeLock`, `crypto`, `clipboard`, `getUserMedia` or `console` under
  `src/phone/`. No absolute URL or scheme; the socket scheme follows `location.protocol`. The token appears only in
  the `Authorization` header and `?pair=` (PH3), never in a toast or a log. Every threshold and timing equals the
  design README: hold 200 ms, tap 260 ms, double tap 300 ms / 50 px, slide 14 px, lock 110 px, overlay ×130, magnify
  95 px, drag-up −50 px, toast 1.4 s, flash .45 s after 70 ms. The book's differences are honoured: Condition after the
  last shot, swap, × recentres, Left/Right, shooting order. The dock's 260 px scroller is absolutely positioned and has
  `pointer-events: none`, so it takes no room or touches from the stage.
- **Guide** (flow §7A): `01_product.md`, `02_architecture.md` and `05_working.md` were updated for the phone app.

## Assumptions (step 4)
None contradicts 10J or the plan:
- Free shot always live fits LV-D5 as revised (no sleep; frames flow whenever the camera session is up) and 10J's
  "untemplated: live and Capture only".
- Recentring on every marker removal (×, choosing another shot, Retake) follows card LV9's reasoning ("otherwise the
  screen says no focus point while the camera still focuses at the old spot").
- Leaving out "All template shots captured" is allowed: §6 lists the toast texts, it does not require each one.
- Untemplated "Extra image NN added": §6's "does not occur" refers to a templated session (card LV7).
All twelve are in the narrative's "Choices made for you".

## Hard constraints (step 5)
H1: no external write. H2: phone Delete = Deny, and the bytes stay. H3: frames are drawn as received, and the template
is grey by a CSS filter only. H4: the phone holds no state of its own beyond token and overlay, and it re-reads the
Mac. H5: none. H6: none. H7: `delete_announced` is device-free. H8: events come from core or the app-service module
(`session_delete`), never from a route handler. Secrets: the token is never logged or shown, and error bodies carry
no paths. No live service in any test.

## Findings
None blocking (B1–B4: none).

## Notes written to backlog
`docs/implementation/backlog_live/LV3_phone_app.md` (the track's backlog folder, `master_plan_live.md` §3; the prompt's
`docs/implementation/backlog.md` is the main track's):
1. Row C24(a)'s text "1 main" contradicts its own fixture (two Main photos). The test asserts the truthful counts
   (plan lesson).
2. The capture toast depends on `exposed` arriving before the re-read that first lists the photo. It is missing with
   a provider that does not report `exposed`, or when the file lands first, and a foreign photo landing meanwhile
   takes the toast (→ LV4).
3. After a rebuild, the old App's hub keeps the carried observer. This is harmless (extra re-reads).
4. jsdom here has no `PointerEvent`, but `master_plan_live.md` §3 says it does (plan lesson).
5. The overlay tick compares against the render's value, so a batched move can double-tick.
6. The side-by-side look comparison with the prototype was not done by the reviewer (unattended session, no
   browser). Thresholds were checked by reading (→ LV4 / owner try-out).

## Lessons for the plans
- A row's exact outcome must be computed from the fixture the row names (C24(a)).
- `master_plan_live.md` §3: drop "jsdom has the Pointer API". Say that tests send mouse events of pointer types with
  a `pointerId`.
- The capture-toast rule ("toast once the session re-read shows where it landed") leaves the matching of a capture to
  a photo to the client. If LV4 shows it misleads, the server could return the photo id bound to a capture id.

## Write perimeter of this session
`docs/implementation/plans/LV3_phone_app.md` (Tracker line, Review log entry) ·
`docs/implementation/narratives/LV3_phone_app.md` (new) · `docs/implementation/backlog_live/LV3_phone_app.md` (new) ·
this handoff. No production or test file was edited, not even temporarily. No mutation probes. `apps/desktop/dist/`
was rebuilt by `npm run build` (git-ignored). Nothing committed (the runner commits).
