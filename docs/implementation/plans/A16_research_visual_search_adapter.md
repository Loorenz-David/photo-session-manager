---
phase: A16_research_visual_search_adapter
title: The real visual search — the app sends the cover photo's reduced copy to the search service the owner picked and gets back the matching pages in order
difficulty: normal
projection: yes
depends_on: [A11_research_call_gateway, A01_research_probes_and_search_benchmark]
blocked_on: the owner's pick of the visual-search route (evidence doc, "Owner picks", after A01)
---
# A16 — Visual-search adapter, the one the owner picked (08A §4, §12A.1; 08B MC-A3, MC-A8, MC-A9; RESEARCH-VERIFY-6, -7)

## Goal
Implement `VisualSearchProvider` for **one** route: Google Lens through SerpApi, or Google Cloud Vision Web
Detection. The port stays as it is, so the other can be added later. **Not in this phase:** the other route, page
reading (A17), ranking or site rules (core, A04).

## Task 0 (gate)
Read `docs/intention/evidence/research_providers.md` §Owner picks. If no visual-search route is recorded there,
report `STATE: BLOCKED` (reason: `owner pick missing: visual-search route`) and change nothing. Otherwise build the
variant named there and delete the other variant's section from this plan's criteria in the Review log entry
("variant built: …").

## Fold before dispatch (coordinator)
After the pick: strike the other variant below, and check the fixture names against the fixtures README.

## Read first
`master_plan_research.md` §3, §6.1, §6.6, §7.4 · book `14` (MA12, MA15, MA16), `08A` §4, `08B` MC-A3 (candidates and
rank), MC-A5 (only a `ResearchImage` reaches a port), MC-A8 (`image_search_call`), MC-A9 (redaction: `api_key`,
`key`, `x-goog-api-key`), §Grounding owed (RESEARCH-VERIFY-6, -7) · evidence doc (A01's sections) · the fixtures
folder of the picked route · code `crates/integrations/src/research/{http.rs, openai/}` (A15, the pattern),
`ports/research.rs`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/integrations/src/research/visual_search/{mod.rs, <route>.rs}`,
`crates/testkit/tests/suite/research/a16_research_visual_search_adapter.rs` (+ `mod` line). Edited:
`crates/integrations/src/research/mod.rs`, `crates/integrations/src/lib.rs` (built when
`research.providers.visual_search` names it and the key exists), own tracker row.

## Core mechanisms
None.

## Ordered tasks
1. The request(s) of the route exactly as A01 recorded them (SerpApi: upload, then search with the returned id;
   Cloud Vision: one annotate call with the image inline).
2. Candidates: the route's page results in the route's own order → `Candidate {url, title, rank}` with rank 1 for
   the first; results without a page URL are dropped.
3. Usage: one `image_search_call` per search (and what A01 recorded for an upload).
4. Failures → `CallFailure` from the captured error replies; the key's place in the request declared as secret.
5. If the route has a size limit below the 1024 copy's size (A01, RESEARCH-VERIFY-7), the adapter declares
   `max_image_bytes`, and the stage asks `ResearchImages::render` for a smaller long edge; the adapter never
   re-encodes an image itself.

## Criteria (6 criteria, 7 rows)
Boundary: `VisualSearchProvider::search` over `FakeResearchHttp` replaying the route's captures.
| Row | Boundary | Input (fixture, probe) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | candidates | the route's search reply cut to 3 results (RESEARCH-VERIFY-6) | three candidates with the capture's URLs and titles, ranks 1, 2, 3 in the capture's order | MC-A3 |
| C1(b) | candidates | the same reply with the results array emptied | no candidate; not a failure | MC-A3 |
| C2(a) | usage | the search reply (and, for SerpApi, the upload reply) | the usage A01 recorded for one search: `image_search_call` × the recorded count | MA15 |
| C3(a) | secrets | any search | the `Exchange` shows the key's query parameter or header as `<redacted>`; the key value appears nowhere in it, including in a URL | MA16 (RESEARCH-VERIFY-7) |
| C4(a) | errors | the route's captured error reply (invalid key) | `CallFailure` with the capture's status; `classify` gives `key` | MC-A9 |
| C5(a) | image | a `ResearchImage` of 1024 | the bytes sent equal the stored copy's bytes (same SHA-256); for a route with a size limit, a copy within the limit is requested from the renderer and sent | MA12 |
| C6(a) | end to end | A12's full-run script with this adapter replaying the capture, the fake model and page reader | the page reader is asked for the capture's pages in MC-A3 order; the call record of the search has one outbound row for the cover | MA3 |

**Variant SerpApi (Lens).** Fixtures `serpapi_lens/`: upload reply, search reply, error reply. C2(a) counts what
A01 measured per upload and per search. C5(a) uses the limit A01 measured.
**Variant Cloud Vision.** Fixtures `cloud_vision/`: web detection reply, error reply. Candidates come from the
pages-with-matching-images list in its order; C5(a) has no size step unless A01 found a limit.

## Delegated choices
Which of the route's result lists beyond the page list are used (none, unless A01's report shows they add correct
pages; record it) · adapter id spelling.

## Manual try-out
Nothing in the app until the key can be entered (A19). The capture this adapter is tested with is one of the
owner's own pieces from A01.

## Notes
- The Lens-through-a-vendor route's legal position is in the evidence doc; the adapter must stay replaceable, which
  the port guarantees.
- Review mode: this phase's module, `research::a12…::`, clippy, fmt.

## Review log
