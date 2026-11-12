---
phase: R3_research_rules
title: Research rules — how an answer becomes a valid form value, which website a page belongs to, how sure a suggestion is, what a call costs and what to do when one fails, and what research is allowed to write over
difficulty: high
projection: no
depends_on: [R2_research_foundation]
---
# R3 — Rules: the pure rules of research, and what research may write

**Tracker:** NOT_STARTED · 2026-11-12 · planner · —

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
The pure rule modules of the research crate (`psm_research::rules`: value mapping, sites, fingerprints, confidence, cost, call rules) and the write path (`psm_research::writes`: MC-A1's write table, cleared set and lowering, on the app's store). All in `crates/research`, with their tests in that crate. **Not here:** any provider call, run or stage (R4), the photographer's actions (R5).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 5 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R3_research_rules.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Model output to a stored value | 5 | 18 | no |
| U2 | Sites, page identity, reading order, fingerprints | 7 | 28 | no |
| U3 | Confidence derivation | 6 | 32 | no |
| U4 | Cost, stop rule, the day; error classes, attempts, redaction | 7 | 29 | no |
| U5 | What research may write (the write table) | 6 | 25 | no |
| **Total** | | **31** | **132** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`.

## Unit 1 — Model output to a stored value

**Unit id:** `R3·U1` · **Replaces:** A03 criteria C2–C6 · **Changes app files:** no

### Goal
The mapping of a model's output to a stored value for each core type, and what a listing "states" for a key (MC-A2), as pure functions in the research crate. **Not in this unit:** name key and sameness (core, R2·U1), the write table (U5), site rules (U2), confidence (U3).

### What changed from the superseded plan (this wins over any older wording below)
- Name key and sameness are no longer built here: they are `psm_core::mechanisms::{name_key, sameness}` (R2·U1; rows A03·C1 and A03·C7 are there). This unit builds `value_map` only, in `crates/research/src/rules/value_map.rs`.
- There is no `UnitNames` parameter: unit names come from `definition.validations.unit_names` (as in R2·U1). Row A03·C5(d)'s input is worded accordingly; its outcome is unchanged.

### Read first
`master_plan.md` §3, §6.1, §6.9, §7.2 · `master_plan_research.md` §3, §6.1–§6.3, §6.5 · book `01`, `14` (MA6, MA11),
`08A` §5 (Field rules), `08B` **MC-A2** (all), **MC-A1** (cleared set: the sameness table), `05A` MC-9 · code
`crates/core/src/mechanisms/values.rs` (`canonicalize`, `canonical_json`, `canonical_decimal`),
`crates/core/src/domain/attribute.rs` (`CoreType`, `Definition`, `Definition::units`).

### Files
New: `crates/research/src/rules/{mod.rs, value_map.rs}`. Edited: `crates/research/src/lib.rs`, `crates/research/Cargo.toml`.

### Core mechanisms (the only functions with unit tests)
`rules::value_map::{map_output, statement, page_statement}` (one inline test per row).

### Ordered tasks
1. `map_output(&Definition, &Json) -> Mapped`: the MC-A2 table. The stored form is always produced by the existing `canonicalize`, so a mapped value is valid by construction. Name comparison uses `psm_core::mechanisms::name_key`. For `dimension` and `weight`, the seven neutral unit names (`mm, cm, m, in` / `g, kg, lb`) are mapped to the definition's unit codes by `definition.validations.unit_names`; conversion into the definition's first listed unit uses exact integer arithmetic (1 in = 25.4 mm, 1 lb = 453.59237 g), rounding half-even to one decimal place; **no floating point**. A unit name that is not one of the seven, or a first unit with no `unit_names` entry, fails.
2. `statement` (one stated text → `Stated::Mapped(value)` or `Stated::Unmapped(name key)`) and `page_statement` (all texts one page states for a key → the single statement, or none when it states two values that are not the same under `psm_core::mechanisms::sameness::same`).

### Criteria (5 criteria, 18 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A03·C2(a) | `map_output`, choice `[Oak, Teak]` | `"Teak"` | `Teak` | MA6 |
| A03·C2(b) | choice `[Oak, Teak]` | `"teak "` | `Teak` (unique choice with the same name key) | MC-A2 |
| A03·C2(c) | choice `[Oak, oak]` | `"oak"`; `"OAK"` | `oak` (byte-equal wins); fail (two choices share the key) | MC-A2 |
| A03·C2(d) | choice `[Oak, Teak]` | `"Pine"`; `12` | fail; fail | MA6 |
| A03·C3(a) | multi_choice `[Oak, Teak, Rosewood]` | `["teak", "Oak", "Plastic"]` | `["Oak","Teak"]` (definition order), dropped `["Plastic"]` | MC-A2 |
| A03·C3(b) | multi_choice | `["Plastic"]`; `[]`; `"Teak"` | fail; fail; fail | MA6 |
| A03·C4(a) | text | `"  Oak table "`; `"   "` | canonical `Oak table`; fail | MC-A2 |
| A03·C4(b) | url | `"https://a.example/x"`; `"ftp://a.example"`; `"a.example/x"` | stored; fail; fail | MC-A2 |
| A03·C4(c) | integer | `12`; `"12"`; `12.5`; `"1e3"`; `"1,200"` | `12`; `12`; fail; fail; fail | MC-A2 |
| A03·C4(d) | decimal | `"12.5"`; `12.5`; `"12,5"` | canonical `12.5`; canonical `12.5`; fail | MC-A2 |
| A03·C4(e) | boolean, date, color | `true`; `"2026-02-30"`; `"2026-02-28"`; `"#A1B2C3"`; `"red"` | `true`; fail; stored; `#a1b2c3`; fail | MC-A2 |
| A03·C5(a) | dimension, definition's first unit = the code for `cm` | `{value: 10, unit: "in"}` | value `25.4` in that unit code | MC-A2 |
| A03·C5(b) | dimension, first unit `cm` | `{value: "2.5", unit: "mm"}`; `{value: "3.5", unit: "mm"}` | `0.2`; `0.4` (half-even at one decimal) | MC-A2 |
| A03·C5(c) | weight, first unit `kg` | `{value: 1, unit: "lb"}` | `0.5` | MC-A2 |
| A03·C5(d) | dimension | unit `"ft"`; a definition whose `validations.unit_names` has no entry for its first unit; `{value: 10}` | fail; fail; fail | MA6 |
| A03·C6(a) | `statement`, choice `[Hans J. Wegner]` | stated `Hans J Wegner` | `Mapped("Hans J. Wegner")` | MC-A2 |
| A03·C6(b) | `statement`, same definition | stated `Unknown Maker` | `Unmapped("unknown maker")` | MC-A2 (R-IA24) |
| A03·C6(c) | `page_statement`, single-valued key | a page stating `Wegner` and `Mogensen`; a page stating `Wegner` and `WEGNER` | none; one statement | MC-A2 |

