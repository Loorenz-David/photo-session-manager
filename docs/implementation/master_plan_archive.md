# Archived and shop-only items editable — master plan (v1.2, 2026-11-23)

Short on purpose. It adds to `docs/implementation/master_plan.md` (roles, standing rules §3, registry §6, environment
§7) and wins over it on a name listed here. Process: `docs/process/flow.md` §10 (milestones of units), with the deltas
of §3 below. Planner: Claude (`pipeline-planner`), from book `10H_archived_items_editable.md` (RATIFIED 2026-11-23,
changelog A32; it amends 10F's read-only rule and Q3).

## 1. Goal
Build book `10H_archived_items_editable.md`: every row of the sessions list shows a picture (also for items whose
files this Mac cleared, from Shopify, fetched small and cached), the list has pages of 25, every product search result
shows a picture, an archived or shop-only item opens in the normal workspace with its Shopify pictures, the first
photo edit **brings the pictures back** (full-size JPEGs into the session folder and Capture One) after which the item
is an ordinary session whose **Publish sends only what changed**, and Details stay editable and publishable without
bringing anything back. Semantics live in 10H (and 10F for clearing, 10D for composition, 07A/07B for publish). §5
below is the part 10H leaves to planning: the code-level rules of its five contracts **MC-H1…MC-H5**, each with the
invariants **M-H1…M-H5** it serves. Where 10H and this file differ, 10H wins and this file is amended.

## 2. Milestones (one runner phase each; order in `docs/implementation/phases_archive.txt`)
| Id | Builds | Units | Criteria | Rows | Changes app files | Projection | depends_on | Useful on its own |
|---|---|---|---|---|---|---|---|---|
| `H1_points_and_remote_record` | the adapter reads Shopify's points after every answer and background work waits when they are low; typed "picture gone" downloads, full-size downloads, the batched URL read (`nodes(ids:)`), picture sizes in the readback; the record of an item's remote pictures (archived and shop-only), the first-open read, Refresh from Shopify, retention kept away from items without local files | 2 | 12 | 56 | yes | **yes** | — | nothing shows yet; the Mac knows which pictures Shopify holds for an archived or adopted product and never spends points in the background when they are low |
| `H2_list_pictures_pages_search` | the row-picture rule (never a missing file), the cover cache and its background fetch, stale-URL refresh; the paged sessions list (25 per page) behind a new command; page numbers and live row pictures in the UI; pictures on the product search results | 4 | 14 | 67 | yes | no | H1 | the sessions list shows a picture on every row and has pages; the New session and Assign product searches show pictures |
| `H3_bring_back` | JPEG photos in the photo pipeline (preview, frame, orientation); bring back (download all at full size, then import in one transaction, retire the cleared rows, link each photo to its Shopify media); the proof that a publish afterwards uploads only what changed; retention only after the next publish | 3 | 14 | 64 | yes | **yes** | H2 | nothing shows yet; the service can turn a remote item into an ordinary session and republish only changes |
| `H4_remote_workspace` | the remote view (pictures in the background, 4 at a time), activation that never moves the capture folder for a remote item, the details-only publish; the normal workspace for remote items with the note, the edit gate, the Bring back dialog and progress, Refresh from Shopify | 3 | 14 | 66 | yes | **yes** | H3 | the owner opens an archived or shop-only item like any session, edits its details, brings its photos back and republishes |
Totals (computed from the plan files by script): 4 milestones, 12 units, 54 criteria, 253 rows; every unit has at most 8
criteria. Each milestone needs the one before approved (its `depends_on`), gate = the **Tracker** line of that plan
file. Start each with `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase <id>` (the runner resolves any id as
`docs/implementation/plans/<id>.md`); `run_phase.sh all` reads `phases.txt`, not this list.

## 3. Process deltas (flow §10 applies; these add to it)
- **No mechanism-inventory gate.** §5 stands in for it (the orchestrator's instruction, as for the writer and
  conversations): one contract per mechanism 10H names (MC-H1…MC-H5), each resolved to code-level rules. 10H's
  measurement ledger is its §9 (**M-H1…M-H5**). Trace cells cite `M-H<n>`, `MC-H<n>.<item>` (the items of §5), a 10H
  section, a 10F/10D/07A rule, `M41`, or `H1`–`H8`. `[rule]` only for UI rendering rows.
- **Milestones of units** (flow §10): one implementer session works the units in order with a
  `CHECKPOINT (not approved):` commit after each, one review, one fix budget, one narrative per milestone. Reason for
  more than 8 criteria per milestone: as for research, the writer and conversations, a phase cycle per unit costs more
  than the work.
- **Implementer Claude Opus** (`PSM_IMPLEMENTER=claude`), reviewer Opus. **No plan sets `implementer:`**: flow.md log
  v1.3 (2026-11-08, "no new plan may carry `implementer: fable`") is the process authority (same decision as
  `master_plan_conversations.md` §3). If the owner lifts it, `H2` (units 3–4) and `H4` (unit 3) are the frontend-heavy
  parts.
- **Projection** for `H1` (every Shopify shape a test feeds the adapter: `extensions.cost`, `nodes(ids:)`, the
  readback's sizes, download statuses; rule 17), `H3` (bring back M-H2, the publish linkage M-H3) and `H4` (the
  capture folder must not move, the details-only publish must not touch media); not for `H2`.
- **Tests** (flow §4, §4A, §10): one behaviour test per row at the boundary it names; unit tests only for a plan's
  **Core mechanisms**. While working: the milestone's own stems and UI files. The full suite runs **once, at the end
  of each milestone** (all four change app files).
- **Rule 9, here:** no test, no session and no tool run ever calls Shopify or its CDN, and **no session launches the
  built app** (a launched build would read the owner's real shop credentials from `settings.json`). Adapter rows run
  the real `ShopifyAdapter` over `psm_testkit::fake_transport::FakeTransport`; service rows run over
  `FakeCommerce`. The manual try-out of each plan is the owner's or the orchestrator's, on the installed app.
- **Rule 17, here.** The Shopify shapes this track adds are grounded as follows, and a row's fixture cell names which:
  - `extensions.cost.throttleStatus {maximumAvailable, currentlyAvailable, restoreRate}`: **captured**: every P12
    fixture carries it (e.g. `crates/integrations/src/commerce/shopify/fixtures/sv4_job_poll_done.json`:
    `currentlyAvailable: 1999` an integer, `maximumAvailable: 2000.0` and `restoreRate: 100.0` floats), and
    shopify.dev "GraphQL Admin API rate limits" shows the same keys. Numbers are read as `f64`. The **throttled**
    reply (`errors[].extensions.code = "THROTTLED"` beside the top-level `extensions.cost`, H1·C1(f)) is
    schema-grounded from that page (no captured fixture has an `errors` body with a cost block); the code is the one
    `errors::is_throttled` already reads.
  - `nodes(ids: [ID!]!): [Node]!` (shopify.dev Admin GraphQL 2026-10 `queries/nodes`): the list has one entry per
    id, in order, **`null` for an id Shopify no longer has**. `MediaImage.image.url`, `MediaImage.originalSource
    {fileSize: Int, url}` (`objects/MediaImageOriginalSource`: the `url` there is "valid only for a short period", so
    the stored URL is always `image.url`), `GenericFile {url, originalFileSize: Int}` (`objects/GenericFile`). These
    are **schema-grounded, not captured**: new fixture files say so in their `source` field
    (`"schema-grounded: shopify.dev Admin GraphQL 2026-10 <pages>; not captured"`), and the orchestrator's live check
    after H4 is their first real contact. H1 adds `originalSource.fileSize` and `GenericFile.originalFileSize` to the
    existing `MEDIA_READBACK` that 10F's clearing uses, so that live check **includes one clearing verification (the
    readback)**, not only covers; a wrong field name would make every clearing report "Shopify could not be read". No
    build carrying H1 is installed before that check (H1 projection r0).
  - Download failures: the shape our code receives is `Download {status, bytes}` built by
    `ReqwestTransport::download` from the HTTP response (`transport.rs:193`, changed in H1·U1 to return the status
    instead of an error string); the adapter maps `403`/`404` to `Error::ImageGone`.
  - What a full-size download (`fetch_original`) sends back: **not grounded** (H3 projection r0, L6). Nothing records
    that the CDN answers JPEG bytes; `ReqwestTransport::get_file` sends no `Accept` header and a product made in the
    Shopify admin may hold PNG originals, while `image::open` picks its decoder from the file extension only. So
    MC-H4.3 refuses a body that does not start with `FF D8 FF` (row H3·C6(f)), and the orchestrator's live check after
    H4 records the first bytes and the content type of one full-size download made through `ReqwestTransport`.
  - Capture One and JPEG sources (does the dictionary find, sync and process a `.jpg` in a capture folder like a RAW):
    **not in the evidence doc today**. H3·U1 task 0 is the probe that records it in
    `docs/intention/evidence/capture_one_16.8.6.md`; the suite never depends on it (the fake accepts any path).
- **Row ids** `H<n>·C<k>(<row>)`, numbered once per milestone across its units; tests named after their row
  (`h1_c7a_<slug>`). Reviewer notes: `docs/implementation/backlog_archive/<milestone id>.md`.
- **Secrets.** No shop domain, client id/secret, token, real product id or real CDN URL in a test, fixture, log,
  handoff or commit; fixtures use `gid://shopify/MediaImage/<digits>` and `https://cdn.shopify.com/s/files/1/0000/0000/0000/files/<name>.jpg`
  placeholders.
- **Guide.** A milestone that changes a flow, table, command, event, cache path or rule updates `docs/guide/` in the
  same change (flow §7A); each plan carries it as a task. `docs/implementation/STATUS_archive.md` is the
  orchestrator's; `STATUS.md` and `master_plan.md` §4 are not used or edited by these milestones.
- **Join:** after H4 and the live check the orchestrator opens the PR into `main`.

## 4. Names (registry; exact)
**Code layout.** Core: `crates/core/src/domain/remote.rs` (vocabulary), `crates/core/src/mechanisms/remote.rs` (pure
rules, inline unit tests), `crates/core/src/services/remote.rs` (`RemoteItems`: record, covers, search pictures, view,
queue) and `crates/core/src/services/bring_back.rs` (`impl RemoteItems { bring_back }`). No name of Shopify, "CDN",
"GraphQL" or "points" in `crates/core` (H7): core says *remote picture*, *platform*, *call budget*. Adapter:
`crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, transport.rs}`. Storage: migrations
`crates/infra/src/sqlite/migrations/020_remote_items.sql` (H1) and `021_publish_details_only.sql` (H4), store methods
in `crates/infra/src/sqlite/store.rs`. JPEG previews: `crates/infra/src/rawpreview.rs` (`JpegSource`).

**Vocabulary** (`psm_core::domain::remote`, serde snake_case): `RemoteKind {Archived, ShopOnly}` · `RemoteState
{Local, Unknown, Remote(RemoteKind)}` (wire: `local`, `unknown`, `archived`, `shop_only`) · `CallBudget {available:
f64, maximum: f64}` · `PictureTarget {Cover {session_id}, Search {external_id}, Remote {session_id, role, position}}`
· `RemotePictureView {role, position, media_id, path: Option<PathBuf>, gone: bool}` · `RemoteView {item_id, state,
keep_files, pictures, bring_back_bytes: i64, check_failed: bool, bring_back: Option<BringBackProgress>}` ·
`BringBackProgress {done: usize, total: usize}` · `BringBackState {Running, Done, Failed}` · `BringBackOutcome
{photo_ids: Vec<i64>, synced: bool}` · `BringBackInput {session_id, item_id, files: Vec<BroughtBack>,
link_role_condition, at}` · `BroughtBack {role (as `ArchivedImage`'s), position, media_id, path, sha256}` (H3
projection r0, L12) · `SessionPage {groups: Vec<PageGroup {label, total, rows}>, counts,
unassigned_count, page: u32, pages: u32, total: usize}` (`psm_core::domain::session`) · `ShopResult {item: Item,
thumbnail_url: Option<String>}` (`psm_core::services::items`). Existing types that grow: `ArchivedImage` (**the
remote picture**; `domain/storage.rs:53`) gains `bytes: Option<i64>`; `MediaImage` (`domain/publish.rs:61`) gains
`bytes: Option<i64>`; `ItemRetention` (`domain/storage.rs:45`) gains `restored_at: Option<i64>`, `remote_checked_at:
Option<i64>`; `PublishItem` and `PublishPreviewItem` gain `details_only: bool`; `SessionRow` gains `cover_pending:
bool`; `RetentionInput` gains `shop_only: bool`, `restored_at: Option<i64>`. A remote picture whose stored `url` is
empty is **gone** (Shopify no longer has it).

**Constants** (`psm_core::mechanisms::remote`): `PAGE_SIZE = 25` · `PARALLEL_DOWNLOADS = 4` · `COVER_EDGE = 320` ·
`SEARCH_EDGE = 160` · `SEARCH_CACHE_LIMIT_BYTES = 200_000_000` · `URL_REFRESH_CHUNK = 25` · `BUDGET_FLOOR = 0.25` ·
`DOWNLOAD_RETRIES = 2` · `UNKNOWN_PICTURE_BYTES = 1_500_000` · `REFRESH_COOLDOWN_MS = 600_000`; the 1600 px view
size stays `services::retention::ARCHIVED_IMAGE_EDGE` (`retention.rs:25`).

**Pure rules** (`mechanisms/remote.rs`; each is a plan's Core mechanism): `must_wait(Option<CallBudget>) -> bool` ·
`remote_state(RemoteInput {cleared, linked, adopted, photo_rows, checked, pictures}) -> RemoteState` ·
`cover_name(media_id) -> String` (`cover-<first 12 hex of sha256(media_id)>.jpg`) · `search_name(url) -> String`
(`<first 16 hex of sha256(url without its query)>.jpg`) · `page_of(total, page) -> (page, pages, range)` ·
`evict(entries: &[(name, bytes, modified_ms)], limit) -> Vec<name>` · `bring_back_names(folder_key, gallery: usize,
condition: usize, taken: &dyn Fn(&str) -> bool) -> Vec<String>` · `estimate_bytes(&[ArchivedImage]) -> i64` ·
`row_label` for remote rows lives in `SessionsService` (§5 MC-H3.7). In the adapter: `projected(available, maximum,
restore_per_s, elapsed_s) -> f64` (`commerce/shopify/adapter.rs`, inline unit test).

**Ports.** `CommerceAdapter` (`crates/core/src/ports/commerce.rs`) gains three read-only methods, none taking a
`ConfirmedRun`: `media_urls(&self, media_ids: &[String]) -> Result<Vec<Option<MediaImage>>>` (one entry per id, in
order; `None` = the platform no longer has it; default: `Err(Provider("not supported"))`), `fetch_original(&self, url:
&str) -> Result<Vec<u8>>` (the image at full size; default as above), `budget(&self) -> Option<CallBudget>` (default
`None` = never wait). `fetch_image` and `fetch_original` answer `Err(Error::ImageGone)` when the platform says the
picture is not there. `Store` gains (each "one transaction" of §5 is one method): `set_remote_record(item_id, images:
&[ArchivedImage], checked_at)`, `update_remote_urls(item_id, &[(media_id, Option<url>, Option<bytes>)])` (`None` url
stores the empty string), `remote_input(session_id) -> RemoteInput` (one read), `bring_back(BringBackInput) ->
Vec<i64>`; `mark_item_cleared` and `archived_images` (`store.rs:3236`, `:3255`) carry `bytes`; `item_retention`
(`store.rs:3210`) the two new times. `HttpTransport::get_bytes` (`transport.rs:54`) is replaced by `download(&self,
url: &str) -> Result<Download {status: u16, bytes: Vec<u8>}>` (a non-200 answer is a `Download`, not an error).

**Storage.** `020_remote_items.sql`: `ALTER TABLE item_retention ADD COLUMN restored_at INTEGER; ALTER TABLE
item_retention ADD COLUMN remote_checked_at INTEGER; ALTER TABLE archived_images ADD COLUMN bytes INTEGER;`.
`021_publish_details_only.sql`: `ALTER TABLE publish_items ADD COLUMN details_only INTEGER NOT NULL DEFAULT 0;`. Each
with its `M::up` line in `SqliteStore::migrations()` (`store.rs:67`). `archived_images` keeps its name and is the
table of remote pictures for both kinds. No new table, so "Delete session" (`session_delete.rs:256-257` already clears
`item_retention` and `archived_images`) needs no new entry. **The rewind test:**
`crates/testkit/tests/suite/p27_retire_shape_on_arrival.rs:354-368` drops every `ALTER`-added column before it rewinds
`user_version`; each new column gets its `DROP COLUMN` line there in the milestone that adds it, or that test fails
with "duplicate column name".

**Caches** (under `<data>/previews/`, already readable by the webview): `remote/<item_id>/cover-<12 hex>.jpg` (one
file per item, 320 px) · `remote/search/<16 hex>.jpg` (160 px, at most 200 MB, oldest first) ·
`archived/<item_id>/<role>-<NN>-<12 hex>.jpg` (1600 px; unchanged from P62, `retention.rs:180`). All written to
`<name>.part` and renamed.

**Service.** `RemoteItems` (`App.remote: Arc<RemoteItems>`, also in `AppHandles`; always present, H6: without a
commerce integration every item is `Local` and nothing is fetched). Constructor: `RemoteItems::new(store: Arc<dyn
Store>, commerce: Option<Arc<dyn CommerceAdapter>>, editing: Arc<dyn EditingProvider>, previews_root: PathBuf,
capture_root: Arc<dyn Fn() -> Result<PathBuf> + Send + Sync>, clock, events, health, history: Arc<History>,
search_cache_limit: u64)` is the final form; each milestone takes only the arguments it reads (no unused field under
`-D warnings`): H1 `(store, commerce, previews_root, clock, events)`, H2 adds `health` and `search_cache_limit`, H3
adds `editing`, `capture_root`, `history`. The `capture_root` closure `App::build` passes calls
`editing.inspect_document()` and canonicalizes its `capture_root`, **uncached** (bring back is a press;
`ArrivalService::capture_root()` caches its first answer forever; H3 projection r0, L4). Methods: H1 `state(session_id)`, `ensure_record(session_id)` (the first-open read),
`refresh(session_id)`, `run_pending() -> Pending {Idle, Waiting, Worked}` · H2 `request_covers(source, session_ids)`,
`row_picture(record) `, `item_picture(item_id)`, `search_picture(external_id, url)` · H3 `bring_back(session_id)` · H4
`view(session_id)`. **Background:** one thread `remote_background` in `App::build` (only when `run_startup`, like
`storage_background`, `crates/integrations/src/lib.rs:1163`) calls `run_pending` and sleeps 250 ms on `Idle` or
`Waiting`; `shutdown` joins it with `BACKGROUND_JOIN_BOUND`. One `run_pending` takes the current batch of queued
downloads and runs them at most `PARALLEL_DOWNLOADS` at a time (`std::thread::scope`), then the batch's URL refresh
and retries (MC-H1.4), then queued first-open reads. Tests call `run_pending` themselves (`TestApp` never starts the
thread). `SessionsService` gets `with_remote(&RemoteItems)`.

**Errors** (`psm_core::Error` → `CommandError.code`): `ImageGone` → `image_gone` · `NotRemote` → `not_remote` ·
`BringBackRunning` → `bring_back_running` · `BringBackFailed {reason}` → `bring_back_failed` (message `Could not
bring back the photos: <reason>`). `FilesCleared` (`files_cleared`) stays for an archived item's photo operations.
**Health code:** `remote_picture_gone`, detail `<n> pictures are no longer in the shop: <SKU>, <SKU>` (`1 picture is
no longer in the shop: <SKU>`). Core's sentences say *the shop*, never the platform's name (H7: the debt of
`services/storage.rs:844-852` is not extended); the words that name Shopify are the frontend's.

**Events.** `AppEvent::PictureReady {target: PictureTarget, path: String}` → window event `picture_ready {kind:
'cover' | 'search' | 'remote', session_id?, external_id?, role?, position?, path}` (listener `onPictureReady` in
`apps/desktop/src/services/events.ts`) · `AppEvent::BringBack {session_id, done, total, state: BringBackState, error:
Option<String>}` → `bring_back_progress` (listener `onBringBack`). Existing `SessionChanged` is emitted when an item's
remote state or record changes.

**Commands** (registered in `specta_builder()`): H2 `sessions_page(filter, query, created_from, only_ids, page)`
(`commands/sessions.rs`; `sessions_list` stays for the other callers), `items_search` / `items_search_shop` answer
`ItemDto` with `picture_path: Option<String>` and `session_id: Option<i64>` · H4 `session_remote_get(session_id)`,
`session_remote_refresh(session_id)`, `session_bring_back(session_id)` in a new
`apps/desktop/src-tauri/src/commands/remote.rs`; `item_archive_get` is removed and `item_keep_files_set` answers the
`RemoteViewDto`. Wrappers: `apps/desktop/src/services/remote.ts`; `services/sessions.ts` (`sessionsPage`).

**Words** (exact). Frontend: `Photos are on Shopify. Editing a photo brings them back to this Mac first.` · `Bring back
now` · `Bring back <N> photos from Shopify to edit them? About <X> MB.` (`Bring back 1 photo from Shopify to edit it?
About <X> MB.` for one) with `Bring back` / `Cancel` · `Bringing back <k> of <N> photos…` · `Try again` · `Archived in
Shopify` (the row label, both kinds) · `Refresh from Shopify` · `Checking Shopify…` · `Could not check Shopify for
this product's photos.` · `Details only: the photos stay as they are on Shopify.` and the button `Publish details` (publish confirmation). `<X>` = the
estimate in MB rounded to a whole number, at least 1. Core (the `BringBackFailed` message is `Could not bring back the
photos: <reason>`): reasons `<n> photos could not be downloaded` (`1 photo could not be downloaded`) · `<n> photos
are no longer in the shop` (`1 photo is no longer in the shop`) · `the shop could not be reached` · `the capture
folder is not known yet` · `wait until the publish has finished`.

**Log lines.** `remote.record item= kind= pictures=` · `remote.cover item= outcome=` · `remote.urls_refreshed asked=
found= gone=` · `remote.bring_back item= photos= bytes= outcome=` · `remote.budget_wait available= maximum=` (when a
background step waits; logged by core, so it names no platform, H7). Never a URL with a query, never a token.

**Frontend.** H2: `features/sessions/{Pagination.tsx, usePictures.ts, ResultThumb.tsx}`. H4:
`features/workspace/remote/{remotePhotos.ts (pure), useRemoteItem.ts, RemoteNote.tsx, BringBackDialog.tsx}`;
`ArchivedView.tsx` and its test `p62_archived_ui.test.tsx` are removed.

**Tests.** Suite stems `crates/testkit/tests/suite/{h1_shopify_reads.rs, h1_remote_record.rs, h2_row_pictures.rs,
h2_pages_and_search.rs, h3_jpeg_photos.rs, h3_bring_back.rs, h3_publish_after_bring_back.rs, h4_remote_workspace.rs,
h4_details_only_publish.rs}` (each with its `mod` line in `main.rs`); desktop command tests in
`apps/desktop/src-tauri/src/lib.rs` (`h2_tests`, `h4_tests`); UI `features/<f>/__tests__/h<n>_<slug>.test.tsx`.
`FakeCommerce` grows, each addition in the milestone whose rows first call it (charter rule 4: no unused
scaffolding): **H1** per-media `bytes` (`set_media_bytes`, gallery media and condition files alike), `budget:
Option<CallBudget>` (`set_budget`), `media_readback` answering `url: None` for a gallery media whose status is
`Processing`; **H2** `media_urls` with `url_reads: Vec<Vec<String>>`, `image_faults` (per URL: a queue of `Gone` /
`Fail` answers, `fail_image`), `set_fetch_delay(ms)` with `max_fetches_in_flight()`; **H3** `fetch_original` with
`original_fetches: Vec<String>`, images served as real JPEGs (`SyntheticJpeg`) when a test registers them
(`set_image(media_id, bytes)`); `fake_image_bytes` stays the default.

## 5. Contracts (stand in for the mechanism inventory; plans implement them verbatim)

Notation: *record* = the item's rows in `archived_images` (its remote pictures, gallery then condition, each in
position order); *cover picture* = the record's gallery picture with the lowest position; *press* = work started by a
command the photographer triggered; *background* = work run by `run_pending` or the storage scan.

### MC-H5 — The call budget (10H §8)
1. **Read.** After every GraphQL answer whose body carries `extensions.cost.throttleStatus`, the adapter stores
   `maximumAvailable`, `currentlyAvailable`, `restoreRate` (each read as `f64`) and the monotonic time. An answer
   without it leaves the stored value as it was.
2. **Estimate.** `budget()` = `None` before the first reading, else `{available: projected(...), maximum}` with
   `projected = min(maximum, currentlyAvailable + restoreRate × seconds since the reading)`.
3. **Wait rule.** `must_wait(b)` ⇔ `b = Some` ∧ `maximum > 0` ∧ `available < BUDGET_FLOOR × maximum`. Background
   steps that make a GraphQL call check it first: the URL refresh (MC-H1.4) and the queued first-open read (MC-H3.2)
   are left in the queue (`run_pending` answers `Waiting`); 10F's verification (`storage.rs:827 clear_item`) is not
   started for any further item of that scan and the scan is requested again. Nothing sleeps while holding a lock.
4. **A press is never delayed:** `refresh`, `bring_back` (its own URL refresh and, for an `Unknown` item, its read),
   publish, search and every existing command call the adapter without consulting `must_wait`.
5. **Calls this chapter adds** are exactly: `media_urls` (one request per `URL_REFRESH_CHUNK` ids) and
   `media_readback` (first-open read, Refresh from Shopify). Listing a page, showing a row, searching and downloading
   pictures add no GraphQL call (downloads go to the CDN).

### MC-H1 — The row picture (10H §4)
1. **Rule** (`row_picture`, used by `list`, `page` and the item search): (i) the session's main photo at position 1:
   its `adjusted_preview_path` if that file exists, else its `original_preview_path` if that file exists; (ii) else,
   when the item's state is `Remote(_)` and its cover picture is not gone, the cover cache file if it exists; (iii)
   else none. `cover_pending` ⇔ (ii) applied but the file does not exist. **No path whose file is missing is ever
   returned** (M-H4), in the paged and the unpaged list alike.
2. **Cover file.** `<previews>/remote/<item_id>/<cover_name(media_id)>`, from `fetch_image(url, COVER_EDGE)`. After a
   cover is written, every other `cover-*.jpg` in that folder is deleted.
3. **Fetching.** `page` and the searches hand `request_covers(source, sessions)` the pending rows of what they return
   and return at once. A new request from the same `source` (`Page`, `Search`) replaces that source's not-yet-started
   downloads. Downloads run only in `run_pending`, at most `PARALLEL_DOWNLOADS` at once. Each finished cover emits
   `PictureReady {Cover {session_id}, path}`.
4. **Stale URLs.** A download that answers `ImageGone` marks its picture *stale* for this batch. After the batch's
   downloads, when the budget allows (MC-H5.3): the stale pictures' media ids, without those refreshed less than
   `REFRESH_COOLDOWN_MS` ago (monotonic, in memory), go to `media_urls` in chunks of `URL_REFRESH_CHUNK`; for each
   answer `Some {url: Some(u), bytes}` the stored URL (and bytes) is updated and the download retried once; `None` or
   a missing URL stores the picture as **gone** (empty URL). A gone picture is never downloaded or refreshed again
   until its record is replaced. A page of 25 rows therefore costs at most one `media_urls` request. Any other
   download error is logged and neither refreshed nor retried in that batch.
5. **Health.** The service keeps, in memory, the items with gone pictures (`item → n`); non-empty → Health
   `remote_picture_gone` with the count and SKUs, empty → cleared. An item leaves the map when its record is replaced
   (first-open read, Refresh from Shopify, clearing), it is brought back, or its session is deleted.
**Invariant.** M-H4 for every list and search answer; no mutating adapter method is called by any step of MC-H1
(M-H1).

### MC-H2 — Pages and search pictures (10H §5)
1. **Page.** `SessionsService::page(filter, query, created_from: Option<i64>, only_ids: Option<&[i64]>, page)`: the
   rows of `list(filter, query)` (`sessions.rs:408`), kept when `created_at ≥ created_from` (when given) and `id ∈
   only_ids` (when given), in `list`'s order (newest created first, id descending on a tie); `total` = their number;
   `pages = max(1, ceil(total / PAGE_SIZE))`; the requested page is clamped to `1..=pages`; the rows are those at
   indexes `(page−1)×25 .. page×25`. Groups are `list`'s groups cut to those rows, each with `total` = its row count
   before the cut; empty groups are left out. `counts` and `unassigned_count` are `list`'s (over all sessions).
2. **Callers.** Only the Sessions page uses `sessions_page`. `sessions_list` is unchanged in shape for `Header.tsx:52`,
   `SessionsPanel.tsx:54`, `PickerDialogs.tsx:32`, `usePublish.ts:13`, `reviewQueue.ts:68`, `VisitBar.tsx:28`; its
   rows follow MC-H1.1 but it requests no cover.
3. **UI.** Changing the filter, the search text, the time filter or "To review" asks for page 1; a `sessions_changed`
   event re-asks for the page shown. The time filter is sent as `created_from` (the start of its first local day, the
   existing `withinRange` rule of `lib/format.ts:39`), "To review" as `only_ids` (the ids the research overview says
   wait for review).
4. **Search, local results:** `ItemDto.picture_path` = the row picture of the item's session (none without a
   session), `session_id` its session; a pending cover is requested (`source = Search`).
