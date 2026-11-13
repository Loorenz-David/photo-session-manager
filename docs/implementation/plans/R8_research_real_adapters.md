---
phase: R8_research_real_adapters
title: Research with real providers — the app can ask OpenAI to look at photos and read listings, send the cover photo to Google Lens (through SerpApi), read listing pages by plain download, and knows the shop's own web addresses so an item is never evidence for itself
difficulty: high
projection: no
depends_on: [R4_research_engine, R1_research_benchmark]
blocked_on: nothing. The picks are recorded (2026-11-13, evidence doc "Owner picks") — Lens via SerpApi (U2), plain HTTP page reading (U3)
---
# R8 — Real adapters: the model, Google Lens through SerpApi, plain HTTP page reading, the shop's own addresses

**Tracker:** NOT_STARTED · 2026-11-12 · planner · —

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
The three real adapters, in `crates/research/src/adapters/`, each written only from R1's captures (charter rule 17), registered in `adapters::build` and selected by default; and the two Shopify facts research needs (public hosts and the item's own listing; unit names in the imported definitions). **Not here:** default model names and the shipped price table (R10), a Cloud Vision adapter, page reading through a model tool, the shop's other web addresses (`webPresences`, see U3 Notes).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 3 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every older row went (the rows added on 2026-11-13 are listed in the Amendments section below, not there). Name each test after its row (`a15_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R8_research_real_adapters.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | OpenAI model adapter; the HTTP seam; default provider ids; two carried rows (secret forms, `wait_ends_at`) | 7 | 16 | yes (`crates/integrations/src/lib.rs`) |
| U2 | Visual-search adapter: Google Lens through SerpApi | 6 | 7 | no |
| U3 | Page reader: plain HTTP; shop hosts, own listing and unit names from the shop | 7 | 12 | yes |
| **Total** | | **20** | **35** | |

**Gate for the whole milestone.** The evidence doc's section "Owner picks" names the visual-search route and the page-reading route (it does, 2026-11-13). If that section is missing when the session starts, report `STATE: BLOCKED` (reason: `owner pick missing: …`) and change nothing.

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` (with changelog 11) · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`. Also the fixtures README (`crates/research/fixtures/README.md`), the evidence doc `docs/intention/evidence/research_providers.md` (V1–V9 and "Owner picks") and R1's Review log.

## Rules for every unit (orchestrator, 2026-11-13)
- **Captured, derived, held.** A provider-owned shape is used only as a capture under `crates/research/fixtures/` (never edited by hand). Where a row needs a shape that was not captured, the row says **derived**: the test builds it in test code from a named capture (envelope kept, one named part replaced) and its comment cites the evidence. A shape with no evidence is **structurally held**: not implemented, not tested, trigger "the first such reply recorded by a call record".
- **Real type names** (`crates/research/src/{ports.rs, domain.rs}`): `ModelReply {output: Json, usage, exchange}`, `SearchReply {candidates: Vec<SearchCandidate {url, rank, title}>, usage, exchange}`, `PageReply {obtained, final_url, title, text, images, usage, exchange}`, `CallFailure {sent, status, kind, quota, text, usage}` with `FailureKind {Connect, Timeout, Reset, Status, Schema, Refusal}`. `CallFailure` has **no** `exchange` field and there is no `ModelOutput` type and no kind `Http`. **No port signature changes in R8.** The only signature that changes is the constructor `adapters::build` (U1 task 7).
- **Usage units (08B MC-A8, changelog 11).** An adapter reports **only units whose count is greater than 0**, in the order `input_token`, `cached_input_token`, `cache_write_token`, `output_token`, `web_search_call` (model), `image_search_call` (search), `page_fetch` (reader). A request that was answered reports `Some(list)` (the list may be empty); `None` only when the reply carries no usage at all.
- **Tests never reach a network.** Adapter tests run over `FakeResearchHttp`; the only sockets are loopback listeners on port 0 (rows A15·C5(b), (c)). No DNS lookup happens on the fake path (U3 task 2).

## Carried from earlier reviews (orchestrator, 2026-11-12; wins over older wording below)
- **U1–U3, "no credit" vs "bad key" (R3 review; amended 2026-11-13, L8, L20).** `rules::calls::classify` checks 401/403 before the quota flag. No provider's out-of-credit reply was captured (OpenAI's codes are documented only and ambiguous; SerpApi's was not seen), so **no adapter sets `quota: true` in R8** (structurally held). An out-of-credit account therefore shows as its HTTP status says: 429 → transient, three tries, then the run fails with the provider's text.
- **Knowing for failed runs (R4 review).** `store::next_attempt` counts the `unknown` call a kill left, so a unit re-issued after a kill gets two transient tries instead of three. No change required; do not "fix" retry counts in the adapters around it.
- **Secrets in the export scan (R7 review, MA16).** Now U1 task 8 and row A15·C5(d).
- **`ResearchStatus.wait_ends_at` (R7 review).** Now row A15·C7(a).
- **Long calls and settings saves (R7 review, knowing).** A settings save that rebuilds the runtime detaches the old
  research thread after 1 s; with real calls up to 180 s two runners briefly exist. `store::start_run`'s guard keeps one
  running run; no change required unless a rebuild must cancel in-flight research.
- **Picks recorded (orchestrator, 2026-11-13).** The evidence doc's "Owner picks" names Lens via SerpApi (U2) and plain
  HTTP for every site (U3). Cloud Vision is the named replacement and is not built here.
- **U3, Shopify storefronts throttle (R1 review).** Now U3 task 3 and row A17·C3(b).

