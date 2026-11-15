---
phase: T2_template_placement
title: Photos file themselves — in a session whose item type has a shooting template each arriving photo goes to the container of its place in the shooting order, by when it was shot and not when it arrived; a container's split, shape and zoom are applied; photos after the last container arrive as Condition; the session shows the containers faded and numbered, with an info button that opens the template
difficulty: high
projection: no
depends_on: [T1_templates_and_settings]
blocked_on: none
---
# T2 — Photos file themselves

**Tracker:** NOT_STARTED · 2026-11-16 · planner · plan written from 10G (ratified 2026-11-16)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `10G` §4 and §5, by the placement contract of `master_plan_templates.md` §5 (P1–P8), which this plan implements verbatim. (U1) The pure rules: shot order, expected place, zoom, preset values. (U2) Placement on arrival, presets, the session's template view, and the minimum that keeps a hand change from being undone. (U3) The session view: faded numbered containers and the info button. **Not here:** the warning, the marks, drop onto a container, Back to template (T3); retake, Shift the rest back (T4). A session whose type has no template behaves exactly as before (P66).

## How to work this plan (flow.md §10, master_plan_templates.md §3)
One runner phase made of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `T2·C<n>(<row>)`; tests named after their row (`t2_c4a_<slug>`). Open details: the smallest option consistent with 10G and `master_plan_templates.md` §5, recorded under **Assumptions**.

| Unit | What it builds | Criteria | Rows | Changes |
|---|---|---|---|---|
| U1 | Shot order and the other pure rules | 2 | 13 | `crates/core` only |
| U2 | Placement on arrival, presets, the template view, hand changes kept | 5 | 20 | Rust, generated bindings |
| U3 | The session view | 1 | 5 | TypeScript |
| **Total** | | **8** | **38** | |

## Read first (whole phase)
`docs/process/flow.md` (§4, §4A, §6, §10) · `docs/implementation/master_plan_templates.md` (all; **§5 is the contract**) · `docs/implementation/master_plan.md` §3, §7 · book `10G` §2, §4, §5, §7, §8 (A2, A3, A4, A9) · book `05A` MC-2, MC-4, MC-5 · `10E` (the capture ledger: exposures, deliveries, the FIFO rule) · `10D` MC-27 · `docs/guide/02_architecture.md` §Arrival, §Compose and render · `docs/guide/04_change_impact.md` rows "The schema", "Tauri command/DTO", "Add a behaviour-test file" · the T1 plan (what exists) · then each unit's Read first.

## The fixture the rows use ("Chair-3")
A template for item type "Chair" with containers K1, K2, K3 (final numbers 1, 2, 3); shooting order K2, K1, K3 (shot 1 → K2, shot 2 → K1, shot 3 → K3); K3 has presets split + zoom 20, K1 and K2 none. S is a session of an item of type "Chair" and is the active session. F1…F5 are camera files `DSC00001.ARW`…`DSC00005.ARW`; "arrives" means `TestApp::arrival_photo` (a delivery through the ledger, then the watcher). W is `FIFO_WINDOW_MS`.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Order of shots:** by the time of the shutter; among camera files shot within five minutes of each other the camera's own file number decides, because a file that arrives out of turn can be given its neighbour's time. Capture One's tether and imported files are ordered by the file's time.
- **A shot the camera reported whose file has not arrived keeps its container empty** for up to five minutes; after that the following photos close the gap at the next arrival.
- **The very first shot of a session arriving after the second:** for a few seconds the second sits in the first's container; when the first arrives both are right.
- **A template changed while a session is being shot** applies from the next arriving photo; photos you placed by hand stay.
- **Photos that were in the session before its template existed stay where they are** (they count as shots).
- **A type whose template has no container behaves as a type without a template.**
- **Extra Main photos come after the containers** in the Main order.
- **In this phase a hand change is kept and nothing more:** no warning yet, no way back (T3).

## Unit 1 — Shot order and the other pure rules

**Unit id:** `T2·U1` · **Rows:** T2·C1–C2

### Goal
The rules P4, P5 and P8's arithmetic as pure functions with unit tests. Nothing calls them yet.

