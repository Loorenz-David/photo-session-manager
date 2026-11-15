---
plan: R8_research_real_adapters
role: implement
round: 1
state: DONE
date: 2026-11-13
actor: opus (Claude Opus 5.5, pipeline-implementer)
---
# Handoff: R8 research with real providers, implement r1

## What was built
**Research can now call real services instead of stand-ins.** By default the app now uses OpenAI to look at photos and read listings, Google Lens (through SerpApi) to find similar pieces from the cover photo, and a plain download from the Mac to read the listing pages it finds. It also asks the shop for its own web addresses, so a piece's own shop listing is never used as evidence for itself.

- **Model (OpenAI).** Each stage's question goes out exactly as the engine wrote it, with our photos (research copies only) and stored listing images. The answer must fit a strict per-stage format that holds only the keys that stage reads. A model never sends a confidence (photo reads carry only `sure`). The reported usage is turned into the price-table units (net input, cached input, cache write, output, web searches), counting only units above 0. Errors map to the existing classes: bad key → "key rejected", 429 → retry, other 4xx → request. A refusal becomes "not determined". The key is read on every call and is never in the call record.
- **Visual search (Google Lens through SerpApi).** The cover's research copy is uploaded unchanged, then searched by the returned image id (`hl=en`, `country=se`). Matches come back in Lens's order. Each search costs one `image_search_call`. The key is `<redacted>` in the record.
- **Page reading (plain download).** One download per page. The reader follows redirects itself (at most 5) and fetches up to 4 main images (JPEG only). It never reaches local-network addresses, checked twice: in the reader, and in the address lookup. It waits 2 s before each request to a Shopify-hosted site or to a site that said 429, and waits out one `Retry-After`. A page that cannot be read (refused, 404, timeout, a dead domain, too many redirects, no text) is recorded as "could not be read" and the run goes on. The reader never fails or holds a run.
- **Shop facts (Shopify).** The shop's primary domain and its myshopify host, plus the linked product's storefront URL and handle (no `webPresences`, card 1). Imported dimension and weight fields now carry the unit names research converts by (mm, cm, m, in, g, kg, lb).
- **Plumbing.** Default provider ids are `openai`, `serpapi_lens`, `page_http`, and a saved `""` means the default. `adapters::build(settings, secrets, files, wait)` builds the three adapters. The secret scan also finds a key in its JSON-escaped form. `ResearchStatus.wait_ends_at` is tested. `TestApp::with_research_using` is new.

## Tests → criterion rows
All 35 rows have one behaviour test at the boundary the row names, asserting its exact outcome. No core-mechanism unit tests (the plan lists none).