### Delegated choices
Internal number representation for the unit conversion (integers only) · error text inside `Mapped::Fail` (it is shown in the trace; one short sentence).

### Notes
- Hazard: an empty name key (C1(g)) must never match a choice whose key is also empty; `map_output` fails on it.
- A mapped choice is byte-equal to one of the definition's current choices (MA11); the test for C2(b) asserts the
  returned string, not a normalised form.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib rules::value_map` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 2 — Sites, page identity, reading order, fingerprints

**Unit id:** `R3·U2` · **Replaces:** A04 (all rows) · **Changes app files:** no

### Goal
Pure functions for MC-A3 (host, trusted entries, site class and identity, page identity, the item's own listing,
reading order) and for the fingerprint of MC-A7. A wrong answer here counts one dealer as two sites, or redoes paid
work. **Not in this phase:** the settings key and its UI (R4·U4, R6·U2), reading pages (R4·U3, R8·U3), the shop's hosts from
the shop (R8·U3: here they are input data).

### Read first
`master_plan_research.md` §3, §6.2, §6.3, §6.5 · book `14` (MA11, MA14), `08A` §6 (Trusted sites), `08B` **MC-A3**
(all), **MC-A7** (Fingerprint paragraph) · evidence `research_providers.md` (the nine trusted sites) · code
`crates/research/src/rules/` (R3·U1), `crates/core/src/mechanisms/hash.rs` (SHA-256 use in core).

### Files
New: `crates/research/src/rules/{sites.rs, fingerprint.rs}`. Edited: `crates/research/src/rules/mod.rs`, `crates/research/Cargo.toml` and `Cargo.lock` (`url`, `idna`: already in the lock file through `reqwest`).

### Core mechanisms (the only functions with unit tests)
`sites::{host, TrustedSites::{parse_entry, add, remove, reorder, matches}, site_of, page_identity, is_own_listing,
reading_order}` · `fingerprint::{canonical, fingerprint}`.

### Ordered tasks
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

### Criteria (7 criteria, 28 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A04·C1(a) | `host` | `HTTPS://WWW.Example.COM:8443/x` | `www.example.com` | MC-A3 |
| A04·C1(b) | `host` | `https://example.com./a` | `example.com` | MC-A3 |
| A04·C1(c) | `host` | `https://bücher.example/` | `xn--bcher-kva.example` | MC-A3 |
| A04·C1(d) | `host` | `ftp://example.com/`, `example.com/x`, `mailto:a@example.com` | none, none, none | MC-A3 |
| A04·C2(a) | `parse_entry` | `https://www.Pamono.com/x`; `auktion.sweakronan.se`; `localhost` | `pamono.com`; `auktion.sweakronan.se`; refused | MC-A3 |
| A04·C2(b) | `add`, list `[etsy.com]` | `shop.etsy.com`; `etsy.com` | both refused with `already covered by etsy.com` | MC-A3 |
| A04·C2(c) | `add`, list `[shop.etsy.com]` | `etsy.com` | refused with `already covered by shop.etsy.com` | MC-A3 |
| A04·C2(d) | `add`, shop hosts `[beyo.example]` | `beyo.example`; `www.beyo.example`; `example` | refused `already covered by beyo.example`; the same; refused (no dot) | MC-A3 |
| A04·C2(e) | `reorder`, `remove` | `[a.example, b.example, c.example]`, move `c.example` first, remove `a.example` | `[c.example, b.example]`, priorities 0 and 1 | MC-A3 |
| A04·C3(a) | `site_of`, shop `[beyo.example]`, trusted `[etsy.com, auktion.sweakronan.se]` | `https://beyo.example/p`; `https://en.beyo.example/p` | both class shop, identity `shop` | MA11 |
| A04·C3(b) | same | `https://www.etsy.com/l/1`; `https://shop.etsy.com/l/1` | both trusted, identity `etsy.com` | MA11 |
| A04·C3(c) | same | `https://notetsy.com/x`; `https://sweakronan.se/x` | untrusted `notetsy.com`; untrusted `sweakronan.se` | MA11 |
| A04·C3(d) | same | `https://www.dealer.example/x`; `http://192.168.1.5/x` | untrusted `dealer.example`; untrusted (never trusted, even with `192.168.1.5` typed as an entry: `parse_entry` refuses an IP literal) | MC-A3 |
| A04·C4(a) | `page_identity` | `HTTP://WWW.Example.com:80/a?utm_source=x&b=2&a=1#frag` | `http://www.example.com/a?a=1&b=2` | MC-A3 |
| A04·C4(b) | `page_identity` | `https://example.com`; `https://example.com:443/`; `https://example.com:8443/` | first two equal `https://example.com/`; third keeps `:8443` | MC-A3 |
| A04·C4(c) | `page_identity` | a URL carrying each of `utm_campaign`, `srsltid`, `gclid`, `fbclid`, `mc_cid`, `mc_eid` and `id=7` | only `id=7` remains | MC-A3 |
| A04·C4(d) | `page_identity` | `?b=1&a=2&a=1` | query `a=1&a=2&b=1` | MC-A3 |
| A04·C5(a) | `is_own_listing`, own URL `https://beyo.example/products/oak-table`, tail `[products, oak-table]` | `https://beyo.example/products/oak-table?utm_source=x`; `https://beyo.example/en/products/oak-table` | own; own | MA11 (R-IA9) |
| A04·C5(b) | same | `https://beyo.example/products/oak-table-2`; `https://etsy.com/products/oak-table` | not own; not own | MA11 |
| A04·C5(c) | no listing data | any URL | not own | MC-A3 |
| A04·C6(a) | `reading_order`, shop `[beyo.example]`, trusted `[b.example, a.example]` | ranks: untrusted 1, `a.example` 2, `b.example` 3, shop 4 | shop, `b.example`, `a.example`, untrusted | MC-A3 |
| A04·C6(b) | same | two pages on `a.example` at ranks 5 and 2 | rank 2 before rank 5 | MC-A3 |
| A04·C6(c) | same | two untrusted pages with equal rank | the smaller `U` first | MC-A3 |
| A04·C6(d) | same | the same page twice (once with `?utm_source=x`) at ranks 4 and 1; the own listing at rank 1; `not a url` | one entry at rank 1; own listing absent and reported apart; invalid entry discarded | MC-A3 |
| A04·C7(a) | `fingerprint` | stage 2, unit `{"rev": 3, "cover": 7}` | equals SHA-256 of the bytes `{"stage":2,"unit":{"cover":7,"rev":3},"v":1}` | MC-A7 |
| A04·C7(b) | `fingerprint` | the same unit with a string in NFD and in NFC | equal fingerprints | MA14 |
| A04·C7(c) | `fingerprint` | units `{"ids": [1,2]}` and `{"ids": [2,1]}` | different fingerprints | MC-A7 |
| A04·C7(d) | `canonical` | a unit containing `1.5` | error | MC-A7 |

