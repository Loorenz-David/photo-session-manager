---
phase: H3_bring_back
title: Bring back — the app can download an archived or shop-only product's published pictures from Shopify at full size into its session folder and make them the session's photos (all of them or none), JPEG photos work like RAW photos in every photo operation, and the next Publish uploads only the photos that were changed since
difficulty: high
projection: yes
depends_on: [H2_list_pictures_pages_search]
blocked_on: none
---
# H3 — Bring back

**Tracker:** NOT_STARTED

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `10H` §7 by contract **MC-H4** (`docs/implementation/master_plan_archive.md` §5), with its invariants M-H1, M-H2, M-H3, M-H5. (U1) JPEG photos in the photo pipeline. (U2) `RemoteItems::bring_back`: download everything, then import in one transaction. (U3) The proof that a publish afterwards sends only changes, and that retention waits for the next publish. **Not here:** the command, the prompt, the progress on screen, activation after bring back (H4); getting the RAW back (10H §10); any change to the publish service's code (MC-H4.7: the links are enough — if a U3 row needs a publish change, that is a finding to report, not to work around).

## How to work this plan (flow.md §10, master_plan_archive.md §3)
One runner phase of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `H3·C<n>(<row>)`; tests named after their row (`h3_c1a_<slug>`). Open details: the smallest option consistent with 10H §7 and MC-H4, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_archive/H3_bring_back.md`. **Never call Shopify or its CDN, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | JPEG photos in the photo pipeline | 2 | 10 | **yes** |
| U2 | Bring back: download all, then import in one transaction | 7 | 34 | **yes** |
| U3 | Publish after bring back sends only changes; retention | 5 | 18 | **yes** (tests; production only if a row fails) |
| **Total** | | **14** | **62** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6A, §10) · `docs/implementation/master_plan_archive.md` §3 (Rule 9, Rule 17), §4, **§5 MC-H4 (every item), MC-H5.4, MC-H3.1** · book `10H` §2 (decision 3: the JPEG), §7, §9 · book `10F` (Rules, Q1, Q4, M41) · book `10D_composition_and_framing.md` (MC-27 composition values and frames, MC-28 master and output, MC-29 links and replacement) · book `07A`/`07B` as cited by `crates/core/src/services/publish.rs` (MC-11 adoption, MC-12 reconciliation, MC-21 retouch) · `docs/guide/03_rules_and_principles.md` (H1–H5) · `docs/guide/04_change_impact.md` §Persistence and §Shooting templates · the H1 and H2 plans (what exists).

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Nothing to see yet:** the next milestone puts this behind the workspace.
- **Bring back is all or nothing:** if one picture cannot be downloaded after three tries, nothing changes and the downloaded files are removed.
- **The files are named after the product's folder** (`<folder>-main-01.jpg` …); a second bring back of the same product adds `-b2`.
- **Bringing back is not an edit:** a published product stays "Published", and it is not cleared again before its next publish.
- **The earlier photo entries of a cleared product are kept, hidden** (their RAWs are gone); the session's undo list starts empty after a bring back.
- **A brought-back photo starts exactly as published:** no crop, no turn, no house style applied again.

## Unit 1 — JPEG photos in the photo pipeline

**Unit id:** `H3·U1` · **Rows:** H3·C1–C2 · **Changes app files:** yes

### Goal
The `original_preview` job handles a JPEG source (MC-H4.6); everything else in the pipeline is shown to work on a photo whose file is a JPEG; the Capture One facts are probed and recorded.

### Read first
Master plan §5 **MC-H4.6** · `crates/integrations/src/lib.rs` (`OriginalPreviewJob` :1526-1565) · `crates/infra/src/rawpreview.rs` (`EmbeddedJpeg` :8-27, `compose_frame` :93-117, `orientation` :241) · `crates/core/src/ports/raw_previews.rs` · `crates/infra/src/imaging/{mod.rs (`dimensions` :20, `thumbnail` :26-45), metadata.rs (the `exif` reader in use)}` · `crates/infra/src/sqlite/store.rs` (`adopt_frame_tx` :326-404: the first-frame branch and the P58 camera → Capture One rule at :363-384) · `crates/core/src/services/photos.rs` (`import` :25-92: copies any file) · `crates/core/src/services/arrival.rs` (`import` :274-283, `accept` :284-381) · `crates/core/src/mechanisms/arrival.rs` (`is_candidate` :12-36) · `crates/core/src/services/composition.rs` (`set_unrecorded` :80-95) · `crates/core/src/services/previews.rs` (`request_initial` :257, `on_processing_done` :398-470) · `crates/integrations/src/editing/capture_one/adapter.rs` (`variant_background` :169-184, `resync_folder` :189-213) · `crates/testkit/src/fixtures/{synthetic_jpeg.rs, synthetic_raw.rs}` · `crates/testkit/tests/suite/p58_fast_compose_on_camera_preview.rs` (frame rows on the original preview) · `docs/intention/evidence/capture_one_16.8.6.md` (the VERIFY list and its format).

### Files
New: `crates/testkit/tests/suite/h3_jpeg_photos.rs`. Edited: `crates/infra/src/rawpreview.rs`, `crates/integrations/src/lib.rs`, `crates/testkit/src/fixtures/synthetic_jpeg.rs`, `crates/testkit/tests/suite/main.rs`, `docs/intention/evidence/capture_one_16.8.6.md`, `docs/guide/02_architecture.md`.

### Core mechanisms (the only functions with unit tests)
None (the rows observe the job's outcome).

### Ordered tasks
0. **Capture One probe (flow §6A; evidence, not a criterion).** With Capture One running and a session document open: copy one JPEG into a capture folder of that document, then through the adapter's own scripts (`sync_folder`, `find_image`, a `PSM Preview` process of it, a `PSM Master` process of it) record in `docs/intention/evidence/capture_one_16.8.6.md` a new item **VERIFY-JPEG**: whether the file is listed after `sync_folder`, whether `find_image` answers for a `.jpg` path, whether both recipes export it, the export's pixel size against the JPEG's, and whether a settings file (`.cos`) appears for it and when. The probe file is left where it is (H2: never delete in a capture folder). **When Capture One is not running or no document is open, write `VERIFY-JPEG: UNPROBED (<date>)` there and in the Review log and continue**: the suite never depends on it, and the orchestrator runs it before the live check. Never fire the shutter for this probe.
1. `JpegSource` in `rawpreview.rs`, an implementation of `RawPreviews`: reads the file, takes its pixel size and EXIF orientation (tag `0x0112`; absent or outside 1–8 → none); writes `dest` = the JPEG itself when its long edge is at most 1600 px, else a copy scaled to 1600 px on the long edge (temp + rename, like `EmbeddedJpeg`); answers `Extracted {orientation, compose_frame: Some(frame)}` with the frame = the pixel size, width and height swapped for orientations 5–8 (no minimum size). A file that does not decode answers `Ok(None)`.
2. `OriginalPreviewJob::run`: a target whose extension is `jpg` or `jpeg` (any case) uses `JpegSource`, every other target `EmbeddedJpeg` as today. The rest of the job is unchanged (`set_original_preview`, `adopt_frame` with `FrameSource::Camera`, `PhotoChanged`).
3. `SyntheticJpeg::with_orientation(w, h, colour, orientation: u8)` (an `rgb` JPEG with an Exif APP1 segment carrying tag `0x0112`).
4. Guide: `02_architecture.md` (§Arrival / previews: a photo's file may be a JPEG; which job path it takes; the watcher still accepts RAWs only).
5. Tests for the rows.

### Criteria (2 criteria, 10 rows)
Boundary: `TestApp::with_fake_document()`; a JPEG enters as a photo through `PhotosService::import` (the existing import, which copies any file) into an active session; outcome = the stored photo and composition after `tick`, the files written, the operations the fake Capture One received.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H3·C1(a) | the `original_preview` job (after `tick`) | an imported `SyntheticJpeg::rgb(3000, 2000, …)` | the photo's original preview file exists and measures 1600 × 1067; `orientation` is null; its composition's frame is 3000 × 2000 with source `camera` | MC-H4.6; 10H §7.3 |
| H3·C1(b) | same | `SyntheticJpeg::with_orientation(3000, 2000, …, 6)` | `orientation` 6; frame 2000 × 3000 | MC-H4.6 ("its orientation its EXIF orientation") |
| H3·C1(c) | same | `SyntheticJpeg::rgb(800, 600, …)` | the original preview's bytes equal the imported file's; frame 800 × 600 | MC-H4.6 (never enlarged; no minimum size) |
| H3·C1(d) | same | a file named `x.jpg` holding the bytes `not a jpeg` | the photo has no original preview and no frame; the job is done (not queued again after a further `tick`) | MC-H4.6 |
| H3·C1(e) | same | an imported `SyntheticRaw` named `a.ARW` (embedded 6240 × 4160 JPEG matching its frame) | as before this unit: the preview is the embedded JPEG and the frame 6240 × 4160 with source `camera` | MC-H4.6 ("every other extension keeps the embedded-JPEG path") |
| H3·C2(a) | `CompositionService::set` | the photo of H3·C1(a); values `kind normal, shape free, quarter_turns 1, rect (0.1, 0.1, 0.5, 0.5)` | saved (revision 1); no `frame_unknown` | 10H §7.3 ("every photo operation that works on a RAW works on it"); 10D MC-27 |
| H3·C2(b) | `ReviewService` | two imported JPEG photos: classify the second as condition, then back to main, then reorder it to position 1 | the session view lists it at main position 1 | 10H §7.3 |
| H3·C2(c) | preview render (`tick`, then `complete_all_renders`) | the photo of H3·C1(a) | the fake Capture One received a process operation for the JPEG's variant; the photo has an adjusted preview; the composition's frame source is `preview` (Capture One's measurement replaced the camera one) | 10H §7.4; MC-H4.6 (P58 rule) |
| H3·C2(d) | export of the session (`ExportService`, masters completed with `complete_all_renders`) | one imported JPEG photo, classified main | the export holds one output for it | 10H §7.3; 10D MC-28 |
| H3·C2(e) | the watcher path (`app.watcher_event`) | a file `late.jpg` written into the active session's capture folder | no photo row is added | MC-H4.6 (the candidate rule is unchanged) |

### Delegated choices
Whether `JpegSource` uses `kamadak-exif` or reads the tag by hand · how a JPEG over 1600 px is scaled (the existing `thumbnail` code path with quality 80 is enough).

### Notes
- Hazard: `DynamicImage::resize` (`imaging/mod.rs:30`) **enlarges** a smaller image to the bounds; for a JPEG at or under 1600 px copy the file, do not call it (H3·C1(c)).
- Hazard: the original preview is unrotated pixels and `orientation` tells the UI how to show it, as for an embedded camera preview; do not bake the rotation into the file and also store the orientation.
- Hazard: an imported JPEG gets the house style like any import (`follow.photo_arrived`, `arrival.rs:368-372`); that is today's import behaviour and is not changed here. Bring back (U2) does not go through `accept`, so its photos get no style.
- H3 (hard constraint): scaling a JPEG is not decoding a RAW; it stays in `crates/infra` (`rawpreview.rs` / `imaging`), the only crate that decodes pixels.

### Unit test commands
`cargo test -p psm-infra --lib rawpreview` · `cargo test -p psm-testkit --test suite h3_jpeg_photos::` · `cargo test -p psm-testkit --test suite p58_fast_compose_on_camera_preview::` · `cargo test -p psm-testkit --test suite p06_live_previews::` · `cargo clippy -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — Bring back: download all, then import in one transaction

