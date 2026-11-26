---
plan: live
role: plan
round: 1
state: OWNER_DECISIONS_PENDING
date: 2026-11-26
actor: claude-opus-5.5 (pipeline-planner)
---
# Handoff — live preview track planned (book 10J)

Gate passed: `docs/intention/book/10J_live_preview.md` reads RATIFIED (owner 2026-11-26, D18). No mechanism-inventory
session (owner); `master_plan_live.md` §5 stands in for it (contracts FC, HB, CS, PH). Four milestones: three build
milestones of units plus the owner-attended hardware check. Nothing committed, no code changed, the book not edited.
One owner card, which blocks nothing but one row of the phone's item drawer.

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1 — Where does the phone's "Article number" come from?**
- **Question:** which value should the phone's item drawer show as "Article number"?
- **Story:** your phone design lists name, SKU and article number for the piece in front of you. Session Manager's
  items have a SKU and a barcode, and nothing called an article number. If the phone guesses, you might read the
  barcode while checking a chair's maker label and think the two disagree.
- **Branches:** (a) the item's barcode; (b) a shop product field you name (for example `custom.article_number`);
  (c) leave the row out until the property forms come.
- **Recommendation:** (b), naming the field: an article number is a fact about the product, not a stock code.
- **On silence:** the gate holds for that row only: the phone hides "Article number"; everything else is built.
- **Trace:** 10J LV-D17; master plan §5 PH6; LV2·C9(b), LV3·C24(a)(b).

## Files written (write perimeter)
Documents (all new, untracked, not committed):
- `docs/implementation/master_plan_live.md`
- `docs/implementation/phases_live.txt`
- `docs/implementation/plans/LV1_camera_live_view.md`
- `docs/implementation/plans/LV2_current_shot_and_phone_server.md`
- `docs/implementation/plans/LV3_phone_app.md`
- `docs/implementation/plans/LV4_hardware_check.md`
- `docs/implementation/handoffs/reviewer/handoff_live_plan_r1.md` (this file)

Environment side effects of the probes (all git-ignored, no tracked file changed; `git status` shows only the files
above): `apps/desktop/node_modules/` (`npm ci`), `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin`
and `target/camera-sidecar/stub/` (sidecar stub build), `target/` (the Rust test build), logs under
`/tmp/psm-live-probe/`. No archgraph in this repo (none to update). No code, no book, no other track's files, no
`STATUS.md`.

## Counts (derived by script from the plan files)
| Milestone | Units | Criteria | Rows | Rows per unit | App files | Projection | Implementer | difficulty |
|---|---|---|---|---|---|---|---|---|
| `LV1_camera_live_view` | 3 | 17 | 61 | 17 · 22 · 22 | yes (sidecar C++, core, integrations, testkit) | no | default | high |
| `LV2_current_shot_and_phone_server` | 5 | 29 | 101 | 31 · 14 · 14 · 27 · 15 | yes (core, infra, integrations, desktop, frontend) | no | default | high |
| `LV3_phone_app` | 4 | 25 | 79 | 18 · 29 · 13 · 19 | yes (frontend `src/phone/`, guide) | no | **fable** | high |
| `LV4_hardware_check` | 3 | 20 | 20 | 8 · 6 · 6 | no (evidence docs only) | no | — (owner + orchestrator) | normal |
| **Total** | **15** | **91** | **261** | every unit ≤ 8 criteria | | | | |

Projection off for all, reason (master plan §3): every test fixture is this repo's own shape (sidecar protocol, frame
record, phone wire); the Sony SDK calls are ported from lab code proven on the a7R V and checked only by LV4; the
dependency-owned shapes at test boundaries (axum's WebSocket close frame as `tokio-tungstenite` receives it; browser
Pointer/Fullscreen/Vibration APIs stubbed in jsdom) are standard and stated where used.

