# Price labels from Pricing — master plan (v1.2, 2026-11-23)

Short on purpose. It adds to `docs/implementation/master_plan.md` (roles, standing rules §3, registry §6, environment
§7), `master_plan_conversations.md` (the Pricing connection, adapter, fake and pull it extends) and
`master_plan_archive.md` (the remote items and picture caches it reuses), and wins over them on a name listed here.
Process: `docs/process/flow.md` §10 (milestones of units), with the deltas of §3 below. Planner: Claude
(`pipeline-planner`), from book `09C_pricing_labels.md` (RATIFIED 2026-11-23, changelog A33).

## 1. Goal
Build book `09C_pricing_labels.md`: a **Labels** box in the sessions list whose rows, filters, counts, search and pages
come from the labels provider (Beyo Pricing); **Generate label / Reprint label** from every session's ⋯ menu and in
batches from the Labels box; a **check** dialog (what will happen, grouped by template, with a picture of the real
label) before anything changes; the PDFs saved to `~/Documents/Session Manager Labels/` and opened in Preview; a
**label marker** on every session row and on the session page; markers and counts kept in step by the existing 09B
event pull. Semantics live in 09C (and 09B for the connection, 10H for shop-only items and picture caches). Pricing's
behaviour is owned by its integration package (`~/Desktop/Developer/pricing-app/docs/integrations/session-manager/`,
read-only, another repo): `LABELS.md`, `API.md` › Price labels, `EVENTS_AND_WEBHOOKS.md`, `openapi-integration.json`.
§5 below is the part 09C leaves to planning: the code-level rules of its four contracts **MC-L1…MC-L4** and of its
connection rule (§4 of the chapter, here **ACC**), each with the invariants **M-L1…M-L6** it serves. Where 09C and this
file differ, 09C wins and this file is amended; where the package and this file differ on an API shape, the package
wins.

## 2. Milestones (one runner phase each; order in `docs/implementation/phases_labels.txt`)
| Id | Builds | Units | Criteria | Rows | Changes app files | Projection | depends_on | Useful on its own |
|---|---|---|---|---|---|---|---|---|
| `L1_labels_provider_and_marker` | the labels port and vocabulary in core; the Beyo Pricing adapter's seven label calls and the binary transport; the access rule and its one stored value; the display copy, the status read, the label events in the 09B pull and the `labels_changed` event; Settings → Pricing's Labels line; the marker on Open / Published / All rows and on the session page; the ⋯ entry in its disabled states | 4 | 19 | 109 | yes | **yes** | — | every session row and the session page say whether the product's price label is printed, missing or stale, and follow Pricing within the 10-second check |
| `L2_labels_box` | the provider-backed list (filters, counts, search), rows matched to sessions or shown as shop products, their pictures, opening an unmatched row (adopt, no write), Select all N; the Labels box UI with Previous / Next pages, selection and refresh | 3 | 14 | 78 | yes | no | L1 | the photographer sees the labels still to print, searches and filters them through Pricing, and opens any of them as a session |
| `L3_generate_and_download` | the check (chunked, merged, grouped by template), the preview picture, runs with one key per chunk and safe retries, the result, saving to the Labels folder and opening in Preview; the check and done dialogs; the ⋯ entry and the batch bar wired; the errors of 09C §10 | 4 | 17 | 107 | yes | **yes** | L2 | the photographer generates and prints price labels, one or many |
Totals (computed from the plan files by script): 3 milestones, 11 units, 50 criteria, 294 rows; every unit has at most 8
criteria. Each milestone needs the one before approved (its `depends_on`), gate = the **Tracker** line of that plan
file. Start each with `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase <id>` (the runner resolves any id as
`docs/implementation/plans/<id>.md`); `run_phase.sh all` reads `phases.txt`, not this list.

## 2A. Owner decisions (1, answered 2026-11-23)

**Card 1 — When does the Labels box show its number?** **Answered 2026-11-23: Yes, as soon as the list opens** (the owner chose the recommendation; the plans stand as written).
- **Question:** Should the Labels box show its number as soon as the Sessions list opens (yes), or only after you
  have gone into the box once (no)?
- **Story:** You start Session Manager in the morning. Pricing holds 12 priced products without a label. With "no",
  the Labels box shows no number until you click into it; you would have to open it to learn whether anything needs
  printing. With "yes" it reads 12 at once, next to Open and Published, and drops as labels are printed.
- **Branches:** **Yes** — one small extra question to Pricing each time the Sessions list opens and after a label
  changes (never more than one every 2 seconds). **No** — no extra question; the number stays empty until your first
  visit to the box after each launch.
- **Recommendation:** Yes: the number is what tells you to open the box.
- **On silence:** L1 is built (it does not depend on this); L2 does not start.
- **Trace:** 09C §5 "Place" against §12 scenario 1; MC-L1.6; rows L2·C5, L2·C7(d), L2·C10(a), L2·C14(c).

The plans are written for **yes**. If the owner answers **no**: criterion L2·C5 and the tasks marked "(card 1)" in the
L2 plan are removed, L2·C10(a) reads "no number until the first list answer of this launch", L2·C14(c) reads "no
call", and `labels_summary` loses its `refresh` argument (row L2·C7(d) then asks without it).

## 3. Process deltas (flow §10 applies; these add to it)
- **No mechanism-inventory gate.** §5 stands in for it (as for the writer, conversations and archive tracks): one
  contract per mechanism 09C names (MC-L1…MC-L4) plus its connection rule (ACC), each resolved to code-level rules.
  09C's measurement ledger is its §9 (**M-L1…M-L6**). Trace cells cite `M-L<n>`, `MC-L<n>.<item>` / `ACC.<item>` (the
  items of §5), a 09C section or scenario (`§12.<n>`), a 09B rule (`CR<n>`, `MC-C1.<n>`), a 10H rule, `H1`–`H8`, or
  a **registry entry of §4** (`master plan §4 <entry>`, written short as the entry's name: `Adapter paths`, `Rule 17`
  (§3), `Ports`, `Words`, `Event`, `Commands`, `Storage`), when that entry states the asserted behaviour (L1
  projection r0, D12). `[rule]` only for UI rendering rows.
- **Milestones of units** (flow §10): one implementer session works the units in order with a
  `CHECKPOINT (not approved):` commit after each, one review, one fix budget, one narrative per milestone. Reason for
  more than 8 criteria per milestone: as for research, the writer, conversations and archive, a phase cycle per unit
  costs more than the work.
- **Implementer Claude Opus** (`PSM_IMPLEMENTER=claude`), reviewer Opus. **No plan sets `implementer:`** (flow.md log
  v1.3). The frontend-heavy parts are L1·U4, L2·U3 and L3·U4.
- **Projection** for `L1` (every Pricing label shape a test feeds the adapter, rule 17; the pull change, M-L1, M-L6)
  and `L3` (the only writing call: M-L2, M-L3, M-L4; the download address rule); not for `L2` (its Pricing shape is
  grounded in L1, its Shopify read is the existing `adopt`).
- **Tests** (flow §4, §4A, §10): one behaviour test per row at the boundary it names; unit tests only for a plan's
  **Core mechanisms**. While working: the milestone's own stems and UI files. The full suite runs **once, at the end
  of each milestone** (all three change app files).