### Read first
`master_plan_templates.md` §5 P2–P5, P8 · `crates/core/src/mechanisms/capture_ledger.rs` (`seq_of`, `stem_key`, `file_key`) · `crates/core/src/domain/camera.rs` (`FIFO_WINDOW_MS`) · `crates/core/src/mechanisms/composition.rs` (`default_rect`, `turned_frame`, `validate`) · `crates/core/src/domain/composition.rs`.

### Files
New: `crates/core/src/mechanisms/shot_order.rs`. Edited: `crates/core/src/mechanisms/{mod.rs, template.rs}`, `crates/core/src/domain/template.rs`.

### Core mechanisms (the only functions with unit tests)
`mechanisms::shot_order::order` · `mechanisms::template::{expected, zoomed, preset_values}`.

### Ordered tasks
1. `shot_order::order(shots, holds)`: input shots `{photo_id, at, counter: Option<(prefix, number)>}` and hold times; output the entries in shot order, each a shot or a hold. The four steps of P4, exactly. It never panics on any input (no comparator that is not a total order: sort only by the tuples P4 names).
2. `template::expected(index, containers)` (P5) · `template::zoomed(rect, zoom)` (P8) · `template::preset_values(presets, current values, frame)` (P8: kind, shape, orientation Named, turns/flip/straighten kept, rect).
3. Unit tests for the rows.

### Criteria (2 criteria, 13 rows)
Boundary: the named pure function. Times in milliseconds; a shot is written `name(at, file)`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T2·C1(a) | `shot_order::order` | five shots without a counter: ids 1, 2, 3 at 10, 30, 20; ids 7 and 5 both at 40 | 1, 3, 2, 5, 7 | 10G §4.1 (MC-4) |
| T2·C1(b) | same | A(10, DSC00001), B(20, DSC00003), C(30, DSC00002) | A, C, B | 10G §4.1 (the counter decides among received files) |
| T2·C1(c) | same | A(10, DSC00001), B(20, DSC00003), one hold at 30 | A, hold, B | 10G §4.1 (a reported shot holds its place), A3 |
| T2·C1(d) | same | A(10, DSC00001), B(20, DSC00002), one hold at 15 | A, B, hold | 10G §4.1 |
| T2·C1(e) | same | B(10, DSC00002), one hold at 20 | B, hold | 10G §4.1 (first shot late: recorded point) |
| T2·C1(f) | same | A(10, DSC00001), B(20, DSC00003), C(30, DSC00005), one hold at 40 | A, hold, B, C | 10G §4.1 |
| T2·C1(g) | same | A(20, DSC09999), B(10, DSC00001) | A, B | 10G §4.1 (counter restart) |
| T2·C1(h) | same | A(0, DSC00005), B(W + 1, DSC00004) | A, B | 10G §4.1 (time decides outside the window) |
| T2·C1(i) | same | A(10, DSC00002), T(15, no counter), B(20, DSC00001) | B, T, A | 10G §4.1 (tether and imports keep their time) |
| T2·C1(j) | same | A(10, DSC00002), B(20, IMG00001) | A, B | 10G §4.1 |
| T2·C2(a) | `template::expected` | the Chair-3 containers; index 1, 2, 3, 4 | K2 (slot 2), K1 (slot 1), K3 (slot 3), Condition | 10G §4.2, §4.4 |
| T2·C2(b) | `template::zoomed` | rect (0, 0, 1, 1) zoom 20; rect (0, 0.25, 1, 0.5) zoom 50; rect (0.1, 0.2, 0.3, 0.4) zoom 0 | (0.1, 0.1, 0.8, 0.8); (0.25, 0.375, 0.5, 0.25); unchanged | 10G §2 (centred zoom-in crop) |
| T2·C2(c) | `template::preset_values` | frame 6000×4000, current values normal / original / no turn; presets split + zoom 20; then presets shape 1:1, zoom 0 | first: kind split, rect (0.1, 0.2, 0.8, 0.6), and `validate` accepts it; second: kind normal, shape 1:1, rect (1/6, 0, 2/3, 1) | 10G §4.3 (10D MC-27) |

### Delegated choices
Type names inside the two modules · float comparison tolerance in the tests (1e-9).

### Notes
- The rule in words is in `master_plan_templates.md` §5 P4; the rows are its table. If a row and P4 disagree, stop and report.

### Unit test commands
`cargo test -p psm-core mechanisms::shot_order` · `cargo test -p psm-core mechanisms::template` · `cargo clippy -p psm-core --all-targets -- -D warnings` · `cargo fmt --all -- --check`