## Ledger coverage (rows tracing to each entry; every entry served)
| Entry | LV1 | LV2 | LV3 | LV4 | Total |
|---|---|---|---|---|---|
| M-LV1 (newest frames, slow viewer harmless) | 7 | 1 | 1 | 3 | 12 |
| M-LV2 (nothing unless Live; fresh frame on switch) | 3 | 2 | 2 | 2 | 9 |
| M-LV3 (one photo per capture, placed as a camera-button shot) | 2 | 1 | 1 | 2 | 6 |
| M-LV4 (current shot binding, retake, Condition) | — | 7 | 1 | 4 | 12 |
| M-LV5 (no setting written but the focus point) | 1 | — | — | 1 | 2 (+ FC6 reading check in LV1's Review mode) |
| M-LV6 (latency, frame rate; targets by the owner) | — | — | — | 3 | 3 (operational; log lines `live.source`/`live.viewer` give the numbers) |
| MC-LV1 | 9 | — | 2 | — | 11 |
| MC-LV2 | 4 | 4 | 2 | 2 | 12 |
| MC-LV3 | — | 8 | 3 | 1 | 12 |
Master-plan contracts: FC 32 rows, HB 49, CS 35, PH 87. The "at hardware acceptance" items of 10J and the lab (Wi-Fi
focus, sharp where tapped, several phones, reconnect after sleep, iPhone reachability) are LV4·C15, C16, C5, C6, C7.

## Milestone map (plain words)
**LV1 — Live frames and focus reach the app.** The camera helper (C++) gains live view ported from the lab: it pulls
the camera's picture at the camera's pace, never forwards the same frame twice, and sends frames on their own channel
(a private socket), never on the command line. Focus from a point runs on its own thread with focused/not and the
camera's focus frame; a capture waits for a running focus. Every way the helper stops ends the camera session cleanly.
In Rust, a live view hub keeps only the newest frame and hands it to viewers that show Live. **What the owner can try:**
nothing visible yet; the tests show it, and with the camera connected the log shows ~25 frames a second arriving.

**LV2 — The current shot and the phone server.** Placement changes to the book's current shot: each shutter (phone, Mac,
camera button) is bound to the shot current at that moment, so late Wi-Fi files land right; Retake denies the old photo
at once and reopens the shot; the Mac's "Next" outline is the current shot. Settings → Phone pairs a phone with a QR
code (one revocable token per phone). The app's own server answers paired phones (session, capture, focus, shot
changes, moves, deletes) and streams live frames only to phones showing Live. **What the owner can try:** shoot with the
camera button and watch "Next" move; shoot fast over Wi-Fi and see each photo in the right container; pair a phone and
see a simple "Connected to <item>" page; remove the phone again.

**LV3 — The phone app (Shoot Companion, by Fable).** The owner's design, high fidelity: hold for live, slide up to lock,
tap to focus, × to recentre, slide sideways for the template overlay, capture with flash and haptics, the shot dock in
shooting order, preview, Retake, Delete, Move (swap), the item drawer and the empty "Fill item properties" page;
Full screen on Android, home-screen install on iPhone, plain HTTP, no secure-context API. **What the owner can try:**
the whole working loop from the phone, as in 10J's acceptance story.

**LV4 — Hardware check (owner-attended).** Twenty checks on the packaged app with the a7R V, the iPhone and the Android
phone, using the lab's procedures: frame rate and delay on USB and Wi-Fi, no stale frame on a switch, several phones,
reconnect after sleep, iPhone install and reachability, captures from every button with late files, retake, Condition
after the last shot, tap-to-focus and sharp photos, × recentres, no camera setting changed, clean stop. **What the owner
does:** runs it with the orchestrator and sets the frame-rate and delay targets (M-LV6) from the measurements.

## Estimated size
- **LV1:** large. C++ port ~700–900 lines (live view, runner, focus worker, pure classes and their tests); Rust
  ~900–1,200 lines (hub, protocol, frame reader, adapter mapping, fake sidecar) plus ~700 lines of tests. Comparable to
  P45 + P46 together. One implement session of several hours.
- **LV2:** the largest. The placement rewrite under MC-LV3 touches the most delicate existing code (template placement,
  ledger binding, rewritten t2/t3/t4 tests); plus the server, WebSocket, pairing and Settings page. ~2,500–3,000 lines
  including tests. Expect the most review findings here (U1).
