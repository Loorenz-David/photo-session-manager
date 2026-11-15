---
plan: R12_research_cost
role: implement
round: 1
state: DONE
date: 2026-11-15
actor: opus (claude, pipeline-implementer)
---
# Handoff — R12_research_cost, implement round 1

Gate: intention RATIFIED (2026-10-24); `R11_research_on_arranged` Tracker line APPROVED (2026-11-15);
R12 was NOT_STARTED. As the plan says for a `run_phase.sh` launch, U1–U3 and U4 tasks 1–2 are built
with their automated rows; **U4 tasks 3–6 (the 42-piece measurement, rows A26·C14–C16 `[manual]`) are
the orchestrator's, with the owner present** (keys from `~/.psm_research.env`, `--ceiling-usd 20`,
`--settings docs/test-items/accept_settings_owner.json`, out folder `~/psm-research-bench/a26`).

## What was built
- **The photo look in one call (U1).** Research now looks at a piece's photos together in **one**
  model call instead of one call per photo: at most 8 photos (a new setting, "observe_max_photos"),
  Main photos first in their shooting order, then unreviewed ones; Condition photos never. The model
  is told photo 1 is the wide shot (what the piece is, how many) and the others are closer shots
  (wood, material, colour, surface), gives one answer per field, and lists which photos show a label
  or stamp; only those get a label read. The panel shows one "Looked at n photos" step; when photos
  were left out it says how many. A change to a photo beyond the 8 starts no new run. Runs made
  before this milestone are never reused (fingerprint and instructions version 2). The shipped
  settings now carry the owner's detailed-condition table (None/Minor → Very Good, Moderate → Good,
  Major → Worn, never Excellent; N/A counts for nothing).
- **The product part of a listing page (U2).** The page reader hands the read model a short extract
  instead of the whole page: the title, the page's descriptions, what its product data states for
  creator, manufacturer, material and production date (never the "brand", which names the seller),
  and the visible text from the page's heading on, at most 10,000 characters. Menus before the
  heading and "more from this dealer" lists far below no longer reach the model.
- **Same kind of piece, or no comparison (U3).** The read model is told what the piece is (the item
  type) and says whether the page lists the same kind; a page that clearly lists another kind (a
  table when the item is a chair) is not compared with the photo, and its read step says so. When the
  model is unsure or gives no answer, the page is compared as before.
