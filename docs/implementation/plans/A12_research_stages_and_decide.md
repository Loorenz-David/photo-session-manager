---
phase: A12_research_stages_and_decide
title: One research run from start to finish — look at the photos, find and read matching listings, compare, read labels and condition photos, then fill the form with suggestions, each with its computed confidence, reason, sources and a step-by-step trail
difficulty: high
projection: yes
depends_on: [A11_research_call_gateway]
---
# A12 — The seven stages, unit commit, steps, Decide and its writes (08A §5, §8; 08B MC-A7 commit rule, MC-A3, MC-A4, MC-A1, MC-A8; MA1, MA2, MA5, MA6, MA14)

## Goal
`ResearchService::execute(run_id)`: run the stages of 08A §5 for a run that is already `running`, against the
ports, through the gateway; persist each unit's output with its fingerprint; write the trace steps; decide every
field with A05's functions and write through A07's `ai_suggest` / `ai_lower`; end `finished`, `stopped_at_cap` or
`failed`. All provider replies in this phase are the fakes' scripted replies (our shapes). **Not in this phase:**
creating, queueing and starting runs, the job, settings on disk (A13); resume after a kill, reuse and follow-ups
(A14); real adapters (A15–A17).

## Read first
`master_plan_research.md` (all of §6, §7.5) · book `01`, `14` (MA1, MA2, MA5, MA6, MA14), `08A` §5 (all), §6, §8, §9,
§10, `08B` **MC-A7** (Units, Storage, Commit rule), **MC-A3** (reading order, page limit, site of a page, own
listing), **MC-A4**, **MC-A1**, **MC-A2**, **MC-A8** (Stop rule), MC-A5 (image sizes per stage) · code
`mechanisms/research/*` (A03–A07), `services/research/gateway.rs` (A11), `ports/research.rs` (A10),
`services/attributes.rs` (`ai_suggest`, `ai_lower`, `form_get`, presets), `crates/core/src/ports/commerce.rs`,
`crates/testkit/src/{fake_research.rs, fake_commerce.rs}`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/core/src/services/research/{stages.rs, decide.rs}`, `crates/testkit/tests/suite/research/
a12_research_stages_and_decide.rs` (+ `mod` line).
Edited: `crates/core/src/services/research/mod.rs`, `crates/core/src/domain/research.rs` (`ResearchStep`,
`StageOutput`, `FormSnapshot`, `ResearchConfig`, the per-stage output shapes), `crates/core/src/ports/{store.rs,
commerce.rs}` (`CommerceAdapter::{public_listing, unit_names}` with default bodies returning nothing),
`crates/infra/src/sqlite/store.rs` (steps, stage outputs), `crates/testkit/src/{fake_commerce.rs, test_app.rs}`,
own tracker row.

## Core mechanisms
None new.

## Ordered tasks
1. **Snapshots.** `execute` works only from the run's `settings_snapshot` (`ResearchConfig`: per-stage model, effort,
   image long edge; `max_pages`; caps; price table; trusted sites; shop hosts; own listing; attribution keys;
   language; versions) and `form_snapshot` (item type; the ordered layout fields with key, label, core type,
   choices, description, current value and provenance). This phase's tests write both when they insert the run;
   A13 fills them at start.
2. **Stage units** (MC-A7 table) with `unit_key` and `fingerprint` (A04): 1 observe — one per eligible `main` or
   `unreviewed` photo, output includes the visible reads with `sure` and `shows_marking`; 2 find — one, the cover
   (main position 1); 3 read — one per page in MC-A3 reading order until `max_pages` pages are **obtained** or the
   stop rule says skip: the page reader, then a model call that returns `{stated, value}` per researched key
   (mapped by A03); 4 compare — one per page that has images, the cover against the page's stored images, verdict
   `match` / `similar` / `different`; 5 labels — one per photo with `shows_marking`; 6 condition — one, over the
   `condition` photos (none → no call, every condition field not determined); 7 decide.
3. **Commit rule.** When a unit's last call returns, one transaction commits its `research_stage_outputs` row, its
   steps (`seq` = max of the run + 1, gapless) and its calls' responses. Step kinds: `looked` (1, 5, 6),
   `searched` (2), `read` (3; source URL and title; an unobtainable page is a `read` step `could not be read` and
   does not count; the own listing is a step `this item's own listing` and is not read), `compared` (4),
   `concluded` and `not_determined` (7). Each step names the photo ids it used and the field keys it led to.
