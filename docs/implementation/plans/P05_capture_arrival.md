---
phase: P05_capture_arrival
title: Capture arrival — hook dispatch, candidate filter, stability, attribution, original preview, import
difficulty: high
projection: no
depends_on: [P04_capture_one_adapter]
---
# P05 — Capture arrival: hook dispatch, candidate filter, stability, attribution, original preview, import

## Goal
Turn hook payloads, watcher events and the startup scan into exactly one photo row per accepted RAW in the right
session (MC-2, M1/M9), extract the camera's embedded preview byte-exactly (MC-20, M27), record `captured_at` from
the file (MC-4), enqueue the follow-up jobs (hash, original preview, preview render), and add file import and
"assign Unassigned to an item". **Not in this phase:** handling `processing_done` and `variant_adjusted` beyond
routing them to a handler that P06 fills; any UI (P08 shows arrivals).

## Read first
master plan §6.5 (`photos`, `raw_state`, `capture_events`, `jobs`), §6.6, §6.7 (P05 commands), §7.5 · book `01`,
`14`, `06` (Arrival, Unassigned, Import), `04` S-C, S-E · contracts `05A` MC-1 (producers), MC-2 (all sections), MC-3
(after stability, once per path), MC-4 (`captured_at`); `10A` MC-15 (payload table, consumption), MC-20 · evidence
`capture_one_16.8.6.md` VERIFY-1 (argv shapes; capture-done unprobed), "Other runtime observations" (`.capture`
hidden location, embedded preview sizes, EXIF offset).

## Files expected to change
`crates/core/src/mechanisms/hook_payload.rs` (new), `crates/core/src/services/{arrival.rs (new), editing_events.rs
(new: dispatcher + `EditingEvents` handler trait with P05 capture handling and P06 stubs), photos.rs (new: import,
assign_unassigned)}`, `crates/core/src/ports/raw_previews.rs` (new), `crates/infra/src/rawpreview.rs` (new, MC-20),
`crates/infra/src/{spool.rs (dispatch after row), watcher.rs (capture-root wiring)}`, `crates/core/src/domain/photo.rs`,
Store trait + impl (insert-or-get in one tx with jobs and raw_state), `apps/desktop/src-tauri/src/commands/photos.rs`
(`photos_import`, `photos_assign_unassigned`), bindings, `crates/testkit/src/{fixtures/synthetic_raw.rs (new),
test_app.rs (`watcher_event`, `scan()`, `tick`, capture root in the fake document)}`, `crates/testkit/tests/p05_capture_arrival.rs` (new).

## Core mechanisms (unit tests allowed)
- `mechanisms::hook_payload::parse(json) -> Result<HookEvent>` (MC-15 payload table; `capture_done` yields every
  string in `argv` plus `last_captured_file` as candidates).
- `mechanisms::arrival::is_candidate(path, capture_root, raw_extensions) -> bool` (MC-2 candidate filter).
- `mechanisms::arrival::attribute(parent_dir, capture_root, sessions_by_folder_key, active_session) -> SessionId` (MC-2 attribution rules 1–3).
- `psm_infra::rawpreview::largest_embedded_jpeg(bytes) -> Option<(offset, len, width, height)>` and `orientation(bytes) -> Option<u8>` (MC-20).

## Ordered tasks
1. `parse` + `HookEvent` enum; `EditingEventsDispatcher` called by `Spool::drain` after the `capture_events` row commits:
   `CaptureDone` → `ArrivalService::on_hook_paths(candidates)`; `ProcessingDone`/`VariantAdjusted` → `EditingEvents`
   handler (P06; P05 ships a logging stub and names it in the handoff).