- **The second measurement, prepared (U4 tasks 1–2).** The measurement tool now measures the shipped
  settings (R10's models and prices) instead of carrying its own model names; the owner's shop
  set-up is a file (`docs/test-items/accept_settings_owner.json`: six fields left alone, three extra
  shop addresses, the detailed-condition table).

## Tests → criterion rows
| Row | Test | Assertion shape |
|---|---|---|
| A26·C1(a) | `psm-research --lib rules::photo_plan::tests::a26_c1a_mains_by_position_then_unreviewed_never_condition` | exact ids and roles, left_out 0 — as the row |
| A26·C1(b) | `…::a26_c1b_ten_mains_limit_eight` | exact ids of positions 1–8, left_out 2 — as the row |
| A26·C1(c) | `…::a26_c1c_limit_one_keeps_the_wide_photo` | exactly `[(pos 1, wide)]` — as the row |
| A26·C1(d) | `…::a26_c1d_only_condition_photos_none` | empty plan — as the row |
| A26·C2(a) | `engine photo_look::a26_c2a_one_observe_request_for_three_photos_in_position_order` | one request; images equal the three renders in position order (third photo reordered to 1); input `photos` exact; one `looked` step with the three ids; outbound rows of that call = the three photos — as the row |
| A26·C2(b) | `photo_look::a26_c2b_ten_photos_default_limit_sends_eight` | 8 images; step ids = positions 1–8; detail contains "2 photos were not looked at" — as the row |
| A26·C3(a) | `photo_look::a26_c3a_the_observe_read_writes_a_visible_field` | woods `["Teak"]` at medium; one `concluded` step naming the key — as the row |
| A26·C3(b) | `photo_look::a26_c3b_a_listed_marking_photo_gets_one_labels_request` | exactly one Labels request, unit and image of the third photo — as the row |
| A26·C3(c) | `photo_look::a26_c3c_an_index_outside_the_photos_sent_is_ignored` | no Labels request; `finished`; detail names index 5 — as the row |
| A26·C4(a) | `photo_look::a26_c4a_a_new_condition_photo_reuses_the_observe_unit` | no Observe request; stage 1 output `reused_from_run_id` = first run — as the row |
| A26·C4(b) | `photo_look::a26_c4b_a_recomposed_photo_costs_one_observe_call_over_all` | `arranged` true; one Observe request carrying the three renders — as the row |
| A26·C4(c) | `photo_look::a26_c4c_a_photo_beyond_the_limit_changes_nothing` | `arranged` false; one run — as the row |
| A26·C5(a) | `photo_look::a26_c5a_kill_while_the_observe_call_is_in_flight` | observe calls `[(unknown,1),(answered,2)]`; one stage 1 output; one `looked` step — as the row |
| A26·C6(a) | `photo_look::a26_c6a_observe_max_photos_default_and_validation` | default 8; 0 refused naming the field; 3 accepted — as the row |
| A26·C6(b) | `photo_look::a26_c6b_units_of_version_1_are_not_reused` | earlier failed run with R11-form v1 outputs (Observe per photo, Find with the same identity as now); new arranged run: snapshot versions 2/2, no output reused, Observe and Find called — as the row |
| A26·C7(a) | `photo_look::a26_c7a_three_photos_in_one_request_with_marking_photos` | body content `[text, image×3]` with the three data URLs in order; strict schema members exactly `marking_photos`, `reads`; reply `marking_photos [2]` — as the row |
| A26·C7(b) | `photo_look::a26_c7b_a_request_without_photos_keeps_the_one_photo_schema` | schema `reads`, `shows_marking`; **deviation**: the capture as captured is refused (Schema) as before; a derived answer is accepted — see Deviations 1 |
| A26·C8(a) | `--lib adapters::page_reader::extract::tests::a26_c8a_the_committed_shop_page` | first line `Title: Swedish bookshelf in oak`; description contains the sentence; no brand line; page text starts with the heading, ≤ 10,000 — as the row |
| A26·C8(b) | `…::a26_c8b_page_text_starts_at_the_heading` | page-text part equals `visible[7800..17800]` exactly — as the row |
| A26·C8(c) | `…::a26_c8c_no_heading_takes_the_first_10000_characters` | equals first 10,000 — as the row |
| A26·C8(d) | `…::a26_c8d_a_first_heading_not_in_the_visible_text_starts_at_the_start` | exact page text from the start — as the row |
| A26·C8(e) | `…::a26_c8e_stated_lists_creator_manufacturer_material_never_brand` | stated part exactly three lines in order; brand name absent — as the row |
| A26·C8(f) | `…::a26_c8f_a_description_equal_but_for_white_space_is_kept_once` | descriptions exactly `[product, meta]` — as the row |
| A26·C8(g) | `…::a26_c8g_a_product_block_that_is_not_json_is_ignored` | extract equals title + page text exactly — as the row |
| A26·C8(h) | `…::a26_c8h_the_whole_extract_is_cut_at_30000_characters` | exactly 30,000 chars — as the row |
| A26·C9(a) | `engine adapter_pages::a26_c9a_the_page_text_is_the_product_extract` | `text == product_extract(body)`; title, final URL, images — as the row (A17·C1(a)'s text assertion also became this equality) |
| A26·C10(a) | `engine same_kind::a26_c10a_a_page_of_another_kind_is_not_compared` | Compare requests exactly `[e2]`; e1 read detail exact — as the row |
| A26·C10(b) | `same_kind::a26_c10b_the_read_request_names_the_piece` | `input.piece == "Dining Chairs"` — as the row |
| A26·C10(c) | `same_kind::a26_c10c_a_refused_read_still_compares` | Compare requests exactly `[e1]` — as the row |
| A26·C10(d) | `same_kind::a26_c10d_a_page_of_another_kind_counts_toward_the_page_limit` | reader URLs exactly `[e1]` — as the row |
| A26·C11(a) | `same_kind::a26_c11a_another_kind_names_no_designer` | no row; `not_determined` names the key; "Designer X" in no decide step — as the row (sets `attribution_keys` itself) |
| A26·C12(a) | `same_kind::a26_c12a_the_read_schema_gains_the_kind_answer_only_with_the_piece` | with piece: members exactly `keys, page_kind, same_kind`, reply same_kind false; without: members `keys`, capture as it is accepted — as the row |
| A26·C13(a) | `psm-research-tool u4_accept::a26_c13a_accept_measures_the_shipped_models` | every stage's model/effort = `ResearchSettings::default()`; attribution keys = the two — as the row |
| A26·C13(b) | `u4_accept::a26_c13b_the_owner_settings_leave_six_fields_alone` | snapshots carry the six keys and three hosts; none of the six in any model request's fields (plus: without the file some do — not vacuous) — as the row |
| A26·C14(a), C15(a), C16(a) | `[manual]`, not run here | the orchestrator's measurement |
| Owner row (carried, no id) | `engine settings_r11::a26_owner_detailed_condition_default` | the default table derives Very Good / Good / Worn from the shop's own choices, never Excellent |

Every test in the two new test files and the new inline modules is in this map. Amended existing
tests are listed under Deviations 2.

## Assumptions
1. Observe unit key `photos`; the model's member `marking_photos` (indexes from 1); roles in the input as `{"index": n, "role": "wide"|"closer"}`.
2. Step texts: "Looked at n photos" · "n photos were not looked at (the limit is 8)" · "marking photo 5 ignored: only 3 photos were sent" · "Could not read the photos".
3. Whether a new run is needed still compares the chosen photos as a **set**: reordering two closer photos does not by itself start a run (the existing "non-cover swap queues nothing" row stays true); a new cover does.
4. If a photo is deleted between rendering and sending, the input still names it; marking indexes are mapped through the photos actually sent (plan note, MC-A5).
5. Detailed-condition default: key `custom.detailed_condition`, the shop's full choice strings; **N/A counts for nothing** (the owner's answer does not mention N/A).
6. Extract labels `Title: `, `Description: ` (one line per text), `Stated:` block, `Page text:` block; the first `<h1>` counts even if hidden.
7. A page with no visible text but a title or description is now read (the HTTP reader checks the extract for emptiness; that file was not in U2's list).
8. The `piece` is sent only when the item type is not empty; it is always in the Read identity. Model members `same_kind` (true/false/null) and `page_kind`.
9. The measurement's day cap is USD 20 (the plan's U4 text), not R9's 25.

## Deviations
1. **A26·C7(b):** the row says the observe capture "as captured" is accepted; it cannot be — its answer is R1's version-0 shape (`fields`, `piece_type`), which no engine Observe schema has accepted (`a15_c1a` records the same for the Sol capture). The test pins the old schema for requests without `photos`, the as-captured refusal unchanged from before R12, and acceptance of an answer in that schema.
2. **Existing tests amended beyond the plan's pin table** (each forced by the plan's own rules): `a04_c7a` (fingerprint bytes `"v":2`); `a14_c3c` (a new cover changes the Observe identity: one Observe call, not reused); `a15_c6a` (one observe reply; Read answers gain the two members after U3); `A25·C11(a)` (detailed-condition assertions removed; the owner row replaces them); `a12_c6b` (one refused Observe unit: woods not written); `export.rs` helper `Six` → `Four`. As the table says: `a14_c1a` deleted; `a14_c3a`, `a14_c3d` amended; `a12_c1a–c1c`, `a12_c3a`, `a12_c3b` keep ids and outcomes with one Observe unit.
3. **Files outside a unit's list:** `crates/research/src/rules/run_rules.rs` (doc comments only); engine tests `api.rs`, `export.rs`, `settings_r11.rs` (U1) and `adapter_openai.rs` (U3) for the script changes; `harness.rs` and (in U1/U3) `fakes.rs` needed no change.
4. **Closing gate fix in the research crate's fakes:** `research::r11_arranged::a25_c20b` and `a25_c20e` (app tests) script the Observe reply under the old key `photo:<id>`. As the Closing gate directs, `FakeModel` now answers the one Observe unit, when it is not scripted itself, with a reply scripted for one of its photos under that key. No app file changed; the failing stem was re-run green.
5. No red baseline recorded (flow §4 has no mutation regime).

