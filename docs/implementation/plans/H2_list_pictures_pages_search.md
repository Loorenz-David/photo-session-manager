---
phase: H2_list_pictures_pages_search
title: A picture on every row, pages, and pictures in the search — the sessions list never shows a broken picture: an archived product's row shows its first Shopify picture, fetched small in the background and kept on this Mac; the list has pages of 25 with page numbers; the New session and Assign product searches show each result's first picture
difficulty: normal
projection: no
depends_on: [H1_points_and_remote_record]
blocked_on: none
---
# H2 — A picture on every row, pages, and pictures in the search

**Tracker:** IMPLEMENTED (2026-11-23, claude-opus implementer r1: 4 units, 67 rows tested, closing gate green; handoff `handoffs/implementer/handoff_H2_list_pictures_pages_search_implement_r1.md`)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `10H` §4 and §5 by contracts **MC-H1** and **MC-H2** (`docs/implementation/master_plan_archive.md` §5). (U1) The row-picture rule, the cover cache, the background fetch and the stale-URL refresh. (U2) The paged list and its command. (U3) The list UI: placeholder, live pictures, page numbers. (U4) Pictures on search results, backend and UI. **Not here:** the workspace of a remote item, its 1600 px pictures, activation (H4); bring back (H3); pages in the pickers and other lists (10H §10: only the sessions list is paged).