| Row | Test | Assertion shape |
|---|---|---|
| A15·C1(a) | `adapter_openai::a15_c1a_usage_of_the_sol_capture` | exact usage list + `call_cost` 3549 (exact) |
| A15·C1(b) | `adapter_openai::a15_c1b_web_search_calls_are_the_billed_count` | contains the four named units (as the row says) |
| A15·C1(c) | `adapter_openai::a15_c1c_incomplete_reply_is_a_schema_failure_with_usage` | `sent`, `Schema`, exact usage |
| A15·C2(a) | `adapter_openai::a15_c2a_invalid_key` | (401, Status, false), `Key` |
| A15·C2(b) | `adapter_openai::a15_c2b_rate_limit_is_transient` | (429, Status, false), `Transient` |
| A15·C2(c) | `adapter_openai::a15_c2c_context_length_is_a_request_error` | (400, Status), `Request` |
| A15·C3(a) | `adapter_openai::a15_c3a_refusal` | `Refusal`, exact text, exact usage |
| A15·C3(b) | `adapter_openai::a15_c3b_output_missing_a_schema_key_is_a_schema_failure` | (true, 200, Schema); with the key `Ok` and exact output |
| A15·C4(a) | `adapter_openai::a15_c4a_request_form` | exact key set, exact data URL, no `detail`, exact enum-or-null schema, Swedish, 4000 |
| A15·C4(b) | `adapter_openai::a15_c4b_instructions_model_effort_and_schema_per_stage` | per stage: instructions, model, reasoning, exact top-level keys; `""` → no `reasoning` |
| A15·C5(a) | `adapter_openai::a15_c5a_the_key_is_redacted_in_the_exchange` | `<redacted>`, key absent from the Exchange |
| A15·C5(b) | `adapter_openai::a15_c5b_closed_port_is_not_sent` | (false, Connect), no URL in text |
| A15·C5(c) | `adapter_openai::a15_c5c_silent_server_times_out_after_send` | (true, Timeout) |
| A15·C5(d) | `adapter_openai::a15_c5d_escaped_key_is_redacted_in_the_call_record` + `export::a15_c5d_an_escaped_key_in_a_file_stops_the_export` | `<redacted>`, neither form present; export error text exact, no folder left |
| A15·C6(a) | `adapter_openai::a15_c6a_full_run_with_the_openai_adapter` | form filled (designer A / Teak / Minor, Finished), exactly 10 model calls with the exact costs |
| A15·C7(a) | `lifecycle::a15_c7a_wait_ends_at_for_a_run_waiting_for_the_editor` | `Some(now+90 000)`, `Some(now)`, `None` |
| A16·C1(a) | `adapter_search::a16_c1a_candidates_in_the_reply_order` | exact three candidates; search used the upload's `image_id` |
| A16·C1(b) | `adapter_search::a16_c1b_no_visual_matches_is_no_candidate` | empty, `Ok`, exact usage (both forms) |
| A16·C2(a) | `adapter_search::a16_c2a_one_search_credit_for_upload_and_search` | exact usage |
| A16·C3(a) | `adapter_search::a16_c3a_api_key_is_redacted_in_the_exchange` | `<redacted>` in multipart and URL, key absent |
| A16·C4(a) | `adapter_search::a16_c4a_upload_too_large_is_a_request_error` | (400, Status, false), `Request`, one request only |
| A16·C5(a) | `adapter_search::a16_c5a_the_stored_copy_is_uploaded_unchanged` | SHA-256 equal to the stored copy |
| A16·C6(a) | `adapter_search::a16_c6a_full_run_reads_the_lens_results_in_site_order` | exact read order; exactly one outbound row (cover) |
| A17·C1(a) | `adapter_pages::a17_c1a_a_shop_page_is_read_by_download` | obtained, exact title, phrase, image bytes, waits exactly 2 000 before each beyovintage.se image request, none before the page |
| A17·C1(b) | `adapter_pages::a17_c1b_redirects_are_followed_up_to_five_hops` | final URLs exact; 6 hops → not obtained |
| A17·C2(a) | `adapter_pages::a17_c2a_one_page_fetch_per_read` | exact usage |
| A17·C3(a) | `adapter_pages::a17_c3a_a_refused_page_is_could_not_be_read` + `adapter_pages::a17_c3a_full_run_skips_the_refused_page` | `Ok`, not obtained, exact usage; steps: 1 "could not be read" + 2 titled, `Finished` |
| A17·C3(b) | `adapter_pages::a17_c3b_throttled_shop_waits_once_then_reads` | exactly one 60 000 before the 2nd page request, other waits 2 000 before image requests; second case exactly `[(60 000, 1)]` |
| A17·C3(c) | `adapter_pages::a17_c3c_transport_failures_are_could_not_be_read` | four cases: `Ok`, not obtained, exact usage |
| A17·C3(d) | `adapter_pages::a17_c3d_a_dead_listing_domain_does_not_hold_the_run` | "could not be read" for the dead page, others titled, `Finished` |
| A17·C4(a) | `research::r8_shop_facts::a17_c4a_hosts_and_the_items_own_listing` | exact hosts, URL, tail; no `webPresences`; null case |
| A17·C4(b) | `research::r8_shop_facts::a17_c4b_an_unlinked_item_gets_the_hosts_only` | exact hosts, `None`, `""`; query without `product` |
| A17·C5(a) | `research::r8_shop_facts::a17_c5a_a_dimension_converts_by_the_adapters_unit_names` | exactly `{"value": "254", "unit": "MILLIMETERS"}` |
| A17·C6(a) | `research::r8_shop_facts::a17_c6a_the_items_own_listing_is_no_evidence_in_the_wired_app` | own listing never requested and listed as "this item's own listing"; the other page read, site class `shop`, `Finished` |
| A17·C7(a) | `adapter_pages::a17_c7a_build_selects_the_default_adapters` | exact ids in all three cases; snapshot ids exact |