2. `ArrivalService`: candidate filter (Sony adapter declares `["arw"]`), stability for watcher/scan candidates
   (two `stat`s ≥ 1 s apart equal; re-check each 1 s; give up at 120 s with an error `capture_events` row), hook paths
   skip stability; attribution by parent folder; one transaction: insert-or-get photo (`unreviewed`, `position null`,
   `uid`, `captured_at = mtime ms`, `source`), `raw_state` upsert (`path_key`), enqueue `Hash`, `OriginalPreview`,
   `RenderPreview`; emit `photo_added`. Dedupe is the transaction only (hook, watcher, scan may race).
3. Watcher wired to the capture root (root + one level) at startup; startup scan of the root and every immediate
   subfolder (R-I16).
4. `rawpreview`: TIFF walk (IFD0 chain + SubIFDs), `0x0201/0x0202` candidates, `FFD8…FFD9` validity, largest pixel
   area from SOF, byte copy temp + rename to `previews/…/original.jpg`, orientation `0x0112` on the photo rows of that RAW.
   `OriginalPreview` job handler; `Hash` job handler (`sha256_file` → `raw_hash` on every row of the RAW).
5. `PhotosService::import(paths)` → same path as arrival with `source = import`, attribution rule 3 (active session or
   Unassigned), then `EditingProvider::import(paths)`; `assign_unassigned(item_id)` → moves every Unassigned photo to
   the item's session (creating it if none) via `ReviewService::move_photos`.
6. `SyntheticRaw::build(streams: &[(w, h, valid_trailer: bool)], orientation, exif_datetime)` in testkit (minimal TIFF
   writer; JPEG streams are real minimal JPEGs with SOF0 headers). Tests.