**Unit id:** `H3·U2` · **Rows:** H3·C3–C9 · **Changes app files:** yes

### Goal
`bring_back_names`, `estimate_bytes`, `Store::bring_back`, `RemoteItems::bring_back`, `AppEvent::BringBack`, the three errors.

### Read first
Master plan §4 (Vocabulary, Ports, Errors, Events, Words), §5 **MC-H4.1–5** and its invariants · `crates/core/src/services/remote.rs` (H1, H2) · `crates/infra/src/sqlite/store.rs` (`arrival_insert` :2425-2500: how a photo row, its composition, `raw_state` and its jobs are inserted; `insert_default_composition` :227; `upsert_photo_link` :2974; `delete_photo_row` and `photo_places`; `mark_item_cleared` :3236) · `crates/infra/src/sqlite/migrations/{002_domain.sql (photos :35-52), 004_compositions.sql, 005_photo_link_slots.sql, 015_photo_places.sql}` · `crates/core/src/domain/{photo.rs, composition.rs (`Shape::Original` :27), publish.rs (`PhotoLink` :242-256)}` · `crates/core/src/services/arrival.rs` (`capture_root` :92-106) · `crates/core/src/services/history.rs` (`retire_session` :309-341) · `crates/core/src/ports/editing.rs` (`sync_folder` :58) · `crates/core/src/services/sessions.rs` (`view` :348-401: which photos a session shows) · `crates/core/src/services/storage.rs` (`cleared_item_photos` :917, `item_files` :931) · `crates/core/src/services/previews.rs` (`check_missing_files` :165-225: a missing file of a non-deleted photo is reported on Health) · `crates/core/src/services/template_placement.rs` (`place_arrival` :29, `put_by_hand` :694, `renumber_main` :593) with `docs/implementation/master_plan_templates.md` §5 · `crates/testkit/tests/suite/p62_retention_after_publish.rs` (helpers :50-300) · `crates/testkit/tests/suite/a17_delete_session.rs`.

