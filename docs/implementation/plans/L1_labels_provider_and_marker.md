---
phase: L1_labels_provider_and_marker
title: The labels provider and the label marker — every session row and the session page say whether the product's price label is printed, missing or made stale by a price change, as Pricing says it, and follow Pricing within the regular 10-second check; Settings → Pricing says whether the key may read labels; nothing about labels is stored on this Mac except that one fact about the key
difficulty: high
projection: done (r0 amendments applied 2026-11-23)
depends_on: []
blocked_on: none
---
# L1 — The labels provider and the label marker

**Tracker:** IMPLEMENTED (2026-11-23, claude-opus-5.5 implementer: 4 units, 109 rows → 109 tests + 8 core-mechanism tests; workspace suite, clippy, fmt, typecheck, npm test green)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `09C` §3, §4, §7, §8 by contracts **ACC**, **MC-L3** and **MC-L4** (`docs/implementation/master_plan_labels.md` §5), and the adapter half of MC-L1 and MC-L2. (U1) The labels vocabulary and port in core, the Beyo Pricing adapter's three reading calls, its error mapping, its sentences and the two id forms. (U2) The adapter's check, preview, run and download calls over a binary transport. (U3) The access rule, the display copy, the status read, the label events in the 09B pull, `LabelsChanged`. (U4) The commands, Settings → Pricing's Labels line, the marker on Open / Published / All rows and on the session page, the ⋯ entry in its disabled states. **Not here:** the Labels box and its list (L2); anything that calls `run`, `plan`, `preview` or `download` from the service or the screen (L3: in L1 these four exist only as adapter methods with adapter tests); the enabled ⋯ entry (L3).

