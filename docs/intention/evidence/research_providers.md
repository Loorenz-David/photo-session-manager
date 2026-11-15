# Evidence — research providers (visual search, page reading, model)

Researched 2026-11-12 (web research by a subagent; nothing here was called from this repo yet).
Tags: **[V]** read on the cited page · **[V-s]** from a search summary of that page · **[U]** unverified.
Serves chapter `book/08A_research_agent.md` §4. Every [U] and [V-s] row is a probe owed before a
plan may rely on it.

## Reverse image search

| Option | Status | Image input | Returns | Price | Main risk |
|---|---|---|---|---|---|
| Google Lens, direct | no official API [V-s] | — | — | — | not available |
| Google Cloud Vision, Web Detection | first-party, no deprecation notice [V] | base64 bytes inline, `gs://`, or web URL [V] | `webEntities`, `bestGuessLabels`, full/partial matching images, `pagesWithMatchingImages` (url, pageTitle), `visuallySimilarImages`; 10 results by default [V] | first 1,000 units/month free, then USD 3.50 per 1,000 [V-s] | quality on product photos reported as generic labels (forum anecdotes only, no benchmark found) |
| SerpApi, Google Lens API | third party that scrapes Google | public URL, or `POST https://serpapi.com/image` (multipart, 500 KB max) → `image_id` [V]; id reportedly expires after 10 min [V-s] | `visual_matches`: position, title, link, source, thumbnail, image; `type` all / exact_matches / visual_matches [V] | free 250/month; USD 25 / 1,000; 75 / 5,000; 150 / 15,000 [V]; "U.S. Legal Shield" (up to USD 2M) only from the USD 150 plan [V] | legal and continuity (below) |
| SearchApi, Google Lens | third party that scrapes Google | public URL only [V] | as above plus price, currency, stock when shown [V] | USD 40 / 10,000; 100 / 35,000 [V] | same class; needs the photo at a public address |
| Bing Visual Search | retired 2025-08-11 [V] | — | — | — | gone |
| TinEye API | first-party licensed index | [U] | [U] | prepaid bundles, valid two years [V]; current prices [U] | built for exact / near-duplicate, probably not look-alikes [U] |
| Google Custom Search JSON API | closed to new customers, ends 2027-01-01 [V] | text only | — | — | dead end |

Sources: `docs.cloud.google.com/vision/docs/detecting-web`, `cloud.google.com/vision/pricing`,
`serpapi.com/google-lens-api`, `serpapi.com/google-lens-upload-an-image`, `serpapi.com/pricing`,
`searchapi.io/docs/google-lens`, `searchapi.io/pricing`,
`learn.microsoft.com/en-us/lifecycle/announcements/bing-search-api-retirement`,
`developers.google.com/custom-search/v1/overview`, `help.tineye.com/article/169-purchasing-search-bundles`.

**Google v. SerpApi** (N.D. Cal., 4:25-cv-10826-YGR): filed 2025-12-19 under the DMCA
anti-circumvention provisions [V-s]; complaint dismissed 2026-07-20 [V, SerpApi's blog]; amended
complaint dismissed 2026-11-09 with leave to amend until 2026-11-30 [V-s / V, trade press]. The
court has not ruled that the scraping is lawful. A separate Reddit suit against SerpApi is pending
[V-s]. Court orders themselves were not read. **Consequence:** the Lens-through-a-vendor route is
not a clean first-party route; Google contends it breaks its terms. Using it is an owner decision
(chapter 08A, owner card 3), and the adapter must be replaceable.

## Model provider: OpenAI API

- Image input: URL, base64 data URL, or file id (Responses API) [V] `developers.openai.com/api/docs/guides/images-vision.md`.
- Built-in `web_search` tool: text queries only (no image-as-query) [V]; actions search,
  `open_page`, `find_in_page` on reasoning models [V]; `filters.allowed_domains` up to 100 domains,
  subdomains included, Responses API only [V]; USD 10 per 1,000 calls plus content tokens at model
  rates [V] `developers.openai.com/api/docs/guides/tools-web-search.md`, `/api/docs/pricing`.
- Image input and `web_search` in one request: not stated; inferred to work [U].

## Not verified (probes owed)

1. Whether auctionet, pamono, 1stdibs, vntg and the owner's shop let OpenAI's page fetcher, or a
   plain HTTP fetch from the Mac, read a listing page. This decides the page-reader adapter.
2. Lens-via-vendor vs Cloud Vision Web Detection on 20–30 of the owner's known pieces (recall of
   the right listing in the top 10).
3. SerpApi upload: credit cost, accepted formats, `image_id` lifetime.
4. Cloud Vision price (page not opened), TinEye capabilities.
5. Image input combined with `web_search` in a single OpenAI request.

## Owner inputs for the benchmark

**Trusted sites, in priority order (owner 2026-11-12).** These are the owner's entries for the
setting `research.trusted_sites` (08A §6); the app itself ships with an empty list.

1. `etsy.com`
2. `pamono.com`
3. `bukowskis.com`
4. `midcenturymobler.com`
5. `vntg.com`
6. `maleforsimports.com`
7. `auktion.sweakronan.se`
8. `vgliving.co`
9. `auctionet.com`

Probe 1 (can a listing page be read, and by which route) runs against each of these.

**Test pieces:** `docs/test-items/beyo_test_samples_1.json` (42 pieces) and `docs/test-items/answer_key_corrections.json`.