### Delegated choices
Whether `url`/`idna` are used or the parsing is written over `idna` alone · the type of a rank (an integer) · how
the own listing is reported apart (a second return value).

### Notes
- Rule 17: `idna` and `url` own two behaviours these rows depend on (IDNA mapping of `bücher`, what counts as an
  absolute URL). The rows state standard results; the projection confirms them against the installed versions
  (`idna 1.1.0`, `url 2.5.8` in the local registry) before the implementer starts.
- Hazard: suffix matching without the dot (`notetsy.com`) is the classic defect; C3(c) is its row.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib rules::sites` · `cargo test -p psm-research --lib rules::fingerprint` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 3 — Confidence derivation

**Unit id:** `R3·U3` · **Replaces:** A05 (all rows) · **Changes app files:** no

### Goal
The pure function that turns recorded evidence into a value and a confidence for one field, for each of the four
field classes (attribution, measured, visible, condition), exactly as MC-A4's tables say. This is the mechanism
ranked first for silent failure: a false designer at `medium` is published unchecked. **Not in this phase:**
gathering the evidence (R4·U3), writing the value (R3·U5), the reason text (R4·U3, model-written).

### Read first
`master_plan_research.md` §3, §6.3, §6.5 · book `14` (MA2, MA11), `08A` §6 (all), `08B` **MC-A4** (all), MC-A2 (what
a listing states, unmapped values), MC-A3 (site identity) · code `crates/research/src/rules/{value_map.rs, sameness.rs,
sites.rs}` (R3·U1, R3·U2).

### Files
New: `crates/research/src/rules/confidence.rs`, `crates/research/src/domain.rs` (the evidence types). Edited: `crates/research/src/{lib.rs, rules/mod.rs}`.

### Core mechanisms (the only functions with unit tests)
`confidence::{field_class, decide_attribution, decide_measured, decide_visible, decide_condition}`.

### Ordered tasks
1. `field_class(&Definition, in_layout, attribution_keys)`: first match of never-written, condition (the
   definition's group is `condition`), attribution (key in `attribution_keys`), measured (`dimension`, `weight`),
   visible. (The never-written test itself is `write_table::never_written`, R3·U5; here it is an input flag.)
2. `decide_attribution(labels, listings)`: levels, support and the decision of MC-A4. Only `match` and `similar`
   listing items count. Values are compared with `sameness::same`; an unmapped statement is its own value and can
   never be written. `Decision` carries the value, the confidence, `sources` (the page identities of the listing
   items supporting the written value at the decided level, in the order given) and `named` (the other values, for
   the reason).
3. `decide_measured(listings)`: the attribution function over `match` items only, a `high` result lowered to `medium`.
4. `decide_visible(photo_read, listings)`: the six-row table, `w` = `decide_attribution` over listing items only.
5. `decide_condition(photo_read, has_condition_photo)`: the MC-A4 paragraph; listings are not an input.

### Criteria (6 criteria, 32 rows)
Notation: `e1…e5` are five different trusted entries; "untrusted" sites are different hosts; A, B are values that
are not the same under `S`; every listing item is on its own page.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A05·C1(a) | `decide_attribution` | a label names A | A, high | MA2 |
| A05·C1(b) | same | shop `match` states A | A, high | MA2 |
| A05·C1(c) | same | shop `similar` states A, nothing else | A, low | MA2 |
| A05·C1(d) | same | `match` on e1 and on e2, both A | A, medium | MA2 |
| A05·C1(e) | same | two `match` pages on e1, both A | A, low | MA11 |
| A05·C1(f) | same | `match` on e1 = A, `match` on an untrusted site = A | A, low | MA2 |
| A05·C1(g) | same | label A; `match` on e1 and e2 = B | A, high; B in `named` | MA2 |
| A05·C1(h) | same | label A on one photo, label B on another | not determined | MA2 |
| A05·C1(i) | same | `match` e1, e2 = A; `match` e3, e4, e5 = B | B, low; A in `named` | MA2 |
| A05·C1(j) | same | `match` e1, e2 = A; `match` e3, e4 = B | not determined | MA2 |
| A05·C1(k) | same | `similar` on two untrusted sites = A, `similar` on one = B | A, low | MA2 |
| A05·C1(l) | same | `match` e1 and e2 state an unmapped value | not determined; the value in `named` | MA11 (R-IA24) |
| A05·C1(m) | same | only `different` verdicts, on e1 and e2, stating A | not determined | MA2 |
| A05·C1(n) | same | no evidence | not determined | MA2 |
| A05·C2(a) | `decide_measured` | `match` on e1 and e2 state 120 cm and 1200 mm | 120 cm, medium (same value under `S`) | MC-A4 |
| A05·C2(b) | same | shop `match` states 120 cm | 120 cm, medium (high is capped) | MC-A4 |
| A05·C2(c) | same | `similar` on e1 and e2 state 120 cm | not determined | MC-A4 |
| A05·C3(a) | `decide_visible` | photo read v, sure; listings give w at medium, `S(v, w)` | v, high | MC-A4 (R-IA11) |
| A05·C3(b) | same | v, sure; w at high, not `S(v, w)` | v, low; w in `named` | MC-A4 |
| A05·C3(c) | same | v, sure; w at low | v, medium | MC-A4 |
| A05·C3(d) | same | v, sure; no listing result | v, medium | MC-A4 |
| A05·C3(e) | same | v, unsure; w at high and `S(v, w)` | v, low | MC-A4 |
| A05·C3(f) | same | no photo read; w at medium | w, low | MC-A4 |
| A05·C3(g) | same | no photo read; no listing result | not determined | MC-A4 |
| A05·C4(a) | `decide_condition` | no Condition photo; a sure read is passed anyway | not determined | MC-A4 |
| A05·C4(b) | same | one Condition photo; sure read `Minor` | `Minor`, medium | MC-A4 (R-IA12) |
| A05·C4(c) | same | one Condition photo; unsure read `Minor` | `Minor`, low | MC-A4 |
| A05·C4(d) | same | one Condition photo; sure read `None` for a part no photo shows | `None`, medium | MC-A4 (card 4) |
| A05·C5(a) | `field_class` | a key in `attribution_keys` whose group is `condition`; a `dimension` key in `attribution_keys`; a `dimension` key outside it; a `choice` key outside it; a never-written flag | condition; attribution; measured; visible; never-written | MC-A4 |
| A05·C6(a) | `Decision.sources` | C1(d)'s input | the two pages' identities, in the order given | MC-A4 |
| A05·C6(b) | same | C1(a)'s input (label only); C1(i)'s input | empty; the three pages stating B | MC-A4 |
| A05·C6(c) | same | label A and shop `match` = A and `match` on e1 = A | A, high; sources = the shop page only (the level decided is high) | MC-A4 |

### Delegated choices
The exact field names of `Decision` and of the evidence types · how `named` orders its entries (by level, then by
support, then by first appearance is the suggestion). Not delegated: any outcome of the tables.

### Notes
- Each row's fixture must make its own rule the only reason for the outcome (charter rule 2): C1(e) uses two pages on
  **one** entry and nothing else; C1(f) uses exactly one trusted and one untrusted site.
- The function takes no confidence from its inputs. A later phase's test (A12·C8) feeds a model reply that claims
  one and shows it is ignored.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib rules::confidence` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 4 — Cost, stop rule, the day; error classes, attempts, redaction

