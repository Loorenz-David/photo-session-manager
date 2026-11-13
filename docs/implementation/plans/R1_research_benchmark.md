---
phase: R1_research_benchmark
title: Research benchmark — the provider questions are answered with real calls, both visual-search routes are measured on the owner's 42 pieces, the nine trusted sites are read, and three models are compared stage by stage, so the owner can pick the search route, the page-reading route and a first model per stage
difficulty: high
projection: no
depends_on: []
blocked_on: the owner's keys (master_plan_research.md §7.6)
---
# R1 — Research benchmark: probes, visual-search comparison, first model comparison

**Tracker:** IMPLEMENTED · 2026-11-12 · opus · U1 `60f6d6c`, U2 `7b3a2f8`; 14/14 tool tests, clippy, fmt, workspace check green; Lens measured on 42 pieces, Cloud Vision blocked (billing disabled, owner card); 683 model calls USD 5.82 of 50.

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
Recorded evidence and two reports from which the owner picks the visual-search adapter, the page-reading route and a first model per stage; sanitized provider captures that R8 uses as fixtures; and the tool crate `crates/research-tool` (bin `psm-research`) that R9 grows into the dev-Mac report and replay tool. Real calls with real keys: never part of any automated suite (charter rule 9). **Spending ceiling USD 50** (U1 15, U2 35). **Not here:** any app code, the research crate's engine, the end-to-end measurement (R9).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 2 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R1_research_benchmark.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Provider probes and visual-search benchmark | 8 | 13 | no |
| U2 | First per-stage model comparison | 5 | 9 | no |
| **Total** | | **13** | **22** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`.

## Unit 1 — Provider probes and visual-search benchmark

**Unit id:** `R1·U1` · **Replaces:** A01 (all rows) · **Changes app files:** no

### Goal
Answer, with real calls and recorded evidence, everything the research contracts leave to a probe, and measure the
two visual-search candidates on the owner's test pieces. Product: (1) a small Rust command-line tool, `psm-research`
(crate `crates/research-tool`), that makes the calls, records every exchange and enforces a spending ceiling;
(2) `docs/intention/evidence/research_providers.md` extended with the probe answers and the benchmark tables;
(3) sanitized captures in `crates/research/fixtures/` that later phases use as fixtures;
(4) a report from which the owner picks the visual-search adapter and the page-reading route.
**Not in this phase:** any app code (`crates/core`, `crates/infra`, `apps/`), any port or adapter of the app, the
per-stage model comparison (R1·U2), the end-to-end measurement (R9·U2). This is a spike with real keys: its real calls
are never part of the automated suite (charter rule 9).

### What changed from the superseded plan (this wins over any older wording below)
- Fixtures are written to `crates/research/fixtures/<provider>/` (the research crate owns them), not under `crates/integrations`. If `crates/research` does not exist yet (R2 not run), create only the `fixtures/` folder; R2 builds the crate around it.
- The tool's tests live **in the tool crate** (`crates/research-tool/tests/tool/main.rs`, one test program, `autotests = false`), not in the testkit suite. The testkit does not depend on the tool.
- The tracker is this plan's Tracker line.

### Precondition
- `~/.psm_research.env` exists (mode 0600) and names `OPENAI_API_KEY`, `SERPAPI_API_KEY`, `GOOGLE_VISION_API_KEY`
  (master_plan_research.md §7.6). `~/.psm_shopify.env` exists (`master_plan.md` §3) for the one read-only shop query.
- The session loads them with `set -a; . ~/.psm_research.env; . ~/.psm_shopify.env; set +a` and never copies a value
  anywhere. If a name is missing the session reports `STATE: BLOCKED` at task 0 and does nothing else.
- The implementer's sandbox must reach the network. If it cannot, the orchestrator relaunches with
  `PSM_IMPLEMENTER=claude`.
- **Spending ceiling: USD 15 for this phase** (model calls; the two search vendors stay inside their free monthly
  allowances by the call caps below). Hard caps enforced by the tool: SerpApi ≤ 130 searches and ≤ 130 uploads,
  Cloud Vision ≤ 130 units, OpenAI ≤ 80 calls.

### The tool
- **Recording layout = the export layout of MC-A11**, so the same reader serves `report` and `replay` in R9·U1:
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
  of the tool here; R9·U1 replaces them with `psm_research::rules::{sites, cost}`.

### Scoring definitions
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

### Read first
`docs/process/flow.md` · `master_plan.md` §3 (secrets), §6.1, §6.2, §7.2 · `master_plan_research.md` (all) · book `01`,
`14` §Research agent, `08A` §4, §5, §6, §10A, §12A, `08B` MC-A3, MC-A8, MC-A9, MC-A11 and **§Grounding owed** ·
`docs/intention/evidence/research_providers.md` (all; its cited documentation pages are the source for request
shapes) · `docs/test-items/beyo_test_samples_1.json`, `docs/test-items/answer_key_corrections.json` ·
`crates/integrations/src/commerce/shopify/transport.rs` (the HTTP pattern: private runtime thread, no URL in errors),
`crates/integrations/examples/shopify_probe.rs` and `docs/intention/evidence/shopify_admin.md` (how P12 recorded a probe).

### Files
New: `crates/research-tool/{Cargo.toml, src/main.rs, src/lib.rs, src/{budget, recording, answer_key, http, openai, serpapi, cloud_vision, pages, shop, probes, bench_search, report}.rs, tests/tool/{main.rs, u1_probes_and_search.rs}}` · `crates/research/fixtures/{openai,serpapi_lens,cloud_vision,pages,shop}/*.json` and `crates/research/fixtures/README.md`.
Edited: workspace `Cargo.toml` (`members` gains `crates/research-tool`), `Cargo.lock`, `docs/intention/evidence/research_providers.md`, this plan's Tracker line. Outside the repo, never committed: `~/psm-research-bench/a01/`.

### Core mechanisms (the only functions with unit tests)
None in `psm-core`. The tool's `budget` and `answer_key` modules are tested through the three automated criteria.

### Ordered tasks
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

### Criteria (8 criteria, 13 rows, 3 `[rule]`, 5 `[manual]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A01·C1(a) | `answer_key` scoring | a piece whose key designer is `Hans J. Wegner`; result titles `Wegner chair`, `HANS J WEGNER - CH23` | first not naming, second naming | `[rule]` 08A §12A |
| A01·C1(b) | own listing | handle `oak-table`; URLs `https://<shop>/en/products/oak-table`, `https://<shop>/products/oak-table?x=1`, `https://<shop>/products/oak-table-2`, `https://other.example/products/oak-table` | first two own, last two not | MC-A3 |
| A01·C1(c) | corrected-key bucket | the real corrections file and test file | every correction is scored in the corrected bucket and none in the owner-verified bucket; the count equals the file's `corrections` length | `[rule]` corrections file |
| A01·C2(a) | `budget` | ceiling 1.00, spent 0.97, next call's worst case 0.04 | refused; no request leaves (the HTTP seam stub received nothing) | MA5 |
| A01·C2(b) | `budget` | a provider's call cap reached | refused, naming the cap | MA5 |
| A01·C2(c) | command line | `bench-search` without `--ceiling-usd` | exits non-zero before any request | MA5 |
| A01·C3(a) | `recording` | an exchange whose URL query, a header and the response body each contain the key value, once plain and once URL-encoded | nothing containing the value exists under the out folder; declared parts read `<redacted>`; the command reports the call id | MA16 |
| A01·C3(b) | `recording` | a request with two images | the call file holds `{"image_sha256"}` twice and `images/` holds the two files named by their SHA-256 | MA9 |
| A01·C4(a) | evidence doc `[manual]` | probes V1–V10 and evidence probe 4 | one row each: question, method, answer, *observed* with its capture file or *documented only* with the page and reason | RESEARCH-VERIFY-1…10 |
| A01·C5(a) | evidence doc `[manual]` | every piece of the test file, both routes | the benchmark tables of §Scoring definitions, the per-piece table (first naming rank and site per route), calls and cost | `[rule]` 08A §12A.1 |
| A01·C6(a) | evidence doc `[manual]` | the nine trusted sites and the shop, both routes | one row per site and route: readable or not, status, final URL, text present, images found | RESEARCH-VERIFY-8 |
| A01·C7(a) | fixtures folder `[manual]` | the captures | at least: `openai/` (ok reply with image and strict schema, usage per candidate model, incomplete reply, invalid-key error, web-search reply), `serpapi_lens/` (upload reply, search reply cut to 3 results, an error reply), `cloud_vision/` (web detection reply cut to 3 results, an error reply), `pages/` (the shop's own listing page in full; for every other site only status, headers, final URL and sizes — **no third-party page body**), `shop/` (hosts and product URL reply); the README lists each with probe id and date; a scan for the three keys and the shop secret finds nothing | RESEARCH-VERIFY-1…9 (charter rule 17) |
| A01·C8(a) | Review log `[manual]` | the run | calls per provider and model cost in USD, each within its cap and the ceiling | HA6 |

### Delegated choices
Tool module split and command-line flags beyond the names above · the candidate model ids (recorded) · how page
text is extracted for scoring (plain tag stripping is enough) · the exact table layout in the evidence doc · crate
choices for the tool (`reqwest` as pinned; a command-line parser of the implementer's choice, version recorded).
Not delegated: the ceiling, the caps, the politeness rule, what may be committed.

### Notes
- The reviewer writes the two picks as decision cards (charter format) in the narrative; the legal position of the
  Lens-through-a-vendor route (evidence doc, Google v. SerpApi) is part of that card's story, in plain words.
- The owner's answers are recorded by the orchestrator in the evidence doc under **Owner picks**; R8·U2 and R8·U3 read
  them there.
- A probe answer that contradicts 08B (for example, an incomplete reply that is billed but reports no usage) is
  reported in the handoff as a finding against the contract. Do not adapt the contract here.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research-tool u1_probes_and_search::` · `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 2 — First per-stage model comparison

**Unit id:** `R1·U2` · **Replaces:** A02 (all rows) · **Changes app files:** no

### Goal
Compare the candidate models, stage by stage, on the three stages for which the test set holds the answer:
**observe** (visible fields from the cover photo), **read** (what a listing page states), **condition** (severity
fields from the condition photos). Record accuracy, cost and time per piece and model in the evidence doc. The
owner makes a first pick of model and effort per stage from it; R9·U2 confirms it on the finished engine.
**Not in this phase:** compare (no labelled match set exists; measured in R9·U2), labels (the key does not say which
pieces carry a label), decide (confidence is computed by code, the model only writes the reason); any app code.

### What changed from the superseded plan (this wins over any older wording below)
- Fixtures go to `crates/research/fixtures/openai/`; the tests go to `crates/research-tool/tests/tool/u2_stage_models.rs`.

### Precondition
As R1·U1, with `OPENAI_API_KEY` only. `~/psm-research-bench/a01/` (R1·U1's recordings: reduced copies, fetched pages) is
present on this Mac; otherwise `STATE: BLOCKED`. **Spending ceiling: USD 35**, call cap 700.

### Read first
`master_plan_research.md` · R1·U1's plan (the tool, recording layout, budget) and its Review log · evidence doc
(R1·U1's sections: usage fields, strict schema, image limits, candidate models) · book `08A` §5 (stages), §6, §9,
`08B` MC-A2 (what a listing states, choice mapping), MC-A4 (condition rule) · the test file and the corrections
file · `crates/integrations/src/commerce/shopify/fixtures/sv1_definitions_p1.json` (the definitions and their
choices for the scored keys).

### Files
`crates/research-tool/src/{bench_models.rs, instructions/…, score.rs, main.rs, lib.rs, report.rs}`, `crates/research-tool/tests/tool/{main.rs, u2_stage_models.rs}`, `crates/research/fixtures/openai/` (one strict-schema reply per scored stage; README updated), `docs/intention/evidence/research_providers.md` (section "Benchmark 2").

### Core mechanisms (the only functions with unit tests)
None in `psm-core`. `score` is tested through C1.

### Ordered tasks
1. `score` and its tests (C1), and the resume rule (C2), before any real call.
2. Draft instructions and strict output schemas, version 0, for the three stages (`instructions/`). They are the
   starting point R8·U1 takes over; keep each to what the stage's output in 08A §5 needs. Observe returns the visible
   fields with a `sure` flag and `shows_marking`; read returns, per researched key, `{stated, value}`; condition
   returns one severity per condition field with a `sure` flag. The definitions' own choice strings are given to
   the model (MC-A2).
3. Matrix (`psm-research bench-models`): three candidate models at the provider's default effort on all three
   stages; then the best model per stage at one lower and one higher effort. Inputs: observe — the cover copy of
   every piece; read — for every piece with attribution, up to three pages R1·U1 fetched (the first naming result,
   and the first two non-naming results on trusted sites or the shop); condition — every piece with condition
   images, at most six images, long edge 2048.
4. Score against the key; write "Benchmark 2" (per stage and model: correct / wrong / not determined, schema
   failures, refusals, USD per piece, seconds per piece; corrected-key bucket apart; every disagreement listed).
5. Copy one sanitized reply per stage into the fixtures folder; closing gate once.

### Criteria (5 criteria, 9 rows, 2 `[rule]`, 3 `[manual]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A02·C1(a) | `score` | choice key, key value `Teak`, answers `Teak`, `teak`, `Oak`, none | correct, correct (mapped by name key), wrong, not determined | MC-A2 |
| A02·C1(b) | `score` | multi-choice key, key `["Teak","Oak"]`, answers `["Oak","Teak"]`, `["Teak"]` | correct, wrong | MC-A2 |
| A02·C1(c) | `score` | text key, key `Hans J. Wegner`, answers `hans j wegner`, `Hans Wegner` | correct, wrong | MC-A2 |
| A02·C1(d) | `score` | a field listed in the corrections file | scored against the corrected value in the corrected bucket; absent from the owner-verified bucket | `[rule]` corrections file |
| A02·C1(e) | `score` | read stage, a page that does not name the key, answers none / a name | correct / wrong | MC-A2 |
| A02·C2(a) | `bench-models` resume | a run stopped after 3 of 5 calls (HTTP seam stub), started again | 2 requests leave; 5 recordings exist; spend counted once | MA5 |
| A02·C3(a) | evidence doc `[manual]` | the matrix | the tables of task 4 for every stage, model and effort run | `[rule]` 08A §9 |
| A02·C4(a) | fixtures `[manual]` | one reply per stage | sanitized, listed in the README with date and model; secret scan clean | charter rule 17 |
| A02·C5(a) | Review log `[manual]` | the run | calls and USD, within the cap and the ceiling | HA6 |

### Delegated choices
The wording of the version-0 instructions · which two extra effort levels · table layout. Not delegated: the
ceiling, the inputs per stage, the scoring rules.

### Notes
- The owner's answer is recorded by the orchestrator in the evidence doc under **Owner picks**. It sets nothing in
  the app yet; R10·U1 writes the defaults after R9·U2.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research-tool u2_stage_models::` · `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test -p psm-research-tool` · `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`. **No app suite**: the only file outside the tool crate and `crates/research/fixtures/` is the workspace `Cargo.toml` (one `members` entry) and `Cargo.lock`.

## Review mode
The reviewer runs `cargo test -p psm-research-tool`, `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`; re-runs **no** real call; checks the evidence doc against the recordings in `~/psm-research-bench/` on this Mac; scans `crates/research/fixtures/` for the key values and for third-party page bodies. The narrative's **Needs you** carries three picks as decision cards with the measured numbers: visual-search route (with the legal position of the Lens-through-a-vendor route in plain words), page-reading route, model and effort per stage.

## Manual try-out (the narrative is built from this)
1. Read "Benchmark 1" in `docs/intention/evidence/research_providers.md`: for each search route, how many of your pieces had a correct listing in the first ten results, published and unpublished apart.
2. Open `~/psm-research-bench/a01/report.md`, go to the check sheet (12 pieces) and mark for each route whether the right listing really is among the first ten.
3. Read "Page reading": which of your nine sites can be read, and by which route.
4. Read "Benchmark 2": per stage, which model answered correctly how often, and what a piece costs.
5. Answer the three picks in **Needs you**.

## Review log

### 2026-11-12 · implement r1 · opus
**Built.** The tool crate `crates/research-tool` (package `psm-research-tool`, bin `psm-research`):
- `recording`: one path per request, in this order: resume (same request hash → recorded call
  returned, nothing sent), budget, send, record. Layout MC-A11; fields MC-A9; declared parts
  `<redacted>`; fail-closed scan for every key value, plain and URL-encoded, naming the call id.
- `budget`: integer micro-USD, prices from `prices.json`, ceiling and per-provider call caps.
- `answer_key` (naming, own listing, buckets) and `score` (MC-A2 mapping by core `name_key`).
- Provider modules: `openai`, `serpapi`, `cloud_vision`, `pages`, `shop`.
- Commands: `probes` (V1–V10, ep4), `bench_search` + `report` (Benchmark 1, check sheet),
  `bench_models` (matrix, resume, report), `fixtures` (sanitized export), `recost`.
- Version-0 instructions in `src/instructions/`.

Results are in the evidence doc ("R1 results", "Benchmark 1", "Page reading", "Findings against 08B",
"Benchmark 2"). Captures are in `crates/research/fixtures/` (32 files + README). Checkpoints: U1
`60f6d6c`, U2 `7b3a2f8`. Outside the tool crate and the fixtures folder only the workspace
`Cargo.toml` (one member), `Cargo.lock` and the evidence doc changed.

**Spend (A01·C8, A02·C5).**
- U1: OpenAI 38 calls (cap 80), USD 0.243 (ceiling 15; includes a USD 0.018 worst-case charge for
  the V10 timeout). SerpApi 45 searches (cap 130) and 44 uploads (cap 130), free plan (45 of 250
  monthly searches by the tool's count; the account figure lags; uploads cost no credit). Cloud Vision 2 calls (cap 130), both 403 and
  unbilled. Shop: 2 read-only calls plus 1 token call that was redone. Plain page fetches: 375
  records (21 superseded after 429/403), at most one request per 2 s per host.
- U2: OpenAI 645 calls (cap 700), USD 5.580 (ceiling 35; includes worst-case charges for 3 calls
  that failed after send).
- Total model spend USD 5.82 of the milestone's 50.

**Judgment calls** (all 15 with reasons in the handoff §Assumptions). The ones to look at first:
- Candidate models `gpt-6.1-sol` / `gpt-6.1-terra` / `gpt-6-luna` (Astra left out on price).
- Cache-write tokens priced at the 30-min write rate instead of input.
- Hedged piece 26 scored in a third bucket `FromText`.
- A recorded 401/403/429/5xx is retried on a re-run (moved to `superseded/`).
- U2 scored keys: observe `wood_type`; read `designer`/`manufacturer`; condition the piece's
  `condition_*` keys.
- Best model per stage by (correct − wrong).

**Deviations** (handoff §Deviations):
- The Cloud Vision route and its success fixture are missing: the key's project has billing
  disabled (owner card).
- V9 ran on the test store (no product fields observed).
- V8 pages were chosen from the benchmark's results.
- Row A01·C3(a) is tested with three exchanges.
- U2 choices come from `claude_design/reference/SHOPIFY_PRODUCT_METAFIELDS.md`, because
  `sv1_definitions_p1.json` is the test store's.
- One V9 token recording was deleted by hand so the probe could re-run (the token was already
  redacted).
- U2 modules were drafted during U1's wait; the version-0 instruction files landed in U1's
  checkpoint.
- Tracker actor `opus` (this session), not `codex`.

**Observations for the reviewer and planners.**
1. Findings against 08B, reported and not adapted: MC-A8 has no cache-write unit; the web-search
   count is `tool_usage.web_search.num_requests`; shop other domains need `read_markets`; the
   model page tool returns no page text.
2. Shopify storefronts (the owner's shop, midcenturymobler, vgliving) throttle plain fetches from
   one client after a short burst (429, `Retry-After: 60`, not lifted by waiting).
3. pamono's country sites (`pamono.se/.eu/.co.uk`) are not covered by the trusted entry
   `pamono.com`.
4. Row A02·C1(e)'s rule counts a correct reading of *another* piece's designer as wrong. The
   evidence doc splits the read results by page kind; a candidate refinement for R9·U2's scoring.
5. Piece 30's corrected `wood_type` "Rosewood" is not a shop choice ("Santos Rosewood" is).
6. The invalid-key reply echoes the key masked to its last 4 characters.
7. No real call is in any suite: the 14 tests use a stub seam.