---

# R1 results (2026-11-12, `psm-research`, recordings in `~/psm-research-bench/a01/` on the dev Mac)

Everything below was called from this repo's tool (`crates/research-tool`, bin `psm-research`) with the
owner's keys from `~/.psm_research.env`. Every exchange is a call file under the out folder
(`<out>/<piece n>-<handle>/run-bench/calls/…`, `<out>/probes/<id>/calls/…`, `<out>/pages/calls/…`);
sanitized copies are in `crates/research/fixtures/` (README there). "Capture" names below are call ids
in the out folder. *Observed* = seen in a recorded reply; *documented only* = read on the cited page
(downloaded the same day to `<out>/docs/`), with the reason it was not provoked.

## Candidate models (R1·U1 task 0)

Prices: `<out>/prices.json`, filled from `developers.openai.com/api/docs/pricing` (Standard tier, short
context, read 2026-11-12). Model list: `developers.openai.com/api/docs/models` and `GET /v1/models` with the key.

| Role | Model id | Input / cached / 30-min cache write / output, USD per 1M tokens | Why |
|---|---|---|---|
| flagship | `gpt-6.1-sol` | 2.00 / 0.10 / 2.50 / 10.00 | "near-Astra performance at a lower cost"; image input |
| mid | `gpt-6.1-terra` | 1.00 / 0.05 / 1.25 / 5.00 | "fast, efficient model for visual reasoning"; image input |
| small | `gpt-6-luna` | 0.10 / 0.01 / 0.125 / 0.50 | "most efficient model"; image input |

Excluded: `gpt-6-astra` (10 / 50; one 2048 image call alone is a large part of the USD 0.50 run cap).
Web search tool: USD 10 per 1,000 calls plus search content tokens at model rates. All three take
`reasoning.effort`; default `medium` (Sol: `low…`; Terra and Luna also `none`).

## Probe answers (RESEARCH-VERIFY-1…10, evidence probe 4)

