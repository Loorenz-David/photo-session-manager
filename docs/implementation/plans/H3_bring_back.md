---
phase: H3_bring_back
title: Bring back — the app can download an archived or shop-only product's published pictures from Shopify at full size into its session folder and make them the session's photos (all of them or none), JPEG photos work like RAW photos in every photo operation, and the next Publish uploads only the photos that were changed since
difficulty: high
projection: done (r0 amendments applied 2026-11-23)
depends_on: [H2_list_pictures_pages_search]
blocked_on: none
---
# H3 — Bring back

**Tracker:** IMPLEMENTED (2026-11-23, claude opus implementer r1: 3 units, 64 rows → 59 behaviour tests + 5 unit tests, all green; closing gate green: workspace suite, clippy, fmt, typecheck, npm test; checkpoints fb8d34f, c9dad23, 747a2c4; handoff `handoffs/implementer/handoff_H3_bring_back_implement_r1.md`)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `10H` §7 by contract **MC-H4** (`docs/implementation/master_plan_archive.md` §5), with its invariants M-H1, M-H2, M-H3, M-H5. (U1) JPEG photos in the photo pipeline. (U2) `RemoteItems::bring_back`: download everything, then import in one transaction. (U3) The proof that a publish afterwards sends only changes, and that retention waits for the next publish. **Not here:** the command, the prompt, the progress on screen, activation after bring back (H4); getting the RAW back (10H §10); any change to the publish service's code (MC-H4.7: the links are enough — if a U3 row needs a publish change, that is a finding to report, not to work around).

## How to work this plan (flow.md §10, master_plan_archive.md §3)
One runner phase of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `H3·C<n>(<row>)`; tests named after their row (`h3_c1a_<slug>`). Open details: the smallest option consistent with 10H §7 and MC-H4, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_archive/H3_bring_back.md`. **Never call Shopify or its CDN, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | JPEG photos in the photo pipeline | 2 | 10 | **yes** |
| U2 | Bring back: download all, then import in one transaction | 7 | 36 | **yes** |
| U3 | Publish after bring back sends only changes; retention | 5 | 18 | **yes** (tests; production only if a row fails) |
| **Total** | | **14** | **64** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6A, §10) · `docs/implementation/master_plan_archive.md` §3 (Rule 9, Rule 17), §4, **§5 MC-H4 (every item), MC-H5.4, MC-H3.1** · book `10H` §2 (decision 3: the JPEG), §7, §9 · book `10F` (Rules, Q1, Q4, M41) · book `10D_composition_and_framing.md` (MC-27 composition values and frames, MC-28 master and output, MC-29 links and replacement) · book `07A`/`07B` as cited by `crates/core/src/services/publish.rs` (MC-11 adoption, MC-12 reconciliation, MC-21 retouch) · `docs/guide/03_rules_and_principles.md` (H1–H5) · `docs/guide/04_change_impact.md` §Persistence and §Shooting templates · the H1 and H2 plans (what exists).

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Nothing to see yet:** the next milestone puts this behind the workspace.
- **Bring back is all or nothing:** if one picture cannot be downloaded after three tries, nothing changes and the downloaded files are removed.
- **The files are named after the product's folder** (`<folder>-main-01.jpg` …); a second bring back of the same product adds `-b2`.
- **Bringing back is not an edit:** a published product stays "Published", and it is not cleared again before its next publish.
- **The earlier photo entries of a cleared product are kept, hidden** (their RAWs are gone); the session's undo list starts empty after a bring back.
- **A brought-back photo starts exactly as published:** no crop, no turn, no house style applied again.
- **A picture the shop sends back that is not a JPEG counts as not downloaded** (projection r0, L6): the bring back fails and says so, instead of bringing back a photo the app cannot show or frame.
- **In a product with a shooting template, the brought-back photos keep the published order and count as placed by hand** (projection r0, L3): a photo shot afterwards fills a free container or goes to Condition, and never becomes the first picture in place of the published cover.

## Unit 1 — JPEG photos in the photo pipeline

**Unit id:** `H3·U1` · **Rows:** H3·C1–C2 · **Changes app files:** yes

### Goal
The `original_preview` job handles a JPEG source (MC-H4.6); everything else in the pipeline is shown to work on a photo whose file is a JPEG; the Capture One facts are probed and recorded.

### Read first
Master plan §5 **MC-H4.6** · `crates/integrations/src/lib.rs` (`OriginalPreviewJob` :1573-1611) · `crates/infra/src/rawpreview.rs` (`EmbeddedJpeg` :8-27, `compose_frame` :93-117, `orientation` :241) · `crates/core/src/ports/raw_previews.rs` · `crates/infra/src/imaging/{mod.rs (`dimensions` :20, `thumbnail` :26-45), metadata.rs (the `exif` reader in use)}` · `crates/infra/src/sqlite/store.rs` (`adopt_frame_tx` :326-404: the first-frame branch and the P58 camera → Capture One rule at :363-384) · `crates/core/src/services/photos.rs` (`import` :25-92: copies any file) · `crates/core/src/services/arrival.rs` (`import` :274-283, `accept` :284-381) · `crates/core/src/mechanisms/arrival.rs` (`is_candidate` :12-36) · `crates/core/src/services/composition.rs` (`set_unrecorded` :80-95) · `crates/core/src/services/previews.rs` (`request_initial` :257, `on_processing_done` :398-470) · `crates/integrations/src/editing/capture_one/adapter.rs` (`variant_background` :169-184, `resync_folder` :189-213) · `crates/testkit/src/fixtures/{synthetic_jpeg.rs, synthetic_raw.rs}` · `crates/testkit/src/fake_capture_one.rs` (`add_image` :442, `find_image` :853-863, `output_bytes` :344-349: not a decodable JPEG) and `test_app.rs` (`complete_render_with` :1003) · `crates/testkit/tests/suite/p58_fast_compose_on_camera_preview.rs` (frame rows on the original preview; :107-123 a render completed with a real JPEG) · `docs/intention/evidence/capture_one_16.8.6.md` (the VERIFY list and its format).

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
| H3·C2(c) | preview render (`tick`, then `complete_render_with(<the preview job>, SyntheticJpeg::rgb(1500, 1000, …))`, as `p58` :107-123) | the photo of H3·C1(a), its file first registered with the fake Capture One (`a.fake.add_image(<its raw_path>, &["v1"], false)`) | the fake Capture One received a process operation for the JPEG's variant; the photo has an adjusted preview; the composition's frame is 1500 × 1000 with source `preview` (Capture One's measurement replaced the camera one) | 10H §7.4; MC-H4.6 (P58 rule) |
| H3·C2(d) | export of the session (`ExportService`, masters completed with `complete_all_renders`) | one imported JPEG photo, classified main | the export holds one output for it | 10H §7.3; 10D MC-28 |
| H3·C2(e) | the watcher path (`app.watcher_event`) | a file `late.jpg` written into the active session's capture folder | no photo row is added | MC-H4.6 (the candidate rule is unchanged) |

### Delegated choices
Whether `JpegSource` uses `kamadak-exif` or reads the tag by hand · how a JPEG over 1600 px is scaled (the existing `thumbnail` code path with quality 80 is enough).

### Notes
- Hazard: `DynamicImage::resize` (`imaging/mod.rs:30`) **enlarges** a smaller image to the bounds; for a JPEG at or under 1600 px copy the file, do not call it (H3·C1(c)).
- Hazard: the original preview is unrotated pixels and `orientation` tells the UI how to show it, as for an embedded camera preview; do not bake the rotation into the file and also store the orientation.
- Hazard: an imported JPEG gets the house style like any import (`follow.photo_arrived`, `arrival.rs:368-372`); that is today's import behaviour and is not changed here. Bring back (U2) does not go through `accept`, so its photos get no style.
- H3 (hard constraint): scaling a JPEG is not decoding a RAW; it stays in `crates/infra` (`rawpreview.rs` / `imaging`), the only crate that decodes pixels.
- Test setup (projection r0, checked in code): the fake Capture One issues a process operation only for an image it knows, so H3·C2(c) and H3·C2(d) first register the JPEG with `a.fake.add_image(<raw_path>, &["v1"], false)` (the `p58` pattern). `complete_all_renders` completes a preview with `output_bytes`, which do not decode, so `on_processing_done` measures nothing and the frame stays `camera`: H3·C2(c) completes its preview with `complete_render_with` and a real `SyntheticJpeg` of the photo's orientation (L2). H3·C1(a): `image` 0.25.10 rounds 2000 × 1600/3000 to **1067**, and decoding does not apply the EXIF rotation. H3·C1(b): `SyntheticJpeg::with_orientation` can write its APP1 with `JpegEncoder::set_exif_metadata`; the test can read it back with `kamadak-exif`'s `read_from_container` (`rawpreview::orientation` walks a bare TIFF and does not find APP1 by itself). H3·C1(d): the job's status is read by SQL, as `p05_capture_arrival.rs:427` does.

### Unit test commands
`cargo test -p psm-infra --lib rawpreview` · `cargo test -p psm-testkit --test suite h3_jpeg_photos::` · `cargo test -p psm-testkit --test suite p58_fast_compose_on_camera_preview::` · `cargo test -p psm-testkit --test suite p06_live_previews::` · `cargo clippy -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — Bring back: download all, then import in one transaction

