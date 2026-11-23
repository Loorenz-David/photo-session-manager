---
phase: H4_remote_workspace
title: The normal workspace for archived and shop-only products — opening one shows the usual stage, Main and Condition with its Shopify pictures (arriving in the background) and a line saying the photos are on Shopify; opening it never moves the camera's folder; Details can be edited and published without touching the pictures; the first photo edit asks "Bring back N photos from Shopify to edit them?" and, once they are back, the product is an ordinary session
difficulty: high
projection: yes
depends_on: [H3_bring_back]
blocked_on: none
---
# H4 — The normal workspace for archived and shop-only products

**Tracker:** NOT_STARTED

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `10H` §6 (and the prompt and progress of §7) by contract **MC-H3** (`docs/implementation/master_plan_archive.md` §5), with M-H1 and M-H4. (U1) Backend: the remote view with background pictures, activation that leaves the capture folder alone, what a remote session hands the screen, the three commands. (U2) The details-only publish and the rows of remote items. (U3) The workspace UI: the pictures in the standard workspace, the note, the edit gate, the Bring back dialog and progress, the capture target, Refresh from Shopify, the publish confirmation. **Not here:** any change to bring back itself (H3) or to the list pictures (H2); pages in other lists; getting the RAW back.

## How to work this plan (flow.md §10, master_plan_archive.md §3)
One runner phase of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `H4·C<n>(<row>)`; tests named after their row (`h4_c1a_<slug>`). Open details: the smallest option consistent with 10H §6–§7 and MC-H3, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_archive/H4_remote_workspace.md`. **Never call Shopify or its CDN, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Backend: remote view, activation, what the screen is given, commands | 4 | 21 | **yes** |
| U2 | Details-only publish; the rows of remote items | 5 | 17 | **yes** |
| U3 | The workspace UI | 5 | 28 | **yes** |
| **Total** | | **14** | **66** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `docs/implementation/master_plan_archive.md` §3, §4 (Vocabulary, Commands, Words, Events, Frontend), **§5 MC-H3 (every item), MC-H1.3–4, MC-H5.3–4, MC-H4.1, .5** · book `10H` §1, §3, §6, §7 (prompt, progress), §9 · book `10F` (Rules as amended, Q2) · book `04` S-B and S-E (what opening and leaving a session do to the capture folder) · `docs/implementation/master_plan_conversations.md` §5 **MC-C3** (the visit: `captureTarget`) · `docs/guide/03_rules_and_principles.md` (H1, H4, H6, H8; CR4) · `docs/guide/04_change_impact.md` rows "A Tauri command signature", "An error code" · the H1–H3 plans (what exists).

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Opening an archived or shop-only product never moves the camera's folder;** new photos keep going where they went. After a bring back the folder follows it like any session (unless you opened it as a visit from a conversation).
- **If Shopify cannot be read when an adopted shop product is first opened,** the session behaves as before this change and says so, with Try again.
- **Publishing a remote product sends its details only;** the confirmation says "Details only: the photos stay as they are on Shopify."
- **Both kinds carry the "Archived in Shopify" label,** except while their publish is running or failed.
- **"Bring back now"** in the note line starts the same question without an edit (the way to edit in Capture One).
- **Selecting and looking at pictures, the Details form, the writer and conversations need no bring back.**
- **Refresh from Shopify** (the session's ⋯ menu) is offered for remote products only.

## Unit 1 — Backend: remote view, activation, what the screen is given, commands

**Unit id:** `H4·U1` · **Rows:** H4·C1–C4 · **Changes app files:** yes

### Goal
`RemoteItems::view` with its background downloads, `SessionsService::activate` for remote sessions, `SessionsService::view` and the attachable photos under M-H4, the media-list guard, and the commands `session_remote_get`, `session_remote_refresh`, `session_bring_back`.

### Read first
Master plan §5 **MC-H3.2–5, .8**, **MC-H1.3–4** · `crates/core/src/services/retention.rs` (`ArchiveService` :160-249: what it did, `cached_path` :180) · `crates/core/src/services/remote.rs` and `bring_back.rs` (H1–H3) · `crates/core/src/services/sessions.rs` (`activate` :201-269, `deactivate` :271, `view` :348-401) · `crates/core/src/services/arrival.rs` (:308-318) and `photos.rs` (:27-30): what arrives while a cleared session is active · `crates/core/src/services/attributes.rs` (:791-803) · `crates/core/src/services/conversations.rs` (`link_preview` :104, `attachable` :1579) · `apps/desktop/src-tauri/src/commands/{items.rs (:343-430: the archive DTOs and commands, error codes :140-160, `files_cleared` at :151), sessions.rs (`session_get` :234, `session_activate` :472)}` and `src-tauri/src/lib.rs` (`specta_builder()`, the inline command tests `c3_tests`) · `crates/testkit/tests/suite/{p62_retention_after_publish.rs (`c1e_…` :464, `c3a_…` :604, `c3b_…` :680), a16_switch_cost_and_filing_target.rs (how activation is observed on the fake Capture One, :60-260), c4_conversations_visit.rs (C4·C8: a read never moves the active session)}`.

### Files
New: `apps/desktop/src-tauri/src/commands/remote.rs`, `crates/testkit/tests/suite/h4_remote_workspace.rs`. Edited: `crates/core/src/{domain/remote.rs, services/remote.rs, services/retention.rs, services/sessions.rs, services/attributes.rs, services/conversations.rs}`, `crates/integrations/src/lib.rs`, `apps/desktop/src-tauri/src/{lib.rs (`specta_builder()`, `h4_tests`), commands/{mod.rs, items.rs, sessions.rs}}`, `apps/desktop/src/services/{bindings.ts (generated), remote.ts (new wrapper file), retention.ts}`, `crates/testkit/tests/suite/{main.rs, p62_retention_after_publish.rs}`, `docs/guide/{02_architecture.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
None (the pure rules exist since H1–H3).

### Ordered tasks
1. `RemoteView`, `RemotePictureView` (master plan §4). `RemoteItems::view(session_id) -> Option<RemoteView>` by MC-H3.5: `None` for a session without an item; `Local` → no pictures; `Unknown` → no pictures and `queue_record(session_id)`; `Remote(_)` → the record's pictures with their cache paths, missing files queued with source `View(session_id)` (a new `View` request for the same session replaces that session's not-started downloads; other sessions' stay). `check_failed` = the last first-open read of this item failed (kept in memory, cleared by a successful read); `bring_back` = the running mark's progress (H3).
2. `run_pending`: view pictures join the download batch (`fetch_image(url, ARCHIVED_IMAGE_EDGE)` → `.part` → rename → `PictureReady {Remote {session_id, role, position}, path}`), under the same limit and the stale rule of MC-H1.4 (the item's stale ids, chunks of 25, budget wait, one retry, gone → Health). A failed queued first-open read sets `check_failed` and emits `SessionChanged`.
3. `ArchiveService::view` and its `download` are removed; `set_keep_files` moves to `RemoteItems`. `item_archive_get` stays in this unit, answering from `RemoteItems::view` (cached paths only), so the frontend still compiles; U3 removes it with its last caller. `item_keep_files_set` answers the `RemoteViewDto`.
4. Activation (MC-H3.4): `SessionsService::activate` first reads the session's state (`with_remote`; without one, as today); `Remote(_)` → `Ok(())` before any editing, camera or store call. `session_activate` attaches `app.remote`.
5. M-H4 for a remote session's other readers (MC-H3.5): `SessionsService::view` answers no photos (and no compositions or upload states) when the state is `Remote(_)`; `link_preview` answers a path only when its file exists.
6. Guard (MC-H3.8): the media-list refusal at `attributes.rs:793-803` uses the remote state (`Remote(_)`) instead of `cleared_at` alone.
7. Commands (`commands/remote.rs`, thin, `spawn_blocking`): `session_remote_get(session_id) -> Option<RemoteViewDto>`, `session_remote_refresh(session_id) -> RemoteViewDto` (`RemoteItems::refresh`, then `view`), `session_bring_back(session_id) -> BringBackOutcomeDto` (`RemoteItems::bring_back`). `RemoteViewDto {item_id, state: 'local' | 'unknown' | 'archived' | 'shop_only', keep_files, pictures: [{role, position, media_id, path, gone}], bring_back_bytes, check_failed, bring_back: {done, total} | null}`. Wrapper `apps/desktop/src/services/remote.ts` (`getRemote`, `refreshRemote`, `bringBack`).
8. Existing tests that change, and only so: `p62_…::c3a_…` asserts the pictures' files after `run_pending` (the view no longer downloads) and reads them through `RemoteItems::view`; nothing else in `p62` changes in this unit.
9. Guide: `02_architecture.md` (§Remote items: the view, the three commands, activation; the P62 archived view paragraph is rewritten), `04_change_impact.md` (rows: "a new reader that hands a photo's preview path to the screen → the existing-file rule or the remote view, M-H4"; "anything that activates a session → `SessionsService::activate`'s remote rule and `useCaptureTarget`'s state gate").
10. Tests for the rows.

### Criteria (4 criteria, 21 rows)
Boundary: `RemoteItems::view` / `run_pending`, `SessionsService::{activate, view}`, `AttributesService`, `ConversationsService::attachable` on `TestApp::with_fake_shop()` (with `with_pricing()` for the attachable rows); the command handlers for H4·C4. Outcome = returned values, files, the operations the fake Capture One received, `active_session_id`, what `FakeCommerce` recorded.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H4·C1(a) | `RemoteItems::view` | an archived item (3 + 2) whose sizes are 1 000 000 each, nothing cached | state `Remote(Archived)`; five pictures (gallery 1–3, condition 1–2) each with `path` none and `gone` false; `bring_back_bytes` 5 000 000; no fetch was recorded when it returned | MC-H3.5; 10H §6 ("opening never waits") |
| H4·C1(b) | `run_pending` after (a) | fetch delay 50 ms, 10 pictures in a second item viewed too | every picture's file exists at `<previews>/archived/<item_id>/<role>-<NN>-<12 hex>.jpg`; each fetch was `(url, 1600)`; one `PictureReady {Remote {session, role, position}, path}` per picture; `max_fetches_in_flight()` is 4 | MC-H3.5; 10H §6 (1600 px, 4 at a time, each as it arrives) |
| H4·C1(c) | `view` again | after (b) | every picture carries its file's path; a further `run_pending` records no fetch | MC-H3.5 (cached) |
| H4·C1(d) | `view` | a `Local` session; the Unassigned session | state `Local` with no pictures and the item's `keep_files`; `None` | MC-H3.5 |
| H4·C1(e) | `view`, `run_pending`, `view` | an adopted item never opened, its product has 2 gallery media | first view: state `Unknown`, no pictures, `check_failed` false; after the run (one `media_readback` read, a `SessionChanged`): state `Remote(ShopOnly)` with two pictures | MC-H3.2, .5; 10H §6 (first open) |
| H4·C1(f) | same | as (e) with the shop unreachable | after the run: state `Unknown`, `check_failed` true, a `SessionChanged` was emitted; when the shop answers again, the next `view` + run gives `Remote(ShopOnly)` and `check_failed` false | MC-H3.2 |
| H4·C1(g) | `view` + `run_pending` | an archived item with 30 gallery pictures whose URLs all answer `Gone` once, then serve at the URLs `media_urls` returns | exactly two `media_urls` calls (25 and 5 ids); 30 files written | MC-H1.4; MC-H5.5; 10H §6 ("when a picture's URL fails") |
| H4·C1(h) | `FakeCommerce::mutation_ops()` | after (a)–(g) | empty | M-H1 |
| H4·C2(a) | `SessionsService::activate` | session B (`Local`) is active; `activate(A)` for an archived A | `Ok`; the fake Capture One received no operation after the call began; `active_session_id` is still B | MC-H3.4; 10H §6 ("never moves the camera's capture folder") |
| H4·C2(b) | same | no session active; `activate(S)` for a `Remote(ShopOnly)` S | `Ok`; no fake Capture One operation; no active session | MC-H3.4 |
| H4·C2(c) | same | camera-independent mode (`TestApp::with_camera()`); B is the filing target; `activate(A)` archived | the filing target is still B (no target change, no camera call) | MC-H3.4 |
| H4·C2(d) | same | `activate(L)` for a `Local` L; `activate(U)` for an `Unknown` adopted item | both as today: `set_capture_folder` and `set_current_collection` for `<root>/<folder key>`, the session active | MC-H3.4 ("every other state activates as today") |
| H4·C2(e) | same, after bring back | the archived A of (a) brought back (H3), then `activate(A)` | `set_capture_folder` and `set_current_collection` for `<root>/<A's folder key>`; A is active | MC-H3.4; 10H §7.4 |
| H4·C2(f) | arrival | B active, `activate(A)` archived as in (a), then a capture arrives (as `p05` delivers one) | its photo row belongs to B | 10H §6 ("new photos keep going where they went"); M-H2 (nothing joins a remote item) |
| H4·C3(a) | `SessionsService::view` | an archived session (its five cleared photo rows still name deleted previews) | `photos` is empty | M-H4; MC-H3.5 |
| H4·C3(b) | same | that session after its bring back | five photos (the brought-back ones) | MC-H3.5 |
| H4·C3(c) | `ConversationsService::attachable` | a conversation of an archived item (published links `ready`, preview files deleted) | the photos are listed with their media ids and `preview_path` none | M-H4; MC-H3.5 |
| H4·C3(d) | `AttributesService` write | a media-list attribute on a `Remote(ShopOnly)` item; a text attribute on the same item and on an archived item | `Err(FilesCleared)`; saved; saved | MC-H3.8; 10H §6 ("every attribute but media lists") |
| H4·C4(a) | command `session_remote_get` (desktop test `h4_tests`) | an archived session | `state: "archived"`, five pictures, `bring_back_bytes`, `check_failed: false`, `bring_back: null`; `null` for the Unassigned session | MC-H3.5 |
| H4·C4(b) | command `session_bring_back` | a `Local` session | the error code `not_remote` | MC-H4.1 |
| H4·C4(c) | command `session_remote_refresh` | an archived session whose shop gallery was reordered | the answer's gallery pictures are in the new order | MC-H3.3 |

### Delegated choices
Whether `check_failed` is a map or a field of the queue · whether `item_archive_get` is kept as a thin alias until U3 or removed at once together with a temporary frontend shim (the unit must end with `npm run typecheck` green).

### Notes
- Hazard (the capture folder, CR4's cousin): `activate` must return before the first `log.step` / `set_target` (`sessions.rs:209-212`), not after `ensure` — `ensure` can launch Capture One.
- Hazard: `view` is called on every session change the workspace reloads on; it must stay a read plus a queue push. No fetch, no GraphQL.
- Hazard: an `Unknown` item is **activated as today** by the backend (H4·C2(d)); the gate for it is the frontend's (U3, MC-H3.4). Do not make `activate` wait for a read.
- Hazard: `SessionsService::view` is also what the conversations visit and the form read; for a remote session they get no photos, which the form already handles for a session without photos.
- H4·C2(f) is `p62`'s `c1e` for a session that was never activated: both must hold.
- **Carried from H3's projection r0 (fixtures that bring back: H4·C2(e), H4·C3(b), H4·C9(g)):** register every picture with `FakeCommerce::set_image` as a real `SyntheticJpeg` (the default `fake_image_bytes` does not start with `FF D8 FF`, and MC-H4.3 refuses such a body, so the bring back would fail), and register each brought-back file with the fake Capture One (`add_image`) before its renders are asked for. H3's `brought_back` helper in `h3_publish_after_bring_back.rs` does both. The capture-root source `RemoteItems` uses is uncached `inspect_document` (MC-H4, H3 U2 task 5), so a fixture with no open document cannot bring back.

### Unit test commands
`cargo test -p psm-testkit --test suite h4_remote_workspace::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::` · `cargo test -p psm-testkit --test suite a16_switch_cost_and_filing_target::` · `cargo test -p psm-testkit --test suite c3_conversations_answer::` · `cargo test -p psm-testkit --test suite c4_conversations_visit::` · `cargo test -p psm-desktop` · `cd apps/desktop && npm run typecheck` · `cargo clippy -p psm-core -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — Details-only publish; the rows of remote items

**Unit id:** `H4·U2` · **Rows:** H4·C5–C9 · **Changes app files:** yes

### Goal
Migration 021, `details_only` from preview to run (MC-H3.6), retention's exclusion at service level, and the counts, status and label of remote rows (MC-H3.7).

### Read first
Master plan §5 **MC-H3.6, .7, .8** · `crates/core/src/services/publish.rs` (`split` :593, `preview` :661-746, `confirm` :748-910 with the P62 refusal at :755-760, `run_item` :947-1080, `prepare` :1146-1279, `upload` :1446-1500, `update` :1720-1900: the gallery block :1733-1804 and the condition-photos value at :1820-1842) · `crates/core/src/domain/publish.rs` (`PublishPreviewItem` :144, `PublishItem` :190) · `crates/infra/src/sqlite/store.rs` (how `publish_items` rows are written and read: `publish_item`, the insert in the confirming transaction) · `crates/infra/src/sqlite/ensure.rs` (the research columns of `publish_items`: do not recreate the table) · `crates/core/src/mechanisms/status.rs` (`derive_status` :17, `row_label` :62) · `crates/core/src/services/sessions.rs` (`list` :408-507: the label at :448-456, `status_input` :510) · `apps/desktop/src-tauri/src/commands/publish.rs` (the preview DTO) · `crates/testkit/tests/suite/{p62_retention_after_publish.rs (`c3b_…` :680-742), p13_publish_runs.rs, p14_publish_update.rs}` (how a publish is confirmed and run in tests; `confirm_as_shown`) · `crates/testkit/tests/suite/p27_retire_shape_on_arrival.rs` :346-372.

### Files
New: `crates/infra/src/sqlite/migrations/021_publish_details_only.sql`, `crates/testkit/tests/suite/h4_details_only_publish.rs`. Edited: `crates/core/src/{domain/publish.rs, services/publish.rs, services/sessions.rs, services/storage.rs}`, `crates/infra/src/sqlite/store.rs`, `crates/integrations/src/lib.rs` (the publish service gets `RemoteItems`), `apps/desktop/src-tauri/src/commands/publish.rs`, `apps/desktop/src/services/bindings.ts` (generated), `crates/testkit/tests/suite/{main.rs, p27_retire_shape_on_arrival.rs, p62_retention_after_publish.rs}`, `docs/guide/{02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. Migration `021_publish_details_only.sql` with its `M::up` line and its `DROP COLUMN` line in the rewind test (`p27_…`). `PublishItem.details_only`, `PublishPreviewItem.details_only` and the preview DTO field; the store reads and writes the column.
2. `preview` (MC-H3.6): for a `Remote(_)` session: `main`, `condition`, both thumbnail lists and `announced_removals` empty, `unreviewed` 0, `create_item` false, `details_only` true, **no** `item_get` call; everything else (details counts, suggestions, conflicts) as today.
3. `confirm`: remove the cleared refusal (`publish.rs:755-760`); a session previewed as `details_only` is frozen with empty photo lists and `details_only = true` in the confirming transaction.
4. `run_item` for a `details_only` item: `prepare` returns "ready" without `ensure_export` and without touching links; `upload` has no work (total 0); `update` skips the gallery block and leaves the condition-photos key out of the writes (also when `run_attributes()` holds it); the rest of `update` and the search-text step are unchanged. The item ends `done` and `sessions.published_at` is set as for any publish.
5. Rows of remote items (MC-H3.7) in `SessionsService::list` (and so `page`): counts from the record, `unreviewed_count` 0, the status input's missing-preview count 0; the label rule. The storage scan's `shop_only` input (H1) is already filled; H4·C8 is its service row.
6. Existing tests that change, and only so: `p62_…::c3b_…` — its publish part now expects the confirmation to be **accepted** as details-only (it was `files_cleared`); its photo-operation refusals are unchanged.
7. Guide: `02_architecture.md` (§Publish: the details-only run), `03_rules_and_principles.md` (the M-H table of H3 gains "a details-only run never touches media: `publish_items.details_only`"), `04_change_impact.md` (a row "a new media-touching publish step → skip it for `details_only`").
8. Tests for the rows.

### Criteria (5 criteria, 17 rows)
Boundary: `PublishService` (`preview`, `confirm`, the run to its end), `SessionsService::list` and `StorageService` on `TestApp::with_fake_shop()`; outcome = the preview, the persisted publish item, what `FakeCommerce` recorded during the run, its gallery and condition field afterwards, list rows.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H4·C5(a) | `preview` | an archived session (3 + 2) | one item: `main` and `condition` empty, no thumbnails, `announced_removals` empty, `details_only` true, `create_item` false; `FakeCommerce` recorded no `item_get` read | MC-H3.6; M-H4 |
| H4·C5(b) | same | a `Local` published session | `details_only` false; `main` holds its photos (as today) | MC-H3.6 |
| H4·C5(c) | same | a `Remote(ShopOnly)` session | as (a) | MC-H3.6 |
| H4·C6(a) | confirm + run | an archived item (3 + 2); its title changed and one text attribute edited | the publish item ends `done`; the shop's title and that attribute hold the new values; during the run `FakeCommerce::mutation_ops()` holds no staged upload, upload, media creation, file creation, media deletion or reorder; the gallery is `[m1, m2, m3]` and the condition field `[c1, c2]` | MC-H3.6 and its invariant; 10H §6 ("Publish of details only") |
| H4·C6(b) | the store | (a), read after `kill_and_restart` | the publish item has `details_only` true and empty frozen photo lists | MC-H3.6 ("decided at confirmation"); H4 |
| H4·C6(c) | confirm + run across a restart | as (a), killed after the attribute write (`fail_after_effect("attributes_write")`), restarted, the run resumed | `done`; the gallery and the condition field are unchanged; no media mutation in the whole history | H4; MC-H3.6 |
| H4·C6(d) | confirm + run | a `Remote(ShopOnly)` item with an edited text attribute | `done`; the attribute is written; no media mutation; `published_at` is set; the state is still `Remote(ShopOnly)` | MC-H3.6; 10H §6 |
| H4·C6(e) | the store | after (a) | `cleared_at` unchanged and the state still `Remote(Archived)` | MC-H3.6 |
| H4·C7(a) | confirm + run | an archived item whose stored attributes hold the condition-photos key (as its earlier publish left it) and an edited text attribute | the values written to the shop do not include the condition-photos key; the shop's condition field is `[c1, c2]` | MC-H3.6 ("and the condition-photos value") |
| H4·C8(a) | `StorageService` scan | the item of H4·C6(d) after its publish, the clock 30 days later | not cleared: `cleared_at` null, its record's pictures intact, no `media_readback` read by the scan | MC-H3.8 |
| H4·C9(a) | `list` | a `Remote(ShopOnly)` item with a record of 3 gallery and 1 condition pictures, never published here | its row: `main_count` 3, `condition_count` 1, `unreviewed_count` 0, status `ready`, label `archived` | MC-H3.7; 10H §6 (the label stays on the row) |
| H4·C9(b) | same | an archived item, clean | status `published`, label `archived`, counts 3 and 2 | MC-H3.7 |
| H4·C9(c) | same | an archived item with an edited attribute (dirty) | status `ready`, label `archived` | MC-H3.7 |
| H4·C9(d) | same | an archived item while its details-only publish is in step `update` | label `publishing` with the step's percent (75) | MC-H3.7 |
| H4·C9(e) | same | an archived item whose details-only publish failed in step `update` | label `publish_failed` | MC-H3.7 |
| H4·C9(f) | same | an archived item whose details-only publish was cancelled | label `publish_cancelled` | MC-H3.7 |
| H4·C9(g) | same | an archived item after its bring back (H3), nothing changed | status `published`, label `published`, counts 3 and 2 (its own photos) | MC-H3.7; MC-H4.4(e) |

### Delegated choices
Where in `run_item` the `details_only` branches sit (one early branch per step is the smallest) · the progress text of a details-only run (the existing `Updating the product`).

### Notes
- Hazard (the reason for the flag): without it a publish of an archived item would see its cleared photo rows as "main", stage an export of files that are gone, and — if the photo lists were simply empty — `update` would reconcile the gallery against **nothing** and delete every picture (they are all "own"). The condition field has the same trap: with no frozen condition photo and the key among the attributes, `publish.rs:1820-1842` would write an empty list over the damage report photos. `details_only` must skip the gallery block entirely, not run it with empty lists.
- Hazard: `details_only` is read from the persisted item on every resume (H4·C6(b), (c)); never recompute it from the item's state (a bring back between a failure and a retry would turn a details-only item into a media publish with empty frozen lists).
- Hazard: `publish_items` carries research columns added by `ensure.rs`; add the column with `ALTER TABLE`, never by recreating the table.
- Hazard (MC-H3.7): the label rule replaces `sessions.rs:448-450` ("cleared → Archived" unconditionally); a running or failed publish must show through.
- The publish popover's words are U3's.
- **Carried from H1's projection r0 (for H4's projection):** retention's exclusion is keyed on `Remote(ShopOnly)`. An adopted, photo-less item that is still `Unknown` (never read) is not excluded. Today such a session can never be Ready (`status.rs:33`), so this matters only if the details-only publish can complete for an `Unknown` item. MC-H3.6 says an `Unknown` item "is confirmed as today", so H4's projection confirms that path cannot reach "published and clean" with zero photos (else H4·C8 needs an `Unknown` row).

### Unit test commands
`cargo test -p psm-testkit --test suite h4_details_only_publish::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::` · `cargo test -p psm-testkit --test suite p13_publish_runs::` · `cargo test -p psm-testkit --test suite p14_publish_update::` · `cargo test -p psm-testkit --test suite p03_session_state_list::` · `cargo test -p psm-testkit --test suite p27_retire_shape_on_arrival::` · `cargo test -p psm-desktop` · `cd apps/desktop && npm run typecheck` · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings`

## Unit 3 — The workspace UI

**Unit id:** `H4·U3` · **Rows:** H4·C10–C14 · **Changes app files:** yes

### Goal
The standard workspace for a remote session, the note, the edit gate, the Bring back dialog with progress, the capture-target gate, Refresh from Shopify, the details-only confirmation; `ArchivedView` and `item_archive_get` go.

### Read first
Master plan §4 (Words, Events, Frontend), §5 **MC-H3.4, .5**, **MC-H4.1, .3, .5** · `apps/desktop/src/App.tsx` (**all of it**: `useCaptureTarget` :95-111, `captureTarget(nav)` :163, the conversations context) · `apps/desktop/src/features/conversations/visit.ts` · `apps/desktop/src/features/workspace/{WorkspacePage.tsx (whole), useWorkspaceState.ts, Stage.tsx, PhotoPanel.tsx, PhotoTile.tsx, PhotoMenu.tsx (:8), useKeyboard.ts, ArchivedView.tsx, ItemDetailsCard.tsx}` · `apps/desktop/src/features/shell/Header.tsx` (:40-60 the session identity, :214-240 the actions hidden for `archived`) · `apps/desktop/src/features/sessions/{DeleteSession.tsx (`SessionActionsMenu` :27), StatusPill.tsx (:81-86)}` · `apps/desktop/src/features/publish/PublishPopover.tsx` · `apps/desktop/src/services/{remote.ts, retention.ts, events.ts, photos.ts}` · `apps/desktop/src/features/workspace/__tests__/{p62_archived_ui.test.tsx, p08_workspace_ui.test.tsx}` and `apps/desktop/src/features/conversations/__tests__/c4_visit.test.tsx` (how `App` is rendered with mocked services and the activate/deactivate call list asserted) · `docs/intention/evidence/design_handoff.md`.

### Files
New: `apps/desktop/src/features/workspace/remote/{remotePhotos.ts, useRemoteItem.ts, RemoteNote.tsx, BringBackDialog.tsx}`, `apps/desktop/src/features/workspace/__tests__/{h4_remote_workspace.test.tsx, h4_bring_back.test.tsx}`, `apps/desktop/src/features/shell/__tests__/h4_capture_target.test.tsx`. Edited: `apps/desktop/src/App.tsx`, `apps/desktop/src/features/workspace/{WorkspacePage.tsx, useWorkspaceState.ts, workspace.css}`, `apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/features/sessions/DeleteSession.tsx`, `apps/desktop/src/features/publish/PublishPopover.tsx`, `apps/desktop/src/services/{retention.ts, bindings.ts (generated)}`, `apps/desktop/src-tauri/src/{lib.rs, commands/items.rs}` (removal of `item_archive_get`), the test files that render `App` or `WorkspacePage` with mocked services (a `services/remote.ts` mock answering `local`; see Notes), `docs/guide/{01_product.md, 02_architecture.md}`. Removed: `apps/desktop/src/features/workspace/ArchivedView.tsx`, `apps/desktop/src/features/workspace/__tests__/p62_archived_ui.test.tsx`.

### Core mechanisms (the only functions with unit tests)
`remotePhotos(view)` in `features/workspace/remote/remotePhotos.ts` (pure: the pictures as the workspace's photo objects).

### Ordered tasks
1. `useRemoteItem(sessionId)`: `getRemote(sessionId)` on mount and on `session_changed` for that session; merges `picture_ready {kind: 'remote'}` paths and `bring_back_progress` into its state; exposes `{view, remote: boolean, reload, refresh, bringBack}`. One instance per session on screen; `App.tsx` reads the capture target's state from it (task 5).
2. `remotePhotos(view)`: one photo object per picture — a negative id (`-(index + 1)`), `classification` main / condition, `position`, the picture's `path` as both preview paths (null when not downloaded), a neutral composition (`normal`, `original`, full frame), `integrity: 'ok'`, `upload_state: 'ready'`, `source: 'import'`.
3. `WorkspacePage`: for a remote session the standard workspace renders with `remotePhotos(view)` in place of the session view's photos (which are empty, H4·C3(a)); `RemoteNote` under the header (the sentence and **Bring back now**; for `unknown` without `check_failed`: `Checking Shopify…`; with `check_failed`: the sentence and **Try again** = `reload`); the `ArchivedView` branch (:418-422) goes. **The edit gate:** while the session is remote, every photo-changing action opens `BringBackDialog` and calls no photo command: classify (buttons, swipe, keys), deny, drag (reorder, re-classify, template drop), compose (`r`, the stage button, the menu's compose / split / normal / apply), move, copy, assign. Selecting, stepping, the original/adjusted toggle, Details and the item card are not gated.
4. `BringBackDialog`: the question (`Bring back <N> photos from Shopify to edit them? About <X> MB.`, singular for one; `X = max(1, round(bring_back_bytes / 1 000 000))`), **Bring back** / **Cancel**; after **Bring back**: `Bringing back <k> of <N> photos…` from the progress events (and from `view.bring_back` when the dialog opens while one is running); on success the dialog closes and the workspace reloads (the session is now `local`); on failure the error's message and **Try again**.
5. Capture target (MC-H3.4): `useCaptureTarget(target, state)` calls `activateSession(target)` only when the target's remote state is `local`, or `unknown` with `check_failed`; when the state becomes one of those while the target is unchanged it calls it once; while the state is not yet loaded it calls nothing. Leaving (target undefined or Unassigned) is unchanged. The visit rule is untouched: `target` is still `captureTarget(nav)`.
6. `Header`: the publish actions are no longer hidden for an archived session (:220); the row label is unchanged (`Archived in Shopify`). `SessionActionsMenu`: **Refresh from Shopify** for a remote session (`refreshRemote(sessionId)`, then the workspace's `reload`; a failure is a toast). `PublishPopover`: for a `details_only` item the line `Details only: the photos stay as they are on Shopify.` and the button `Publish details`.
7. Remove `item_archive_get` (command, DTOs it alone used, `getItemArchive`) and `ArchivedView` with its test; `setKeepFiles` answers the remote view.
8. Guide: `01_product.md` (the workflow rows "Archived in Shopify" and "Bring back"; vocabulary: remote item, shop-only), `02_architecture.md` (§Frontend: `features/workspace/remote/`, the capture-target gate).
9. Tests for the rows.

### Criteria (5 criteria, 28 rows)
Boundary: `remotePhotos` (pure); `WorkspacePage` and `App` rendered in UI tests with `services/remote.ts`, `services/photos.ts`, `services/sessions.ts`, `services/compositions.ts`, `services/events.ts` and the other services mocked (outcome = what is rendered and the exact lists of calls to the mocked commands).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H4·C10(a) | `remotePhotos` | a view with gallery 1–2 (paths `/a`, none) and condition 1 (`/c`) | three photos: ids −1, −2, −3; classifications main, main, condition; positions 1, 2, 1; preview paths `/a`, null, `/c` | MC-H3.5 |
| H4·C10(b) | `WorkspacePage` | `getRemote` answers `archived` with 3 gallery and 2 condition pictures, all with paths; `getSessionView` answers no photos | the standard workspace: the stage shows gallery picture 1, the panel a Main group of 3 tiles and a Condition group of 2, each an `<img>` of `previewUrl(path)`; no "archived" page | `[rule]` 10H §6 ("the standard workspace") |
| H4·C10(c) | same | as (b) with gallery picture 2 lacking a path; then `picture_ready {kind: 'remote', session_id, role: 'gallery', position: 2, path: '/b'}` | first a placeholder tile without `<img>`; after the event that tile shows `previewUrl('/b')` and `getRemote` was not called again | M-H4; 10H §6 ("each as it arrives") |
| H4·C10(d) | same | (b) | under the header: `Photos are on Shopify. Editing a photo brings them back to this Mac first.` with a `Bring back now` button | `[rule]` 10H §6 |
| H4·C10(e) | same | `getRemote` answers `shop_only` with 2 gallery pictures | as (b) and (d) with a Main group of 2 | `[rule]` 10H §1 (6), §6 |
| H4·C10(f) | same | `getRemote` answers `unknown`, `check_failed: false`; then `unknown`, `check_failed: true` | `Checking Shopify…`; then `Could not check Shopify for this product's photos.` with `Try again`, whose click calls `getRemote` once more | `[rule]` MC-H3.2 |
| H4·C10(g) | same | (b); press `D` / the Details button | the Details form opens; no Bring back dialog | 10H §6 ("allowed without bringing back") |
| H4·C10(h) | same | `getRemote` answers `local` | the workspace as before this unit: no note, the session view's photos | `[rule]` MC-H3.5 |
| H4·C11(a) | `WorkspacePage` (remote, 5 pictures, `bring_back_bytes` 7 600 000) | click the stage's Condition review button | the dialog `Bring back 5 photos from Shopify to edit them? About 8 MB.` with `Bring back` and `Cancel`; `classifyPhoto` was not called | 10H §6 ("there is no silent download") |
| H4·C11(b) | same | press the classify key for Condition | the dialog; `classifyPhoto` not called | 10H §6 |
| H4·C11(c) | same | the photo menu's Delete | the dialog; `deletePhoto` not called | 10H §6 (hide) |
| H4·C11(d) | same | drag tile 3 before tile 1 and drop | the dialog; `reorderPhoto` not called | 10H §6 (rearrange) |
| H4·C11(e) | same | press `R`; then (dialog closed) the menu's Split | the dialog each time; the compose editor does not open; `setComposition` not called | 10H §6 (rotate, framing) |
| H4·C11(f) | same | the menu's Move to session; the menu's Copy to session; the menu's Assign to product | the dialog each time; no session or item picker opens; `movePhotos`, `copyPhotos` not called | 10H §6 |
| H4·C11(g) | same | click `Bring back now` in the note | the same dialog | `[rule]` master plan §6 |
| H4·C11(h) | same | `Cancel` | the dialog closes; `bringBack` was not called; the workspace is unchanged | 10H §6 (Cancel) |
| H4·C11(i) | same | a view with one picture and `bring_back_bytes` 300 000 | `Bring back 1 photo from Shopify to edit it? About 1 MB.` | `[rule]` 10H §6 |
| H4·C12(a) | `BringBackDialog` | click `Bring back`; then `bring_back_progress {state: 'running', done: 4, total: 10}` | `bringBack(sessionId)` called exactly once; the dialog shows `Bringing back 4 of 10 photos…` and no `Bring back` button | 10H §7.1 (progress) |
| H4·C12(b) | same | `bringBack` resolves; `getRemote` then answers `local` and `getSessionView` 5 photos | the dialog is closed, the note is gone, the panel shows the 5 session photos | 10H §7.2 ("from then on an ordinary session") |
| H4·C12(c) | same | `bringBack` rejects with `Could not bring back the photos: 1 photo could not be downloaded` | that sentence and `Try again`; clicking it calls `bringBack` a second time | 10H §7.1 ("with the reason and Try again") |
| H4·C12(d) | `WorkspacePage` | `getRemote` answers `archived` with `bring_back: {done: 2, total: 5}`; click the Condition review button | the dialog opens on `Bringing back 2 of 5 photos…`; `bringBack` is not called | MC-H4.1 (one at a time) |
| H4·C13(a) | `App` | in session B (`activateSession(B)` made); open archived session A from the sessions panel | since the navigation neither `activateSession` nor `deactivateSession` was called | MC-H3.4; 10H §6 |
| H4·C13(b) | same | on the Sessions page; open archived session A; then go back to the Sessions page | no `activateSession` and no `deactivateSession` call at all | MC-H3.4 |
| H4·C13(c) | same | after (a): Bring back completes (`bringBack` resolves, `getRemote` answers `local`) | `activateSession(A)` called exactly once | MC-H3.4; 10H §7.4 |
| H4·C13(d) | same | archived session A opened as a visit from a conversation (C4), then Bring back completes | no `activateSession` call | CR4; MC-H3.4 ("still the capture target") |
| H4·C13(e) | same | open an adopted session whose `getRemote` first answers `unknown` and then (after `session_changed`) `shop_only`; a second run where it then answers `local`; a third where it answers `unknown` with `check_failed: true` | no call; `activateSession` once; `activateSession` once | MC-H3.4 |
| H4·C14(a) | `SessionActionsMenu` in the workspace header | a remote session: open the ⋯ menu and choose `Refresh from Shopify`; a `local` session: open the menu | `refreshRemote(sessionId)` called once and `getRemote` again after it; no such entry for the local session | `[rule]` 10H §6 (Refresh from Shopify) |
| H4·C14(b) | `Header` and `PublishPopover` | an archived session whose preview answers one `details_only` item | the header offers Publish; the confirmation shows `Details only: the photos stay as they are on Shopify.` and a `Publish details` button whose click confirms the publish | `[rule]` 10H §6 ("Publish of details only") |

### Delegated choices
How the gate is wired (a wrapper around `useWorkspaceState`'s actions is the smallest) · the placeholder tile's look · whether `useRemoteItem` is lifted into `App.tsx` and passed down or shared through a context.

### Notes
- Hazard (photos in the wrong product): the capture-target gate is the only thing that keeps an archived product from becoming the camera's folder. H4·C13(a)–(e) assert the **whole** call list for that reason. The existing navigation tests (`p19_navigation`, `c4_visit`, `a19_review_queue`, `a11_header_review`, `w3_writer_triggers`) mock no remote service today: they get a `services/remote.ts` mock answering `local`, and where an existing assertion on `activateSession` now holds one tick later it may be wrapped in `waitFor`; those are the only edits allowed in them, and a changed expectation there is a signal that normal activation changed.
- Hazard: `useCaptureTarget` must not call `activateSession` in the render where the state is still loading and then again when it arrives (two activations, two script runs). One call per (target, becomes-activatable).
- Hazard: the pseudo-photos have negative ids; nothing may send one to a backend command. The gate is in front of every action that would.
- Hazard: after a successful bring back the reload must come from `session_changed` **or** the resolved call, whichever is first, without showing the remote pictures and the session photos together.
- Hazard: keyboard shortcuts are registered in `useWorkspaceKeyboard` and `WorkspacePage` (`r`, :360); both need the gate.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/workspace src/features/shell src/features/sessions src/features/publish src/features/conversations` · `npm run typecheck` · `cargo test -p psm-desktop` · `cargo clippy -p psm-desktop --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-testkit --test suite h4_remote_workspace::` · `cargo test -p psm-testkit --test suite h4_details_only_publish::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/workspace src/features/shell src/features/publish src/features/conversations` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: `activate` returns for a remote session before any editing or camera call; `view` and the commands never download; a `details_only` run reaches neither the gallery block nor a condition-photos write, and reads the flag from the stored item; `useCaptureTarget` is still the only caller of `activateSession` / `deactivateSession`; no pre-existing navigation test's assertions changed; `p62`'s `c3a` and `c3b` changed only as tasks U1·8 and U2·6 say. Notes go to `docs/implementation/backlog_archive/H4_remote_workspace.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
1. In Sessions, open a product labelled "Archived in Shopify". The usual workspace opens; the pictures appear one after another; a line says "Photos are on Shopify. Editing a photo brings them back to this Mac first."
2. Take a photo with the camera: it does **not** land in this product (it goes where photos went before you opened it).
3. Press Details, change the title, publish: the confirmation says "Details only: the photos stay as they are on Shopify." Check the product on Shopify: new title, same pictures.
4. Drag a picture to another place. The app asks "Bring back N photos from Shopify to edit them? About X MB." Press Bring back and watch "Bringing back 4 of 10 photos…".
5. The product is now an ordinary session: reorder, turn or re-frame a photo, open it in Capture One (the JPEGs are in the session's folder). Publish: only the changed photo is uploaded; the order follows yours.
6. Search (N) for a product photographed on the other Mac and open it: the same workspace with its Shopify pictures, and the same Bring back.
7. In the session's ⋯ menu, "Refresh from Shopify" re-reads the pictures after someone changed them in Shopify's admin.

(For the orchestrator's live check after H4, carried from H1's projection r0: include one 10F clearing verification, i.e. the readback with `originalSource.fileSize` and `GenericFile.originalFileSize`, not only covers; those field names and `nodes(ids:)`'s `null` per unknown id are schema-grounded only.)

(Also for that live check, carried from H3's projection r0, L6: record the first bytes and the content type of one full-size download made through `ReqwestTransport` (`fetch_original`); nothing grounds that the CDN answers JPEG bytes, and MC-H4.3 refuses a body that does not start with `FF D8 FF`.)

## Review log

- **2026-11-23, orchestrator: carried from H1's projection r0** (`handoffs/reviewer/handoff_H1_points_and_remote_record_projection_r0.md`): the "Note for H4's projection" (an `Unknown` item and retention) added to U2 Notes; the live check's clearing verification added under Manual try-out. Rows and counts unchanged.
- **2026-11-23, orchestrator: carried from H3's projection r0** (`handoffs/reviewer/handoff_H3_bring_back_projection_r0.md`): U1 Notes gain the bring-back fixture note (register pictures with `set_image`, files with `add_image`; the uncached capture-root source); Manual try-out gains the live check's full-size download line (L6). Rows and counts unchanged.