| Probe | Question | Method | Answer | Kind |
|---|---|---|---|---|
| V1 (VERIFY-1) | Usage fields; tool calls; does an incomplete reply report usage | one image + strict-schema call per candidate; one `web_search` call (Luna); one call with `max_output_tokens: 16` | `usage` = `input_tokens`, `input_tokens_details.{cached_tokens, cache_write_tokens}`, `output_tokens`, `output_tokens_details.reasoning_tokens`, `total_tokens`. **`cache_write_tokens` is new** and billed at the cache-write rate (1.25× input, 30-min default) *instead of* the input rate (prompt-caching guide). Web searches are counted in the reply's **`tool_usage.web_search.num_requests`** (V5: 3 billed requests for 4 `web_search_call` items); search content tokens are inside `input_tokens` (Luna web search: 8,746 input tokens for a one-line question). The incomplete reply reports full usage (16 output tokens, all reasoning) and `incomplete_details.reason: max_output_tokens`. Default effort echoed in `reasoning.effort: medium`. | observed: `v1-image-schema-gpt-6.1-sol-607a3ff41c52`, `…-terra-b05c0dee34c2`, `…-luna-02c7aad23123`, `v1-web-search-gpt-6-luna-7203b3f79e57`, `v1-incomplete-gpt-6-luna-91e24938d6fc` |
| V2 (VERIFY-2) | Error replies | a made-up key string; ~1.2 M tokens of text | Bad key: HTTP 401, `error.type: invalid_request_error`, `code: invalid_api_key`; the message echoes the key **masked** (`sk-proj-***…0000`, last 4 characters). Oversized: HTTP 400, `invalid_request_error`, `code: context_length_exceeded`, `param: input`, after 3.6 s; neither carried usage or `Retry-After`. Rate limit: 429 (`rate_limit_error` / `slow_down`), no credit: 429 `credit_balance_exhausted` (type may be `insufficient_quota`), spend limits: 429 `organization_spend_limit_exceeded` / `project_spend_limit_exceeded`, server: 500, overloaded: 503 `server_is_overloaded`; `Retry-After` (seconds) "when present" on 429/503. Normal replies carry `x-ratelimit-{limit,remaining,reset}-{requests,tokens,input-images}`. | 401/400 observed: `v2-invalid-key-…`, `v2-oversized-input-…`; 429/500/503 documented only (`developers.openai.com/api/docs/guides/error-codes`, `…/rate-limits`): provoking them needs a burst or an empty account |
| V3 (VERIFY-3) | Strict-schema refusal shape; `incomplete` | from V1 and the structured-outputs guide | Incomplete: `status: "incomplete"`, `incomplete_details: {"reason": "max_output_tokens"}`, output holds only a `reasoning` item (no message). Refusal: a `message` item whose `content` has `{"type": "refusal", "refusal": "<text>"}` instead of `output_text`. No refusal occurred in 38 calls. | incomplete observed (`v1-incomplete-…`); refusal documented only (`…/guides/structured-outputs`): no benign request provokes one |
| V4 (VERIFY-4) | Image limits; `detail`; tokens for a 1024 and a 2048 copy | piece 2's cover at 1024 (101,790 bytes) and 2048 (409,909 bytes), detail auto/high/low/original, each model, effort low, vs a text-only baseline (13 tokens) | Image tokens, the same on all three models: 1024 auto = high = **1,229**; 1024 low = **308**; 2048 high = **3,001**; 2048 original = **4,916**. Default `detail` is `auto` (= high here). Limits: PNG, JPEG, WEBP, non-animated GIF; up to 512 MB per request, up to 1,500 images per request, up to 30,000 patches per image. | tokens observed (`v4-…`, 18 captures); limits documented only (`…/guides/images-vision`) |
| V5 (VERIFY-5) | Image + `web_search` in one request; `allowed_domains` with a subdomain | Terra: photo + web search restricted to the nine trusted sites; a text search restricted to `auktion.sweakronan.se` | **Works in one request** (HTTP 200, completed; the model searched with text it wrote from the photo; it named Poul Hundevad — wrong for piece 2, see Benchmark 1). Results came from `www.pamono.com`, `www.vntg.com`, `www.etsy.com`, `vgliving.co`, `auctionet.com`: an entry covers its `www.` and other subdomains. A subdomain itself as the entry works and keeps results on that host. `max_tool_calls: 3` held for billing (`num_requests: 3`) though 4 `web_search_call` items were returned. | observed: `v5-image-and-web-search-242b0e3eb330`, `v5-allowed-subdomain-811eb70b241d` |
| V6 (VERIFY-6) | Result shape and rank, usage units | the benchmark's searches | Lens (SerpApi, `type=all`): `visual_matches[]` with `position` (1…~60), `title`, `link`, `source`, `source_icon`, `thumbnail`, `image`, sizes, sometimes `price`, `rating`, `in_stock`; also `organic_results`, `ai_overview`, `search_metadata`. **No `exact_matches` flag on any of 2,515 items** with `type=all`. Usage unit: one search credit per search, nothing in the reply itself (`search_metadata.total_time_taken` only). Cloud Vision: **not observed** — every call returned HTTP 403 `BILLING_DISABLED` (the key's Google project has no billing account); shape from the docs: `responses[0].webDetection.{webEntities, bestGuessLabels, fullMatchingImages, partialMatchingImages, pagesWithMatchingImages[{url, pageTitle, fullMatchingImages, partialMatchingImages}], visuallySimilarImages}`, one unit per image. | Lens observed (`2-square-teak-dining-table/run-bench/calls/serpapi-search-…`, 42 more); Vision documented only (`docs.cloud.google.com/vision/docs/detecting-web`) — blocked by billing |
| V7 (VERIFY-7) | SerpApi upload | account before/after; upload; search; oversized upload; same `image_id` later | The key goes in the multipart form field `api_key` (upload) and the query parameter `api_key` (search, account). Upload reply: `{"image_id", "message": "Image uploaded successfully."}` in ~0.4 s. **Uploads cost no credit**; each search costs one (account: 23 used after 23 searches and 25 uploads; the account figures lag a few minutes). Over 500 KB: HTTP 400 `"Image file size too large. Maximum size is 500KB"` (586,494 bytes). The `image_id` still worked **5.6 min** after upload and was gone at **11.9 min** (HTTP 400 `"Image not found."`). Formats: JPEG observed; the docs name no other limit. A 1024 JPEG of the test pieces is 67–220 KB, so no piece needed a smaller copy. | observed: `v7-upload-d6421c3f6ff8`, `v7-search-0595c4df04d9`, `v7-upload-oversized-b173fe4c1d7d`, `v7-search-late-45806bb0118e`, `v7-search-late2-4437b92c3f39`, `v7-account-…` |
| V8 (VERIFY-8) | Page reading by both routes | see "Page reading" below | see below | observed |
| V9 (VERIFY-9) | Shop hosts; a product's storefront URL and handle | client-credentials token, one read-only Admin query on the **test store** (the store in `~/.psm_shopify.env`) | `shop.primaryDomain {host, url, sslEnabled}` and `shop.myshopifyDomain` are readable with the app's scopes (test store: both are its `.myshopify.com` host). **Other domains need the `read_markets` scope** (`webPresences` → `ACCESS_DENIED`). `product(id)` with the live shop's product id returned `null` (the product is not in the test store), so `onlineStoreUrl` / `handle` were not observed. The live shop's storefront URL form is in the test file: `https://beyovintage.se/en/products/<handle>` (also `/products/<handle>` and `/sv/…` in search results). | observed (`v9-hosts-…`, `v9-token-…`); product fields not observed: wrong store |
| V10 (VERIFY-10) | Is a call cut by a 1 s client timeout billed | Terra, image + schema, 1 s timeout | The request left (`is_connect` false), `timed_out: true`, after 1,003 ms. Billing **not observable**: with `store: false` no response id comes back and per-request usage is not queryable with a project key. **`fallback_call_usd` stands.** The tool charged its worst case (USD 0.018) to its own ledger. | observed: `v10-timeout-1s-51680b6243ed`; billing not observable |
| Evidence probe 4 | Cloud Vision price; TinEye | the pricing page read by the tool | Web Detection: **first 1,000 units / month free, then USD 3.50 per 1,000** (USD 3.15 / 2.80 with savings plans); priced per image per feature. TinEye: not pursued (built for exact copies; the plan did not ask for it). | observed: `ep4-vision-pricing-720902b04795`; also `ep4-vision-error-empty-image-…` (403 BILLING_DISABLED) |

RESEARCH-VERIFY-11 is grounded in `master_plan_research.md` §7.4 (not redone).

## Benchmark 1 — visual search (R1·U1)