### Files
New: `crates/core/src/services/bring_back.rs`, `crates/testkit/tests/suite/h3_bring_back.rs`. Edited: `crates/core/src/{domain/remote.rs, mechanisms/remote.rs, ports/store.rs, services/mod.rs, services/remote.rs, events.rs, error.rs}`, `crates/infra/src/sqlite/store.rs`, `crates/integrations/src/lib.rs` (the capture-root source and the history handle given to `RemoteItems`), `apps/desktop/src-tauri/src/{lib.rs (the `bring_back_progress` arm), commands/items.rs (the error codes)}`, `apps/desktop/src/services/{bindings.ts (generated), events.ts (`onBringBack`)}`, `crates/testkit/src/{fake_commerce.rs, test_app.rs}`, `crates/testkit/tests/suite/main.rs`, `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
`mechanisms::remote::{bring_back_names, estimate_bytes}`.

### Ordered tasks
1. `bring_back_names` (MC-H4.2) and `estimate_bytes` (MC-H3.5: `bytes` or `UNKNOWN_PICTURE_BYTES` each).
2. Errors `BringBackRunning`, `BringBackFailed {reason}` with their command codes (`NotRemote` / `not_remote` exists since H1, its projection L3); `BringBackState`, `BringBackProgress`, `BringBackOutcome`; `AppEvent::BringBack` with its `TauriEvents` arm and `onBringBack`.
3. `Store::bring_back(BringBackInput {session_id, item_id, files: Vec<BroughtBack {role, position, media_id, path, sha256}>, link_role_condition, at}) -> Vec<i64>`: **one transaction** doing MC-H4.4 (a)–(e) exactly. In a session whose item type has a shooting template, each new main photo is also marked placed by hand (the mark `put_by_hand` writes) so that a later arrival's placement never moves it; the positions written are the published order.
4. `RemoteItems::bring_back(session_id) -> Result<BringBackOutcome>` (`services/bring_back.rs`): MC-H4.1 preconditions in their order; names (rule 2, `taken` = `store.photos_for_raw(path)` is not empty); downloads (rule 3) with progress events; on any missing picture the clean-up and the failure; the import (task 3); then MC-H4.5. A per-session "running" mark (in memory) gives `BringBackRunning` and the progress H4's view reports; it is cleared on every exit path.
5. `RemoteItems::new` gains `editing`, `capture_root`, `history` (H1 took only what it read; master plan §4 Service), wired in `App::build`; `TestApp::with_fake_shop()` gives `RemoteItems` the test's capture root, history and fake editing. `FakeCommerce` gains (carried from H1's projection, L9) `fetch_original` (records the URL in `original_fetches`; serves `set_image(media_id, bytes)` real JPEGs when registered, else `fake_image_bytes`; consumes H2's `image_faults` and honours H2's `set_fetch_delay` as `fetch_image` does) and `set_image`.
6. Guide: `01_product.md` (the workflow row "Bring back"), `02_architecture.md` (§Remote items: the bring-back sequence), `03_rules_and_principles.md` (H2's exceptions list gains: the app deletes only the files a failed bring back itself downloaded; a table "10H invariants M-H1–M-H5" with where each bites), `04_change_impact.md` (rows: "a new table or column that names a photo → retire it in `Store::bring_back` (a) like `delete_photo_row`"; "the publish link fields → `Store::bring_back` (c) writes them for brought-back photos").
7. Tests for the rows.

### Criteria (7 criteria, 34 rows)
Boundary: `bring_back_names` / `estimate_bytes` (pure); `RemoteItems::bring_back` on `TestApp::with_fake_shop()` with a fake Capture One document (outcome = the files in the session folder, stored rows read through the store, the session view, what `FakeCommerce` and the fake Capture One recorded, emitted events). "Archived item (3 + 2)" = an item with 3 main and 2 condition photos, published, then cleared by the real clearing; its Shopify media are `m1, m2, m3` and `c1, c2`, each registered with `set_image` as a distinct `SyntheticJpeg`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H3·C3(a) | `bring_back_names` | `("BEYO-12", 2, 1, nothing taken)` | `["BEYO-12-main-01.jpg", "BEYO-12-main-02.jpg", "BEYO-12-condition-01.jpg"]` | MC-H4.2; 10H §7.1 |
| H3·C3(b) | same | `("BEYO-12", 2, 1, "BEYO-12-main-02.jpg" taken)` | `["BEYO-12-main-01-b2.jpg", "BEYO-12-main-02-b2.jpg", "BEYO-12-condition-01-b2.jpg"]` | MC-H4.2 (the whole set carries the suffix) |
| H3·C3(c) | same | as (b) with `BEYO-12-condition-01-b2.jpg` taken too | every name carries `-b3` | MC-H4.2 |
| H3·C3(d) | same | `("K", 12, 0, nothing taken)` | twelve names, the last `K-main-12.jpg` | MC-H4.2 |
| H3·C3(e) | `estimate_bytes` | pictures with `bytes` `Some(1_000_000)`, `None`; no pictures | `2_500_000`; `0` | MC-H3.5; 10H §6 ("About X MB") |
| H3·C4(a) | `bring_back` | the archived item (3 + 2), folder key `BEYO-12` | `Ok`; the folder `<capture root>/BEYO-12/` holds exactly the five files `BEYO-12-main-01…03.jpg`, `BEYO-12-condition-01…02.jpg` beside whatever it held before; each file's bytes are the JPEG registered for its media; no `.part` file | MC-H4.2, .3; 10H §7.1 |
| H3·C4(b) | `FakeCommerce` | (a) | `original_fetches` = the five recorded URLs (any order); `image_fetches` (sized downloads) unchanged by the call | MC-H4.3; 10H §7.1 (full size) |
| H3·C4(c) | the session view | (a) | five photos: main 1–3 then condition 1–2, their files in the order `m1, m2, m3`, `c1, c2`; each photo's `raw_hash` is its file's SHA-256 | MC-H4.4(b); 10H §7.2 |
| H3·C4(d) | the store | (a) | the five earlier photo rows still exist, each `deleted` with no position, no preview path and integrity `raw_missing`; none has a link for the item; none appears in the session view | MC-H4.4(a); 10H §7.2 ("hidden, not deleted") |
| H3·C4(e) | the store | (a) | `item_retention`: `cleared_at` null, `restored_at` = the clock's now; the state is `Local`; the session's `change_seq` equals its value before the call and its list row is `Published` | MC-H4.4(d), (e); 10H §7.2 |
| H3·C4(f) | the store | (a) | each new photo's composition: kind `normal`, shape `original`, quarter turns 0, flip none, straighten 0, rect `(0, 0, 1, 1)`, revision 0 | MC-H4.4(b); 10H §7.3 ("no crop and no turn") |
| H3·C4(g) | the store | (a) | each new photo has exactly one link for the item: role `gallery` (main) or `attribute:<condition key>` (condition), slot `single`, `upload_state` `ready`, `external_id` = its media id, revision 0, no superseded id | MC-H4.4(c); 10H §7.5 |
| H3·C4(h) | the fake Capture One | (a) | it received `sync_folder:<capture root>/BEYO-12`; it received no `apply_style` operation for any of the five files | MC-H4.5; 10H §7.4 |
| H3·C4(i) | emitted events | (a) | `BringBack {Running}` with `done` 1, 2, 3, 4, 5 of `total` 5 (in that order), then `BringBack {Done}`, and a `SessionChanged` for the session | MC-H4.3, .5; 10H §7.1 (progress) |
| H3·C4(j) | `HistoryService::state` | the archived item had one undo step before clearing; (a) | the session has nothing to undo and nothing to redo | MC-H4.5 |
| H3·C5(a) | `bring_back` | an adopted item with a record (2 gallery, 1 condition), never photographed here | `Ok`; three photos (main 1–2, condition 1) with files `<folder key>-main-01.jpg` …; three `ready` links to `m1, m2`, `c1`; `restored_at` set; state `Local` | MC-H4.4; 10H §2 (decision 2), §7 |
| H3·C5(b) | after (a), `tick` | — | each new photo has an original preview and a frame equal to its JPEG's pixel size | MC-H4.4(b) (jobs), MC-H4.6 |
| H3·C5(c) | `bring_back` | an adopted item never opened (`Unknown`) whose product has 2 gallery media | `Ok` with two photos; `FakeCommerce` recorded exactly one `media_readback` read | MC-H4.1; MC-H5.4 (a press reads at once) |
| H3·C6(a) | `bring_back` | the archived item (3 + 2); `m2`'s URL answers `Fail` three times | `Err(BringBackFailed)` with message `Could not bring back the photos: 1 photo could not be downloaded`; `m2`'s URL was fetched exactly 3 times; the session view and the photo rows are as before the call; `cleared_at` still set; the folder holds none of the five names and no `.part`; the last event is `BringBack {Failed}` | M-H2; MC-H4.3; 10H §7.1 (retried twice; nothing imported) |
| H3·C6(b) | same | `m2`'s URL answers `Fail` twice, then serves the picture | `Ok`; five photos | MC-H4.3 |
| H3·C6(c) | same | `m2`'s URL answers `Gone` once; `media_urls` returns a new URL that serves it; budget `{100, 1000}` | `Ok`; exactly one `media_urls` call; `m2`'s stored URL is the new one; five photos | MC-H4.3; MC-H5.4 |
| H3·C6(d) | same | the record's picture `m2` is gone (empty URL, as H2·C4(b) leaves it) | `Err(BringBackFailed)` naming `1 photo is no longer in the shop`; no download was made; nothing changed | MC-H4.1 |
| H3·C6(e) | same, across a restart | `FakeCommerce::fail_after_effect("fetch_original")` (the simulated kill during the downloads), then `kill_and_restart`, then `bring_back` again | after the restart: the state is still `Remote(Archived)` and the session view unchanged; the second call is `Ok` and the folder holds exactly one file per name (five), each with its media's bytes | M-H2; H4 (idempotent resume) |
| H3·C7(a) | `bring_back` | a session whose item is `Local` | `Err(NotRemote)`; no fetch | MC-H4.1 |
| H3·C7(b) | same | an app without an integration (`TestApp::with_fake_document()`) | `Err(NoIntegration)` | MC-H4.1; H6 |
| H3·C7(c) | same | two threads call for the same archived item; the fake's fetch delay holds the first one in its downloads | one call answers `Ok` with five photos, the other `Err(BringBackRunning)`; the session has five visible photos | MC-H4.1 |
| H3·C7(d) | same | the archived item whose latest publish item's step is set to `update` through the store (a publish in flight) | `Err(BringBackFailed)` naming `wait until the publish has finished`; no fetch | MC-H4.1 |
| H3·C7(e) | same | the capture-root source answers an error (no document) | `Err(BringBackFailed)` naming `the capture folder is not known yet`; no fetch | MC-H4.1 |
| H3·C8(a) | `FakeCommerce::mutation_ops()` | after H3·C4(a), C5(a), C6(a), C6(c) | empty | M-H1; 10H §7.6 |
| H3·C8(b) | the fake Capture One's operations | after H3·C4(a) | none of them removes, deletes or moves an image or a file (the list holds `sync_folder` and lookups only) | M-H2 |
| H3·C8(c) | "Delete session" (`psm_integrations::session_delete::delete`) | the session of H3·C5(a) after its bring back | its three JPEG files are in the fake Trash; `FakeCommerce` recorded no read and no mutation during the delete | M-H5; H2 (Trash, never a permanent delete) |
| H3·C8(d) | Health (`check_missing_files` on focus) | after H3·C4(a) | Health holds no `file_missing` (the retired rows are not "deleted outside Session Manager") | MC-H4.4(a) |
| H3·C9(a) | `bring_back` in a templated session | an archived item whose item type has a shooting template with 3 containers; (3 + 0) pictures | the three new main photos are at positions 1–3 in the order `m1, m2, m3` | MC-H4.4(b); 10G P7 (the template never reorders them) |
| H3·C9(b) | same, then an arrival | after (a), a new RAW arrives in the session | the three brought-back photos keep their order relative to each other | MC-H4.4 (placed by hand) |

### Delegated choices
How the "running" mark is kept · the exact `BringBack {Running}` cadence when several downloads finish together (the `done` values must still be 1…N in order) · whether a leftover file is removed before the download or replaced by the rename.

### Notes
- Hazard (M-H2, the heart of this unit): the import is **one** store method in **one** transaction, called only after every file exists. No photo row may be written while a download is still running, and no file may be left behind by a failed attempt.
- Hazard: `UNIQUE(session_id, raw_path)` on `photos` (`002_domain.sql:51`). After a product was brought back, republished and cleared again, its retired rows still carry the first set of names: that is what rule 2's suffix is for (H3·C3(b)). Do not rename or delete the retired rows.
- Hazard: the retired rows must be `deleted` **and** `raw_missing` with their preview paths nulled: `deleted` keeps them out of Health's missing count (`previews.rs:205`), the nulled paths keep M-H4 for every later reader, and `check_missing_files` then finds nothing to change (H3·C8(d)).
- Hazard: do not write `photo_denials` rows for the retired photos (P66's undo window and RAW deletion do not apply: there is no file).
- Hazard: do not touch `sessions.change_seq` (MC-H4.4(e)); `arrival_insert` does (`store.rs:2425`, the `change_seq` update near its end), which is why bring back has its own store method and does not call it.
- Hazard: the jobs queued for the new photos are `original_preview` and `render_preview`; the hash is known (computed while the file is written), so no `hash` job.
- Hazard: `sync_folder` needs Capture One; when it fails (not running) the bring back still succeeds with `synced = false`. The existing lookup re-syncs a folder whose photo is on disk but not listed (`adapter.rs:176-183`), so the first render repairs it.
- H3·C7(d): the publish steps cannot be reached naturally for a cleared item before H4; the row sets the step through the store, which is the state H4's details-only publish produces.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-testkit --test suite h3_bring_back::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::` · `cargo test -p psm-testkit --test suite a17_delete_session::` · `cargo test -p psm-testkit --test suite t2_template_placement::` · `cargo test -p psm-desktop` · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings`

## Unit 3 — Publish after bring back sends only changes; retention

**Unit id:** `H3·U3` · **Rows:** H3·C10–C14 · **Changes app files:** yes (tests; production only if a row fails)

### Goal
The rows that prove M-H3 on the real publish service, the shop-only "own media" rule, and MC-H4.8 on the real storage scan.

### Read first
Master plan §5 **MC-H4.7, MC-H4.8** · `crates/core/src/services/publish.rs` (`preview` :661-746, `confirm` :748, `run_item` :947, `prepare` :1146-1279, `upload` :1446-1690, `update` :1720-1810) · `crates/core/src/mechanisms/{publish.rs (`replacement_candidates` :40), reconcile.rs (`plan` :17-46)}` · `crates/core/src/services/previews.rs` (`on_variant_adjusted` :529, `check_sidecars` :801-838) · `crates/infra/src/sqlite/store.rs` (`adjusted_event`, the `raw_state` update at :1921) · `crates/core/src/services/storage.rs` (`retention_scan` :702) · `crates/testkit/tests/suite/{p14_publish_update.rs (:674-860: how a test raises the retouch generation with `adjusted_event` and publishes), p30_outputs_in_export_and_publish.rs (:673: a composition revision replaces the media in place), p62_retention_after_publish.rs}` · `crates/testkit/src/fake_commerce.rs` (`mutation_ops` :167, `uploaded_sha`) and `fake_capture_one.rs` (:95-100, :390-410: settings sidecars).

### Files
New: `crates/testkit/tests/suite/h3_publish_after_bring_back.rs`. Edited: `crates/testkit/tests/suite/main.rs`, `docs/guide/04_change_impact.md`. Production files only when a row fails (then: the smallest change, declared under **Deviations** with the row that needed it).

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. Helpers in the test file: `brought_back(a, sku, main, condition)` (publish, clear, bring back, `tick`, `complete_all_renders`), `publish(a, session)` (preview, confirm, run to the end, completing the renders it asks for), `uploads_since(a, mark)` (the upload-type operations in `mutation_ops()` after a mark).
2. Tests for the rows.
3. Guide: `04_change_impact.md` (a row "the link fields `replacement_candidates` reads, or `reconcile::plan`'s own set → brought-back photos rely on them: `h3_publish_after_bring_back::`").

### Criteria (5 criteria, 18 rows)
Boundary: `PublishService` and `StorageService` on `TestApp::with_fake_shop()`; outcome = the mutations `FakeCommerce` recorded during the publish, its gallery and condition field afterwards, stored links, files on disk. "Brought-back item (3 + 2)" = the archived item of U2 after `bring_back`, with gallery `[m1, m2, m3]` and condition `[c1, c2]` on the fake shop.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H3·C10(a) | publish | the brought-back item (3 + 2), nothing changed | the publish completes (`done`); during it `FakeCommerce` recorded no staged upload, no upload, no media or file creation and no media deletion; the gallery is `[m1, m2, m3]` and the condition field `[c1, c2]` | M-H3; 10H §7.5 ("an untouched photo is not uploaded again") |
| H3·C10(b) | same | as (a), but first the fake Capture One writes a settings sidecar for photo 1's JPEG and the sidecar check runs (`tick`) | as (a): nothing uploaded | M-H3; MC-H4.7 (a first settings file is not an edit) |
| H3·C10(c) | `preview` | the brought-back item | `announced_removals` is empty | MC-H4.7; 10H §7.5 |
| H3·C11(a) | publish | photo 2's composition set to a tighter rect | exactly one upload; the gallery is `[m1, n, m3]` with `n` a new id; `m2` is no longer in the gallery; photo 2's link is `ready` with `external_id` `n`, revision 1 and `m2` among its superseded ids | M-H3; 10H §7.5 (replaces its media) |
| H3·C11(b) | same | photo 3 turned a quarter (nothing else) | exactly one upload; the gallery is `[m1, m2, n]` | M-H3; 10H §7.5 (rotation) |
| H3·C11(c) | same | a Capture One edit of photo 1 (`adjusted_event(raw, true)` as `p14` does) | exactly one upload; the gallery is `[n, m2, m3]` | M-H3; 10H §7.5 (Capture One edit); 07B MC-21 |
| H3·C11(d) | same | photo 1 changed to a split | exactly two uploads; the gallery is `[l, r, m2, m3]`; `m1` is gone | M-H3; 10D MC-29 |
| H3·C12(a) | same | main reordered to photos `[3, 1, 2]` | no upload; one reorder; the gallery is `[m3, m1, m2]` | M-H3; 10H §7.5 ("a new order reorders the gallery") |
| H3·C12(b) | same | photo 2 denied (hidden) | no upload; the gallery is `[m1, m3]` | 10H §7.5 ("a hidden photo removes its media") |
| H3·C12(c) | same | condition photo 1's composition changed | exactly one upload (a file); the condition field is `[n, c2]`; the gallery is unchanged | M-H3; 10D MC-29 §Damage report |
| H3·C12(d) | same | a new RAW arrives in the session (main, last) and its renders complete | exactly one upload; the gallery is `[m1, m2, m3, n]` | M-H3 ("plus new ones") |
| H3·C13(a) | publish | a shop-only item brought back (H3·C5(a): `[m1, m2]`), main reordered to `[2, 1]` | the publish completes; no `foreign_media_added`; no upload; the gallery is `[m2, m1]` | 10H §7.5 ("count as the app's own"); MC-H4.7 |
| H3·C13(b) | same | the same item, photo 1's composition changed | exactly one upload; the gallery is `[n, m2]`; `m1` is gone | 10H §7.5 ("may reorder, replace and remove them") |
| H3·C13(c) | `preview` | the same item right after its bring back | `announced_removals` is empty and `create_item` is false | MC-H4.7 |
| H3·C14(a) | `StorageService` scan | the brought-back item (3 + 2), untouched; the clock 30 days later | not cleared: the five JPEG files exist, `cleared_at` is null, and the scan made no `media_readback` read for it | MC-H4.8; 10H §7.7 |
| H3·C14(b) | same | after (a): photo 2's composition changed, published (done), the clock past the retention days | cleared: `cleared_at` set, the five JPEG files are gone, the record is `[m1, n, m3]` and `[c1, c2]` | MC-H4.8; 10F §3; M41 |
| H3·C14(c) | same | as (b) but "Keep files" set before the scan | not cleared | 10F Q2 |
| H3·C14(d) | `bring_back` a second time | after (b) | `Ok`; the files are `<key>-main-01-b2.jpg` … (the first set's names belong to the earlier photo rows) and the session shows 3 + 2 photos | MC-H4.2; H2 (never over an earlier photo's name) |

### Delegated choices
The helper names · whether each row builds its own app or rows share one where the outcome allows.

### Notes
- Hazard: these rows are the acceptance of MC-H4.4(c). If `FakeCommerce` records an upload in H3·C10(a), the cause is the link written by `Store::bring_back` (generation, revision, state, role, slot), not the publish service.
- Hazard: a publish still renders a master and an output for **every** photo of the session (`ensure_export`, `publish.rs:1281`), also the untouched ones; that costs time but uploads nothing. Do not "optimise" it here.
- Hazard (H3·C10(b)): the fake writes sidecars only when asked (`fake_capture_one.rs:390-410`); the row must make the sidecar appear **after** the bring back and before the publish.
- H3·C12(b) uses the photo's deny (P66); the undo window closes at the publish, which deletes that JPEG file: that is P66's rule for a denied photo and is expected.

### Unit test commands
`cargo test -p psm-testkit --test suite h3_publish_after_bring_back::` · `cargo test -p psm-testkit --test suite p14_publish_update::` · `cargo test -p psm-testkit --test suite p30_outputs_in_export_and_publish::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-testkit --test suite h3_jpeg_photos::` · `cargo test -p psm-testkit --test suite h3_bring_back::` · `cargo test -p psm-testkit --test suite h3_publish_after_bring_back::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::` · clippy on the touched crates, fmt. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: `Store::bring_back` is one transaction and is called only after every file exists; `bring_back` calls no adapter method that takes a `ConfirmedRun`; every exit path clears the "running" mark and, on failure, removes only the files this attempt wrote; the publish service was not changed (or the change is declared with its row); the evidence doc holds VERIFY-JPEG (probed or UNPROBED). Notes go to `docs/implementation/backlog_archive/H3_bring_back.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
Nothing to try on screen yet: the next milestone adds the workspace and the Bring back button. (For the orchestrator's live check after H4: VERIFY-JPEG in the Capture One evidence, and a bring back of one archived test product followed by a publish with one re-framed photo — Shopify must show one replaced picture and the others untouched.)

## Review log

- **2026-11-23, orchestrator: carried from H1's projection r0** (`handoffs/reviewer/handoff_H1_points_and_remote_record_projection_r0.md`). L3: `NotRemote` is added by H1 (U2 task 2 now adds only the two bring-back errors). L9: `FakeCommerce::fetch_original` / `original_fetches` and `set_image` moved here (U2 task 5). L10: `RemoteItems::new` gains `editing`, `capture_root`, `history` here (U2 task 5). Rows and counts unchanged.