- **Rule 9, here:** no test, no session and no tool run ever calls Pricing, Shopify or any CDN, and **no session
  launches the built app** (the Keychain is shared by every build, and a launched build reads the owner's shop
  credentials). Adapter rows run the real `BeyoPricingAdapter` over a recording `PricingHttp`; service rows run it over
  `psm_testkit::fake_pricing::FakePricing` (extended per milestone, §4 Tests) with `FakeCommerce` as the shop. **No
  test opens Preview and no test writes into the owner's Documents folder:** every `TestApp` installs
  `FakeLabelFiles` (§4 Files) and its home is its temporary folder (`test_app.rs:502`). The manual try-out of each plan
  is the owner's or the orchestrator's, on the installed app. **The first real Generate is the owner's, after L3**
  (09C §13: it marks products Printed in the live Pricing); the orchestrator's live check after L3 is read-only (list,
  status, templates, one check and one preview) unless the owner says otherwise.
- **Rule 17, here.** Every Pricing shape a test feeds the adapter is built from the package's schema
  (`openapi-integration.json` components `IntegrationLabelProductPage`, `IntegrationLabelStatus`,
  `IntegrationLabelTemplate`, `IntegrationPlan`, `IntegrationLabelRun`, `ErrorEnvelope`, `StoreEvent`) and the examples
  of `API.md` › Price labels and `EVENTS_AND_WEBHOOKS.md` › Price-label events (read 2026-11-23). The fixture files of
  §4 keep the **shape** of those examples field for field, fill what the examples elide (`…`) from the schema's
  required fields, and say so in a `"_source"` key (`"schema-grounded: openapi-integration.json <component> + API.md
  example; values neutral; not captured"`). The fake serialises through the adapter's own wire types. Facts read from
  the schema that the examples do not show and rows depend on: `sku`, `productType`, `priceMinor`, `labelPrintedAt`,
  `thumbUrl`, `createdAt` are nullable; `labelResetReason` may be absent; `IntegrationLabelStatus.label` is null with
  an `error` for an unknown product; `LabelStatusOut.hasPrice` is **nullable** ("null when the product isn't in the
  label universe"); `IntegrationPlanItem.template` is nullable; `idempotencyKey` is 8–80 characters; plan and run
  items take `productId` (≤ 32 characters) or `sku`; `q` ≤ 100 characters; `limit` 1–500; `RunItemOut.outcome` is
  `rendered | failed` and `lifecycleResult` is `marked_printed | reprinted | no_price | null`; `artifacts[].downloadUrl`
  is a **path** (`/integration/v1/stores/{storeId}/labels/runs/{runId}/artifacts/{artifactId}`), not an address;
  preview and artifact answers are `application/pdf` bytes; label events have `caseId: null` and no case fields in
  `data`. Transport errors are the two shapes `ReqwestPricing` produces (`transport.rs` `map_error`: `Error::Timeout`,
  `Error::Provider("http: …")`). **Not grounded** (the live check after L3 is their first contact): the status codes
  Pricing really sends for an invalid cursor, and that a PDF body starts with `%PDF-` (the PDF format's own header;
  MC-L2.12 refuses anything else).
- **The preview picture is verified, not assumed** (planner probe on the dev Mac, macOS 26.5.2, 2026-11-23): for a
  hand-built one-page vector PDF of 70 × 100 mm (MediaBox 198.43 × 283.46 pt), `sips -s format png -Z 1200 in.pdf
  --out out.png` wrote an 840 × 1200 PNG rendered from the vectors (sharp text and hairlines; plain `sips -s format
  png` gives 198 × 283, and `-s dpiWidth/dpiHeight` do not change that); `qlmanage -t -s 1200` gives the same picture
  under another file name. For a file that is not a PDF, `sips` exits 13 with "Cannot extract image from file" and
  writes no output file. The PNG has an alpha channel, so the dialog shows it on a white tile. `sips` is the choice
  (one command, the output name is ours). Not verified: older macOS on the work Mac (the bundle's minimum is 13.0);
  a failed raster only replaces the picture by a sentence, Generate is not blocked (MC-L2.5).
- **Row ids** `L<n>·C<k>(<row>)`, numbered once per milestone across its units; tests named after their row
  (`l1_c7a_<slug>`). Reviewer notes: `docs/implementation/backlog_labels/<milestone id>.md`.
- **Secrets and company data.** The Pricing key is typed only by the owner, in the installed app; it lives only in the
  Keychain and is never read, printed or logged by a session (CR2 holds unchanged: the label calls use the same
  in-memory key closure and bearer header). No key, real store id, shop domain, real product, price, template name or
  rule of the owner's business in a test, fixture, log, handoff or commit: fixtures use the ids already in the
  repository's C1 fixtures (store `e8f6b61a-…`, product `15531808096409`) and neutral values (`TEST-1802`, "Test
  table", "Label 70×100", `labels-2026-11-20.pdf`, `https://cdn.shopify.com/s/files/1/0000/0000/0000/files/<name>_160x.jpg`).
- **Guide.** A milestone that changes a flow, table, command, event, cache path or rule updates `docs/guide/` in the
  same change (flow §7A); each plan carries it as a task. `docs/implementation/STATUS_labels.md` is the
  orchestrator's; `STATUS.md` and `master_plan.md` §4 are not used or edited by these milestones.
- **Join:** after L3 and the live check the orchestrator opens the PR into `main`.

## 4. Names (registry; exact)
**Code layout.** Core: `crates/core/src/domain/label.rs` (vocabulary), `crates/core/src/ports/labels.rs`
(`LabelsProvider`, `LabelStore`, `LabelFiles`, `LabelsLink`), `crates/core/src/mechanisms/labels.rs` (pure rules and
the display copy, inline unit tests), `crates/core/src/services/labels.rs` (`LabelsService`: access, markers, list,
open; L3 adds `crates/core/src/services/label_runs.rs` = `impl LabelsService { check, preview, generate, partial,
download }`). **No name of Pricing, Shopify, "öre", "kronor", `gid`, "Preview" or a Pricing URL in `crates/core`**
(H7, 09C §3): core says *labels provider*, *label state*, *check*, *run*, *product*, *labels folder*; the only
integration name core uses is the existing constant `services::attributes::LINK_INTEGRATION` (no new literal).
Adapter: `crates/integrations/src/conversations/beyo_pricing/{labels.rs (impl LabelsProvider for BeyoPricingAdapter +
inline tests), label_types.rs (wire types), transport.rs, fixtures/labels_*.json, fixtures/events_labels.json}`.
Wiring: `crates/integrations/src/labels.rs` (builds the service, the label-files test seam). Storage:
`crates/infra/src/sqlite/labels.rs` + migration `crates/infra/src/sqlite/migrations/022_labels_access.sql`. Files:
`crates/infra/src/label_files.rs`.

**Vocabulary** (`psm_core::domain::label`, serde snake_case; times UTC milliseconds `i64`; money = the provider's
minor units as `i64`, never a float): `LabelStatus {Printed, NotPrinted}` · `ResetReason {PriceChanged, SkuChanged,
BarcodeChanged}` · `LabelState {status, printed_at: Option<i64>, has_price: Option<bool>, reset: Option<ResetReason>}`
· `LabelsAccess {Off(ConnectionState), Unknown, Allowed, Denied}` (stored part: `unknown | allowed | denied`) ·
`LabelFilter {MissingPriceTag, NotPrinted, Printed, All}` · `LabelCounts {total, has_price, missing_price,
missing_price_tag, printed, not_printed}` (`i64`) · `LabelProduct {product_id, sku: Option<String>, title,
product_type: Option<String>, price_minor: Option<i64>, state: LabelState, thumb_url: Option<String>, created_at:
Option<i64>}` · `LabelProductPage {products, next_cursor: Option<String>, counts, work_from: Option<String>}`
(`work_from` = the provider's `YYYY-MM-DD`, passed through) · `LabelTemplate {id, name, width_mm: f64, height_mm: f64,
is_default, product_types: Vec<String>}` · `CheckOutcome {MarkPrinted, Reprint, NoPrice, Fail}` · `PlanTemplate {id,
name, width_mm, height_mm}` · `PlanItem {position: usize, product_id, sku, title: Option<String>, product_type,
template: Option<PlanTemplate>, price_minor, status: LabelStatus, printed_at, outcome, issues: Vec<String>, notes:
Vec<String>}` (the messages as given) · `Unresolved {product_id: Option<String>, sku: Option<String>, message}` ·
`PlanSummary {items, ready, failing, mark_printed, reprint, no_price}` · `LabelSize {width_mm, height_mm, count}` ·
`LabelPlan {items, unresolved, summary, sizes}` · `RunStatus {Rendered, Partial, Failed}` · `RunResult {MarkedPrinted,
Reprinted, NoPrice}` · `RunItem {product_id, sku, title, rendered: bool, result: Option<RunResult>, error:
Option<String>}` · `RunCounts {items, rendered, failed, marked_printed, reprinted, no_price}` · `LabelArtifact {id,
width_mm, height_mm, page_count: i64, filename, size_bytes: i64, download_url, expired: bool}` · `LabelRun {id, status,
created_at, counts, artifacts, items, unresolved}` · `LabelChange {product_id, at: i64, kind: LabelChangeKind {Printed
{price_minor: Option<i64>}, Reset {reason: Option<ResetReason>}}}` · `SessionLabel {session_id, product_id:
Option<String>, state: Option<LabelState>}` · `LinkedItem {item_id, sku, external_id: Option<String>, session_id:
Option<i64>}` · `LabelRow {product_id, commerce_id: Option<String>, sku, title, product_type, price_minor, state,
created_at, session: Option<SessionRow>, picture_path: Option<PathBuf>}` · `LabelPage {rows, next_cursor, counts,
work_from}` · `Selected {product_id, sku}` · L3: `CheckGroup {template: Option<PlanTemplate>, product_types:
Vec<String>, items: Vec<PlanItem>}` · `LabelCheck {id, groups, unresolved, summary, sizes}` · `LabelFailure
{product_id: Option<String>, sku: Option<String>, title: Option<String>, message}` · `LabelRunResult {total, rendered,
marked_printed, reprinted, no_price, failures: Vec<LabelFailure>, not_run: Vec<String>, artifacts: Vec<LabelArtifact>,
failed_product_ids: Vec<String>}` · `SavedLabel {path: PathBuf, opened: bool}`. `psm_core::domain::conversation::
EventKind` gains the variant `Label(LabelChange)`.
**Errors.** `LabelsError` (what the adapter answers): `Unreachable(String)`, `BadReply(String)`, `NotAllowed`,
`StoreGone`, `NoProducts`, `RendererUnavailable`, `ListUnavailable`, `Expired`, `Rejected {code, message}`.
`LabelsFailure` (what the service answers): `Off(ConnectionState)`, `Denied`, `Provider(LabelsError)`, `Busy`,
`NothingToGenerate`, `NotFound`, `PreviewFailed(String)`, `SaveFailed(String)`, `Storage(psm_core::Error)`.

**Constants** (`psm_core::mechanisms::labels`): `LIST_PAGE = 25` · `SELECT_ALL_PAGE = 500` · `SELECT_ALL_MAX_PAGES =
200` · `STATUS_CHUNK = 200` · `RUN_CHUNK = 300` · `RUN_RETRIES = 2` · `STATUS_FRESH_MS = 10_000` · `COUNT_FRESH_MS =
10_000` · `QUERY_MAX = 100` · `PREVIEW_EDGE = 1200`. Adapter (`beyo_pricing::labels`): `RUN_TIMEOUT = 60 s`,
`FILE_TIMEOUT = 60 s`. Frontend (`features/labels/labelPages.ts`): `LABELS_REFRESH_MS = 2000`.

**Pure rules** (`mechanisms/labels.rs`; each is a plan's Core mechanism): `chunks(n: usize, size: usize) ->
Vec<Range<usize>>` · `LabelCopy` (the display copy: `start() -> u64`, `apply_answer(started, product_id,
Option<LabelState>, now: Instant)`, `apply_event(&LabelChange)`, `get(product_id) -> Option<LabelState>`,
`fresh(product_id, now, max_age) -> bool`, `clear()`) · `match_product(product_id, sku: Option<&str>, items:
&[(LinkedItem, Option<String>)]) -> Option<usize>` (the second member = the item's provider product id) ·
`merge_plans(Vec<(usize, Result<LabelPlan, Vec<Unresolved>>)>) -> LabelPlan` · `group_check(&LabelPlan,
&[LabelTemplate]) -> Vec<CheckGroup>` · `merge_runs(check items, chunk outcomes) -> LabelRunResult` ·
`label_file_name(raw: &str) -> String` · `numbered_name(name: &str, n: u32) -> String`.

**Ports** (`psm_core::ports::labels`).
- `LabelsProvider: Send + Sync`, every call `-> std::result::Result<_, LabelsError>`: `products(store_id, filter,
  query: Option<&str>, cursor: Option<&str>, limit: u32) -> LabelProductPage` · `status(store_id, product_ids:
  &[String]) -> Vec<Option<LabelState>>` (one entry per id, in order; `None` = the provider does not know the
  product; at most `STATUS_CHUNK` ids) · `templates(store_id) -> Vec<LabelTemplate>` · `plan(store_id, product_ids)
  -> LabelPlan` (at most `RUN_CHUNK`) · `preview(store_id, product_id) -> Vec<u8>` (the PDF) · `run(store_id,
  product_ids, key: &str) -> LabelRun` (**the only writing method**) · `download(store_id, download_url) -> Vec<u8>`;
  and three pure methods (no request): `describe(&self, error: &LabelsError, shop: Option<&str>) -> String` (the
  sentence of **Words**; the only way core obtains one), `product_id_of(&self, commerce_external_id: &str) ->
  Option<String>`, `commerce_id_of(&self, product_id: &str) -> Option<String>` (the two id forms of one product; the
  adapter is the only place that knows them).
- `LabelStore` (implemented for `SqliteStore` in `crates/infra/src/sqlite/labels.rs`): `labels_access() ->
  Result<LabelsAccess>` (never `Off`), `set_labels_access(LabelsAccess) -> Result<()>`, `linked_items() ->
  Result<Vec<LinkedItem>>` (every item with its `LINK_INTEGRATION` link's external id and its latest session, one
  read), `session_products(session_ids: &[i64]) -> Result<Vec<(i64, Option<String>)>>` (each session with its item's
  link external id).
- `LabelFiles` (L3): `raster(&self, pdf: &[u8], name: &str) -> Result<PathBuf>` (a PNG of the first page under
  `<previews>/labels/<name>.png`, long edge `PREVIEW_EDGE`), `clear_previews(&self) -> Result<()>`, `save(&self,
  filename: &str, bytes: &[u8]) -> Result<PathBuf>` (into the labels folder by MC-L2.13), `open(&self, path: &Path)
  -> Result<()>`.
- `LabelsLink` (what the conversations service tells the labels service; set once with
  `ConversationsService::set_labels_link(Weak<dyn LabelsLink>)`, absent = nothing is told): `connected(&self)`,
  `key_changed(&self)`, `changed(&self, changes: &[LabelChange])`.

**Storage.** `022_labels_access.sql`: `ALTER TABLE conversation_sync ADD COLUMN labels_access TEXT NOT NULL DEFAULT
'unknown';` with its `M::up` line in `SqliteStore::migrations()` and its `DROP COLUMN` line in the rewind test
`crates/testkit/tests/suite/p27_retire_shape_on_arrival.rs` (`c4a_…`; without it that test fails with "duplicate column
name"). **This is the only thing 09C stores**: whether the saved key may read labels (a fact about the key, not label
state, M-L1). No other table, column, settings field or file holds anything about labels. `022` is the next free
number at `ae91079`; if another change takes it first, use the next (never renumber an applied file).

**Files.** `psm_infra::label_files::DiskLabelFiles::new(previews_root, labels_dir)`: `raster` writes the PDF to a
temporary file `<previews>/labels/<name>.src.pdf` (never the system's temporary folder: L3 projection r0, D1), runs
`/usr/bin/sips -s format png -Z 1200 <pdf> --out <previews>/labels/<name>.png` (a non-zero exit or a missing output is
an error; the temporary PDF is always removed), `save` by MC-L2.13, `open` runs
`/usr/bin/open -a Preview <path>`. The labels folder is `paths.home.join("Documents/Session Manager Labels")`, always
from `AppPaths.home` (never `$HOME` read again, never a hard-coded user path). `<previews>/labels/` is inside the
previews folder the webview may already read (`asset_protocol_scope`, desktop `lib.rs`). Test seam:
`psm_integrations::labels::with_test_label_files(paths, Arc<dyn LabelFiles>)` (keyed by the data folder, like
`with_test_conversations`); `psm_testkit::fake_label_files::FakeLabelFiles` saves through a real `DiskLabelFiles` on
the test's own home, **never runs `sips` or `open`** (its `raster` writes a fixed 1 × 1 PNG, its `open` records the
path), and can be told to fail the next `raster`, `save` or `open`. `TestApp` installs it in every constructor from L3
on.

**Service.** `LabelsService` (`App.labels: Arc<LabelsService>`, also in `AppHandles`; always present, H6: without a
ready Pricing connection it answers `Off` and makes no request). Final constructor `LabelsService::new(conversations:
Arc<ConversationsService>, provider: Arc<dyn LabelsProvider>, labels: Arc<dyn LabelStore>, store: Arc<dyn Store>,
remote: Arc<RemoteItems>, commerce: Option<Arc<dyn CommerceAdapter>>, files: Arc<dyn LabelFiles>, clock, events)`;
each milestone takes only the arguments it reads (no unused field under `-D warnings`): L1 `(conversations, provider,
labels, clock, events)`, L2 adds `store`, `remote`, `commerce`, L3 adds `files`. One `BeyoPricingAdapter` instance
serves both ports (`conversations::build_service` hands out the same `Arc` twice), so the label calls use the same
live server and in-memory key closures; `crates/integrations/src/labels.rs::build_service` builds the service and
calls `conversations.set_labels_link(Arc::downgrade(&labels))`. Methods: L1 `access()`, `session_labels(session_ids,
refresh)`; `LabelsLink` · L2 `list(filter, query, cursor)`, `summary(refresh)`, `select_all(filter, query)`,
`open_product(product_id)` · L3 `check(product_ids)`, `preview(check_id, product_id)`, `generate(check_id)`,
`partial(check_id)`, `download(check_id, artifact_id)`. Nothing here starts a thread or a timer: every provider call
is made inside a command (a press, or the screen opening) or inside the 09B pull.

**Transport.** `PricingHttp` gains `fn fetch(&self, request: PricingRequest, timeout: Duration) ->
psm_core::Result<RawReply>` with `RawReply {status: u16, content_type: Option<String>, bytes: Vec<u8>}` (the body
untouched; `send` keeps turning a body into JSON and must not be used for a PDF: it reads a non-JSON body with
`from_utf8_lossy`). `ReqwestPricing::fetch` sets the per-request timeout and `Accept: application/pdf,
application/json`; the key goes only in the `Authorization: Bearer` header, errors carry no URL. Used by `preview`,
`download` and `run` (`run` for its longer timeout; its body is parsed from the bytes). `NoTransport`, the adapter
tests' recording transport and `FakePricing` implement it.

**Adapter paths** (under `{server}/integration/v1/stores/{storeId}`): `GET /labels/products?filter=&q=&cursor=&limit=`
(`filter` omitted for `All`; values `missingPriceTag`, `notPrinted`, `printed`; `q` and `cursor` percent-encoded) ·
`GET /labels/status?productIds=<comma-joined>` · `GET /labels/templates` · `POST /labels/plan {"items":
[{"productId": …}]}` · `POST /labels/preview {"productId": …}` · `POST /labels/runs {"items": [{"productId": …}],
"idempotencyKey": …}` · `GET <downloadUrl>`. Products are always addressed by `productId`; no `sku` and no
`templateId` is ever sent (09C L8; §6). Status mapping, for every label call: `401`, `403` → `NotAllowed` · `404` →
`StoreGone` (plan / runs / list / status / templates), `Rejected` (preview: the message as given), `Expired`
(download) · `409` → `Rejected` · `410` → `Expired` · `422` with code `no_products` → `NoProducts`, other `422` →
`Rejected` · `503` with code `renderer_unavailable` → `RendererUnavailable`, with code `product_list_unavailable` →
`ListUnavailable` · other `5xx` and transport errors → `Unreachable` · any other status → `Rejected` when the body is
an error envelope, else `BadReply` · a 2xx body of the wrong shape → `BadReply`.
Fixtures (`fixtures/`): `labels_products.json`, `labels_status.json`, `labels_templates.json`, `labels_plan.json`,
`labels_run.json`, `events_labels.json`.

**Commands** (`apps/desktop/src-tauri/src/commands/labels.rs`, registered in `specta_builder()`; wrapper
`apps/desktop/src/services/labels.ts`): L1 `labels_status(session_ids, refresh) -> {access, labels:
[{session_id, product_id, marker}]}`; `conversations_settings_get` / `conversations_test` / `conversations_key_set`
answer gains `labels: 'off' | 'unknown' | 'allowed' | 'denied'` · L2 `labels_list(filter, query, cursor)`,
`labels_summary(refresh) -> {missing_price_tag: number | null}`, `labels_select_all(filter, query)`,
`labels_open(product_id) -> {session_id}` · L3 `labels_check(product_ids)`, `labels_preview(check_id, product_id) ->
{path}`, `labels_generate(check_id)`, `labels_partial(check_id)`, `labels_download(check_id, artifact_id) -> {path,
opened}`. Wire forms: `marker = {status: 'printed' | 'not_printed', printed_at, has_price, reset: 'price_changed' |
'sku_changed' | 'barcode_changed' | null} | null`; `access = {state: 'off' | 'unknown' | 'allowed' | 'denied', reason:
<connection state> | null}`; filter `'missing_price_tag' | 'not_printed' | 'printed' | 'all'`.
Error codes (`CommandError.code`; message = the sentence): `labels_off` (`data.reason` = the connection state; message
empty: the frontend has 09B's words) · `labels_denied` (`Denied` and `NotAllowed`) · `labels_unreachable` ·
`labels_bad_reply` · `labels_store_gone` · `labels_no_products` · `labels_renderer_unavailable` ·
`labels_list_unavailable` · `labels_expired` · `labels_rejected` (the provider's message as given) · `labels_busy` ·
`labels_nothing` · `labels_preview_failed` · `labels_save_failed` · `not_found` (existing).

**Event.** `AppEvent::LabelsChanged {product_ids: Vec<String>, access: bool}` → window event `labels_changed
{product_ids, access}` (listener `onLabelsChanged` in `apps/desktop/src/services/events.ts`). `access: true` = the
access value changed (re-read it); `product_ids` = the products whose display copy changed.

**Words** (exact). From the adapter's `describe` (they name Pricing): `Session Manager isn't allowed to print labels.
Ask a Pricing admin.` (`NotAllowed`) · `This key is no longer tied to <shop>` (`StoreGone`; "the connected shop"
without one) · `None of these products were found in Pricing.` (`NoProducts`) · `Labels can't be generated right now.
Try again in a moment.` (`RendererUnavailable`) · `This file has expired. Generate the labels again.` (`Expired`) ·
`The product list isn't available right now.` (`ListUnavailable`) · `Pricing can't be reached right now.`
(`Unreachable`) · `Pricing answered in a way this version does not understand` (`BadReply`, 09B's sentence) ·
`Rejected` → its message as given. From core (they name no platform): `Labels are being generated.` (`Busy`) · `None
of these products can be generated.` (`NothingToGenerate`) · `The label preview can't be shown.` (`PreviewFailed`) ·
`Couldn't save the PDF: <reason>.` (`SaveFailed`) · `Not found` (an `Unresolved` built by MC-L2.1).
Frontend (`features/labels/labelWords.ts`): the marker words of MC-L3.5 · `Labels: allowed` · `Labels: this key can't
read labels. Ask a Pricing admin to add labels:read and labels:write.` · `Labels: couldn't check. Press Test
connection.` · `This key can't read labels. Ask a Pricing admin to add labels:read and labels:write.` (the box's body
and the ⋯ hint when denied) · `Open Settings → Pricing` · `Publish first so Pricing can see it` · `Generate label` ·
`Reprint label` · `Missing price tag` · `Not printed` · `Printed` · `All` · `Products created since <d MMM yyyy>` ·
`No price` · `Page <k> of <n>` · `Previous` · `Next` · `<N> selected` · `Select all <N>` · `All <N> selected` ·
`Generate labels for <N>` · `Try again` · the check and done dialogs' sentences of 09C §6 (in the L3 plan, verbatim)
· `Not found in Pricing` · `Can't generate` · `Print at 100 % (not Fit to page).` · `Failed products stay selected so
you can fix and retry.` · `Download PDF` · `Generate PDF` · `Reprint PDF` · `Done` · `Close` · `Cancel`.

**Log lines.** `labels.access state=` · `labels.status asked= answered=` · `labels.events changed=` · `labels.list
filter= rows= more=` · `labels.check products= chunks= ready= failing=` · `labels.run chunk= of= items= attempt=
outcome=` · `labels.download bytes= outcome=`. Never a key, an idempotency key, a URL with a query, a SKU list or a
product title.

**Frontend.** `apps/desktop/src/features/labels/`: L1 `labelWords.ts` (pure), `LabelPill.tsx`, `useSessionLabels.ts`,
`LabelMenuItem.tsx` · L2 `labelPages.ts` (pure), `LabelsBox.tsx`, `LabelRow.tsx`, `LabelsBatchBar.tsx` · L3
`LabelDialog.tsx`, `LabelCheckView.tsx`, `LabelDoneView.tsx`, `useLabelDialog.ts`. Edited: `features/sessions/
{SessionsPage.tsx, Filters.tsx, SessionRow.tsx, DeleteSession.tsx}`, `features/shell/Header.tsx`,
`features/settings/PricingSection.tsx`.

**Tests.** Suite stems `crates/testkit/tests/suite/{l1_labels_access.rs, l1_labels_markers.rs, l2_labels_list.rs,
l3_labels_check.rs, l3_labels_generate.rs, l3_labels_download.rs}` (each with its `mod` line in `main.rs`); adapter
tests inline in `beyo_pricing/labels.rs`; infra tests inline in `crates/infra/src/label_files.rs`; desktop command and
bridge tests in `apps/desktop/src-tauri/src/lib.rs` (`l1_tests`, `l2_tests`, `l3_tests`); UI
`features/labels/__tests__/l<n>_<slug>.test.tsx` (and `features/settings/__tests__/l1_labels_line.test.tsx`).
`FakePricing` grows, each addition in the milestone whose rows first call it (charter rule 4): **L1** `fetch`
(answers `404` until L3), label products as the fake's label state (`add_label_product(LabelSpec {product_id, sku,
title, product_type, price_minor, printed_at, reset, thumb_url, created_at, template})`, `set_label(product_id, …)`),
`GET /labels/status`, `GET /labels/templates` (`add_template`), `deny_labels()` (every `/labels/` path answers `403
insufficient_scope`), `label_printed(product_id)` / `label_reset(product_id, reason)` (change the state and log the
event with `productId` in `data` and in the envelope) · **L2** `GET /labels/products` (newest created first, `filter`,
`q` over SKU / title / product type, `limit`, an opaque `cursor`, `counts` over the searched list, `workFrom` through
`set_work_from`), `repeat_cursor_next()` · **L3** `POST /labels/plan` (outcomes by the package's rules; `fail_label
(product_id, code, message)` makes a product fail in plan and run, `fail_in_run(product_id, code, message)` only in
the run), `POST /labels/preview` (bytes `%PDF-1.4 fake preview <productId>`), `POST /labels/runs` (one run per
`idempotencyKey`: the same key answers the stored run; marks priced unprinted products Printed and logs one
`label.printed` each; one artifact per label size; `runs()` lists the stored runs with their keys), the artifact
download (`expire_artifacts()` → `410 artifact_expired`).

## 5. Contracts (stand in for the mechanism inventory; plans implement them verbatim)

Notation: *connection* = `ConversationsService::connection()` (09B MC-C1.2: no request); *store* = its store id; *A* =
the stored access value; *copy* = the display copy (`LabelCopy`, memory only); *press* = work started by a command.

### ACC — When a label call may be made (09C §4; M-L6)
1. **Calls allowed** ⇔ the connection's state is `Ready` with a store ∧ `A ≠ denied`. Otherwise the service makes no
   request to any `/labels/` path and answers `Off(state)` (state not `Ready`) or `Denied`.
2. **`connected()`** (told by `ConversationsService::connect` after it has left the lane, when the resulting state is
   `Ready`: Test connection, and Save key): one `templates(store)`: `Ok` → `A := allowed`; `NotAllowed` → `A :=
   denied`; any other error → `A := unknown`. Then the copy and the remembered counts are emptied and `LabelsChanged
   {[], access: true}` is emitted. `connected()` applies only this rule (its own three-way assignment and exactly one
   `LabelsChanged`); rule 4 governs every other label call (L1 projection r0, D2).
3. **`key_changed()`** (told by `key_set`, before it connects or removes the key): `A := unknown`, the copy and the
   counts are emptied. (A saved key whose connect then fails leaves `unknown`, never the old key's answer.)
4. **Learning by use.** A label call that succeeds while `A = unknown` → `A := allowed`. A label call that answers
   `NotAllowed` (in any state) → `A := denied`, the copy is emptied, `LabelsChanged {[], access: true}`. So a key
   without the scope costs at most one label call per saved key (Test connection's, or the first after an upgrade
   from a build without labels), and none after.
5. **A label failure never touches the conversations connection:** label calls answer `LabelsError`, never
   `ProviderError`; `conversation_sync.state`, `last_error` and the pull's back-off are not written by any label call
   (a key without `labels:read` keeps its conversations).
6. **Label calls do not take the conversations lane** (they neither read nor move the cursor) and read the store from
   the connection when they start.
7. `access()` = `Off(state)` when the connection is not `Ready`, else `A`.
**Invariant (M-L6).** With no key, an unmatched or gone store, a changed server, or `A = denied`, the recorded
requests hold no `/labels/` path.

### MC-L3 — The display copy and the marker (09C §7; M-L1)
1. **State from the provider.** A status entry or a list row → `LabelState {status, printed_at, has_price, reset}`:
   `labelStatus` `printed` / `notPrinted` (anything else → `BadReply`), `labelPrintedAt`, `hasPrice` (the status
   entry's may be null = unknown), `resetReason` / `labelResetReason` (`price_changed`, `sku_changed`,
   `barcode_changed`; absent, null or an unknown word → none). A status entry whose `label` is null → no state.
2. **The copy** (`LabelCopy`): product id → state, the sequence number of the last event applied to it, and when it
   was last answered (monotonic). `start()` gives the current sequence. `apply_answer(started, id, state, now)`
   stores the state (or removes the entry for `None`) **only when no event was applied to that product after
   `started`** (an event that arrived while the request was in flight wins over the older answer). `apply_event`:
   `Printed` → `{Printed, printed_at = the event's time, has_price = true, reset = none}`; `Reset` → `{NotPrinted,
   printed_at = none, has_price = as it was (unknown without an entry), reset = the reason}`; it creates the entry
   when there is none and raises the sequence.
3. **`session_labels(session_ids, refresh)`**: `LabelStore::session_products` → each session's product id =
   `provider.product_id_of(link external id)` (none without a link). When `refresh` ∧ calls allowed (ACC.1) ∧ at
   least one of the request's products has no entry answered within `STATUS_FRESH_MS`: `started := copy.start()`,
   then one `status(store, ids)` per `STATUS_CHUNK` distinct product ids in request order, each answer applied by
   rule 2. An error is returned (the copy is unchanged). The answer is one `SessionLabel` per session in request
   order, its state read from the copy. When calls are not allowed: no request and every state is none.
4. **Nothing about label state is written to the database** (M-L1): the copy, the counts, a check, a generation and
   its keys live in memory and are gone at quit.
5. **Marker words** (`markerText(state, now)`, first match): no state → no marker · `Printed` → `Label printed ·
   <d MMM>` (local time; `<d MMM yyyy>` when the year is not this year; `Label printed` without a time) · `reset =
   PriceChanged` → `Price changed: new label needed` · `SkuChanged` → `SKU changed: new label needed` ·
   `BarcodeChanged` → `Barcode changed: new label needed` · `has_price = false` → `No price yet` · `has_price = true`
   → `Missing price tag` · `has_price` unknown → no marker.
6. **Where:** a pill in the row's pill line in Open, Published, All and Labels; beside the title on the session's
   page.
7. **When the screen asks** (never delaying the list): the Sessions page calls `labels_status(ids of the page shown,
   refresh)` after each page answer without waiting for it to render the rows; `refresh` is true when that id list
   differs from the list of this mount's last refreshing call, else false (a `sessions_changed` reload of the same
   page asks no provider). The session page calls it for its one session with `refresh = true` when opened. On
   `labels_changed` each of them calls it once more with `refresh = false` (the copy already holds the news); with
   `access: true` it re-reads with `refresh = true`.

### MC-L4 — Staying in step (09C §8)
1. **Adapter.** `label.printed` → `EventKind::Label(LabelChange {product_id, at = the envelope's time, kind: Printed
   {price_minor}})`; `label.reset` → `kind: Reset {reason}` (an unknown reason → none). `product_id` =
   `data.productId`, else the envelope's `productId`; neither → `EventKind::Other`. `conversation_id` none, `for_us`
   false, as for every event that is not a conversation's.
2. **Pull** (MC-C1.6, one branch added): an event with `id > cursor` and kind `Label(change)` → the same transaction
   as "not for us" (`cursor := id`, in the lane), then `change` is added to the page's list. Events with `id ≤
   cursor` are skipped as today (this is the deduplication by id). The **first connection** is unchanged: label
   events are paged past and nothing is told.
3. **Delivery.** Whenever `pull` returns — the page is done, or it stopped on an error — the page's list, when not
   empty, is handed once to `LabelsLink::changed`. `LabelsService` applies each change in order (MC-L3.2) and emits
   one `LabelsChanged {product_ids: the distinct ids in first-appearance order, access: false}`.
4. **Silent.** A label event never sets the check's "applied", never marks a notice: no `ConversationsChanged`, no
   chime, no banner, no toast because of it (owner L7).
5. **The screen**: markers by MC-L3.7; the Labels box by MC-L1.9.
**Invariant.** After a pull page, `S.cursor` ≥ the id of every label event of that page that was applied, and the
copy holds, for each product, the state of its last event in the log order; a redelivered event changes nothing.

### MC-L1 — The Labels box (09C §5)
1. **`list(filter, query, cursor)`**: calls allowed, else `Off` / `Denied`. `q` = the query trimmed, none when empty,
   cut to `QUERY_MAX` characters. One `products(store, filter, q, cursor, LIST_PAGE)`; `cursor` is passed through
   untouched.
2. **Rows**, in the provider's order: `LabelRow` with `state` from the row's own fields (MC-L3.1; `has_price` = the
   row's `hasPrice`), `commerce_id = provider.commerce_id_of(product_id)`.
3. **Matching** (`match_product` over `LabelStore::linked_items`): (i) the item whose link's
   `provider.product_id_of(external id)` equals the row's product id (the lowest item id if several); else (ii) the
   item whose SKU equals the row's SKU exactly (case-sensitive; `items.sku` is unique, so "exactly one" is "one").
   `session` = that item's latest session as `SessionsService::page(All, None, None, Some(&matched session ids), 1)`
   answers it (one call for the page; its name, pills and `cover_preview_path` follow 10H §4, M-H4); an item without
   a session, and no match, give no session.
4. **Pictures.** A row with a session shows the session row's picture (no extra work). A row without one: when
   `thumb_url` starts with `https://` and `commerce_id` is known and a commerce integration is present →
   `RemoteItems::search_picture(commerce_id, thumb_url)`: the cached file, or none while its download waits in 10H's
   queue (`SEARCH_EDGE`, `PictureReady {Search {external_id: commerce_id}}` when it lands); otherwise none. The list
   never downloads.