## Amendments after projection r0 (orchestrator, 2026-11-13; card 1 and ledger L1–L31 of `handoff_R8_research_real_adapters_projection_r0.md`)
The rows and tasks below are already amended in place; this list says where.
- **Card 1 (`read_markets`): No for now**, under the owner's delegation. Research uses the shop's primary domain host and the myshopify host; no `webPresences` query. → U3 task 5, A17·C4(a), U3 Notes; 08B changelog 11(c). The owner may grant `read_markets` later; a follow-up then adds `webPresences`.
- L1 → U3 task 1, A17·C3(a). L2 → U3 task 1, A17·C3(c) (new row); 08B MC-A3 and changelog 11(a).
- L3 → "Rules for every unit", U1 task 4, A15·C1(a)–(c); 08B MC-A8 and changelog 11(b). L4 → A15·C1(b) (kept, outcome 3).
- L5 → U1 Goal, Files, tasks 2 and 3, A15·C4(b), Delegated choices. L6 → U1 header, Files, task 7, Unit test commands, Closing gate.
- L7 → "Rules for every unit", U1 tasks 4–5, A15·C1(c), C2, C3(a), U3 task 1; `master_plan_research.md` §6.6 and §9 (v5).
- L8 → the first carried note, U1 task 5, A15·C2(b), A15·C3(a). L9 → A15·C6(a), U1 Notes. L10 → U1 task 4, A15·C3(b), Delegated choices.
- L11 → U1 Files and Decided. L12 → U1 task 1. L13 → U1 task 2, A15·C4(b). L14 → U1 task 2, A15·C4(a), Decided.
- L15 → A16·C4(a), U2 Notes (option b; no new real call). L16 → U2 task 2, A16·C1(b). L17 → old task 5 struck, A16·C5(a), U2 Notes.
- L18 → the Cloud Vision variant (U2) and the model-tool variant (U3) are struck; both Task 0 sections and "Fold before dispatch" sections are gone (the fold is this amendment).
- L19 → U1 task 1, Delegated choices. L20 → the first carried note, U2 Notes. L21 → U2 task 3, A16·C2(a). L22 → A16·C6(a).
- L23 → U3 task 1, A17·C1(a). L24 → U3 task 1, Not delegated. L25 → U1 task 1, U3 task 1, A17·C1(b). L26 → U1 task 1, U3 task 2.
- L27 → U3 task 3, A17·C1(a), A17·C3(b) (new row). L28 → U3 task 5, A17·C4(a), U3 Notes.
- L29 → A17·C5(a); U3 Notes (OUNCES, latent, not fixed here).
- L30 → option (a): U1 task 6, row A17·C7(a) (new row), "Decided" in U1.
- L31 → rows A15·C5(d) (U1 task 8), A15·C7(a), A17·C3(b); the unit table is recounted: 20 criteria, 34 rows (was 18 / 29; five new rows: A15·C5(d), A15·C7(a), A17·C3(b), A17·C3(c), A17·C7(a)).

**Projection r0, attempt 2 (ledger L1–L13 of the same handoff path; 0 owner cards).** Amended in place:
- L1 → U3 task 1 (`read` never returns `Err`), A17·C3(c) case 3, new row A17·C3(d), the "dead listing domain" Note; 08B MC-A3 body (changelog 11(d)).
- L2 → U1 task 7 (`build(settings, Arc<dyn SecretStore>, ResearchFiles)`, keys read per call), A17·C7(a).
- L3 → U3 Files (`crates/testkit/src/test_app.rs`), "What changed", A17·C6(a).
- L4 → U1 task 3, A15·C4(a), A15·C4(b). L5 → A17·C3(a). L6 → A15·C6(a), U1 Delegated choices.
- L7 → U1 Delegated choices. L8 → U2 task 1, Delegated choices. L9 → A17·C3(b). L10 → A17·C1(b). L11 → U3 task 1.
- L12 → U3 task 5, A17·C4(b), A17·C5(a). L13 → U3 Delegated choices. Trace nit → A16·C6(a) gains MC-A3.
- Recount: U3 7 criteria / 12 rows; total 20 / 35.

Projection is off from attempt 3 (orchestrator, 2026-11-13): two rounds, 0 owner cards after the first; the review is the gate.

## Unit 1 — OpenAI model adapter

**Unit id:** `R8·U1` · **Replaces:** A15 (all rows) · **Changes app files:** yes (`crates/integrations/src/lib.rs`, one call site)

### Goal
Implement `ModelProvider` for the OpenAI Responses API behind an HTTP seam: the engine's instructions sent as they are, a strict output schema per stage built from the engine's own output shape, image input, the mapping of the reported usage to the price-table units, the mapping of every failure to a `CallFailure`, refusals, and the declared secret-bearing parts. **Every provider-owned shape in this unit comes from R1's captures**; nothing is written from memory or from documentation prose. Also here because they are research-crate work the adapters need: the default provider ids, the JSON-escaped secret form, and the `wait_ends_at` row. **Not in this unit:** visual search (U2), page reading (U3), default model names (R10·U1), the settings UI for the key (R6·U2).

### What changed from the superseded plan (this wins over any older wording below)
- The adapter, the HTTP seam and `FakeResearchHttp` are in the research crate (`crates/research/src/adapters/{http.rs, openai/}`, `psm_research::fakes::FakeResearchHttp`); the adapter is registered in `psm_research::adapters::build` (R4·U4). `crates/integrations/src/lib.rs` changes only where it calls `build`.
- The engine owns the instructions (`stages.rs`, `ModelRequest.instructions`; `INSTRUCTIONS_VERSION` in `config.rs`, part of the unit fingerprint). The adapter has **no instructions of its own** and no instructions folder.