Every test in the new test files appears above (no orphan tests). The red baseline was not captured row by row before the production edits. Each discriminating step was reasoned through instead: for example, without the JSON-escaped form the escaped key passes the scan, and without `unit_names` the conversion fails with "no known name".

## Assumptions
(Delegated choices, the smallest option consistent with 08B; details in the plan's Review log.)
1. HTTP seam request shape `{method, url, headers, body: Empty | Json | Multipart(parts), limit}`. `FakeResearchHttp` matches the first rule added with the same method whose URL prefix fits; `once` rules are used up, `always` rules stay. An unscripted request gets a fixed 404 and is recorded, never a panic.
2. The full-run script is reused through `stages.rs` (`full_script()`, `assert_form_filled()`), so `stages.rs` joined U1's files. The `wait_ends_at` test lives in `lifecycle.rs`.
3. Strict schema: a nullable value is `anyOf: [<value>, {"type": "null"}]`. A choice is an enum of the definition's choices, multi-choice an array of it, integer `integer`, decimal `number`, dimension/weight `{value: number, unit: enum of MC-A2's names}`, anything else a string.
4. The text part of a model request is `request.input` with `language` set, sent as JSON text.
5. SerpApi: only `visual_matches` is used. `hl=en`, `country=se` as captured. Ranks run 1, 2, … over results that have a link. An upload reply with no `image_id` is a `Schema` failure.
6. Page reader: at most 4 image requests, page limit 4 MiB, image limit 3 MiB. Images come in the order `og:image`, JSON-LD `Product.image`, then content images by largest declared width, one per picture. Only JPEG bytes are kept, because the model adapter sends every image as JPEG. Image redirects are not followed. Visible text is cut to 30 000 characters. The HTML parser is html5ever 0.39.0 (tokenizer only, already in the lock file).
7. Address guard: non-public addresses are dropped from a lookup, and a host with none left is refused. Broadcast and 0.0.0.0/8 count as non-public too. The live client also refuses non-public IP-literal URLs.
8. GraphQL: `PublicListing($id)` for a linked item and `ShopHosts` (no variables) for an unlinked one, i.e. two operations. `unit_names` goes on platform types `dimension` and `weight` only. Volume maps to a core dimension but has no neutral unit name, so it gets none.
9. A research copy that cannot be read fails the run as a request error, never an offline hold.
10. `build` takes the wiring's `wait` (task 3 allows it). `base64 = 0.23` is used rather than hand-written encoding.
11. `TestApp::with_research_using` leaves `commerce` and `research` as `None`. A restart reopens the app without the custom commerce adapter.
12. Tracker and handoff actor are written as `opus` (the session's true model, as R7 did), not the template's `codex`.

## Deviations
1. **Lazy HTTP client** (`655d312`). The reqwest blocking client is built at the first request instead of in `new`. reqwest panics when a blocking client is built inside an async context, and a settings save rebuilds the app from an async Tauri command. The closing gate caught this: `psm-desktop` `r2_research_command_tests::f_c1a_research_commands_reach_the_wired_engine` and `p50_tests::c8a_support_and_secret_commands_have_their_wire_shapes`. In the real app this would have crashed a settings save. A client that cannot be built counts as a transient failure (the run then fails), not offline.
2. **Files outside the declared perimeter**, each forced by a planned change:
   - `crates/testkit/tests/suite/p12_shopify_adapter.rs`: c4i and c4j now expect `unit_names` (task 6).
   - `crates/testkit/tests/suite/research/r2_foundation.rs`: f_c3a no longer asserts Shopify's empty `public_listing` default (task 5 replaces it; A17·C4 covers it).
   - `docs/implementation/master_plan_research.md` §6.2: the three dependency pins.
3. **A15·C1(a)/(b).** The captured replies' outputs are R1's version-0 shapes, so the adapter answers `Schema` carrying the reply's usage. The tests assert that failure and its usage.

## Files changed
Declared write perimeter (all committed):
- **U1 `0c6dec4`.** `Cargo.lock`, `crates/research/Cargo.toml`, `crates/research/src/adapters/{mod.rs, http.rs, openai/{mod.rs, adapter.rs, schema.rs, usage.rs, errors.rs}}`, `crates/research/src/{config.rs, fakes.rs, rules/calls.rs}`, `crates/research/tests/engine/{main.rs, adapter_openai.rs, export.rs, lifecycle.rs, stages.rs}`, `crates/integrations/src/lib.rs`.
- **U2 `c577f08`.** `crates/research/src/adapters/{mod.rs, visual_search/{mod.rs, serpapi_lens.rs}}`, `crates/research/tests/engine/{main.rs, adapter_search.rs}`.
- **U3 `d37e69e`.** `Cargo.lock`, `crates/research/Cargo.toml`, `crates/research/src/adapters/{mod.rs, page_reader/{mod.rs, http.rs}}`, `crates/research/tests/engine/{main.rs, adapter_pages.rs}`, `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, type_map.rs, fixtures/r8_public_listing.json, fixtures/r8_public_listing_unpublished.json, fixtures/r8_shop_hosts.json}`, `crates/integrations/src/lib.rs`, `crates/testkit/src/test_app.rs`, `crates/testkit/tests/suite/{p12_shopify_adapter.rs, research/mod.rs, research/r8_shop_facts.rs}`.
- **Gate fixes `655d312`.** `crates/research/src/adapters/{http.rs, mod.rs}`, `crates/research/tests/engine/adapter_openai.rs`, `crates/testkit/tests/suite/research/r2_foundation.rs`.
- **Closing commit.** `docs/implementation/plans/R8_research_real_adapters.md` (Tracker line, Review log), `docs/implementation/master_plan_research.md` (§6.2 pins), this handoff.

Nothing under `crates/research/fixtures/` was edited. No mutation probes (flow §4). No architecture-graph delta (none in this project).

## Checkpoint SHA
In order:
1. U1 `0c6dec4`
2. U2 `c577f08`
3. U3 `d37e69e`
4. Closing-gate fixes `655d312`
5. Closing commit (tracker, review log, handoff): the next `CHECKPOINT (not approved): R8 closing …` commit on `feature/agentic`

## Stamp
Tree: `655d312` (code); docs-only changes after it. One cargo command at a time, all in the foreground.

| Command | Result |
|---|---|
| `cargo test --workspace --exclude psm-testkit --no-fail-fast` | ok: core 82, desktop 24, infra 10, integrations 37, research lib 120 + engine 168 + doc 1, research-tool 14 |
| `cargo test -p psm-testkit --lib --bins` | ok (0 tests) |
| `cargo test -p psm-testkit --test suite --no-fail-fast -- p0 p1 p2` | ok: 455 passed |
| `cargo test -p psm-testkit --test suite --no-fail-fast -- p3 p4 p5 p6 research` | 418 passed, 1 failed (`research::r2_foundation::f_c3a…`, the default R8 replaces). Fixed; that file re-run: 10 passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | clean |
| `npm test` | 75 files, 488 tests passed |
| `cargo test -p psm-research` | covered by the first row (lib 120, engine 168, doc 1) |

Pre-existing failures: none.

The first gate run of `psm-desktop` failed on the eager HTTP client (Deviation 1). After the fix, `cargo test -p psm-desktop --lib` gave 24 passed, and the whole first row was re-run green.

Why the workspace run is split: one command may run at most 10 minutes in this session. The two suite partitions (455 + 419) are exactly the 874 suite tests, each run once.

Environment note: the shared build folder held a stale `psm-core` artifact older than R5's code. One `touch crates/core/src/mechanisms/mod.rs` forced a rebuild; no content change.

## Manual try-out
On scratch data (`cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`), shop connected:
1. Settings → Research: paste the model key and the visual-search key. Add your trusted sites in your order. Choose a model per stage (defaults arrive in R10) and enter its prices, **including a cache-write price** (`cache_write_token`). Until R10 ships a price table, a call that reports cache writes is otherwise charged the fallback.
2. Create a session and shoot one photo. Open the form and the research panel: the steps appear (looked, searched, read, compared, concluded).
3. When it says Finished, check a suggested designer: its popover shows the confidence, the reason and the links it relied on.
4. Research a piece already in your shop: its own listing is listed as "this item's own listing" and not used.
5. Research a piece whose search finds an Etsy page: that page shows as "could not be read" and the run still finishes.

A fresh install first shows "Add the model key". After the key is saved it shows "No price is set for openai <model>" until prices are entered. Both are correct.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