**Unit id:** `R3·U4` · **Replaces:** A06 (all rows) · **Changes app files:** no

### Goal
The pure functions of MC-A8 and of the decision half of MC-A9. They take plain values and return plain values; the
gateway (R4·U2) and the run rules (R4·U4) call them. **Not in this phase:** recording calls, sending anything, the price
table's settings key (R4·U4), adapters turning a provider's reply into a `CallFailure` (R8·U1–R8·U3).

### Read first
`master_plan_research.md` §3, §6.3, §6.5, §7.4 · book `14` (MA5, MA15, MA16), `08A` §9, §10, `08B` **MC-A8** (all),
**MC-A9** (all), MC-A6 (Offline), `05A` MC-4 (calendar dates, time zone) · code
`crates/core/src/mechanisms/values.rs` (`canonical_decimal`), `crates/core/src/mechanisms/time.rs`,
`crates/infra/src/jobs.rs` (`backoff`: 1 s, 2 s, …).

### Files
New: `crates/research/src/rules/{cost.rs, calls.rs}`. Edited: `crates/research/src/{domain.rs, rules/mod.rs}`.

### Core mechanisms (the only functions with unit tests)
`cost::{parse_micro_usd, call_cost, next_call, day_spend}` · `calls::{classify, action, redact}`.