### Read first
`master_plan_research.md` §3 (rule 17, keys, HA5), §6.1, §6.6, §7.4 (RESEARCH-VERIFY-11) · book `14` (MA15, MA16), `08A` §4, §5 (stage inputs and outputs), §9, `08B` MC-A2 (the core-type table: what a value of each type looks like), MC-A4 (`sure` flag only; never a confidence), MC-A8 (units), **MC-A9** (record, redaction layer 1, error classes), **§Grounding owed**, changelog 11 · evidence `research_providers.md` (V1–V5, V10; the price rows) · `crates/research/fixtures/openai/` and the README · code `crates/integrations/src/commerce/shopify/transport.rs` (the pattern for a private-runtime HTTP seam), `crates/research/src/{ports.rs, domain.rs, files.rs, config.rs, gateway.rs}`, `crates/research/src/rules/{calls.rs, cost.rs}`, `crates/research/src/stages.rs` and `decide.rs` (the functions that build each stage's `input` and parse its output; find them by name), `crates/research-tool/src/{openai.rs, recording.rs}` (the code that made the captured calls: the request form that is known to work).

### Files
New: `crates/research/src/adapters/{http.rs, openai/{mod.rs, adapter.rs, schema.rs, usage.rs, errors.rs}}`, `crates/research/tests/engine/adapter_openai.rs`. Edited: `crates/research/src/{adapters/mod.rs, fakes.rs, config.rs, rules/calls.rs}`, `crates/research/Cargo.toml` (`reqwest = { version = "0.13", default-features = false, features = ["blocking", "rustls"] }`, the tool crate's set; `base64` if not already there, pin recorded), `crates/research/tests/engine/{main.rs, api.rs or lifecycle.rs (row A15·C7(a)), export.rs (row A15·C5(d), second half)}`, `crates/integrations/src/lib.rs` (the one `adapters::build` call).

### Core mechanisms (the only functions with unit tests)
None (adapter mapping is proven at the port boundary with captured replies).

### Ordered tasks
1. **HTTP seam.** `ResearchHttp` (trait, sync): one request → `{status, headers, body bytes}` or a transport failure `{sent, kind: Connect | Timeout | Reset, text, blocked}`. The request shape is delegated (it must carry a JSON body, a hand-built multipart body and a plain GET) with one constraint: `FakeResearchHttp` records, for each request, the method, the URL, the headers and every body part (so a test can hash the image bytes that were sent). `ReqwestResearchHttp::new(timeout: Duration, guard)`:
   - `is_connect()` → `sent: false`, kind `Connect`; `is_timeout()` after send → `sent: true`, kind `Timeout`; any other transport failure → `sent: true`, kind `Reset`; error text built after `without_url()`.
   - `redirect::Policy::none()` always (a caller that wants redirects follows `Location` itself, U3).
   - `guard` is `Any` or `PublicOnly`. With `PublicOnly` the client uses a custom DNS resolver that refuses every loopback, private, link-local and unspecified address (IPv4 and IPv6); that refusal is reported as a transport failure with `blocked: true`, `sent: false`. `build` gives the page reader a `PublicOnly` client and the two provider adapters an `Any` client. Tests of rows C5(b), (c) construct `Any`.
   - `build` converts `call_timeout_seconds.model` / `.other` from seconds to a `Duration`.
2. **Request building** from `ModelRequest`, in the form `research-tool/src/openai.rs` used for the captured calls. Top-level keys exactly: `input`, `instructions`, `max_output_tokens`, `model`, `store`, `text`, plus `reasoning` only when `request.effort` is not empty (`""` → no `reasoning` key). `instructions` = `request.instructions` verbatim. `max_output_tokens: 4000`. `store` as captured. Each of `request.images` is sent as `data:image/jpeg;base64,<bytes of the ResearchImage>` with **no `detail` key**. Each of `request.listing_images` (a SHA-256) is read from `ResearchFiles::path(sha)` and sent the same way, after our photos; a listing image whose file is missing is left out and the rest is sent (never a failure). `request.input` (with `request.language`) is the text part.
3. **Strict schema per stage** (`schema.rs`), built per call; it **is the engine's output shape**, the keys that stage's parser reads and no others:
   - observe, condition: `{reads: {<key>: {value, sure}}}` (+ `shows_marking: boolean` for the stage whose parser reads it, `stages.rs`); `value` is **nullable** (null = the photo does not show it; the parser already treats null as nothing read), `sure` is a required boolean;
   - read: `{keys: {<key>: {stated, value}}}` with `stated` and `value` nullable;
   - compare: `{verdict}` ∈ `match` / `similar` / `different`;
   - labels: `{labels: {<key>: {stated, value}}, text}`, `stated` and `value` nullable (the parser is `statements(&keys, output, "labels")`); decide: `{reasons: {<key>: string}}`.
   Properties under `reads` / `keys` / `labels` / `reasons` are keyed by the definition keys in `request.input["fields"]` (descriptors `key`, `core_type`, `choices`), **all required, `additionalProperties: false`** at every level. Each `value` schema follows 08B MC-A2's core-type table (a choice field is an `enum` of the definition's own choice strings; a measurement is `{value, unit}`), or null where the bullet above says nullable. No field for a confidence exists.
4. **Reply handling** from the captures:
   - `status: "completed"` with an `output_text` → parse it as JSON → check it against the schema of task 3 with a small hand-written check (required keys present, no extra keys, value kinds; no new crate) → `Ok(ModelReply {output, usage, exchange})`. A reply that fails the parse or the check → `CallFailure {sent: true, status: Some(200), kind: Schema, usage}`.
   - `status: "incomplete"` (`incomplete_max_output_tokens.json`: only a `reasoning` item) → `CallFailure {kind: Schema, usage}` with the usage the reply reports.
   - a `message` item whose content holds `{"type": "refusal", "refusal": "<text>"}` → `CallFailure {sent: true, status: Some(200), kind: Refusal, text: <text>, usage}` (documented shape, evidence V3; the engine turns it into "not determined").
   - **Usage** (`usage.rs`): `input_token` = `input_tokens − input_tokens_details.cached_tokens − input_tokens_details.cache_write_tokens`; `cached_input_token` = `cached_tokens`; `cache_write_token` = `cache_write_tokens`; `output_token` = `output_tokens` (reasoning tokens are inside it); `web_search_call` = `tool_usage.web_search.num_requests`. Only counts > 0 are reported.
5. **Errors** (`errors.rs`): an HTTP error status → `CallFailure {sent: true, status: Some(code), kind: Status, quota: false, text: the provider's error message, usage: None}`. `quota` is never set (carried note 1). Transport failures map as task 1 says.
6. **Default provider ids** (`config.rs`): constants `openai`, `serpapi_lens`, `page_http` (the provider names in R1's captures). `ProviderIds::default()` returns them, and one function (`ProviderIds::resolved()` or equivalent) turns a saved `""` into the default; `build` **and** the run's settings snapshot (`config.rs`, where `providers` is copied) both use the resolved ids, so call records and the price table see the same id. An unknown non-empty id still gives no adapter. `build` returns an adapter for a port as soon as that unit has built it (U1: model only).
7. **`adapters::build(settings: &ResearchSettings, secrets: Arc<dyn SecretStore>, files: ResearchFiles)`** — the one signature change. `crates/integrations/src/lib.rs` passes `parts.secrets.clone()` and `ResearchFiles::new(<the app data dir>)`. The model adapter reads `research-model-key` (U2: `research-search-key`) from the store **on every call**, so a key saved while the app runs is used by the next call (the app does not rebuild the adapters on a secrets save). **`build` never looks at whether a key is set**: it returns the adapter for every known id; the prerequisites (`runs.rs`, `providers.model.is_some() && key_set`) decide "Add the model key".
8. **`Exchange` and secrets.** `authorization` (and any other header the captures show carrying the key) is `<redacted>` in the `Exchange`. In `rules::calls::redact`, add the **JSON-escaped form** of each secret (the secret as `serde_json` writes it inside a string, without the outer quotes) to the forms it replaces; this covers the gateway's call records and the export scan together.

### Criteria (7 criteria, 16 rows)
Boundary: `ModelProvider::call` on the adapter over `FakeResearchHttp`; outcome = the `ModelReply` / `CallFailure` and the request the fake received. Fixture paths are under `crates/research/fixtures/openai/`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A15·C1(a) | usage | `image_strict_schema_gpt-6.1-sol.json` (RESEARCH-VERIFY-1): `input_tokens` 1316, `cache_write_tokens` 1313, `cached_tokens` 0, `output_tokens` 26 | `usage == [("input_token", 3), ("cache_write_token", 1313), ("output_token", 26)]`, nothing else. Cross-check in the same test: `call_cost` of that usage at 2.00 / 2.50 / 10.00 USD per million = **3549** micro-USD, the capture's `cost_micro_usd` | MA15 |
| A15·C1(b) | usage | `image_and_web_search.json` (RESEARCH-VERIFY-1, -5): 4 `web_search_call` items, `tool_usage.web_search.num_requests` 3. The reply is fed to the usage mapping (no R8 request sends a tool; the mapping is kept for MA15) | the usage contains `("web_search_call", 3)`, and `("input_token", 19715)`, `("cache_write_token", 5746)`, `("output_token", 561)` | MA15 |
| A15·C1(c) | usage | `incomplete_max_output_tokens.json` (RESEARCH-VERIFY-1, -3) | `CallFailure {sent: true, kind: Schema}` with `usage == Some([("input_token", 3), ("cached_input_token", 1313), ("output_token", 16)])` | MA15 |
| A15·C2(a) | errors | `error_invalid_key.json` (HTTP 401, RESEARCH-VERIFY-2) | `CallFailure {status: Some(401), kind: Status, quota: false}`; `classify` gives `key` | MC-A9 |
| A15·C2(b) | errors | derived: a reply with status 429 and an empty JSON body (`classify` uses the status alone, `rules/calls.rs`) | `CallFailure {status: Some(429), kind: Status, quota: false}`; `classify` gives `transient`. **Structurally held:** the no-quota reply (never captured; the quota flag is never set) | MC-A9 |
| A15·C2(c) | errors | `error_context_length.json` (HTTP 400, RESEARCH-VERIFY-2) | `CallFailure {status: Some(400), kind: Status}`; `classify` gives `request` | MC-A9 |
| A15·C3(a) | refusal | derived (documented shape, V3): the envelope of `stage_observe_gpt-6-luna.json` with the message content replaced by `[{"type": "refusal", "refusal": "I can't help with that."}]` | `CallFailure {sent: true, kind: Refusal, text: "I can't help with that."}` with the envelope's usage `[("input_token", 3), ("cache_write_token", 1555), ("output_token", 77)]` | MC-A9 |
| A15·C3(b) | schema | derived: the same envelope with `output_text` = an observe payload that fits the task 3 schema for a two-field form, **minus `shows_marking`** | `CallFailure {sent: true, status: Some(200), kind: Schema}`; with `shows_marking` present the same call is `Ok` | MC-A9 |
| A15·C4(a) | request | an observe request with one image at 1024, a choice field with choices `[Oak, Teak]`, language `Swedish` | the body the fake received has exactly the top-level keys `input`, `instructions`, `max_output_tokens`, `model`, `store`, `text` (effort `""`); its image part is `data:image/jpeg;base64,` + the base64 of that `ResearchImage`'s bytes, with no `detail` key; the schema under `text` gives the choice field's `value` as the enum `["Oak", "Teak"]` **or null**; `Swedish` is in the request; `max_output_tokens == 4000` | MC-A2 |
| A15·C4(b) | request | one request for each of observe, read, compare, labels, condition, decide, each with instructions `"I-<stage>"`, model `m-<stage>`, effort `low` (and one observe request with effort `""`) | each body's `instructions` equals its `"I-<stage>"` byte for byte, its `model` is `m-<stage>`, it has a `reasoning` key carrying `low`, and its schema has the top-level keys of that stage in task 3; the `""` request has no `reasoning` key | MC-A7 (unit identity: model, effort, instructions version) |
| A15·C5(a) | secrets | any call | the `Exchange` shows `authorization` as `<redacted>`; the key value appears nowhere in it | MA16 |
| A15·C5(b) | transport | `ReqwestResearchHttp` (`Any`) against a closed port on `127.0.0.1` | `CallFailure {sent: false, kind: Connect}`; its text contains no URL | MC-A9 (RESEARCH-VERIFY-11) |
| A15·C5(c) | transport | `ReqwestResearchHttp::new(200 ms, Any)` against a local listener that accepts and never answers | `CallFailure {sent: true, kind: Timeout}` | MC-A9 |
| A15·C5(d) | secrets | a model key `sk-a"b\c`. (1) A model call through the gateway whose scripted response text holds the key as JSON writes it inside a string (`sk-a\"b\\c`). (2) An export over a run folder in which a file holds that escaped form (seeded the way R7's fail-closed export test seeds its key) | (1) the stored call record's response shows `<redacted>` and contains neither the plain nor the escaped key; (2) the export fails with `Export research stopped: <file> contains a key` and leaves no folder | MA16 |
| A15·C6(a) | end to end | the full-run script `a12_c1a_a_full_run_fills_the_form` (`tests/engine/stages.rs`) with this adapter over `FakeResearchHttp`, the fake search and the fake page reader. Model replies are **derived**: the captured envelope of `stage_observe_gpt-6-luna.json` (observe), `stage_read_gpt-6-luna.json` (read), `stage_condition_gpt-6-luna.json` (condition) and `image_strict_schema_gpt-6-luna.json` (compare, decide), each with `output_text` replaced by JSON that fits the task 3 schema for the harness form (that payload is ours). Price table: `openai` / the harness model at 0.10 input, 0.01 cached, 0.125 cache write, 0.50 output | the run finishes and writes the harness form's named values; the run has exactly 10 model call rows with these `cost_micro_usd`: observe ×2 at **234**, read ×3 at **243**, compare ×3 at **194**, condition ×1 at **4076**, decide ×1 at **194** (the script makes no labels call: observe says `shows_marking: false`) | MA15 |
| A15·C7(a) | status | service level, manual clock, `source_wait_seconds` 120, a queued run that is waiting for the editor, 30 000 ms after the session's last change: (1) an automatic run, (2) a manual run | `ResearchStatus.wait_ends_at` is (1) `Some(now + 90 000)`, (2) `Some(now)`; with no waiting run it is `None` | MC-A6 (readiness) |

### Decided by the orchestrator (not free)
- reqwest: `0.13`, `default-features = false`, features `blocking`, `rustls`; multipart bodies are built by hand as `research-tool/src/recording.rs` does (no `multipart` feature).
- No `detail` key on images (the provider's default); `max_output_tokens: 4000` (R1's value).
- The schema check is hand-written over the schema this adapter generated; no validator crate.
- Adapter ids: `openai`, `serpapi_lens`, `page_http`.

### Delegated choices
The request shape of `ResearchHttp` (within task 1's constraint) · how `FakeResearchHttp` matches a reply to a request (in sequence, or by method + URL prefix), with one constraint: **an unscripted request gets a fixed 404 reply and is recorded; it never panics** (record the choice) · how A15·C6(a) reuses the full-run script: make `full()` in `tests/engine/stages.rs` reusable (that file then joins this unit's Files) or copy the script into `adapter_openai.rs` (record which) · where the `wait_ends_at` test lives (`api.rs` or `lifecycle.rs`, wherever the waiting-for-editor state is already set up; record it). Not delegated: a model never returns a confidence (only `sure` on photo reads), no shape is taken from memory, the instructions are the engine's and are sent unchanged.

### Notes
- C5(b) and C5(c) use loopback sockets on port 0, as `p43_ftp_receiver` does; they are not external services.
- C6(a): compare and decide were never captured (labels neither, and the script makes no labels call), and the captured observe / read / condition payloads are R1's version-0 shapes that the engine's parsers do not read. That is why only the envelopes (status, headers, `usage`) are used; the four expected costs are hand-computed from each capture's usage and equal each capture's own `cost_micro_usd`.
- `cache_write_token` has no row in any shipped price table until R10; until then the owner's own price rows must include it, or a call that reports it is charged the fallback (MC-A8). Say so in the narrative's try-out.
- Default ids mean a fresh install shows "Add the model key" until a key is saved and then "No price is set for openai <model>" until prices are entered: both are correct.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine adapter_openai::` · `cargo test -p psm-research --test engine a15_c` (the two rows outside `adapter_openai.rs`) · `cargo test -p psm-research --lib rules::calls` · `cargo test -p psm-testkit --test suite research::r7_wiring::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 2 — Visual-search adapter: Google Lens through SerpApi

**Unit id:** `R8·U2` · **Replaces:** A16 (all rows) · **Changes app files:** no

### Goal
Implement `VisualSearchProvider` for Google Lens through SerpApi (the owner pick). The port stays as it is, so Cloud Vision can be added later. **Not in this unit:** Cloud Vision, page reading (U3), ranking or site rules (core, R3·U2).

### What changed from the superseded plan (this wins over any older wording below)
- The adapter is `crates/research/src/adapters/visual_search/serpapi_lens.rs`, registered in `adapters::build` under the id `serpapi_lens`.

### Read first
`master_plan_research.md` §3, §6.1, §6.6, §7.4 · book `14` (MA12, MA15, MA16), `08A` §4, `08B` MC-A3 (candidates and rank), MC-A5 (only a `ResearchImage` reaches a port), MC-A8 (`image_search_call`), MC-A9 (redaction: `api_key`), §Grounding owed (RESEARCH-VERIFY-6, -7) · evidence doc V6, V7 · `crates/research/fixtures/serpapi_lens/` · code `crates/research/src/adapters/{http.rs, openai/}` (U1, the pattern), `crates/research/src/ports.rs`, `crates/research-tool/src/recording.rs` (the upload and search calls that were captured).

### Files
New: `crates/research/src/adapters/visual_search/{mod.rs, serpapi_lens.rs}`, `crates/research/tests/engine/adapter_search.rs`. Edited: `crates/research/src/adapters/mod.rs`, `crates/research/tests/engine/main.rs`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. The two requests exactly as captured: the upload (`upload.json`: multipart with the image and `api_key`), then the search by the returned `image_id` (`search_cut_to_3.json`: `engine=google_lens`, `type=all`, `hl`, `country`, `api_key` in the query; the `image_id` is the one the upload returned). The image part is the `ResearchImage`'s stored bytes, unchanged.
2. Candidates: `visual_matches` in the reply's own order → `SearchCandidate {url: link, title, rank}` with rank 1 for the first; results without a page URL are dropped. A 200 reply whose `visual_matches` is **absent or empty** gives no candidates and is not a failure.
3. Usage: `search()` reports `[("image_search_call", 1)]` for the upload and search together (uploads cost no credit, V7).
4. Failures → `CallFailure {sent: true, status: Some(code), kind: Status, quota: false}` from the captured error replies (`error_upload_too_large.json`, `error_image_id_expired.json`: both HTTP 400); transport failures as U1 task 1. `api_key` is `<redacted>` in the `Exchange`, in the multipart part and in the URL.

### Criteria (6 criteria, 7 rows)
Boundary: `VisualSearchProvider::search` over `FakeResearchHttp` replaying the captures.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A16·C1(a) | candidates | `upload.json` then `search_cut_to_3.json` (RESEARCH-VERIFY-6) | three candidates, ranks 1, 2, 3: `https://auctionet.com/en/5091518-poul-hundevad-extendable-dining-table-teak-1960s`, `https://beyovintage.se/en/products/swedish-side-table-in-teak-by-alberts-in-tibro`, `https://www.pamono.se/small-vintage-extendable-dining-table-in-teak-1950`, each with the capture's title | MC-A3 |
| A16·C1(b) | candidates | **derived, not captured** (no recorded reply lacks results): the search capture with (1) `visual_matches: []`, (2) the `visual_matches` key removed | no candidate and `Ok` in both forms; usage as C2(a) | MC-A3 |
| A16·C2(a) | usage | the upload reply and the search reply | `usage == Some([("image_search_call", 1)])`; no unit for the upload | MA15 |
| A16·C3(a) | secrets | any search | the `Exchange` shows `api_key` as `<redacted>` in the upload's multipart part and in the search URL; the key value appears nowhere in it | MA16 (RESEARCH-VERIFY-7) |
| A16·C4(a) | errors | `error_upload_too_large.json` (HTTP 400) as the upload's reply | `CallFailure {status: Some(400), kind: Status, quota: false}`; `classify` gives `request`; no search request was sent. **Structurally held:** the invalid-key reply (never captured; all recorded searches were HTTP 200) | MC-A9 |
| A16·C5(a) | image | a `ResearchImage` of 1024 | the image bytes in the upload the fake received have the same SHA-256 as the stored copy's bytes | MA12 |
| A16·C6(a) | end to end | the full-run script `a12_c1a_a_full_run_fills_the_form` with this adapter replaying the two captures, the fake model and the fake page reader; shop hosts `[beyovintage.se]`, trusted sites `[auctionet.com]`; the harness item is not linked (so the beyovintage page is another product of the shop, not the item's own listing) | the page reader is asked, in this order, for the beyovintage.se URL, the auctionet.com URL, the www.pamono.se URL; the search's call record has one outbound row, for the cover | MC-A3, MA3 |

### Delegated choices
Which of the reply's lists beyond `visual_matches` are used (none, unless R1's report shows they add correct pages; record it) · the values of `hl` and `country` (recommended: constants as captured, `hl=en`, `country=se`; they decide which market's results come back; record them).

### Notes
- The legal position of reaching Lens through a vendor is in the evidence doc; the adapter must stay replaceable, which the port guarantees. Cloud Vision is the named replacement and is not built.
- **No size step.** SerpApi's upload limit is 500 KB (V7) and the 1024 copies measure 67–220 KB, so the adapter declares no size limit and never re-encodes. A copy over the limit would get the captured 400 → `request` (row C4(a)).
- **Structurally held:** SerpApi's invalid-key reply and its out-of-credit reply (neither captured). Until one is recorded, a bad key or an empty account shows as its HTTP status says (401/403 → "key rejected"; 429 → transient, then the run fails).

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine adapter_search::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 3 — Page reader: plain HTTP; shop hosts, own listing and unit names from the shop

**Unit id:** `R8·U3` · **Replaces:** A17 (all rows) · **Changes app files:** yes

### Goal
(1) Implement `PageReader` as a **plain HTTP download from Rust** (the owner pick, for every site). (2) Implement `CommerceAdapter::public_listing` for Shopify (the shop's public hosts, the item's storefront URL and path tail) and `unit_names`, so MC-A3's shop class and own-listing exclusion and MC-A2's unit conversion work with the real shop. **Not in this unit:** reading through a model tool; the shop's other web addresses (`webPresences`); the extraction of what a page states (that is stage 3's model call, R4·U3 / U1).

### What changed from the superseded plan (this wins over any older wording below)
- The page reader is `crates/research/src/adapters/page_reader/http.rs`, registered in `adapters::build` under the id `page_http`; its rows (A17·C1–C3) are research-crate tests.
- `CommerceAdapter::public_listing` exists since R2·U2; this unit implements it for Shopify (rows A17·C4, app suite, `FakeTransport`).
- There is no `unit_names()` method. The Shopify adapter puts `unit_names` (`{mm, cm, m, in, g, kg, lb}` → its own unit codes) into the `validations` of every dimension and weight definition it imports (`attributes_schema`). It is tested where the adapter is (app suite).
- A17·C6(a) runs in the app suite on a wired app (R7 is approved). `TestApp::with_research()` wires `FakeCommerce` and fake providers only, so this unit adds one constructor to the testkit: `TestApp::with_research_using(commerce: Arc<dyn CommerceAdapter>, providers: Providers)`, which does what `with_research()` does with the given commerce adapter and the given providers in the research seam.

### Read first
`master_plan_research.md` §3, §6.1, §6.6 · book `14` (MA11, MA16), `08A` §4, `08B` **MC-A3** (shop hosts, site of a page, own listing, the page sentence amended by changelog 11), MC-A2 (units), MC-A5 (third-party images), MC-A8 (`page_fetch`), MC-A9, §Grounding owed (RESEARCH-VERIFY-8, -9) · evidence doc V8, V9, "Page reading", "Owner picks" 2 · fixtures `pages/`, `shop/` · code `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, type_map.rs (unit codes), value_map.rs, fixtures/}`, `crates/core/src/ports/commerce.rs` (`PublicListing {hosts, item_url, item_path_tail: String}`), `crates/testkit/src/fake_transport.rs`, `crates/research/src/adapters/http.rs`, `crates/research/src/gateway.rs` (`Waiter`), `crates/research/src/rules/sites.rs` (`is_own_listing`).

### Files
New: `crates/research/src/adapters/page_reader/{mod.rs, http.rs}`, `crates/research/tests/engine/adapter_pages.rs`, `crates/testkit/tests/suite/research/r8_shop_facts.rs` (+ `mod` line in `crates/testkit/tests/suite/research/mod.rs`). Edited: `crates/testkit/src/test_app.rs` (the constructor `with_research_using`), `crates/research/src/adapters/mod.rs` (`build` gains a `Waiter` only if it has none to pass; see task 3), `crates/research/Cargo.toml` (an HTML parser; version verified and recorded), `crates/research/tests/engine/main.rs`, `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, type_map.rs, fixtures/}`, `crates/integrations/src/lib.rs` (only if `build` gains the waiter).

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. **Reading a page.** GET the candidate through a `PublicOnly` `ResearchHttp` client with the timeout `call_timeout_seconds.other`, a generic client name, the request headers of the capture (`accept`, `accept-language`), and a byte limit.
   - **Redirects:** the client follows none; the reader follows `Location` itself, at most **5 hops**, checking the address guard (task 2) on every hop; `final_url` = the URL of the last hop that answered 2xx. A sixth redirect → could not be read.
   - **Extraction:** title = `og:title`, else the `name` of the JSON-LD object whose `@type` is `Product`, else `<title>`; the visible text; the main images by the routes R1 found (`og:image`, JSON-LD, then the largest content images).
   - **Requests per page:** one request for the page document (per hop), plus at most N image requests within a byte limit (N and both limits are delegated; state them). The guard and the pace (task 3) apply to every request, image hosts included. Images go back in `PageReply.images` (third-party bytes: stored by the gateway, never outbound rows).
   - **Usage:** `[("page_fetch", 1)]` per `read()` in which at least one request was sent, however many hops or images.
   - **Could not be read (08B MC-A3, changelog 11).** Each of these is `Ok(PageReply {obtained: false, usage: Some([("page_fetch", 1)]), exchange: status + headers, …})`, **never** a `CallFailure`: any non-2xx status from the site after the Retry-After handling of task 3 (Etsy's 403, a 429 that stays, a 404, a 5xx); the reader's own timeout or a reset **after send**; a body over the byte limit; too many redirects; a page with no readable text. A request the guard refused before anything was sent is `obtained: false` with `usage: Some([])`.
   - **`PageReader::read` never returns `Err`** (08B MC-A3, changelog 11(d)). A transport failure of the page request with nothing sent (connect, DNS, the guard) → `Ok(PageReply {obtained: false, usage: Some([]), …})`; one after send (timeout, reset) → `Ok(PageReply {obtained: false, usage: Some([("page_fetch", 1)]), …})`. A dead listing domain is therefore "could not be read", never an offline hold; offline is detected by the model and search calls. A failed **image** request never fails the read: that image is left out.
2. **Address guard** (candidates come from search results, so the reader must not be steerable into the local network). Two halves, both checked for the page URL, every redirect hop and every image URL:
   - in the reader, without any lookup: only `http` / `https`; never a host that is an IP literal or `localhost` → `obtained: false` (nothing sent);
   - in `ReqwestResearchHttp` (`PublicOnly`, U1 task 1): the custom DNS resolver refuses loopback, private, link-local and unspecified addresses; the reader maps that failure (`blocked: true`) to `obtained: false`, not to a `CallFailure`.
   Over `FakeResearchHttp` no name is ever resolved.
3. **Slow hosts.** The reader takes a `gateway::Waiter` (the wiring's sleep; a recording closure in tests; `build` passes it, changing `build`'s arguments and `lib.rs` only if needed) and keeps, in memory, the set of **paced hosts**: a host that has answered with `powered-by: Shopify` or an `x-shopid` header, or with a 429. Before every request to a paced host the reader calls the waiter with **2 000 ms**; other hosts get no wait. A 429 with `Retry-After: n` seconds, `n` ≤ `call_timeout_seconds.other`, → the waiter is called once with `n × 1000` and the request is sent once more (one retry per request; the 2 000 ms pace is not added to it); a 429 without the header, with a larger `n`, or a second 429 → could not be read. One request at a time app-wide is already guaranteed (MC-A8).
4. Register the reader in `adapters::build` (id `page_http`; it needs no key, so `reader_key` stays unused).
5. `ShopifyAdapter::public_listing(external_id)`: one query `shop { primaryDomain { host url } myshopifyDomain } product(id: $id) { handle onlineStoreUrl }` — **no `webPresences`** (it needs `read_markets`, which the app does not have; card 1 = No for now). `hosts` = the primary domain's host and the myshopify host, without duplicates. `item_url` = `onlineStoreUrl` (null → `None`). `item_path_tail` = `"products/<handle>"`. An unlinked item (`external_id == ""`, as `research/src/shop.rs` passes it) → the query carries no `product` field: hosts, `item_url` `None`, tail `""`. A `product: null` for a linked id → the same as unlinked.
6. Schema import: the `validations` of every dimension and weight definition gain `unit_names` (the seven neutral names → this adapter's unit codes, its own constants in `type_map.rs`).

### Criteria (7 criteria, 12 rows)
Boundary: `PageReader::read` over `FakeResearchHttp` with a recording waiter; `CommerceAdapter::public_listing` over `FakeTransport`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A17·C1(a) | read | `pages/shop_listing_page.json` (RESEARCH-VERIFY-8; `https://beyovintage.se/en/products/swedish-bookshelf-in-oak-5`, reply header `powered-by: Shopify`); the fake answers the page's `og:image` URL with test bytes | `obtained == true`; `title == "Swedish bookshelf in oak"` (its `og:title`, not the `<title>` "Vintage Swedish bookshelf in oak buy in Stockholm"); `text` contains a phrase the test quotes from the capture's product description; `images` holds the fake's bytes for the `og:image` URL (≥ 1 image); the waiter was called with 2 000 before each image request to `beyovintage.se` and not before the page request | MC-A3 |
| A17·C1(b) | redirect | derived (HTTP semantics; no redirect was recorded, V8): the fake answers `https://a.example/x` with 301 and `Location: https://b.example/y`, and that with the 200 of `shop_listing_page.json` | `obtained == true`, `final_url == Some("https://b.example/y")`. Five 301 answers in a row and then the 200: `obtained == true`, `final_url` = the fifth `Location`. Six 301 answers in a row: `obtained == false` | MC-A3 (RESEARCH-VERIFY-8) |
| A17·C2(a) | usage | the read of C1(a) (one page request and its image requests) | `usage == Some([("page_fetch", 1)])` | MA15 |
| A17·C3(a) | unreadable | `pages/www_etsy_com.json` (HTTP 403). (1) `read()` alone; (2) the full-run script `a12_c1a_a_full_run_fills_the_form` with this reader and `max_pages: 2`: the page that is first in reading order gets that 403, the two pages after it get the 200 of `pages/shop_listing_page.json` (derived: envelope kept) | (1) `Ok` with `obtained == false`, `usage == Some([("page_fetch", 1)])`; (2) the steps hold one `read` step "could not be read" and two `read` steps with a title (both readable pages were read, so the 403 page did not count toward `max_pages`), and the run's state is exactly **`Finished`** | MC-A3 (R-IA10) |
| A17·C3(b) | throttled | `pages/shop_429_throttled.json` (HTTP 429, `Retry-After: 60`) then the 200 of `shop_listing_page.json`, `call_timeout_seconds.other` 60. Second case: the 429 twice | first case: the waiter's record holds exactly one 60 000 entry, made before the second page request; every other entry is 2 000 and precedes an image request; `obtained == true`; `usage == Some([("page_fetch", 1)])`. Second case: `obtained == false` after exactly one 60 000 entry | MC-A3 (RESEARCH-VERIFY-8) |
| A17·C3(c) | transport | scripted transport failures on the page request: (1) `{sent: true, kind: Timeout}`; (2) `{sent: true, kind: Reset}`; (3) `{sent: false, kind: Connect}`; (4) `{sent: false, blocked: true}` | (1), (2): `Ok`, `obtained == false`, `usage == Some([("page_fetch", 1)])`; (3): `Ok`, `obtained == false`, `usage == Some([])`; (4): `Ok`, `obtained == false`, `usage == Some([])`. No case returns `Err` | MC-A3 (changelog 11(a), (d)) |
| A17·C3(d) | dead domain | the full-run script `a12_c1a_a_full_run_fills_the_form` with this reader; the page request of one candidate is scripted `{sent: false, kind: Connect}`, the other candidates get the 200 of `pages/shop_listing_page.json` (derived: envelope kept) | that page is a `read` step "could not be read"; the other pages are read (a `read` step with a title each); the run's state is exactly **`Finished`** | MC-A3 (changelog 11(d)) |
| A17·C4(a) | shop hosts | **schema-grounded, not observed** fixture in `shopify/fixtures/`: the shop part of `shop/hosts_query.json` with `primaryDomain.host` `beyovintage.se` and `myshopifyDomain` `example.myshopify.com`, and a product part built to the Admin GraphQL schema (`handle: String!` = `swedish-bookshelf-in-oak-5`, `onlineStoreUrl: URL` = `https://beyovintage.se/products/swedish-bookshelf-in-oak-5`) | `hosts == ["beyovintage.se", "example.myshopify.com"]`; `item_url == Some("https://beyovintage.se/products/swedish-bookshelf-in-oak-5")`; `item_path_tail == "products/swedish-bookshelf-in-oak-5"`; the query the fake received contains no `webPresences`. Second case, `onlineStoreUrl: null`: `item_url == None`, same hosts and tail | MA11 |
| A17·C4(b) | unlinked item | no external id, which reaches the adapter as `public_listing("")`; the shop part of `shop/hosts_query.json` as captured (primary = myshopify = `example.myshopify.com`) | `hosts == ["example.myshopify.com"]`; `item_url == None`; `item_path_tail == ""`; the query the fake received has no `product` field | MC-A3 |
| A17·C5(a) | unit names | a dimension definition as this adapter's `attributes_schema` returns it (its `validations.unit_names` filled; first unit code `MILLIMETERS`), then R3·U1's `map_output` | `{value: 10, unit: "in"}` is stored as exactly the JSON `{"value": "254", "unit": "MILLIMETERS"}` (the value is a string, `map_output`'s canonical decimal form) | MC-A2 |
| A17·C6(a) | end to end | `TestApp::with_research_using(commerce, providers)`: `commerce` = the real `ShopifyAdapter` over `FakeTransport` (the C4(a) fixture); `providers` = the fake model, the fake search and the `page_http` reader over `FakeResearchHttp` with the harness waiter. The item is linked with `external_id` = the product gid the fixture's variables answer. Candidates: the item's own listing with a locale prefix (`https://beyovintage.se/en/products/swedish-bookshelf-in-oak-5`) and another product of the shop | the own listing is not read and is no evidence (listed as the item's own listing); the other page is read and counts as site class `shop` | MA11 (R-IA9) |
| A17·C7(a) | selection | `adapters::build` with an **empty secret store** in every case and (1) default `ResearchSettings`, (2) settings saved with `providers` all `""`, (3) `providers.model` = `"nope"` | (1), (2): all three ports are `Some`, with ids `openai`, `serpapi_lens`, `page_http`, and a run's settings snapshot carries those three ids; (3): `model` is `None`, the other two `Some` | MC-A6 (prerequisites), MC-A8 (price table keyed by provider id) |

### Delegated choices
The HTML parser crate (pin recorded) · the image count N and the two byte limits (state them) · text extraction details · the text phrase quoted in C1(a) · the GraphQL operation name of task 5's query (`FakeTransport` keys replies by operation name and variables), and whether the unlinked case is the same operation without the product part or a second one (record both).
Not delegated: the address guard of task 2 (it has no criterion row of its own because no contract states it yet; `master_plan_research.md` §8 lists it for the orchestrator; the reviewer checks both halves by reading; row A17·C3(c) case 4 covers its mapping), one request for the page document per hop and at most N image requests, the hop limit 5, the 2 000 ms pace, no JavaScript execution.

### Notes
- A site that needs scripts to show its text cannot be read by download; that is an outcome ("could not be read"), not a defect. Etsy and vntg stay unread (evidence, "Owner picks" 2); their search results still count by title.
- **Other shop addresses.** With card 1 = No, research knows the primary domain and the myshopify host only. That is correct while the shop answers on no other address. If the owner later grants `read_markets`, a follow-up adds `webPresences` to the query of task 5 and its hosts to `hosts`.
- **Weights (latent, not fixed here).** `WEIGHT_UNITS` starts with `OUNCES`, which is not one of MC-A2's seven neutral names, so a weight value could not be converted ("the field's unit has no known name"). The owner's shop has no weight-type metafield, so nothing fails today. Do not reorder the constant or add `oz` in this milestone; it is with the orchestrator.
- **A dead listing domain (orchestrator, 2026-11-13; overrides L1's transport clause for the page reader only):** every
  transport failure of the **page reader** (DNS failure, connection refused, connect timeout, timeout or reset after
  send) is `obtained:false` ("could not be read"), never an offline hold, so one dead listing domain cannot hold a
  run forever. Offline detection stays with the model and visual-search calls, which run before any page. Rows A17·C3(c)
  case 3 and A17·C3(d). (08B changelog 11(d).)

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine adapter_pages::` · `cargo test -p psm-testkit --test suite research::r8_shop_facts::` · `cargo test -p psm-testkit --test suite p12_shopify_adapter::` · `cargo test -p psm-testkit --test suite research::r7_wiring::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

(Row A17·C7(a) is a research-crate test; put it in `adapter_pages.rs` so the first command runs it.)

## Closing gate (once, after the last unit; replaces the template's closing gate)
U1 and U3 changed app files (`crates/integrations/src/lib.rs`, the Shopify adapter), so the full gate runs once: `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test` · `cargo test -p psm-research`. One cargo command at a time.

## Review mode
The reviewer runs: `cargo test -p psm-research --test engine adapter_openai::`, `a15_c`, `adapter_search::`, `adapter_pages::`, `cargo test -p psm-research --lib rules::calls`, `cargo test -p psm-testkit --test suite research::r8_shop_facts::`, `research::r7_wiring::`, `p12_shopify_adapter::`, `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`, workspace clippy; the implementer's full-suite stamp stands. Re-runs no real call. Checks every provider-owned fixture cell against the fixtures README, that every **derived** shape is built in test code from the capture its row names, and that nothing under `crates/research/fixtures/` was edited. Reads the address guard (U3 task 2). Notes: `docs/implementation/backlog_research/R8_research_real_adapters.md`.

## Manual try-out (the narrative is built from this)
On scratch data, shop connected, R7 in place:
1. Settings → Research: paste the model key and the visual-search key; add your nine sites in your order; choose a model per stage (defaults arrive in R10) and enter its prices, including the cache-write price.
2. Create a session and shoot one photo. Open the form and the research panel: steps appear (looked, searched, read, compared, concluded).
3. When it says Finished, check a suggested designer: its popover shows the confidence, the reason and the links it relied on.
4. Check a piece already in your shop: its own listing is listed as "this item's own listing" and not used.
5. A piece whose search finds an Etsy page: that page shows as "could not be read" and the run still finishes.

## Review log