5. **After the answer:** `started := copy.start()` was taken before the request; every row's state is applied by
   MC-L3.2; `counts` are remembered (memory) together with whether the request had a search.
6. **The box's number** (card 1, §2A): `summary(refresh)` = the remembered `counts.missing_price_tag`; with `refresh`
   ∧ calls allowed ∧ (nothing remembered ∨ the remembered answer had a search ∨ it is older than `COUNT_FRESH_MS`):
   one `products(store, MissingPriceTag, None, None, 1)` first (its counts are remembered, its row is applied to the
   copy). Not allowed → none, no request. Inside the box the number is the last list answer's (search applied).
7. **`select_all(filter, query)`**: `products(store, filter, q, cursor, SELECT_ALL_PAGE)` from no cursor until
   `next_cursor` is none; a cursor seen before in this walk, or more than `SELECT_ALL_MAX_PAGES` pages → `BadReply`;
   the answer is `Selected {product_id, sku}` for every row **with a SKU**, in the provider's order; any error →
   that error and nothing else (never a partial selection).
8. **`open_product(product_id)`**: match by rule 3 → that item's latest session, or `SessionsService::create(item)`
   when it has none; no match → `commerce_id_of(product_id)` (none → `InvalidValue`) → the existing
   `ItemsService::adopt` (one read of the shop product, **no write**, M-L5) → `SessionsService::create` → the session
   id. Errors of `adopt` (`SkuCollision`, a product without a SKU, `NoIntegration`) pass through unchanged and create
   nothing. From then on the row is matched by rule 3(i).
