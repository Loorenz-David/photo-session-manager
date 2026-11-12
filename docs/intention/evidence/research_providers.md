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
