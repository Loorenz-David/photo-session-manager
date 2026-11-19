---
phase: W1_product_note
title: What research saw — research's two photo looks also write a short factual note, each observation tagged with the fields it bears on; your review decisions replace what research saw on those fields; the research panel shows the note, says when the photos changed since they were looked at, and offers Look again for the two photo looks only
difficulty: high
projection: no
depends_on: []
blocked_on: none
---
# W1 — The product note

**Tracker:** APPROVED (2026-11-19, opus reviewer r1: APPROVED_WITH_NOTES; 54 rows verified at their boundaries, review-mode checks green, 8 notes in `backlog_writer/W1_product_note.md`)

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. `master_plan.md` §4 and `STATUS.md` are not used or edited by writer milestones.)

## Goal
Book `08C` §4 by contract WC1 and the decision rules of WC2 (`docs/implementation/master_plan_writer.md` §5). (U1) Stages 1 and 6 ask for, parse and store the note; the instructions version moves to 3. (U2) Which note counts, the owner's decisions over it, staleness, and the `look` run. (U3) The two commands and the note in the research panel. **Not here:** the writer (W2), anything the writer sends or writes, Settings → Writer.

## How to work this plan (flow.md §10, master_plan_writer.md §3)
One runner phase of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `W1·C<n>(<row>)`; tests named after their row (`w1_c3a_<slug>`). Open details: the smallest option consistent with 08C and WC1/WC2, recorded under **Assumptions** in the handoff. Reviewer notes: `docs/implementation/backlog_writer/W1_product_note.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The note in stages 1 and 6 | 5 | 12 | no |
| U2 | Which note, the owner's decisions, staleness, Look again | 7 | 36 | no |
| U3 | The commands and the note in the research panel | 2 | 6 | **yes** |
| **Total** | | **14** | **54** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §10) · `docs/implementation/master_plan_writer.md` (all; **§5 WC1, WC2 are the contract**) · `docs/implementation/master_plan_research.md` §6.3, §6.4, §6.5, §7.2 · book `08C` §3, §4, §6 · `08B` MC-A1 (cleared set), MC-A6, MC-A7, MC-A12 · `docs/guide/02_architecture.md` §Research · `docs/guide/04_change_impact.md` §Research.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **The note is written by the calls research already makes;** no extra call, a paragraph more output.
- **Every stage is looked at again once** after this change, because the instructions changed (a one-time cost on the next research of each session).
- **Your decision replaces what research saw**, and a value you typed by hand also counts as yours.
- **"Photos changed" means a photo research would look at now differs** from the ones it looked at (another photo among the first ones, a recompose, other condition photos); a new photo beyond the photo limit is not a change.
- **Look again looks at the photos only**: no listing search, no new suggestions; units whose photos did not change are copied, not paid again.
- **Look again is offered only when the photos changed** (or research never got to look at them) and nothing else runs on the session.

## Unit 1 — The note in stages 1 and 6

**Unit id:** `W1·U1` · **Rows:** W1·C1–C5 · **Changes app files:** no

### Goal
WC1 items 1–4: request, schema, stored shape, reuse.

### Read first
`crates/research/src/stages.rs` (`OBSERVE`, `CONDITION`, `observe`, `condition`, `reads`, the `marking_photos` index mapping), `crates/research/src/adapters/openai/schema.rs` (`schema_for`), `crates/research/src/config.rs` (`INSTRUCTIONS_VERSION`), `crates/research/src/fakes.rs` (`FakeModel::reply`), `crates/research/tests/engine/{harness.rs, stages.rs, photo_look.rs}`.

### Files
Edited: `crates/research/src/{stages.rs, config.rs, adapters/openai/schema.rs}`, `crates/research/tests/engine/main.rs`. New: `crates/research/tests/engine/w1_note.rs`. Existing engine tests whose stage 1 or 6 fixtures assert an exact output object or an instructions version literal follow the new shape (find them by grep for `"marking_photo_ids"`, `instructions_version`).

### Core mechanisms (the only functions with unit tests)
`schema_for` (Observe and Condition entries) · the note parser in `stages.rs` (one function, name delegated).

### Ordered tasks
1. Stage 1 input gains `note_fields` (WC1.1); stage 6 input gains `photos` and `note_fields`. Instructions gain the sentences of master plan §9.3. `INSTRUCTIONS_VERSION = 3`.
2. `schema_for`: the two `note` entries (WC1.2).
3. Parse and store `note` in both unit outputs (WC1.3). A reply without `note` stores `[]`.
4. Tests for the rows.

### Criteria (5 criteria, 12 rows)
Boundary: `schema_for` (pure); a research run on the engine harness with `FakeModel` (outcome = the requests the fake received, the committed unit outputs).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W1·C1(a) | `schema_for` | an Observe request whose input has `photos` and two fields | the schema's top-level properties are `reads`, `marking_photos`, `note` (all required); `note.items` has properties `text` (string) and `keys` (array of string), both required, `additionalProperties: false` | WC1.2 |
| W1·C1(b) | same | a Condition request with two fields | top-level `reads`, `note`; `note.items` properties `photo` (integer), `text`, `keys`, all required | WC1.2 |
| W1·C2(a) | a run, stage 1 | layout: Details `custom.wood` (in `fill_keys`), Details `custom.shape` (not in `fill_keys`), Condition `custom.condition_scratch`, Product `product.title`; one Main photo | the stage 1 request's `note_fields` keys are exactly `custom.wood`, `custom.shape`, `custom.condition_scratch` in layout order, each with its label; its `fields` list only `custom.wood` (unchanged rule); the instructions end with the note sentence of §9.3 | WC1.1 |
| W1·C2(b) | two runs | run 1 committed stage 1 with a settings snapshot whose `instructions_version` is 2 (written by the test); run 2 (`arranged`, same photos) | run 2 sends a stage 1 request (no reuse) and its unit identity has `instructions_version` 3 | WC1.1, MC-A7 |
| W1·C3(a) | a run, stage 1 | the fake answers `note: [{text: "Frame in teak", keys: ["custom.wood", "custom.unknown", "custom.wood"]}, {text: "Four tapered legs", keys: []}]` | the stage 1 output's `note` is `[{text: "Frame in teak", keys: ["custom.wood"]}, {text: "Four tapered legs", keys: []}]` | WC1.3 |
| W1·C3(b) | same | one observation `"   "`, one of 350 characters ending in "ø" past character 300 | the blank one is absent; the long one is stored with exactly its first 300 characters | WC1.3 |
| W1·C3(c) | same | 25 observations | the first 20, in reply order | WC1.3 |
| W1·C3(d) | same | a reply with no `note` key | the output has `"note": []` and the run finishes | WC1.3 |
| W1·C4(a) | a run, stage 6 | three Condition photos with sources, a Condition field chosen | the request's `photos` is `[{index:1},{index:2},{index:3}]` in the order the three images are attached; `note_fields` are the Condition-step keys | WC1.1 |
| W1·C4(b) | same | the second photo is deleted between the render and the send (the gateway drops it, as in the existing dropped-photo test); the fake answers notes for photos 1 and 2 of the two images it received | the note for index 1 carries the first photo's id, the note for index 2 the third photo's id | WC1.3 |
| W1·C4(c) | same | a note with `photo: 7` (out of range) | stored with `photo_id: null` | WC1.3 |
| W1·C5(a) | two `arranged` runs, nothing changed between them | run 1 committed stage 1 with a note | run 2 makes no stage 1 call and its stage 1 output (copied, `reused_from_run_id` = run 1) carries run 1's note unchanged | WC1.4, MC-A7 |

### Delegated choices
The note parser's name and module · how `note_fields` labels are read (the run's form snapshot or the definitions).

### Notes
- Hazard: many engine tests reply with a fixed stage 1 object; a reply without `note` must keep working (C3(d)), so they need no change unless they assert the full output object.
- Recorded captures under `crates/research/fixtures/openai/` have no `note`; a replay of them still parses (C3(d)).

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine w1_note::` · `cargo test -p psm-research --lib adapters::openai::schema` · `cargo test -p psm-research` (the instructions version touches identities everywhere) · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo check --workspace`

## Unit 2 — Which note, the owner's decisions, staleness, Look again

**Unit id:** `W1·U2` · **Rows:** W1·C6–C12 · **Changes app files:** no

### Goal
WC1 items 5–9 and the decision rules of WC2 (decision, reviewed, the note as the writer will receive it). `WriterApi` with `note` and `look_again`.

### Read first
Master plan §5 WC1.5–9, WC2 · `crates/research/src/{runs.rs (arranged, run_again, status, overview, tick, start, may_start_run), store.rs (runs, all_runs, runs_in_state), stages.rs (execute, usable_photos, photo_plan, reuse), rules/run_rules.rs (start_order, may_start), rules/photo_plan.rs, domain.rs (RunTrigger), api.rs, api_impl.rs, service.rs}` · `crates/core/src/services/attributes.rs` (feedback, `form_get`'s `title_from_integration`) · `crates/core/src/mechanisms/values.rs` (`selected_condition_photos`).

### Files
New: `crates/research/src/writer/{mod.rs, api.rs, note.rs}`. Edited: `crates/research/src/{lib.rs, domain.rs, runs.rs, store.rs, stages.rs, rules/run_rules.rs, api_impl.rs}`, `crates/research/tests/engine/w1_note.rs`.

### Core mechanisms (the only functions with unit tests)
`writer::note::decision` · `writer::note::reviewed` · `writer::note::stale`.

### Ordered tasks
1. `RunTrigger::Look` (`"look"`). The research views exclude `look` and `write` (W2 adds the variant; exclude the string now) as WC1.9 lists; `run_again` cancels a queued `look`/`write` run.
2. `writer::note::decision`, `reviewed` (WC2), over the row, the latest feedback row and (for `product.title`) the stored state of WC4.3.
3. Note selection (WC1.5), photo numbers (WC1.6), replacement and `research_saw` (WC2 "Input": `note` and `research_saw` only), staleness (WC1.7): `writer::note::assemble(session) -> NoteView {observations, research_saw, stale_main, stale_condition, missing_main}`; each observation carries its `ref`, original text, shown text, keys, photo label, `owner`, and the decision of each key.
4. `look_again` (WC1.8) and the `look` run's execution: stages 1 and 6 only, reuse as WC1.8, ends `finished`, no write. Start order tier 0 for `look` (WC3 "Start").
5. `WriterApi { note, look_again }` implemented for `ResearchService`.
6. Tests for the rows.

### Criteria (7 criteria, 36 rows)
Boundary: the named pure function; `WriterApi` and `ResearchApi` on the engine harness (outcome = the returned view, run rows, the requests the fakes received, attribute rows). Rows (a)–(k) of C6 are adjacent pairs of WC2's order.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W1·C6(a) | `decision` | row `ai` + `cleared` | `Dismissed` | WC2 (rule 1) |
| W1·C6(b) | same | row `human` + `cleared` | `Dismissed` | WC2 (rule 1) |
| W1·C6(c) | same | row `default`, live, `valid = false` | `Empty` | WC2 (rules 1/2) |
| W1·C6(d) | same | row `ai` + `suggested` at `low` | `Unchecked` (low) | WC2 (rules 2/3) |
| W1·C6(e) | same | row `ai` + `accepted` | `Accepted` | WC2 (rules 3/4) |
| W1·C6(f) | same | row `human` live; feedback rows `kept_previous` (at 1) then `edited` (at 2) | `Corrected` | WC2 (rule 5; latest feedback) |
| W1·C6(g) | same | row `human` live; no feedback | `Typed` | WC2 (rule 5) |
| W1·C6(h) | same | row `default` live; latest feedback `kept_previous` | `KeptPrevious` | WC2 (rule 6) |
| W1·C6(i) | same | row `default` live; no feedback | `ShopDefault` | WC2 (rule 6) |
| W1·C6(j) | same | row `integration` live; latest feedback `cleared` (an older clear, then the shop value read again) | `Imported` | WC2 (rule 6) |
| W1·C6(k) | same | no row | `Empty` | WC2 (rule 7) |
| W1·C7(a) | `reviewed` | row `ai` + `accepted`, feedback `accepted` | true | WC2, HW3 |
| W1·C7(b) | same | feedback `cleared` exists; row now `ai` + `suggested` (a later suggestion) | false | WC2, HW3 |
| W1·C7(c) | same | row `human` live, no feedback | false | WC2, HW3 |
| W1·C7(d) | same | row `human` + `cleared`, no feedback | true | WC2, HW3 |
| W1·C7(e) | `AttributesService` then `reviewed` | an `ai` + `suggested` row; Keep previous through the service; then undo of that step | after Keep previous true; after the undo false (its feedback row is gone) | WC2, MC-A12 |
| W1·C8(a) | `note` | `N1` observation "Frame in oak" tagged `custom.wood`; the row `ai` + `accepted` "Teak" | shown text "Wood: Teak (owner's decision)", `owner` true; `research_saw` has nothing for it | WC2, 08C §4 |
| W1·C8(b) | same | the same observation; the row `human` "Teak", no feedback (`Typed`) | shown text "Wood: Teak (owner's decision)"; `research_saw` = `[{key: custom.wood, text: "Frame in oak"}]` | WC2 |
| W1·C8(c) | same | the same observation; the row `integration` "Oak" | shown text "Frame in oak", `owner` false; `research_saw` has `custom.wood` with "Frame in oak" | WC2 |
| W1·C8(d) | same | the same observation; the row `ai` + `cleared` whose cleared set is {"Oak"} | shown text "Wood: not stated (the owner dismissed Oak)", `owner` true | WC2 |
| W1·C8(e) | same | an observation tagged `[custom.wood, custom.shape]`, wood `Accepted` "Teak", shape `Empty` | shown text "Wood: Teak (owner's decision)"; `research_saw` has `custom.shape` with the original text | WC2 |
| W1·C8(f) | same | `N6` observations for Condition photos P1, P2, P3 (positions 1, 2, 3); the damage report excludes P2 | P1's shows "condition photo 1", P3's "condition photo 2"; P2's is marked "photo no longer in the condition photos" and is not among the observations the writer will receive | WC1.6 |
| W1·C9(a) | `note` | runs of the session: 1 `arranged` finished with a note, 2 `look` finished with another note, 3 `browse` | the note shown is run 2's | WC1.5 |
| W1·C9(b) | same | run 1 `arranged` with a note; run 2 `manual` whose stage 1 output has no `note` key (written by the test) | the note shown is run 1's | WC1.5 |
| W1·C10(a) | `note` | `N1` over Main P1, P2; nothing changed | `stale_main` false, `stale_condition` false | WC1.7 |
| W1·C10(b) | same | P2 recomposed (revision + 1) after the look | `stale_main` true | WC1.7 |
| W1·C10(c) | same | `observe_max_photos` 2; a third Main photo added at position 3 | `stale_main` false | WC1.7 |
| W1·C10(d) | same | `N6` over C1; a Condition photo C2 added | `stale_condition` true | WC1.7 |
| W1·C10(e) | same | no `N6`; one Condition photo; a Condition field in `fill_keys` | `stale_condition` true | WC1.7 |
| W1·C11(a) | `look_again`, then one tick and the job | `N1` stale (P2 recomposed), `N6` unchanged; keys and prices set | `queued: true`; one run, trigger `look`; it sends exactly one stage 1 request and no request of stages 2–7; its stage 6 output is copied from the earlier run; it ends `finished`; no `attribute_values` row changed; afterwards `stale_main` false | WC1.8 |
| W1·C11(b) | `look_again` | `N1` stale; the session has a `queued` `manual` research run | `queued: false`, reason code `Busy`; no new run | WC1.8 |
| W1·C11(c) | same | nothing stale, `N1` present | `queued: false`, reason `NotStale` | WC1.8 |
| W1·C11(d) | same | stale; no integration and no model key | reason `NoIntegration`; with the integration and no model key: `NoModelKey` | WC1.8 |
| W1·C12(a) | `ResearchApi::arranged` | the session's runs: 1 `arranged` `failed`, 2 `look` `finished` | `arranged` answers true and queues an `arranged` run | WC1.9, A19 |
| W1·C12(b) | `ResearchApi::status`, `overview` | runs 1 `arranged` `finished`, 2 `look` `running` | `status.run.id` is 1 and its state `finished`; `overview` reports the session from run 1 (not `researching`) | WC1.9 |
| W1·C12(c) | `ResearchApi::run` (Run again) | the session's queued run is a `look` run | that run is `cancelled` with error "research started again"; one `queued` `manual` run exists | WC1.9 |

### Delegated choices
How `look` reuses the earlier unit (the existing `reuse` with a widened lookup, or a look-specific path) · the shape of `NoteView` beyond the fields named.

### Notes
- Hazard: `store::runs` is the research views' list; `tick` reads `runs_in_state`, which must keep listing `look` runs, or a queued look run never starts (C11(a) catches it).
- Hazard: `title_from_integration` today lives inside `form_get`; WC4.3 needs the same rule from the research crate. Read it through the store (item, link title base), not by calling `form_get`.

### Unit test commands
`cargo test -p psm-research --lib writer::note` · `cargo test -p psm-research --test engine w1_note::` · `cargo test -p psm-research --test engine arranged:: overview:: lifecycle::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo check --workspace`

## Unit 3 — The commands and the note in the research panel

**Unit id:** `W1·U3` · **Rows:** W1·C13–C14 · **Changes app files:** yes

### Goal
The `WriterApi` handle in the app, the commands `writer_note` and `writer_look_again`, and the note in the research panel.

### Read first
Master plan §4 (API, Commands, Frontend) · `apps/desktop/src-tauri/src/{commands/research.rs (engine, unavailable), commands/mod.rs, lib.rs (specta_builder, the command tests)}` · `crates/integrations/src/lib.rs` (`build_research`, `ResearchWiring`, the handles carrying `research`) · `apps/desktop/src/services/research.ts` · `apps/desktop/src/features/form/{ResearchPanel.tsx, __tests__/r6_research_panel.test.tsx}` · `docs/intention/evidence/design_handoff.md`.

### Files
New: `apps/desktop/src-tauri/src/commands/writer.rs`, `apps/desktop/src/services/writer.ts`, `apps/desktop/src/features/form/ProductNote.tsx`, `apps/desktop/src/features/form/__tests__/w1_product_note.test.tsx`, `crates/testkit/tests/suite/research/w1_note_seam.rs`. Edited: `crates/integrations/src/lib.rs`, `apps/desktop/src-tauri/src/{commands/mod.rs, lib.rs, state.rs}` (as needed for the handle), `apps/desktop/src/services/bindings.ts` (generated), `apps/desktop/src/features/form/ResearchPanel.tsx`, `crates/testkit/tests/suite/research/mod.rs`.

### Core mechanisms
None.

### Ordered tasks
1. The handles that carry `research` also carry `writer: Option<Arc<dyn WriterApi>>` (the same service). Commands `writer_note(session_id) -> NoteDto` and `writer_look_again(session_id) -> RequestedDto {queued, reason: Option<ReasonDto>}`; `research_unavailable` without the engine. Regenerate the bindings.
2. `ProductNote.tsx` in the research panel, under the steps: heading "What research saw"; one line per observation with its photo label; an observation shown with the owner's decision carries a small "Your decision" mark; a banner "Photos changed since they were looked at" with **Look again** when `stale_main` or `stale_condition`, "No product note yet" with **Look again** when `missing_main`; the block re-reads on `research_changed`. A refusal shows its sentence.
3. Tests for the rows.

### Criteria (2 criteria, 6 rows)
Boundary: the wired app (`TestApp`) through the command layer; `ResearchPanel` rendered in a UI test with `services/writer.ts` mocked.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W1·C13(a) | the wired app, `writer_note` | a researched session (fakes) whose `N1` has one observation tagged with a field the owner accepted | the answer lists the observation with `owner` true and the replaced text; `stale_main` false | WC1, WC2 |
| W1·C13(b) | the wired app, `writer_look_again` | the same session after a Main photo is recomposed | answers `queued: true`; one `look` run exists; a second call answers `queued: false` with the `Busy` sentence | WC1.8 |
| W1·C13(c) | the desktop command test | an app built without the research engine | `writer_note` and `writer_look_again` answer `research_unavailable`; `bindings.ts` contains both command names | HW7, MA7 |
| W1·C14(a) | `ResearchPanel` | a note of two observations, one `owner` true | "What research saw" lists both; only the owner one carries "Your decision" | `[rule]` 08C §4 |
| W1·C14(b) | same | `stale_main` true; click **Look again** | the banner "Photos changed since they were looked at" shows; `writerLookAgain` is called once with the session id | `[rule]` 08C §4 |
| W1·C14(c) | same | no observations, `missing_main` true | "No product note yet" with a **Look again** button | `[rule]` 08C §4 |

### Delegated choices
Exact layout of the block within the panel (design handoff tokens) · whether the block is collapsible.

### Notes
- Hazard (rule 10): the wired app must hand the writer handle on the default build path, not only in tests; C13(a) runs through the wired app.

### Unit test commands
`cargo test -p psm-testkit --test suite research::w1_note_seam::` · `cargo test -p psm-desktop` (regenerates the bindings) · `cd apps/desktop && npx vitest run src/features/form` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time. A failure is fixed and only the failing files re-run.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-research` · `cargo test -p psm-testkit --test suite research::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/form` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: every research view listed in WC1.9 excludes `look` and `write`, and `tick` does not; a `look` run has no code path to `writes::suggest_in`. Notes go to `docs/implementation/backlog_writer/W1_product_note.md`.