- **LV3:** large UI. ~2,000–2,500 lines of TypeScript and CSS plus ~1,200 lines of component tests; many thresholds,
  each a row.
- **LV4:** one owner session of 2–3 hours with the camera and both phones.

## Probed on this Mac (2026-11-26)
- `tools/sidecar/build.sh --stub --test`: 2.9 s, ctest 100 % of 1 test program passed.
- `cd apps/desktop && npm ci`: needed — **this worktree had no `node_modules`**; 2.3 s.
- `npm run typecheck`: 3.3 s clean. `npx vitest run src/features/sessions`: 13 files, 67 tests, 5.8 s.
- `cargo test -p psm-testkit --test suite -- t2_template_placement:: p45_sidecar_and_protocol::`: 36 passed in 1 min
  0.6 s from a cold build of this worktree.
- **Finding for the coordinator:** `cargo test -p psm-testkit --test suite a:: b::` (two filters before `--`) is refused
  by cargo ("unexpected argument"); several filters must go after `--`. The live plans use the working form. Earlier
  tracks' plans (e.g. `W3_writer_review.md` Unit test commands) list several filters before `--`; flow §4A's wording may
  deserve a one-line correction (coordinator's call; not edited here).
- crates.io reachable (the new crates `qrcode`, axum's `ws` feature with `tokio-tungstenite`, and the npm package
  `@fontsource/ibm-plex-mono` need one fetch). `qrcode` 0.15 has an `svg` feature; axum 0.8.9's `ws` feature exists.
- The app icon `apps/desktop/src-tauri/icons/icon.png` is 512 × 512 (used for the phone manifest).
- Not run: the full Rust suite (~9 min, flow §4A), the SDK build, the packaged build (LV4 preparation).

## Process deltas recorded (master plan §3)
- No mechanism-inventory session for this track (owner); §5 FC/HB/CS/PH stand in.
- flow §10 milestones of units apply; reason for > 8 criteria per milestone recorded; each unit ≤ 8.
- `implementer: fable` on `LV3_phone_app` only (owner, supersedes flow log v1.3 for this track); LV1/LV2 never.
- Projection off, reason recorded. Test runs: unit commands while working, the full gate once per build milestone.
- Schema by idempotent `ensure` steps, not a numbered migration (this branch runs beside `main`; the research track's
  precedent); a build of this branch can open the owner's real data.
- LV4 is never dispatched by the runner (task 0 answers BLOCKED, as P52).

## Planning decisions the coordinator should know (recorded in the plans as narrative points)
1. **After a Condition shot the current shot returns to the first empty template shot** (CS4; 10J's "else the first
   empty one" applies; the prototype stayed on Free shot; the prompt says the book wins). LV2 P1.
2. **A photo with no shutter report takes the current shot at arrival** (CS6): Capture One tether, or a file beyond
   10E's 300 s FIFO window. 10J says "every exposure"; files without one are not addressed; this is the only
   consistent reading. LV2 P2.
3. **Shift the rest back stays on the Mac** with binding semantics (CS8): 10J's amendment names only §6's refill and
   retake as replaced. LV2 P3.
4. **Late files** use 10G rule 1's existing counter correction through slot owners (CS2), so a file the ledger's FIFO
   matched to an earlier exposure still lands in the shot its camera counter belongs to; the first file of a burst
   arriving late is in the wrong container only until its predecessor arrives (rule 5 moves template-placed photos).
5. **Not planned (out of this track):** naming "camera in use by another computer" (10J §Status says 10E should; the
   lab saw only a generic `0x8202`, so it needs evidence first); writing `LiveView_Image_Quality` (none required).
6. **Server lifetime:** one `PhoneServer` per process resolving the current `AppHandles` per request; hub and
   `CurrentShots` ride on the camera runtime (carried through settings-save rebuilds, 10E R-I18).

## Notes for the orchestrator
- Owner card 1 needs no gate: if answered, the coordinator adds the source to LV2·C9(b) (and LV3·C24(b) already covers
  a non-null value).
- LV2 creates `docs/implementation/backlog_live/` on the first reviewer note; the folder does not exist yet.
- Every plan carries a **Tracker** line `NOT_STARTED (2026-11-26, planner)`.