42 pieces, the reduced copy (long edge 1024, `psm_infra` thumbnail function) of each piece's first
image. Full report with the check sheet: `~/psm-research-bench/a01/report.md`; numbers:
`search_scores.json`. Definitions as in the R1 plan (own listing removed before ranking; a naming
result has the key designer or manufacturer in its title or fetched page text; groups computed from
the file's `published_online` and `attribution` fields).

**Cloud Vision Web Detection, measured 2026-11-13 01:04** (owner attached billing to the key's Google Cloud
project; the orchestrator ran `psm-research bench-search --routes cloud_vision --out ~/psm-research-bench/a01
--ceiling-usd 15`, which rewrote `report.md`; 42 successful searches, 44 Vision units in total, inside the free
1,000; model spend of the out folder unchanged at USD 0.243). The first attempt (2026-11-12) returned HTTP 403
`BILLING_DISABLED` on every call. 21 of 42 pieces returned no web result at all from Vision.

| Route | Group | Pieces | Naming result in first 10 | … only shop or trusted results | Median rank of first naming | Own listing returned |
|---|---|---|---|---|---|---|
| Lens via SerpApi | published, with attribution | 25 | **24** | 23 | 1 | 13 |
| Lens via SerpApi | unpublished, with attribution | 7 | **6** | 6 | 1 | 0 |
| Lens via SerpApi | published / unpublished, without attribution | 5 / 5 | (no key: first 10 listed in the report) | — | — | 0 / 0 |
| Cloud Vision Web Detection | published, with attribution | 25 | **8** | 5 | 8 | 4 |
| Cloud Vision Web Detection | unpublished, with attribution | 7 | **0** | 0 | — | 0 |
| Cloud Vision Web Detection | published / unpublished, without attribution | 5 / 5 | (no key: first 10 listed in the report) | — | — | 0 / 0 |

- **Exact copies:** with `type=all` Lens marked no result as an exact copy of the photo, so the
  with/without-exact counts are equal. The unpublished group (photos not online) is the clean
  measure: 6 of 7, every first naming result at rank 1.
- Keys outside the owner-verified bucket: piece 26 (hedged, from text: "Holger Christensens
  Møbelmagasiner") — Lens returned "Holger Christiansen cabinet" (vntg.com, rank 5), which the
  naming rule does not count (spelling); piece 30 (corrected: "Tidaholms Möbelfabrik") — no naming result.
- Misses: piece 2 (hedged key Magnus Olesen / Durup; Lens and the V5 model both point to Poul
  Hundevad listings), piece 34 (Glas Mäster mirror). Piece 11's first naming result is on blocket.se.
- **Where the first naming result was:** the owner's own shop 12 (another listing of the same
  designer or model), auctionet.com 9, pamono.se / .eu / .co.uk 4, bukowskis.com 2, others 3.
  **pamono's country sites are not covered by the trusted entry `pamono.com`** (a different host,
  MC-A3); they would need their own entries.
- **Lens vs Vision:** on the clean measure (photos never online) Lens 6 of 7, Vision 0 of 7; published pieces
  Lens 24 of 25 (median rank 1), Vision 8 of 25 (median rank 8; 7 pieces with exact copies in its first 10).
  Vision is faster (0.81 s vs 3.11 s per search) and first-party.
- Calls: 42 searches + 42 uploads (+3 searches and 2 uploads in V6/V7); mean 3.1 s per search
  (1.6–11.0 s), upload 0.4 s. 45 of the free plan's 250 monthly searches used.
- A naming result is not proof of the right listing: the report's **check sheet** (pieces 1–4,
  21–24, 31–34; first 10 of each route with links and an empty "right listing" column) is for the
  owner. The report also lists 100 results on the shop or a trusted site that do not name the key
  (review candidates, `scoring_rules.disagreements`).

## Page reading (RESEARCH-VERIFY-8, probe V8)

One listing page per trusted site and the owner's shop, by (a) a plain HTTP GET from the tool
(generic client name, ≥ 2 s between requests to a host) and (b) OpenAI `web_search` restricted to
the site's domain and asked to open the exact URL (Terra, effort low). Captures `v8-http-<site>-…`,
`v8-model-<site>-…`; the benchmark's 375 plain page fetches are in `<out>/pages/`.

| Site | (a) HTTP status | (a) final URL / redirects | (a) title and attribution in the HTML | (a) images found | (b) model page tool |
|---|---|---|---|---|---|
| beyovintage.se (shop) | **429** "Too many requests", `Retry-After: 60` (benchmark: 7 × 200 first, then 62 × 429; waiting 60 s did not lift it) | same URL, none | not in this reply; in the 200 replies: title, description with designer and maker, product JSON | og:image and gallery in the 200 replies | **read**: final URL, title, attribution verbatim, 5 + 6 images |
| etsy.com | **403** (bot block) | same, none | no | no | opened; the listing had become "This item is unavailable" |
| pamono.com | **200** | same, none | yes / yes (designer and maker in the HTML) | og:image, 172 `<img>` | **403** to the model's fetcher: not read |
| bukowskis.com | **200** | same, none | yes / yes | og:image, 48 `<img>` | read: final URL, title, attribution |
| midcenturymobler.com | **429** (a Shopify store, after the shop burst) | same, none | no | no | read: title; "Designer: Unknown, Manufacturer: Slagelse Møbelfabrik" |
| vntg.com | **403** ("Just a moment…", a script challenge) | same, none | no | no | **403**: not read |
| maleforsimports.com | **200** | same, none | yes / (no attribution on the page) | og:image, 39 `<img>`, product JSON-LD | read; says neither designer nor maker is stated |
| auktion.sweakronan.se | **200** | same, none | yes / yes ("Bodafors, Bertil Frighagen") | 8 `<img>`, no og:image | read: attribution verbatim |
| vgliving.co | **429** (a Shopify store) | same, none | no | no | read: "Design Svend Aage Madsen"; maker not stated |
| auctionet.com | **200** | same, none | yes / yes | og:image, 15 `<img>` | read: attribution verbatim |