## Unit 2 — Placement on arrival

**Unit id:** `T2·U2` · **Rows:** T2·C3–C7

### Goal
P1–P8 in the app: an arriving photo is placed in one transaction; presets are applied; the template view can be read; a place the person chose is never changed by a later arrival.

### Read first
`master_plan_templates.md` §4, §5 · `crates/core/src/services/arrival.rs` (`accept`: where a photo is inserted and `photo_added` is emitted) · `crates/core/src/services/review.rs` (classify, reorder, delete, transfer) · `crates/core/src/services/composition.rs` · `crates/core/src/mechanisms/position.rs` · `crates/infra/src/sqlite/store.rs` (`arrival_insert`, `set_photo_review`, `set_positions`, `save_composition_in`, `adopt_frame_tx`, `delivery_for_path`, `ledger_exposures`) · `crates/core/src/services/camera_ledger.rs` (`exposure_observed`, `status_for`) · `crates/testkit/tests/suite/p44_capture_ledger.rs` (helpers `expose`, `deliver`) and `p66_approve_on_arrival_and_session_order.rs` · `crates/testkit/src/test_app.rs` (`arrival_photo`, `ledger`, `store`, `history`).

### Files
New: `crates/infra/src/sqlite/migrations/015_photo_places.sql` · `crates/core/src/services/template_placement.rs` · `crates/testkit/tests/suite/t2_template_placement.rs`. Edited: `crates/core/src/{services/mod.rs, services/arrival.rs, services/review.rs, services/composition.rs, ports/store.rs, domain/template.rs}` · `crates/infra/src/sqlite/store.rs` · `apps/desktop/src-tauri/src/{commands/templates.rs, lib.rs}` · `apps/desktop/src/services/{bindings.ts (generated), templates.ts}` · `crates/testkit/tests/suite/main.rs` · `crates/testkit/src/test_app.rs` only for a helper with a caller here.

