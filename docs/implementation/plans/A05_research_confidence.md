---
phase: A05_research_confidence
title: Research core, part 3 — how sure a suggestion is, computed by code from the evidence (a read label, matching listings on the shop or on trusted sites, what the photos show), never by the model
difficulty: high
projection: yes
depends_on: [A04_research_sites_and_fingerprints]
---
# A05 — Confidence derivation (08B MC-A4; 08A §6, HA2)

## Goal
The pure function that turns recorded evidence into a value and a confidence for one field, for each of the four
field classes (attribution, measured, visible, condition), exactly as MC-A4's tables say. This is the mechanism
ranked first for silent failure: a false designer at `medium` is published unchecked. **Not in this phase:**
gathering the evidence (A12), writing the value (A07), the reason text (A12, model-written).

## Read first
`master_plan_research.md` §3, §6.3, §6.5 · book `14` (MA2, MA11), `08A` §6 (all), `08B` **MC-A4** (all), MC-A2 (what
a listing states, unmapped values), MC-A3 (site identity) · code `mechanisms/research/{value_map.rs, sameness.rs,
sites.rs}` (A03, A04).

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/core/src/mechanisms/research/confidence.rs`; the evidence types in `crates/core/src/domain/research.rs`
(new file: `Verdict`, `SiteClass`, `FieldClass`, `LabelItem {photo_id, value: Stated}`, `ListingItem {page, site,
verdict, value: Stated}`, `PhotoRead {value, sure}`, `Decision`). Edited: `domain/mod.rs`, `mechanisms/research/mod.rs`,
own tracker row.

## Core mechanisms (unit tests allowed; one inline test per row, `mc_a4_<slug>`)
`confidence::{field_class, decide_attribution, decide_measured, decide_visible, decide_condition}`.

## Ordered tasks
1. `field_class(&Definition, in_layout, attribution_keys)`: first match of never-written, condition (the
   definition's group is `condition`), attribution (key in `attribution_keys`), measured (`dimension`, `weight`),
   visible. (The never-written test itself is `write_table::never_written`, A07; here it is an input flag.)
2. `decide_attribution(labels, listings)`: levels, support and the decision of MC-A4. Only `match` and `similar`
   listing items count. Values are compared with `sameness::same`; an unmapped statement is its own value and can
   never be written. `Decision` carries the value, the confidence, `sources` (the page identities of the listing
   items supporting the written value at the decided level, in the order given) and `named` (the other values, for
   the reason).
3. `decide_measured(listings)`: the attribution function over `match` items only, a `high` result lowered to `medium`.
4. `decide_visible(photo_read, listings)`: the six-row table, `w` = `decide_attribution` over listing items only.
5. `decide_condition(photo_read, has_condition_photo)`: the MC-A4 paragraph; listings are not an input.

## Criteria (6 criteria, 32 rows)
Notation: `e1…e5` are five different trusted entries; "untrusted" sites are different hosts; A, B are values that
are not the same under `S`; every listing item is on its own page.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `decide_attribution` | a label names A | A, high | MA2 |
| C1(b) | same | shop `match` states A | A, high | MA2 |
| C1(c) | same | shop `similar` states A, nothing else | A, low | MA2 |
| C1(d) | same | `match` on e1 and on e2, both A | A, medium | MA2 |
| C1(e) | same | two `match` pages on e1, both A | A, low | MA11 |
| C1(f) | same | `match` on e1 = A, `match` on an untrusted site = A | A, low | MA2 |
| C1(g) | same | label A; `match` on e1 and e2 = B | A, high; B in `named` | MA2 |
| C1(h) | same | label A on one photo, label B on another | not determined | MA2 |
| C1(i) | same | `match` e1, e2 = A; `match` e3, e4, e5 = B | B, low; A in `named` | MA2 |
| C1(j) | same | `match` e1, e2 = A; `match` e3, e4 = B | not determined | MA2 |
| C1(k) | same | `similar` on two untrusted sites = A, `similar` on one = B | A, low | MA2 |
| C1(l) | same | `match` e1 and e2 state an unmapped value | not determined; the value in `named` | MA11 (R-IA24) |
| C1(m) | same | only `different` verdicts, on e1 and e2, stating A | not determined | MA2 |
| C1(n) | same | no evidence | not determined | MA2 |
| C2(a) | `decide_measured` | `match` on e1 and e2 state 120 cm and 1200 mm | 120 cm, medium (same value under `S`) | MC-A4 |
| C2(b) | same | shop `match` states 120 cm | 120 cm, medium (high is capped) | MC-A4 |
| C2(c) | same | `similar` on e1 and e2 state 120 cm | not determined | MC-A4 |
| C3(a) | `decide_visible` | photo read v, sure; listings give w at medium, `S(v, w)` | v, high | MC-A4 (R-IA11) |
| C3(b) | same | v, sure; w at high, not `S(v, w)` | v, low; w in `named` | MC-A4 |
| C3(c) | same | v, sure; w at low | v, medium | MC-A4 |
| C3(d) | same | v, sure; no listing result | v, medium | MC-A4 |
| C3(e) | same | v, unsure; w at high and `S(v, w)` | v, low | MC-A4 |
| C3(f) | same | no photo read; w at medium | w, low | MC-A4 |
| C3(g) | same | no photo read; no listing result | not determined | MC-A4 |
| C4(a) | `decide_condition` | no Condition photo; a sure read is passed anyway | not determined | MC-A4 |
| C4(b) | same | one Condition photo; sure read `Minor` | `Minor`, medium | MC-A4 (R-IA12) |
| C4(c) | same | one Condition photo; unsure read `Minor` | `Minor`, low | MC-A4 |
| C4(d) | same | one Condition photo; sure read `None` for a part no photo shows | `None`, medium | MC-A4 (card 4) |
| C5(a) | `field_class` | a key in `attribution_keys` whose group is `condition`; a `dimension` key in `attribution_keys`; a `dimension` key outside it; a `choice` key outside it; a never-written flag | condition; attribution; measured; visible; never-written | MC-A4 |
| C6(a) | `Decision.sources` | C1(d)'s input | the two pages' identities, in the order given | MC-A4 |
| C6(b) | same | C1(a)'s input (label only); C1(i)'s input | empty; the three pages stating B | MC-A4 |
| C6(c) | same | label A and shop `match` = A and `match` on e1 = A | A, high; sources = the shop page only (the level decided is high) | MC-A4 |

## Delegated choices
The exact field names of `Decision` and of the evidence types · how `named` orders its entries (by level, then by
support, then by first appearance is the suggestion). Not delegated: any outcome of the tables.

## Manual try-out
Nothing to see in the app yet. `cargo test -p psm-core mechanisms::research::confidence` lists the evidence rule
row by row.

## Notes
- Each row's fixture must make its own rule the only reason for the outcome (charter rule 2): C1(e) uses two pages on
  **one** entry and nothing else; C1(f) uses exactly one trusted and one untrusted site.
- The function takes no confidence from its inputs. A later phase's test (A12 C8) feeds a model reply that claims
  one and shows it is ignored.
- Review mode: `cargo test -p psm-core mechanisms::research`, clippy, fmt.

## Review log