- **Plain HTTP reads 5 of 10** (pamono, bukowskis, maleforsimports, sweakronan, auctionet) with the
  title and attribution in the HTML, no script needed. Blocked: etsy and vntg (403, bot protection).
  Throttled: the three Shopify storefronts (the shop, midcenturymobler, vgliving) answered 429 after
  a burst from this Mac; the shop answered 200 to its first fetches. In the benchmark plain HTTP got
  HTTP 200 from auctionet 84/84, bukowskis 35/35, pamono.se 22/22, and 403 from vntg 19/19, etsy 16/16.
- **The model page tool reads 8 of 10** (not pamono, not vntg) and returns the final URL in its
  action (`open_page.url`) and in its answer, **but never the page text**: only the model's own
  summary. A stage-3 reply from this route cannot be replayed with the same page content (MC-A11),
  and its "stated" text is the model's quotation. Cost ≈ USD 0.005–0.011 per page (Terra, low).
- No redirect was seen on any of the ten listing URLs.

## Findings against 08B (reported, not adapted — R1 plan Notes)

1. **MC-A8 unit names lack the cache-write unit.** OpenAI GPT-6 models report
   `input_tokens_details.cache_write_tokens`, billed at 1.25× the input rate *instead of* the input
   rate; MC-A8's units (`input_token`, `cached_input_token`, `output_token`, …) would charge these
   tokens as plain input (a small under-count, ≤ 25 % of input cost). A `cache_write_token` unit is needed.
2. **Web searches are counted by `tool_usage.web_search.num_requests`**, not by the number of
   `web_search_call` items (V5: 3 vs 4). MC-A8's `web_search_call` unit should read that field.
3. **Shop hosts (VERIFY-9):** other domains need the `read_markets` scope; MC-A3's "other domains"
   cannot be read with the app's current scopes.
4. **The model page tool returns no page text (VERIFY-8)**, so MC-A11 replay identity cannot hold
   for stage 3 on that route (08B already says so; confirmed).
5. An incomplete reply reports usage (VERIFY-1): no contract change; a timed-out call's billing is
   not observable (VERIFY-10): `fallback_call_usd` stands.

## Benchmark 2 — first per-stage model comparison (R1·U2)

645 OpenAI calls, USD 5.58 (ceiling 35, cap 700), recordings in `~/psm-research-bench/a02/`, full
table and every wrong answer in `~/psm-research-bench/a02/report_models.md`. Version-0 instructions
and strict schemas: `crates/research-tool/src/instructions/`; the choice strings given to the model
are the live shop's (`claude_design/reference/SHOPIFY_PRODUCT_METAFIELDS.md`). Answers are scored
by name key (MC-A2, no aliases); an answer that maps to no choice is "not determined". `ov` =
owner-verified key, `ck` = corrected key (`answer_key_corrections.json`). No schema failure and no
refusal in any cell.

**Observe** (stage 1, cover photo at 1024; scored field `wood_type`, 36 ov + 2 ck pieces):

| Model | Effort | Correct (ov) | Wrong (ov) | Not determined (ov) | ck correct / wrong / n.d. | USD per piece | s per piece |
|---|---|---|---|---|---|---|---|
| gpt-6.1-sol | default (medium) | 20 | 7 | 9 | 0 / 1 / 1 | 0.0049 | 4.3 |
| gpt-6.1-sol | low | 21 | 9 | 6 | 0 / 1 / 1 | 0.0044 | 4.2 |
| gpt-6.1-sol | high | 21 | 9 | 6 | 0 / 1 / 1 | 0.0058 | 7.9 |
| gpt-6.1-terra | default | 2 | 3 | 31 | 0 / 0 / 2 | 0.0024 | 3.1 |
| gpt-6-luna | default | 3 | 2 | 31 | 0 / 0 / 2 | 0.0003 | 3.4 |

Terra and Luna mostly answered "not determined" (an empty list, `sure: false`), which the
version-0 instruction allows ("If the photo does not let you tell, give an empty list"). Sol
commits. Most wrong answers are the "Teak" / "Dark Teak" pair, or one wood of a two-wood key. The
one corrected-key "wrong" is piece 30: the corrections file says "Rosewood", which is not one of the
shop's choices ("Santos Rosewood" is), and Sol answered "Santos Rosewood". The correction itself
needs the shop's choice string.

**Read** (stage 3, R1·U1's fetched pages, tag-stripped text ≤ 12,000 characters; 59 pages of 31
pieces; scored keys `designer`, `manufacturer`):