### Core mechanisms
None new (U1's).

### Ordered tasks
1. Migration `015_photo_places.sql` (`master_plan_templates.md` §4, `IF NOT EXISTS`) and its `M::up` line.
2. Store reads and writes for: the template of a session (P1); the shots of a session with their keys (P2); the holds (P3; `now` is the caller's clock); the `photo_places` rows; writing a photo's place (classification, position through the existing `set_photo_review` / `set_positions`, and its row); the composition without a frame (kind, shape, origin, `pending_zoom`).
3. `template_placement::place_arrival(store, clock, events, photo_id)` (P6): for a templated session, one transaction: the new photo's row; the shot order; every movable photo to its expected place; presets (P8) for every photo whose place the template changed; orders (P7); `touch_session`; `enqueue_mirror_tag` for each photo whose classification changed. After the commit: `photo_changed` for each photo that changed (not the new one) and `session_changed`. No history step. Not templated: nothing is written.
4. `ArrivalService::accept` calls it right after a photo was inserted and before `photo_added` is emitted. An error is logged (`tracing::warn!`) and the arrival goes on.
5. `adopt_frame_tx`: in the branch that writes the first rect, a photo with `pending_zoom` gets `zoomed(rect, pending_zoom)` and the column is cleared. No other branch changes.
6. Hand changes are kept: `ReviewService::{classify, reorder, delete}` on a photo of a templated session set its row's `place_by_hand = 1` (the row is created when missing) in the same transaction and then renumber Main by P7. A move (`transfer`, not a copy) deletes the moved photo's `photo_places` row in the same transaction. `CompositionService::{set, apply, reset}` on a photo of a templated session set `frame_by_hand = 1` (P8 then leaves its composition alone). Untemplated sessions: no row is written, nothing else changes.
7. `template_placement::session_view(session_id)` → `TemplateView { templated, containers, photos: [{photo_id, slot, by_hand, differs}], differs, can_shift_back }`: `slot` only for a Main photo that has one; `by_hand` = the row's `place_by_hand` or no row; in this phase `differs` is false for every photo, the count 0 and `can_shift_back` false (T3 and T4 fill them; the shape does not change). Deleted photos are not listed. Command `template_session_view`, wrapper, bindings.
8. Tests for the rows.

### Criteria (5 criteria, 20 rows)
Boundary: stored photos (`store().photos_in_session`), `photo_places` rows, compositions, the collecting event sink, `HistoryService::state`, `session_view`, after arrivals on `TestApp`. Fixture Chair-3 unless the row says otherwise.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T2·C3(a) | arrivals | F1, F2, F3 arrive in that order | F1 Main slot 2, F2 Main slot 1, F3 Main slot 3; Main positions F2 = 1, F1 = 2, F3 = 3; each has a row with `place_by_hand` 0 | 10G §4.2, A3 |
| T2·C3(b) | arrivals | then F4, F5 | F4 and F5 are Condition, positions 1 and 2; Main as in (a) | 10G §4.4, A4 |
| T2·C3(c) | arrivals | a session of type "Table" (no template for it; the Chair template exists); three files arrive | all Main, positions 1, 2, 3 in arrival order; no `photo_places` row for them; `session_view.templated` is false | 10G §4 (first sentence), A9 |
| T2·C3(d) | arrivals | a template for "Table" with no container; three files arrive in a "Table" session | as (c) | 10G §4, P1 |
| T2·C3(e) | arrivals | the item's type is typed "chair " (lower case, a space at the end); F1 arrives | F1 Main slot 2 | 10G §2, P1 |
| T2·C3(f) | `HistoryService::state(S)` | after (b) | no undo step and no redo step | 10G §6.5 (arrival placement is not an undo step) |
| T2·C4(a) | ledger + arrivals | three shutters are reported for S (`exposure_observed` at t, t + 2 s, t + 4 s); F1 arrives, then F3; then F2 | after F3: F3 Main slot 3, no Main photo has slot 1, Main positions F1 = 1, F3 = 2. After F2: F2 slot 1; Main positions F2 = 1, F1 = 2, F3 = 3 | 10G §4.1, §4.2, A3 |
| T2·C4(b) | arrivals, event sink | no shutter reported; F1, F3 arrive; then F2 | after F3: F3 slot 1. After F2: F2 slot 1, F3 slot 3, and the events emitted by F2's arrival include `photo_changed` for F3 and `session_changed` for S | 10G §4.5, §4.6 |
| T2·C4(c) | ledger + arrivals | as (a) up to F3 (F3 in slot 3, slot 1 empty); the clock moves W + 1 s; F4 arrives | F3 slot 1; F4 slot 3 | 10G §4.1 (P3: a hold ends) |
| T2·C4(d) | arrivals | a "Stool" session, no template; G1 arrives. Then a template for "Stool" with two containers (shooting order = final order) is made; G2 arrives | G2 Main slot 2; G1 is Main, has no row and no slot; Main positions G2 = 1, G1 = 2 | 10G §4.5 (never moved), P6 |
| T2·C4(e) | arrivals without the ledger | in S's folder two RAW files with file times m + 1 s (`b.ARW`) and m (`a.ARW`); the watcher accepts `b.ARW`, then `a.ARW` | after the first: b slot 2. After the second: a slot 2, b slot 1 | 10G §4.1 (tether: file time) |
| T2·C5(a) | compositions | after C3(a): `adopt_frame` for F3's RAW (6000×4000, camera), then for F1's | F3: kind split, origin user, rect (0.1, 0.2, 0.8, 0.6), state ok. F1: kind normal, origin default, the rect an untemplated photo of that frame gets | 10G §4.3, A3 |
| T2·C5(b) | compositions | as C4(b), with `adopt_frame` (6000×4000) for each file right after it arrives | after F2's arrival F3's composition is kind split, origin user, rect (0.1, 0.2, 0.8, 0.6) | 10G §4.3, §4.5 |
| T2·C5(c) | compositions | no shutter reported; F1, F2, F4 arrive (F4 in slot 3), `adopt_frame` (6000×4000) for F4; then F3 arrives | F3 slot 3; F4 is Condition and its composition is kind normal, origin default, the default rect | 10G §4.3, P8 |
| T2·C6(a) | `session_view(S)` | after C3(b) | `templated` true; three containers: (final 1, shot 2, no presets), (final 2, shot 1, no presets), (final 3, shot 3, split, zoom 20); photos F2 slot 1, F1 slot 2, F3 slot 3, F4 and F5 without a slot; `by_hand` false for all; `differs` 0; `can_shift_back` false | 10G §5 |
| T2·C6(b) | the wired app, command `template_session_view`; the generated bindings | a session with no template | the answer has `templated` false and no container; `bindings.ts` contains `template_session_view` | 10G §5, A9 |
| T2·C7(a) | `ReviewService::classify`, then an arrival | after C3(b): F4 classified Main; then F6 arrives | after the classify: F4 is Main position 4, no slot, its row has `place_by_hand` 1. After F6: F6 is Condition; F4 is still Main position 4 | 10G §6.1, A5 (nothing reverts) |
| T2·C7(b) | `ReviewService::classify`, then arrivals | no shutter reported; F1 arrives; F1 classified Condition; then F3, then F2 arrive | F1 stays Condition; after F2: F2 slot 1, F3 slot 3 | 10G §4.5 (a photo placed by hand is never moved) |
| T2·C7(c) | `ReviewService::move_photos` | after C3(a): F1 moved to S2, another session of type "Chair"; then G arrives in S2 with a later time | F1 has no `photo_places` row and is Main in S2; G is Main slot 1 (F1 counts as S2's first shot); F1 stays Main without a slot (Main positions G = 1, F1 = 2) | 10G §6.1, P6 |
| T2·C7(d) | `CompositionService::set`, then an arrival | no shutter reported; F1, F3 arrive (F3 in slot 1), `adopt_frame` (6000×4000) for F3; the person gives F3 a valid normal composition of their own (`CompositionService::set`, a rect smaller than the default); then F2 arrives | F3 is in slot 3 and its composition is still exactly what the person set (kind normal, that rect); its row has `frame_by_hand` 1 | 10G §6.1 (a compose edit is never undone) |

### Delegated choices
Store method names and how many there are · whether placement reads everything inside the transaction or reads then writes with a check · the file layout of `template_placement.rs` · the DTO field names of `TemplateView`.

### Notes
- Hazard: `arrival_photo` ignores its session argument; the file joins the session of the matched shutter, else the active session. Activate S in the tests.
- Hazard: an exposure that still waits for a second file (RAW + JPEG) is `expecting` although its RAW arrived; P3 excludes it. C4(a) would not catch counting it; read P3.
- Hazard: `photo_places.photo_id` references `photos`; foreign keys are on. Nothing in this phase deletes a photo row that has a place row (a copy has none); T3 extends `delete_photo_row`.
- Hazard: a placement that fails must not lose the arrival (task 4).
- Hazard: `arrival_photo` writes the file name as the file's bytes, so the same camera file name delivered twice is filed as a duplicate, not as a photo. Use fresh names for a second session (G…, `DSC00011.ARW`…).
- 10D decides what happens to a preset when Capture One's frame arrives (`adopt_frame_tx`, unchanged branches): same orientation → re-fitted; other orientation with origin `user` → `needs_review`. Not re-tested here.
- An undone Classify in a templated session can leave Main out of container order until the next arrival; T3 closes that.

### Unit test commands
`cargo test -p psm-testkit --test suite t2_template_placement::` · `cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::` · `cargo test -p psm-testkit --test suite p28_composition_foundation::` · `cargo test -p psm-testkit --test suite p58_fast_compose_on_camera_preview::` · `cargo test -p psm-testkit --test suite p44_capture_ledger::` · `cargo test -p psm-desktop` · `cargo clippy -p psm-core -p psm-infra -p psm-desktop --all-targets -- -D warnings` · `cargo fmt --all -- --check`

## Unit 3 — The session view

**Unit id:** `T2·U3` · **Rows:** T2·C8

### Goal
In a templated session the Main area shows the containers in final order: an empty one faded with its number, a filled one as the photo is shown today; an info button opens the template. An untemplated session looks exactly as today.

### Read first
book `10G` §5 · `apps/desktop/src/features/workspace/{PhotoPanel.tsx, PhotoGrid.tsx, PhotoTile.tsx, WorkspacePage.tsx, useWorkspaceState.ts, workspace.css}` · `apps/desktop/src/services/{events.ts (onWorkspaceEvents), templates.ts}` · `apps/desktop/src/features/settings/TemplateExample.tsx` · `docs/intention/evidence/design_handoff.md`.

### Files
New: `apps/desktop/src/features/workspace/{ContainerGrid.tsx, TemplateInfoDialog.tsx, useTemplateView.ts}` · `apps/desktop/src/features/workspace/__tests__/t2_template_containers.test.tsx`. Edited: `PhotoPanel.tsx`, `WorkspacePage.tsx`, `workspace.css`.

### Core mechanisms
None.

### Ordered tasks
1. `useTemplateView(sessionId)`: reads `template_session_view` when the session opens and again on every `photo_added`, `photo_changed` and `session_changed` of that session; a failed read leaves the view untemplated.
2. `PhotoPanel` takes the view. Templated: the Main section renders `ContainerGrid`: for each container in final order, the Main photos whose slot is that number as today's tiles (same handlers, menu, selection, drag), or, when there is none, one placeholder (`data-container="<n>"`, `data-empty`, faded, showing the number); after the containers, the Main photos without a slot and the unreviewed ones, as today. The Main heading and counts are today's. Not templated: today's `PhotoGrid`, unchanged.
3. An info button ("Template", `aria-label="Show template"`) in the Main heading, only when templated, opens `TemplateInfoDialog`: every container in final order with its number, its example through `TemplateExample` (black and white), "Shot <n>" and its presets in words ("Split", "Shape 4:5", "Zoom 20 %"; nothing when it has none).
4. Placeholders are not photos: they are not selectable, not counted, and never reach export or publish (they exist only in this component).

### Criteria (1 criterion, 5 rows, all `[rule]`)
Boundary: `PhotoPanel` (with the view passed in) and `useTemplateView` in UI tests; `services/templates` and the event service mocked.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T2·C8(a) | `PhotoPanel` | a view with three containers; one Main photo with slot 2 | the Main section shows, in order: a placeholder `data-container="1"` with `data-empty` and the text 1, the photo's tile, a placeholder `data-container="3"`; the heading counts 1 main | `[rule]` 10G §5, A2 |
| T2·C8(b) | same | three Main photos with slots 1, 2, 3 and two Main photos without a slot | five tiles: the three in container order, then the two; no placeholder | `[rule]` 10G §5 |
| T2·C8(c) | same | a view with `templated` false; three Main photos | the three tiles as today; no placeholder; no "Show template" button | `[rule]` 10G §5, A9 |
| T2·C8(d) | same | the view of (a), K3 with split and zoom 20 and an example; press "Show template" | a dialog lists three entries numbered 1, 2, 3; entry 3 shows an image with `data-bw`, "Shot 3" and "Split · Zoom 20 %" | `[rule]` 10G §5 (info button) |
| T2·C8(e) | `useTemplateView` | mounted for session 7; then a `photo_changed` event for session 7; then one for session 8 | `getTemplateView(7)` was called twice, and never for 8 | `[rule]` 10G §4.6 (live) |

### Delegated choices
The look of the placeholder (design tokens; faded, dashed or tinted) · the icon of the info button · the dialog's layout · the folded strip (leave it as today).

### Notes
- Dragging onto a placeholder does nothing in this phase (T3 makes it a drop target).
- Tile numbering of filled containers stays today's output numbering.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/workspace/__tests__/t2_template_containers.test.tsx src/features/workspace` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time. A failure is fixed and only the failing files re-run (`p33_split_grid` C4(d) is known to fail now and then under load: rerun it alone).

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core mechanisms::shot_order` · `cargo test -p psm-core mechanisms::template` · `cargo test -p psm-testkit --test suite t2_template_placement::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/workspace` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: `order` follows P4 step by step; `adopt_frame_tx` changed only in its first branch; an untemplated session writes no `photo_places` row on any path; a failed placement cannot fail an arrival. Notes go to `docs/implementation/backlog.md` under T2.

## Manual try-out (the narrative is built from this)
On scratch data, with the dining-chair template of T1 (say five containers, the third with Split and Zoom 20):
1. Make a new session for an item of type Dining chair and open it: the Main area shows five faded, numbered containers and a small Template button. Press it: the black-and-white examples with their numbers and presets.
2. Shoot the angles in your shooting order. Each photo lands in its own container, whatever order the files arrive in. The photo of the split angle is split and zoomed.
3. Keep shooting after the last angle: those photos arrive under Condition.
4. Move a Condition photo to Main: it stays there when the next photo arrives.
5. Open a session of a type without a template: everything is as before.

## Review log
