---
plan: R1_research_benchmark
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: OWNER_DECISIONS_PENDING
date: 2026-11-13
actor: opus (pipeline-reviewer, Opus 5.5)
---
# R1_research_benchmark — review round 1: APPROVED_WITH_NOTES

All 22 rows (U1 13, U2 9) hold. The 14 automated rows each have a test at the row's boundary that would fail on a
wrong outcome. The 8 manual rows were checked against the recordings in `~/psm-research-bench/` on this Mac, and the
Lens and Vision headline counts were recomputed independently from the raw replies. The perimeter equals the
declared files. The review-mode checks are green. The fixtures hold no key value and no third-party page body. No
B1–B4. Nine notes went to the research backlog. The plan's three owner picks are in the narrative as decision cards
(below, verbatim). They do not block approval; R8 waits on them.

## ⚠ OWNER DECISIONS REQUIRED (3)

**Card 1 — Which photo search should research use?**
- **Question:** Google Lens through SerpApi (A) or Google Cloud Vision (B)?
- **Story:** Seven of your pieces have photos that were never online. For 6 of them, Lens found a page naming the designer or maker, nearly always as the first result. Vision found none. On 25 published pieces, Lens found 24 and Vision 8, and only 4 once Vision's copies of your own photos are set aside. But Lens reaches Google through SerpApi, which Google is suing for scraping. Google's claims were dismissed twice, and it may refile until 30 November. No court has ruled the scraping lawful.
- **Branches:** A — research usually finds the maker from the photo; free for 250 searches a month, then USD 25 per 1,000; if SerpApi is blocked, photo search stops until Vision is switched in. B — Google's own service, free up to 1,000 a month and faster (about 1 s), but it rarely finds the maker.
- **Recommendation:** A, with Vision kept as the replacement: 6 of 7 against 0 of 7 decides it. Mark the check sheet first if you want to confirm the hits.
- **On silence:** the photo-search connection is not built; the gate holds.
- **Trace:** 08A §4, §12A.1 · evidence Benchmark 1 · R8·U2.

**Card 2 — How should research read a listing page?**
- **Question:** A plain download from the Mac (A) or OpenAI's page tool (B)?
- **Story:** A plain download read 5 of your 10 sites, with the designer and maker in the page: auctionet, bukowskis, pamono, maleforsimports and sweakronan. Etsy and vntg block it. Your shop, midcenturymobler and vgliving are Shopify stores and refuse after a quick burst, so they must be read slowly. OpenAI's page tool read 8 of 10, all but pamono and vntg, for about one cent a page. But it returns only the model's summary, never the page itself, so later you could not see what the page actually said.
- **Branches:** A — exact page text, kept and checkable later, free; etsy and vntg stay unread (their results still count by title). B — two more sites, a cost per page, no page text kept, and the page-reader design must change.
- **Recommendation:** A, with Shopify stores read slowly: it keeps evidence you can check.
- **On silence:** the page-reading connection is not built; the gate holds.
- **Trace:** 08A §4 page reader · evidence Page reading · R8·U3.

**Card 3 — First AI model per research step?**
- **Question:** Photo look: gpt-6.1-sol, low effort. Page reading: gpt-6.1-terra, low. Condition: gpt-6.1-terra, high. Agree?
- **Story:** On cover photos (36 pieces), Sol named the wood right 21 times and wrong 9. Terra and Luna mostly declined to answer. On pages that name the maker, all three models were right 28–31 times out of 41. On condition photos (203 values), Terra at high effort was right 150 times, Sol 146 and Luna 124. Most misses were one step off, such as None instead of Minor.
- **Branches:** Yes — the best measured accuracy, about USD 0.04 a piece for the model. Cheaper page reading — Luna: 56 right instead of 60, at an eighth of the cost. Faster condition — Terra at default effort: 145 right, about 12 s a piece instead of 21 s.
- **Recommendation:** Yes; the final measurement on the finished agent confirms or changes it.
- **On silence:** no model defaults are written; the gate holds.
- **Trace:** 08A §5, §9 · evidence Benchmark 2 · R9·U2, R10·U1.

