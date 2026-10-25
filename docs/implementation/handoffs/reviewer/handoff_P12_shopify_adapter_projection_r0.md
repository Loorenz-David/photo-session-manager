---
plan: P12_shopify_adapter
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-10-26
actor: Claude Opus 5.5 (pipeline-projectionist, plan-projection doctrine under docs/process/flow.md)
---
# Projection of P12_shopify_adapter (round 0)

The Shopify phase can be built, but the plan is not ready to hand to an implementer. Several test rows assume
Shopify behaves in ways it most likely does not. Creating a product with its SKU is probably two calls, not one,
so a crash between them could leave a duplicate product. Saving a batch of fields is probably all-or-nothing.
Number fields cannot round-trip byte for byte. Each of these would produce tests that agree with the code and
disagree with Shopify. Three more gaps stop the tests being written at all: tests have no legal way to obtain the
"confirmed publish" token every write needs; nothing in any phase connects the adapter to the app; and the
"delete photo from Shopify" call that the next phase depends on is not in any phase. One question needs you:
whether the shop the one-time probe writes to is a test store or your live shop. Everything else the orchestrator
can fix in the plan text. The implementer should start only after that and after your answer.

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1: Which Shopify store may the one-time probe write to?**
- **Question:** Is `68c5b4-6a.myshopify.com` a development/test store, or your live shop?
- **Story:** Before the Shopify part is built, the session runs a probe once. It creates a hidden draft product
  "PSM-PROBE", uploads three tiny test photos and a file, and adds about nine temporary custom fields to the
  store's field list, then deletes all of it. That shop holds your real field list (warehouse locations, reviews,
  Etsy text). If it is the live shop, those temporary fields show up in your admin for a few minutes. If the probe
  stops halfway, a draft product or test fields stay behind until someone deletes them.
- **Branches:**
  - *Test store:* the probe runs as planned.
  - *Live shop, writes OK:* the probe runs there. Anything it fails to clean up is listed for you.
  - *Live shop, no writes:* the field list is read from the live shop without changing it. All writes go to a
    free development store you create, with its own token.
- **Recommendation:** if it is the live shop, take the third branch. The book's own plan was "dev store until
  upload is proven".
- **On silence:** the Shopify phase does not start. It is already waiting for your token.
- **Trace:** P12 task 0, Precondition, Notes · book 01 H1 · raw_intention.md:123.

## Gate check
- Intention status header: **RATIFIED** (owner, 2026-10-24; intention.md:3). Pass.
- No upstream gate handoff for P12 in `OWNER_DECISIONS_PENDING` (handoffs/reviewer has none for P12; STATUS.md
  "Needs the owner: Nothing"). Pass.
- `depends_on` [P10_attributes]: APPROVED (master_plan.md:80). Pass.
- Size: 8 criteria, 54 rows, 0 `[rule]`. I counted the table myself: C1 4, C2 1, C3 2, C4 20, C5 16, C6 7, C7 2,
  C8 2. That matches the plan's own count.
- Precondition (token) is the orchestrator's check, not mine. It is not met yet (STATUS.md:35).
- **Projection limit, stated:** this session had no web access (WebFetch/WebSearch were refused), so
  shopify.dev could not be read. The Shopify-behaviour claims below (L1, L3, L4, L15, L17, L18, L7) come from the
  projectionist's knowledge of the Admin GraphQL API and are marked *believed*. Each amendment turns the claim into
  an explicit probe question for task 0. If a belief is wrong, it costs one probe line, not a fix round.

## Decision ledger
Classification: **PG** plan gap (amend the plan) · **IG** intention gap (route upstream; orchestrator fold-back
into 07A/09) · **FC** free choice (record an explicit delegation).

