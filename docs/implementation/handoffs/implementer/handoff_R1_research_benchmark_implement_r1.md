---
plan: R1_research_benchmark
role: implement
round: 1
state: DONE
date: 2026-11-12
actor: opus
---
# R1 research benchmark — implement r1

## What was built

**For the owner, in product terms.** A measuring tool and its results. Every one of your 42 test
pieces was searched by photo through Google Lens (via SerpApi). For 30 of the 32 pieces that have a
designer or maker in your data, a page naming it was among the first ten results, usually the first.
That includes 6 of the 7 pieces whose photos have never been online. The second route, Google Cloud
Vision, could not be measured: Google refuses every call because the key's Google Cloud project has
no billing account (card 1). Ten listing pages (one per trusted site plus your shop) were read two
ways:
- by a plain download from the Mac: 5 of 10 readable;
- by OpenAI's page tool: 8 of 10 readable, but it returns only the model's summary, not the page.

Three OpenAI models were compared on three research steps (what the cover photo shows, what a
listing page states, how worn the piece is from the condition photos), with cost and time per piece.

**Code.** A new crate `crates/research-tool` (package `psm-research-tool`, bin `psm-research`):
- `recording`: the one path every request takes. It checks resume (same request hash means nothing
  is sent), then budget, then sends, then records. Records use the MC-A11 layout with MC-A9 fields;
  declared secret parts read `<redacted>`; a fail-closed scan for every key value, plain and
  URL-encoded, runs before any write.
- `budget`: integer micro-USD, `prices.json`, ceiling and per-provider call caps.
- `answer_key`: naming result, own listing, corrected-key and from-text buckets.
- Provider modules: `http`, `openai`, `serpapi`, `cloud_vision`, `pages`, `shop`.
- Commands: `probes` (V1–V10, ep4), `bench_search`, `report` (Benchmark 1 report and check sheet),
  `fixtures` (sanitized copies), `score` and `bench_models` (U2), plus `recost`.
- Version-0 stage instructions in `src/instructions/`.

Captures are in `crates/research/fixtures/` (README there). The evidence doc gained the R1 sections.

## Tests → criterion rows (Task 0 coverage map)

No red baseline was captured: the tests were written after the production modules they exercise
(same session, before any real call). All tests are in `crates/research-tool/tests/tool/` (one
program, `autotests = false`).

| Row | Test | Assertion shape vs row |
|---|---|---|
| A01·C1(a) | `u1_probes_and_search::a01_c1a_naming_result_ignores_case_and_punctuation` | exact: first not naming, second naming |
| A01·C1(b) | `a01_c1b_own_listing_is_the_shop_path_ending_in_the_handle` | exact: the four URLs, first two own |
| A01·C1(c) | `a01_c1c_every_correction_is_scored_in_the_corrected_bucket_only` | exact: real files; each correction once, Corrected, corrected value; none OwnerVerified; count = corrections length (6) |
| A01·C2(a) | `a01_c2a_budget_refuses_a_call_that_would_reach_the_ceiling_before_sending` | exact: Ceiling refusal (970 000 + 40 000 ≥ 1 000 000), stub received nothing |
| A01·C2(b) | `a01_c2b_budget_refuses_when_a_call_cap_is_reached_naming_the_cap` | exact: Cap{openai, 80}; message names the cap; stub received nothing |
| A01·C2(c) | `a01_c2c_bench_search_without_ceiling_exits_non_zero_before_any_request` | exact: `run` ≠ 0 with the stub untouched; the binary itself exits non-zero naming `--ceiling-usd` |
| A01·C3(a) | `a01_c3a_a_key_value_never_reaches_the_out_folder` | exact, in three exchanges (see Deviations): plain and URL-encoded echoes refused naming the call id (never the value); declared-only exchange written with `key=<redacted>` and header `<redacted>`; no file under the folder holds either form |
| A01·C3(b) | `a01_c3b_two_images_are_recorded_by_sha256_and_stored_once_each` | exact: two `{"image_sha256"}` refs; `images/` holds exactly the two `<sha>.jpg` files |
| A01·C4–C8 | `[manual]` | evidence doc §R1 results, fixtures README, Review log |
| A02·C1(a) | `u2_stage_models::a02_c1a_choice_answers_are_mapped_by_name_key` | exact: correct, correct, wrong, not determined |
| A02·C1(b) | `a02_c1b_multi_choice_is_correct_only_for_the_same_set` | exact: correct, wrong |
| A02·C1(c) | `a02_c1c_text_is_compared_by_name_key_without_aliases` | exact: correct, wrong |
| A02·C1(d) | `a02_c1d_a_corrected_field_is_scored_against_the_corrected_value_only` | exact: piece 39 wood_type scored (Corrected, Correct) for Cherry, (Corrected, Wrong) for Beech; no OwnerVerified entry |
| A02·C1(e) | `a02_c1e_a_page_that_does_not_name_the_key_states_nothing` | exact: none → correct, a name → wrong |
| A02·C2(a) | `a02_c2a_a_stopped_run_resumes_without_repeating_calls_or_spend` | exact: 3 sent, then 2 sent on restart; 5 recordings; ledger 5 × 1 500 micro-USD, counted once |
| A02·C3–C5 | `[manual]` | evidence doc "Benchmark 2", fixtures README, Review log |

