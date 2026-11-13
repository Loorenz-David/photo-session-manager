---
phase: R8_research_real_adapters
title: Research with real providers — the app can ask OpenAI to look at photos and read listings, send the cover photo to the visual search the owner picked, read listing pages by the route the probes showed to work, and knows the shop's own web addresses so an item is never evidence for itself
difficulty: high
projection: yes
depends_on: [R4_research_engine, R1_research_benchmark]
blocked_on: R1's recorded evidence (U1); the owner's pick of the visual-search route (U2) and of the page-reading route (U3), recorded in the evidence doc under "Owner picks"
---
# R8 — Real adapters: the model, the visual search the owner picked, page reading, the shop's own addresses

**Tracker:** NOT_STARTED · 2026-11-12 · planner · —

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
The three real adapters, in `crates/research/src/adapters/`, each written only from R1's captures (charter rule 17), registered in `adapters::build`; and the two Shopify facts research needs (public hosts and the item's own listing; unit names in the imported definitions). **Not here:** default model names (R10), the other search route, the other page-reading route.

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 3 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R8_research_real_adapters.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | OpenAI model adapter | 6 | 14 | no |
| U2 | Visual-search adapter (the picked route) | 6 | 7 | no |
| U3 | Page reader (the picked route); shop hosts, own listing and unit names from the shop | 6 | 8 | yes |
| **Total** | | **18** | **29** | |

**Gate for the whole milestone.** If the evidence doc has no section "Owner picks" naming the visual-search route and the page-reading route, report `STATE: BLOCKED` (reason: `owner pick missing: …`) before U1 and change nothing. (U1 alone needs only R1's evidence; the orchestrator may launch this milestone with a note in the Review log `U1 ONLY: <reason>` to build U1 while the picks are open; then stop after U1's checkpoint and report `STATE: BLOCKED` with the reason `picks missing for U2, U3`.)

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`. Also the fixtures README (`crates/research/fixtures/README.md`) and R1's Review log.

## Carried from earlier reviews (orchestrator, 2026-11-12; wins over older wording below)
- **U1–U3, "no credit" vs "bad key" (R3 review).** `rules::calls::classify` checks 401/403 before the quota flag. Each adapter sets the quota flag from the provider's real quota reply (RESEARCH-VERIFY-2, -3, -11) only where status is not 401/403, so an out-of-credit account never reads "key rejected".
- **Knowing for failed runs (R4 review).** `store::next_attempt` counts the `unknown` call a kill left, so a unit re-issued after a kill gets two transient tries instead of three. No change required; do not "fix" retry counts in the adapters around it.
- **Secrets in the export scan (R7 review, MA16).** `export.rs` writes requests and responses as JSON string fields, so a
  secret containing `"`, `\` or a control character appears escaped and the plain / URL-encoded scan misses it. When the
  adapters add their real key formats, add the JSON-escaped form of each secret to the scan's forms (fail closed), with
  a row using a key that contains `"`.
- **`ResearchStatus.wait_ends_at` (R7 review).** No backend row checks it (`source_wait_seconds × 1000 − since_change`,
  0 for a manual run). Add one service-level row.
- **Long calls and settings saves (R7 review, knowing).** A settings save that rebuilds the runtime detaches the old
  research thread after 1 s; with real calls up to 180 s two runners briefly exist. `store::start_run`'s guard keeps one
  running run; no change required unless a rebuild must cancel in-flight research.
- **Picks recorded (orchestrator, 2026-11-13).** The evidence doc's "Owner picks" names Lens via SerpApi (U2) and plain
  HTTP for every site (U3). Cloud Vision is the named replacement and is not built here.
- **U3, Shopify storefronts throttle (R1 review).** The owner's shop, midcenturymobler and vgliving answer `429` with
  `Retry-After: 60` after a short burst, not lifted by one wait. The HTTP page reader keeps one request per host at a
  time with a per-host pace for Shopify-hosted sites (recognised by the shop facts or the `x-shopid` / `powered-by:
  Shopify` reply header), honours `Retry-After` up to the call timeout, and otherwise records "could not be read"
  (R4). Add a row with a fake host answering 429 then 200.

## Unit 1 — OpenAI model adapter

**Unit id:** `R8·U1` · **Replaces:** A15 (all rows) · **Changes app files:** no

