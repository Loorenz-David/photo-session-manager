---
phase: R12_research_cost
title: Research costs less — the photos are looked at in one call instead of one call each, with the first photo treated as the wide shot and the later ones as the closer shots; a listing page is cut down to its product part before it is read; a page about another kind of piece is not compared; then the 42-piece measurement is run again so cost and accuracy can be compared with the first
difficulty: high
projection: no
depends_on: [R11_research_on_arranged]
blocked_on: U4's measurement is owner-attended (the keys of master_plan_research §7.6 and a spending ceiling of USD 20); U1–U3 are not blocked
---
# R12 — Research costs less

**Tracker:** NOT_STARTED · 2026-11-15 · planner · plan written (16 criteria, 37 rows, 4 units; 3 rows `[manual]`); no owner card

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
Lower what a run spends without lowering what it gets right, then measure it. (U1) Stage 1 looks at the chosen Main photos in **one** model call, told which photo is the wide shot and which are closer; today it makes one call per photo and that is about half of a run's cost with the models of R10. (U2) The page reader hands the read model the product part of a listing page instead of the whole visible text. (U3) The read model also says whether the page lists the same kind of piece; a page that lists another kind is not compared. (U4) The 42-piece acceptance measurement again, owner-attended, recorded beside the first. **Not here:** a shoot-template concept (see "What the shoot template would need"); skipping the read model on structured data (see "Why no skip rule"); any app file; any change to which values may be written or at what confidence.

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 4 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids of this plan are `A26·C<n>(<row>)`; criteria are numbered once across the units. Name each test after its row (`a26_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R12_research_cost.md`. **If launched by `run_phase.sh`, the implementer builds U1–U3 and tasks 1–2 of U4 with their automated rows, then reports `STATE: DONE`; the measurement (U4 tasks 3–6) is the orchestrator's, with the owner present, like R9·U2.**

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The photo look in one call, by position | 7 | 17 | no |
| U2 | The product part of a listing page | 2 | 9 | no |
| U3 | Same kind of piece, or no comparison | 3 | 6 | no |
| U4 | The second acceptance measurement | 4 | 5 | no |
| **Total** | | **16** | **37** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §7 · `docs/implementation/master_plan_research.md` (all; §6.8 holds the names this plan adds, §7.4 the facts the planner verified for it on 2026-11-15) · book `01`, `14` §Research agent, `08A`, `08B` (**changelog 12 (d), (e) are the owner's decisions this plan implements**) · `docs/implementation/backlog_research/owner_ideas.md` (the cost levers with their measured shares) · evidence `docs/intention/evidence/research_providers.md` ("Page reading", "Acceptance measurement", "Model comparison", "Owner pick of models") · the R11 plan (the `arranged` trigger, `excluded_keys`, the derived field) · then each unit's own Read first.

## Where the money goes now (why the units are in this order)
First measurement, per piece (evidence "Acceptance measurement"): observe USD 0.101 (one call per Main photo, about 0.011 each), read 0.081, compare 0.071, total about 0.31. R10 then moved read to a model at about one tenth of the price and compare to one at half: by the price ratios read is about 0.01 and compare about 0.035 a piece, the estimated total about 0.20 (evidence "Model comparison"), and **observe is now about half of it**. These are estimates; U4 measures. So U1 is the large lever. U2 and U3 save little money on the new models; they remove wrong reads (a listing page's "more from this dealer" list names other designers) and the comparison calls of pages that cannot match. U4 says what the three were worth.

## What this plan supersedes in the book (the orchestrator folds these into 08A and 08B; until then this section is the rule)
| Book text | Replaced by |
|---|---|
| 08A §5 stage table row 1 "each Main photo, looked at once and remembered (owner card 2)"; 08B MC-A7 Units row 1 "one per eligible `main`/`unreviewed` photo" with its identity fields and `shows_marking: bool` | U1: **one** stage 1 unit per run over the chosen photos. Identity: the ordered list of (photo id, composition revision, role), long edge, model, effort, instructions version, language. Output: the reads and the photos that show a marking. Authority: the owner's cost idea of 2026-11-13 (lever 2: "several photos in one call") and changelog 12 (d). Owner card 2's other half stands: Compare uses the cover. |
| 08A §5 "Photos used": every Main and unreviewed photo is looked at | U1: at most `observe_max_photos` (default 8), Main photos first by position. Find, Compare, Labels and Condition are unchanged. |
| 08B MC-A7 "Follow-up trigger set": "the set of stage 1 unit identities" | the stage 1 unit identity (the chosen photos). A change to a photo beyond the limit changes nothing. |
| 08B MC-A7 Units row 3 (Read identity) and row 4 (Compare: "one per page with images") | U3: the Read identity gains the item type; Compare has one unit per page with images **whose read did not say it lists another kind of piece**. |
| 08B MC-A7 `RESEARCH_FINGERPRINT_VERSION = 1` | 2 (MC-A7: "a change to any identity field list bumps it"). The instructions version becomes 2. |
| 08A §4, page reader port: "returns its text and main images" | U2: its text is the product extract of the page; images as before. |
| 08B changelog 12 (d) "the template decides which photos serve which field" | built as position only (first Main photo wide, the others closer); see the next section. |
| owner idea 1, second half ("skip the model when structured data names the field outright") | not built; see "Why no skip rule". |

## What the shoot template would need (changelog 12 (d); nothing here builds it)
There is no shoot template in the code or the book: 08A §3 lists "shot templates" under "Not existing" and §12 defers them to the photography-assistant chapter. What research needs from one is small: **per item type, a name for each Main position** (wide, side, back, detail, label) **and, per name, the fields it serves**. Until that exists, the only fact available is the owner's own: the first Main photo is always the wide one. U1 uses exactly that, inside one function (`rules::photo_plan::look_at`), and lets the model read each field from the photo that shows it best, because it now sees them together. When a template lands, that function takes its names instead of positions; nothing else changes.

## Why no skip rule (verified on the recorded pages; `master_plan_research.md` §7.4)
On the 375 listing pages R1 recorded, no trusted site states a designer or a maker in a structured field of its own. A schema.org product block exists on the owner's shop (37 of 42 pages) and on a few dealers, and its `brand` names **the seller** (the owner's shop, two other dealers), not the maker. Only one site gives a designer as a `creator`, on 3 of its 35 pages. A rule "take the maker from the structured data and skip the model" would therefore almost never fire, and when it fired on `brand` it would write the dealer as the maker. So U2 hands the structured block to the model as text and never decides from it.

## Tests that pin what this plan changes (found by grep; each is handled in the unit named)
| Test | File | Pins | What happens, where |
|---|---|---|---|
| `a12_c1a_a_full_run_fills_the_form`, `a12_c1b_steps_are_gapless_and_in_stage_order`, `a12_c1c_stage_outputs_one_per_unit_with_fingerprints`, `a12_c3a_units_and_image_sizes`, `a12_c3b_label_gives_high_without_sources`, and the helper `full_script` | `crates/research/tests/engine/stages.rs` | one Observe unit and one `looked` step per photo; unit keys `photo:<id>` | U1: kept with their outcomes; the script answers one Observe unit, and the counts of units and steps follow (rows A12·C1(a)–(c), C3(a), (b) keep their ids, their input cells read "one Observe unit") |
| `a14_c1a_kill_when_the_second_observe_call_arrives`, `a14_c3a_follow_up_for_a_new_main_photo`, `a14_c3d_follow_up_for_a_recomposed_photo`, the helper `script` | `crates/research/tests/engine/resume.rs` | a kill between two Observe calls; a new or recomposed photo costs one Observe call for that photo only | U1: `a14_c1a` deleted (A26·C5(a) replaces it); `a14_c3a`, `a14_c3d` amended: the change costs one Observe call over all chosen photos (A26·C4(b) states it) |
| the Observe rows of the adapter tests (`OBSERVE_LUNA`, the derived observe form) | `crates/research/tests/engine/adapter_openai.rs` | the Observe schema `reads` + `shows_marking` | U1: kept green unchanged (a request without `photos` keeps that schema: A26·C7(b)) |
| tests that script or count Observe per photo | `crates/research/tests/engine/{export.rs, images_and_storage.rs, api.rs, lifecycle.rs, defaults.rs, arranged.rs, derived.rs, settings_r11.rs}` | incidental (a run must get through stage 1) | U1: follow the changed script helper |
| the run-building helpers | `crates/research-tool/tests/tool/{u3_report_and_replay.rs, u4_accept.rs}` | per-photo Observe replies | U1: follow; a replay of a run exported before R12 still sends its recorded per-photo request (A26·C7(b)) |
| the page-text rows A17·C1–C3 | `crates/research/tests/engine/adapter_pages.rs` | `PageReply.text` is the visible text | U2: the text assertion becomes the extract (A26·C9(a)); title, images, redirects, pacing rows unchanged |
| the Read rows of the adapter tests (`stage_read_gpt-6-luna.json`) | `crates/research/tests/engine/adapter_openai.rs` | the Read schema `keys` | U3: kept green unchanged (a request without `piece` keeps that schema: A26·C12(a)) |
No shipped default changes value. One default is new (`observe_max_photos`, 8); A24·C2(a) stands.

## Carried from the owner (2026-11-15; orchestrator)
- **U1 or the first unit touching `crates/research/src/config.rs`: ship the detailed-condition table filled.** The
  owner answered R11's card with A: worst condition level None → Very Good, Minor → Very Good, Moderate → Good,
  Major → Worn, never Excellent (choice strings as in the shop definitions fixture). R11 ships the table empty
  (its "on silence"); this phase writes A as the default and adds one row: `ResearchSettings::default()` carries
  that table. The owner will tweak it later in Settings → Research.

## Unit 1 — The photo look in one call, by position

**Unit id:** `R12·U1` · **Rows:** A26·C1–C7 · **Changes app files:** no

### Goal
Stage 1 becomes one unit: the chosen photos go to the model together, in order, each named wide or closer, and the model answers once per field. Which photos show a marking comes back as a list, so Labels still reads only those. **Not in this unit:** Find, Compare, Labels, Condition (unchanged); a template.

### Read first
`master_plan_research.md` §3 (Rule 17), §6.5, §6.6, §6.8, §7.4 · book `08A` §5, `08B` MC-A5 (order per call carrying photos; several photos in one call are already how Condition works), MC-A7, MC-A8 (stop rule), changelog 12 (d) · evidence "Acceptance measurement" (cost shares), "Model comparison" (photo look) · fixtures `crates/research/fixtures/openai/{stage_observe_gpt-6-luna.json, stage_condition_gpt-6-luna.json}` (RESEARCH-VERIFY-3: one image costs about 1,560 input tokens with its instructions; six images go in one request) · code `crates/research/src/{stages.rs (observe, labels, usable_photos, execute), decide.rs (Evidence::photo_read), runs.rs (input_now), rules/run_rules.rs (InputSnapshot), rules/fingerprint.rs, config.rs, fakes.rs, adapters/openai/{schema.rs, adapter.rs}}`.

### Files
New: `crates/research/src/rules/photo_plan.rs`, `crates/research/tests/engine/photo_look.rs`. Edited: `crates/research/src/{rules/mod.rs, rules/fingerprint.rs, config.rs, stages.rs, decide.rs, runs.rs, fakes.rs, adapters/openai/schema.rs}`; `crates/research/tests/engine/{main.rs, stages.rs, resume.rs, harness.rs}` and the engine test files of the table above where their script helper changes; `crates/research-tool/tests/tool/{u3_report_and_replay.rs, u4_accept.rs}`; `master_plan_research.md` §6.3 (the two version constants).

### Core mechanisms (the only functions with unit tests)
`rules::photo_plan::look_at` (which photos, in which order, with which role).

### Ordered tasks
1. `rules::photo_plan::look_at`: from the run's usable photos (eligible, with a source) and the limit, the photos to look at: Main photos by position (ties by id), then unreviewed photos by id, cut at the limit; Condition photos never. The Main photo at the lowest position has the role wide; every other chosen photo has the role closer.
2. Setting `observe_max_photos` (default 8, a whole number of at least 1), in `ResearchSettings`, `validate` and the run's settings snapshot.
3. Stage 1 is one unit. One model call carries the chosen photos in order and, in its input, each photo's index (from 1) and role beside the visible fields. Instructions, version 2: the wide photo says what the piece is and how many there are; wood, material, colour and surface are read from the closer photos; one answer per field with sure or unsure; list the indexes of the photos that show a label, stamp or marking. The output keeps the reads and the ids of the marking photos; an index outside the photos sent is ignored and named in the step detail. One `looked` step names every photo sent; when the limit left photos out, its detail says how many.
4. A photo that can no longer be rendered (removed since, MC-A5) is left out of the call, as Condition does; when none can be rendered the unit commits "could not read the photos" with no read.
5. Labels: one unit per photo the Observe output lists. Decide: the photo read of a visible field is the Observe unit's read for it.
6. `input_now` and the input snapshot: the stage 1 part is the chosen photos. `RESEARCH_FINGERPRINT_VERSION` and `INSTRUCTIONS_VERSION` become 2.
7. OpenAI adapter: for an Observe request whose input carries `photos`, the strict schema is the reads plus a list of whole numbers for the marking photos; a request without `photos` (a replay of a run exported before R12) keeps the schema it had.
8. Fakes and the script helpers of the engine and tool tests follow; the tests of the table above.

### Criteria (7 criteria, 17 rows)
Boundary: the named rule; full runs on the research harness with scripted fakes (outcome = the requests the fake model received, steps, outbound rows, stage outputs, written values); the OpenAI adapter over `FakeResearchHttp`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A26·C1(a) | `rules::photo_plan::look_at` | Main photos at positions 1, 2, 3, one unreviewed photo, two Condition photos; limit 8 | the three Main photos in position order, then the unreviewed one; roles wide, closer, closer, closer | MC-A7 (changelog 12 d) |
| A26·C1(b) | same | ten Main photos; limit 8 | the photos at positions 1 to 8; two left out | MC-A7 |
| A26·C1(c) | same | three Main photos; limit 1 | the photo at position 1 only, role wide | MC-A7 |
| A26·C1(d) | same | only Condition photos | none | MC-A5 |
| A26·C2(a) | a full run | a session with three Main photos | exactly one Observe request; it carries the three photos' images in position order and names them 1 wide, 2 closer, 3 closer; one `looked` step of stage 1 names the three photo ids; that call has exactly three outbound rows, one per photo | MA12, MA3 |
| A26·C2(b) | a full run | ten Main photos, default limit | the one Observe request carries eight images; the `looked` step names eight photo ids and its detail says two photos were not looked at | MC-A7 |
| A26·C3(a) | a full run | the Observe reply reads wood as Teak, sure; no listing | wood is written Teak at `medium`; its `concluded` step names the key | MA11 (MC-A4 visible) |
| A26·C3(b) | a full run | three Main photos; the Observe reply lists marking photo 3 | exactly one Labels request, for the third photo | MC-A7 |
| A26·C3(c) | a full run | three Main photos; the Observe reply lists marking photo 5 | no Labels request; the run ends `finished`; the `looked` step's detail names the ignored index | MC-A7 |
| A26·C4(a) | a second `arranged` run | after a `finished` run, a Condition photo is added; arranged again | no Observe request in the second run; its stage 1 output is a copy with `reused_from_run_id` | MA14 (MC-A7 Reuse) |
| A26·C4(b) | a second `arranged` run | three Main photos; after a `finished` run the second is recomposed; arranged again | `arranged` answers yes; the second run makes exactly one Observe request carrying all three photos | MA14 |
| A26·C4(c) | `arranged` | ten Main photos, limit 8; after a `finished` run the photo at position 10 is recomposed | `arranged` answers no; no new run | MC-A7 |
| A26·C5(a) | kill and resume | the app is killed while the Observe call is in flight; restart | the open call is closed `unknown` and one more Observe call is made; the run ends with one stage 1 output and one `looked` step of stage 1 | MA4, MA14 |
| A26·C6(a) | `ResearchSettings::default()`, `ResearchApi::validate_settings` | the default; a patch with `observe_max_photos` 0; one with 3 | the default is 8; 0 is refused and the text names the field; 3 is accepted | MC-A7 (stage 1 unit) |
| A26·C6(b) | a run after an upgrade | the latest earlier run's stage outputs carry fingerprints of version 1 (written as R11 wrote them); an `arranged` run with the same photos | the new run's settings snapshot says fingerprint version 2 and instructions version 2; it reuses none of those units | MA14 |
| A26·C7(a) | OpenAI adapter, `ModelProvider::call` | an Observe request with three photos; the reply is derived from `fixtures/openai/stage_observe_gpt-6-luna.json` (RESEARCH-VERIFY-3; envelope kept, the answer text replaced by reads and marking photos `[2]`); the three-image body shape is the one `fixtures/openai/stage_condition_gpt-6-luna.json` shows for six | the request body carries three images in order and a strict schema whose members are the reads and the marking photos; the reply is returned with marking photos `[2]` | MA16 (rule 17) |
| A26·C7(b) | same | an Observe request without `photos`, answered by `stage_observe_gpt-6-luna.json` as captured | accepted as before (schema: reads and `shows_marking`) | MA9 (replay) |

### Delegated choices
The unit key of the one Observe unit · the wording of the instructions beyond what task 3 fixes · how the roles are written in the request input.

### Notes
- Hazard (MC-A5): the gateway re-reads eligibility in the transaction that records the call and drops photos deleted since; the indexes the model answers with are indexes of the photos **sent**, so the mapping from index to photo id is made from the sent list, never from the chosen list.
- Hazard: `Evidence::photo_read` took the first photo's read, which was the wide photo's. After this unit there is one read per field; do not keep a "first wins" loop.
- The stop rule (MC-A8) is unchanged: one Observe call is one call.
- Expected cost of stage 1, to be confirmed by U4 and not asserted by any row: about USD 0.03 a piece instead of 0.10 (eight images in one request, instructions and answer paid once).

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib rules::photo_plan` · `cargo test -p psm-research --test engine photo_look::` · `cargo test -p psm-research --test engine stages::` · `cargo test -p psm-research --test engine resume::` · `cargo test -p psm-research --test engine adapter_openai::` · `cargo test -p psm-research` · `cargo test -p psm-research-tool` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 2 — The product part of a listing page

**Unit id:** `R12·U2` · **Rows:** A26·C8–C9 · **Changes app files:** no

### Goal
The light local parser of the owner's idea: the page reader returns, as the page's text, an extract made on the Mac: the title, the description the page gives in its structured data, a few stated properties, and the visible text from the page's heading on, cut short. Menus before the heading and the lists of other pieces after the product no longer reach the read model. **Not in this unit:** deciding anything from the page without the model.

### Read first
`master_plan_research.md` §3 (Rule 17), §6.8, **§7.4 (the survey of the recorded pages: where headings sit, what structured data exists)** · book `08A` §4 (page reader), `08B` MC-A3 (pages), MC-A11 (replay identity) · evidence "Page reading" · fixture `crates/research/fixtures/pages/shop_listing_page.json` (RESEARCH-VERIFY-8: the one page body committed in full) · code `crates/research/src/adapters/page_reader/{mod.rs (read_html), http.rs}`, `crates/research/tests/engine/adapter_pages.rs`.

### The extract (this is the contract of the new mechanism)
Parts in this order, each on its own labelled line or block; an empty part is left out; a text equal to an earlier part after collapsing white space is left out.
1. **Title:** `og:title`, else the name of the schema.org Product block, else `<title>` (as today).
2. **Description:** the Product block's description; `og:description`; the page's `description` meta.
3. **Stated:** from the Product block only, the properties `creator`, `manufacturer`, `material` and `productionDate`, each as "<property>: <text>", where the text is the property's string or the `name` of its object. **`brand` is never listed** (it names the seller on the recorded pages).
4. **Page text:** the visible text, starting at the first place where the text of the page's first `<h1>` occurs in it, or at its start when the page has no `<h1>` or that text does not occur; at most 10,000 characters.
A Product block that cannot be parsed is ignored. The whole extract is cut at the existing limit of 30,000 characters.

### Files
New: `crates/research/src/adapters/page_reader/extract.rs`. Edited: `crates/research/src/adapters/page_reader/mod.rs` (the text of `HtmlPage` is the extract), `crates/research/tests/engine/adapter_pages.rs`.

### Core mechanisms (the only functions with unit tests)
`adapters::page_reader::extract::product_extract` (inline unit tests on documents built in the test; one on the committed shop page).

### Ordered tasks
1. `product_extract`, from what the tokenizer pass of `read_html` already collects, plus the first `<h1>`'s text, `og:description` and the `description` meta.
2. `read_html` returns the extract as the page's text. Title and images are as before.
3. The page tests: the text assertion of A17·C1–C3 becomes the extract.

### Criteria (2 criteria, 9 rows)
Boundary: `product_extract` on an HTML document; `PageReader::read` on the plain-HTTP reader over `FakeResearchHttp`. Rows (b)–(g) are about our own cutting and ordering, on documents the test builds; rows (a) and C9(a) use the real page.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A26·C8(a) | `product_extract` | the body of `fixtures/pages/shop_listing_page.json` (RESEARCH-VERIFY-8): one `<h1>` "Swedish bookshelf in oak", `og:title` the same, a Product block with name, description and `brand` "Beyo Vintage" | the first line is the title "Swedish bookshelf in oak"; the description part contains "Bookshelf manufactured in Sweden in the 1950s-60s."; there is no stated line for `brand`; the page-text part begins with "Swedish bookshelf in oak" and is at most 10,000 characters | MA11 (rule 17) |
| A26·C8(b) | same | a document whose `<h1>` text first occurs 7,800 characters into a visible text of 20,000 (the shape of the recorded pamono pages, §7.4) | the page-text part is exactly the 10,000 characters from that place | MC-A7 (stage 3 input) |
| A26·C8(c) | same | a document with no `<h1>` (as on the recorded bukowskis pages) and 12,000 characters of visible text | the page-text part is its first 10,000 characters | MC-A7 |
| A26·C8(d) | same | a document with two `<h1>`; the first one's text does not occur in the visible text | the page-text part starts at the start of the visible text | MC-A7 |
| A26·C8(e) | same | a Product block with `creator` an object named "Niels Otto Møller" (the recorded bukowskis shape), `manufacturer` a string, `material` a string, and `brand` an object | the stated part is three lines, for creator, manufacturer and material, in that order; no line for brand | MA2 |
| A26·C8(f) | same | `og:description` equal to the Product block's description except for white space; a different `description` meta | the description part holds the Product description once and the meta text once | MC-A7 |
| A26·C8(g) | same | a Product block that is not valid JSON (seen on a recorded dealer site), an `og:title`, visible text | the extract has the title and the page text; no description and no stated part; no error | MC-A3 (changelog 11 d: the page reader never fails a call) |
| A26·C8(h) | same | a Product description of 40,000 characters | the extract is exactly 30,000 characters | MC-A7 (stage 3 input) |
| A26·C9(a) | `PageReader::read` | the plain-HTTP reader answered by `fixtures/pages/shop_listing_page.json` | `PageReply.text` equals `product_extract` of that body; title, final URL and images are as in A17·C1 | MA9 (MC-A11) |

### Delegated choices
The labels of the parts · how the first `<h1>`'s text is collected in the tokenizer pass.

### Notes
- Third-party page bodies are not committed (fixtures README), so only the owner's page is a real-page row; the survey numbers behind rows (b) and (c) are in `master_plan_research.md` §7.4 and were read from the R1 recordings outside the repo.
- The 10,000 characters are set from that survey: on the recorded pamono pages the heading sits about 7,800 characters in and the "Creator / Manufacturer" block ends within about 9,700 characters after it at the 90th percentile; every other recorded trusted site's pages have a median under 6,000 characters in all.
- The Read unit's identity does not name the extract; the instructions version (2, from U1) already separates runs before and after this milestone.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib adapters::page_reader` · `cargo test -p psm-research --test engine adapter_pages::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 3 — Same kind of piece, or no comparison

**Unit id:** `R12·U3` · **Rows:** A26·C10–C12 · **Changes app files:** no

### Goal
A page that clearly lists another kind of piece (a table when the item is a chair) is not compared with the photo. The read model, which already has the page, says so in the same call; no word list is kept on the Mac. **Not in this unit:** dropping a page before it is read; any change to what a `match` or `similar` page counts for.

### Why the read model and not a local word list
No artifact holds the words for kinds of furniture in the languages the listing sites use (the recorded titles are Swedish, Danish, English, German), and a title often names no kind at all ("Niels Møller model 71, set of 6"). The read call is made anyway and costs nothing more for one more answer. The rule fails open: when the model is unsure or gives no answer, the page is compared.

### Read first
`master_plan_research.md` §6.8 · book `08A` §5 (stages 3 and 4), `08B` MC-A4 (only `match` and `similar` count), MC-A7 (units 3 and 4) · `backlog_research/owner_ideas.md` lever 3 · evidence "Model comparison" (reading listings: "wrong is mostly a page about another piece") · code `crates/research/src/{stages.rs (read, compare, execute), decide.rs (Evidence::listings), adapters/openai/schema.rs}`, `crates/research/tests/engine/{stages.rs, adapter_openai.rs}`.

### Files
Edited: `crates/research/src/{stages.rs, adapters/openai/schema.rs, fakes.rs}`, `crates/research/tests/engine/{stages.rs (helpers), main.rs}`. New: `crates/research/tests/engine/same_kind.rs`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. The Read request's input gains the piece: the item type of the run's form snapshot. Instructions, version 2: besides the statements, say whether the page lists the same kind of piece, answering no only when it clearly lists another kind, and name the kind the page lists. The item type joins the Read unit's identity.
2. The Read unit's output keeps that answer and the page's kind. No answer (the model call was not determined) counts as the same kind.
3. Compare: no unit for a page whose read says another kind. The page's read step gets the detail "lists another kind of piece (<kind>): not compared". The page still counts toward `max_pages` (it was obtained).
4. OpenAI adapter: for a Read request whose input carries the piece, the strict schema is the statements plus the same-kind answer and the page's kind; a request without it keeps the schema it had.

### Criteria (3 criteria, 6 rows)
Boundary: full runs on the research harness with scripted fakes; the OpenAI adapter over `FakeResearchHttp`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A26·C10(a) | a full run | two obtained pages with images; the read of the first answers another kind ("table"), the second the same kind | exactly one Compare request, for the second page; the first page's read step carries "lists another kind of piece (table): not compared" | MC-A7 |
| A26·C10(b) | a full run | the Read request of a run whose item type is "Dining Chairs" | the request's input names the piece as "Dining Chairs" | MC-A7 |
| A26·C10(c) | a full run | one obtained page with images whose read model call is refused | one Compare request for that page | MC-A9 (refusal) |
| A26·C10(d) | a full run | `max_pages` 1; the first page in reading order is obtained and answers another kind; a second candidate exists | the second candidate is not read | MC-A3 (reading order) |
| A26·C11(a) | a full run | designer is an attribution key set by the test; a trusted page answers another kind and states designer X; nothing else | designer is not determined; no row is written; X is not named among other values | MA2 |
| A26·C12(a) | OpenAI adapter, `ModelProvider::call` | a Read request with the piece, answered by a reply derived from `fixtures/openai/stage_read_gpt-6-luna.json` (RESEARCH-VERIFY-3; envelope kept, the answer text replaced by statements, same kind no, kind "table"); then a Read request without the piece answered by that capture as it is | the first: the strict schema has the three members and the reply is returned with same kind no; the second: accepted as before | MA16 (rule 17), MA9 |

### Delegated choices
The member names of the two new answers · the wording of the instruction.

### Notes
- A page with no verdict already counts for nothing in every decision (MC-A4: only `match` and `similar` count); this unit adds no rule there, row C11(a) pins it for this path.
- Carried from the R10 review: C11(a) sets `attribution_keys` itself.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine same_kind::` · `cargo test -p psm-research --test engine stages::` · `cargo test -p psm-research --test engine adapter_openai::` · `cargo test -p psm-research` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 4 — The second acceptance measurement

**Unit id:** `R12·U4` · **Rows:** A26·C13–C16 · **Changes app files:** no

### Goal
Run the 42 pieces again on the engine as it now ships, and record the result beside the first measurement so cost per piece and accuracy can be compared line by line. **Not in this unit:** any change to the engine (a defect found is a finding for the orchestrator); any change to the shipped defaults.

### How this unit runs
Owner-attended, like R9·U2. The implementer builds tasks 1–2 and their two automated rows. **Tasks 3–6 are the orchestrator's, with the owner present.** **Spending ceiling: USD 20** (`--ceiling-usd 20`; the scratch settings' day cap is 20; the first measurement cost about USD 12.6 and this one is expected under 9). Keys come from `~/.psm_research.env` (`master_plan_research.md` §7.6), never from the Keychain. Out folder `~/psm-research-bench/a26`, outside the repo.

### Read first
`master_plan_research.md` §3 (Rule 9, Keys, Spending ceilings), §7.2, §7.6 · the R9 plan, Unit 2 (how the first measurement was run and scored) · evidence "Owner answers", "Acceptance measurement", "Model comparison", "Owner pick of models" · `docs/test-items/{beyo_test_samples_1.json, answer_key_corrections.json}` · code `crates/research-tool/src/{accept.rs, score.rs, lib.rs}`, `crates/research-tool/tests/tool/u4_accept.rs`.

### Files
Edited: `crates/research-tool/src/accept.rs`, `crates/research-tool/tests/tool/u4_accept.rs`, `docs/intention/evidence/research_providers.md` (a new section "Acceptance measurement 2 (R12)", written by the orchestrator's run). New: `docs/test-items/accept_settings_owner.json`. Outside the repo: `~/psm-research-bench/a26/`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `accept` builds its settings from the shipped ones: the stages, prices and limits of `ResearchSettings::default()` (so the measured models are R10's), then the measurement's own entries as before (the two attribution keys of the test file, the twelve trusted sites, the visual search priced at 0, the day cap), then the `--settings` file on top. It no longer carries its own model names.
2. `docs/test-items/accept_settings_owner.json`: the owner's shop set-up for R11's settings. `excluded_keys`: `custom.totalwidth`, `custom.totaldepth`, `custom.totalheight`, `custom.seatheightchairs`, `custom.weight_definition`, `custom.damage_details`. `extra_shop_hosts`: `beyovintage.com`, `beyovintage.myshopify.com`, `68c5b4-6a.myshopify.com` (evidence "Owner answers" 3). `detailed_condition`: the key `custom.detailed_condition` and the levels of the owner's answer to R11's card; **if the card is unanswered the file has no `detailed_condition` and the evidence section says detailed condition was not derived**.
3. `[manual]` Run `accept` over all 42 pieces with that settings file and the ceiling; then `score-accept`.
4. `[manual]` Record in the evidence doc, under "Acceptance measurement 2 (R12)", the same tables as the first measurement, and one comparison table, first against second: cost per piece (median, range) per group; time per piece (median); the share of cost per stage; designer and maker (sure right, sure wrong, low right, low wrong, left open) per group; each other scored field right, wrong, open. State what differs in the set-up (models of R10, fields left alone, detailed condition derived or not).
5. `[manual]` Check over every run's `writes.json` and steps: no attribution above `low` without a read label or `match` pages on two trusted sites (MA2 as R11 amended it: a page of the shop no longer qualifies); none of the six left-alone keys written; spend within the ceiling. Record the three answers in this plan's Review log.
6. Findings: anything that contradicts a contract is reported to the orchestrator with the run id; it is not fixed here. If accuracy of the photo look fell against the first measurement, say so in the narrative with the numbers: the owner decides whether U1 stays.

### Criteria (4 criteria, 5 rows, 3 `[manual]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A26·C13(a) | `accept` (HTTP seam stub replaying captures, as A23·C2(a)) | two pieces, no `--settings` | each run's settings snapshot has, for every stage, the model and effort of `ResearchSettings::default()`; its attribution keys are the test file's two | MA2, MA5 (rule 10) |
| A26·C13(b) | `accept` | the same two pieces with `--settings docs/test-items/accept_settings_owner.json` | each run's settings snapshot carries the six left-alone keys and the three extra shop addresses; none of the six keys is among the fields of any model request the stub received | MA10 |
| A26·C14(a) | evidence doc `[manual]` | all 42 pieces | the tables of task 4, including the comparison with the first measurement | `[rule]` 08A §11 |
| A26·C15(a) | Review log `[manual]` | every run of the measurement | the three checks of task 5, each with its count | MA2, MA10, HA6 |
| A26·C16(a) | evidence doc `[manual]` | the measurement's call records | per stage: USD per piece (median) and share of the total, beside the first measurement's (observe 32 %, read 26 %, compare 23 %) | MA5 |

### Delegated choices
Table layout · which pieces illustrate a difference in the narrative.

### Notes
- The photos of the test pieces are finished shop images, not studio previews, and they are all Main: the first is the cover. Results on live captures may differ; the narrative says so.
- The tool starts each run by the manual request, which reuses nothing; that is what a measurement wants.
- Visual search is priced 0 in the measurement and USD 0.025 in the app (R10 review note): a cost per piece seen in the app is that much higher.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research-tool u4_accept::` · `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test -p psm-research` · `cargo test -p psm-research-tool` · `cargo test -p psm-testkit --test suite research::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`. One cargo command at a time. **No full app suite** (no app file changed; flow §10). The `research::` stems of the app suite are run once because they drive the wired engine with the crate's fakes (R7·U1): a change of the stage 1 unit can change them although no app file changed. If one fails, the fix is in the research crate's fakes, not in an app file; if only an app file could fix it, report `STATE: BLOCKED`.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-research` · `cargo test -p psm-research-tool` · `cargo test -p psm-testkit --test suite research::` · the two clippy commands · fmt · `cargo check --workspace`. **No full app suite.** Only B1–B4 block. The `[manual]` rows are judged from the evidence doc section and the Review log entry when the orchestrator has run the measurement; before that they are open, and the review covers U1–U3 and the two automated rows of U4. The reviewer also checks that every test of "Tests that pin what this plan changes" was handled as the table says. Notes go to `docs/implementation/backlog_research/R12_research_cost.md`.

## Manual try-out (the narrative is built from this)
On scratch data (`PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`), shop connected, keys entered, set up as in R11's try-out:
1. Shoot one piece with five or six photos, the first one wide. Sort them and press **Arranged — next**.
2. Open that item's form and the research panel: there is one "Looked at n photos" step instead of one per photo, and the run is finished sooner than before.
3. Settings → Research → Export research for today; in the zip, the calls folder of that run holds one observe call. Its cost is in `run.json`.
4. After the orchestrator's measurement: read "Acceptance measurement 2 (R12)" in the evidence document. The comparison table says what a piece costs now and whether designer, maker and wood are right as often as before.

## Review log