## Files changed
Production: `crates/research/src/rules/photo_plan.rs` (new), `crates/research/src/rules/{mod.rs, fingerprint.rs, run_rules.rs}`, `crates/research/src/{config.rs, stages.rs, decide.rs, runs.rs, fakes.rs}`, `crates/research/src/adapters/openai/schema.rs`, `crates/research/src/adapters/page_reader/{extract.rs (new), mod.rs}`, `crates/research-tool/src/accept.rs`.
Tests: `crates/research/tests/engine/{photo_look.rs (new), same_kind.rs (new), main.rs, stages.rs, resume.rs, adapter_openai.rs, adapter_pages.rs, api.rs, export.rs, settings_r11.rs}`, `crates/research-tool/tests/tool/{u3_report_and_replay.rs, u4_accept.rs}`.
Docs and data: `docs/test-items/accept_settings_owner.json` (new), `docs/implementation/master_plan_research.md` (§6.3 the two version constants), `docs/implementation/plans/R12_research_cost.md` (Tracker line, Review log), this handoff.
No mutation-probe files (no mutation regime). No architecture graph in this project. Nothing outside the repo was written.

## Checkpoint SHA
In order: U1 `af0e9ce` · U2 `64ef178` · U3 `6e7c9d0` · U4 tasks 1–2 `5a2d855` · closing (fakes fix, tracker, Review log, this handoff): the `CHECKPOINT (not approved): R12 closing gate` commit that contains this file (its SHA is in the session's final message).

## Stamp
Closing gate, once, one command at a time, on the tree of `5a2d855` plus the `fakes.rs` fix:
| Command | Result |
|---|---|
| `cargo test -p psm-research` | ok — 154 unit, 222 engine, 1 doc |
| `cargo test -p psm-research-tool` | ok — 28 |
| `cargo test -p psm-testkit --test suite research::` | first run: 75 passed, 2 failed (`r11_arranged::a25_c20b`, `a25_c20e`, old Observe unit key); fixed in `fakes.rs`; re-run `research::r11_arranged::` ok — 16 |
| `cargo clippy -p psm-research --all-targets -- -D warnings` | ok |
| `cargo clippy -p psm-research-tool --all-targets -- -D warnings` | ok |
| `cargo fmt --all -- --check` | ok |
| `cargo check --workspace` | ok |
No full app suite (no app file changed; flow §10). Pre-existing failures: none seen.

## Manual try-out
On scratch data (`cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`), shop connected, keys entered, set up as in R11's try-out:
1. Shoot one piece with five or six photos, the first one wide. Sort them and press **Arranged — next**.
2. Open that item's form and the research panel: one "Looked at n photos" step instead of one per photo.
3. Settings → Research → Export research for today; in the zip, the run's `calls` folder holds one observe call (its request lists the photos with their roles); the cost is in `run.json`.
4. A listing page about another kind of piece shows "lists another kind of piece (…): not compared" on its read step.
5. After the orchestrator's measurement: "Acceptance measurement 2 (R12)" in `docs/intention/evidence/research_providers.md`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. (Assumption 5, N/A counting for nothing in the default detailed-condition table, is a setting the owner can change in Settings → Research.)
