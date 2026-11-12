---
phase: A01_research_probes_and_search_benchmark
title: Research benchmark, part 1 — the provider probes are answered with real calls, both visual-search routes are measured on the owner's 42 pieces and the nine trusted sites are read, so the owner can pick the visual-search adapter and the page-reading route
difficulty: high
projection: no
depends_on: []
blocked_on: the owner's keys (master_plan_research.md §7.6)
---
# A01 — Provider probes and visual-search benchmark (08A §12A; RESEARCH-VERIFY-1…10)

## Goal
Answer, with real calls and recorded evidence, everything the research contracts leave to a probe, and measure the
two visual-search candidates on the owner's test pieces. Product: (1) a small Rust command-line tool, `psm-research`
(crate `crates/research-tool`), that makes the calls, records every exchange and enforces a spending ceiling;
(2) `docs/intention/evidence/research_providers.md` extended with the probe answers and the benchmark tables;
(3) sanitized captures in `crates/integrations/src/research/fixtures/` that later phases use as fixtures;
(4) a report from which the owner picks the visual-search adapter and the page-reading route.
**Not in this phase:** any app code (`crates/core`, `crates/infra`, `apps/`), any port or adapter of the app, the
per-stage model comparison (A02), the end-to-end measurement (A23). This is a spike with real keys: its real calls
are never part of the automated suite (charter rule 9).

## Precondition (the orchestrator checks before launch; otherwise do not launch)
- `~/.psm_research.env` exists (mode 0600) and names `OPENAI_API_KEY`, `SERPAPI_API_KEY`, `GOOGLE_VISION_API_KEY`
  (master_plan_research.md §7.6). `~/.psm_shopify.env` exists (`master_plan.md` §3) for the one read-only shop query.
- The session loads them with `set -a; . ~/.psm_research.env; . ~/.psm_shopify.env; set +a` and never copies a value
  anywhere. If a name is missing the session reports `STATE: BLOCKED` at task 0 and does nothing else.
- The implementer's sandbox must reach the network. If it cannot, the orchestrator relaunches with
  `PSM_IMPLEMENTER=claude`.
- **Spending ceiling: USD 15 for this phase** (model calls; the two search vendors stay inside their free monthly
  allowances by the call caps below). Hard caps enforced by the tool: SerpApi ≤ 130 searches and ≤ 130 uploads,
  Cloud Vision ≤ 130 units, OpenAI ≤ 80 calls.

## Read first
`docs/process/flow.md` · `master_plan.md` §3 (secrets), §6.1, §6.2, §7.2 · `master_plan_research.md` (all) · book `01`,
`14` §Research agent, `08A` §4, §5, §6, §10A, §12A, `08B` MC-A3, MC-A8, MC-A9, MC-A11 and **§Grounding owed** ·
`docs/intention/evidence/research_providers.md` (all; its cited documentation pages are the source for request
shapes) · `docs/test-items/beyo_test_samples_1.json`, `docs/test-items/answer_key_corrections.json` ·
`crates/integrations/src/commerce/shopify/transport.rs` (the HTTP pattern: private runtime thread, no URL in errors),
`crates/integrations/examples/shopify_probe.rs` and `docs/intention/evidence/shopify_admin.md` (how P12 recorded a probe).

## Dependencies and gate
Gate: 08A reads RATIFIED and 08B CONTRACTS RATIFIED. **This phase's tracker row and gate are in
`docs/implementation/master_plan_research.md` §4** (not `master_plan.md`). Reviewer notes go to
`docs/implementation/backlog_research.md`.