## How to work this plan (flow.md §10, master_plan_labels.md §3)
One runner phase of 4 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `L1·C<n>(<row>)`; tests named after their row (`l1_c1a_<slug>`). Open details: the smallest option consistent with 09C and ACC / MC-L3 / MC-L4, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_labels/L1_labels_provider_and_marker.md`. **Never call Pricing, Shopify or a CDN, never launch the built app, never read or print the Pricing key** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Vocabulary, port, the adapter's reads, errors, sentences, ids | 6 | 35 | **yes** |
| U2 | The adapter's check, preview, run and download | 4 | 23 | **yes** |
| U3 | Access, the display copy, the status read, label events in the pull | 4 | 28 | **yes** |
| U4 | Commands, the Settings line, the marker, the ⋯ entry (disabled states) | 5 | 23 | **yes** |
| **Total** | | **19** | **109** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `docs/implementation/master_plan_labels.md` §3, §4, **§5 ACC, MC-L3, MC-L4, MC-L2.12, MC-L2.14**, §6 · book `09C` §3, §4, §7, §8, §9, §10 · book `09B` §4, §5 (the connection and the pull this extends) · `docs/implementation/master_plan_conversations.md` §4 (Adapter, Service, Secret), §5 MC-C1.1–.3, .6 · the Pricing package (read-only): `LABELS.md` §1, §3.6, §4, §6, `API.md` › Price labels and › Errors, `EVENTS_AND_WEBHOOKS.md` › Price-label events, `openapi-integration.json` (the components named in master plan §3 Rule 17) · `docs/guide/03_rules_and_principles.md` (H6, H7, H8, CR2, CR5) · `docs/guide/04_change_impact.md` rows "A Tauri command signature", "A new `AppEvent` variant", "A new `ALTER TABLE … ADD COLUMN` migration", "A Pricing reply shape or endpoint".

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Whether the key may read labels is remembered,** so a key without the permission is asked once, not at every launch.
- **A key that cannot read labels keeps its conversations:** a refused label question never marks the Pricing connection as refused.
- **Markers are read again only when the page shows other sessions,** and never more than once per 10 seconds for the same products.
- **A stale label wins over "No price yet".**
- **"Label printed · 4 Nov"** shows the year when it is not this year.
- **A product Pricing does not list** (older than its Work-from date) shows a marker only when Pricing knows it as printed or reset.
- **Test connection clears the markers on screen** and they are read again.
- **Until label generation lands (L3)** the ⋯ menu shows the label entry only where it explains why it is off ("Publish first so Pricing can see it", or the connection's reason).

## Unit 1 — Vocabulary, port, the adapter's reads, errors, sentences, ids

**Unit id:** `L1·U1` · **Rows:** L1·C1–C6 · **Changes app files:** yes

### Goal
`psm_core::domain::label`, `psm_core::ports::labels::LabelsProvider`, `PricingHttp::fetch`, and `impl LabelsProvider for BeyoPricingAdapter` for `products`, `status`, `templates`, `describe`, `product_id_of`, `commerce_id_of`, with the error mapping every label call shares.

### Read first
Master plan §4 (Vocabulary, Errors, Ports, Transport, Adapter paths, Words), §3 Rule 17 · `crates/integrations/src/conversations/beyo_pricing/{mod.rs, adapter.rs (`send` :49, `encode` :71, `envelope` :88, `read_error` :95, `parse` :120, `time` :126, the tests' `Recording` :417-447), transport.rs (`PricingRequest` :41, `PricingHttp` :60, `ReqwestPricing::send` :129-173), types.rs (`ErrorEnvelope` :118)}` · `crates/core/src/ports/conversations.rs` (the shape of a provider port) · `crates/core/src/domain/conversation.rs` (`ConnectionState` :216) · `crates/integrations/src/conversations/mod.rs` (`NoTransport` :141) · `crates/testkit/src/fake_pricing.rs` (`impl PricingHttp` :772) · the package's `API.md` examples for `labels/products`, `labels/status`, `labels/templates` and the schema components `IntegrationLabelProductPage`, `IntegrationLabelProduct`, `IntegrationLabelCounts`, `IntegrationLabelStatus`, `LabelStatusOut`, `IntegrationLabelTemplate`, `ErrorEnvelope`.

### Files
New: `crates/core/src/domain/label.rs`, `crates/core/src/ports/labels.rs`, `crates/integrations/src/conversations/beyo_pricing/{labels.rs, label_types.rs}`, `crates/integrations/src/conversations/beyo_pricing/fixtures/{labels_products.json, labels_status.json, labels_templates.json}`. Edited: `crates/core/src/{domain/mod.rs, ports/mod.rs}`, `crates/integrations/src/conversations/beyo_pricing/{mod.rs, transport.rs, adapter.rs}`, `crates/integrations/src/conversations/mod.rs`, `crates/testkit/src/fake_pricing.rs`.

### Core mechanisms (the only functions with unit tests)
None (the adapter rows are behaviour tests at the port).

### Ordered tasks
1. `domain/label.rs`: the L1 part of master plan §4 Vocabulary (`LabelStatus`, `ResetReason`, `LabelState`, `LabelsAccess`, `LabelFilter`, `LabelCounts`, `LabelProduct`, `LabelProductPage`, `LabelTemplate`, `CheckOutcome`, `PlanTemplate`, `PlanItem`, `Unresolved`, `PlanSummary`, `LabelSize`, `LabelPlan`, `RunStatus`, `RunResult`, `RunItem`, `RunCounts`, `LabelArtifact`, `LabelRun`, `LabelChange`, `LabelChangeKind`, `SessionLabel`, `LabelsError`). No provider or platform word in it (H7).
2. `ports/labels.rs`: `LabelsProvider` with the six methods this unit implements (`products`, `status`, `templates`, `describe`, `product_id_of`, `commerce_id_of`), signatures as master plan §4 Ports. U2 adds `plan`, `preview`, `run` and `download` to the trait together with their implementation, so each checkpoint compiles without a placeholder body.
3. `transport.rs`: `RawReply`, `PricingHttp::fetch` (master plan §4 Transport); `ReqwestPricing::fetch` (per-request timeout, `Accept: application/pdf, application/json`, body bytes untouched, `content-type` read from the answer); `NoTransport::fetch` → the same error as its `send`; the adapter tests' recording transport gains `fetch` with its own reply queue and records the request and the timeout (make it reachable from `labels.rs` tests, e.g. a `#[cfg(test)] pub(super) mod testing`); `FakePricing::fetch` records the request like `send` and answers `404 not_found` (its label routes come in U3 and later).
4. `label_types.rs`: the wire types of the three reads (camelCase, unknown fields ignored, nullable and optional fields as `Option` with `#[serde(default)]`), serialisable (the fake uses them). An enum field the domain never reads (here `LabelStatusOut.source`) is left out of the struct or kept as `String`, never a strict enum; `labelStatus` stays strict (L1·C1(g)) (projection r0, D4(iii); the same rule for U2's types, U2 task 1).
5. `labels.rs`: `impl LabelsProvider for BeyoPricingAdapter`: `products`, `status` (no request for an empty id list), `templates`; one error function for label calls (master plan §4 Adapter paths "Status mapping"; **never `ProviderError`**, ACC.5); `describe` (master plan §4 Words); `product_id_of` (`gid://shopify/Product/<one or more ASCII digits>` → the digits; anything else none) and `commerce_id_of` (one or more ASCII digits → `gid://shopify/Product/<digits>`; anything else none).
6. The three fixtures (master plan §3 Rule 17 and "Secrets and company data": the examples' shape, neutral values, a `"_source"` key the wire types ignore). `labels_products.json`: one product with every field set (`labelStatus: "notPrinted"`, `labelResetReason: "price_changed"`, `priceMinor: 890000`), `nextCursor: "bzoxMDA"`, the six counts `214, 180, 34, 12, 168, 46`, `workFrom: "2026-11-19"`. `labels_status.json`: one known product (`printed`, `labelPrintedAt: "2026-11-20T09:12:00Z"`, `hasPrice: true`, `source: "export"`, `printedPriceMinor`, `printedSku`) and one unknown (`label: null` with its `error`). `labels_templates.json`: one template (`widthMm: 70`, `heightMm: 100`, `isDefault: true`, two product types).
7. Tests for the rows, inline in `labels.rs`.

### Criteria (6 criteria, 35 rows)
Boundary: `BeyoPricingAdapter` as `LabelsProvider` over the recording transport (server `https://pricing.example`, key `k1`, store `S` = the C1 fixtures' store id). Outcome = the returned value and the recorded requests. "One GET `<path>`" = exactly one request, method GET, that address, bearer `k1`, no body.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L1·C1(a) | `products(S, MissingPriceTag, None, None, 25)` | the transport answers 200 with `labels_products.json` | one product `{product_id "15531808096409", sku Some, title, product_type Some, price_minor Some(890000), state {NotPrinted, printed_at None, has_price Some(true), reset Some(PriceChanged)}, thumb_url Some, created_at Some(the fixture's time in ms)}`; `next_cursor Some("bzoxMDA")`; counts `{total 214, has_price 180, missing_price 34, missing_price_tag 12, printed 168, not_printed 46}`; `work_from Some("2026-11-19")` | MC-L3.1; 09C §5; Rule 17 |
| L1·C1(b) | same | same | one GET `https://pricing.example/integration/v1/stores/S/labels/products?filter=missingPriceTag&limit=25` | MC-L1.1; Adapter paths |
| L1·C1(c) | `products(S, f, Some("teak bord & co"), Some("a+b/c="), 500)` for `f` = `NotPrinted`, `Printed`, `All` | 200 with the fixture | the three addresses carry `filter=notPrinted`, `filter=printed`, and **no** `filter` parameter; each carries `q=teak%20bord%20%26%20co`, `cursor=a%2Bb%2Fc%3D`, `limit=500` | Adapter paths; 09C §5 |
| L1·C1(d) | `products` | a product with `sku`, `productType`, `priceMinor`, `labelPrintedAt`, `createdAt` null, `thumbUrl` and `labelResetReason` absent, `hasPrice: false`; `nextCursor: null` | those fields `None`; `state {NotPrinted, None, Some(false), None}`; `next_cursor None` | Rule 17 (nullable fields) |
| L1·C1(e) | `products` | a product with `labelStatus: "printed"`, `labelPrintedAt: "2026-11-20T09:12:00Z"` | `state {Printed, Some(1_795_165_920_000), Some(true), None}` | MC-L3.1 |
| L1·C1(f) | `products` | `labelResetReason: "sku_changed"`; `"barcode_changed"`; `"something_new"` | `reset` `Some(SkuChanged)`; `Some(BarcodeChanged)`; `None` | MC-L3.1 |
| L1·C1(g) | `products` | `labelStatus: "queued"` | `Err(BadReply(_))` | MC-L3.1 |
| L1·C2(a) | `status(S, ["15531808096409", "999"])` | 200 with `labels_status.json` | `[Some({Printed, Some(1_795_165_920_000), Some(true), None}), None]` | MC-L3.1; 09C §7 |
| L1·C2(b) | same | same | one GET `…/stores/S/labels/status?productIds=15531808096409,999` | Adapter paths |
| L1·C2(c) | `status` | an entry with `hasPrice: null`, `labelStatus: "notPrinted"`, `resetReason: "price_changed"` | `Some({NotPrinted, None, None, Some(PriceChanged)})` | MC-L3.1; Rule 17 (`hasPrice` nullable) |
| L1·C2(d) | `status(S, [])` | nothing queued | `Ok([])`; no request | MC-L3.3 |
| L1·C2(e) | `status(S, ["1", "2"])` | 200 with one entry only | `Err(BadReply(_))` (one entry per id is the contract) | Ports |
| L1·C3(a) | `templates(S)` | 200 with `labels_templates.json` | `[LabelTemplate {id, name, width_mm 70.0, height_mm 100.0, is_default true, product_types: the two}]`; one GET `…/stores/S/labels/templates` | MC-L2.2; ACC.2 |
| L1·C3(b) | `templates(S)` | 200 with `[]` | `Ok([])` | ACC.2 |
| L1·C4(a) | `templates(S)` | 401 with code `invalid_api_key` | `Err(NotAllowed)` | ACC.4; 09C §10 |
| L1·C4(b) | `status(S, ["1"])` | 403 with code `insufficient_scope` | `Err(NotAllowed)` | ACC.4; 09C §10 |
| L1·C4(c) | `products(…)` | 404 with code `not_found` | `Err(StoreGone)` | 09C §10 |
| L1·C4(d) | `products(…)` | 503 with code `product_list_unavailable` | `Err(ListUnavailable)` | 09C §10 |
| L1·C4(e) | `products(…)` | 500 with code `internal_error`; 502 with a non-JSON body | `Err(Unreachable(_))` for both | 09C §10 |
| L1·C4(f) | `status(S, ["1"])` | the transport fails with `Error::Timeout`; with `Error::Provider("http: connection refused")` | `Err(Unreachable(_))` for both | 09C §10; Rule 17 (transport shapes) |
| L1·C4(g) | `templates(S)` | 200 with `{"unexpected": true}`; 200 with the JSON string `"<html>"` | `Err(BadReply(_))` for both | Adapter paths |
| L1·C4(h) | `products(…)` | 422 with `{"error": {"code": "validation_error", "message": "Bad cursor."}}` | `Err(Rejected {code: "validation_error", message: "Bad cursor."})` | Adapter paths |
| L1·C4(i) | `products(…)` | the fixture with an extra unknown field on the page, on the product and on the counts | the same value as L1·C1(a) | Rule 17 (additive API) |
| L1·C5(a) | `describe(&NotAllowed, Some("shop-a.myshopify.com"))` | — | `Session Manager isn't allowed to print labels. Ask a Pricing admin.` | 09C §10 |
| L1·C5(b) | `describe(&StoreGone, Some("shop-a.myshopify.com"))`; with `None` | — | `This key is no longer tied to shop-a.myshopify.com`; `This key is no longer tied to the connected shop` | 09C §10 |
| L1·C5(c) | `describe(&NoProducts, None)` | — | `None of these products were found in Pricing.` | 09C §10 |
| L1·C5(d) | `describe(&RendererUnavailable, None)` | — | `Labels can't be generated right now. Try again in a moment.` | 09C §10 |
| L1·C5(e) | `describe(&Expired, None)` | — | `This file has expired. Generate the labels again.` | 09C §10 |
| L1·C5(f) | `describe(&ListUnavailable, None)` | — | `The product list isn't available right now.` | 09C §10 |
| L1·C5(g) | `describe(&Unreachable("x".into()), None)` | — | `Pricing can't be reached right now.` | 09C §10 |
| L1·C5(h) | `describe(&BadReply("x".into()), None)` | — | `Pricing answered in a way this version does not understand` | master plan §4 Words |
| L1·C5(i) | `describe(&Rejected {code: "no_template", message: "No template applies."}, None)` | — | `No template applies.` | 09C §10 ("the message as given") |
| L1·C6(a) | `product_id_of("gid://shopify/Product/15531808096409")` | — | `Some("15531808096409")` | MC-L1.3; 09C §5 |
| L1·C6(b) | `product_id_of` of `"gid://shopify/ProductVariant/15531808096409"`, `"15531808096409"`, `"gid://shopify/Product/"`, `"gid://shopify/Product/12a"`, `"product-3"` | — | `None` for each | MC-L1.3 |
| L1·C6(c) | `commerce_id_of("15531808096409")`; of `""`, `"12a"`, `"gid://shopify/Product/1"` | — | `Some("gid://shopify/Product/15531808096409")`; `None` for the other three | MC-L1.8 |

### Delegated choices
How the recording transport is shared between `adapter.rs` and `labels.rs` tests · whether the label error function lives in `labels.rs` or beside `read_error` · module-private helper names.

### Notes
- Hazard: `PricingHttp::send` turns every body into JSON (`from_utf8_lossy` for a non-JSON body, `transport.rs:152-158`). A PDF through `send` is silently damaged. Preview, download and run use `fetch`.
- Hazard: `read_error` / `write_error` (`adapter.rs:95`, `:104`) map 401/403 to `ProviderError::Refused`, which the conversations service turns into the `refused` connection state. **No label call may use them** (ACC.5).
- Hazard: `time` (`adapter.rs:126`) parses RFC 3339 with or without fraction; `workFrom` is a date (`YYYY-MM-DD`) and is passed through as text, never parsed as a time.
- `1_795_165_920_000` is 2026-11-20T09:12:00Z; compute it in the test from the text rather than trusting this number if they disagree.

### Unit test commands
`cargo test -p psm-integrations --lib conversations::beyo_pricing` · `cargo check --workspace` · `cargo clippy -p psm-core -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — The adapter's check, preview, run and download

**Unit id:** `L1·U2` · **Rows:** L1·C7–C10 · **Changes app files:** yes

### Goal
`plan`, `preview`, `run` and `download` of `LabelsProvider` on the adapter. Nothing in the app calls them before L3.

### Read first
Master plan §4 (Adapter paths, Transport), **§5 MC-L2.12, MC-L2.14** · the package's `API.md` sections `POST …/labels/plan`, `…/preview`, `…/runs`, `GET …/artifacts/…`, and the schema components `IntegrationPlanRequest`, `IntegrationRunItemIn`, `IntegrationPlan`, `IntegrationPlanItem`, `IntegrationPlanTemplate`, `IntegrationPlanIssue`, `UnresolvedItem`, `IntegrationPlanSummary`, `IntegrationPlanSize`, `IntegrationPreviewRequest`, `IntegrationRunCreate`, `IntegrationLabelRun`, `RunCounts`, `RunArtifactOut`, `RunItemOut`, `IssueOut` · U1's `labels.rs`.

### Files
New: `crates/integrations/src/conversations/beyo_pricing/fixtures/{labels_plan.json, labels_run.json}`. Edited: `crates/core/src/ports/labels.rs` (the four methods join the trait), `crates/integrations/src/conversations/beyo_pricing/{labels.rs, label_types.rs}`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. Wire types for plan, preview request, run request and run answer. **`RunItemOut.lifecycleResult` is parsed tolerantly:** `marked_printed`, `reprinted`, `no_price` map to `RunResult`; anything else, `"none"` and null included, maps to `None` (projection r0, D4(i): the schema's enum says `null`, but `API.md`'s run example sends `"none"` and `examples/client.ts` types it as a plain string; a strict enum would turn a 201 run that already marked products Printed into `BadReply`, and in L3 Try again returns the same run and the same error). Enum fields the domain never reads (`IntegrationPlanTemplate.source`, `RunItemOut.templateSource`, `requestedByKind`) are left out of the struct or kept as `String` (D4(iii)). The strictness the domain wants stays: `labelStatus` (L1·C1(g)), `outcome` (L1·C7(e)), the run's `status`.
2. `plan` (JSON through `send`), `preview` (`fetch`, `FILE_TIMEOUT`), `run` (`fetch` with `RUN_TIMEOUT`, the 201 body parsed from the bytes; a non-2xx answer's error envelope is parsed from the bytes too), `download` (MC-L2.12, `fetch`, `FILE_TIMEOUT`).
3. Fixtures. `labels_plan.json`: the `API.md` plan example with its three items written out in full (item 0 `markPrinted` with a template and `priceMinor: 890000`; item 1 `noPrice`, `hasPrice: false`, `priceMinor: null`; item 2 `fail` with one issue `{code: "missing_code_value", message: "Barcode: The product's variant has no barcode in Shopify."}`), one `unresolved` entry (`sku: "NOPE"`, `productId: null`, `sku_not_found`), `summary {items 3, ready 2, failing 1, markPrinted 1, reprint 0, noPrice 1}`, `sizes [{70, 100, 2}]`. `labels_run.json`: the `API.md` run example with every required field of `IntegrationLabelRun`, `RunArtifactOut` and `RunItemOut` present (status `partial`, counts `{2, 1, 1, 1, 0, 0}`, one artifact whose `downloadUrl` is `/integration/v1/stores/<S>/labels/runs/r1/artifacts/a1`, item 0 `rendered` / `marked_printed`, item 1 `failed` / `lifecycleResult: null` with its `error`, `unresolved: []`).
4. Tests for the rows.

### Criteria (4 criteria, 23 rows)
Boundary: as U1. "PDF bytes" = a byte string starting with `%PDF-1.4`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L1·C7(a) | `plan(S, ["1", "2"])` | 200 with `labels_plan.json` | one POST `…/stores/S/labels/plan` whose body is exactly `{"items": [{"productId": "1"}, {"productId": "2"}]}` (no `sku`, no `templateId`, no `idempotencyKey`) | Adapter paths; 09C L8; MC-L2.14 |
| L1·C7(b) | same | same | items: position 0 `{outcome MarkPrinted, template Some {name, 70.0, 100.0}, price_minor Some(890000), status NotPrinted, issues [], notes []}`; position 1 `{outcome NoPrice, price_minor None}`; position 2 `{outcome Fail, issues ["Barcode: The product's variant has no barcode in Shopify."]}`; `unresolved [{product_id None, sku Some("NOPE"), message: the fixture's}]`; `summary {3, 2, 1, 1, 0, 1}`; `sizes [{70.0, 100.0, 2}]` | MC-L2.1; 09C §6.1; Rule 17 |
| L1·C7(c) | `plan` | an item with `outcome: "reprint"`, `labelStatus: "printed"`, `labelPrintedAt` set; an item with `template: null`, `title: null`, a note `{code: "url_not_published", message: "M"}` | `{outcome Reprint, status Printed, printed_at Some}`; `{template None, title None, notes ["M"]}` | Rule 17 (nullable `template`, `title`) |
| L1·C7(d) | `plan` | 422 with code `no_products` | `Err(NoProducts)` | 09C §10 |
| L1·C7(e) | `plan` | an item with `outcome: "skip"` | `Err(BadReply(_))` | Adapter paths |
| L1·C7(f) | `plan` | 403 `insufficient_scope`; 404 `not_found`; 500 | `Err(NotAllowed)`; `Err(StoreGone)`; `Err(Unreachable(_))` | Adapter paths |
| L1·C8(a) | `preview(S, "1")` | `fetch` answers 200, `application/pdf`, PDF bytes | those bytes; one POST `…/stores/S/labels/preview` through `fetch` with body `{"productId": "1"}`, bearer `k1`, timeout `FILE_TIMEOUT` | MC-L2.5; Adapter paths |
| L1·C8(b) | `preview` | 200 whose body is `{"error": …}` JSON bytes; 200 with empty bytes | `Err(BadReply(_))` for both | MC-L2.12 (`%PDF-`) |
| L1·C8(c) | `preview` | 404 `sku_not_found` with message `M1`; 409 `sku_ambiguous` with `M2`; 422 `no_template` with `M3` | `Err(Rejected {code, message})` with that code and `M1` / `M2` / `M3` | 09C §10 ("the message as given") |
| L1·C8(d) | `preview` | 503 `renderer_unavailable` | `Err(RendererUnavailable)` | 09C §10 |
| L1·C8(e) | `preview` | 403 `insufficient_scope`; `Error::Timeout` | `Err(NotAllowed)`; `Err(Unreachable(_))` | Adapter paths |
| L1·C9(a) | `run(S, ["1", "2"], "sm-0123456789")` | `fetch` answers 201 with `labels_run.json` | one POST `…/stores/S/labels/runs` through `fetch` whose body is exactly `{"items": [{"productId": "1"}, {"productId": "2"}], "idempotencyKey": "sm-0123456789"}`, timeout `RUN_TIMEOUT` | MC-L2.7; M-L3 |
| L1·C9(b) | same | same | `LabelRun {id, status Partial, created_at, counts {2, 1, 1, 1, 0, 0}, artifacts [{id, 70.0, 100.0, page_count 1, filename, size_bytes 18211, download_url "/integration/v1/stores/<S>/labels/runs/r1/artifacts/a1", expired false}], items [{rendered true, result Some(MarkedPrinted), error None}, {rendered false, result None, error Some("Barcode: The product's variant has no barcode in Shopify.")}], unresolved []}` | MC-L2.9; Rule 17 |
| L1·C9(c) | `run` | status `rendered` with an item `lifecycleResult: "reprinted"`; status `failed` with `artifacts: []`; an item `lifecycleResult: "no_price"`; the answer without `unresolved`; an item `outcome: "failed"`, `lifecycleResult: "none"` | `Rendered` / `Some(Reprinted)`; `Failed` / no artifact; `Some(NoPrice)`; `unresolved []`; that item `result None` (the run parses, no `BadReply`) | Rule 17 (`unresolved` optional; `lifecycleResult` read tolerantly, `API.md` run example, projection r0 D4) |
| L1·C9(d) | `run` | 422 `no_products` | `Err(NoProducts)` | 09C §10; MC-L2.7 |
| L1·C9(e) | `run` | 503 `renderer_unavailable`; 500; `Error::Timeout` | `Err(RendererUnavailable)`; `Err(Unreachable(_))`; `Err(Unreachable(_))` | MC-L2.7 (retryable) |
| L1·C9(f) | `run` | 403 `insufficient_scope`; 404; 422 `validation_error` with message `M` | `Err(NotAllowed)`; `Err(StoreGone)`; `Err(Rejected {.., message: M})` | MC-L2.7 (not retryable) |
| L1·C9(g) | `run` | 201 whose body is not a run | `Err(BadReply(_))` | Adapter paths |
| L1·C10(a) | `download(S, "/integration/v1/stores/S/labels/runs/r1/artifacts/a1")` | `fetch` answers 200 with PDF bytes | those bytes; one GET `https://pricing.example/integration/v1/stores/S/labels/runs/r1/artifacts/a1` through `fetch`, bearer `k1`, timeout `FILE_TIMEOUT` | MC-L2.11, .12 |
| L1·C10(b) | `download` with `"https://evil.example/integration/v1/stores/S/labels/runs/r1/artifacts/a1"`; `"//evil.example/x"`; `"/integration/v1/stores/OTHER/labels/runs/r1/artifacts/a1"`; `"/integration/v1/stores/S/labels/runs/../../cases"`; `"/integration/v1/stores/S/labels/runs/r1/artifacts/a1?x=1"`; `"/integration/v1/stores/S/cases"` | nothing queued | `Err(BadReply(_))` for each; **no request recorded** | MC-L2.12; CR2 (the key goes only to the configured server) |
| L1·C10(c) | `download` (a valid path) | 410 `artifact_expired`; 404 `not_found` | `Err(Expired)` for both | 09C §10 |
| L1·C10(d) | `download` | 200 with bytes not starting `%PDF-` | `Err(BadReply(_))` | MC-L2.12 |
| L1·C10(e) | `download` | 403; 500 | `Err(NotAllowed)`; `Err(Unreachable(_))` | Adapter paths |

### Delegated choices
Whether the download-address check is a free function or a method · how the run's error envelope is read from bytes.

### Notes
- Hazard: the store id in the address check is compared in the adapter's own percent-encoded form (`encode`, `adapter.rs:71`); the C1 store id is a UUID, so encoded and raw are equal there, but the comparison must use the encoded form.
- Hazard: `run` is the only writing call of this chapter. It has no caller outside the adapter tests until L3; do not wire it anywhere (M-L2).
- The run answer's `items[].title`, `variantId`, `templateRevisionId`, `templateSource`, `priceMinor`, `artifactId`, `pageIndex` are required by the schema and nullable; the fixture carries them all.
- Hazard (projection r0, D4): the Pricing package contradicts itself on `RunItemOut.lifecycleResult` (schema enum with `null` against `API.md`'s `"none"`). Task 1's tolerant parse is what keeps a successful run that marked products Printed from reading as "Pricing answered in a way this version does not understand"; L1·C9(c)'s last input pins it.

### Unit test commands
`cargo test -p psm-integrations --lib conversations::beyo_pricing` · `cargo clippy -p psm-integrations --all-targets -- -D warnings`

## Unit 3 — Access, the display copy, the status read, label events in the pull

**Unit id:** `L1·U3` · **Rows:** L1·C11–C14 · **Changes app files:** yes

### Goal
`LabelsService` (access, `session_labels`, `LabelsLink`), `LabelCopy`, `LabelStore` with migration 022, the `Label` event kind in the adapter and the pull, `AppEvent::LabelsChanged`, the wiring in `App::build`, and the fake's label state.

### Read first
Master plan §4 (Ports `LabelStore` / `LabelsLink`, Storage, Service, Event, Tests), **§5 ACC, MC-L3.1–.4, MC-L4** · `crates/core/src/services/conversations.rs` (`connect` :527-569, `test_connection` :480, `key_set` :488-508, `calls_allowed` :573, `pull` :785-868, `first_connection` :643-725, `complete_check` :890, `check_failed` :917) · `crates/core/src/domain/conversation.rs` (`ProviderEvent` :155, `EventKind` :165) · `crates/integrations/src/conversations/beyo_pricing/adapter.rs` (`event` :221-252) · `crates/integrations/src/conversations/mod.rs` (`build_service` :190, `with_test_conversations` :153) · `crates/integrations/src/lib.rs` (`App`, `AppHandles`, the conversations block around :1228) · `crates/core/src/events.rs` · `crates/infra/src/sqlite/conversations.rs` (how `ConversationStore` is implemented for `SqliteStore`; `item_sessions` :725) · `crates/infra/src/sqlite/migrations/019_conversations.sql` · `crates/infra/src/sqlite/store.rs` (`migrations()`, `item_link` :2711, `session_for_item` :2533) · `crates/testkit/tests/suite/p27_retire_shape_on_arrival.rs` (the `DROP COLUMN` list in `c4a_…`) · `crates/testkit/src/fake_pricing.rs` (`State`, `push_event` :194, `raw_event` :451, `fail_matching` :480, `redeliver` :528, `route` :569) · `crates/testkit/src/test_app.rs` (`with_pricing` :703, `PRICING_KEY` :58) · `crates/testkit/src/fake_commerce.rs` (`add_product` :281: its ids are `product-<n>`) · `crates/testkit/tests/suite/c1_conversations_sync.rs` (`connected` :27, how a test drives `tick`) · `crates/core/src/services/attributes.rs` (`LINK_INTEGRATION` :351).

### Files
New: `crates/core/src/{mechanisms/labels.rs, services/labels.rs}`, `crates/infra/src/sqlite/{labels.rs, migrations/022_labels_access.sql}`, `crates/integrations/src/labels.rs`, `crates/integrations/src/conversations/beyo_pricing/fixtures/events_labels.json`, `crates/testkit/tests/suite/{l1_labels_access.rs, l1_labels_markers.rs}`. Edited: `crates/core/src/{mechanisms/mod.rs, services/mod.rs, services/conversations.rs, domain/conversation.rs, events.rs}`, `crates/integrations/src/conversations/beyo_pricing/{adapter.rs, labels.rs}`, `crates/integrations/src/{lib.rs, conversations/mod.rs}`, `crates/infra/src/sqlite/{mod.rs, store.rs}`, `crates/testkit/src/{fake_pricing.rs, fake_commerce.rs, test_app.rs}`, `crates/testkit/tests/suite/{main.rs, p27_retire_shape_on_arrival.rs, c1_conversations_connection.rs}` (in the last, only `c1_c9a_test_connection_saves_the_matching_store`'s request expectation (`:302`) becomes `["GET /stores", "GET /stores/<id>/labels/templates"]`, the ACC.2 contract; projection r0, D3), `apps/desktop/src-tauri/src/lib.rs` (the `TauriEvents` arm, so the workspace compiles; its row is L1·C15(d)).

### Core mechanisms (the only functions with unit tests)
`mechanisms::labels::{chunks, LabelCopy}`.

### Ordered tasks
1. `mechanisms/labels.rs`: the constants of master plan §4 that L1 reads (`STATUS_CHUNK`, `STATUS_FRESH_MS`), `chunks(n, size)` (consecutive ranges covering `0..n`, the last one shorter; `n = 0` → none) and `LabelCopy` (MC-L3.2). Every `now: Instant` handed to `LabelCopy` by the service is `clock.mono()` (never `Instant::now()`: only `mono()` moves with the test clock, so L1·C13(c) can be written; projection r0, D9). Unit tests: `chunks(0, 200)`, `(200, 200)`, `(201, 200)`; an answer applied after a later event for the same product is ignored and for another product is stored; `Reset` keeps `has_price` and creates an entry with `has_price` unknown; `fresh` true within the age and false after it and without an entry.
2. Migration `022_labels_access.sql` with its `M::up` line, and its `DROP COLUMN` line in the rewind test (`ALTER TABLE conversation_sync DROP COLUMN labels_access`). `LabelStore` for `SqliteStore` (`labels_access`, `set_labels_access`, `session_products`; `linked_items` belongs to L2).
3. `EventKind::Label(LabelChange)`; the adapter's `event` maps `label.printed` and `label.reset` by MC-L4.1 (fixture `events_labels.json`: the two events exactly as `EVENTS_AND_WEBHOOKS.md` describes them, `caseId: null`, `productId` in the envelope and in `data`).
4. `ports::labels::LabelsLink`; `ConversationsService::set_labels_link`; `connect` calls `connected()` after the lane is left when the resulting state is `Ready` (ACC.2); `key_set` calls `key_changed()` first (ACC.3); `pull` collects label changes and hands them over by MC-L4.2–.3 on every way out of the function; `first_connection` is not changed. `Label` events never touch `memory.applied` or the marks (MC-L4.4).
5. `LabelsService` (L1 constructor of master plan §4 Service): `access()` (ACC.7), `session_labels` (MC-L3.3), `impl LabelsLink` (ACC.2, ACC.3, MC-L4.3), the access changes of ACC.4 on every label call it makes **except `connected()`'s own templates read**: `connected()` applies only ACC.2 (its own three-way assignment and exactly one `LabelsChanged`), so a `NotAllowed` there emits one event, not two (ACC.2 as amended; projection r0, D2; L1·C12(b)). `AppEvent::LabelsChanged` and its `TauriEvents` arm → `labels_changed`.
6. Wiring: `crates/integrations/src/labels.rs::build_service`; `conversations::build_service` returns the adapter `Arc` as well (one instance, two ports); `App.labels` and `AppHandles.labels`.
7. `FakePricing` (master plan §4 Tests, the **L1** list): label state, `GET /labels/status` (one entry per id in order; an unknown id → `label: null` with `product_not_found`), `GET /labels/templates`, `deny_labels()`, `label_printed`, `label_reset`. `FakeCommerce::add_product_with_id(external_id, sku, title, barcode)` so a test's shop product can carry `gid://shopify/Product/<digits>` (first caller L1·C13(a)).
8. Guide: `02_architecture.md` (a §Labels: the service, the copy, the access value, the pull branch, the event), `03_rules_and_principles.md` (M-L1…M-L6 with where each bites, as far as L1 builds them), `04_change_impact.md` (rows: a Pricing label shape → `label_types.rs`, fixture, mapping, `FakePricing`; the `LabelsChanged` payload → the `TauriEvents` arm, `onLabelsChanged`).
9. Tests for the rows.

### Criteria (4 criteria, 28 rows)
Boundary for C11: the adapter's `events_after` over the recording transport. For C12–C14: `App.labels` and `App.conversations` on `TestApp::with_pricing()`; outcome = returned values, `FakePricing::request_lines()`, the events the test sink collected, the database. "Connected" = `key_set(Some(PRICING_KEY))` answered `Ready` and the first connection was run by `tick` (as `c1_conversations_sync.rs` `connected`). "A linked session for product P" = `add_product_with_id("gid://shopify/Product/P", …)` adopted with `ItemsService::adopt` and given a session, and the fake Pricing holding a label product `P`. "No label request" = no recorded request line contains `/labels/`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L1·C11(a) | `events_after` | 200 with `events_labels.json` | event 1 `kind Label(LabelChange {product_id: the fixture's, at: its "at" in ms, kind: Printed {price_minor: Some(890000)}})`, `conversation_id None`, `for_us false`; event 2 `kind Label(.. Reset {reason: Some(PriceChanged)})` | MC-L4.1; Rule 17 |
| L1·C11(b) | same | `label.reset` with `reason` `"sku_changed"`; `"barcode_changed"`; `"other"` | `Some(SkuChanged)`; `Some(BarcodeChanged)`; `None` | MC-L4.1 |
| L1·C11(c) | same | `label.printed` whose `data` has no `productId` and whose envelope has `productId: "7"`; one with neither | `Label(.. product_id "7" ..)`; `Other` | MC-L4.1 |
| L1·C11(d) | same | `label.printed` without `priceMinor` | `Printed {price_minor: None}` | MC-L4.1 |
| L1·C12(a) | `access()`, `session_labels([s], true)` | no key saved; `s` a linked session | `Off(NoKey)`; `[{session_id s, product_id Some(P), state None}]`; no label request | ACC.1; M-L6 |
| L1·C12(b) | `key_set(Some(PRICING_KEY))` | the fake has `deny_labels()` | the connection is `Ready`; exactly one label request, `GET /stores/<id>/labels/templates`; `access()` = `Denied`; one `LabelsChanged {[], access: true}` | ACC.2; 09C §4; §12.11 |
| L1·C12(c) | after (b): `session_labels([s], true)` twice | — | every state `None`; no further label request | ACC.1; M-L6; §12.11 |
| L1·C12(d) | `key_set(Some(PRICING_KEY))` | the fake allows labels | `access()` = `Allowed`; after `kill_and_restart`, `access()` = `Allowed` and the restart made no label request | ACC.2; Storage |
| L1·C12(e) | `key_set(Some(PRICING_KEY))` | `fail_matching("/labels/templates", 1, 500)` | the connection is `Ready` and its `last_error` is none; `access()` = `Unknown` | ACC.2, ACC.5 |
| L1·C12(f) | after (e): `session_labels([s], true)` | the fake answers status | one `GET …/labels/status?…`; `access()` = `Allowed` | ACC.4 |
| L1·C12(g) | connected and `Allowed`; then `deny_labels()` and `session_labels([s], true)` | — | `Err(Provider(NotAllowed))`; `access()` = `Denied` (also after `kill_and_restart`); `LabelsChanged {[], access: true}`; the connection's state is still `Ready` and the next `tick` still asks `/events` | ACC.4, ACC.5 |
| L1·C12(h) | connected and `Allowed`; the server setting is changed to another valid address | `session_labels([s], true)` | `access()` = `Off(StoreUnmatched)`; no label request | ACC.1; M-L6; MC-C1.3 |
| L1·C12(i) | two runs, each from an app connected and `Allowed`: (1) `key_set(None)`; (2) on a fresh app, with no `key_set(None)` before it, `key_set(Some("k-other"))` with `refuse_connection_next(1)` (the stores read fails) | — | (1): `access()` = `Off(NoKey)` and the stored access is `unknown` (`LabelStore::labels_access()`); (2): the stored access is `unknown`, whatever the connection says (each branch of `key_set` resets on its own; projection r0, D8) | ACC.3 |
| L1·C13(a) | `session_labels([s1, s2], true)` | connected; `s1` a linked session for `P1`, printed in the fake at T; `s2` a session whose item has no link | exactly one label request, `GET /stores/<id>/labels/status?productIds=P1`; `[{s1, Some(P1), Some({Printed, Some(T), Some(true), None})}, {s2, None, None}]` | MC-L3.3; 09C §7 |
| L1·C13(b) | after (a): `session_labels([s1], false)`; on a fresh app before any answer: the same | — | the state of (a) with no further label request; `state None` with no label request | MC-L3.3; 09C §7 (display copy) |
| L1·C13(c) | after (a), within 10 s of the manual clock: `session_labels([s1], true)`; then after the clock moved 11 s: again | — | no label request; then one status request | MC-L3.3 (`STATUS_FRESH_MS`) |
| L1·C13(d) | `session_labels([s], true)` | `s` linked to a product the fake's Pricing does not hold | `state None` for it | MC-L3.1 (`label: null`) |
| L1·C13(e) | after (a): the fake fails the next status with 500; the clock moved 11 s; `session_labels([s1], true)` | — | `Err(Provider(Unreachable(_)))`; then `session_labels([s1], false)` still answers the state of (a) | MC-L3.3 |
| L1·C13(f) | after (a) and L1·C14(a) | the database file | no table whose name contains `label`; the columns whose name contains `label` are exactly `attribute_definitions.label` (pre-existing, `002_domain.sql`: an attribute field's display name) and `conversation_sync.labels_access`, whose value is one of `unknown`, `allowed`, `denied` (projection r0, D1) | M-L1; MC-L3.4 |
| L1·C14(a) | `tick` | connected; `fake.label_printed(P1)` at time T (P1 was not printed) | `session_labels([s1], false)` = `{Printed, Some(T), Some(true), None}`; `S.cursor` = that event's id; exactly one `LabelsChanged {[P1], access: false}` | MC-L4.2, .3; §12.8 |
| L1·C14(b) | after (a): `fake.label_reset(P1, "price_changed")`, `tick` | — | `{NotPrinted, None, Some(true), Some(PriceChanged)}`; one more `LabelsChanged {[P1], false}` | MC-L4.3; MC-L3.2; §12.9 |
| L1·C14(c) | as (a) | — | no `ConversationsChanged` was emitted by that check (no notice, `chime` and `banner` never true) | MC-L4.4; 09C §8 (owner L7) |
| L1·C14(d) | after (a): `fake.redeliver(<that event>)`, `tick` | — | no second `LabelsChanged`; the state is unchanged | MC-L4.2 (dedupe by id) |
| L1·C14(e) | `tick` | one page holding `label.printed` for `P1`, `label.printed` for `P2`, `label.reset` for `P1` | exactly one `LabelsChanged {[P1, P2], false}`; `P1` is `NotPrinted` with the reason, `P2` is `Printed` | MC-L4.3 |
| L1·C14(f) | `tick` | one page holding `label.printed` for `P1` and then a question's `message.created` whose conversation read fails with 500 | `LabelsChanged {[P1], false}` was emitted; `P1` is `Printed`; `S.cursor` = the label event's id; the check is recorded as failed as today | MC-L4.3 (every way out) |
| L1·C14(g) | the first connection (`key_set` on a fake whose log already holds two label events), then `tick` until idle | — | no `LabelsChanged` with a product; `session_labels([s1], false)` = `None`; `S.cursor` = the last event's id | MC-L4.2; 09C §8 |
| L1·C14(h) | `tick` | `label.reset(P9, "sku_changed")` for a product no session here has | the copy holds `P9` as `{NotPrinted, None, None, Some(SkuChanged)}` (seen through a session linked to `P9` afterwards with `refresh = false`) | MC-L3.2 (entry created) |
| L1·C14(i) | `tick` | a `label.printed` event whose product id is missing everywhere | the cursor moves past it; no `LabelsChanged` | MC-L4.1, .2 |

### Delegated choices
How the service holds the copy and the remembered access (a mutex each is enough) · whether `connected()` reads the stored access before writing · names of the service's private helpers.

**Decided at projection r0** (`handoffs/reviewer/handoff_L1_labels_provider_and_marker_projection_r0.md`; the projectionist's recommendation is the rule): (D10) a `None` status answer keeps an answered-at time in the copy: the product is held with no state and counts as fresh for `STATUS_FRESH_MS` (`fresh` true, `get` none), so a product Pricing does not know is asked at most once per 10 seconds like any other (planning's point "never more than once per 10 seconds for the same products"; MC-L3.2's "removes the entry" reads as "holds no state"); (D11) `FakePricing::label_printed` and `label_reset` return the logged event's id, as `raw_event` does (`fake_pricing.rs:451`); the test reads the event's time from the fake (or the method returns it together with the id), which is how L1·C13(a) / C14(a) learn `S.cursor` and `T`.

### Notes
- Hazard: `connect` holds the lane while it reads the stores (`conversations.rs:536-560`). `connected()` makes a request; call it **after** the lane guard is dropped, or the pull and every reply wait behind a templates read.
- Hazard: `pull` returns early through `?` at several places (`conversations.rs:786-866`). The label changes collected so far must reach `LabelsLink::changed` on each of them (their cursor has already moved): collect into a local list and hand it over in one place that every return passes.
- Hazard: `key_set` also runs for a key that is the same as before; ACC.3 still resets (cheap, and correct for a key whose scopes changed).
- Hazard: the `LabelsLink` is a `Weak`; the conversations service must not keep the labels service alive, nor the reverse beyond `Arc<ConversationsService>` (no cycle).
- Hazard: every `match` on `EventKind` in the workspace gains the `Label` arm; the first connection's loop and the notices ignore it.
- The fake's status answer for a printed product: `labelPrintedAt` = the fake clock's time of the print; `hasPrice` = whether the fake product has a price.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::labels` · `cargo test -p psm-integrations --lib conversations::beyo_pricing` · `cargo test -p psm-testkit --test suite l1_labels_access::` · `cargo test -p psm-testkit --test suite l1_labels_markers::` · `cargo test -p psm-testkit --test suite c1_conversations_sync::` · `cargo test -p psm-testkit --test suite c1_conversations_connection::` · `cargo test -p psm-testkit --test suite p27_retire_shape_on_arrival::` · `cargo check --workspace` · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 4 — Commands, the Settings line, the marker, the ⋯ entry (disabled states)

**Unit id:** `L1·U4` · **Rows:** L1·C15–C19 · **Changes app files:** yes

### Goal
`labels_status`, the `labels` field of the Pricing settings answer, the window event, `markerText`, the pill on session rows and on the session page, the Labels line in Settings → Pricing, the ⋯ entry where it is disabled.

### Read first
Master plan §4 (Commands, Event, Words, Frontend), **§5 MC-L3.5–.7**, 09C §4, §6 "⋯ menu entry", §7 · `apps/desktop/src-tauri/src/commands/conversations.rs` (`settings_dto` :73, `connect_error` :91, `blocking` :102, `conversations_test` :156) · `apps/desktop/src-tauri/src/commands/remote.rs` (a thin command file) · `apps/desktop/src-tauri/src/lib.rs` (`specta_builder()`, `impl EventSink for TauriEvents`, the `c1_tests` module :546 for the test harness) · `apps/desktop/src/services/{conversations.ts, events.ts, sessions.ts}` · `apps/desktop/src/features/sessions/{SessionsPage.tsx, SessionRow.tsx (:54-61 the pill line), DeleteSession.tsx (`SessionActionsMenu` :28-84)}` · `apps/desktop/src/features/shell/Header.tsx` (:247 the session page's ⋯; where the title is rendered) · `apps/desktop/src/features/settings/PricingSection.tsx` (:131-135) · `apps/desktop/src/features/conversations/connectionLine.ts` (09B's words for each connection state) · `apps/desktop/src/features/sessions/__tests__/h2_list_pages_and_pictures.test.tsx` (how the page is tested with mocked services) · `docs/intention/evidence/design_handoff.md` (pill tokens).

### Files
New: `apps/desktop/src-tauri/src/commands/labels.rs`, `apps/desktop/src/services/labels.ts`, `apps/desktop/src/features/labels/{labelWords.ts, LabelPill.tsx, useSessionLabels.ts, LabelMenuItem.tsx}`, `apps/desktop/src/features/labels/__tests__/{l1_marker_words.test.ts, l1_markers.test.tsx, l1_label_menu.test.tsx}`, `apps/desktop/src/features/settings/__tests__/l1_labels_line.test.tsx`. Edited: `apps/desktop/src-tauri/src/{lib.rs, commands/mod.rs, commands/conversations.rs}`, `apps/desktop/src/services/{bindings.ts (generated), events.ts, conversations.ts}`, `apps/desktop/src/features/sessions/{SessionsPage.tsx, SessionRow.tsx, DeleteSession.tsx}`, `apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/features/settings/PricingSection.tsx`, `apps/desktop/src/styles/globals.css`, the existing UI tests that render `SessionsPage`, `Header` or `PricingSection` (directly or through `App`), in `src/features/{sessions, shell, settings, workspace, publish, conversations, pending, compose}/__tests__/` (`compose`: its `harness.tsx` renders `App`): they gain `onLabelsChanged` in their `@/services/events` mock (a factory mock without it throws on access, so `useSessionLabels`'s listener would crash them), a `@/services/labels` mock answering access off, and `labels: 'off'` in typed `ConversationsSettings` fixtures (`settings()` in `c1_pricing_settings`, and the same pattern in `a28_test_feedback`, `t1_settings_drawer`: `typecheck` includes tests); **no assertion changes** (projection r0, D7), `docs/guide/{02_architecture.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
`labelWords.ts::markerText`.

### Ordered tasks
1. `commands/labels.rs`: `labels_status(session_ids, refresh)` (thin: one `LabelsService::session_labels` call on a blocking worker plus `access()`), its DTOs (master plan §4 Commands wire forms), the `LabelsFailure` → `CommandError` mapping of master plan §4 (the sentence from `provider.describe` through the service; `labels_off` carries `data.reason`). `settings_dto` gains `labels`. Registered in `specta_builder()`; `bindings.ts` regenerated; wrappers in `services/labels.ts`; `onLabelsChanged` in `services/events.ts`.
2. `labelWords.ts`: `markerText(marker, now)` by MC-L3.5 (the date from the local calendar day of `printed_at`, `d MMM`), the constants of master plan §4 Words that L1 shows.
3. `useSessionLabels(sessionIds)`: MC-L3.7 (the call after each new id list, `refresh` by the id-list rule, one `labels_changed` listener, the answer kept by session id together with `access`); never blocks rendering and never throws into the page (a failed call leaves the pills as they were).
4. `LabelPill` in `SessionRow`'s pill line (the pill line is rendered when there is a marker, too) and beside the title in `Header` for a session page.
5. `PricingSection`: the Labels line under the connection line (MC words: `allowed`, `denied`, `unknown`; nothing for `off`).
6. `LabelMenuItem` in `SessionActionsMenu` (both callers pass the session's label and the access): in L1 it renders **only** when it is disabled. **First match in this order** (projection r0, D6): no `labelsStatus` answer yet (label undefined) → not rendered; `product_id` null → `Generate label`, disabled, hint `Publish first so Pricing can see it` (in every access state); access `off` → disabled, hint = 09B's words for that state; access `denied` → disabled, hint `This key can't read labels. Ask a Pricing admin to add labels:read and labels:write.`; access `unknown` → not rendered; otherwise (it would be enabled) → not rendered until L3. **The `off` hint's source** (projection r0, D5): `reason = "no_key"` → `CONNECT_PRICING` (`Connect Pricing in Settings → Pricing`); any other `off` reason → when the menu opens, the existing `conversationsSettingsGet()` is called and the hint is `connectionLine(settings.connection)`, with `CONNECT_PRICING` while it loads, when it fails or when `connectionLine` answers null (the menu already fetches on open, `sessionFilesMisplaced`, `DeleteSession.tsx:41`; `connectionLine` needs `shop`, `store_name`, `visible_stores`, which the access value does not carry).
7. Guide rows for the command, the DTO and the event. Tests for the rows.

### Criteria (5 criteria, 23 rows)
Boundary for C15: the command bodies and the event bridge in the desktop crate's `l1_tests` on `TestApp::with_pricing()`. For C16–C19: components rendered in UI tests with `@/services/labels`, `@/services/events` and the other services mocked (outcome = what is rendered and the exact list of `labelsStatus` calls).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L1·C15(a) | `labels_status([s1, s2], true)` | the app of L1·C13(a) | `{access: {state: "allowed", reason: null}, labels: [{session_id: s1, product_id: "P1", marker: {status: "printed", printed_at: T, has_price: true, reset: null}}, {session_id: s2, product_id: null, marker: null}]}` | MC-L3.3; Commands |
| L1·C15(b) | `labels_status([s1], true)` | no key saved | `{access: {state: "off", reason: "no_key"}, labels: [{s1, "P1", null}]}` | ACC.7 |
| L1·C15(c) | `labels_status([s1], true)` | connected; the fake answers 500 on status | `CommandError {code: "labels_unreachable", message: "Pricing can't be reached right now."}` | 09C §10; Commands |
| L1·C15(d) | the event bridge | `AppEvent::LabelsChanged {product_ids: ["P1"], access: false}` | the window event `labels_changed` with payload `{product_ids: ["P1"], access: false}` | Event |
| L1·C15(e) | `conversations_test` | a key with labels; a key under `deny_labels()`; no key | the answer's `labels` is `"allowed"`; `"denied"`; `"off"` | 09C §4; Commands |
| L1·C16(a) | `PricingSection` | settings with `labels: "allowed"` | the line `Labels: allowed` | 09C §4 |
| L1·C16(b) | same | `labels: "denied"` | `Labels: this key can't read labels. Ask a Pricing admin to add labels:read and labels:write.` | 09C §4 |
| L1·C16(c) | same | `labels: "unknown"` | `Labels: couldn't check. Press Test connection.` | ACC.2 |
| L1·C16(d) | same | `labels: "off"` | no line starting with `Labels:` | 09C §4 |
| L1·C17(a) | `markerText` | `{printed, printed_at: 4 Nov of this year}`; of last year; `printed_at: null` | `Label printed · 4 Nov`; `Label printed · 4 Nov <last year>`; `Label printed` | MC-L3.5; 09C §7 |
| L1·C17(b) | `markerText` | `not_printed` with `reset` `price_changed`; `sku_changed`; `barcode_changed` (each with `has_price: true`) | `Price changed: new label needed`; `SKU changed: new label needed`; `Barcode changed: new label needed` | MC-L3.5; 09C §7 |
| L1·C17(c) | `markerText` | `not_printed`, no reset, `has_price` true; false; null | `Missing price tag`; `No price yet`; no marker (`null`) | MC-L3.5; 09C §7 |
| L1·C17(d) | `markerText` | `not_printed`, `reset: price_changed`, `has_price: false`; `marker: null` | `Price changed: new label needed`; no marker | MC-L3.5 (order) |
| L1·C18(a) | `SessionsPage` (Open) | `sessionsPage` answers rows 1–3; `labelsStatus` stays pending | the three rows are on screen; exactly one call `labelsStatus({session_ids: [1, 2, 3], refresh: true})` | MC-L3.7 ("never delaying it"); 09C §7 |
| L1·C18(b) | same | `labelsStatus` then answers row 1 printed (4 Nov), row 2 `not_printed` with a price, row 3 `marker: null` | row 1's pill line shows `Label printed · 4 Nov`, row 2's `Missing price tag`, row 3 has no label pill | MC-L3.6; 09C §7 |
| L1·C18(c) | same | a `sessions_changed` event (the page answers the same rows); then page 2 is opened (rows 4–5) | `labelsStatus({session_ids: [1, 2, 3], refresh: false})`; then `labelsStatus({session_ids: [4, 5], refresh: true})` | MC-L3.7 |
| L1·C18(d) | same | `labels_changed {product_ids: ["P1"], access: false}`; then `labels_changed {[], access: true}` | one call with `refresh: false`; then one with `refresh: true`; the pills follow each answer | MC-L3.7; MC-L4.5 |
| L1·C18(e) | same | `labelsStatus` rejects | the rows stay; no toast; no label pill | MC-L3.7; CR5 |
| L1·C18(f) | `Header` on a session page | `labelsStatus` answers that session printed on 4 Nov | one call `labelsStatus({session_ids: [id], refresh: true})`; the pill `Label printed · 4 Nov` beside the title | MC-L3.6, .7; 09C §7 |
| L1·C19(a) | `SessionActionsMenu` of a row, opened | the row's label has `product_id: null`, access `allowed` | an item `Generate label`, disabled, with the hint `Publish first so Pricing can see it` | 09C §6; §12.12 |
| L1·C19(b) | same | `product_id: "P1"`, access `{off, no_key}`; access `denied`; access `{off, store_gone}` with `conversationsSettingsGet` answering a `store_gone` connection for shop `shop-a.myshopify.com` | disabled with the hint `Connect Pricing in Settings → Pricing`; disabled with `This key can't read labels. Ask a Pricing admin to add labels:read and labels:write.`; disabled with `This key is no longer tied to shop-a.myshopify.com` | 09C §4; 09B §4 (`connectionLine`, projection r0 D5) |
| L1·C19(c) | same | `product_id: "P1"`, access `allowed`; access `unknown` | no label item in the menu (it arrives enabled in L3); "Delete session…" is still there | 09C §13 (L1: disabled states only) |
| L1·C19(d) | the session page's ⋯ (`Header`) | that session's label has `product_id: null`, access `allowed` | the same disabled item and hint as (a) | 09C §6 |

### Delegated choices
The pill's look (design handoff tokens; neutral for printed, the warning token for missing, stale and no price) · where exactly beside the title the pill sits · how the hint is shown (a `title` or a second line) · whether `useSessionLabels` is one hook shared by the page and the header or two.

**Decided at projection r0** (`handoffs/reviewer/handoff_L1_labels_provider_and_marker_projection_r0.md`; the projectionist's recommendation is the rule): (D13) `markerText`'s month is a fixed English abbreviation (`Jan` … `Dec`) after the local calendar day (`4 Nov`, `4 Nov 2025`), never `toLocaleDateString` (jsdom / en-US gives `Nov 4`).

### Notes
- Hazard: `SessionsPage` reloads on every `sessions_changed` (`SessionsPage.tsx:73-83`), which fires while photos arrive. The id-list rule (MC-L3.7) is what keeps that from asking Pricing; L1·C18(c) is its row.
- Hazard: the date in `Label printed · 4 Nov` is the local calendar day; build the test's times from local dates, not from UTC midnight.
- Hazard: `SessionRow` renders the pill line only when `row.arranged || research || conversations` (`SessionRow.tsx:54`); a row with only a label marker needs it too.
- Hazard: the existing UI tests of the Sessions page, the header and Settings → Pricing must keep passing with a labels mock that answers nothing; do not change their assertions. 47 test files mock `@/services/events` with a factory (e.g. `sessions/__tests__/h2_list_pages_and_pictures.test.tsx:20`, `shell/__tests__/h4_capture_target.test.tsx:43`, `conversations/__tests__/c2_header_icon.test.tsx:17`); those that render `SessionsPage` or a session `Header` need `onLabelsChanged` in it. The only edits allowed in them are the three of Files (projection r0, D7).
- The hints for the `off` states reuse `connectionLine` (09B's words) over `conversationsSettingsGet()` fetched when the menu opens; `no_key` has its own constant `CONNECT_PRICING` (task 6, projection r0 D5).

### Unit test commands
`cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/labels src/features/sessions src/features/shell src/features/settings src/features/workspace src/features/publish src/features/conversations src/features/pending src/features/compose` (every folder whose tests render `SessionsPage`, `Header` or `PricingSection`; projection r0, D7) · `npm run typecheck` · `cargo clippy -p psm-desktop --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core --lib mechanisms::labels` · `cargo test -p psm-integrations --lib conversations::beyo_pricing` · `cargo test -p psm-testkit --test suite l1_labels_access::` · `cargo test -p psm-testkit --test suite l1_labels_markers::` · `cargo test -p psm-testkit --test suite c1_conversations_connection::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/labels src/features/sessions src/features/shell src/features/settings src/features/workspace src/features/publish src/features/conversations src/features/pending src/features/compose` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: no label call goes through `read_error` / `write_error` or returns `ProviderError`; `run` has no caller outside the adapter's tests; nothing in `crates/core` names Pricing, Shopify or a `gid`; `connected()` is called outside the lane; the only schema change is `conversation_sync.labels_access`; no label word, key or SKU list is logged; the fixtures carry neutral values and a `_source`; the existing conversations tests changed only where `EventKind` gained an arm, and in `c1_conversations_connection.rs::c1_c9a_test_connection_saves_the_matching_store`, whose request list gains the templates read and nothing else (projection r0, D3); the existing UI tests changed only as U4's Files allow (D7); `connected()` makes no ACC.4 change of its own (D2); the label copy is fed `clock.mono()` (D9). Notes go to `docs/implementation/backlog_labels/L1_labels_provider_and_marker.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
Needs a Pricing key with `labels:read` (Settings → Pricing → Replace).
1. Settings → Pricing → Test connection: under the connection line it says "Labels: allowed". With the old key (no label permission) it says "Labels: this key can't read labels…", and conversations keep working.
2. Open Sessions. Rows of published products show a small label marker: "Label printed · <date>", "Missing price tag" or "No price yet". The list appears first; the markers a moment later.
3. In Pricing, print a label for a product that shows "Missing price tag". Within about ten seconds its marker here reads "Label printed · today", with no sound.
4. In Shopify, change the price of a product whose label is printed. Within about ten seconds its marker reads "Price changed: new label needed".
5. Open a published session: the same marker stands beside its title.
6. Open the ⋯ menu of a session that was never published: "Generate label" is greyed out with "Publish first so Pricing can see it".

## Review log

- **2026-11-23, orchestrator: projection r0 amendments applied** (`handoffs/reviewer/handoff_L1_labels_provider_and_marker_projection_r0.md`, AMENDMENTS_REQUIRED, 0 owner cards; master plan v1.1). Where each ledger row landed:
  - **D1** (C13(f) could never pass): row L1·C13(f) outcome names both `label` columns, `attribute_definitions.label` (pre-existing) and `conversation_sync.labels_access`.
  - **D2** (`connected()` and ACC.4): master plan ACC.2 gains "`connected()` applies only ACC.2; ACC.4 governs every other label call"; U3 task 5 says the same.
  - **D3** (C1·C9(a) breaks): U3 Files add `c1_conversations_connection.rs` (only `c1_c9a`'s request list gains `GET /stores/<id>/labels/templates`); U3 Unit test commands and Review mode add `c1_conversations_connection::`; Review mode names the allowed change.
  - **D4** (`lifecycleResult: "none"`): U2 task 1 (tolerant parse; enum fields the domain never reads not strictly typed; strict `labelStatus`, `outcome`, run `status`), U1 task 4 (the same rule for `LabelStatusOut.source`), row L1·C9(c) gains the input `outcome "failed"`, `lifecycleResult "none"` → `result None`, U2 Notes hazard.
  - **D5** (the `off` hint's data): U4 task 6 (non-`no_key` `off` → `conversationsSettingsGet()` on menu open, `connectionLine(settings.connection)`, `CONNECT_PRICING` while loading / on failure / on null); row L1·C19(b) gains the input `{off, store_gone}` → `This key is no longer tied to shop-a.myshopify.com`; U4 Notes.
  - **D6** (precedence of the ⋯ entry): U4 task 6 is a first-match list starting with "no answer yet → not rendered" and "`product_id` null → Publish first, in every access state".
  - **D7** (existing UI tests perimeter): first option taken (the listener stays in `events.ts`, master plan §4 Event unchanged; the alternative needed a §4 amendment). U4 Files name the three allowed edits (`onLabelsChanged` in the events mock, a labels mock answering off, `labels: 'off'` in typed settings fixtures) and the folders; U4 Unit test commands and Review mode run vitest over `labels, sessions, shell, settings, workspace, publish, conversations, pending` and also `compose` (found while applying: `compose/__tests__/harness.tsx` renders `App`); U4 Notes hazard (47 files mock `@/services/events`).
  - **D8** (C12(i) not discriminating): row L1·C12(i) runs two independent cases from a connected, `Allowed` app and asserts the stored `unknown` after `key_set(None)` too.
  - **D9** (which clock): U3 task 1: `now` = `clock.mono()`.
  - **D10** (freshness after a `None` answer, delegated): U3 "Decided at projection r0": answered-at time kept, no state, fresh for `STATUS_FRESH_MS`.
  - **D11** (fake return values, delegated): U3 "Decided at projection r0": `label_printed` / `label_reset` return the event id; the time read from the fake or returned with it.
  - **D12** (trace notation): master plan §3 trace grammar admits a §4 registry entry; no row retraced.
  - **D13** (month words, delegated): U4 "Decided at projection r0": fixed English month abbreviations after the local calendar day.
  Counts recounted by script from the three plan files: L1 U1 6 / 35, U2 4 / 23, U3 4 / 28, U4 5 / 23 = 19 / 109 (unchanged); totals 11 units, 50 criteria, 294 rows (master plan §2 unchanged). Outcomes otherwise unchanged, so no new projection round (`projection: done`).
- **2026-11-23, implementer (Claude Opus 5.5, `pipeline-implementer`): L1 implemented, round 1** (handoff `handoffs/implementer/handoff_L1_labels_provider_and_marker_implement_r1.md`). Checkpoints: U1 `28395bc`, U2 `793950f`, U3 `499b734`, U4 `8870b3f`.
  - **Built.** U1: `domain/label.rs`, `ports/labels.rs`, `PricingHttp::fetch` + `RawReply` (`ReqwestPricing`, `NoTransport`, the adapter tests' `Recording`, `FakePricing` → 404), `label_types.rs`, `labels.rs` (`products`, `status`, `templates`, `describe`, `product_id_of`, `commerce_id_of`, one error function `label_error`, never `ProviderError`). U2: `plan` (through `send`), `preview` / `run` / `download` (through `fetch`; `%PDF-` check; the download path check), `RUN_TIMEOUT` / `FILE_TIMEOUT`; `run` has no caller outside the adapter's tests. U3: `mechanisms/labels.rs` (`chunks`, `LabelCopy`, `STATUS_CHUNK`, `STATUS_FRESH_MS`), migration `022_labels_access.sql` + `LabelStore` for `SqliteStore`, `EventKind::Label` and the adapter's `label.printed` / `label.reset` mapping, `LabelsLink` (`connect` → `connected()` after the lane, `key_set` → `key_changed()` first, `pull` hands the page's changes over on every way out), `LabelsService` (`access`, `session_labels`, ACC.4 in `learn`), `AppEvent::LabelsChanged` + `TauriEvents` arm, `integrations::labels::build_service`, `App.labels` / `AppHandles.labels`, `FakePricing` label state and routes, `FakeCommerce::add_product_with_id`, guide §Labels / M-L rules / change-impact rows. U4: `labels_status`, `settings_dto.labels`, bindings, `services/labels.ts`, `onLabelsChanged`, `labelWords.ts` (`markerText`), `useSessionLabels`, `LabelPill` (rows and session header), `LabelMenuItem` (disabled states only), the Labels line in Settings → Pricing, pill styles, guide rows.
  - **Assumptions** (each the smallest option consistent with 09C / ACC / MC-L3 / MC-L4): a 409, 422 or other status without an error envelope is `BadReply` (`Rejected` needs a code and a message); the download path check also refuses a backslash and a percent-encoded dot segment (URL parsers read them as `/` and `..`), which only refuses more than MC-L2.12; `RunItemOut.outcome` is read tolerantly (`rendered` iff the word is `rendered`), like `lifecycleResult`; `LabelCopy::clear` also ignores answers to requests started before it (a Test connection may switch stores mid-read); `LabelsFailure` lives in `services/labels.rs`; `key_changed()` and a learned `allowed` emit no event (ACC.3 / ACC.4 name none; L1·C12(b) needs exactly one); `useSessionLabels` guards its listener and its call, so the existing UI tests needed no events or labels mock (only the two typed `ConversationsSettings` fixtures gained `labels: 'off'`); the ⋯ hint is a second line under "Generate label"; printed pills are neutral, the others use the `--warn` token; one hook for the page and the header; `FakePricing` logs label events in its first store and `event_at_ms(id)` gives an event's time (D11).
  - **Closing gate.** `cargo test --workspace` found one red test outside the perimeter: `p01_storage_identity::c1a_open_creates_all_domain_columns` lists every column of every table, so `conversation_sync` gained `labels_access` there (the second tripwire of an `ADD COLUMN` migration; guide `04_change_impact.md` row updated to name it). Re-run of that file and of the doc-tests green; clippy, fmt, typecheck, `npm test` green.
  - **Deviations.** `domain/label.rs` changed in U2's checkpoint (`PlanItem.product_id` and `RunItem.product_id` are `String`: the schema makes them required strings). `test_app.rs` and `services/conversations.ts` (listed as edited) needed no change. The closing `cargo test --workspace` outlived the 10-minute foreground limit and finished in the background; no other cargo command ran meanwhile.