## How to work this plan (flow.md §10, master_plan_archive.md §3)
One runner phase of 4 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `H2·C<n>(<row>)`; tests named after their row (`h2_c1a_<slug>`). Open details: the smallest option consistent with 10H §4–§5 and MC-H1/MC-H2, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_archive/H2_list_pictures_pages_search.md`. **Never call Shopify or its CDN, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Row pictures: rule, cover cache, background fetch, stale URLs | 5 | 25 | **yes** |
| U2 | Pages: the rule and the command | 3 | 14 | **yes** |
| U3 | The list UI: placeholder, live pictures, page numbers | 2 | 12 | **yes** |
| U4 | Search pictures, backend and UI | 4 | 16 | **yes** |
| **Total** | | **14** | **67** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `docs/implementation/master_plan_archive.md` §3, §4 (Constants, Pure rules, Caches, Service, Events, Commands), **§5 MC-H1, MC-H2, MC-H5.3, MC-H3.7** · book `10H` §4, §5, §8, §9 (M-H1, M-H4, M-H5) · the H1 plan (what `RemoteItems`, `FakeCommerce` and the adapter already offer) · `docs/guide/04_change_impact.md` rows "A Tauri command signature", "A new `AppEvent` variant", "`AppPaths` layout".

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **A row never shows a broken picture:** it shows this Mac's preview, or the Shopify picture once fetched, or a neutral placeholder.
- **Pictures are fetched only for the page you are looking at,** four at a time, after the list is already on screen.
- **A picture Shopify no longer has** stays a placeholder and is reported once on Health.
- **The search, the filter tabs, the time filter and "To review" look through every session;** only the result is cut into pages.
- **Page numbers** show every page up to 7, otherwise the first, the last and those around the current one.
- **A shop product you have not opened yet shows the placeholder in the list** (its pictures are read at first opening); in the search it shows the shop's picture.

## Unit 1 — Row pictures: rule, cover cache, background fetch, stale URLs

**Unit id:** `H2·U1` · **Rows:** H2·C1–C5 · **Changes app files:** yes

### Goal
`row_picture` in `list`, the cover cache, `request_covers`, the download batch in `run_pending` with MC-H1.4 and MC-H1.5, `AppEvent::PictureReady` up to the window event.

### Read first
Master plan §5 **MC-H1** (every item), **MC-H5.3** · `crates/core/src/services/sessions.rs` (`list` :408-512; `cover_preview_path` :475) · `crates/infra/src/sqlite/store.rs` (`session_list_records` :2338-2375: the cover SQL at :2359) · `crates/core/src/domain/session.rs` (`SessionRow`, `SessionListRecord` :157) · `crates/core/src/services/remote.rs` (H1) · `crates/core/src/services/retention.rs` (`cached_path` :180, `download` :220-240: the `.part` + rename pattern) · `crates/core/src/services/health.rs` (`set` :76, `clear` :96) · `crates/core/src/events.rs` · `apps/desktop/src-tauri/src/lib.rs` (`impl EventSink for TauriEvents`: one arm per event) · `apps/desktop/src-tauri/src/commands/sessions.rs` (`SessionRowResponse` :308-330) · `crates/core/src/services/storage/session_delete.rs` (`delete_session` :123) and `crates/integrations/src/session_delete.rs` (the entry point `delete` the command and the tests call) · `crates/testkit/tests/suite/a17_delete_session.rs` (its harness, :244).

### Files
New: `crates/testkit/tests/suite/h2_row_pictures.rs`. Edited: `crates/core/src/{domain/remote.rs, domain/session.rs, mechanisms/remote.rs, services/remote.rs, services/sessions.rs, events.rs}`, `crates/integrations/src/{lib.rs, session_delete.rs}`, `apps/desktop/src-tauri/src/{lib.rs, commands/sessions.rs}`, `apps/desktop/src/services/{bindings.ts (generated), events.ts}`, `crates/testkit/src/fake_commerce.rs`, `crates/testkit/tests/suite/main.rs`, `docs/guide/{02_architecture.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
`mechanisms::remote::cover_name`.

### Ordered tasks
1. `cover_name` (master plan §4), and `RemoteItems::refresh` now uses it: H1 deletes every `remote/<item_id>/cover-*` on a refresh (it had no `cover_name`); from here it keeps `cover_name(new cover picture)` (MC-H3.3; carried from H1's projection, L8). `PictureTarget`; `AppEvent::PictureReady` with its `TauriEvents` arm → `picture_ready` (payload of master plan §4 Events) and the `onPictureReady` listener in `services/events.ts`.
2. `RemoteItems::row_picture(session_id, record's main-photo preview paths) -> (Option<PathBuf>, cover_pending)` by MC-H1.1. `SessionsService::list` uses it for every row when a `RemoteItems` is attached (`with_remote`; every production caller of `list` attaches it), and without one applies rule (i) and (iii) only (the existing-file check). `SessionRow` gains `cover_pending`; `SessionRowResponse` likewise.
3. The queue: `request_covers(source: CoverSource {Page, Search}, session_ids)` replaces that source's not-started cover downloads with those of the given sessions that are pending (MC-H1.3); it never downloads. `run_pending` (H1) first takes the queued downloads as one batch: at most `PARALLEL_DOWNLOADS` at once (`std::thread::scope`), each `fetch_image(url, COVER_EDGE)` → `.part` → rename → delete other `cover-*.jpg` → `PictureReady {Cover}`; then MC-H1.4 (stale ids → budget check → `media_urls` in chunks → `update_remote_urls` → one retry each; the cool-down map uses `clock.mono()`); then MC-H1.5 (Health); then the queued first-open reads of H1. It answers `Waiting` only when a GraphQL step was held back by the budget and nothing else was done.
4. "Delete session" (`psm_integrations::session_delete::delete`, which has the app's paths and `RemoteItems`): after the rows go, remove the folders `<previews>/remote/<item_id>/` and `<previews>/archived/<item_id>/` when the item row was deleted (cache files, not RAWs: plain removal), and drop the item from the gone map. No adapter call (M-H5).
5. Guide: `02_architecture.md` (§Persistence: the `previews/remote/` layout; §Remote items: the row-picture rule, the queue, the stale rule), `04_change_impact.md` (a row "anything that hands the screen a preview path → the existing-file rule of `RemoteItems::row_picture`, M-H4").
6. Tests for the rows. **Carried from H1's projection (L9, L10):** `RemoteItems::new` gains `health` (MC-H1.5; H1 took only what it read, master plan §4 Service), wired in `App::build`; `FakeCommerce` gains `media_urls` (answers from the shop's media and files, `None` for an id it does not hold; records each call's ids in `url_reads`), `image_faults` (`fail_image(url, Vec<ImageFault {Gone, Fail}>)`, consumed one per `fetch_image` of that URL), `set_fetch_delay(ms)` and `max_fetches_in_flight()` (as `max_uploads_in_flight`); each with a caller in this unit's rows (H2·C3(b), H2·C4).

### Criteria (5 criteria, 25 rows)
Boundary: `SessionsService::list` with `RemoteItems` on `TestApp::with_fake_shop()` (outcome = the row's `cover_preview_path` and `cover_pending`), `RemoteItems::{request_covers, run_pending}` (outcome = files on disk, `FakeCommerce`'s recorded fetches / URL reads / mutations, emitted events, Health). "Archived item" and "adopted item" as in the H1 plan.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H2·C1(a) | `list` | a session whose main photo 1 has an adjusted preview file on disk | that path; `cover_pending` false | MC-H1.1(i); M-H4 |
| H2·C1(b) | same | the adjusted preview path is stored but its file was deleted; the original preview file exists | the original preview path | MC-H1.1(i); M-H4 |
| H2·C1(c) | same | both files deleted; the item is `Local` | none; `cover_pending` false | MC-H1.1(iii); M-H4 |
| H2·C1(d) | same | an archived item (its photo rows still name the deleted previews) whose cover cache file exists | the cover cache path | MC-H1.1(ii); 10H §4 |
| H2·C1(e) | same | an archived item without a cover cache file | none; `cover_pending` true | MC-H1.1(ii) |
| H2·C1(f) | same | an adopted item with a record (H1 first-open read done) and a cover cache file; an adopted item never read (`Unknown`) | the cover cache path; none with `cover_pending` false | MC-H1.1; 10H §3 |
| H2·C1(g) | same | an archived item whose record has only condition pictures | none; `cover_pending` false | MC-H1.1 (no cover picture) |
| H2·C2(a) | `request_covers(Page, [s])` then `run_pending` | an archived item whose gallery picture 1 is media `m1` | the file `<previews>/remote/<item_id>/cover-<first 12 hex of sha256("…m1")>.jpg` holds the bytes the fake serves for `m1`'s URL; `FakeCommerce` recorded the fetch `(m1's URL, 320)`; no `.part` file remains | MC-H1.2; 10H §4 |
| H2·C2(b) | same | after (a): Refresh from Shopify with the gallery reordered so `m2` is first; request and run again | the folder holds exactly one `cover-*.jpg`, the one named after `m2` | MC-H1.2 ("a new cover media id replaces it") |
| H2·C2(c) | same | after (a): request and run again | no further fetch recorded | MC-H1.3 (cached) |
| H2·C2(d) | emitted events | (a) | exactly one `PictureReady {Cover {session_id: s}, path}` with the file's path | MC-H1.3; 10H §4 ("an event, not a reload") |
| H2·C3(a) | `SessionsService::page` (U2) / `list` + `request_covers` | 3 archived items pending; the request is made and control returns | no fetch was recorded before `run_pending` is called | MC-H1.3 ("never delaying the list") |
| H2·C3(b) | `run_pending` | 10 archived items requested; `FakeCommerce::set_fetch_delay(50)` | 10 covers written; `max_fetches_in_flight()` is 4 | MC-H1.3; 10H §4 (at most 4 at a time) |
| H2·C3(c) | `request_covers` twice, then `run_pending` | `Page` with sessions A, B; then `Page` with sessions C, D (all pending) | fetches recorded for C and D only | MC-H1.3 (only the page being shown) |
| H2·C3(d) | same | `Page` with A, B; then `Search` with C | fetches recorded for A, B and C | MC-H1.3 (a source replaces only its own) |
| H2·C4(a) | `run_pending` | 3 archived items requested; `FakeCommerce` answers `Gone` once for each cover URL and then serves it at the URL `media_urls` returns | exactly one `media_urls` call, with the 3 cover media ids; each item's stored cover URL is the new one; 3 covers written (6 fetches recorded: 3 gone, 3 retried) | MC-H1.4; MC-H5.5; 10H §4 (one batched query, retry once) |
| H2·C4(b) | same | one item's cover media was deleted from the fake shop (`media_urls` answers `None`) and its URL answers `Gone` | that picture's stored URL is empty; no cover file; Health `remote_picture_gone` = `1 picture is no longer in the shop: <its SKU>`; a second request + `run_pending` records no fetch and no `media_urls` call for it | MC-H1.4, .5; 10H §4 |
| H2·C4(c) | same | the cover URL answers `Gone`, `media_urls` returns a new URL that answers `Gone` too | no cover; within 10 minutes of the clock a second request + run makes no `media_urls` call; after 10 minutes it makes one | MC-H1.4 (cool-down); MC-H5.5 |
| H2·C4(d) | same | the cover URL answers `Fail` (not gone) | no cover; no `media_urls` call | MC-H1.4 |
| H2·C4(e) | same | 2 items requested, one `Gone` and one fine; budget `{100, 1000}`; then budget `{900, 1000}` and `run_pending` again | first run: the fine item's cover is written, no `media_urls` call; second run: one `media_urls` call and the other cover written | MC-H5.3; 10H §8 |
| H2·C4(f) | Health | after (b): Refresh from Shopify of that item (the shop now has a first gallery picture) | Health no longer holds `remote_picture_gone` | MC-H1.5 |
| H2·C5(a) | `FakeCommerce::mutation_ops()` | after H2·C2(a), C3(b), C4(a) and C4(b) | empty | M-H1 |
| H2·C5(b) | "Delete session" (`psm_integrations::session_delete::delete`, as `a17` calls it) | an archived item with a cover file and one `archived/<item_id>/` file | both folders are gone; `FakeCommerce` recorded no read and no mutation during the delete | M-H5; 10H §9 |
| H2·C5(c) | same | after H2·C4(b) (the item is in Health), delete its session | Health no longer holds `remote_picture_gone` | MC-H1.5 |
| H2·C5(d) | `cover_name` | `"gid://shopify/MediaImage/1001"` | `cover-` + the first 12 lowercase hex characters of its SHA-256 + `.jpg` (the test computes the digest with `sha2`) | MC-H1.2 |

### Delegated choices
The queue's data structure · whether one `run_pending` handles covers and (from H4) view pictures in one batch or two.

### Notes
- Hazard (M-H4): today the cover SQL returns the stored path whether or not the file exists (`store.rs:2359`), which is the "?" the owner saw: 10F deletes the previews and leaves the paths. The fix is the existing-file check in **one** place (`row_picture`), not a clean-up of the stored paths (bring back retires those rows later, H3).
- Hazard: `list` loads every session; `row_picture` adds two `is_file` checks per row at most. Do not add a per-row store read: take what the record already carries, and fetch the remote inputs for all sessions in one read (a `Store` method returning the cover picture and state inputs per session id is the implementer's choice).
- Hazard: `request_covers` must never block on `run_pending` (a download in flight holds no lock the request needs).
- Hazard: a `Gone` answer and a transport failure are different (H1·C6): only `ImageGone` leads to a refresh.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-testkit --test suite h2_row_pictures::` · `cargo test -p psm-testkit --test suite p03_session_state_list::` · `cargo test -p psm-testkit --test suite a17_delete_session::` · `cargo test -p psm-desktop` · `cargo clippy -p psm-core -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — Pages: the rule and the command

**Unit id:** `H2·U2` · **Rows:** H2·C6–C8 · **Changes app files:** yes

### Goal
`page_of`, `SessionsService::page`, the command `sessions_page`; `sessions_list` unchanged in shape.

### Read first
Master plan §5 **MC-H2.1–2** · `crates/core/src/services/sessions.rs` (`list` :408-512) · `crates/core/src/mechanisms/time.rs` (`group_by_activity` :10) · `crates/core/src/domain/session.rs` (`SessionList` :150, `ListGroup`) · `apps/desktop/src-tauri/src/commands/sessions.rs` (`SessionListResponse` :340 and its `From` :347-407, `sessions_list` :409-423) · `crates/testkit/tests/suite/p03_session_state_list.rs` (how list tests create sessions with chosen creation times).

### Files
New: `crates/testkit/tests/suite/h2_pages_and_search.rs`. Edited: `crates/core/src/{domain/session.rs, mechanisms/remote.rs, services/sessions.rs}`, `apps/desktop/src-tauri/src/{lib.rs (`specta_builder()`, `h2_tests`), commands/sessions.rs}`, `apps/desktop/src/services/{bindings.ts (generated), sessions.ts}`, `crates/testkit/tests/suite/main.rs`.

### Core mechanisms (the only functions with unit tests)
`mechanisms::remote::page_of`.

### Ordered tasks
1. `PAGE_SIZE`, `page_of(total, page) -> (page, pages, start..end)` (MC-H2.1: clamp, at least one page).
2. `SessionsService::page(filter, query, created_from, only_ids, page) -> SessionPage` by MC-H2.1, built on `list`; then `request_covers(Page, the page's pending rows)` when a `RemoteItems` is attached.
3. Command `sessions_page(filter, query, created_from, only_ids, page)` → `SessionPageResponse {groups: [{label, total, rows}], counts, unassigned_count, page, pages, total}` (rows are `SessionRowResponse`), registered in `specta_builder()`; wrapper `sessionsPage` in `services/sessions.ts`. `sessions_list` keeps its signature and shape.
4. Tests for the rows.

### Criteria (3 criteria, 14 rows)
Boundary: `page_of` (pure); `SessionsService::page` on `TestApp` (sessions created through `ItemsService::create_local` at chosen clock times); the command handler for H2·C8.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H2·C6(a) | `page_of` | `(60, 1)`; `(60, 3)` | `(1, 3, 0..25)`; `(3, 3, 50..60)` | MC-H2.1; 10H §5 (25 per page) |
| H2·C6(b) | same | `(60, 9)`; `(60, 0)` | `(3, 3, 50..60)`; `(1, 3, 0..25)` | MC-H2.1 (clamped) |
| H2·C6(c) | same | `(0, 1)` | `(1, 1, 0..0)` | MC-H2.1 (at least one page) |
| H2·C6(d) | same | `(25, 1)`; `(26, 2)` | `(1, 1, 0..25)`; `(2, 2, 25..26)` | MC-H2.1 (the boundary) |
| H2·C7(a) | `page` | 60 sessions created at 60 distinct times; filter all, no query, page 1 | 25 rows: the 25 newest created, newest first; `total` 60, `pages` 3, `page` 1 | MC-H2.1 |
| H2·C7(b) | same | same store, page 3 | the 10 oldest, newest first | MC-H2.1 |
| H2·C7(c) | same | same store; query = the SKU of the 40th newest session; page 1 | one row, that session; `total` 1, `pages` 1 | MC-H2.1; 10H §5 ("a search finds items on any page") |
| H2·C7(d) | same | 60 sessions of which the 30 oldest are published; filter published; page 2 | 5 rows; `total` 30, `pages` 2 | MC-H2.1 (filter first, then cut) |
| H2·C7(e) | same | `created_from` = the creation time of the 10th newest | `total` 10 | MC-H2.1, .3 (time filter) |
| H2·C7(f) | same | `only_ids` = the ids of the 1st, 30th and 55th newest | `total` 3, all on page 1, newest first | MC-H2.1, .3 ("To review") |
| H2·C7(g) | same | 30 sessions created today (one group "Today"), page 1 and page 2 | page 1: group `Today` with 25 rows and `total` 30; page 2: group `Today` with 5 rows and `total` 30 | MC-H2.1 (groups cut, totals kept) |
| H2·C7(h) | same | (c) and (d) | `counts` = `{open: 30, published: 30, all: 60}` in both (as `list` reports them) | MC-H2.1 |
| H2·C8(a) | command `sessions_page` (desktop test `h2_tests`) | 30 sessions, page 2 | the response carries `page: 2`, `pages: 2`, `total: 30` and 5 rows, each with `cover_pending` | MC-H2.2 |
| H2·C8(b) | command `sessions_list` | the same 30 sessions | 30 rows (unpaged, as before) | MC-H2.2; 10H §5 ("other callers keep the unpaged list") |

### Delegated choices
Whether `page` re-uses `list`'s row vector or shares a private builder with it.

### Notes
- Hazard: `list`'s order comes from `group_by_activity` (created time descending, id descending); cut **after** grouping order is fixed, or a group's rows swap pages on equal times.
- Hazard: `only_ids: Some([])` means "no session" (`total` 0), not "no filter".
- The page is cut in the service, not in SQL: `session_list_records` already loads every session and the goal of paging is the pictures and Shopify's points, not the query.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-testkit --test suite h2_pages_and_search::` · `cargo test -p psm-testkit --test suite p03_session_state_list::` · `cargo test -p psm-desktop` · `cd apps/desktop && npm run typecheck`

## Unit 3 — The list UI: placeholder, live pictures, page numbers

**Unit id:** `H2·U3` · **Rows:** H2·C9–C10 · **Changes app files:** yes

### Goal
`SessionsPage` on `sessionsPage` with page numbers; rows, the hover card and the workspace sessions panel show the row picture or a placeholder; a `picture_ready` cover event updates its row.

### Read first
Master plan §5 **MC-H2.2–3**, **MC-H1.3** · `apps/desktop/src/features/sessions/{SessionsPage.tsx (whole), SessionRow.tsx (:39-78), HoverPreview.tsx, GroupHeader.tsx, TimeFilter.tsx, reviewQueue.ts (`waitingForReview`)}` · `apps/desktop/src/lib/format.ts` (`withinRange` :39, `startOfDay`) · `apps/desktop/src/features/workspace/SessionsPanel.tsx` (:25-60) · `apps/desktop/src/services/{sessions.ts, events.ts}` · `apps/desktop/src/features/sessions/__tests__/{p07_shell_sessions_ui.test.tsx, a19_review_queue.test.tsx}` (how the page is tested with mocked services) · `docs/intention/evidence/design_handoff.md` (tokens for the placeholder and the page control).

### Files
New: `apps/desktop/src/features/sessions/{Pagination.tsx, usePictures.ts}`, `apps/desktop/src/features/sessions/__tests__/h2_list_pages_and_pictures.test.tsx`. Edited: `apps/desktop/src/features/sessions/{SessionsPage.tsx, SessionRow.tsx, HoverPreview.tsx, GroupHeader.tsx}`, `apps/desktop/src/features/workspace/SessionsPanel.tsx`, `apps/desktop/src/lib/format.ts` (a `rangeStart(range, now)` beside `withinRange`), `apps/desktop/src/styles/globals.css`, and the existing tests that mock `listSessions` for the Sessions page (`p07_shell_sessions_ui`, `a19_review_queue`, `a10_changed_after_publish`, `a17_delete_session`, `c2_session_badges`, `r11_arranged_tag`, `w3_writer_triggers`: their mock becomes `sessionsPage` with the same rows; no assertion changes).

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `usePictures()`: one `onPictureReady` listener; it keeps `cover` paths by session id (and, from U4, `search` paths by external id). `Thumb` (in `HoverPreview.tsx`) shows `<img>` when it has a path and otherwise a neutral placeholder (`row-thumb-empty`, an icon, no `<img>`).
2. `SessionsPage`: `page` state (1); the data comes from `sessionsPage({filter, query, created_from: rangeStart(range), only_ids: reviewOnly ? waiting ids : null, page})`; the client-side `range` / `reviewOnly` row filters are removed (the backend applies them); the "To review" count comes from the research overview alone. Changing filter, search text, time filter or "To review" sets page 1; `sessions_changed` reloads the page shown; when the answer's `page` differs from the one asked (clamped), the state follows it. Selection, batch bar and the empty states are unchanged.
3. `Pagination` under the list, shown when `pages > 1`: numbered buttons (all when `pages ≤ 7`; else first, last, current ± 1 with gaps), the current one `aria-current="page"`, and Previous / Next.
4. `SessionRow` and `HoverPreview` take the row's picture as `pictures.cover(row.id) ?? row.cover_preview_path`; `GroupHeader` shows the group's `total`; `SessionsPanel` uses the same `Thumb`.
5. Tests for the rows.

### Criteria (2 criteria, 12 rows)
Boundary: `SessionsPage` (and `SessionsPanel` for H2·C10(e)) rendered in UI tests with `services/sessions.ts`, `services/events.ts` and the other services mocked (outcome = what is rendered and the exact list of `sessionsPage` calls).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H2·C9(a) | `SessionsPage` | `sessionsPage` answers page 1 of 3 with 25 rows | the 25 rows and a page control with `1` (current), `2`, `3`; with `pages: 1` no page control | `[rule]` 10H §5 (page numbers under the list) |
| H2·C9(b) | same | click `2` | `sessionsPage` called with `page: 2` and the same filter and query | `[rule]` 10H §5 |
| H2·C9(c) | same | on page 2: click the Published tab; then (again on page 2) type `oak` and wait for the debounce | each makes one call with `page: 1` and the new filter / `query: "oak"` | MC-H2.3; 10H §5 ("goes to page 1") |
| H2·C9(d) | same | on page 2: choose "Last 7 days"; then (again on page 2) press "To review" with sessions 4 and 9 waiting | calls with `page: 1` and `created_from` = the start of the local day six days ago; with `page: 1` and `only_ids: [4, 9]` | MC-H2.3 |
| H2·C9(e) | same | on page 2: a `sessions_changed` event | one call with `page: 2` | MC-H2.3 |
| H2·C9(f) | same | asked page 3, the answer says `page: 2`, `pages: 2` | `2` is the current page in the control | MC-H2.1 (clamped) |
| H2·C9(g) | same | a group `Today` with 5 rows and `total: 30` | its header shows 30 | `[rule]` MC-H2.1 |
| H2·C10(a) | `SessionRow` in the page | a row with `cover_preview_path: "/p/a.jpg"` | an `<img>` whose source is `previewUrl("/p/a.jpg")` | `[rule]` 10H §4 |
| H2·C10(b) | same | a row with no path (`cover_pending` true or false) | the placeholder; no `<img>` in the row's thumbnail | M-H4; 10H §4 (neutral placeholder) |
| H2·C10(c) | same | after (b): `picture_ready {kind: 'cover', session_id: <row>, path: "/p/c.jpg"}` | that row shows `previewUrl("/p/c.jpg")`; `sessionsPage` was not called again | MC-H1.3; 10H §4 ("an event, not a reload") |
| H2·C10(d) | `HoverPreview` | hover the row of (a); hover the row of (c) | the card's image is the same picture as the row's in both | `[rule]` 10H §4 (hover card) |
| H2·C10(e) | `SessionsPanel` | `listSessions` answers a row with a path and a row without | the first shows its `<img>`, the second the placeholder | `[rule]` 10H §4 (workspace sessions panel) |

### Delegated choices
The placeholder's look and the page control's styling (design handoff tokens) · whether `page` lives in component state or the URL hash (state is the smallest).

### Notes
- Hazard: `selectable` and "Select all" work on the rows shown, so on the page shown. Keep it; a selection is dropped when its row leaves the page (the existing effect at `SessionsPage.tsx:75-81`).
- Hazard: the other callers of `listSessions` (`Header.tsx:52`, `SessionsPanel.tsx:54`, `PickerDialogs.tsx:32`, `usePublish.ts:13`, `reviewQueue.ts:68`, `VisitBar.tsx:28`) stay on the unpaged list. Do not convert them (10H §10).
- Hazard: `rangeStart` must give exactly the bound `withinRange` uses (local calendar days), or the time filter shifts by a day.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/sessions src/features/workspace src/features/shell` · `npm run typecheck`

## Unit 4 — Search pictures, backend and UI

**Unit id:** `H2·U4` · **Rows:** H2·C11–C14 · **Changes app files:** yes

### Goal
`search_shop_pictures`, the search-picture cache with its 200 MB rule, `ItemDto.picture_path` / `session_id`, and the result tiles of New session and Assign product.

### Read first
Master plan §5 **MC-H2.4–6** · `crates/core/src/services/items.rs` (`search_local` :37-56, `search_shop` :58-101, `search` :104) · `crates/integrations/src/commerce/shopify/adapter.rs` (`item_search` :854-890, `item_find` :892-938: where `thumbnail_url` is filled) · `crates/core/src/domain/publish.rs` (`RemoteItem` :77) · `apps/desktop/src-tauri/src/commands/items.rs` (`ItemDto` :11-25 and its `From<Item>`, `items_search` :277, `items_search_shop` :294) · `crates/testkit/src/fake_commerce.rs` (`item_find` :327, products are `RemoteItem`s) · `crates/testkit/tests/suite/p18_item_search.rs` · `apps/desktop/src/features/sessions/NewSessionDialog.tsx` (:200-214) · `apps/desktop/src/features/workspace/PickerDialogs.tsx` (`ItemPickerDialog` :76-155) · `apps/desktop/src/features/sessions/__tests__/p18_item_search.test.tsx`.

### Files
New: `apps/desktop/src/features/sessions/ResultThumb.tsx`, `apps/desktop/src/features/sessions/__tests__/h2_search_pictures.test.tsx`, `crates/integrations/src/commerce/shopify/fixtures/h2_search_featured.json`. Edited: `crates/core/src/{mechanisms/remote.rs, services/items.rs, services/remote.rs}`, `crates/integrations/src/lib.rs` (the `search_cache_limit` argument), `apps/desktop/src-tauri/src/{lib.rs (`h2_tests`), commands/items.rs}`, `apps/desktop/src/services/{bindings.ts (generated), items.ts}`, `apps/desktop/src/features/sessions/{NewSessionDialog.tsx, usePictures.ts}`, `apps/desktop/src/features/workspace/PickerDialogs.tsx`, `crates/testkit/tests/suite/{h2_pages_and_search.rs, h1_shopify_reads.rs}`, `docs/guide/02_architecture.md`.

### Core mechanisms (the only functions with unit tests)
`mechanisms::remote::{search_name, evict}`.

### Ordered tasks
1. `search_name`, `evict` (master plan §4, MC-H2.6).
2. `ItemsService::search_shop_pictures(query) -> Vec<ShopResult>` (the body of `search_shop`, keeping `hit.thumbnail_url`); `search_shop` becomes its `.item` projection, so no existing caller changes.
3. `RemoteItems::item_picture(item_id) -> (Option<PathBuf>, Option<session_id>, pending)` (the row picture of the item's session) and `search_picture(external_id, url) -> Option<PathBuf>` (the cache file when it exists; otherwise the download is queued with source `Search`, MC-H2.5). `run_pending` writes search pictures with `fetch_image(url, SEARCH_EDGE)`, emits `PictureReady {Search}`, then applies `evict` to the folder with the service's `search_cache_limit` (MC-H2.6). `RemoteItems::new` gains `search_cache_limit` here (`SEARCH_CACHE_LIMIT_BYTES` in `App::build`; carried from H1's projection, L10).
4. Commands: `ItemDto` gains `picture_path: Option<String>`, `session_id: Option<i64>` (both `None` from every other command that answers an `ItemDto`). `items_search` fills them from `item_picture` and requests pending covers (`request_covers(Search, …)`); `items_search_shop` uses `search_shop_pictures` and `search_picture`.
5. Fixture `h2_search_featured.json` (schema-grounded; `source` says so): the request and response of `sv6_search_immediate.json` with the product's `featuredMedia` = `{"preview": {"image": {"url": "https://cdn.shopify.com/s/files/1/0000/0000/0000/files/f.jpg"}}}`.
6. `ResultThumb {item}`: the picture (`pictures.search(external_id)` or `pictures.cover(session_id)` or `item.picture_path`) when there is one, else the first letter as today. Used by both result lists (`NewSessionDialog.tsx:204`, `PickerDialogs.tsx:136`); the "Create" row keeps its plus tile.
7. Guide: `02_architecture.md` (the search-picture cache and its limit).
8. Tests for the rows.

### Criteria (4 criteria, 16 rows)
Boundary: `search_name` / `evict` (pure); the adapter over `FakeTransport` (H2·C12(a)); the command handlers on a `TestApp`-backed state (desktop tests `h2_tests`) or `ItemsService` + `RemoteItems` on `TestApp::with_fake_shop()`; the two dialogs in UI tests with services mocked.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H2·C11(a) | `items_search` | a local item whose session's main photo 1 has a preview file | its `picture_path` = that file, `session_id` = its session | MC-H2.4; 10H §5 |
| H2·C11(b) | same | a local item without a session | `picture_path` and `session_id` are null | MC-H2.4 |
| H2·C11(c) | same, then `run_pending` | an archived item without a cover file | `picture_path` null; after the run its cover file exists and `PictureReady {Cover {its session}}` was emitted | MC-H2.4; MC-H1.3 |
| H2·C12(a) | `ShopifyAdapter::item_search` | the transport replays `h2_search_featured.json` (schema-grounded) | the item's `thumbnail_url` = the fixture's URL (and stays `None` for `sv6_search_immediate.json`, captured with `featuredMedia: null`) | MC-H2.5; 10H §5 (`featuredMedia`) |
| H2·C12(b) | `search_shop_pictures` | a `FakeCommerce` product with `thumbnail_url` `U` not on this Mac | one result with `thumbnail_url: Some(U)` | MC-H2.5 (no longer dropped) |
| H2·C12(c) | `items_search_shop`, then `run_pending` | the product of (b) | first answer: `picture_path` null; after the run: the file `<previews>/remote/search/<search_name(U)>` exists, the fetch `(U, 160)` was recorded, `PictureReady {Search {external_id}}` was emitted; a second search answers `picture_path` = that file and records no fetch | MC-H2.5; 10H §5 (160 px, same cache) |
| H2·C12(d) | same | a product without `thumbnail_url` | `picture_path` null; no fetch after the run | MC-H2.5 |
| H2·C12(e) | same | the picture's URL answers `Gone` | no file, no event, no `media_urls` call | MC-H2.5 (dropped, no refresh) |
| H2·C13(a) | `evict` | `[("a", 120, 1), ("b", 60, 2), ("c", 50, 3)]`, limit 200 | `["a"]` | MC-H2.6; 10H §5 (oldest first) |
| H2·C13(b) | same | `[("a", 120, 1), ("b", 80, 2)]`, limit 200 | `[]` | MC-H2.6 (at the limit nothing goes) |
| H2·C13(c) | same | `[("a", 150, 1), ("b", 150, 1), ("c", 150, 2)]`, limit 200 | `["a", "b"]` | MC-H2.6 (name ascending on a tie) |
| H2·C13(d) | `run_pending` with a `RemoteItems` whose `search_cache_limit` is the size of two fake pictures | three search pictures written one after another (the clock advancing) | the folder holds the two newest files | MC-H2.6 |
| H2·C13(e) | `search_name` | `"https://cdn.shopify.com/a/f.jpg?v=1"` and `"https://cdn.shopify.com/a/f.jpg?v=2"` | the same name: the first 16 hex characters of the SHA-256 of `https://cdn.shopify.com/a/f.jpg`, plus `.jpg` | MC-H2.5 |
| H2·C14(a) | `NewSessionDialog` | `searchItems` answers an item with `picture_path: "/p/x.jpg"` and one without | the first result's tile is an `<img>` of `previewUrl("/p/x.jpg")`; the second shows its first letter | `[rule]` 10H §5 |
| H2·C14(b) | same | a shop result (`external_id` `E`) without a path; then `picture_ready {kind: 'search', external_id: E, path: "/p/s.jpg"}` | the tile changes from the letter to the picture | `[rule]` 10H §5 ("only while a picture loads") |
| H2·C14(c) | `ItemPickerDialog` | the inputs of (a) and (b), and a local result with `session_id` 7 followed by `picture_ready {kind: 'cover', session_id: 7, path}` | the same three outcomes in the Assign product dialog | `[rule]` 10H §5 (New session, Assign product) |

### Delegated choices
Tile size and fit (cover) · where `usePictures` is mounted for the dialogs.

### Notes
- Hazard: `search_shop` filters out shop hits that are already local or linked (`items.rs:75-84`); `search_shop_pictures` keeps exactly that filter.
- Hazard: the search folder is one flat folder under `remote/`; item folders are numeric ids, so `search` never collides.
- Hazard: `evict` runs on the files' modified times; write the file (rename) before listing the folder, and never evict the file just written unless it alone exceeds the limit.
- The captured search fixtures all have `featuredMedia: null` (the probe product had no picture); the non-null form is the document's own selection (`graphql.rs:11-12`) and the parsing at `adapter.rs:874-878` and `:927-929`.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-testkit --test suite h2_pages_and_search::` · `cargo test -p psm-testkit --test suite h1_shopify_reads::` · `cargo test -p psm-testkit --test suite p18_item_search::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/sessions src/features/workspace` · `npm run typecheck` · `cargo clippy -p psm-core -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-testkit --test suite h2_row_pictures::` · `cargo test -p psm-testkit --test suite h2_pages_and_search::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/sessions src/features/workspace` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: every path `list` and the searches hand out passed the existing-file rule; `page` and the search commands return without downloading; the only GraphQL call the queue makes is `media_urls` and it is behind `must_wait`; the six other `listSessions` callers are untouched; the existing Sessions-page tests changed only their mock, not their assertions. Notes go to `docs/implementation/backlog_archive/H2_list_pictures_pages_search.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
1. Open Sessions. Rows of products whose files were cleared ("Archived in Shopify") first show a grey placeholder and, a moment later, their first picture. No row shows a broken "?".
2. With more than 25 sessions, page numbers appear under the list. Go to page 2; type part of a title that is on page 1: it is found and the list is back on page 1.
3. Hover a row: the large card shows the same picture.
4. Press N and type part of a product's name: every result has a picture (this Mac's, or the shop's), or its first letter while the picture loads.
5. Quit and reopen: the pictures are there at once (they are kept on this Mac).

## Review log

- **2026-11-23, orchestrator: carried from H1's projection r0** (`handoffs/reviewer/handoff_H1_points_and_remote_record_projection_r0.md`). L8: U1 task 1, `refresh` keeps `cover_name(new cover)` (H1 deletes every `cover-*`). L9: the `FakeCommerce` additions `media_urls`/`url_reads`, `image_faults`/`fail_image`, `set_fetch_delay`/`max_fetches_in_flight` moved here (U1 task 6; `fake_commerce.rs` added to U1 Files). L10: `RemoteItems::new` gains `health` (U1 task 6) and `search_cache_limit` (U4 task 3; `crates/integrations/src/lib.rs` added to U4 Files). Rows and counts unchanged.
- **2026-11-23, implementer r1 (Claude Opus; checkpoints U1 `5e1a0d6`, U2 `a54ad20`, U3 `b9cb01f`, U4 `ba13aed`).**
  Built: the row-picture rule (`RowPictures::row_picture`, read once per answer by `RemoteItems::row_pictures` through a
  new one-query `Store::remote_covers`), the cover cache and queue (`request_covers`, `run_pending` downloading 4 at a
  time, the stale-URL refresh behind `must_wait` with the 10-minute cool-down, Health `remote_picture_gone`),
  `PictureReady` → `picture_ready`, Delete session removing the item's picture caches; `page_of`,
  `SessionsService::page`, `sessions_page`; the Sessions page on `sessionsPage` with page numbers, placeholder tiles and
  live covers; `search_shop_pictures`, `item_pictures` / `search_picture`, the 200 MB search cache with `evict`,
  `ItemDto.picture_path` / `session_id`, `ResultThumb` in New session and Assign product. Tests: one per row (H2·C1–C14,
  67 rows). Assumptions and deviations: see the handoff (main ones: the Store read `remote_covers` and the split of the
  list record's cover path into adjusted/original, in `ports/store.rs` and `infra/sqlite/store.rs` outside the listed
  files; search pictures queued in their own list so the local and shop searches do not replace each other's;
  `ItemDto`'s new fields are left out when empty (TS optional); the Sessions-page tests answer `sessionsPage` from their
  existing `listSessions` mock through `__tests__/pageFromList.ts`, so no assertion changed, and three more such tests
  than listed needed it: `p18_pending_state`, `p15_publish_ui`, `c4_visit`).