## Evidence
| What | Scope | Command | Tree | Result |
|---|---|---|---|---|
| Gate | — | plan Tracker line; `git log` | `c337a91` clean | IMPLEMENTED; `60f6d6c`, `7b3a2f8`, `0446282` present |
| Perimeter | — | `git diff --stat 60f6d6c^ 7b3a2f8`; `git show --stat 0446282`; `git diff --stat 7b3a2f8 HEAD` | same | 59 paths = handoff Files changed; closing = plan + handoff; tool crate unchanged since `7b3a2f8`; evidence doc +16 lines by the orchestrator (`942abdd`, Vision rows) |
| Tool tests | L1/L2 (whole crate) | `cargo test -p psm-research-tool` | same | 14/14 |
| Lint | — | `cargo clippy -p psm-research-tool --all-targets -- -D warnings` | same | clean |
| Format | — | `cargo fmt --all -- --check` | same | clean |
| Workspace | — | `cargo check --workspace` | same | clean |
| App suite | — | not run (no app file in the diff; plan Closing gate, flow §10) | — | — |
| Spend vs recordings | manual | tally of every `calls/*.json` under `~/psm-research-bench/a01`, `a02` (scratch script outside the repo) | — | U1 OpenAI 38 / USD 0.243 472; SerpApi 45 search, 44 upload; shop 2; Vision 42×200 + 1×403; pages 394 (incl. the Vision run's fetches). U2 OpenAI 645 / USD 5.580 194 |
| Benchmark 1 variation | manual | independent recompute from raw SerpApi / Vision replies + fetched pages | — | Lens 24/25, 6/7, median rank 1 = report; Vision 8/25, 0/7 = report (a rough recompute that keeps `<script>` text adds pieces 5 and 12 via "IKEA" in scripts; the tool strips scripts, correctly) |
| V8 statuses | manual | status/final URL of the 10 `v8-http-*` records | — | = Page reading table |
| Fixtures secrets | manual | scan of `crates/research/fixtures/`, `crates/research-tool/`, `docs/intention/evidence/` for the 6 values in `~/.psm_research.env`, `~/.psm_shopify.env` (plain, `quote`, `quote_plus`) | — | 0 hits |
| Third-party bodies | manual | body type of every `pages/*.json` and web-search fixture; grep for `<html`/`<!doctype` | — | only `pages/shop_listing_page.json` (owner's shop) has a body; third-party records `body: null`; web-search fixtures hold model output only |

## Verified correct
- **A01·C1(a)** `answer_key::is_naming` folds case and drops non-alphanumerics, then substring: "Wegner chair" ✗,
  "HANS J WEGNER - CH23" ✓. Test asserts both exactly.
- **A01·C1(b)** `is_own_listing`: shop host or subdomain, path (trailing `/` trimmed, query ignored) ends in
  `/products/<handle>`. `-2` suffix and a foreign host are rejected. All four URLs asserted.
- **A01·C1(c)** `key_entries` on the real files: every correction appears once, in `Corrected`, with the corrected
  value, never in `OwnerVerified`; count = 6 = file length. A correction for an absent field is added in `Corrected`.
- **A01·C2(a/b)** `Caller::call` order: resume → `Ledger::check` (cap first, then `spent + worst ≥ ceiling`) →
  send. Refusal returns before `seam.send`; the stub's `received` is asserted empty. Cap message names
  `openai = 80`.
- **A01·C2(c)** `run_inner` requires `--ceiling-usd` for `probe|bench-search|bench-models` before creating the out
  folder or building the caller. Tested through `run` (stub empty) and through the binary with the keys unset
  (stderr names the flag).
- **A01·C3(a)** Redaction of `key`/`api_key` query values, the six secret headers and `api_key`/`client_secret`/
  `access_token` body fields; then a fail-closed scan of the record and every image (plain, percent-encoded,
  `+`-form) before anything is written; the error names the call id only. The test would fail if redaction or the
  scan were missing (the declared-only exchange would be refused, or the echo would be written).
- **A01·C3(b)** `replace_markers` records `{"image_sha256"}` per image; images written once as `images/<sha>.jpg`.
- **A02·C1(a–e)** `score::map_choice` = byte-equal, else the unique `name_key` match (MC-A2); none/ambiguous →
  not determined. Multi-choice compares mapped sets. Text by `name_key`, no aliases. `score_field` uses the entry's
  bucket. `score_read(false, …)` per the row. Each test input discriminates (e.g. `teak` maps only via `name_key`).
- **A02·C2(a)** A recorded call returns from disk before the budget check and is not added to the ledger. The
  ledger restarts from recordings. 3 sent, then 2; 5 files; 5 × 1 500 micro-USD in both the file sum and the live
  ledger.
- **Manual rows** A01·C4 (one row per V1–V10 + ep4, observed with capture or documented with page and reason),
  C5 (tables per route and group, per-piece table in `report.md`, calls and cost), C6 (one row per site and route),
  C8 and A02·C5 (spend within caps and ceilings; above), A02·C3 (tables per stage, model and effort, ov/ck apart,
  disagreements listed in `report_models.md`), A02·C4 (one reply per stage, README dated, scan clean).
- **Hard constraints.** H1: V9 is one read-only GraphQL query (no mutation in `shop.rs`/`probes.rs`). H3: reduced
  copies via `psm_infra::imaging::ImageOpsImpl::thumbnail`, no new pixel code. H2, H5, H6, H7, H8: no app file
  touched. Secrets: values only from the environment; errors use `without_url()`; nothing printed before the scan.
  No live call in any test (stub seam).
- **Assumptions 1–15.** None contradicts the intention. They are listed for the owner in the narrative's "Choices
  made for you". Notes on two of them: #12's read rule is the plan's rule; #13 (live shop's choices instead of
  `sv1_definitions_p1.json`) is the correct reading of MC-A2, because the plan's file is the test store's.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
All nine are in `docs/implementation/backlog_research/R1_research_benchmark.md`. That is the research track's
backlog, as the plan says. The prompt's generic `backlog.md` is not used for research milestones.

| Note | Destination |
|---|---|
| Successful Cloud Vision fixture missing (A01·C7 row, declared deviation; recordings now exist) | orchestrator / R8·U2, only if Vision is picked |
| `psm-research fixtures` would now write piece 2's 200 Vision reply as `error_billing_disabled.json` | R9·U1 (grows the tool) |
| Evidence Benchmark 1 table omits "naming without exact copies" (Vision 8 → 4) | orchestrator, with Owner picks |
| Fixture count: 31 captures + README, not 32 | record only |
| A02·C1(e) scores a correct reading of another piece's designer as wrong | R9·U2 plan |
| Piece 30 correction "Rosewood" is not a shop choice | owner / orchestrator (corrections file) |
| pamono country hosts not covered by `pamono.com` | owner, trusted-sites setting |
| Shopify storefronts throttle plain fetches | R8·U3 |
| Page tool returns no page text → cannot satisfy the page-reader port as written | owner card 2 / R8·U3 |

## Lessons for the plans
- A manual fixture row whose capture depends on an external precondition (billing, a product in the store) should
  name what happens when the precondition is lifted later: who exports the fixture, and when.
- The plan named `sv1_definitions_p1.json` as the choice source for U2. It is the test store's, not the live
  shop's. Rows that score against shop choices should cite the live shop's definitions.

## Write perimeter (this session)
- `docs/implementation/plans/R1_research_benchmark.md`: Tracker line and Review log entry.
- `docs/implementation/narratives/R1_research_benchmark.md` (new).
- `docs/implementation/backlog_research/R1_research_benchmark.md` (new).
- This handoff (new).
- Outside the repo: scratch scripts in `/tmp/r1review/` (read-only over the recordings and env files; they print
  file names and counts, never a value).
- No production file, fixture, recording or tool-recorded state touched; no mutation probes (flow §4); no commit.