| # | Decision point the artifacts do not determine | Class | Proposed routing |
|---|---|---|---|
| L1 | **`item_create` atomicity vs M18.** MC-11 prepare (07A:56–59) assumes `item.create` leaves a product that a later search by SKU finds. *Believed:* since API 2024-04, `productCreate` takes no variants, SKU or barcode. The default variant's SKU (`inventoryItem.sku`) and barcode need a second mutation (`productVariantsBulkUpdate`), or the single-call `productSet` with one default variant. If a kill falls between create and the SKU write, or before `item_links` is written, resume searches by SKU, finds 0, and creates a **second product**. Shopify's product search is also index-backed, so a product created seconds earlier may not be found yet. Either path silently breaks M18. No SV id covers create or identity shapes. | IG + PG | Add **SHOPIFY-VERIFY-7** to task 0 (and to 07A's Grounding list, via fold-back). It records the create mutation that sets status DRAFT + default-variant SKU + barcode in one call, if one exists (expect `productSet` with `synchronous: true`), and the identity-write mutation. Extend **SV-6**: search by SKU immediately after create, then at +5 s and +30 s, and record whether the product is found. If there is no atomic create, or search lags, MC-11 needs a recovery lookup that does not depend on the search index. One example: a deterministic handle from `folder_key`, looked up by handle before create. That is a 07A MC-11 amendment by the orchestrator, and the implementer stops (L14). |
| L2 | **C3(a)** says "request has `status: DRAFT`, variant `sku`, `barcode`": one request. Per L1 this request shape may not exist. | PG | Rewrite: "returns the new external id. The create request(s) are the SV-7 shape. A read-back of the product shows `status DRAFT` and the default variant's `sku`/`barcode` = the inputs". |
| L3 | **C3(x) `item_set_identity`.** (1) The boundary omits `&ConfirmedRun`, though Goal says every mutating method takes one. (2) The mutation shape is not grounded (*believed*: SKU lives on `inventoryItem`, barcode on the variant, both written via `productVariantsBulkUpdate`). (3) The return value is unstated ("read-back returns K-2" from what?). (4) Where the diff happens is unstated. P14 C3(c) (P14_publish_update.md:59) expects `item_set_identity` to send `sku` only and never clear a barcode the app does not hold. | PG | Signature `item_set_identity(run, external_id, sku: &str, barcode: Option<&str>) -> Result<RemoteIdentity{sku, barcode}>`. The adapter reads the default variant, writes only differing fields (a `None` barcode is never sent), and returns the read-back. C3(x) asserts: one recorded mutation whose variables carry `sku "K-2"` and no barcode, and the return value `{sku: "K-2", barcode: "123"}`. Grounded by SV-7. |
| L4 | **C5(p) partial failure (rule 17).** *Believed:* `metafieldsSet` is atomic per call (if any entry fails, none in that call persist). Its `userErrors` (`MetafieldsSetUserError`) point at entries by `elementIndex` / `field` path, not by key. So "the other keys reported written" cannot happen inside the failing batch: the test and the code would agree with each other and not with Shopify. Also `Error::AttributeVerification{keys}` does not exist (crates/core/src/error.rs:1–43 has no such variant), and error.rs is outside the perimeter. | PG | SV-5 records atomicity explicitly (one deliberate invalid value in a 3-entry call, then read back the other two). C5(p): `Err(attribute_verification{keys: [the two]})`, keys mapped from `elementIndex`. The row makes no claim about other keys in the same batch; keys of earlier, successful batches are the only ones reported written. The authoritative check stays P14's canonical read-back. Add `crates/core/src/error.rs` to the perimeter. |
| L5 | **C5(a–n) "byte for byte (identity)" contradicts MC-9.** `canonicalize` strips trailing zeros (crates/core/src/mechanisms/values.rs tests: money `"3.00"`→`"3"`, rating `"10.0"`→`"10"`). Shopify values such as the rating scale (the reference shows `scale_min=1.0; scale_max=5.0`), money `"x.00"` and dimension numbers will not come back byte-identical. The implementer must either fail the row or choose fixture values with no trailing zeros, which is bending the fixture. M20's "round trip is the identity" is satisfied at canonical level, because writes happen only when canonical values differ (MC-13). Also: the list omits `weight`; "money/url" bundles two cases in one row; and the platform→core direction (`currency_code`→`currency`, `scale_min/max`→`min/max`, rich-text tree→paragraphs, list JSON-string→array) has no named function. | PG | Rewrite C5(a–n): for each fixture string `s`, `c = canonicalize(read(s))` equals the stated canonical JSON, **and** `canonicalize(read(to_platform_string(c))) == c`. Byte equality `to_platform_string(c) == s` is kept only for types whose fixture string is already in normal form (text, textarea, choice, url, color, boolean, date, integer, list types). Add `weight`; split money and url (C5 becomes 16 + 1 rows; still 8 criteria). Add `type_map::from_platform_string(core_type, platform_type, s) -> Json` to **Core mechanisms**. Delegate (FC) how a non-representable rich-text value is signalled. Recommend `attributes_read` returns it verbatim, flagged so the service stores it `valid = false` (MC-9). |
| L6 | **C4 input lacks 9 of the types it tests.** The 68 definitions (counted: 68) use 13 types. None is `number_decimal`, `weight`, `volume`, `boolean`, `date`, `date_time`, `color`, `money` or single `product_reference`. So rows (h)(j)(k)(l)(m)(n)(o)(q) and part of (t) have no input in "the 68-definition fixture". Outcomes are also underspecified: writability is stated only for (t); `validations` for dimension/weight/volume must carry `units` (the core reads `validations.units`: crates/core/src/domain/attribute.rs `Definition::units`), but no unit codes are named; rating's validations form is "min/max in validations", with no exact form. | PG | C4 input = the captured `metafieldDefinitions` response (68) **plus** the captured read-back of task 0's scratch definitions (`psm_probe_*`, one per missing type). Both are captured, not hand-written. Outcome cell per row: `writable` true for (a)–(s) except (n); false for (n) and (t). `validations.units` = the unit codes SV-1 records for that Shopify type. Rating `validations = {"min": "<canonical>", "max": "<canonical>"}` from `scale_min/scale_max`. |
| L7 | **Writability vs definition access.** Several definitions belong to other apps' namespaces (`reviews.*`, `shopify--discovery--*`, `globo--filter--*`, `app.*`). *Believed:* a definition's `access.admin` can forbid writes from our token. MC-13 makes writability depend on type only, so a photographer who fills `reviews.rating` would fail every publish's update step with `attribute_verification`. | IG + PG | SV-1 records each definition's `access` and one write attempt per foreign namespace on the probe product. `writable = type-writable ∧ access permits`. 07A MC-13 gets a one-line fold-back. |
| L8 | **No legal way for tests to get a `ConfirmedRun`.** The plan gives it a "crate-private constructor used by P13" (plan:38; P13 plan:43–44 `from_persisted` crate-private). The tests live in `psm-testkit`, a separate crate, so every mutating row (C3, C5(o)(p), C6(a)(c)(e), C7, C8(b)) cannot be written. The type is also listed in two homes (`ports/commerce.rs` and `domain/publish.rs`). `Error::NotConfirmed` does not exist yet. | PG | Recommend: P12 owns the single constructor `ConfirmedRun::from_persisted(&dyn Store, run_id) -> Result<ConfirmedRun>`. It is public, fields private, no other constructor, and it fails `not_confirmed` unless the `publish_runs` row has `confirmed_at`. Add a `Store::confirmed_run(id)` read (store.rs + sqlite impl in the perimeter) and a `TestApp` helper that inserts a confirmed run (its caller is every mutating row). Home: `domain/publish.rs`. Amend P13 task 1 to reuse it. (Alternative, weaker: a feature-gated test constructor. Features unify under `cargo test --workspace`.) |
| L9 | **Media delete primitive is in no phase.** MC-12 step 3 deletes `(current − D) ∩ (own ∪ announced)`, and "a not-found reply counts as deleted". P13 C5(f) relies on it. Neither P13's nor P14's perimeter contains `ports/commerce.rs` or the Shopify `adapter.rs`, so the real adapter would never get it. | PG | Add `product_delete_media(run, product, media_ids)` to P12's methods. Task 0 records its request shape and not-found reply (SV-4 section). Add row **C6(h)**: a fixture not-found reply → `Ok` (MC-12 step 3). |
| L10 | **Adapter wiring is owned by no phase.** No plan builds a `ShopifyAdapter` from `settings.integration` into `App` (crates/integrations/src/lib.rs:49–67 has no commerce field). None imports `attributes_schema` through `AttributesService::import_definitions` (crates/core/src/services/attributes.rs:170) plus link rows, or adds `item_search` to `ItemsService::search` (deferred "to P12" by P07 plan:52–53). P13–P16 perimeters do not include it. P12's Manual try-out step 2 therefore cannot happen in P12. "Health shows Shopify: token valid" is P16's try-out (P16 plan:71; `check_token` arrives there). | PG (master plan) | Recommend: (a) rewrite P12's Manual try-out to what P12 delivers: the evidence doc, the probe product created and deleted, the suite green; (b) master-plan amendment assigning the wiring (App commerce field from settings, schema refresh into definitions + links, search integration) to the start of P13, the first consumer, with its own row. The other option, a P12 wiring task, would add files and a row to an already large phase. |
| L11 | **`RemoteItem` / `RemoteMedia` fields are not defined.** C1(b) shows `{external_id, sku, title, has_media}` and C2(a) a media list. The barcode is needed by L3 and P14. P07's New-session dialog shows a thumbnail. | PG | Define once in `domain/publish.rs`: `RemoteItem{external_id, title, sku: Option, barcode: Option, thumbnail_url: Option, media: Vec<RemoteMedia>}` and `RemoteMedia{id, filename, status, position}`. `item_search` may leave `media` empty. C1(b)'s exact outcome lists the fields. |
| L12 | **C5(o) "token in the header, never in the body" cannot be observed.** `HttpTransport::post_graphql(query, variables)` carries no headers, so FakeTransport cannot see a header. | PG | C5(o) asserts the ⌈30/L⌉ batching, and that no recorded `query`/`variables` contains the configured token string. Header placement is delegated (FC) to `ReqwestTransport`, optionally checked against a loopback server (rule 9 allows it), with no extra row. |
| L13 | **Sync/async transport and dependencies.** All ports are sync (`EditingProvider`, `Store`). `reqwest` is pinned without `blocking` (master_plan.md:140). `crates/integrations/Cargo.toml` has no reqwest, and neither it nor `Cargo.lock` is in the perimeter. *Known pitfall:* a `reqwest::blocking` client created or dropped inside a tokio runtime panics, and Tauri async commands exist (e.g. commands/export.rs:29). | PG + FC | Perimeter += `crates/integrations/Cargo.toml`, `Cargo.lock`. Note the chosen reqwest features in master plan §6.2. Delegate the mechanism (blocking client used only off the async runtime, or a private runtime inside `ReqwestTransport`), with the no-panic constraint stated. |
| L14 | **No stop rule for when the probe disagrees with a contract.** The Notes say "amend 07A through the orchestrator", but not what the implementer does in the meantime, or on relaunch. | PG | The implementer **absorbs** literal values of Shopify-owned shapes that rows cite by SV id: L, position base, uniquifier text, poll shape, value strings. Each is recorded under Assumptions. The implementer **stops** (`BLOCKED`, after a `CHECKPOINT` commit of the evidence doc + fixtures) when an answer breaks a contract mechanism: media/file filename unreadable (MC-11 adoption), no atomic create or search lag (L1), access-blocked writes (L7), or partial-failure semantics beyond L4. On relaunch the probe is **not** re-run if `shopify_admin.md` exists. |
| L15 | **C6(g)'s uniquifier is a guess.** `k-ab12cd34ef56-1.jpg` assumes a `-1` suffix. *Believed:* `fileCreate` has `duplicateResolutionMode` (`APPEND_UUID` default, `RAISE_ERROR`, `REPLACE`), which points to a `_<uuid>`-style suffix. MC-21 revision names (`-r<n>`) also share the stem, so the stem matcher must tell a uniquifier from a revision. | PG | C6(g) input = "the two filenames SV-2 records", with no literal. C7(a) states the `duplicateResolutionMode` sent: recommend explicit `APPEND_UUID`, so SV-2's observation is the behaviour relied on. Pass a note to P13 that the adoption matcher is `stem + (SV-2 uniquifier)? + ".jpg"`, never a plain prefix. |
| L16 | **C2(a) and C6(e) need 3 media.** Task 0 uploads the JPEG twice, which gives 2 media. | PG | Task 0 uploads three: two under one filename (SV-2) and one distinct. It reorders all three (SV-4). |
| L17 | **C6(c): which mutation creates media is not settled.** The method is named `product_create_media`; ch09 says "`productUpdate` media". *Believed:* `productCreateMedia` is deprecated in recent versions in favour of `productUpdate(media:)` / `productSet`, and the new media id comes back through `product.media`. | PG | Task 0 records the 2026-10 media-create mutation and where its media id is returned (SV-2 section). C6(c) cites it. The method name can stay. |
| L18 | **C6(a)/(b): upload method.** The transport has only `put_bytes`, but `stagedUploadsCreate`'s `httpMethod` is not stated (*believed*: the default for IMAGE is POST multipart). With PUT, the target's `parameters` are sent as request headers, so "in order" means nothing. | PG | C6(a): the request also carries `httpMethod: PUT`. C6(b): `put_bytes` URL = the fixture URL, and its headers equal the fixture `parameters` as a name→value map. The PUT form is confirmed by SV-2's capture. |
| L19 | **Polling parameters.** C6(d)(f) and C7(a) need a poll interval and a give-up rule, and tests must not sleep for real. C6(d)'s `Err` variant is unnamed. | FC + PG | Delegate interval and attempt cap. Require them to be injectable (tests use 0 ms). Exhaustion → `Error::Timeout`. C6(d) `FAILED` → `Error::Provider(<fixture message>)` (an existing variant). FakeTransport keying (delegated) must support per-key response sequences. |
| L20 | **Sanitizing fixtures for a public repo; probe cleanup; token scopes.** Only the shop domain and token are redacted. Staged-upload targets carry signed URL credentials (`X-Goog-Credential`/`Signature`-style query and parameters), and resource/CDN URLs carry shop file paths. Cleanup on a mid-run failure is unspecified. The Precondition names no token scopes. | PG | Redaction list = shop domain, token, every signed-URL query string and staged `parameters` value (placeholders, shape kept). The probe records each created id before the next step and prints leftovers on failure (Review log). Precondition lists the scopes the probe needs: product, file and metafield-definition read/write. |
| L21 | **`attributes_schema` must not emit `product.*` keys.** Task 3 says "`product` for title/description/tags". The built-ins own those keys, and `import_definitions` rejects duplicates of them (attributes.rs:179–186). | PG | Task 3: the adapter emits no Product-group definitions. Title, description and tags stay the core built-ins. |
| L22 | **Persistence and inputs of the attribute methods.** Task 3 says `attributes_schema` "writes core definitions + `attribute_definition_links`", but an adapter has no Store, and the Store has no link methods (only `replace_attribute_definitions`, store.rs:71). `attributes_write` needs each key's platform type and namespace. For `media_list` it sends file ids, not the canonical `{"excluded":…}`. | PG | `attributes_schema() -> Vec<(Definition, DefinitionLink{platform_type, platform_meta: Json})>`, opaque to core and persisted by the wiring owner (L10). `attributes_write(run, external_id, &[WriteValue{key, link, value}])`, formatted by `to_platform_string`; `media_list` carries file ids (C7(b)). C4's boundary is the return value. |
| L23 | **Search breadth.** ch09's capability is "find items by SKU, barcode or title", and the design handoff says "Search Shopify by title or SKU". P12 delivers exact SKU only. | FC | Record: P12 = exact SKU (MC-11). Title/barcode search goes with the wiring owner (L10), or is named under "Not yet" in that phase's narrative. |
| L24 | **Minor delegations to record.** Cursor pagination of `metafieldDefinitions` (68 fit in one page). A single newline inside a `description_html` paragraph (kept as is). `capabilities()` has no row. `upload` (PUT to the staging host) takes `&ConfirmedRun` or not. Title and description write methods have no row. | FC | List under Delegated choices. Recommend `upload` takes `&ConfirmedRun` for uniformity with MC-10. |
| L25 | **Stale pointers.** master_plan.md:17 and :57 say MC-1…MC-21 and M1–M28 (MC-22 and M29 exist). ch09 says the type map is in `evidence/design_handoff.md`, but it is in 07A MC-13. 05B MC-22 is cited by C3(x) but missing from Read-first. | PG / IG | Coordinator fixes the master plan and Read-first. A one-line book fold-back fixes ch09. |

## Reality checks
- **Files expected to change.** The new files are correctly absent today: `crates/integrations/src/commerce/**`,
  `examples/shopify_probe.rs`, `crates/testkit/src/fake_transport.rs`, `fixtures/shopify.rs`,
  `tests/p12_shopify_adapter.rs`. **Not marked new but absent:** `crates/core/src/ports/commerce.rs` and
  `crates/core/src/domain/publish.rs`. **Missing from the perimeter:** `crates/core/src/{error.rs, ports/mod.rs,
  domain/mod.rs, ports/store.rs}`, `crates/infra/src/sqlite/store.rs` (L8), `crates/integrations/{Cargo.toml,
  src/lib.rs}`, `Cargo.lock`, `crates/testkit/src/{lib.rs, fixtures/mod.rs, test_app.rs}`.
  `crates/infra/src/settings_file.rs` exists, and `integration` is already `Option<serde_json::Value>` (line 23),
  so "integration block read" means a typed parse of an existing key.
- **Citations.** master plan §3 (secrets, line 64), §6.1, §6.4, §6.5, §7.5 resolve and say what the plan claims.
  Book 01 H1/H4/H7, 14, 09, 08 §Schema, 07A MC-10…14 and Grounding owed, 05A MC-9: resolve. The reference file
  holds exactly 68 definitions (counted). 05B MC-22 is cited but not listed (L25).
- **Prior-phase outputs used.** `CoreType` (18 variants), `Definition` with `validations.units`, `canonicalize`
  and `import_definitions` exist as P10 left them. `attribute_definition_links` exists only as a table
  (002_domain.sql:89–95), with no Store methods. `CommerceAdapter`, `FakeCommerce` and `ConfirmedRun` do not
  exist yet. P11's form uses the hand-built testkit `shop_definitions()`, not adapter output.

## Criteria decidability (two questions per row: can the test be written now with one exact outcome? is the fixture producible by Shopify at 2026-10?)
| Rows | Writable now? | Producible? | Finding |
|---|---|---|---|
| C1(a) | yes, once SV-6 is captured | yes | — |
| C1(b) | no: `RemoteItem` fields undefined | yes | L11 |
| C1(c) | yes | probe makes one product; the 2-product response is a declared edit of the captured shape | declare "synthetic from SV-6 shape" |
| C1(d) | yes | only if SV-6 shows case-insensitive search; otherwise the row is a declared synthetic guard | record per SV-6 |
| C2(a) | yes after L16 | needs 3 media (L16); filename readability is SV-2 | L16, L14 |
| C3(x) | no: no run, return value or diff site | mutation shape not grounded | L3, L1 |
| C3(a) | no: single-request assumption | believed not producible | L1, L2 |
| C4(a)–(g), (i), (p), (r)–(t) | partly: writability and validations not stated | yes (types the shop has) | L6, L7 |
| C4(h), (j)–(o), (q), part of (t) | no: no input | only via scratch-definition capture | L6 |
| C5(a)–(n) | no: byte identity fails for decimal forms; weight missing | values for 5 types only via scratch | L5, L6 |
| C5(o) | half: header half unobservable | yes (L from SV-5) | L12 |
| C5(p) | no: error variant missing; "others written" | believed not producible (atomic) | L4 |
| C6(a) | yes after `httpMethod` added | yes | L18 |
| C6(b) | "in order" undefined for headers | PUT form per SV-2 | L18 |
| C6(c) | yes once the mutation is fixed | mutation choice per version | L17 |
| C6(d) | `Err` variant unnamed; real sleeps | yes | L19 |
| C6(e) | yes after SV-4 | needs 3 media | L16 |
| C6(f) | yes | yes | — |
| C6(g) | literal names are invented | believed wrong suffix form | L15 |
| C7(a) | yes after the duplicate mode is stated | yes | L15 |
| C7(b) | yes | yes | — |
| C8(a) | yes (exact string given) | ours | — |
| C8(b) | yes | yes | — |
| all mutating rows | **no**: no way to construct `ConfirmedRun` from testkit | — | L8 |

## Trace verification
- Rows → ledger: every trace cell resolves (MC-11, M19, MC-22/M29, M18, M20, MC-13, M16, MC-12, M21). C6(c)'s
  M18 and C2(a)'s M19 are indirect (they are inputs of the mechanism) but acceptable. The new C6(h) (L9) traces to
  MC-12 step 3.
- Ledger → rows: MC-10/M17 is claimed (Read-first, Goal: "every mutating method takes `&ConfirmedRun`"), but no
  P12 row serves it. It is enforced by signature, and P13 C2(a) serves M17 at runtime. Recorded as accepted, not
  a finding. After L8, the constructor's refusal (`not_confirmed`) could be one row if the coordinator wants it.

## Write perimeter of this session
This handoff file only. No plan, intention, code, tracker or Review-log edits; no commits; no archgraph (none in
this repo).