## Files expected to change (write perimeter)
New: `crates/research-tool/{Cargo.toml, src/main.rs, src/lib.rs, src/…}` (modules: `budget`, `recording`,
`answer_key`, `http`, `openai`, `serpapi`, `cloud_vision`, `pages`, `shop`, `probes`, `bench_search`, `report`) ·
`crates/integrations/src/research/fixtures/{openai,serpapi_lens,cloud_vision,pages,shop}/*.json` plus a `README.md`
there naming each file, its probe id and its capture date · `crates/testkit/tests/suite/research/{mod.rs,
a01_research_probes_and_search_benchmark.rs}` (create `mod.rs` and the `mod research;` line in
`crates/testkit/tests/suite/main.rs` if absent).
Edited: workspace `Cargo.toml` (`members` gains `crates/research-tool`), `crates/testkit/Cargo.toml` (dependency on
`psm-research-tool`), `docs/intention/evidence/research_providers.md` (new sections, below),
`master_plan_research.md` §4 (own row) and §6.2 (pins recorded).
Outside the repo, never committed: the recording folder `~/psm-research-bench/a01/`.

## Core mechanisms (unit tests allowed)
None in `psm-core`. The tool's `budget` and `answer_key` modules are tested through the three automated criteria.

## The tool (what later phases rely on)
- **Recording layout = the export layout of MC-A11**, so the same reader serves `report` and `replay` in A22:
  `<out>/<piece n>-<handle>/run-bench/calls/<call id>.json` and `images/<sha256>.jpg`; probes under
  `<out>/probes/<probe id>/calls/…`. A call file has the MC-A9 record fields (provider, model, effort, request with
  method, URL, non-secret headers and body, each image replaced by `{"image_sha256": …}`; response status, headers
  and body; usage; cost in micro-USD; started, finished, duration; error).
- **Secrets.** Declared secret-bearing parts (headers `authorization`, `x-api-key`, `x-goog-api-key`; query
  parameters `key`, `api_key`) are recorded as `"<redacted>"`. Before any file is written, its text is scanned for
  every key value from the environment, plain and URL-encoded; a hit stops the write and the command exits non-zero
  naming the call id, never the value. Error text is built with `reqwest::Error::without_url()`.
