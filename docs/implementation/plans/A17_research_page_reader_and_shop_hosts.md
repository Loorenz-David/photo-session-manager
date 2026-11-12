---
phase: A17_research_page_reader_and_shop_hosts
title: Reading a listing page for real, by the route the probes showed to work, and knowing the shop's own web addresses so the shop counts as the best source and an item is never used as evidence for itself
difficulty: high
projection: yes
depends_on: [A12_research_stages_and_decide, A01_research_probes_and_search_benchmark]
blocked_on: the page-reading route pick (evidence doc, "Owner picks", after A01)
---
# A17 — Page reader by the chosen route; shop hosts, own listing and unit names from the shop (08A §4; 08B MC-A3, MC-A2, MC-A9, MC-A11; RESEARCH-VERIFY-8, -9)

## Goal
(1) Implement `PageReader` by the route recorded after A01: **plain HTTP fetch from Rust**, or **the model
provider's page-open tool restricted to the candidate's domain**. (2) Implement `CommerceAdapter::public_listing`
for Shopify (the shop's public hosts, the item's storefront URL and path tail) and `unit_names`, so MC-A3's shop
class and own-listing exclusion and MC-A2's unit conversion work with the real shop. **Not in this phase:** the
other route; the extraction of what a page states (that is stage 3's model call, A12/A15).

## Task 0 (gate)
Read the evidence doc §Owner picks. No page-reading route recorded → `STATE: BLOCKED` (reason: `owner pick
missing: page-reading route`), change nothing. If the pick is "plain HTTP for some sites, the model tool for
others", build the HTTP route here and report the rest as a decision for the orchestrator.

## Fold before dispatch (coordinator)
Strike the other variant; check fixture names against the fixtures README; copy the per-site "readable or not"
table's outcome into C3's input cell.

## Read first
`master_plan_research.md` §3, §6.1, §6.6 · book `14` (MA11, MA16), `08A` §4, `08B` **MC-A3** (shop hosts, site of a
page, own listing), MC-A2 (units), MC-A5 (third-party images), MC-A8 (`page_fetch`, `web_search_call`), MC-A9,
MC-A11 (replay of stage 3 through a model-side tool), §Grounding owed (RESEARCH-VERIFY-8, -9) · evidence doc
(A01's page-reading table; the shop probe) · fixtures `pages/`, `shop/` · code
`crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, type_map.rs (unit codes), fixtures/}`,
`crates/core/src/ports/commerce.rs` (A12's two methods), `crates/testkit/src/fake_transport.rs`,
`crates/integrations/src/research/http.rs`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/integrations/src/research/page_reader/{mod.rs, <route>.rs}`,
`crates/testkit/tests/suite/research/a17_research_page_reader_and_shop_hosts.rs` (+ `mod` line), a sanitized
Shopify fixture for the hosts query if A01's capture needs trimming. Edited:
`crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, type_map.rs}`, `crates/integrations/src/research/
mod.rs`, `crates/integrations/src/lib.rs`, `crates/integrations/Cargo.toml` (an HTML parser, for the HTTP route;
version verified and recorded), own tracker row.

## Core mechanisms
None.

## Ordered tasks
1. **Variant HTTP.** GET the candidate with the timeout `call_timeout_seconds.other`, a generic client name, a
   byte limit, redirects followed and the final URL reported; only `http`/`https`; **never** a host that is an IP
   literal, `localhost`, or resolves to a loopback, private or link-local address (candidates come from search
   results, so the reader must not be steerable into the local network). Extract the title, the visible text and
   the main images (the routes A01 found: Open Graph, JSON-LD, the largest content images); download at most a
   fixed number of images within a byte limit and hand them to the gateway's store (third-party images are never
   outbound rows). Usage: one `page_fetch`. A status or a page A01 recorded as unreadable → `CallFailure` kind
   `Http` with that status; the stage records `could not be read`.
   **Variant model tool.** One model call with the page-open tool restricted to the candidate's host; the reply's
   page text and, if A01 found it is returned, the final URL; usage from the reply (`web_search_call` and tokens).
   No images unless A01 found the tool returns them.
2. `ShopifyAdapter::public_listing(external_id)`: the hosts and, for a linked product, its storefront URL and the
   path tail (`products`, handle), from the query A01 recorded. An unlinked item → hosts only.
3. `ShopifyAdapter::unit_names()`: the seven neutral names mapped to this adapter's unit codes (its own constants
   in `type_map.rs`).

## Criteria (6 criteria, 8 rows)
Boundary: `PageReader::read` over `FakeResearchHttp`; `CommerceAdapter::public_listing` over `FakeTransport`.
| Row | Boundary | Input (fixture, probe) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | read | the shop's own listing page capture (`pages/`, RESEARCH-VERIFY-8) | the reply's title equals the product title in the test file for that piece; its text contains the designer named in that piece's metafields; at least one image is returned | MC-A3 |
| C1(b) | redirect | a capture (or a `FakeResearchHttp` script built from the recorded redirect headers) where the candidate redirects to another URL | `final_url` is the redirect target | MC-A3 (RESEARCH-VERIFY-8) |
| C2(a) | usage | one read | exactly the unit of the route: one `page_fetch`, or the model reply's recorded counts | MA15 |
| C3(a) | unreadable | the status A01 recorded for a site that refused the route | `CallFailure` with that status; in A12's script the page becomes a `read` step `could not be read` and does not count toward `max_pages` | MC-A3 (R-IA10) |
| C4(a) | shop hosts | the hosts-and-product capture (`shop/`, RESEARCH-VERIFY-9) | `hosts` = the capture's primary, other and platform domains; `item_url` = its storefront URL; `item_path_tail` = `[products, <handle>]` | MA11 |
| C4(b) | unlinked item | no external id | hosts only; no item URL, empty tail | MC-A3 |
| C5(a) | unit names | `unit_names()` then A03's `map_output` for a dimension definition of this adapter | `{value: 10, unit: "in"}` is stored as 25.4 in the definition's first unit code | MC-A2 |
| C6(a) | end to end | A12's script with the real Shopify adapter over `FakeTransport` and this page reader over captures; a candidate that is the item's own listing with a locale prefix in its path | it is not read and is no evidence; a page on another product of the shop counts as site class shop | MA11 (R-IA9) |


## Delegated choices
The HTML parser crate (pin recorded) · the image and byte limits (state them) · text extraction details.
Not delegated: the address guard of task 1 (it has no criterion row because no contract states it yet;
`master_plan_research.md` §8 lists it for the orchestrator; the reviewer checks it by reading), one request per page,
no JavaScript execution.

## Manual try-out
Nothing in the app until A19. The page this reader is tested with is one of the owner's own listings.

## Notes
- A site that needs scripts to show its text cannot be read by the HTTP route; that is an outcome
  (`could not be read`), not a defect. A01's table says which sites these are.
- Replay (A22): a stage 3 call made through a model-side tool cannot reproduce the page content; the adapter marks
  such calls in the record so the tool can say so.
- Review mode: this phase's module, `p12_shopify_adapter::`, `research::a12…::`, clippy, fmt.

## Review log