## Manual try-out (the narrative is built from this)
On scratch data (`PSM_DATA_DIR=$HOME/psm-research-dev`), with the research keys set and a few fields chosen:
1. Shoot or import a session, press "Arranged — next", let research finish.
2. Open Details → research panel: "What research saw" lists short observations, condition ones with "condition photo n".
3. In AI review, correct one value (for example the wood). The matching observation now shows your value with "Your decision".
4. Recompose a Main photo: the banner "Photos changed since they were looked at" appears. Press **Look again**: only the photo looks run (the cost line grows by a few cents at most), no new suggestions appear, and the banner goes.

## Review log
- **2026-11-19 · implement r1 (Claude Opus 5.5, `pipeline-implementer`).** Built the three units in order, one
  checkpoint each: U1 `853cf91`, U2 `c6279fb`, U3 `06463ae`. **U1:** stage 1 sends `note_fields` (every Details and
  Condition layout key with its label), stage 6 sends `photos: [{index}]` and the Condition-step `note_fields`; the
  §9.3 sentences end `OBSERVE` and `CONDITION`; `INSTRUCTIONS_VERSION = 3`; `schema_for` adds `note` to the Observe
  (photos form) and Condition schemas; `stages::parse_note` stores the note per WC1.3 (`photo_id` mapped over the
  images actually sent). **U2:** `RunTrigger::Look`; `store::runs` and the two overview queries leave out `browse`,
  `look`, `write` (`tick`/`runs_in_state` and the export keep them); start order tier 0 = `manual`, `look`;
  `writer::note::{decision, reviewed, stale}` (pure, unit-tested) and `ResearchService::assemble` (the `NoteView`),
  `look_again` and the `look` run (`stages::look_again`: stage 1 over the current plan, stage 6 when usable Condition
  photos and a chosen Condition field exist, reuse from the latest earlier photo-looking run, ends `finished`, no
  stage 2–5/7, no write path); Run again / Arranged cancel a queued `look`/`write` run ("research started again") in
  the inserting transaction; `WriterApi {note, look_again}` in `api_impl.rs`. **U3:** `App`/`AppHandles::writer`
  (same service as `research`, default build path), `commands/writer.rs` (`writer_note`, `writer_look_again`,
  `research_unavailable` without the engine), bindings regenerated, `services/writer.ts`, `ProductNote.tsx` under
  the panel's steps; guide `02_architecture.md` updated (flow §7A).
  **Judgment calls:** (1) a stage 1/6 output without `note` (pre-W1) is never reused, so the stage 6 unit (whose
  identity has no instructions version) is looked at again once and Look again cannot loop on a note-less copy;
  (2) not-determined photo looks store `note: []`; (3) `research_saw` = keys whose decision is not Accepted,
  Corrected, KeptPrevious or Dismissed (Typed and Empty included; C8(e) needs Empty); (4) a stage 6 observation with
  `photo_id: null` is shown without a photo and kept in the writer input; (5) the selected condition photos are the
  first media-list field's `selected` in the form (the damage report), else every Condition photo by (position, id);
  (6) staleness rebuilds the note unit's MC-A7 identity with the current photos under that run's own settings/form
  snapshot and compares fingerprints (the identity is not stored), photo limit from live settings; (7) Look again
  checks in WC1.8's order: Busy, NotStale, NoIntegration, NoModelKey, PriceMissing (the Unassigned session is an
  error, as for Run again); (8) a queued look run is blocked only by its own prerequisites and has no source wait
  (it starts like `manual`); (9) "Arranged" cancels a queued look/write run like Run again (one queued run per
  session); (10) a look run stopped by the cost rule ends `stopped_at_cap`; (11) `product.title`'s WC4.3 stored
  state is read in `note.rs` (no note key can be the title in W1; W2 moves it to `writes::stored_state`);
  (12) the note block re-reads with the panel's research status (one `research_changed` listener), is not
  collapsible.
  **Deviations (files outside the unit lists):** `decide.rs` (`end` → `pub(crate)`) and `writes.rs` (`cleared_set`
  → `pub(crate)`), visibility only; engine tests `adapter_openai.rs`, `photo_look.rs`, `stages.rs` follow the new
  schema (`note` required) and version 3; `FormDialog.tsx` passes `sessionId` to the panel (one prop; without it the
  note never shows in the app, rule 10); `docs/guide/02_architecture.md`; `commands/writer.rs` carries its own
  `unavailable()` rather than editing `commands/research.rs`; `state.rs` unchanged. The closing suite ran detached
  (the tool's 10-minute foreground limit), one run.
  **Test count:** 54 rows → 54 row tests (one each, named after the row), plus two unit tables on core
  mechanisms: `writer::note::tests::w1_c10_stale_rules` (`stale`, behind C10) and `stages::tests::w1_parse_note_rules`
  (the note parser, behind C3/C4).
- **2026-11-19 · review r1 (Claude Opus 5.5, `pipeline-reviewer`). Verdict APPROVED_WITH_NOTES.** Gate: tracker
  IMPLEMENTED, checkpoints `853cf91`, `c6279fb`, `06463ae` (+ closing `44680c6`) in `git log`; committed paths equal
  the handoff's declared files per unit (the out-of-list files `decide.rs`, `writes.rs`, three engine tests,
  `FormDialog.tsx`, the guide are declared deviations). Review-mode checks run once on the code tree of `06463ae`:
  `cargo test -p psm-research` 191 + 284 + 1 passed · `--test suite research::` 82 passed · `cargo test -p psm-desktop`
  51 passed (bindings regenerated byte-identical) · `vitest src/features/form` 86 passed · clippy (research,
  integrations, desktop, testkit; `-D warnings`) clean · fmt clean · typecheck clean. Implementer's full-suite stamp
  (1696/0) stands. **Verified correct by reading:** WC1.1–3 (note_fields from the form by step, §9.3 sentences
  verbatim, schema entries strict, parser: trim/drop/300 chars by char/keys ⊆ note_fields deduped/20 max/photo index
  over images actually sent/null out of range/missing → `[]`); WC1.4 reuse unchanged + note-less pre-W1 photo looks
  never copied; WC1.5 selection over `all_runs` by `looks_at_photos` with the `note` key; WC1.6 numbering over the
  form's damage-report selection; WC1.7 by rebuilding the unit identity under the note run's snapshot; WC1.8 order
  Busy → NotStale → NoIntegration → NoModelKey → PriceMissing, look run = stages 1 and 6 only and ends via
  `decide::end` (no path to `decide::decide`/`writes::suggest_in`); WC1.9: `store::runs`, both overview queries,
  `status`, `arranged`, research reuse exclude `look`/`write`, `tick`/`runs_in_state` do not; Run again/Arranged
  cancel a queued look/write run in the inserting transaction; WC2 decision order (first match) and `reviewed`
  exactly as written; `WriterApi` on the default wiring path; commands thin, `research_unavailable` without the
  engine. All 54 row tests observe their row's outcome at its boundary and would fail on a wrong outcome. H1–H8:
  no violation (no external write, no RAW/pixel change, no platform concept in `crates/core`, logic outside Tauri).
  Assumptions: none contradicts 08C. **Findings:** none blocking. **Notes** (8):
  `docs/implementation/backlog_writer/W1_product_note.md` (stale guide row in `04_change_impact.md`; banner that
  cannot clear when no Condition field is chosen; stage 6 `photos` list after a gateway drop; WC1.8 wording;
  `research_saw` rule to settle before W2; C7(e) parenthetical unobserved; duplicated title constant; loose row
  triggers). Probe files: none (flow §4).