### Goal
Implement `ModelProvider` for the OpenAI API behind an HTTP seam: per-stage instructions and strict output schemas
(version 1, starting from R1·U2's version 0 when it exists), image input at the stage's size, the mapping of the
reported usage to the price-table units, the mapping of every failure to a `CallFailure`, refusals, and the
declared secret-bearing parts. **Every provider-owned shape in this phase comes from R1·U1's captures**; nothing is
written from memory or from documentation prose. **Not in this phase:** visual search (R8·U2), page reading (R8·U3),
default model names (R10·U1), the settings UI for the key (R6·U2).

### What changed from the superseded plan (this wins over any older wording below)
- The adapter, the HTTP seam and `FakeResearchHttp` are in the research crate (`crates/research/src/adapters/{http.rs, openai/}`, `psm_research::fakes::FakeResearchHttp`); the adapter is registered in `psm_research::adapters::build` (R4·U4), not in `crates/integrations/src/lib.rs`. `reqwest` becomes a dependency of `psm-research` at the workspace's pinned version and features.

### Fold before dispatch
R1·U1's evidence may rename or split the capture files. Before this phase is dispatched, check each fixture named in
the criteria against `crates/research/fixtures/README.md` and amend the cells. A row whose shape
R1·U1 recorded as *documented only* is marked **structurally held** with its trigger ("the first such reply recorded
by a call record") instead of being implemented against prose. If R1·U1's evidence contradicts MC-A8 or MC-A9, stop
and route the contradiction to the intention.

### Read first
`master_plan_research.md` §3 (rule 17, keys, HA5), §6.1, §6.6, §7.4 (RESEARCH-VERIFY-11) · book `14` (MA15, MA16),
`08A` §4, §5 (stage inputs and outputs), §9, `08B` MC-A2 (what the model is asked to return), MC-A4 (`sure` flag
only; never a confidence), MC-A8 (units), **MC-A9** (record, redaction layer 1, error classes), **§Grounding owed**
· evidence `research_providers.md` (R1·U1's sections for RESEARCH-VERIFY-1…5 and -10) ·
`crates/research/fixtures/` · code `crates/integrations/src/commerce/shopify/transport.rs` (the
pattern for a private-runtime HTTP seam), `crates/research/src/ports.rs`, `crates/research/src/domain.rs` (`CallFailure`),
`crates/research-tool/src/instructions/` (R1·U2's version 0, if R1·U2 has run).

### Files
New: `crates/research/src/adapters/{http.rs, openai/{mod.rs, adapter.rs, usage.rs, errors.rs, instructions/…, schemas/…}}`, `crates/research/tests/engine/adapter_openai.rs`. Edited: `crates/research/src/{adapters/mod.rs, fakes.rs}`, `crates/research/Cargo.toml`, `crates/research/tests/engine/main.rs`.

### Core mechanisms (the only functions with unit tests)
None (adapter mapping is proven at the port boundary with captured replies).

### Ordered tasks
1. `ResearchHttp` (trait: one request → status, headers, body bytes, or a transport failure with `sent`) and
   `ReqwestResearchHttp`: `is_connect()` → `sent: false`; any other transport failure → `sent: true`; error text
   built after `without_url()`; timeout from `call_timeout_seconds.model`.
2. Request building per stage from `ModelRequest`: instructions (`INSTRUCTIONS_VERSION = 1`, in the requested
   language; choice fields carry the definition's own choice strings), the strict schema, the images.
3. Reply handling from the captures: structured output → `ModelOutput::Json`; the refusal item →
   `ModelOutput::Refusal(text)`; an incomplete reply or one that fails the schema → `CallFailure` kind `Schema`;
   usage → `(input_token, cached_input_token, output_token, web_search_call)` counts exactly as the capture
   reports them.
4. Errors from the captures → `CallFailure {status, kind: Http, quota flag, usage if reported}`.
5. `Exchange` for the record: `authorization` (and any other header R1·U1 found to carry the key) as `<redacted>`.

### Criteria (6 criteria, 14 rows)
Boundary: `ModelProvider::call` on the adapter over `FakeResearchHttp`; outcome = the `ModelReply` / `CallFailure`
and the request the fake received. Fixture paths are under `crates/research/fixtures/openai/`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A15·C1(a) | usage | the ok reply with an image and a strict schema (RESEARCH-VERIFY-1) | usage units and counts equal the capture's own usage figures, one unit per reported field the price table names | MA15 |
| A15·C1(b) | usage | the reply of a call that used the web search tool (RESEARCH-VERIFY-1, -5) | `web_search_call` equals the number of tool calls the capture reports | MA15 |
| A15·C1(c) | usage | the incomplete reply (RESEARCH-VERIFY-1, -3) | kind `Schema`; `usage` present or absent exactly as the capture shows (absent → the gateway charges the fallback) | MA15 |
| A15·C2(a) | errors | the invalid-key reply (RESEARCH-VERIFY-2) | `CallFailure` with the capture's status; `classify` gives `key` | MC-A9 |
| A15·C2(b) | errors | the rate-limit reply and the no-quota reply (RESEARCH-VERIFY-2; structurally held if R1·U1 recorded them as documented only) | `transient` for the first; quota flag set and `quota` for the second | MC-A9 |
| A15·C2(c) | errors | the oversized-input reply (RESEARCH-VERIFY-2) | `request` | MC-A9 |
| A15·C3(a) | refusal | the refusal reply (RESEARCH-VERIFY-3; structurally held if not captured) | `ModelOutput::Refusal` with the capture's text | MC-A9 |
| A15·C3(b) | schema | the ok reply with one required field removed from its output | kind `Schema` | MC-A9 |
| A15·C4(a) | request | an observe request with one image at 1024 and choices `[Oak, Teak]`, language `Swedish` | the body the fake received carries the image bytes of that `ResearchImage`, both choice strings and the language; its top-level shape equals the request R1·U1 recorded for an image + schema call (same keys) | MC-A2 |
| A15·C4(b) | request | each of the stages observe, read, compare, labels, condition, decide | each is sent with its own instructions and schema, and the model and effort given in the request | MC-A7 (unit identity: model, effort, instructions version) |
| A15·C5(a) | secrets | any call | the `Exchange` shows `authorization` as `<redacted>`; the key value appears nowhere in it | MA16 |
| A15·C5(b) | transport | `ReqwestResearchHttp` against a closed port on `127.0.0.1` | `CallFailure {sent: false}`; its text contains no URL | MC-A9 (RESEARCH-VERIFY-11) |
| A15·C5(c) | transport | `ReqwestResearchHttp` against a local listener that accepts and never answers, timeout 200 ms | `CallFailure {sent: true}`, kind `Timeout` | MC-A9 |
| A15·C6(a) | end to end | R4·U3's full-run script with this adapter (over `FakeResearchHttp` replaying captures for the model calls) and the fake search and page reader | the run finishes and writes its values; every call row's cost is non-zero and equals `call_cost` of the recorded usage | MA15 |

### Delegated choices
The wording of the instructions (kept short; each stage asks only for its output of 08A §5) · schema field names ·
how the adapter id is spelled in `research.providers.model`. Not delegated: a model never returns a confidence
(only `sure` on photo reads), and no shape is taken from memory.

### Notes
- C5(b) and C5(c) use loopback sockets on port 0, as `p43_ftp_receiver` does; they are not external services.
- C6(a) needs one captured reply per stage; R1·U2 writes them. If R1·U2 has not run, the row uses the image + schema
  capture for every stage and says so under Assumptions.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine adapter_openai::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 2 — Visual-search adapter (the picked route)

**Unit id:** `R8·U2` · **Replaces:** A16 (all rows) · **Changes app files:** no

### Goal
Implement `VisualSearchProvider` for **one** route: Google Lens through SerpApi, or Google Cloud Vision Web
Detection. The port stays as it is, so the other can be added later. **Not in this phase:** the other route, page
reading (R8·U3), ranking or site rules (core, R3·U2).

### What changed from the superseded plan (this wins over any older wording below)
- The adapter is `crates/research/src/adapters/visual_search/<route>.rs`, registered in `adapters::build`.

### Task 0
Read `docs/intention/evidence/research_providers.md` §Owner picks. If no visual-search route is recorded there,
report `STATE: BLOCKED` (reason: `owner pick missing: visual-search route`) and change nothing. Otherwise build the
variant named there and delete the other variant's section from this plan's criteria in the Review log entry
("variant built: …").

### Fold before dispatch
After the pick: strike the other variant below, and check the fixture names against the fixtures README.

### Read first
`master_plan_research.md` §3, §6.1, §6.6, §7.4 · book `14` (MA12, MA15, MA16), `08A` §4, `08B` MC-A3 (candidates and
rank), MC-A5 (only a `ResearchImage` reaches a port), MC-A8 (`image_search_call`), MC-A9 (redaction: `api_key`,
`key`, `x-goog-api-key`), §Grounding owed (RESEARCH-VERIFY-6, -7) · evidence doc (R1·U1's sections) · the fixtures
folder of the picked route · code `crates/research/src/adapters/{http.rs, openai/}` (R8·U1, the pattern),
`crates/research/src/ports.rs`.

### Files
New: `crates/research/src/adapters/visual_search/{mod.rs, <route>.rs}`, `crates/research/tests/engine/adapter_search.rs`. Edited: `crates/research/src/adapters/mod.rs`, `crates/research/tests/engine/main.rs`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. The request(s) of the route exactly as R1·U1 recorded them (SerpApi: upload, then search with the returned id;
   Cloud Vision: one annotate call with the image inline).
2. Candidates: the route's page results in the route's own order → `Candidate {url, title, rank}` with rank 1 for
   the first; results without a page URL are dropped.
3. Usage: one `image_search_call` per search (and what R1·U1 recorded for an upload).
4. Failures → `CallFailure` from the captured error replies; the key's place in the request declared as secret.
5. If the route has a size limit below the 1024 copy's size (R1·U1, RESEARCH-VERIFY-7), the adapter declares
   `max_image_bytes`, and the stage asks `ResearchImages::render` for a smaller long edge; the adapter never
   re-encodes an image itself.

### Criteria (6 criteria, 7 rows)
Boundary: `VisualSearchProvider::search` over `FakeResearchHttp` replaying the route's captures.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A16·C1(a) | candidates | the route's search reply cut to 3 results (RESEARCH-VERIFY-6) | three candidates with the capture's URLs and titles, ranks 1, 2, 3 in the capture's order | MC-A3 |
| A16·C1(b) | candidates | the same reply with the results array emptied | no candidate; not a failure | MC-A3 |
| A16·C2(a) | usage | the search reply (and, for SerpApi, the upload reply) | the usage R1·U1 recorded for one search: `image_search_call` × the recorded count | MA15 |
| A16·C3(a) | secrets | any search | the `Exchange` shows the key's query parameter or header as `<redacted>`; the key value appears nowhere in it, including in a URL | MA16 (RESEARCH-VERIFY-7) |
| A16·C4(a) | errors | the route's captured error reply (invalid key) | `CallFailure` with the capture's status; `classify` gives `key` | MC-A9 |
| A16·C5(a) | image | a `ResearchImage` of 1024 | the bytes sent equal the stored copy's bytes (same SHA-256); for a route with a size limit, a copy within the limit is requested from the renderer and sent | MA12 |
| A16·C6(a) | end to end | R4·U3's full-run script with this adapter replaying the capture, the fake model and page reader | the page reader is asked for the capture's pages in MC-A3 order; the call record of the search has one outbound row for the cover | MA3 |

**Variant SerpApi (Lens).** Fixtures `serpapi_lens/`: upload reply, search reply, error reply. C2(a) counts what
R1·U1 measured per upload and per search. C5(a) uses the limit R1·U1 measured.
**Variant Cloud Vision.** Fixtures `cloud_vision/`: web detection reply, error reply. Candidates come from the
pages-with-matching-images list in its order; C5(a) has no size step unless R1·U1 found a limit.

### Delegated choices
Which of the route's result lists beyond the page list are used (none, unless R1·U1's report shows they add correct
pages; record it) · adapter id spelling.

### Notes
- The Lens-through-a-vendor route's legal position is in the evidence doc; the adapter must stay replaceable, which
  the port guarantees.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine adapter_search::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 3 — Page reader (the picked route); shop hosts, own listing and unit names from the shop

**Unit id:** `R8·U3` · **Replaces:** A17 (all rows) · **Changes app files:** yes

### Goal
(1) Implement `PageReader` by the route recorded after R1·U1: **plain HTTP fetch from Rust**, or **the model
provider's page-open tool restricted to the candidate's domain**. (2) Implement `CommerceAdapter::public_listing`
for Shopify (the shop's public hosts, the item's storefront URL and path tail) and `unit_names`, so MC-A3's shop
class and own-listing exclusion and MC-A2's unit conversion work with the real shop. **Not in this phase:** the
other route; the extraction of what a page states (that is stage 3's model call, R4·U3/R8·U1).

### What changed from the superseded plan (this wins over any older wording below)
- The page reader is `crates/research/src/adapters/page_reader/<route>.rs`, registered in `adapters::build`; its rows (A17·C1–C3) are research-crate tests.
- `CommerceAdapter::public_listing` exists since R2·U2; this unit implements it for Shopify (rows A17·C4, app suite, `FakeTransport`).
- There is no `unit_names()` method. The Shopify adapter puts `unit_names` (`{mm, cm, m, in, g, kg, lb}` → its own unit codes) into the `validations` of every dimension and weight definition it imports (`attributes_schema`); row A17·C5(a) reads: a dimension definition as `attributes_schema` returns it, then `map_output` stores `{value: 10, unit: "in"}` as 25.4 in the definition's first unit code. It is tested where the adapter is (app suite).
- A17·C6(a) runs in the app suite on the wired `TestApp::with_research()` when R7 is approved; if R7 is not yet approved, it runs on the research harness with the real Shopify adapter over `FakeTransport` handed to `CommerceShopFacts`, and says so under Assumptions.

### Task 0
Read the evidence doc §Owner picks. No page-reading route recorded → `STATE: BLOCKED` (reason: `owner pick
missing: page-reading route`), change nothing. If the pick is "plain HTTP for some sites, the model tool for
others", build the HTTP route here and report the rest as a decision for the orchestrator.

### Fold before dispatch
Strike the other variant; check fixture names against the fixtures README; copy the per-site "readable or not"
table's outcome into C3's input cell.

### Read first
`master_plan_research.md` §3, §6.1, §6.6 · book `14` (MA11, MA16), `08A` §4, `08B` **MC-A3** (shop hosts, site of a
page, own listing), MC-A2 (units), MC-A5 (third-party images), MC-A8 (`page_fetch`, `web_search_call`), MC-A9,
MC-A11 (replay of stage 3 through a model-side tool), §Grounding owed (RESEARCH-VERIFY-8, -9) · evidence doc
(R1·U1's page-reading table; the shop probe) · fixtures `pages/`, `shop/` · code
`crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, type_map.rs (unit codes), fixtures/}`,
`crates/core/src/ports/commerce.rs` (R4·U3's two methods), `crates/testkit/src/fake_transport.rs`,
`crates/research/src/adapters/http.rs`.

### Files
New: `crates/research/src/adapters/page_reader/{mod.rs, <route>.rs}`, `crates/research/tests/engine/adapter_pages.rs`, `crates/testkit/tests/suite/research/r8_shop_facts.rs` (+ `mod` line). Edited: `crates/research/src/adapters/mod.rs`, `crates/research/Cargo.toml` (an HTML parser, for the HTTP route; version verified and recorded), `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, type_map.rs, fixtures/}`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. **Variant HTTP.** GET the candidate with the timeout `call_timeout_seconds.other`, a generic client name, a
   byte limit, redirects followed and the final URL reported; only `http`/`https`; **never** a host that is an IP
   literal, `localhost`, or resolves to a loopback, private or link-local address (candidates come from search
   results, so the reader must not be steerable into the local network). Extract the title, the visible text and
   the main images (the routes R1·U1 found: Open Graph, JSON-LD, the largest content images); download at most a
   fixed number of images within a byte limit and hand them to the gateway's store (third-party images are never
   outbound rows). Usage: one `page_fetch`. A status or a page R1·U1 recorded as unreadable → `CallFailure` kind
   `Http` with that status; the stage records `could not be read`.
   **Variant model tool.** One model call with the page-open tool restricted to the candidate's host; the reply's
   page text and, if R1·U1 found it is returned, the final URL; usage from the reply (`web_search_call` and tokens).
   No images unless R1·U1 found the tool returns them.
2. `ShopifyAdapter::public_listing(external_id)`: the hosts and, for a linked product, its storefront URL and the
   path tail (`products`, handle), from the query R1·U1 recorded. An unlinked item → hosts only.
3. Schema import: the `validations` of every dimension and weight definition gain `unit_names` (the seven neutral names → this adapter's unit codes, its own constants in `type_map.rs`).

### Criteria (6 criteria, 8 rows)
Boundary: `PageReader::read` over `FakeResearchHttp`; `CommerceAdapter::public_listing` over `FakeTransport`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A17·C1(a) | read | the shop's own listing page capture (`pages/`, RESEARCH-VERIFY-8) | the reply's title equals the product title in the test file for that piece; its text contains the designer named in that piece's metafields; at least one image is returned | MC-A3 |
| A17·C1(b) | redirect | a capture (or a `FakeResearchHttp` script built from the recorded redirect headers) where the candidate redirects to another URL | `final_url` is the redirect target | MC-A3 (RESEARCH-VERIFY-8) |
| A17·C2(a) | usage | one read | exactly the unit of the route: one `page_fetch`, or the model reply's recorded counts | MA15 |
| A17·C3(a) | unreadable | the status R1·U1 recorded for a site that refused the route | `CallFailure` with that status; in R4·U3's script the page becomes a `read` step `could not be read` and does not count toward `max_pages` | MC-A3 (R-IA10) |
| A17·C4(a) | shop hosts | the hosts-and-product capture (`shop/`, RESEARCH-VERIFY-9) | `hosts` = the capture's primary, other and platform domains; `item_url` = its storefront URL; `item_path_tail` = `[products, <handle>]` | MA11 |
| A17·C4(b) | unlinked item | no external id | hosts only; no item URL, empty tail | MC-A3 |
| A17·C5(a) | unit names | a dimension definition as this adapter's `attributes_schema` returns it (its `validations.unit_names` filled), then R3·U1's `map_output` | `{value: 10, unit: "in"}` is stored as 25.4 in the definition's first unit code | MC-A2 |
| A17·C6(a) | end to end | R4·U3's script with the real Shopify adapter over `FakeTransport` and this page reader over captures; a candidate that is the item's own listing with a locale prefix in its path | it is not read and is no evidence; a page on another product of the shop counts as site class shop | MA11 (R-IA9) |

### Delegated choices
The HTML parser crate (pin recorded) · the image and byte limits (state them) · text extraction details.
Not delegated: the address guard of task 1 (it has no criterion row because no contract states it yet;
`master_plan_research.md` §8 lists it for the orchestrator; the reviewer checks it by reading), one request per page,
no JavaScript execution.

### Notes
- A site that needs scripts to show its text cannot be read by the HTTP route; that is an outcome
  (`could not be read`), not a defect. R1·U1's table says which sites these are.
- Replay (R9·U1): a stage 3 call made through a model-side tool cannot reproduce the page content; the adapter marks
  such calls in the record so the tool can say so.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine adapter_pages::` · `cargo test -p psm-testkit --test suite research::r8_shop_facts::` · `cargo test -p psm-testkit --test suite p12_shopify_adapter::` · `cargo check --workspace`

## Closing gate (once, after the last unit; replaces the template's closing gate)
U3 changed app files (the Shopify adapter), so the full gate runs once: `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test` · `cargo test -p psm-research`. One cargo command at a time.

## Review mode
The reviewer runs: `cargo test -p psm-research --test engine adapter_openai::`, `adapter_search::`, `adapter_pages::`, `cargo test -p psm-testkit --test suite research::r8_shop_facts::`, `p12_shopify_adapter::`, `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`, workspace clippy; the implementer's full-suite stamp stands. Re-runs no real call. Checks every provider-owned fixture cell against the fixtures README. Notes: `docs/implementation/backlog_research/R8_research_real_adapters.md`.

## Manual try-out (the narrative is built from this)
On scratch data, shop connected, R7 in place:
1. Settings → Research: paste the model key and the visual-search key; add your nine sites in your order; choose a model per stage (defaults arrive in R10) and enter its prices.
2. Create a session and shoot one photo. Open the form and the research panel: steps appear (looked, searched, read, compared, concluded).
3. When it says Finished, check a suggested designer: its popover shows the confidence, the reason and the links it relied on.
4. Check a piece already in your shop: its own listing is listed as "this item's own listing" and not used.

## Review log