4. **Decide.** For every layout field: `field_class`; build the evidence items from the committed outputs (site of
   a page from the reader's final URL when it reports one, else the candidate URL); A05's decision; then one model
   call (stage 7) writes the reason for each decided field in `research.language`; one transaction applies the
   writes through `ai_suggest` (value, **computed** confidence, reason, `sources`) or `ai_lower` (a field decided
   not determined whose row is `ai` + `suggested`), and commits the `concluded` step per written key and one
   `not_determined` step listing every other field with its reason (including `not researched` and `matches a
   value you cleared`). The stage 7 output stores per key the value, confidence, reason, sources and the MC-A1
   outcome.
5. **Endings.** `SkipToDecide` from the gateway → skip the remaining reading units, go to Decide, end
   `stopped_at_cap` with the writes; `EndAtCap` before Decide → `stopped_at_cap`, no write; `RunFailed` → `failed`
   with stage and error, earlier values untouched; a refusal in a unit → that unit's output is "not determined"
   with the refusal text, the run continues; otherwise `finished` with `stage_reached` 7.

## Criteria (8 criteria, 24 rows)
Boundary: `ResearchService::execute` on `TestApp::with_research()` with scripted fakes and a `running` run row;
outcome = `attribute_values`, `research_steps`, `research_stage_outputs`, `research_runs`, the fakes' request logs.
Shop host `beyo.example`; trusted `[e1.example, e2.example]`; layout: `designer` (text, an attribution key),
`wood` (multi-choice), `width` (dimension), `scratches` (severity choice, Condition step), `title` (Product step).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | a full run | two main photos, one condition photo; search returns pages on e1, e2 and an untrusted site; e1 and e2 state designer A and are `match`; observe reads wood `Teak`, sure; condition reads `Minor`, sure | `designer` = A, medium, sources = the two pages; `wood` = `["Teak"]`, medium; `scratches` = `Minor`, medium; all `ai` + `suggested`; run `finished`, stage 7 | MA2 |
| C1(b) | steps | the same run | `seq` is 1…n with no gap; kinds in order: `looked` ×2, `searched`, `read` ×3, `compared` (one per page with images), `looked` (condition), `concluded` per written key, one `not_determined`; every written key appears in some step's field keys | MA6 |
| C1(c) | stage outputs | the same run | one row per unit of the MC-A7 table, each with the fingerprint A04 computes for its identity; the stage 7 row lists per key value, confidence, reason, sources, outcome | MA14 |
| C2(a) | reading order | candidates ranked: untrusted 1, e2 2, e1 3, shop (another product) 4 | the page reader is called for shop, e1, e2, untrusted, in that order | MC-A3 |
| C2(b) | page limit | `max_pages` 2; four candidates, the first unobtainable | three reader calls; the first is a `read` step `could not be read`; two pages obtained; the fourth never requested | MC-A3 (R-IA10) |
| C2(c) | own listing | a candidate that is the item's own listing (shop host, the item's path tail) | never read; a step `this item's own listing`; it is no evidence (designer not determined when it was the only page) | MA11 (R-IA9) |
| C2(d) | redirect | the reader reports a final URL on e1 for a candidate on an untrusted host | the page counts as e1 | MC-A3 |
| C3(a) | units and images | two main, one unreviewed, two condition photos; one main photo's observe output has `shows_marking` | observe ×3 at long edge 1024; labels ×1 at 2048; condition ×1 with two images at 2048; find and compare use the cover at 1024 (the model fake's and the search fake's logs) | MC-A7, MC-A5 |
| C3(b) | label | the label unit reads designer A | `designer` = A, high, sources empty | MA2 |
| C3(c) | no condition photo | no photo classified `condition` | no stage 6 call; `scratches` not determined; nothing written for it | MC-A4 |
| C4(a) | not determined | no page states a designer, no label | no `designer` row is created; the `not_determined` step lists it with a reason | MA6 |
| C4(b) | protected values | before the run: `designer` is `human`; `wood` is `ai` + `accepted` | both rows byte-identical after the run; the last step says why | MA1 |
| C4(c) | cleared | `designer` was cleared with value A (feedback row); the run decides A | not written; the step says `matches a value you cleared` | MA1 |
| C4(d) | lowering | `designer` holds `ai` + `suggested` A, medium, from before; this run finds no support | the row is A, `low`, reason starting `No longer supported:` | MA2 |
| C4(e) | never written | `title` (Product step) | untouched; named `not researched` | MA10 |
| C4(f) | measured | `width`: e1 and e2 `match` pages state 120 cm; a photo read offers 118 cm | 120 cm at medium (the photo read is not used) | MC-A4 |
| C5(a) | cap, reserve | cap 0.50, reserve = the stage 7 fallback; scripted usage makes cost + reserve reach the cap after the second page | the remaining read and compare units are skipped; Decide runs; run `stopped_at_cap`; the values supported so far are written | MA5 (card 5) |
| C5(b) | cap reached | the cost equals the cap before Decide | run `stopped_at_cap`; no attribute row written | MA5 |
| C5(c) | bound | C5(a)'s run | run cost ≤ cap + the cost of the one call that crossed it | MA5 |
| C6(a) | failure | the page reader returns three transient faults for the second page | run `failed`, error names stage `read`; rows written by an earlier run are unchanged; no stage 7 output | MC-A9 |
| C6(b) | refusal | the model refuses one observe unit | that unit's output is "not determined" with the refusal text; the run finishes; other fields are written | MC-A9 |
| C7(a) | language and choices | `language` = `Swedish`; `wood` choices `[Oak, Teak]` | every model request carries the language; the read and observe requests carry the choices; the stored `wood` value is the definition's own string | MC-A2 |
| C7(b) | snapshot | the trusted list in the live settings is changed while the run executes (from the fake's hook) | reading order and site classes follow the run's snapshot | MC-A3 |
| C8(a) | confidence is computed | the stage 7 reply claims `"confidence": "high"` for a designer supported by one `similar` page | stored confidence `low` | MA11 |

## Delegated choices
The per-stage output JSON shapes and the request field names (ours) · `unit_key` spellings · step summary wording
except the quoted texts · how reasons are requested (one stage 7 call for all decided fields). Not delegated: one
call per unit kind as in the MC-A7 table, calls strictly sequential, every provider call through the gateway.

## Manual try-out
Nothing visible yet (no real provider, no trigger). The run can be read as a story in the test:
`cargo test -p psm-testkit --test suite research::a12_research_stages_and_decide::`.

## Notes
- Rule 2 companion: each row's fixture gives its outcome exactly one cause (C1(a): exactly two trusted `match`
  pages for the designer; C3(b): a label and no listing).
- The fakes' reply shapes are this repo's port shapes. No row here depends on what OpenAI or a search vendor
  returns; those rows are in A15–A17.
- Rule 10 is served in A24 (shipped defaults reach a run).
- Review mode: this phase's module, `research::a07…::`, `research::a11…::`, clippy, fmt.

## Review log