### Ordered tasks
1. `parse_micro_usd(&str) -> Option<i64>`: an MC-9 decimal string with at most 6 fractional digits, exact, no float.
2. `call_cost(usage: Option<&[(unit, count)]>, sent, provider, model, &PriceTable) -> i64`: `ceil(Σ usage × p / 10^6)`
   with `p = usd_per_million × 10^6` as an integer, in `i128`; no usage reported → `fallback_call_usd`; never sent →
   0; a reported unit missing from the table → the fallback for the whole call.
3. `next_call(run_cost, cap, reserve, stage) -> NextCall`: stages 1–6 → `Call` iff `run_cost + reserve < cap`, else
   `SkipToDecide`; stage 7 → `Call` iff `run_cost < cap`, else `EndAtCap`.
4. `day_spend(calls: &[(started_at_ms, cost)], now_ms, tz) -> i64`: the sum over calls whose **local calendar date**
   equals today's in `tz`.
5. `classify(&CallFailure) -> ErrorClass` and `action(class, attempt) -> CallAction {HoldOffline, Retry {wait_ms},
   FailRun, NotDetermined}` with `RESEARCH_CALL_ATTEMPTS = 3` and waits of 1 s then 2 s.
6. `redact(text, secrets) -> Redacted {text, hit}`: every occurrence of each secret, plain and URL-encoded, becomes
   `<redacted>`. An empty secret is ignored.