5. **Search, shop results:** `search_shop_pictures` keeps `RemoteItem.thumbnail_url` (today dropped at
   `items.rs:86-97`). `ItemDto.picture_path` = `<previews>/remote/search/<search_name(url)>` if it exists; otherwise
   the download `fetch_image(url, SEARCH_EDGE)` is queued (`source = Search`) and emits `PictureReady {Search
   {external_id}, path}`. A failed search-picture download is dropped (no refresh: the next search carries a fresh
   URL).
6. **Eviction.** After each search-picture write: when the folder's files total more than the limit, `evict` names
   the oldest (by modified time, name ascending on a tie) until the total is at most the limit; they are deleted.

### MC-H3 — Remote items in the workspace (10H §3, §6)
1. **State** (`remote_state`): `cleared` → `Remote(Archived)`; else `adopted ∧ linked ∧ photo_rows = 0` (the item's
   `origin` is `linked`, it has a link for the integration, its session has no photo row of any classification):
   `¬checked` → `Unknown`, `checked ∧ pictures ≥ 1` → `Remote(ShopOnly)`, `checked ∧ pictures = 0` → `Local`; else
   `Local`. A session without an item, and every item when no integration is configured, is `Local`.
2. **First-open read** (`ensure_record`): for an `Unknown` item, under one process-wide lock (so two callers make one
   read): `media_readback(external id)`; when every picture has a URL, one transaction replaces the record (gallery
   in the answer's order, positions 1…, then condition) and sets `remote_checked_at`; then `SessionChanged`. A picture
   without a URL (still processing) or a failed read writes nothing (the item stays `Unknown`; `view` reports
   `check_failed`). `view` queues this read for an `Unknown` item (background, MC-H5.3); `bring_back` runs it at once.
3. **Refresh from Shopify** (`refresh`, a press): `Remote(_)` only (else `NotRemote`); the same read, the record
   replaced (`cleared_at` untouched), then cache files of the item that no longer match the record are deleted
   (`remote/<item_id>/cover-*` not the new cover's, `archived/<item_id>/*` not named by a current picture). In H1,
   before `cover_name` exists, every `cover-*` is deleted; H2 narrows it to keep `cover_name(new cover)`.
4. **Activation.** `SessionsService::activate` (`sessions.rs:201`) of a session whose state is `Remote(_)` answers
   `Ok` with **no** editing call, no camera target change and no change of the active session. Every other state
   activates as today. The frontend (`useCaptureTarget`, `App.tsx:95`) calls `activateSession(target)` only when the
   target's remote state is `local`, or `unknown` with `check_failed`; it calls it once when the state becomes one of
   those while the session is still the capture target (after bring back, or when the first-open read finds no
   pictures or fails). Leaving a session behaves as today.
5. **View** (`view`): for `Remote(_)`: the record's pictures, each with its cache file if it exists
   (`archived/<item_id>/…`, `ARCHIVED_IMAGE_EDGE`), `bring_back_bytes = estimate_bytes(record)` (`bytes` or
   `UNKNOWN_PICTURE_BYTES` each); missing files are queued (`source = View(session_id)`, same queue, limit, stale
   rule and `PictureReady {Remote {…}}` as MC-H1.3–4, with the item's stale ids in chunks of 25). **The command
   never downloads** (the P62 download inside `ArchiveService::view`, `retention.rs:195-240`, goes).
   `SessionsService::view` (`sessions.rs:348`) of a `Remote(_)` session answers **no photos** (an archived item's
   cleared rows, whose preview paths name deleted files, never reach the screen: M-H4); the conversations'
   attachable photos (`services/conversations.rs` `link_preview`) carry a preview path only when its file exists.
6. **Details-only publish.** For a session whose state is `Remote(_)`: `preview` answers no photos, no announced
   removals, `details_only = true`, and makes no platform read for media; `confirm` (the P62 refusal at
   `publish.rs:755-760` goes) freezes `frozen_main = []`, `frozen_condition = []`, `announced_removals = []` and
   persists `details_only = 1`. A run of a `details_only` item: Prepare stages no export, Upload uploads nothing, and
   Update **skips the gallery block** (`publish.rs:1733-1804`: no reconcile, delete, reorder or readback) **and the
   condition-photos value** (never part of the attribute writes, even when the frozen attributes hold its key); every
   other write (title, type, tags, description, attributes, search texts) is as for any publish. `details_only` is
   decided at confirmation and never recomputed by a run. An `Unknown` item is confirmed as today.
7. **Rows of remote items** (`list` and `page`): `main_count` / `condition_count` = the record's gallery / condition
   picture counts, `unreviewed_count = 0`, the status input's missing-preview count 0; the label is the publish
   label while the latest publish is running, held, failed or cancelled, else `Archived` (both kinds).
8. **Guards.** A media-list attribute write is refused with `FilesCleared` for `Remote(_)` (today: cleared only,
   `attributes.rs:793-803`). The photo guards of a cleared item (`retention.rs:30-43` and their callers) stay.
   Retention (`mechanisms/retention.rs:40`) answers `Never` for an item whose state is `Remote(ShopOnly)`.
**Invariant.** No step of MC-H3 other than a confirmed publish calls a mutating adapter method (M-H1); a
`details_only` run calls none of `staged_uploads_create(_many)`, `upload`, `product_create_media(_many)`,
`file_create(_many)`, `product_delete_media`, `product_reorder_media`; opening or viewing a remote item never changes
the active session.

### MC-H4 — Bring back (10H §7)
1. **Preconditions** (in order): an integration (`NoIntegration`); state `Remote(_)` — an `Unknown` item is read
   first (MC-H3.2) — else `NotRemote`; no bring back of this session running (`BringBackRunning`); the session's
   latest publish is not in `Held`/`Prepare`/`Upload`/`Update`/`Describe` (`BringBackFailed {wait until the publish
   has finished}`); the capture root is known (`BringBackFailed {the capture folder is not known yet}`); no picture of
   the record is gone (`BringBackFailed {<n> photos are no longer in the shop}`).
2. **Names.** `bring_back_names(folder_key, gallery, condition, taken)`: `<folder_key>-main-<NN>.jpg` and
   `<folder_key>-condition-<NN>.jpg` (`NN` = position, two digits at least); `taken(name)` ⇔ a photo row of any
   session has the canonical `<capture root>/<folder_key>/<name>` as `raw_path` (a store error is kept and fails the
   bring back before any download; the names still end). If any name of the set is taken, the whole set carries the suffix `-b2`
   before `.jpg` (`…-main-01-b2.jpg`), then `-b3`, … (the smallest for which none is taken). The folder is `<capture
   root>/<folder_key>/` (created when missing).
3. **Download.** Each picture: `fetch_original(url)` to `<name>.part`, renamed to `<name>` (replacing a leftover file
   of an interrupted attempt, which by rule 2 no photo row names); at most `PARALLEL_DOWNLOADS` at once; a failed
   download is tried `DOWNLOAD_RETRIES` more times; an `ImageGone` first goes through one `media_urls` for the item's
   stale ids (a press: no budget wait) and continues at the new URL with the full 1 + `DOWNLOAD_RETRIES` tries (the
   `Gone` attempt not counted); an answer `None` or without a URL stores that picture as gone (MC-H1.4) and the bring
   back fails with `<n> photos are no longer in the shop`. A downloaded body that does not start with the JPEG marker
   `FF D8 FF` counts as a failed download (same retries; rule 17). Progress: `BringBack {Running, done, total}`
   after each file. **If any picture is still missing:** every file this attempt wrote is deleted, nothing is
   imported, the item stays remote, `BringBack {Failed, error}` and `BringBackFailed`. A `Store::bring_back` error
   after every download succeeded gets the same clean-up and event and is returned unchanged.
4. **Import** (`Store::bring_back`, one transaction, only when every file is on disk):
   (a) every photo row the session had: `classification = deleted`, `position` null, the three preview paths null,
   `integrity = raw_missing`; its `photo_links` rows for the item are deleted; its `photo_places` row is kept with
   `released = 1` (created so when missing), so the retired shot leaves a template's shot count as a released deleted
   shot does (10G §6.4) and leaves no container to refill; the row itself stays;
   (b) one photo per picture, gallery then condition, in position order: `classification` main / condition,
   `position` 1…, `raw_path` = the **canonical** path of its file (as `arrival_insert`, `store.rs:2447`: Capture One
   edit events find a photo by its canonical path), `raw_hash` = the file's SHA-256, `source = import`, `captured_at =
   created_at = now`; its composition `kind normal, shape original, orientation named, quarter_turns 0, flip none,
   straighten 0, rect (0, 0, 1, 1), origin default, revision 0`; a `raw_state` row; the jobs `original_preview` and
   `render_preview` (as `store.rs:2496-2501`, without `hash`); in a session whose item type has a shooting template,
   the main photo at position *k* is placed by hand (`place_by_hand = 1`) with `slot` = the `final_no` of the
   container whose `final_no = k` (null past the last container), so the published order is kept and an arrival
   never takes a published photo's place (10G §4.4, §6.1);
   (c) one `photo_links` row per new photo: role `gallery` or `attribute:<commerce.condition_photos_key()>` (as
   `SessionsService::view`), slot `single`, `external_id` = the picture's media id, `upload_state = ready`,
   `revision 0`, `retouch_gen` = the photo's `raw_state.retouch_gen` as stored in this transaction (a row for the path
   may pre-exist; never the literal 0), `composition_rev 0`, no superseded ids;
   (d) `item_retention`: `cleared_at` null, `restored_at = now` (the record rows stay: they are ignored while the item
   is `Local` and replaced at the next clearing);
   (e) **`sessions.change_seq` is not changed** (bringing back is not an edit: a published item stays published).
5. **Around and after the transaction:** the session's undo and redo lists are emptied by
   `History::retire_session(session, retired photo ids, act)` (`history.rs:309`), with `Store::bring_back` run
   **inside** its `act` (under the undo lock, so no undo step naming a retired photo applies in between); after it: `editing.sync_folder(folder)` (`ports/editing.rs:58`;
   a failure is logged and reported as `synced = false`, never undone); `BringBack {Done}`, `SessionChanged`. No style
   is applied and no colour tag is mirrored for the new files.
6. **JPEG photos.** The `original_preview` job of a source whose extension is `jpg`/`jpeg` (`lib.rs:1573`) writes the
   JPEG scaled to at most 1600 px on its long edge (never enlarged) as the original preview, stores its EXIF
   orientation (none → null) and adopts the frame = its pixel size, width and height swapped for orientations 5–8,
   with `FrameSource::Camera` (Capture One's measurement replaces it by the P58 rule, `store.rs:363-384`). Every other
   extension keeps the embedded-JPEG path. The watcher's candidate rule (`mechanisms/arrival.rs:12`) is unchanged: a
   `.jpg` in a capture folder is never an arrival.
7. **Publish afterwards** needs no new publish code: an untouched photo's link is `ready` at its generation and
   revision, so `replacement_candidates` (`mechanisms/publish.rs:40`) leaves it and Upload skips it
   (`publish.rs:1495`); a changed composition (`revision > composition_rev`) or a Capture One edit (`retouch_gen`
   raised by `adjusted_event`, `store.rs:1921`) replaces its media through `superseded_ids` (`publish.rs:1247-1264`);
   a hidden photo's media is unwanted and own, so reconciliation removes it; the links make a shop-only item's media
   **own** (`publish.rs:1749-1774`), so it is never announced or refused as foreign. Capture One writing a first
   settings file for a brought-back JPEG is not an edit (`previews.rs:810-814` adopts the first fingerprint). While
   the session is published and unchanged the sidecar check does not look at it at all (`PreviewScope::Live`,
   `store.rs:592`, A16), so that clause bites only once something else in the session changed (row H3·C10(b)).
8. **Retention after bring back.** `eligibility` answers `Never` while `restored_at` is set and `published_at ≤
   restored_at` (no publish completed since the bring back). The next completed publish makes the item eligible again
   by 10F's rule; its clearing replaces the record with the verified images.
**Invariants.** M-H2: after `bring_back` returns or the process dies at any point, the session has either all N new
photos or none, and neither the platform nor Capture One's catalogue was asked to delete anything. M-H1: `bring_back`
calls no mutating adapter method. M-H3: for a brought-back item, the uploads of the next publish are exactly the
outputs of photos whose composition revision or retouch generation changed since the bring back, plus photos added
since. M-H5: "Delete session" of any item of this chapter makes no adapter call.

## 6. Delegated choices (decided here; each is one sentence in a narrative's "Choices made for you")
- **A shop-only product gets its Shopify pictures the first time its session is opened** (one read); until then its
  row shows the placeholder.
- **If Shopify cannot be read at that first opening,** the session behaves as it does today (the camera's folder
  follows it) and says "Could not check Shopify for this product's photos." with Try again; the alternative would send
  new photos to another product without telling you.
- **A shop product is "shop-only" only while its session has no photo of its own.** Assigning Unassigned photos to it,
  or photos arriving in its folder, make it an ordinary session, as today.
- **"About X MB" is exact when Shopify told the app the sizes** (items read or cleared from now on) **and assumes
  1.5 MB per photo otherwise** (10F's measured output size).
- **The brought-back files are named after the product's folder** (the SKU made safe for a file name), and a second
  bring back of the same product adds `-b2` to the names, so no earlier photo is ever overwritten.
- **Bringing photos back does not mark the product as changed:** a published product stays "Published" until you edit
  something, and it is not cleared again before its next publish.
- **Bringing back empties that session's undo list** (the steps before it name photos that no longer exist).
- **A brought-back photo is not given the house style again** (the published picture already has its look).
- **Both kinds of remote product carry the "Archived in Shopify" label,** except while a publish of theirs is
  running or failed. An adopted shop product you never published from this Mac is listed under Open as before.
- **Publishing a remote product sends its details only;** the confirmation says so, and the pictures and the damage
  report photos on Shopify are left exactly as they are.
- **Refresh from Shopify is offered for remote products only** (an ordinary session's photos are on this Mac).
- **A picture Shopify no longer has** shows the placeholder, is reported once on Health, and is not asked for again
  until Refresh from Shopify.
- **A search result's picture that fails to download** keeps its letter tile (the next search carries a fresh
  address).
- **"Bring back now"** in the note line starts the same prompt without an edit (the way to Capture One edits, which
  cannot start inside the app).
- **Page numbers** show every page when there are at most 7, otherwise the first, the last and the ones around the
  current page.
- **The JPEG's EXIF orientation is honoured**; a brought-back photo's frame is its pixel size until Capture One has
  measured it.
- **Research and "Arranged"** are unchanged: they need photos on this Mac, so they act on a remote product only after
  bring back.

## 7. Environment pointers
Commands and caveats: `master_plan.md` §7, `docs/guide/05_working.md`. While working: `cargo test -p psm-core --lib
mechanisms::remote` · `cargo test -p psm-integrations --lib commerce::` · `cargo test -p psm-infra --lib rawpreview` ·
`cargo test -p psm-testkit --test suite <stem>::` · `cargo test -p psm-desktop` (regenerates
`apps/desktop/src/services/bindings.ts` through `export_bindings`; never hand-edit) · `cd apps/desktop && npx vitest
run <files>` · `npm run typecheck` · `cargo clippy -p <crate> --all-targets -- -D warnings` · `cargo check
--workspace`. One filter per `cargo test` invocation. Closing gate of every milestone (once): `cargo test --workspace`
· `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run
typecheck` · `npm test`. One cargo command at a time, never in the background, never piped. Known intermittent tests:
`docs/guide/04_change_impact.md` §Fragile spots. No new dependency is expected (`sha2`, `image`, `kamadak-exif` are
already in the crates that need them). Migration numbers: `020` and `021` are the next free files in
`crates/infra/src/sqlite/migrations/` at `7f839cc`; if another change takes one first, use the next number (never
renumber an applied file). Checked at `7f839cc`: every existing path this plan set cites resolves (script), and the `file:line`
citations were read in the code at that commit; the lines move as milestones land, so symbols are named beside them.

## 8. Log
- **v1, 2026-11-23 (planner).** Written from 10H and the code at `7f839cc`. No owner card. Points for the orchestrator
  in `handoffs/reviewer/handoff_archive_plan_r1.md`.
- **v1.1, 2026-11-23 (orchestrator, H1 projection r0).** `handoffs/reviewer/handoff_H1_points_and_remote_record_projection_r0.md`
  (AMENDMENTS_REQUIRED, 0 owner cards): log line `shopify.budget` renamed `remote.budget_wait` (L4, H7); the
  `FakeCommerce` additions split by first caller H1/H2/H3 (L9); the `RemoteItems` constructor grows per milestone (L10);
  MC-H3.3's cover pruning in H1 deletes every `cover-*` (L8); rule 17 gains the throttled reply (L2, row H1·C1(f)) and
  the live check after H4 includes a clearing verification (reality check). Counts: H1 56 rows; totals 54 criteria,
  251 rows.
- **v1.2, 2026-11-23 (orchestrator, H3 projection r0).** `handoffs/reviewer/handoff_H3_bring_back_projection_r0.md`
  (AMENDMENTS_REQUIRED, 0 owner cards): MC-H4.2 and .4(b) store and test canonical paths (L5); MC-H4.3 refuses a
  non-JPEG body, stores a picture gone when its refresh finds no URL, gives a refreshed URL the full tries, and cleans
  up after a store failure (L6, L11, L10); MC-H4.4(a)/(b) pin the templated-session marks from 10G (retired rows
  released, brought-back main photos placed by hand in their container; L3); MC-H4.4(c) the condition role from
  `condition_photos_key()` and `retouch_gen` as stored (L13, L16); MC-H4.5 runs the import inside
  `retire_session`'s `act` (L8); MC-H4.7 gains the A16 sentence (L1); §4 Vocabulary `BringBackInput` / `BroughtBack`
  (L12), Service the uncached capture-root closure (L4); rule 17 the full-size download's body (L6, live check after
  H4). Counts: H3 64 rows (rows H3·C6(f), H3·C9(c) added); totals 54 criteria, 253 rows.
