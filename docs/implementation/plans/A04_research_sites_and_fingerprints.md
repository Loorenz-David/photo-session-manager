---
phase: A04_research_sites_and_fingerprints
title: Research core, part 2 — which website a page belongs to (the shop, a trusted site, anything else), when two links are the same page, which pages are read first, and the fingerprint that tells whether a piece of work was already done
difficulty: high
projection: yes
depends_on: [A03_research_values_and_sameness]
---
# A04 — Sites, page identity, reading order (08B MC-A3) and unit fingerprints (MC-A7 `F`)

## Goal
Pure functions for MC-A3 (host, trusted entries, site class and identity, page identity, the item's own listing,
reading order) and for the fingerprint of MC-A7. A wrong answer here counts one dealer as two sites, or redoes paid
work. **Not in this phase:** the settings key and its UI (A13, A19), reading pages (A12, A17), the shop's hosts from
the shop (A17: here they are input data).

## Read first
`master_plan_research.md` §3, §6.2, §6.3, §6.5 · book `14` (MA11, MA14), `08A` §6 (Trusted sites), `08B` **MC-A3**
(all), **MC-A7** (Fingerprint paragraph) · evidence `research_providers.md` (the nine trusted sites) · code
`crates/core/src/mechanisms/research/` (A03), `crates/core/src/mechanisms/hash.rs` (SHA-256 use in core).

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/core/src/mechanisms/research/{sites.rs, fingerprint.rs}`. Edited: `mechanisms/research/mod.rs`,
`crates/core/Cargo.toml` and `Cargo.lock` (`url`, `idna`: already in the lock file through `reqwest`), own tracker row.

## Core mechanisms (unit tests allowed; one inline test per row, `mc_a3_<slug>` / `mc_a7_<slug>`)
`sites::{host, TrustedSites::{parse_entry, add, remove, reorder, matches}, site_of, page_identity, is_own_listing,
reading_order}` · `fingerprint::{canonical, fingerprint}`.

## Ordered tasks
1. `host(url)`: absolute `http`/`https` only; lowercase; IDNA to ASCII; trailing dot stripped; port ignored.
2. `TrustedSites` (ordered, disjoint entries): `parse_entry` (a domain or a URL → host with one leading `www.`
   removed; must contain a dot), `add` (refused when equal to, a parent of, or a subdomain of an existing entry or
   a shop host; the refusal text is `already covered by <the entry or shop host>`), `remove`, `reorder`, `matches`
   (`h = e` or `h` ends with `"." + e`).
3. `site_of(url, shop_hosts, trusted)`: first match of shop, trusted, untrusted; IP-literal hosts are never trusted.
4. `page_identity(url)`: MC-A3 `U`.
5. `is_own_listing(url, &OwnListing {url, path_tail}, shop_hosts)`: true when `U(url)` equals `U` of the item's
   storefront URL, or when the page is on a shop host and the last segments of its path equal `path_tail` (the
   adapter supplies the segments, for example `products`, `<handle>`; a locale prefix before them is allowed). With
   no listing data it is never true.
6. `reading_order(candidates, shop_hosts, trusted, own)`: discard invalid URLs, drop the own listing (kept apart for
   the trace), merge equal `U` keeping the best rank, order: shop by rank; trusted by entry priority, then rank;
   untrusted by rank; ties by `U` ascending in byte order.
7. `fingerprint::canonical(&Json)`: keys sorted by UTF-8 bytes, no insignificant white space, strings NFC, integers
   as integers, arrays in order, **a float is an error**. `fingerprint(stage, unit)` = lowercase hex SHA-256 of the
   canonical form of `{"stage": n, "unit": …, "v": 1}`. `RESEARCH_FINGERPRINT_VERSION = 1`.

## Criteria (7 criteria, 28 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `host` | `HTTPS://WWW.Example.COM:8443/x` | `www.example.com` | MC-A3 |
| C1(b) | `host` | `https://example.com./a` | `example.com` | MC-A3 |
| C1(c) | `host` | `https://bücher.example/` | `xn--bcher-kva.example` | MC-A3 |
| C1(d) | `host` | `ftp://example.com/`, `example.com/x`, `mailto:a@example.com` | none, none, none | MC-A3 |
| C2(a) | `parse_entry` | `https://www.Pamono.com/x`; `auktion.sweakronan.se`; `localhost` | `pamono.com`; `auktion.sweakronan.se`; refused | MC-A3 |
| C2(b) | `add`, list `[etsy.com]` | `shop.etsy.com`; `etsy.com` | both refused with `already covered by etsy.com` | MC-A3 |
| C2(c) | `add`, list `[shop.etsy.com]` | `etsy.com` | refused with `already covered by shop.etsy.com` | MC-A3 |
| C2(d) | `add`, shop hosts `[beyo.example]` | `beyo.example`; `www.beyo.example`; `example` | refused `already covered by beyo.example`; the same; refused (no dot) | MC-A3 |
| C2(e) | `reorder`, `remove` | `[a.example, b.example, c.example]`, move `c.example` first, remove `a.example` | `[c.example, b.example]`, priorities 0 and 1 | MC-A3 |
| C3(a) | `site_of`, shop `[beyo.example]`, trusted `[etsy.com, auktion.sweakronan.se]` | `https://beyo.example/p`; `https://en.beyo.example/p` | both class shop, identity `shop` | MA11 |
| C3(b) | same | `https://www.etsy.com/l/1`; `https://shop.etsy.com/l/1` | both trusted, identity `etsy.com` | MA11 |
| C3(c) | same | `https://notetsy.com/x`; `https://sweakronan.se/x` | untrusted `notetsy.com`; untrusted `sweakronan.se` | MA11 |
| C3(d) | same | `https://www.dealer.example/x`; `http://192.168.1.5/x` | untrusted `dealer.example`; untrusted (never trusted, even with `192.168.1.5` typed as an entry: `parse_entry` refuses an IP literal) | MC-A3 |
| C4(a) | `page_identity` | `HTTP://WWW.Example.com:80/a?utm_source=x&b=2&a=1#frag` | `http://www.example.com/a?a=1&b=2` | MC-A3 |
| C4(b) | `page_identity` | `https://example.com`; `https://example.com:443/`; `https://example.com:8443/` | first two equal `https://example.com/`; third keeps `:8443` | MC-A3 |
| C4(c) | `page_identity` | a URL carrying each of `utm_campaign`, `srsltid`, `gclid`, `fbclid`, `mc_cid`, `mc_eid` and `id=7` | only `id=7` remains | MC-A3 |
| C4(d) | `page_identity` | `?b=1&a=2&a=1` | query `a=1&a=2&b=1` | MC-A3 |
| C5(a) | `is_own_listing`, own URL `https://beyo.example/products/oak-table`, tail `[products, oak-table]` | `https://beyo.example/products/oak-table?utm_source=x`; `https://beyo.example/en/products/oak-table` | own; own | MA11 (R-IA9) |
| C5(b) | same | `https://beyo.example/products/oak-table-2`; `https://etsy.com/products/oak-table` | not own; not own | MA11 |
| C5(c) | no listing data | any URL | not own | MC-A3 |
| C6(a) | `reading_order`, shop `[beyo.example]`, trusted `[b.example, a.example]` | ranks: untrusted 1, `a.example` 2, `b.example` 3, shop 4 | shop, `b.example`, `a.example`, untrusted | MC-A3 |
| C6(b) | same | two pages on `a.example` at ranks 5 and 2 | rank 2 before rank 5 | MC-A3 |
| C6(c) | same | two untrusted pages with equal rank | the smaller `U` first | MC-A3 |
| C6(d) | same | the same page twice (once with `?utm_source=x`) at ranks 4 and 1; the own listing at rank 1; `not a url` | one entry at rank 1; own listing absent and reported apart; invalid entry discarded | MC-A3 |
| C7(a) | `fingerprint` | stage 2, unit `{"rev": 3, "cover": 7}` | equals SHA-256 of the bytes `{"stage":2,"unit":{"cover":7,"rev":3},"v":1}` | MC-A7 |
| C7(b) | `fingerprint` | the same unit with a string in NFD and in NFC | equal fingerprints | MA14 |
| C7(c) | `fingerprint` | units `{"ids": [1,2]}` and `{"ids": [2,1]}` | different fingerprints | MC-A7 |
| C7(d) | `canonical` | a unit containing `1.5` | error | MC-A7 |

## Delegated choices
Whether `url`/`idna` are used or the parsing is written over `idna` alone · the type of a rank (an integer) · how
the own listing is reported apart (a second return value).

## Manual try-out
Nothing to see in the app yet. `cargo test -p psm-core mechanisms::research::sites` shows the site rules by name.

## Notes
- Rule 17: `idna` and `url` own two behaviours these rows depend on (IDNA mapping of `bücher`, what counts as an
  absolute URL). The rows state standard results; the projection confirms them against the installed versions
  (`idna 1.1.0`, `url 2.5.8` in the local registry) before the implementer starts.
- Hazard: suffix matching without the dot (`notetsy.com`) is the classic defect; C3(c) is its row.
- Review mode: `cargo test -p psm-core mechanisms::research`, clippy, fmt.

## Review log