### Criteria (7 criteria, 29 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A06·C1(a) | `parse_micro_usd` | `0.50`; `15`; `0.000001` | 500000; 15000000; 1 | MA15 |
| A06·C1(b) | same | `0.0000001`; `1e3`; `1,5`; `-1`; the empty string | none, five times | MA15 |
| A06·C2(a) | `call_cost` | 1000 `input_token` at 2.50 and 500 `output_token` at 10.00 USD per million | 7500 | MA15 |
| A06·C2(b) | same | 1 `input_token` at 0.15 per million | 1 (rounded up) | MA15 |
| A06·C2(c) | same | 3 `input_token` and 4 `output_token`, both at 0.15 per million | 2 (summed, then rounded up once) | MA15 |
| A06·C2(d) | same | sent, no usage reported; fallback 0.02 | 20000 | MA15 (R-IA19) |
| A06·C2(e) | same | never sent | 0 | MA15 |
| A06·C2(f) | same | usage names a unit absent from the table; fallback 0.02 | 20000 | MC-A8 |
| A06·C2(g) | same | 10^12 `output_token` at 1000.00 per million | 1000000000000000 (no overflow) | MA15 |
| A06·C3(a) | `next_call`, cap 500000, reserve 20000, stage 3 | cost 479999; cost 480000 | `Call`; `SkipToDecide` | MA5 (card 5) |
| A06·C3(b) | same, stage 7 | cost 499999; cost 500000 | `Call`; `EndAtCap` | MA5 |
| A06·C4(a) | `day_spend`, zone Europe/Stockholm, now = 2026-11-12 10:00 local | calls at 2026-11-11 23:59:59.999 and 2026-11-12 00:00:00.000 local | only the second counts | MA15 |
| A06·C4(b) | same, now on 2026-10-25 (the 25-hour day) at 23:30 local | calls at 00:30 and at the second 02:30 of that date, and one at 2026-10-26 00:10 | the first two count | MA15 |
| A06·C4(c) | same | the same call list evaluated with zone UTC | the sum follows the evaluation zone's dates | MC-A8 |
| A06·C5(a) | `classify` | not sent, connect failure | offline | MC-A9 |
| A06·C5(b) | same | sent, timeout; sent, connection reset | transient; transient | MC-A9 |
| A06·C5(c) | same | status 408; 429 without the quota flag; 500; 502; 503; 504 | transient, six times | MC-A9 |
| A06·C5(d) | same | status 401; 403 | key; key | MC-A9 |
| A06·C5(e) | same | status 429 with the quota flag; status 402 with the quota flag | quota; quota | MC-A9 |
| A06·C5(f) | same | status 400; 404; 422 | request, three times | MC-A9 |
| A06·C5(g) | same | reply not matching the schema; a refusal | schema; refusal | MC-A9 |
| A06·C6(a) | `action` | offline, any attempt | `HoldOffline` | MC-A9 |
| A06·C6(b) | same | transient at attempt 1; 2; 3 | `Retry 1000`; `Retry 2000`; `FailRun` | MC-A9 (R-IA20) |
| A06·C6(c) | same | schema at attempt 1; 3 | `Retry 1000`; `FailRun` | MC-A9 |
| A06·C6(d) | same | key; quota; request, each at attempt 1 | `FailRun`, three times | MC-A9 |
| A06·C6(e) | same | refusal at attempt 1 | `NotDetermined` | MC-A9 |
| A06·C7(a) | `redact`, secret `sk-ab+c/d=` | `Bearer sk-ab+c/d=` ; `?api_key=sk-ab%2Bc%2Fd%3D&q=1` | `Bearer <redacted>`; `?api_key=<redacted>&q=1`; `hit` true both | MA16 |
| A06·C7(b) | same, two secrets | a text holding both, one of them twice | all three occurrences replaced | MA16 |
| A06·C7(c) | same | a text holding no secret; a secret list containing the empty string | unchanged, `hit` false; unchanged, `hit` false | MA16 |

### Delegated choices
A status the contract does not name (for example 505) is classified `request` and recorded under Assumptions ·
the time-zone type used in `day_spend` (`chrono` with `chrono-tz` in tests, as `mechanisms/time.rs`) · whether the
URL-encoded form also matches lowercase hex escapes (it should; record it).