Every test in the files maps to a row; no orphan tests.

## Assumptions

1. **Candidate models** `gpt-6.1-sol` (flagship), `gpt-6.1-terra` (mid), `gpt-6-luna` (small), from the
   models page of 2026-11-12. `gpt-6-astra` was left out on price (USD 10 / 50 per 1M tokens).
2. **Prices** come from the pricing page (Standard, short context), saved in `<out>/prices.json` with
   URL and date. Cache-write tokens are charged at the "30m write" rate *instead of* the input rate
   (prompt-caching guide). Web searches are counted from `tool_usage.web_search.num_requests`.
3. **Worst case of a call** (for the ceiling): every text byte counted as a token, priced at the
   higher of the input and cache-write rates. Each image is counted as 6,000 tokens; V4 measured at
   most 4,916. A web search adds USD 0.01 plus 40,000 content tokens (V1/V5 saw ≤ 25,461 input
   tokens in total), up to `max_tool_calls`. A call that left but got no reply is charged its worst
   case to the ledger.
4. **Search vendors cost USD 0** in the ledger (free allowances); only their call caps apply. The
   shop (cap 4) and page fetches (no cap; politeness instead) are not money.
5. **Naming match** = case-folded, every character that is not a letter or digit removed, then a
   substring test. So "HANS J WEGNER - CH23" names "Hans J. Wegner", and "Wegner chair" does not.
   Diacritics are kept (Wørts ≠ Wörtz).
6. **Hedged names:** pieces 2, 37 and 38 use their metafields. Piece 26 has empty metafields, so its
   hedged name "Holger Christensens Møbelmagasiner" (description, `data_notes`) is scored in a third
   bucket, `FromText`, so that the corrected bucket still holds exactly the six corrections.
   Groups ("with/without attribution") follow the file's `attribution` field.
7. **Exact copies:** with `type=all` Lens marks none, so the with/without columns are equal; no
   extra `type=exact_matches` search was bought.
8. **Disagreements (U1):** results on the shop or a trusted site in the first 10 that do not name
   the key are listed as review candidates.