| Model | Effort | Correct | Wrong | Not determined | On pages naming the key: correct / wrong / n.d. | On pages not naming it: correct / wrong | USD per piece | s per piece |
|---|---|---|---|---|---|---|---|---|
| gpt-6.1-terra | default | 59 | 37 | 7 | 29 / 5 / 7 | 30 / 32 | 0.0047 | 4.5 |
| gpt-6.1-terra | low | 60 | 37 | 6 | 30 / 5 / 6 | 30 / 32 | 0.0045 | 4.2 |
| gpt-6.1-terra | high | 59 | 37 | 7 | 29 / 5 / 7 | 30 / 32 | 0.0051 | 5.9 |
| gpt-6.1-sol | default | 58 | 39 | 6 | 31 / 4 / 6 | 27 / 35 | 0.0089 | 7.7 |
| gpt-6-luna | default | 56 | 40 | 7 | 28 / 6 / 7 | 28 / 34 | 0.0006 | 6.3 |

Read the two right-hand columns separately. **On a page that does not name the key, the plan's rule
(row A02·C1(e)) counts any name as wrong**, but nearly all of those "wrong" answers are the model
correctly reporting the designer the page gives for *another* piece. That is the job of stage 3; its
evidence weight is decided in stage 4 (compare) and stage 7. On pages that name the key all three
models are right 28–31 times out of 41. Most of their 4–6 "wrong" answers are name forms the
no-alias rule treats as different ("Thorsø" for "Thorsø Møbelfabrik", "Ulferts Möbler" for "Ulferts",
"Carl Malmsten (1888–1972)", "Henry Kjærnulf" for "Henning Kjaernulf").

**Condition** (stage 6, ≤ 6 condition photos at 2048; 28 pieces; scored keys the piece's
`condition_*` metafields, 203 field values):

| Model | Effort | Correct | Wrong | Not determined | HTTP errors | USD per piece | s per piece |
|---|---|---|---|---|---|---|---|
| gpt-6.1-terra | high | **150** | 53 | 0 | 0 | 0.0285 | 21.0 |
| gpt-6.1-terra | default | 145 | 53 | 0 | 1 (timeout at 180 s) | 0.0290 | 11.7 |
| gpt-6.1-terra | low | 128 | 51 | 0 | 2 (connection failed after send) | 0.0304 | 13.3 |
| gpt-6.1-sol | default | 146 | 57 | 0 | 0 | 0.0557 | 11.3 |
| gpt-6-luna | default | 124 | 63 | 16 | 0 | 0.0029 | 9.4 |

The wrong answers are mostly one step apart (None ↔ Minor, Minor ↔ Moderate), and "N/A" where
the key says "None" for veneer or upholstery. The USD per piece for a cell with failed calls
includes the tool's worst-case charge for each (no usage came back).

**Best per stage by (correct − wrong), as measured here:**
- observe: `gpt-6.1-sol`, effort low or default;
- read: `gpt-6.1-terra`, effort low;
- condition: `gpt-6.1-terra`, effort high.

The owner's first pick goes under **Owner picks**; R9·U2 confirms it on the finished engine.

## Owner picks (2026-11-13)

Decided by the orchestrator under the owner's delegation of 2026-11-12 night ("take care of this and continue resolve
questions given the intention"), each by the recommendation of the R1 narrative's card, which follows from the
evidence above. The owner may change any of them; R8 and R10 read this section.

1. **Visual-search route: Google Lens via SerpApi** (card 1, A). Photos never online: Lens 6 of 7, Cloud Vision 0 of
   7. Published: Lens 24 of 25, Vision 8 of 25, and 4 of 25 once results that are exact copies of the owner's own
   photos are set aside (`report.md`, "Naming in first 10 without exact copies"). Accepted risk: SerpApi reaches
   Google by scraping (Google v. SerpApi, see Reverse image search above); if it is blocked, photo search stops until
   a Cloud Vision adapter is switched in. Cloud Vision is the named replacement and is **not built in R8**; its 42
   successful replies stay in `~/psm-research-bench/a01/*/run-bench/calls/cloud-vision-*.json` for a fixture if it is
   ever needed.