9. **The screen** (`LabelsBox`, state by the pure `pageStep` of `labelPages.ts`):
   - Entering the box, changing the filter or the search (debounced like the sessions search), and **Try again** ask
     the first page (`cursor` none). The cursor stack is `cursors[0] = none`, `cursors[k + 1]` = the `next_cursor` of
     the answer for page `k`; **Next** is offered iff the page shown has a `next_cursor` and asks `cursors[k + 1]`;
     **Previous** asks `cursors[k − 1]` (the cursor already seen, 09C §5); a newer answer for page `k` replaces
     `cursors[k + 1]` and drops the cursors after it; an answer with no row on a page `k > 0`, and a
     `labels_rejected` answer for a cursor, go back to the first page (once).
   - `Page <k + 1> of <n>` with `n = max(k + 1, ceil(count of the filter / LIST_PAGE), 1)`, the filter's count =
     `missing_price_tag`, `not_printed`, `printed` or `total`. No page-number buttons.
   - **Refresh:** on entering, after a run's dialog closes, and on `labels_changed`: the page shown is asked again
     with its own cursor, at most once per `LABELS_REFRESH_MS` (the first at once, the events of the next 2 s folded
     into one more at its end). The rows stay on screen while a refresh is in flight and after a failed one.
   - **Selection:** a map product id → SKU. A row with a SKU has a tick; the header box ticks or clears the page's
     ticked-able rows; when every such row of the page is ticked and the filter's count is larger than the
     selection, the bar offers `Select all <count>` → `labels_select_all` → the selection is exactly what it
     answers, and the bar reads `All <n> selected` (`n` = the selection's size). Page changes and refreshes keep the
     selection; changing the filter or the search, leaving the box (another box, or the page), and Escape clear it.
   - Hidden in the box: the time filter, "To review", the Unassigned card, the time groups, Review & publish.
**Invariant.** No step of MC-L1 calls `run`, and none calls a mutating `CommerceAdapter` method (M-L2, M-L5).

### MC-L2 — Check, generate, download (09C §6; M-L2, M-L3, M-L4, M-L5)
1. **`check(product_ids)`** (a press; changes nothing): the ids without repeats (first kept), at least one
   (`InvalidValue` otherwise); calls allowed; no generation running (`Busy`). One `plan(store, chunk)` per
   `RUN_CHUNK` ids in order; `merge_plans`: items in request order with `position` counted over the whole request
   (`position` := the item's index among the merged items, 0, 1, 2, … in merged order, independent of Pricing's own
   numbering, so unresolved entries and `NoProducts` chunks leave no gap; L3 projection r0, D4),
   `unresolved` concatenated, each `summary` number summed, `sizes` merged where width and height are equal, in
   first-appearance order. A chunk answering `NoProducts` contributes its ids as `Unresolved {message: "Not found"}`;
   when every chunk does, the check answers `NoProducts`. Any other error → that error, nothing kept.
2. **Template words.** One `templates(store)` per check; a group's product types are that template's
   `product_types`, empty when the read fails or the id is not in it (the check never fails because of it).
3. **One check is kept** (memory): `{id: a new UUID v4, the merged plan, generation: none}`. A new check replaces it
   and calls `LabelFiles::clear_previews`. Nothing else names a check.
4. **Groups** (`group_check`): by `template.id` in first-appearance order, items in plan order; the items without a
   template form one last group.
5. **`preview(check_id, product_id)`**: `check_id` is the kept check and `product_id` one of its items, else
   `NotFound` and no request. The first ask: `provider.preview` → bytes → `files.raster(bytes, "<check id>-<product
   id>")` → the PNG's path; a later ask answers that path without a request while the file exists. A `Rejected`
   answer keeps its message; a `raster` failure → `PreviewFailed`. A preview never blocks Generate.
6. **`generate(check_id)`** (the only way a run starts, M-L2): `check_id` is the kept check, else `NotFound` and no
   request. The first call creates the generation: `ready` = the check's items whose outcome is not `Fail`, in plan
   order; none → `NothingToGenerate`, no request; chunks of `RUN_CHUNK`; **each chunk gets its key once**, `sm-` + a
   UUID v4 (39 characters; the package asks 8–80). While a call is working the generation is `running`: another
   `generate` (a double press) answers `Busy` and sends nothing; `check` answers `Busy`.
7. **Each chunk without an outcome, in order:** `provider.run(store, the chunk's product ids, its key)`.
   `Unreachable` or `RendererUnavailable` → sent again with the same key and the same ids, up to `RUN_RETRIES` more
   times, at once; still failing → the generation stops (`running` false), that error is returned, finished chunks
   keep their outcome and are **never sent again**; the next `generate(check_id)` (Try again) continues at this chunk
   with the same key. `NoProducts` → the chunk's outcome is "every product failed" with that error's sentence (no
   retry). `NotAllowed` → ACC.4 and the error (no retry). `StoreGone`, `BadReply`, `Rejected` → the error, no retry;
   the chunk stays without an outcome.
8. **After each finished chunk** the copy follows the run (`started` taken before the request, MC-L3.2): an item with
   `MarkedPrinted` → `{Printed, the run's created_at, true, none}`; `NoPrice` → `{NotPrinted, none, false, reset as
   it was}`; `Reprinted` → `Printed` with the time it had: the **check item's `printed_at`**, else the run's `created_at`
   (L3 projection r0, D5); then `LabelsChanged {those products}`.
9. **The result** (`merge_runs`; this is what reconciles 09C §6.2 "ready products only" with §12 scenario 4 "2 of 3"):
   `total` = the check's items (those that failed the check included; unresolved ones are not items); `rendered`,
   `marked_printed`, `reprinted`, `no_price` = the sums of the runs' counts; `failures`, in this order: the check's
   `Fail` items (message = their issues joined with " · "), then per chunk its run items that were not rendered
   (their error message), its run's `unresolved` (their message), or all its products for a `NoProducts` chunk;
   `not_run` = the products of chunks without an outcome; `artifacts` in chunk order; `failed_product_ids` = the
   failures' and `not_run`'s product ids. A failure's `sku` and `title` are the **check item's** (always kept with
   the check), falling back to the run item's or the `Unresolved`'s own (rule 17: Pricing's `RunItemOut.title` is
   nullable and its `API.md` runs example sends none; L3 projection r0, D3). **Fully generated ⇔ `failures` and `not_run` are both empty** (M-L4).
10. **Kept with the check:** a finished generation's result; `generate(check_id)` again answers it without a
    request. `partial(check_id)` = the result over the finished chunks (for a dialog the photographer leaves after a
    failed chunk); `NotFound` without a generation.
11. **`download(check_id, artifact_id)`**: an artifact of the kept result, else `NotFound` and no request;
    `provider.download(store, artifact.download_url)` → `files.save(artifact.filename, bytes)` → `files.open(path)`;
    a failed open is logged and answered as `opened: false` (the file is saved). `Expired` and `SaveFailed` are
    returned; a second call tries again.
12. **The download address** (the key must never leave the configured server): the adapter sends the request only
    when `download_url` is a path that starts with `/integration/v1/stores/<the store id, percent-encoded as the
    adapter encodes it>/labels/runs/`, holds no `..` segment, no `?`, no `#` and no `//`; it is then appended to the
    live server address. Anything else (an absolute address above all) → `BadReply` with **no request**. A 200 body
    that does not start with `%PDF-` → `BadReply` (preview and download alike).
13. **The labels folder** (`DiskLabelFiles::save`): the folder is created when missing; the name is
    `label_file_name(filename)`: the text after the last `/` or `\`, without control characters and `:`, leading
    dots and surrounding spaces removed, `.pdf` added unless it ends with it (any case), `labels.pdf` when nothing
    is left, the part before `.pdf` cut to 120 characters; when that name is taken (a file or a folder), `numbered_name
    (name, n)` = `<stem> (<n>).pdf` for n = 2, 3, … until free; the bytes are written to `<name>.part` in the folder
    and renamed (a leftover `.part` is overwritten; on a failure the `.part` is removed). Nothing outside the folder
    is ever written.
14. **Nothing else is ever written to the provider** (M-L2): the labels port has one writing method and the adapter
    issues it only as `POST …/labels/runs`; `list`, `status`, `templates`, `plan`, `preview` and `download` are `GET`
    or the two read-only `POST`s. No label step calls a mutating `CommerceAdapter` method, and "Delete session" makes
    no label call (M-L5).
**Invariants.** M-L3: every `POST …/labels/runs` the app issues for one chunk of one press carries the same key,
first attempt, automatic retry and Try again alike; two chunks never share a key; a finished chunk is not sent again.
M-L4: a result with a failure or a product not run never has `rendered = total`, and every such product is in
`failures` or `not_run` with a message.

## 6. Delegated choices (decided here; each is one sentence in a narrative's "Choices made for you")
- **Products are always sent to Pricing by their product id, never by SKU** (the id is known for every Labels row
  and for every published session; a session without a shop product has the entry disabled). 09C's "else SKU" has
  no case left, so nothing is built for it.
- **Whether the key may read labels is remembered** (the one stored fact of this feature), so a key without the
  permission is asked once, not at every launch.
- **A key that cannot read labels keeps its conversations:** a refused label call never marks the Pricing connection
  as refused.
- **Test connection clears the markers on screen** and they are read again (it may have connected another store).
- **"Label printed · 4 Nov"** shows the year when it is not this year.
- **A stale label wins over "No price yet":** a product whose printed label was reset reads "Price changed: new label
  needed" even when it has no price now.
- **A product Pricing does not list** (created before its Work-from date) shows "Label printed" or the reset words
  when Pricing knows them, and no marker otherwise (its price state is unknown to the label list).
- **Markers are read again only when the page shows other sessions,** and never more than once per 10 seconds for the
  same products; a photo arriving does not ask Pricing.
- **A Labels row whose product has the same SKU as a session on this Mac opens that session,** even when that session
  is linked to another shop product (09C's rule; SKUs are unique on this Mac).
- **Opening a Labels row with no session here creates the session at once** (one read of the shop product, nothing
  written to Shopify) and opens it as a shop-only product.
- **The Labels box has Previous and Next only;** "Page 2 of 5" is computed from the filter's count.
- **Select all N selects only products that have a SKU** (09C), so the bar may read "All 58 selected" under "Select
  all 60".
- **The done dialog counts the products the check refused as failed** ("2 of 3 labels generated"), although only the
  products that passed the check are sent to Pricing.
- **A dropped connection during Generate is retried twice at once with the same run key;** after that the dialog
  offers Try again, which continues where it stopped. Chunks already generated are never sent again.
- **Leaving a batch after a failed chunk** shows what was generated so far, with the rest listed as not generated and
  still selected.
- **Run keys are not kept across a quit.** If the app quits during Generate, generating again makes a new run; the
  products already marked come back as reprints and stay Printed.
- **The preview is a picture of the PDF's page** made on this Mac (`sips`), shown on white; if it cannot be made the
  dialog says so and Generate still works.
- **A PDF whose name is taken gets " (2)", " (3)" …**; a name Pricing sends is cleaned so it can only land in the
  Labels folder.
- **If Preview cannot be opened** the PDF is still saved and the dialog says where.
- **macOS asks once** to let Session Manager use the Documents folder, the first time a PDF is saved.
- **Outcome words in the check list** (09C names only the footer): "Will be marked Printed", "Reprint · stays
  Printed", "No price · stays Not printed", "Can't generate".
- **The check can't start while labels are being generated** (one dialog at a time).

## 7. Environment pointers
Commands and caveats: `master_plan.md` §7, `docs/guide/05_working.md`. While working: `cargo test -p psm-core --lib
mechanisms::labels` · `cargo test -p psm-integrations --lib conversations::beyo_pricing::labels` · `cargo test -p
psm-infra --lib label_files` · `cargo test -p psm-testkit --test suite <stem>::` · `cargo test -p psm-desktop`
(regenerates `apps/desktop/src/services/bindings.ts` through `export_bindings`; never hand-edit) · `cd apps/desktop &&
npx vitest run <files>` · `npm run typecheck` · `cargo clippy -p <crate> --all-targets -- -D warnings` · `cargo check
--workspace`. One filter per `cargo test` invocation. Closing gate of every milestone (once): `cargo test --workspace`
· `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run
typecheck` · `npm test`. One cargo command at a time, never in the background, never piped. Known intermittent tests:
`docs/guide/04_change_impact.md` §Fragile spots. No new dependency is expected (`uuid` v4 is in `psm-core`, `chrono`
in `psm-integrations`, `reqwest` and `tokio` in the adapter crate). `/usr/bin/sips` and `/usr/bin/open` are part of
macOS; the one infra test that runs `sips` (L3·C4(c)) needs no network and no app. Checked at `ae91079`: every
existing path this plan set cites resolves (script), and the cited code was read at that commit; lines move as
milestones land, so symbols are named beside them.

## 8. Log
- **v1, 2026-11-23 (planner).** Written from 09C, the Pricing integration package (read 2026-11-23) and the code at
  `ae91079`. One owner card (§2A). Points for the orchestrator in `handoffs/reviewer/handoff_labels_plan_r1.md`.
- **v1.1, 2026-11-23 (orchestrator): L1 projection r0 amendments**
  (`handoffs/reviewer/handoff_L1_labels_provider_and_marker_projection_r0.md`, AMENDMENTS_REQUIRED, 0 owner cards).
  §3 trace grammar admits a §4 registry entry (D12); ACC.2: `connected()` applies only ACC.2, ACC.4 governs every
  other label call (D2). Everything else landed in the L1 plan (its Review log). Counts unchanged (§2).
- **v1.2, 2026-11-23 (orchestrator): L3 projection r0 amendments**
  (`handoffs/reviewer/handoff_L3_generate_and_download_projection_r0.md`, AMENDMENTS_REQUIRED, 0 owner cards).
  §4 Files: `raster`'s temporary PDF lives in `<previews>/labels/` (D1); MC-L2.1: `position` = the index among the
  merged items (D4); MC-L2.8: a `Reprinted` item keeps the check item's `printed_at`, else the run's `created_at` (D5);
  MC-L2.9: a failure's `sku` / `title` come from the check item first (D3). Everything else landed in the L3 plan (its
  Review log). Counts unchanged (§2).