## Criteria (8 criteria, 33 rows; 1 `[rule]` row: C8(d))
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `watcher_event` | `<root>/K1/.hidden.ARW` | zero `photos`, zero `capture_events` rows | M9 (filter) |
| C1(b) | `watcher_event` | `<root>/K1/CaptureOne/x.ARW` | none | M9 |
| C1(c) | `watcher_event` | `<root>/.capture/x.ARW` | none | M9 (`.capture` evidence) |
| C1(d) | `watcher_event` | `<root>/K1/x.jpg` | none | M9 |
| C1(e) | `watcher_event` | `<root>/K1/sub/x.ARW` (two levels) | none | M9 |
| C1(f) | `watcher_event` | `<root>/K1/x.arw` (lower-case extension) | one photo row | M9 |
| C1(g) | `watcher_event` | a directory named `<root>/K1/x.ARW` | none | M9 (regular file) |
| C2(a) | `watcher_event` + `tick` | file grows between the two stats; stable from the third | no row after the second stat; one row after the next re-check (`tick(1000)`) | MC-2 (stability) |
| C2(b) | `watcher_event` + `tick` ×121 | file size changes every second | after 120 s one `capture_events` row with `error` set, zero photos | MC-2 (give up) |
| C3(a) | arrival | `<root>/_Unassigned/x.ARW` | row in session 1 | M9 (rule 1) |
| C3(b) | arrival | `<root>/K1/x.ARW`, session S1 has `folder_key = K1`, S1 `published`, active = S2 | row in S1; S1 `dirty = true` | M9 (rule 2) |
| C3(c) | arrival | `<root>/K1/x.ARW`, active S2 | row in S1 (folder wins over active) | M9 (rule 2) |
| C3(d) | arrival | `<root>/Other/x.ARW`, active S2 | row in S2 | M9 (rule 3) |
| C3(e) | arrival | `<root>/Other/x.ARW`, no active session | row in session 1 | M9 (rule 3) |
| C3(f) | arrival | `<root>/x.ARW` (capture root itself), active S2 | row in S2 | M9 (rule 3) |
| C4(a) | hook `capture_done` then `watcher_event` then `scan` | same RAW | exactly one photo row | M1, M9 |
| C4(b) | `watcher_event` then hook | same RAW | one row | M1 |
| C4(c) | hook, watcher and scan delivered concurrently from three threads | same RAW | one row | M9 (transaction is the only dedupe) |
| C4(d) | after C4(a) | — | three `capture_events` rows (`source` hook, watcher, scan) | M1 (every delivery recorded) |
| C5(a) | arrival | any accepted RAW | `classification = unreviewed`, `position = NULL`, `source = capture`, `uid` is a UUID v4, `raw_state` row exists with `path_key` = first 16 hex of SHA-256 of `canon(path)` | MC-2 (row creation), 10A storage key |
| C5(b) | arrival | synthetic RAW with EXIF DateTimeOriginal 2 h ahead; file mtime `m` | `captured_at == m` (ms), not the EXIF time | M11 (camera clock never used) |
| C5(c) | arrival | any accepted RAW | `jobs` has `original_preview`, `hash`, `render_preview` rows targeting the RAW (`queued`) | MC-2 (enqueue) |
| C6(a) | `OriginalPreview` job | synthetic RAW: IFD0 stream 1088×1080, IFD1 160×120 | `previews/<pk[0..2]>/<pk>/original.jpg` bytes == the IFD0 stream; `original_preview_path` set | M27 |
| C6(b) | job | largest stream lacks the `FFD9` trailer; second-largest valid | second-largest copied | MC-20 (validity) |
| C6(c) | job | orientation tag `6` | `photos.orientation == 6` | MC-20 |
| C6(d) | job | — | RAW bytes and mtime unchanged after extraction | M27, H2 |
| C6(e) | job | RAW without any JPEG stream | no file; `original_preview_path` null; job `done` | MC-20 (no candidate) |
| C7(a) | `App::open` with files pre-existing | `<root>/a.ARW`, `<root>/K1/b.ARW`, `<root>/_Unassigned/c.ARW` | three rows: a → active/Unassigned per rule 3, b → S1, c → session 1 | MC-2 (startup scan, R-I16) |
| C7(b) | `App::open` | `<root>/K1/CaptureOne/x.ARW` and an already-known RAW | no new rows for either | MC-2 |
| C8(a) | `PhotosService::import([p])`, active S2 | `p` outside the capture root | row in S2 with `source = import`; fake received `import([p])` | MC-2 (import path) |
| C8(b) | `import([p])` | no active session | row in session 1 | MC-2 |
| C8(c) | `import([p])` twice | same path, same session | one row | M9 |
| C8(d) | `photos_assign_unassigned(item)` | Unassigned holds u1, u2; item has no session | item's session created; u1, u2 moved there (appended, source order); Unassigned empty | `[rule]` ch06 §Unassigned ("Assign product"), executed through MC-2 move |

## Delegated choices
Stability bookkeeping structure (in-memory map polled by `tick`), the Sony adapter's declared extension set
(`["arw"]` only in the MVP), whether `Hash` runs before or after `OriginalPreview` (both after stability), the
`photo_added` payload beyond `{photo_id, session_id}`, the import file picker (UI in P08).

## Manual try-out
1. With the app and Capture One running and a session activated (P08 brings the UI; until then, set
   `active_session_id` by activating via the Health page's "Activate session" dev control if the implementer adds
   one — optional), press the shutter.
2. `sqlite3 … 'select session_id, raw_path, classification, captured_at from photos;'` shows one row per frame;
   `select source, hook from capture_events;` shows a `hook` row (capture_done) and, after Capture One wrote the
   file, a `watcher` row for the same RAW — and still one photo.
3. Look in `previews/…/original.jpg`: the camera's embedded JPEG.

## Notes
- `capture_done` argv is **unprobed** (evidence VERIFY-1). The parser treats every `argv` string and
  `last_captured_file` as a candidate; any that passes the filter is processed, so both shapes work. P00's spike
  checklist (g) records the real shape when the camera releases; no plan change is needed either way.
- Processing-done and variant-adjusted payloads are parsed here (grounded shapes) and handled in P06.
- Estimated size: ~12 files, ~1 800 lines (the TIFF walker and the synthetic RAW builder are the largest).

## Review log
(empty)