- **Budget.** `--ceiling-usd <n>` is required. Model cost per call = reported usage × `prices.json` (a file in the
  out folder that the session fills from the provider's pricing page on the day, with the page URL and date; never
  from memory). Before each model call the tool refuses if `spent + worst case of this call ≥ ceiling` (worst case
  = the call's `max_output_tokens` and its input size at the listed prices). Per-provider call caps as in the
  precondition. A re-run skips calls already recorded (same request hash).
- **Images.** The pieces' image URLs are the owner's own shop files. The tool downloads each once and makes the
  reduced copy with the existing `psm_infra` thumbnail function (long edge 1024; 2048 where a probe asks) — no new
  pixel code (H3). If SerpApi's upload limit needs a smaller file, lower the long edge and record it.
- **Temporary helpers.** Host matching against the trusted list and the micro-USD sum are small private functions
  of the tool here; A22 replaces them with `psm_core::mechanisms::research::{sites, cost}`.

## Ordered tasks
0. Check the precondition; create the out folder; write `prices.json` from the pricing pages; record the tool's
   candidate model ids (the provider's current vision-capable models: flagship, mid, small) in the evidence doc.
1. Build the tool skeleton with `budget`, `recording`, `answer_key` and their tests (criteria C1–C3) **before any
   real call**.
2. **Probes** (`psm-research probe <id>`), each recorded and answered in the evidence doc as *observed* (capture
   file named) or *documented only* (page URL, and why it could not be provoked):
   - V1 usage: one image + strict-schema call per candidate model; one call with the built-in web search tool; one
     `incomplete` reply (tiny `max_output_tokens`) — which usage fields exist, how tool calls are counted, whether an
     incomplete reply reports usage.
   - V2 errors: a deliberately invalid key string (never the real one), an oversized input; rate limit, no quota and
     server error from the documentation unless seen; the `Retry-After` header.
   - V3 strict schema: the refusal item's shape (documented, or observed if one occurs), `status: incomplete` and its
     reason (observed).
   - V4 image input: size, count and format limits (documented), the `detail` setting and the token count observed
     for a 1024 and a 2048 copy.
   - V5 an image and web search in one request; `allowed_domains` with a subdomain of a trusted site.
   - V6 result shape and rank of both visual-search routes; the usage units each reports.
   - V7 SerpApi upload: where the key goes, credits used per upload and per search (account figures before and
     after), accepted formats and size, whether an `image_id` still works after 11 minutes.
   - V8 page reading, for one listing page on each of the nine trusted sites and on the owner's shop, by both routes:
     (a) a plain HTTP GET from the tool (status, redirects, final URL, whether the listing's title and attribution
     text are in the returned HTML or only after scripts run, which main images can be found); (b) the model
     provider's page-open tool restricted to that domain (is page text returned, is the final URL returned). One
     request per page per route, at most one request every 2 s per host, a generic client name.
   - V9 the shop: public hosts (primary, other, platform domain) and a product's storefront URL and handle, by one
     read-only Admin query against the test store.
   - V10 a call cut by a 1 s client timeout: whether it is billed, if observable; otherwise record "not observable;
     `fallback_call_usd` stands".
   - Evidence probe 4: the Cloud Vision price (page read); TinEye recorded as not pursued.
   RESEARCH-VERIFY-11 is already grounded (master_plan_research.md §7.4); cite it, do not redo it.
3. **Visual-search benchmark** (`psm-research bench-search`): for every piece of the test file, send the reduced
   copy of its first image to both routes; record. Among the first 10 results of each route, fetch by plain HTTP,
   once each, the pages on the shop or on a trusted site and the first three other pages (cached by URL across
   routes and pieces, same politeness as V8). Other results are scored by their title only.
4. **Score** (definitions below), write `<out>/report.md` and the evidence sections.
5. Copy sanitized captures into the fixtures folder (list in C7); run the closing gate once.

## Scoring definitions (the planner's reading of "is the right listing in the first 10 results", 08A §12A)
- **Own listing:** a result on the shop's host (or a subdomain) whose path ends in `/products/<the piece's handle>`,
  with or without a locale prefix. It is removed before ranking and counted separately.
- **Naming result:** a result whose title, or whose fetched page text, contains the piece's key designer or
  manufacturer, compared case-insensitively with punctuation ignored. The key is the test file's metafields; the six
  entries of `answer_key_corrections.json` are scored in their own "corrected key" bucket; for the hedged pieces the
  hedged name is the key.
- **Per route, reported for four groups** (published / unpublished × with attribution / without; group sizes
  computed from the file, not typed): pieces with a naming result in the first 10; the same counting only results on
  the shop or a trusted site; the rank of the first naming result; results the route marks as exact copies of the
  photo, reported with and without them (a published piece's own photos may exist elsewhere; the unpublished group
  is the clean measure); cost and time per search. For pieces without attribution: the first 10 results listed for a
  human look, no score.
- **A naming result is not proof of the right listing** (it can be another piece by the same designer), so the
  count over-credits both routes alike. The report therefore carries a **check sheet**: for 12 pieces (4 per test
  group, chosen by lowest piece number) the first 10 results of each route with links, and an empty column for the
  owner's own "right listing: yes / no". The owner's pick rests on the numbers and on that sheet.
- Every disagreement between a route and the key is listed with its evidence (`answer_key_corrections.json`
  `scoring_rules.disagreements`).

## Criteria (8 criteria, 13 rows, 3 `[rule]`; rows C4–C8 are `[manual]`: recorded, not asserted by the suite)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `answer_key` scoring | a piece whose key designer is `Hans J. Wegner`; result titles `Wegner chair`, `HANS J WEGNER - CH23` | first not naming, second naming | `[rule]` 08A §12A |
| C1(b) | own listing | handle `oak-table`; URLs `https://<shop>/en/products/oak-table`, `https://<shop>/products/oak-table?x=1`, `https://<shop>/products/oak-table-2`, `https://other.example/products/oak-table` | first two own, last two not | MC-A3 |
| C1(c) | corrected-key bucket | the real corrections file and test file | every correction is scored in the corrected bucket and none in the owner-verified bucket; the count equals the file's `corrections` length | `[rule]` corrections file |
| C2(a) | `budget` | ceiling 1.00, spent 0.97, next call's worst case 0.04 | refused; no request leaves (the HTTP seam stub received nothing) | MA5 |
| C2(b) | `budget` | a provider's call cap reached | refused, naming the cap | MA5 |
| C2(c) | command line | `bench-search` without `--ceiling-usd` | exits non-zero before any request | MA5 |
| C3(a) | `recording` | an exchange whose URL query, a header and the response body each contain the key value, once plain and once URL-encoded | nothing containing the value exists under the out folder; declared parts read `<redacted>`; the command reports the call id | MA16 |
| C3(b) | `recording` | a request with two images | the call file holds `{"image_sha256"}` twice and `images/` holds the two files named by their SHA-256 | MA9 |
| C4(a) | evidence doc `[manual]` | probes V1–V10 and evidence probe 4 | one row each: question, method, answer, *observed* with its capture file or *documented only* with the page and reason | RESEARCH-VERIFY-1…10 |
| C5(a) | evidence doc `[manual]` | every piece of the test file, both routes | the benchmark tables of §Scoring definitions, the per-piece table (first naming rank and site per route), calls and cost | `[rule]` 08A §12A.1 |
| C6(a) | evidence doc `[manual]` | the nine trusted sites and the shop, both routes | one row per site and route: readable or not, status, final URL, text present, images found | RESEARCH-VERIFY-8 |
| C7(a) | fixtures folder `[manual]` | the captures | at least: `openai/` (ok reply with image and strict schema, usage per candidate model, incomplete reply, invalid-key error, web-search reply), `serpapi_lens/` (upload reply, search reply cut to 3 results, an error reply), `cloud_vision/` (web detection reply cut to 3 results, an error reply), `pages/` (the shop's own listing page in full; for every other site only status, headers, final URL and sizes — **no third-party page body**), `shop/` (hosts and product URL reply); the README lists each with probe id and date; a scan for the three keys and the shop secret finds nothing | RESEARCH-VERIFY-1…9 (charter rule 17) |
| C8(a) | Review log `[manual]` | the run | calls per provider and model cost in USD, each within its cap and the ceiling | HA6 |


## Delegated choices
Tool module split and command-line flags beyond the names above · the candidate model ids (recorded) · how page
text is extracted for scoring (plain tag stripping is enough) · the exact table layout in the evidence doc · crate
choices for the tool (`reqwest` as pinned; a command-line parser of the implementer's choice, version recorded).
Not delegated: the ceiling, the caps, the politeness rule, what may be committed.

## Manual try-out (the narrative is built from this)
1. Open `docs/intention/evidence/research_providers.md` and read "Benchmark 1": for each of the two search routes,
   how many of your pieces had a correct listing in the first ten results, split into published and unpublished.
2. Open `~/psm-research-bench/a01/report.md`, go to the check sheet (12 pieces) and mark for each route whether
   the right listing really is among the first ten.
3. Read "Page reading": which of your nine sites can be read, and by which route.
4. The narrative's **Needs you** carries two picks, as decision cards with the measured numbers in them: the
   visual-search route (Lens through SerpApi, or Google Cloud Vision) and the page-reading route.

## Notes
- The reviewer writes the two picks as decision cards (charter format) in the narrative; the legal position of the
  Lens-through-a-vendor route (evidence doc, Google v. SerpApi) is part of that card's story, in plain words.
- The owner's answers are recorded by the orchestrator in the evidence doc under **Owner picks**; A16 and A17 read
  them there.
- A probe answer that contradicts 08B (for example, an incomplete reply that is billed but reports no usage) is
  reported in the handoff as a finding against the contract. Do not adapt the contract here.
- Review mode: the reviewer runs this phase's suite module, clippy, fmt; re-runs **no** real call; checks the
  evidence against the recordings on this Mac and scans the fixtures for secrets and third-party page bodies.

## Review log