9. **Lens locale:** `hl=en&country=se` (the owner's market).
10. **Resume rule refinement:** a recorded reply with HTTP 401, 403, 429 or 5xx is not reused on a
    re-run (moved to `superseded/`) so that a refused call can be retried. Every other recording is
    reused. Page reading waits out a 429's `Retry-After` (≤ 120 s) once.
11. **Shop hosts for scoring** = the test file's storefront host (`beyovintage.se`) plus V9's hosts
    (the test store); the live store's `.myshopify.com` host never appeared in results.
12. **U2 scored keys:**
    - observe = `wood_type` (the only visible field the key holds; quantity on a set photographed
      singly measures nothing);
    - read = `designer`, `manufacturer` (pieces with attribution; up to 3 pages: the first naming
      result, then the first two non-naming shop/trusted results among R1·U1's first ten, HTTP 200
      only; page text = tag-stripped HTML cut to 12,000 characters);
    - condition = the `condition_*` keys each piece's metafields carry (its layout); ≤ 6 condition
      photos at 2048.
    - An answer that maps to no choice is *not determined* (MC-A2 failed mapping).
    - Read stage per row A02·C1(e): on a page that does not name the key any name is wrong (the
      report also splits this out, see Observations).
13. **Choice strings** for wood and severities come from
    `claude_design/reference/SHOPIFY_PRODUCT_METAFIELDS.md` (the live shop's definitions), not from
    `sv1_definitions_p1.json` (see Deviations).
14. **Best model per stage** for the effort runs = highest (correct − wrong), owner-verified,
    default effort: observe Sol, read Terra, condition Terra. Lower/higher effort = `low` / `high`.
15. **Command-line parser:** none (hand-written, no new dependency). `reqwest 0.13` gains the
    `blocking` feature in this crate (the version is pinned; the master plan allows `blocking`).

## Deviations

1. **Cloud Vision route not measured** (HTTP 403 `BILLING_DISABLED` on every call). The second
   column of Benchmark 1, the `cloud_vision/` "web detection reply cut to 3 results" fixture
   (row A01·C7(a)) and the Vision part of V6 are therefore missing. The 403 reply is captured.
   Card 1.
2. **V9 ran against the test store** (the store in `~/.psm_shopify.env`): the live product id is not
   there, so `onlineStoreUrl`/`handle` were not observed. Other domains need `read_markets`.
3. **V8 pages were chosen from the benchmark's results**, so task 3 ran before V8. V6 was answered
   from the benchmark's piece-2 search rather than a separate `probe v6` run.
4. **Row A01·C3(a) is tested with three exchanges, not one.** A hit stops the write, so the
   "declared parts read `<redacted>`" part needs an exchange without an echo.
5. **Row A02·C2(a)'s "stopped after 3 of 5"** is produced by a call cap of 3 on the first run
   (deterministic), not by killing a process.
6. **U2 choices source:** the plan names `sv1_definitions_p1.json`, but that is the *test store's*
   definitions (its `wood_type` choices are `oak-1, lumber, jorge, …`; no condition keys). The
   live shop's choices were taken from `claude_design/reference/SHOPIFY_PRODUCT_METAFIELDS.md`.
7. **Two read-only shop calls plus one token call that was redone.** The first V9 run lost the token
   (the reply was redacted before use). I deleted that one token recording by hand
   (`probes/v9/calls/v9-token-*.json`, its token already `<redacted>`) so the probe could run fresh.
   Then `live_body` was added.
8. **Unit order:** U2's modules were drafted while U1's page re-fetch was waiting on the shop's
   throttling. They were held out of the U1 checkpoint (U1 commit builds and tests alone) and
   committed with U2.
9. Extra module `src/fixtures.rs` and commands `score-search`, `score-models`, `recost`,
   `fixtures` (module split is a delegated choice).
10. The superseded page records (21) live in `<out>/pages/superseded/` and are not counted by the
    ledger (all cost 0).
11. The version-0 instruction files (`src/instructions/*.md`) and the `meta` field of
    `recording::Call` were already on disk at the U1 checkpoint and landed in `60f6d6c`. They are
    U2 material and unused by U1.
12. The Tracker actor is `opus` (this session ran on Claude Opus), not `codex`.
13. The `WebFetch` tool was refused by a permission prompt nobody could answer. The pricing,
    models and documentation pages were read by plain HTTP GET (`curl`, generic client name) into
    `~/psm-research-bench/a01/{pricing,docs}/` instead. This is the same kind of read the tool
    itself makes; no permission rule was refused for any command this plan needs.

## Files changed

Code and docs (both checkpoints, `git diff --name-only 60f6d6c^ 7b3a2f8`):
- workspace `Cargo.toml` (one `members` entry), `Cargo.lock` (the tool crate's entry);
- `crates/research-tool/`:
  - `Cargo.toml`;
  - `src/{main, lib, answer_key, bench_models, bench_search, budget, cloud_vision, fixtures, http, openai, pages, probes, recording, report, score, serpapi, shop}.rs`;
  - `src/instructions/{observe, read, condition}.md`;
  - `tests/tool/{main.rs, u1_probes_and_search.rs, u2_stage_models.rs}`;
- `crates/research/fixtures/`: README plus 32 captures in `openai/`, `serpapi_lens/`,
  `cloud_vision/`, `pages/` and `shop/`;
- `docs/intention/evidence/research_providers.md`.

Closing commit: `docs/implementation/plans/R1_research_benchmark.md` (Tracker line, Review log) and
this handoff.

Outside the repo, never committed: `~/psm-research-bench/a01/` (U1 recordings, `prices.json`,
`report.md`, `search_scores.json`, downloaded doc pages) and `~/psm-research-bench/a02/` (U2
recordings, `report_models.md`, `model_scores.json`).

No mutation-probe files (no mutation regime, flow §4). No app file changed, so no app suite ran
(flow §10). No architecture graph in this project.

## Checkpoint SHA

- U1: `60f6d6c` — CHECKPOINT (not approved): R1_research_benchmark U1, provider probes and visual-search benchmark.
- U2: `7b3a2f8` — CHECKPOINT (not approved): R1_research_benchmark U2, first per-stage model comparison.
- Closing (tracker, Review log, handoff): the commit after `7b3a2f8` (see `git log`).

## Stamp

The plan's closing gate, run once on tree `7b3a2f8` (clean apart from this handoff):

| Command | Result |
|---|---|
| `cargo test -p psm-research-tool` | 14 passed, 0 failed (`tool`: 8 `u1_probes_and_search`, 6 `u2_stage_models`) |
| `cargo clippy -p psm-research-tool --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cargo check --workspace` | clean |

Per unit:
- U1 (before `60f6d6c`): the same four commands with `u1_probes_and_search::` (8 passed).
- U2 (before `7b3a2f8`): the same with `u2_stage_models::` (6 passed).

No pre-existing failures. No app suite: no app file changed (flow §10, the plan's closing gate).

Secret scan of `crates/research/fixtures/`, for the three provider keys, the shop client id and
secret, and the test-store domain: 0 files.

## For the coordinator (fold upstream)

1. **08B MC-A8 findings** (evidence doc §Findings against 08B):
   - no `cache_write_token` unit, although GPT-6 replies report `cache_write_tokens` at 1.25× input;
   - web searches are counted by `tool_usage.web_search.num_requests`, not by output items.
2. **RESEARCH-VERIFY-9:** shop "other domains" need the `read_markets` scope. The product
   storefront fields were not observed (the test store does not hold the live product); R8·U3 needs
   a product in the test store, or the live store.
3. **RESEARCH-VERIFY-8:** Shopify storefronts throttle a plain client after a burst, so R8·U3's
   HTTP reader needs a slow per-host pace for Shopify-hosted sites. The model page tool returns no
   page text.
4. **Trusted list:** pamono's country sites are separate hosts, not covered by `pamono.com`.
   Worth a sentence to the owner when the list is set.
5. **Scoring rule A02·C1(e)** counts a correct reading of another piece's designer as wrong.
   R9·U2 may want "page does not name the key" scored apart (the report already splits it).
6. **`answer_key_corrections.json`** piece 30 `wood_type` "Rosewood" is not a shop choice.
7. Reviewer notes go to `docs/implementation/backlog_research/R1_research_benchmark.md` (not
   created by me).

## Manual try-out

1. Open `docs/intention/evidence/research_providers.md`, section **Benchmark 1**: for the Lens
   route, 24 of 25 published and 6 of 7 unpublished pieces with a designer or maker had a page
   naming it in the first ten results, usually the first.
2. Open `~/psm-research-bench/a01/report.md`, go to **Check sheet** (pieces 1–4, 21–24, 31–34) and
   mark for each result whether it is really the right listing (same piece or model). Only the Lens
   column has results; Cloud Vision is blocked (card 1).
3. Read **Page reading** in the evidence doc: which of the nine sites a plain download can read
   (auctionet, bukowskis, pamono, maleforsimports, sweakronan) and which only OpenAI's page tool
   reads (etsy, midcenturymobler, vgliving, your shop when throttled).
4. Read **Benchmark 2** in the evidence doc (full table: `~/psm-research-bench/a02/report_models.md`):
   per stage, which model answered correctly how often and what a piece costs.
5. After attaching billing to the Google project (card 1), run from the repo root:
   `set -a; . ~/.psm_research.env; set +a; cargo run -p psm-research-tool -- bench-search --routes cloud_vision --out ~/psm-research-bench/a01 --ceiling-usd 15`
   (42 Vision units, free allowance; nothing else is re-sent), then the Benchmark 1 tables in
   `report.md` include Vision.

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1 — Turn on billing for the Google Cloud project that holds the Vision key (needed for the visual-search pick).**
- *What happened:* every Cloud Vision call answered HTTP 403 `BILLING_DISABLED` ("This API method
  requires billing to be enabled"). The key itself is accepted, but its Google Cloud project has no
  billing account attached, so the Cloud Vision route of Benchmark 1 has no numbers. The Lens route
  is fully measured.
- *Options:* **A** attach a billing account to that project in the Google Cloud console
  (Billing → link a billing account; the first 1,000 Vision units a month stay free, and this
  benchmark needs 42), then re-run one command (Manual try-out step 5): about 3 minutes, USD 0.
  **B** pick Lens via SerpApi now without the comparison, accepting the legal position recorded in
  the evidence doc (Google v. SerpApi).
  **C** drop Cloud Vision as a candidate.
- *Recommendation:* A. The pick between a disputed vendor route and Google's own route should rest
  on numbers for both, and it costs nothing.
- *What it blocks:* the visual-search adapter pick (R8·U2). Nothing else in R1.

## Card 1 resolved (orchestrator, 2026-11-13)
Owner chose **A**: billing is attached to the Google Cloud project of the Vision key (project `psm-research`,
number 203931298590, billing account "Vision API Billing"; checked in the console by the orchestrator at the
owner's request). The orchestrator ran Manual try-out step 5 on 2026-11-13 01:04 on track B; it exited 0 with 42
successful Vision searches and rewrote `~/psm-research-bench/a01/report.md`. The evidence doc's Benchmark 1 now
carries the Vision rows (orchestrator edit, numbers copied from `report.md`). Handoff state set to DONE so the
runner resumes at the review (`RESUME=1`). Track B was merged into `feature/agentic`; the review runs on track A.