### Notes
- Money is `i64` micro-USD everywhere; a float in this module is a B1 finding.
- C4(b) uses the date `master_plan.md` §7.4 names as the DST fixture day.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib rules::cost` · `cargo test -p psm-research --lib rules::calls` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 5 — What research may write (the write table)

**Unit id:** `R3·U5` · **Replaces:** A07 criteria C1–C6 · **Changes app files:** no

### Goal
`psm_research::writes::{suggest, lower}`: the one door research writes form values through, applying MC-A1 inside one store transaction — the fields never written, the write table per stored state, the prior value (`ai_prior`) and the conflict it defines, the cleared set, lowering, and idempotency. **Not in this unit:** the photographer's actions and their feedback rows (R5·U1; this unit only reads `research_feedback`), publishing (R5·U2), anything that calls the door in production (R4·U3).

### What changed from the superseded plan (this wins over any older wording below)
- The write path is `psm_research::writes::{suggest, lower}` in the research crate, working on the app's `Store` transaction; core's `AttributesService::ai_suggest` is left as it is (nothing in production calls it) and is not the door any more.
- The schema (`ai_prior`, `research_feedback`) and the `Value.ai_prior` field exist since R2·U2; tasks 1 and 2 of the older list are done. Row A07·C7(a) is in R2·U2.
- Tests are in the research crate (`crates/research/tests/engine/writes.rs`), on the harness: a `TestApp` (the testkit is a dev-dependency of `psm-research`) with a real SQLite store. Create the harness file `crates/research/tests/engine/harness.rs` here with what this unit needs; later units extend it.

### Read first
`master_plan.md` §3, §6.5, §6.9, §7.3, §7.5 · `master_plan_research.md` §0 (migrations, test folder), §3, §6.3–§6.5,
§7.4 · book `01`, `14` (MA1, MA10), `08` (Values and provenance, Groups and presets), `08A` §2 (HA1), §5 (What a run
may write), `08B` **MC-A1** (all), MC-A2 (the mapped value arrives already valid) · code
`crates/core/src/services/attributes.rs` (`writes::suggest` line 476, `write`, `same_content` line 81, `clear`,
`active_definitions`, presets), `crates/core/src/domain/attribute.rs` (`Value`), `crates/core/src/ports/store.rs`
(`StoreTx::{attribute_values, put_attribute_value, attribute_preset}`), `crates/infra/src/sqlite/store.rs`
(`migrations()`), `crates/core/src/mechanisms/sameness.rs` (R3·U1).

### Files
New: `crates/research/src/{writes.rs, rules/write_table.rs}`, `crates/research/tests/engine/{main.rs, harness.rs, writes.rs}` (one test program `engine`, `autotests = false`). Edited: `crates/research/Cargo.toml` (dev-dependency `psm-testkit`; `[[test]] name = "engine"`), `crates/research/src/{lib.rs, rules/mod.rs}`.

### Core mechanisms (the only functions with unit tests)
`rules::write_table::{never_written, decide_write, is_conflict}`.

### Ordered tasks
1. `rules::write_table`: `never_written` (key not in the item's current layout; a `product`-step field; core types `media_list`, `reference`, `rich_text`, `money`, `rating`; `writable = false`; `product.title`), `decide_write` (the MC-A1 table, fail closed), `is_conflict` (`ai` + `suggested`, has a prior, not `S(value, prior.value)`).
2. `writes::suggest(store, clock, integration, item_id, key, value, confidence, reason, sources) -> WriteOutcome`: in **one** store transaction read the stored row, the definition, the layout (the presets of the item's type, all three steps) and the cleared set, apply `decide_write`, and write the row (provenance `ai`, status `suggested`, `ai_prior` per the table). Outcomes the caller can put in the trace: written, unchanged, refused (with the table's reason), `matches a value you cleared`, `already holds this value`, `not researched`.
3. Cleared set `C(item, key)`: the stored row's value when it is cleared, plus the `suggested_value` of every `research_feedback` row of the field with action `cleared` or `kept_previous` (store methods from R2·U2); membership by `psm_core::mechanisms::sameness::same`.
4. `writes::lower(…, item_id, key, reason)`: on an `ai` + `suggested` row, the same value with confidence `low`, the sources kept, the reason `No longer supported: <reason>`; on any other row, nothing.
5. Idempotency: a write equal to the stored row in value, validity, provenance, confidence, status, reason and sources is not written (the rule of `same_content`, `services/attributes.rs` line 81). A research write sets the item's activity time as other value writes do, and records no undo step.

### Criteria (6 criteria, 25 rows)
Boundary of every row: `psm_research::writes::suggest` / `writes::lower` on the harness (a `TestApp` with a real SQLite store); the
outcome is the stored row read back (`attribute_values`) and the returned outcome.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A07·C1(a) | write table | no stored row; suggest A, medium | row `ai` + `suggested`, A, medium, `ai_prior` null | MA10 |
| A07·C1(b) | same | stored `default`, live, D; suggest A | row `ai` + `suggested` A; `ai_prior` = `{D, valid, default}` | MA10 (R-IA1) |
| A07·C1(c) | same | stored `integration`, live, valid I; suggest A | row A; `ai_prior` = `{I, true, integration}` | MA10 |
| A07·C1(d) | same | stored `integration`, live, **invalid** raw value; suggest A | row A; `ai_prior` = `{raw, false, integration}` | MA10 |
| A07·C1(e) | same | stored `ai` + `suggested` A with prior D; suggest B | row B; `ai_prior` still D | MA10 |
| A07·C1(f) | same | stored `ai` + `accepted` A; suggest B | refused; row byte-identical to before | MA1 |
| A07·C1(g) | same | stored `human`, live; suggest B | refused; row unchanged | MA1 |
| A07·C1(h) | same | stored `human` + `cleared`; suggest B | refused; row unchanged | MA10 (R-IA8) |
| A07·C1(i) | same | stored `ai` + `cleared` X; suggest X; suggest Y | X: `matches a value you cleared`, row unchanged; Y: row `ai` + `suggested` Y | MA10 |
| A07·C1(j) | same | stored `default` + `cleared` X; stored `integration` + `cleared` X; each: suggest X, then Y | X refused as cleared; Y written | MA10 (R-IA8) |
| A07·C1(k) | same | stored `ai` with no status (written straight through the store); suggest B | refused; row unchanged | MA10 |
| A07·C2(a) | prior and conflict | C1(b)'s row | `is_conflict` true | MC-A1 |
| A07·C2(b) | same | stored `default` `Teak` (text); suggest `teak ` | not written, outcome `already holds this value`; row still `default` | MC-A1 |
| A07·C2(c) | same | stored `ai` + `suggested` A with prior D; suggest a value the same as D under `S` | row holds it with the prior kept; `is_conflict` false | MC-A1 |
| A07·C2(d) | same | a suggestion with no prior | `is_conflict` false | MC-A1 |
| A07·C3(a) | cleared set | feedback row `cleared` with suggested X; current row `ai` + `suggested` Y, then cleared; suggest X; suggest Y; suggest Z | X refused, Y refused, Z written | MA10 (R-IA8) |
| A07·C3(b) | same | feedback row `kept_previous` with suggested X; suggest X | refused as cleared | MA10 |
| A07·C3(c) | same | text field, feedback `cleared` with `Hans J. Wegner`; suggest `hans j wegner` | refused as cleared (membership by `S`) | MA10 |
| A07·C3(d) | same | feedback row `accepted` with suggested X on a now-empty field; suggest X | written | MC-A1 |
| A07·C4(a) | `writes::lower` | stored `ai` + `suggested` A, high, sources `[u1]`; lower with reason `the two listings now disagree` | value A, confidence `low`, sources `[u1]`, reason `No longer supported: the two listings now disagree`, status `suggested` | MA2 |
| A07·C4(b) | same | stored `ai` + `accepted`; stored `human`; no row | nothing changes, three times | MA1 |
| A07·C5(a) | never written | a key not in the item's layout; suggest A | outcome `not researched`; no row created | MA10 |
| A07·C5(b) | same | a `product`-step field in the layout; a `media_list`, a `reference`, a `rich_text`, a `money`, a `rating` field; a definition with `writable = false`; `product.title` | `not researched`, eight times; no row written, the item's title unchanged | MA10 (R-IA6, R-IA7) |
| A07·C6(a) | idempotency | the same suggestion (value, confidence, reason, sources) twice | the second returns unchanged; `updated_at` unchanged; the session's activity time unchanged | MA14 |
| A07·C6(b) | undo | a research write, then `History` state for the session | no undo step exists for it | MC-A12 |

### Delegated choices
The outcome type's name and variants beyond the reasons listed · whether `decide_write` returns the new row or an
instruction · the feedback store methods' names. Not delegated: the reasons' texts quoted above, and that every
read used by the decision happens inside the writing transaction.

### Notes
- Hazard (earned in P67): a new field on `Value` changes many struct literals and one generated DTO. List the
  touched test files under **Deviations**; no test's expectation may change except to name the new field.
- `writes::suggest`'s new refusals are outcomes, not errors, except where today's callers rely on the `human` error; keep
  that error text for the existing P10 test or update that one assertion and say so.
- The cleared set reads `research_feedback`, which only R5·U1 writes in production; this phase's tests insert rows
  through the store method they add (it has a production caller in R5·U1).

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine writes::` · `cargo test -p psm-research --lib rules::write_table` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test -p psm-research` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`. **No app suite** (no file outside `crates/research` changed, except `Cargo.lock`).

## Review mode
The reviewer reads the units in order and runs, once: the milestone's research-crate tests (`cargo test -p psm-research`), `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`. **No app suite** (no app file changed; flow §10). Only B1–B4 block. Notes go to `docs/implementation/backlog_research/R3_research_rules.md`.

## Manual try-out (the narrative is built from this)
Nothing to see in the app yet. `cargo test -p psm-research --lib rules` prints the rules by name, row by row ("a05_c1d_two_trusted_sites_medium", …).

## Review log