**Unit id:** `H3·U2` · **Rows:** H3·C3–C9 · **Changes app files:** yes

### Goal
`bring_back_names`, `estimate_bytes`, `Store::bring_back`, `RemoteItems::bring_back`, `AppEvent::BringBack`, the three errors.

### Read first
Master plan §4 (Vocabulary, Ports, Errors, Events, Words), §5 **MC-H4.1–5** and its invariants · `crates/core/src/services/remote.rs` (H1, H2) · `crates/infra/src/sqlite/store.rs` (`arrival_insert` :2425-2501: how a photo row, its composition, `raw_state` and its jobs are inserted, the canonical path at :2447, the job insert at :2496-2501, the generation-keeping upsert at :2497; the `raw_state` upserts by path at :2300, :2305, :2668; `insert_default_composition` :227; `upsert_photo_link` :2974; `delete_photo_row` and `photo_places`; `mark_item_cleared` :3236) · `crates/infra/src/sqlite/migrations/{002_domain.sql (photos :35-52), 004_compositions.sql, 005_photo_link_slots.sql, 015_photo_places.sql}` · `crates/core/src/domain/{photo.rs, composition.rs (`Shape::Original` :27), publish.rs (`PhotoLink` :242-256)}` · `crates/core/src/services/arrival.rs` (`capture_root` :92-106) · `crates/core/src/services/history.rs` (`retire_session` :309-341) · `crates/core/src/ports/editing.rs` (`sync_folder` :58) · `crates/core/src/services/sessions.rs` (`view` :348-401: which photos a session shows) · `crates/core/src/services/storage.rs` (`cleared_item_photos` :917, `item_files` :931) · `crates/core/src/services/previews.rs` (`check_missing_files` :165-225: a missing file of a non-deleted photo is reported on Health) · `crates/core/src/services/template_placement.rs` (`place_arrival` :29, `gaps` :310-340, `gap_for_arrival` :346-364, `renumber_main` :593, `main_positions` :612-633, `put_by_hand` :694-702) with `docs/implementation/master_plan_templates.md` §5 and book `10G` §4.2, §4.4, §6.4 · `crates/core/src/services/previews.rs` (`on_variant_adjusted` :530-535 looks a photo up by its canonical path) · `crates/testkit/src/fake_capture_one.rs` (`add_image` :442, `sync_folder` :753-771 lists only images a test registered, `inspect_document` :660-680 answers `DocumentNotOpen` when `open_documents` lacks the document) · `crates/testkit/src/fake_commerce.rs` (`fake_image_bytes` :127 and `fake_refreshed_url` :107, `read` :302-318: the simulated kill) · `crates/testkit/tests/suite/p62_retention_after_publish.rs` (helpers :50-300) · `crates/testkit/tests/suite/a17_delete_session.rs`.