2. **Page-reading route: plain HTTP download from the Mac** (card 2, A), for every site. Shopify-hosted storefronts
   (the owner's shop, midcenturymobler, vgliving) are read slowly: one request per host at a time, a per-host pace
   that survives `429` with `Retry-After: 60` (see Page reading), and a page still refused after that is "could not be
   read" (R4's rule). Etsy and vntg stay unread by download; their search results still count by title. OpenAI's page
   tool is not used (it returns no page text, so it cannot satisfy the page-reader port of 08A §4).
3. **First model per stage** (card 3, yes): photo look (observe) `gpt-6.1-sol`, effort low; page reading (read)
   `gpt-6.1-terra`, effort low; condition `gpt-6.1-terra`, effort high. R9's acceptance measurement confirms or
   changes them; R10 writes the shipped defaults from the result.

## Owner answers (2026-11-13 morning)

1. **Piece 30's wood:** the corrected value is the shop's current choice "Santos Rosewood" ("Rosewood" was an older
   choice that maps to it). `docs/test-items/answer_key_corrections.json` updated.
2. **pamono's country sites are trusted:** `pamono.se`, `pamono.eu`, `pamono.co.uk`, each right after `pamono.com`
   in priority (12 entries). The measurement tool's list follows.
3. **The owner's shop addresses:** `beyovintage.se` (primary), `www.beyovintage.se`, `beyovintage.com`,
   `www.beyovintage.com` (the `.com` pair redirect to the primary), `beyovintage.myshopify.com`,
   `68c5b4-6a.myshopify.com`. A result on any of them is the owner's own shop.
4. **Designer and maker:** the owner's live shop names them "Designer" and "Manufacturer"
   (`custom.designer`, `custom.manufacturer`, as in the 42-piece file). The app reads the linked shop's fields as they
   come. The engine's shipped `attribution_keys` default `["beyo.designer"]` is the dev store's key (the store the
   app's Shopify token points at, read 2026-11-13: `beyo.designer` "Designer", no manufacturer field), so it is not
   a default that fits any other shop. Owner, same morning: make it a setting the owner points at the right fields,
   like the condition-photos field (R10 carried notes).
5. **The USD 40:** counts OpenAI API spend (the research agent's model calls on the key in `~/.psm_research.env`),
   not the Codex subscription. Lens searches are priced 0 in the measurement and draw on the SerpApi plan's monthly
   searches; page downloads are free.

## Acceptance measurement (2026-11-13, R9·U2 tasks 3–4)

Run by the orchestrator with the owner present. `accept` over `docs/test-items/beyo_test_samples_1.json` with the
owner picks (observe Sol low, read Terra low, condition Terra high; Lens via SerpApi; plain HTTP pages; 12 trusted
sites). Folder `~/psm-research-bench/a23` (pieces 1–15; the OpenAI account ran out of credit at piece 16 and
16–42 failed at observe) and `a24` (pieces 16–42, `--only`, after the owner added credit). 42 of 42 finished.
Real spend ≈ USD 12.6 (the a23 ledger shows 8.77 because refused calls are booked at their worst case).
Pieces 1–30 are the published group, 31–42 the never-published group.

**Cost and time per piece:** published median USD 0.30 (0.23–0.41), never-published median USD 0.27 (0.23–0.35);
median about 5 min (3.6–11 min). On pieces 1–15: observe 32 % (one Sol call per main photo, ~USD 0.011 each), read
26 % (the slowest step, ~166 s), compare 23 %, everything else under 5 %.

**Designer and maker** (pieces where the shop has a value; "sure" = medium or high):

| Group | Sure: right | Sure: wrong | Low: right | Low: wrong | Left open |
|---|---|---|---|---|---|
| Published (30) | 16 | 7 | 6 | 3 | 15 |
| Never published (12) | 4 | 1 | 2 | 2 | 2 |

Of the 8 sure "wrong": 3 are name forms of the right answer (Erik Wörtz/Wørts, Bertil/B. Fridhagen, AB
Stilarmatur/AB Stilarmatur, Tranås); 2 are pieces where the key may be wrong (7 maker "Architecture & Design" vs AB
Seffle möbelfabrik; 11 maker "Karl Andersson & Söner" vs NK Verkstäder; owner asked); 3 are real (piece 6 designer
and maker taken from the owner's own look-alike "model 77" listing; piece 32 maker Anderstrup vs Nova Stolefabrik).
Counting name forms as right: 23 of 28 sure answers right, 3 certainly wrong. Where the shop has **no** value,
research suggested a name 15 times (12 low, 3 sure: piece 28 designer Erling Torvits; piece 33 Niels O. Møller and
JL Møbelfabrik, which looks like the piece 6 look-alike again); these cannot be scored against the key.

**Other fields, right / wrong / open** (all 42): quantity 40/2/0; country 31/3/6; wood type 25/13/0; years 23/16/1;
detailed condition 21/19/0; width 6/18/18, depth 6/16/20, height 3/19/20, seat height 2/5/8; damage details 1/39/0
(free text, an exact-match score says little). Measurements are mostly wrong when given: they come from other
pieces' listings or from the photo, not from a statement about this piece.

**Read first:** the look-alike listing (pieces 6, 33) and the measurements are the two patterns to fix before the
defaults ship; the model comparison (task 5) has not run yet.

## Model comparison (2026-11-13, R9·U2 task 5)

`replay` of four stages on 12 pieces (2, 6, 11, 16 · 21, 24, 27, 30 · 31, 34, 37, 40: four per test group), each
with two candidate models; the recorded answers are the baseline. Out folder `~/psm-research-bench/cmp`, scored by
`scratchpad/score_cmp.py` against the key and the corrections. `replay` records no cost, so cost is from the
measurement's per-stage figures and the price ratios (Terra about half of Sol, Luna about a tenth of Terra).

| Stage | Model | Right | Wrong | Open | Note |
|---|---|---|---|---|---|
| Photo look (wood, country, years, quantity; majority of all answers) | Sol low (baseline) | 24 | 10 | 13 | ~USD 0.10 per piece |
| | Terra low | 16 | 8 | 23 | about half the cost |
| | Luna low | 4 | 1 | 42 | almost always "cannot tell" |
| Reading listings (designer, maker per page) | Terra low (baseline) | 27 | 50 | 52 | "wrong" is mostly a page about another piece |
| | Sol low | 27 | 52 | 50 | about twice the cost |
| | Luna low | 25 | 58 | 46 | about 1/8 of the cost |
| Comparing photos with a listing (117 pages) | Sol low (baseline) | — | — | — | match 25, similar 64, different 28 |
| | Terra low | — | — | — | match 23, similar 77, different 17; agrees with Sol on 99 of 117 |
| | Luna low | — | — | — | match 56, similar 33, different 28; agrees on 70 — over-matches |
| Condition (condition level fields) | Terra high (baseline) | 23 | 11 | 20 | |
| | Terra low | 13 | 7 | 34 | leaves more open |
| | Luna low | 26 | 25 | 3 | fills everything, half wrong |

**Reading:** Sol stays best for the photo look. Luna reads listings nearly as well as Terra at about 1/8 of the
cost. Terra compares photos almost like Sol at half the cost and is a little more cautious; Luna calls twice as many
pages a match, which would make the look-alike problem (pieces 6, 33) worse. Terra high stays best for condition.
Recommended per stage: photo look Sol low, reading Luna low, comparing Terra low, condition Terra high; estimated
cost per piece about USD 0.20 instead of 0.30. Small sample (12 pieces); the owner picks.

## Owner pick of models (2026-11-15)

The owner took the recommendation of "Model comparison": **observe `gpt-6.1-sol` low · read `gpt-6-luna` low ·
compare `gpt-6.1-terra` low · condition `gpt-6.1-terra` high**. The stages the comparison did not test keep the
measurement's values: labels `gpt-6.1-sol` low (2048), decide `gpt-6.1-terra` low. R10 ships these.

## Acceptance measurement 2 (2026-11-15, R12·U4 tasks 3–6)

Run by the orchestrator on the owner's go. `accept --settings docs/test-items/accept_settings_owner.json` (the six
excluded keys, the three extra shop hosts, detailed-condition table A) over the same 42 pieces with the shipped
defaults (observe Sol low as one call over up to 8 photos, read Luna low on the product extract, compare Terra low,
condition Terra high). Folder `~/psm-research-bench/a25`, script `scratchpad/accept_run3.sh`, comparison
`scratchpad/cmp_measure.py` (first = a23 pieces 1–15 + a24 pieces 16–42). 42 of 42 finished; one photo of piece 28
left out (download failed). The Mac slept about an hour during piece 9 (its 5 331 s is that pause).

**Cost and time per piece:** total USD 5.25 (first 12.54). Published median USD 0.109 (0.068–0.249; first 0.296),
never-published median USD 0.111 (0.065–0.164; first 0.266): **about 63 % less**. Time unchanged: median 326 s
published, 350 s never-published (first 297 / 288).

**Designer and maker** (pieces where the shop has a value; "sure" = medium or high), first → second:

| Group | Sure: right | Sure: wrong | Low: right | Low: wrong | Left open |
|---|---|---|---|---|---|
| Published (30) | 16 → 18 | 7 → 2 | 6 → 8 | 3 → 4 | 15 → 15 |
| Never published (12) | 4 → 2 | 1 → 1 | 2 → 0 | 2 → 2 | 2 → 6 |

All three sure "wrong" are name forms of the right answer (12 ERIK WÖRTZ / Erik Wørts, 20 H.W. Klein / HENRY W.
KLEIN, 34 Glasmäster / Glas Mäster): **no sure answer is really wrong** (first: 3). Piece 6 (the own-shop look-alike)
is now right (Arne Hovmand-Olsen, Mogens Kold). Where the shop has **no** value, research suggested a name 6 times
(first 15): 28 Erling Torvits (medium, two auction pages, as before) and 33 J.L. Møllers (medium, two pamono pages of
a model 71 chair; no longer the own-shop look-alike; the owner is checking 33). The never-published group is more
cautious (open 2 → 6; newly open there: 36 and 38 designer, 32 and 36 maker). Newly open among published: 7, 11
(both pieces where the key may be wrong), 23, 24, 26 maker.
Per field right/wrong/open: designer 21/14/7 → 26/8/8, maker 18/14/10 → 22/7/13.

**Other fields, right / wrong / open, first → second:** wood type 25/13/0 → 29/9/0 (7 Santos Rosewood, 15 Walnut,
26 and 31 Teak+Oak, 39 Cherry now right; 17 now wrong); country 31/3/6 → 29/1/10 (more left open); years 23/16/1 →
25/14/1; quantity 40/2/0 unchanged; condition levels about the same (scratch 25/3 → 22/6, veneer 2/3 → 3/7).
Dimensions, weight and damage details: not researched (0 written).

**Detailed condition 21/19/0 → 0/0/40.** By design of R11 (`rules::derived_condition`): the grade is derived only
when every condition level in the form has a value, and Function, Odor, Stability (and often Missing parts) are never
answered from photos (open on 10 of 10 in both measurements). In the app the person fills those, then the grade is
derived; in the tool nobody does. Owner question: should an empty level count as "None" for the grade.

**Task 5 checks:** excluded keys written: none. Attribution above low: every one has two or more trusted pages or a
label read (pieces 1, 17, 25, 34, 39 are `high` from a label read; the score file lists no `sources` for them).
Spend USD 5.25 within the USD 20 ceiling.

**Finding (both measurements): research fills shop fields that are not facts about the piece.** Written by
research in a25 (similar counts in a23/a24): `custom.price_st` (6, prices of other listings), `custom.compare_at_price`
(4), `custom.reserved` (2), `reviews.rating_count` (4), `custom.location` and `app.item_location` (storage place),
`custom.link` (11, a listing URL), `custom.utstallning`, `custom.dimensionss` (6, a dimensions text field not in the
excluded list), `custom.etsy_description` (42, description text), one Shopify related-products field. All low or
medium suggestions, none checked. Fix without a build: add them to "fields research leaves alone"; the owner decides
whether research should instead fill only a list of chosen fields.