### Files
New: `crates/core/src/services/bring_back.rs`, `crates/testkit/tests/suite/h3_bring_back.rs`. Edited: `crates/core/src/{domain/remote.rs, mechanisms/remote.rs, ports/store.rs, services/mod.rs, services/remote.rs, events.rs, error.rs}`, `crates/infra/src/sqlite/store.rs`, `crates/integrations/src/lib.rs` (the capture-root source and the history handle given to `RemoteItems`), `apps/desktop/src-tauri/src/{lib.rs (the `bring_back_progress` arm), commands/items.rs (the error codes)}`, `apps/desktop/src/services/{bindings.ts (generated), events.ts (`onBringBack`)}`, `crates/testkit/src/{fake_commerce.rs, test_app.rs}`, `crates/testkit/tests/suite/main.rs`, `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
`mechanisms::remote::{bring_back_names, estimate_bytes}`.

### Ordered tasks
1. `bring_back_names` (MC-H4.2) and `estimate_bytes` (MC-H3.5: `bytes` or `UNKNOWN_PICTURE_BYTES` each).
2. Errors `BringBackRunning`, `BringBackFailed {reason}` with their command codes (`NotRemote` / `not_remote` exists since H1, its projection L3); `BringBackState`, `BringBackProgress`, `BringBackOutcome`; `AppEvent::BringBack` with its `TauriEvents` arm and `onBringBack`.
3. `Store::bring_back(BringBackInput {session_id, item_id, files: Vec<BroughtBack {role, position, media_id, path, sha256}>, link_role_condition, at}) -> Vec<i64>`: **one transaction** doing MC-H4.4 (a)–(e) exactly. `BringBackInput` and `BroughtBack` live in `psm_core::domain::remote`; `role` is `ArchivedImage`'s role type (L12). `link_role_condition` = `attribute:<commerce.condition_photos_key()>`, as `SessionsService::view` builds it (`sessions.rs:385`; L13). Each new photo's `raw_path` is the **canonical** path of its file (`fs::canonicalize`, as `arrival_insert` does at `store.rs:2447`; L5): a non-canonical path (`/var/…` instead of `/private/var/…`) would make every later Capture One edit event miss the photo. Each new link's `retouch_gen` is read from the photo's `raw_state` row as stored in the same transaction (not the literal 0: a row for that path may pre-exist; L16). **Templated session** (its item type has a shooting template; L3, from 10G §4.2/§4.4/§6.4): (i) the brought-back main photo at position *k* gets the "placed by hand" mark (`place_by_hand = 1`, the row `put_by_hand` writes) with `slot` = the `final_no` of the container whose `final_no = k`, and `slot` null when *k* is past the last container, so the published order is kept and a later arrival finds those containers filled; (ii) in every session, each retired row (MC-H4.4(a)) keeps a `photo_places` row with `released = 1` (created with `released = 1` when missing; its other columns unchanged), so the retired shots leave the shot count as a released deleted shot does (10G §6.4) and leave no gap to refill. Where the template's shooting order differs from its final order, 10G §6.2's warning may show for these photos; that is expected.
4. `RemoteItems::bring_back(session_id) -> Result<BringBackOutcome>` (`services/bring_back.rs`): MC-H4.1 preconditions in their order; names (rule 2, `taken(name)` = `store.photos_for_raw(<canonical capture root>/<folder key>/<name>)` is not empty (L5); a store error inside `taken` is kept, `taken` answers `false` from then on so the names end, and `bring_back` then fails with that error before any download (L9: fail closed, never an endless suffix loop)); downloads (rule 3) with progress events; a downloaded body that does not start with the JPEG marker `FF D8 FF` counts as a failed download (same retries, same clean-up; L6, MC-H4.3); an `ImageGone` whose `media_urls` answer is `None` or carries no URL stores that picture as gone (`update_remote_urls(item, [(media_id, None, None)])`, MC-H1.4) and fails with `1 photo is no longer in the shop` (`<n> photos …`); a refreshed URL gets the full 1 + `DOWNLOAD_RETRIES` tries, the `Gone` attempt not counted (L11); on any missing picture the clean-up and the failure; the import (task 3), **run inside `History::retire_session(session, retired photo ids, act)`'s `act`** so no undo step naming a retired photo can apply between the import and the retirement (L8; MC-H4.5); a `Store::bring_back` error after every download succeeded gets the same clean-up and `BringBack {Failed}` event and is returned unchanged (L10); then the rest of MC-H4.5. The item's cover and 1600 px caches (`remote/<item_id>/`, `archived/<item_id>/`) are left as they are (L14; the record stays, MC-H4.4(d)). A per-session "running" mark (in memory) gives `BringBackRunning` and the progress H4's view reports; it is cleared on every exit path.
5. `RemoteItems::new` gains `editing`, `capture_root`, `history` (H1 took only what it read; master plan §4 Service), wired in `App::build`. **The production `capture_root` closure** (L4) calls `editing.inspect_document()` and canonicalizes its `capture_root`, with **no cache** (bring back is a press; `ArrivalService::capture_root()` caches the first answer forever, `arrival.rs:92-106`, so it is not used here). `TestApp::with_fake_shop()` gives `RemoteItems` the same closure over the fake editing, the test's history and fake editing. `FakeCommerce` gains (carried from H1's projection, L9) `fetch_original` (records the URL in `original_fetches`; serves `set_image(media_id, bytes)` when registered, else `fake_image_bytes`, which the L6 JPEG check refuses; maps a URL to its media for the stored URL and for every refreshed URL `fake_refreshed_url(id, n)` hands out (L15); consumes H2's `image_faults` and honours H2's `set_fetch_delay` as `fetch_image` does) and `set_image`.
6. Guide: `01_product.md` (the workflow row "Bring back"), `02_architecture.md` (§Remote items: the bring-back sequence), `03_rules_and_principles.md` (H2's exceptions list gains: the app deletes only the files a failed bring back itself downloaded; a table "10H invariants M-H1–M-H5" with where each bites), `04_change_impact.md` (rows: "a new table or column that names a photo → retire it in `Store::bring_back` (a) like `delete_photo_row`"; "the publish link fields → `Store::bring_back` (c) writes them for brought-back photos").
7. Tests for the rows.

### Criteria (7 criteria, 36 rows)
Boundary: `bring_back_names` / `estimate_bytes` (pure); `RemoteItems::bring_back` on `TestApp::with_fake_shop()` with a fake Capture One document (outcome = the files in the session folder, stored rows read through the store, the session view, what `FakeCommerce` and the fake Capture One recorded, emitted events). "Archived item (3 + 2)" = an item with 3 main and 2 condition photos, published, then cleared by the real clearing; its Shopify media are `m1, m2, m3` and `c1, c2`, each registered with `set_image` as a distinct `SyntheticJpeg`. Every other bring-back fixture (shop-only items, templated items) registers each of its pictures the same way: unregistered media answer `fake_image_bytes`, which MC-H4.3's JPEG check refuses (L6). A row that renders a brought-back photo first registers its file (canonical path) with the fake Capture One (`add_image`; L15).
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
| H3·C6(f) | `bring_back` | the archived item (3 + 2); `m2` registered with `set_image` as bytes that start with the PNG signature `89 50 4E 47 0D 0A 1A 0A` | `Err(BringBackFailed)` with message `Could not bring back the photos: 1 photo could not be downloaded`; `m2`'s URL was fetched exactly 3 times; the session view and the photo rows are as before the call; `cleared_at` still set; the folder holds none of the five names and no `.part` | MC-H4.3 (a body that is not a JPEG is a failed download; projection r0 L6); M-H2 |
| H3·C7(a) | `bring_back` | a session whose item is `Local` | `Err(NotRemote)`; no fetch | MC-H4.1 |
| H3·C7(b) | same | an app without an integration (`TestApp::with_fake_document()`) | `Err(NoIntegration)` | MC-H4.1; H6 |
| H3·C7(c) | same | two threads call for the same archived item; the fake's fetch delay holds the first one in its downloads | one call answers `Ok` with five photos, the other `Err(BringBackRunning)`; the session has five visible photos | MC-H4.1 |
| H3·C7(d) | same | the archived item whose latest publish item's step is set to `update` through the store (a publish in flight) | `Err(BringBackFailed)` naming `wait until the publish has finished`; no fetch | MC-H4.1 |
| H3·C7(e) | same | the archived item, then the fake Capture One's `open_documents` cleared (`fake_capture_one.rs:70`), so the production capture-root closure (U2 task 5: `inspect_document`, no cache) answers `DocumentNotOpen` | `Err(BringBackFailed)` naming `the capture folder is not known yet`; no fetch | MC-H4.1 |
| H3·C8(a) | `FakeCommerce::mutation_ops()` | after H3·C4(a), C5(a), C6(a), C6(c) | empty | M-H1; 10H §7.6 |
| H3·C8(b) | the fake Capture One's operations | after H3·C4(a) | the operations the fake recorded **during the call** are exactly `document_info_started`, `document_info` (the capture-root read, U2 task 5) and `sync_folder:<capture root>/BEYO-12`, in that order (`EditingProvider` has no remove, delete or move method; the exact list is what can fail) | M-H2; MC-H4.5 |
| H3·C8(c) | "Delete session" (`psm_integrations::session_delete::delete`) | the session of H3·C5(a) after its bring back | its three JPEG files are in the fake Trash; `FakeCommerce` recorded no read and no mutation during the delete | M-H5; H2 (Trash, never a permanent delete) |
| H3·C8(d) | Health (`check_missing_files` on focus) | after H3·C4(a) | Health holds no `file_missing` (the retired rows are not "deleted outside Session Manager") | MC-H4.4(a) |
| H3·C9(a) | `bring_back` in a templated session | an archived item whose item type has a shooting template with 3 containers whose shooting order equals their final order; (3 + 0) pictures | the three new main photos are at positions 1–3 in the order `m1, m2, m3`; each has a `photo_places` row with `place_by_hand` 1 and `slot` = its position; each of the three retired rows has a `photo_places` row with `released` 1 | MC-H4.4(a), (b); 10G P7 (the template never reorders them), §6.4 |
| H3·C9(b) | same, then an arrival | after (a), a new RAW arrives in the session (`watcher_event` into `<capture root>/<folder key>/`) | the arrival is classified Condition at position 1 (its shot index 4 is past the 3 containers, 10G §4.4); Main is still `m1, m2, m3` at positions 1–3 | MC-H4.4 (placed by hand; retired shots released); 10G §4.4 |
| H3·C9(c) | same, then an arrival | as (a) with a (2 + 0) item (`m1, m2`; 3 containers), then a new RAW arrives in the session | the arrival is Main at position 3 (container 3); `m1, m2` stay at positions 1–2 | MC-H4.4(a) (the retired shots leave the shot count: with them counted the arrival would go to Condition); 10G §4.2, §6.4 |

### Delegated choices
How the "running" mark is kept · the exact `BringBack {Running}` cadence when several downloads finish together (the `done` values must still be 1…N in order; no initial `done: 0` event, H3·C4(i)) · whether a leftover file is removed before the download or replaced by the rename · whether the JPEG-marker check (L6) is a private helper or a pure function in `mechanisms/remote.rs` (not a Core mechanism either way; H3·C6(f) observes it).

Decided at projection r0 (orchestrator; the projectionist's recommendation where one was given, else the smallest reversible option; each is in U2 task 3–5): L8 `Store::bring_back` runs inside `History::retire_session`'s `act` · L9 `taken` fails closed (a store error is kept, the names end, `bring_back` fails before any download) · L10 a store failure after the downloads gets the download failure's clean-up and `BringBack {Failed}`, the store error returned unchanged · L11 a `Gone` picture whose refresh finds no URL is stored as gone and fails the bring back with `1 photo is no longer in the shop`; the refreshed URL gets the full 1 + `DOWNLOAD_RETRIES` tries · L12 `BringBackInput` / `BroughtBack` in `psm_core::domain::remote`, `role` as `ArchivedImage`'s · L13 the condition link role is `attribute:<commerce.condition_photos_key()>` (equal to publish's key whenever the media-list definition exists; publish skips condition outputs when it does not) · L14 the item's cover and 1600 px caches are left · L15 the fixtures register files with the fake Capture One (`add_image`) and `FakeCommerce` maps stored and refreshed URLs to their media · L16 the link's `retouch_gen` is read from the stored `raw_state` row · L3 the templated-session marks of task 3 (from 10G; no owner card) · L4 the uncached `inspect_document` capture-root closure.

### Notes
- Hazard (M-H2, the heart of this unit): the import is **one** store method in **one** transaction, called only after every file exists. No photo row may be written while a download is still running, and no file may be left behind by a failed attempt.
- Hazard: `UNIQUE(session_id, raw_path)` on `photos` (`002_domain.sql:51`). After a product was brought back, republished and cleared again, its retired rows still carry the first set of names: that is what rule 2's suffix is for (H3·C3(b)). Do not rename or delete the retired rows.
- Hazard: the retired rows must be `deleted` **and** `raw_missing` with their preview paths nulled: `deleted` keeps them out of Health's missing count (`previews.rs:205`), the nulled paths keep M-H4 for every later reader, and `check_missing_files` then finds nothing to change (H3·C8(d)).
- Hazard: do not write `photo_denials` rows for the retired photos (P66's undo window and RAW deletion do not apply: there is no file).
- Hazard: do not touch `sessions.change_seq` (MC-H4.4(e)); `arrival_insert` does (`store.rs:2425`, the `change_seq` update near its end), which is why bring back has its own store method and does not call it.
- Hazard: the jobs queued for the new photos are `original_preview` and `render_preview`; the hash is known (computed while the file is written), so no `hash` job.
- Hazard: `sync_folder` needs Capture One; when it fails (not running) the bring back still succeeds with `synced = false`. The existing lookup re-syncs a folder whose photo is on disk but not listed (`adapter.rs:176-183`), so the first render repairs it.
- H3·C7(d): the publish steps cannot be reached naturally for a cleared item before H4; the row sets the step through the store, which is the state H4's details-only publish produces.
- Hazard (MC-1 path identity, L5): temp dirs on macOS are `/var/…`, which canonicalize to `/private/var/…`. `on_variant_adjusted` finds a photo by `canon(raw)` (`previews.rs:530-535`), so a stored `raw_path` that is not canonical silently loses every Capture One edit event and H3·C11(c) would still pass (it calls `adjusted_event` with the stored path). Store the canonical path and compute `taken` on canonical paths.
- Hazard (templated sessions, L3): without the marks of task 3 the retired photos' containers look empty to `gaps` (`template_placement.rs:310-340`), `gap_for_arrival` refills container 1 and `main_positions` sorts slotted photos first (`:612-633`), so the next RAW becomes Main 1 and the next Publish changes the shop's cover. H3·C9(b) and (c) are the rows that fail on that.
- Test setup (projection r0): H3·C6(e)'s "kill" is a panic from `FakeCommerce::read` (`fake_commerce.rs:302-318`); inside `std::thread::scope` workers it re-raises in the caller after every worker ends, so the test wraps the call in `catch_unwind` before `kill_and_restart`. H3·C7(c) needs a test-side sync (wait for the first `original_fetches` entry) before the second call. H3·C4(j): clearing does not touch the undo history (`services/storage.rs:579-639` retires only denied photos), so bring back is the only cause. H3·C8(d) passes only if the retired rows are `deleted` (`previews.rs:205`), so it discriminates.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-testkit --test suite h3_bring_back::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::` · `cargo test -p psm-testkit --test suite a17_delete_session::` · `cargo test -p psm-testkit --test suite t2_template_placement::` · `cargo test -p psm-desktop` · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings`

## Unit 3 — Publish after bring back sends only changes; retention

**Unit id:** `H3·U3` · **Rows:** H3·C10–C14 · **Changes app files:** yes (tests; production only if a row fails)

### Goal
The rows that prove M-H3 on the real publish service, the shop-only "own media" rule, and MC-H4.8 on the real storage scan.

### Read first
Master plan §5 **MC-H4.7, MC-H4.8** · `crates/core/src/services/publish.rs` (`preview` :661-746, `confirm` :748, `run_item` :947, `prepare` :1146-1279, `upload` :1446-1690, `update` :1720-1810) · `crates/core/src/mechanisms/{publish.rs (`replacement_candidates` :40), reconcile.rs (`plan` :17-46)}` · `crates/core/src/services/previews.rs` (`on_variant_adjusted` :529, `check_sidecars` :801-838, run only from `on_focus` / `on_startup` :786-795; the first-fingerprint clause :810-814) · `crates/infra/src/sqlite/store.rs` (`PreviewScope::Live` :592: published-unchanged sessions are left out, A16) · `crates/testkit/src/test_app.rs` (`focus` :997, `write_sidecar` :1130) · `crates/infra/src/sqlite/store.rs` (`adjusted_event`, the `raw_state` update at :1921) · `crates/core/src/services/storage.rs` (`retention_scan` :702) · `crates/testkit/tests/suite/{p14_publish_update.rs (:674-860: how a test raises the retouch generation with `adjusted_event` and publishes), p30_outputs_in_export_and_publish.rs (:673: a composition revision replaces the media in place), p62_retention_after_publish.rs}` · `crates/testkit/src/fake_commerce.rs` (`mutation_ops` :167, `uploaded_sha`) and `fake_capture_one.rs` (:95-100, :390-410: settings sidecars).

### Files
New: `crates/testkit/tests/suite/h3_publish_after_bring_back.rs`. Edited: `crates/testkit/tests/suite/main.rs`, `docs/guide/04_change_impact.md`. Production files only when a row fails (then: the smallest change, declared under **Deviations** with the row that needed it).

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. Helpers in the test file: `brought_back(a, sku, main, condition)` (publish, clear, register each picture with `set_image` as a distinct `SyntheticJpeg`, bring back, register each brought-back file with the fake Capture One (`a.fake.add_image(<raw_path>, &["v1"], false)`, the `p58` pattern; L15), `tick`, `complete_all_renders`), `publish(a, session)` (preview, confirm, run to the end, completing the renders it asks for), `uploads_since(a, mark)` (the upload-type operations in `mutation_ops()` after a mark).
2. Tests for the rows.
3. Guide: `04_change_impact.md` (a row "the link fields `replacement_candidates` reads, or `reconcile::plan`'s own set → brought-back photos rely on them: `h3_publish_after_bring_back::`").

### Criteria (5 criteria, 18 rows)
Boundary: `PublishService` and `StorageService` on `TestApp::with_fake_shop()`; outcome = the mutations `FakeCommerce` recorded during the publish, its gallery and condition field afterwards, stored links, files on disk. "Brought-back item (3 + 2)" = the archived item of U2 after `bring_back`, with gallery `[m1, m2, m3]` and condition `[c1, c2]` on the fake shop.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H3·C10(a) | publish | the brought-back item (3 + 2), nothing changed | the publish completes (`done`); during it `FakeCommerce` recorded no staged upload, no upload, no media or file creation and no media deletion; the gallery is `[m1, m2, m3]` and the condition field `[c1, c2]` | M-H3; 10H §7.5 ("an untouched photo is not uploaded again") |
| H3·C10(b) | same | the brought-back item; photo **2**'s composition set to a tighter rect (so the session is no longer published-unchanged and the sidecar check looks at it); then the fake Capture One writes photo **1**'s first settings sidecar (`a.write_sidecar` / `a.fake.rewrite_settings`) and the sidecar check runs (`a.focus()`) | exactly one upload (photo 2's); photo 1's link is still `ready` with `external_id` `m1` and revision 0; the gallery is `[m1, n, m3]` | M-H3; MC-H4.7 (a first settings file is not an edit; projection r0 L1) |
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
- Hazard (H3·C10(b), projection r0 L1): the fake writes sidecars only when asked (`fake_capture_one.rs:390-410`); the row makes the sidecar appear **after** the bring back and before the publish. `tick` never runs the sidecar check (only `on_focus` / `on_startup` do, `previews.rs:786-795`), so the row calls `a.focus()`. While the session is published and unchanged, `PreviewScope::Live` leaves it out of the check (`store.rs:592`, A16), so the row first changes photo 2: only then does the first-settings clause (`previews.rs:810-814`) decide anything, and photo 1 must stay untouched.
- H3·C12(d): bring back does not activate the session (H4 does), so the RAW is written into `<capture root>/<folder key>/` and delivered with `watcher_event`; folder attribution files it.
- H3·C14: MC-H4.8 is already implemented by H1 (`mechanisms/retention.rs:45-57`, `services/storage.rs:734-735`), so its rows are acceptance only (production changes only if one fails, as for the unit).
- H3·C12(b) uses the photo's deny (P66); the undo window closes at the publish, which deletes that JPEG file: that is P66's rule for a denied photo and is expected.

### Unit test commands
`cargo test -p psm-testkit --test suite h3_publish_after_bring_back::` · `cargo test -p psm-testkit --test suite p14_publish_update::` · `cargo test -p psm-testkit --test suite p30_outputs_in_export_and_publish::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-testkit --test suite h3_jpeg_photos::` · `cargo test -p psm-testkit --test suite h3_bring_back::` · `cargo test -p psm-testkit --test suite h3_publish_after_bring_back::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::` · clippy on the touched crates, fmt. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: `Store::bring_back` is one transaction and is called only after every file exists; `bring_back` calls no adapter method that takes a `ConfirmedRun`; every exit path clears the "running" mark and, on failure, removes only the files this attempt wrote; the publish service was not changed (or the change is declared with its row); the evidence doc holds VERIFY-JPEG (probed or UNPROBED). Notes go to `docs/implementation/backlog_archive/H3_bring_back.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
Nothing to try on screen yet: the next milestone adds the workspace and the Bring back button. (For the orchestrator's live check after H4: VERIFY-JPEG in the Capture One evidence, and a bring back of one archived test product followed by a publish with one re-framed photo — Shopify must show one replaced picture and the others untouched. Also, from projection r0 L6 (rule 17): record the first bytes and the content type of one full-size download made through `ReqwestTransport` (`get_file` sends no `Accept` header, and a product made in the Shopify admin may hold PNG originals); a body that is not a JPEG fails the bring back by MC-H4.3.)

## Review log

- **2026-11-23, orchestrator: carried from H1's projection r0** (`handoffs/reviewer/handoff_H1_points_and_remote_record_projection_r0.md`). L3: `NotRemote` is added by H1 (U2 task 2 now adds only the two bring-back errors). L9: `FakeCommerce::fetch_original` / `original_fetches` and `set_image` moved here (U2 task 5). L10: `RemoteItems::new` gains `editing`, `capture_root`, `history` here (U2 task 5). Rows and counts unchanged.

- **2026-11-23, orchestrator: projection r0 amendments applied** (`handoffs/reviewer/handoff_H3_bring_back_projection_r0.md`, AMENDMENTS_REQUIRED, 0 owner cards; master plan v1.2). L1 H3·C10(b) rewritten so it has a production cause: photo 2's composition changed first (a published-unchanged session is left out of the sidecar check, `PreviewScope::Live`, A16), photo 1's first sidecar written, `a.focus()` (not `tick`) runs the check; exact outcome one upload, photo 1's link `ready` / `m1` / revision 0, gallery `[m1, n, m3]`; U3 Read first and the hazard note say why; master plan MC-H4.7 gains the A16 sentence. L2 H3·C2(c) completes its preview with `complete_render_with(<preview job>, SyntheticJpeg::rgb(1500, 1000, …))` after `a.fake.add_image(…)`, outcome frame 1500 × 1000 source `preview`; U1 Read first and a test-setup note added. L3 (10G-derivable, settled from 10G §4.2/§4.4/§6.4, the recommended option): U2 task 3 pins (a) brought-back main photo *k* placed by hand with `slot` = container `final_no = k` (null past the last container), (b) every retired row keeps a `photo_places` row with `released = 1`, (c) H3·C9(a) checks the marks, H3·C9(b)'s exact outcome is Condition 1 with Main `m1, m2, m3`, and row **H3·C9(c)** added (2 pictures, 3 containers: the arrival is Main 3), the row that fails without (b); master plan MC-H4.4(a)/(b) amended; a narrative point added. L4 U2 task 5: the production `capture_root` closure is `editing.inspect_document()` + canonicalize, uncached; H3·C7(e)'s input clears the fake's `open_documents`; master plan §4 Service says so. L5 U2 tasks 3–4: canonical `raw_path` and canonical `taken`; hazard note; master plan MC-H4.2, MC-H4.4(b). L6 MC-H4.3 and U2 task 4: a body not starting with `FF D8 FF` is a failed download; row **H3·C6(f)** added (PNG-signature bytes → `Err(BringBackFailed)`, 3 fetches, nothing imported); every bring-back fixture registers each picture with `set_image` (the default `fake_image_bytes` is refused); the live check after H4 records the first bytes and content type of one real full-size download (Manual try-out here and in H4, master plan §3 rule 17); a narrative point added. L7 H3·C8(b) exact: the ops during the call are `document_info_started`, `document_info`, `sync_folder:<capture root>/BEYO-12`. L8–L16 delegated (U2 Delegated choices, "Decided at projection r0"; the recommended option where one was given): L8 the import runs inside `retire_session`'s `act` (master plan MC-H4.5 amended); L9 `taken` fails closed without an endless suffix loop (no option recommended between the two: the smallest safe one); L10 a store failure after the downloads → same clean-up and `BringBack {Failed}`, error unchanged; L11 gone after the refresh → stored gone, `1 photo is no longer in the shop`, the refreshed URL gets the full tries (master plan MC-H4.3); L12 `BringBackInput` / `BroughtBack` in `domain::remote` (master plan §4 Vocabulary); L13 the condition role from `commerce.condition_photos_key()` (MC-H4.4(c)); L14 caches left; L15 `add_image` registration and the URL → media mapping incl. `fake_refreshed_url` (U2 task 5, U3 task 1); L16 `retouch_gen` read from the stored `raw_state` row (MC-H4.4(c)). Reality checks: line drift fixed where cited (`OriginalPreviewJob` :1573-1611, `arrival_insert` :2425-2501 with its job insert at :2496-2501; master plan MC-H4.4(b), MC-H4.6); test-setup hints added (C1(a) 1067, C1(b) EXIF writer/reader, C1(d) SQL, C6(e) `catch_unwind`, C7(c) sync, C12(d) `watcher_event`, C4(i) no `done: 0`, C4(j), C8(d)); C14 recorded as acceptance only (MC-H4.8 implemented by H1). Prompt note for the compiler: this track's naming and contract authority is `master_plan_archive.md`, not `master_plan.md`. No H1/H2 code needs a change (every amendment lands in H3's own files or in `FakeCommerce`, which H3 edits anyway); H4 carries the fixture note and the live-check line. Counts: U2 7 criteria / 36 rows; H3 14 / 64 (recounted from the four plan files: 12 units, 54 criteria, 253 rows). Outcomes otherwise unchanged, so no new projection round (`projection: done`).

- **2026-11-23, implementer r1 (Claude Opus, `PSM_IMPLEMENTER=claude`).** Built the three units in order with a
  checkpoint each (U1 `fb8d34f`, U2 `c9dad23`, U3 `747a2c4`). **U1:** `rawpreview::JpegSource` (decodes the JPEG; long
  edge ≤ 1600 → the file itself, else a Lanczos3 copy at 1600 px, quality 80; EXIF orientation via `kamadak-exif`;
  frame = pixel size, swapped for 5–8; a file that does not decode → `Ok(None)`); `OriginalPreviewJob` picks it for
  `.jpg`/`.jpeg` (any case); `SyntheticJpeg::with_orientation` (hand-built TIFF via `set_exif_metadata`); 10 rows.
  **VERIFY-JPEG probed: PASS** (Capture One was running with the owner's `Beyo Studio` session open): listed ~7 s
  after `sync_folder`, `find_image` answers for a `.jpg`, both recipes export it (preview 2560 × 1707, master
  3000 × 2000 = its pixel size), a `.cos` appears ~15 s after processing; recorded in the evidence doc. The probe JPEG
  and its two exports and settings file were left in that session (`Capture/_Unassigned/psm-probe-verify-jpeg-2026-11-23.jpg`,
  `Output/_session_manager/`), never deleted (H2). **U2:** `bring_back_names`, `estimate_bytes`, `DOWNLOAD_RETRIES`,
  `UNKNOWN_PICTURE_BYTES` (Core mechanisms, 5 unit tests); `BringBackInput`/`BroughtBack`/`BringBackProgress`/
  `BringBackState`/`BringBackOutcome`; errors `BringBackRunning`, `BringBackFailed {reason}` (codes
  `bring_back_running`, `bring_back_failed`); `AppEvent::BringBack` → `bring_back_progress` (`onBringBack`);
  `Store::bring_back` (one transaction, MC-H4.4 (a)–(e), templated marks of L3); `RemoteItems::bring_back`
  (`services/bring_back.rs`) with the running mark (`bring_back_progress` accessor for H4), downloads, the one URL
  refresh, clean-up, the import inside `History::retire_session`; `RemoteItems::new` in its registry form;
  `FakeCommerce::fetch_original`/`set_image`/`original_fetches`; 31 behaviour tests. **U3:** 18 behaviour tests on the
  real publish and storage services; **no production change was needed** (MC-H4.7 and MC-H4.8 held as written).
  Assumptions (smallest option, 10H §7 / MC-H4): (1) every error after the running mark is taken (also the
  precondition refusals "wait until…", "capture folder…", "no longer in the shop", and store errors) emits
  `BringBack {Failed, error}` with the progress so far; `NoIntegration`, `NotRemote`, `BringBackRunning` emit nothing;
  (2) a picture found gone at the refresh is also put on Health `remote_picture_gone`, as MC-H1.4's "stored as gone"
  does in H1; (3) the stale ids of the first pass go to `media_urls` once (chunks of 25) after that pass, then the
  refreshed ones get the full tries, a second `Gone` counting as a failed try; (4) a leftover file of an interrupted
  attempt is replaced by the rename, and a failed attempt removes it with its own files (no photo row names it,
  rule 2); (5) the JPEG-marker check is a private constant of `bring_back.rs`; (6) a store error inside `taken` is
  returned unchanged (not wrapped in `BringBackFailed`); (7) in a templated session only the brought-back **main**
  photos get a placement row (condition photos none, as the plan names); (8) the `Done` event carries done = total.
  Deviations: (a) `crates/testkit/tests/suite/h2_pages_and_search.rs` (H2's file) gained the three new
  `RemoteItems::new` arguments at its one direct call (a capture-root closure answering `DocumentNotOpen`; forced by
  the registry constructor, behaviour of that row unchanged); (b) the U3 checkpoint also touches `h3_bring_back.rs`
  (fixtures made `pub(crate)` for reuse); (c) H3·C4(d) uses an archived fixture without `p62`'s two hidden photos so
  exactly five earlier rows exist; (d) H3·C8(a) runs the four scenarios in one test and asserts no mutation **during**
  each bring back (the archived fixtures were published before, so the list is not literally empty there; the
  shop-only case asserts it empty). Observations for the reviewer: a link with a media id that is not `ready` is
  treated by Upload as "attached" and polled, so H3·C10(a) rests on the link's media id and H3·C10(b)/C11 on `ready`
  plus the stored generation/revision (checked by temporary, reverted probes: `upload_state 'none'` turns C10(b) red;
  dropping the `released` marks turns C9(a)/(c) red; a real settings edit after the first `.cos` gives 2 uploads in
  C10(b)). Candidate criterion: none.

