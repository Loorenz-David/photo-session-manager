---
phase: W2_writer_engine
title: The writer, behind fakes — writer settings with the shipped writing guide; what the writer receives; a deterministic check of every text (banned words and patterns, lengths, damage named, grade word, stability, names and years only from fields) with one rewrite; a write run that queues once per session, never beside research, and writes its texts as suggestions under the same rules as research; questions and notes that never ask again about what you decided; cost, day limit and export shared with research
difficulty: high
projection: no
depends_on: [W1_product_note]
blocked_on: none
---
# W2 — The writer engine

**Tracker:** IMPLEMENTED (2026-11-19, codex (Claude Opus implementer) r1: 5 units, 96 rows tested at their boundaries; closing gate green; checkpoints e7481f3, 66ac61b, 7a2724a, c796d2b, 5c6067c)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `08C` §5–§7, §10 by contracts WC2–WC7 of `master_plan_writer.md` §5, entirely inside `crates/research` and proven with its fakes. (U1) `WriterSettings`, the shipped guide, the visible fields and the input. (U2) The post-check. (U3) The run's life: request, prerequisites, start order, `Stage::Write`, schema, snapshot. (U4) The call, the rewrite, the writes. (U5) The safeguard, findings, cost, record and export. **Not here:** the `writer` settings slot and the app wiring, commands, triggers, UI (W3); Settings → Writer (W4); the SEO fields (W5: until then the writer writes title, description and condition report, because the SEO keys are not defined); Restoration (W6).

## How to work this plan (flow.md §10, master_plan_writer.md §3)
One runner phase of 5 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `W2·C<n>(<row>)`; tests named after their row. Open details: the smallest option consistent with 08C and WC2–WC7, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_writer/W2_writer_engine.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Settings, the shipped guide, visible fields, the input | 4 | 12 | one line (`ResearchDeps` in the wiring) |
| U2 | The post-check | 7 | 36 | no |
| U3 | The run's life | 4 | 16 | no |
| U4 | The call, the rewrite, the writes | 5 | 17 | no |
| U5 | Safeguard, findings, cost, record, export | 5 | 15 | **yes** (session delete, schema tripwire list) |
| **Total** | | **25** | **96** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §10) · `master_plan_writer.md` (all; **§5 WC2–WC7 are the contract**, §9 the shipped guide) · `master_plan_research.md` §6.3–§6.7, §7.2 · book `08C` (all), evidence `docs/intention/evidence/publication_writer_research.md` §7, §8 · `08B` MC-A1, MC-A6, MC-A7, MC-A8, MC-A9, MC-A11, MC-A12 · the W1 plan (what `writer::note` provides) · `docs/guide/02_architecture.md` §Research.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **The writer never sees a photo;** it reads the fields you let it see, research's note with your decisions, and the titles of matching listings.
- **A clean text arrives as a normal suggestion, a text that failed the check twice arrives marked low with what failed,** so Accept all leaves it for you.
- **The writer rewrites once at most;** a run costs one call, two when the first texts fail the check.
- **Automatic writing happens once per session;** afterwards only "Write texts" or answering a question writes again.
- **Nothing is written while research or Look again is still busy with the session,** and Run again cancels a writer run that is only waiting.
- **"Write texts" goes ahead of waiting research runs,** because it is one short call.
- **A question or note is never raised about a field you decided in review, or asked again once answered or dismissed.**
- **At most three questions per write** (asks beyond that become notes).

## Unit 1 — Settings, the shipped guide, visible fields, the input

**Unit id:** `W2·U1` · **Rows:** W2·C1–C4 · **Changes app files:** one line (`crates/integrations/src/lib.rs`, the `ResearchDeps` literal)

### Goal
`writer::config::WriterSettings` (defaults, `from_slot`, `validate`, resolution of the instruction and examples), `writer::defaults` (master plan §9 verbatim), the visible-keys rule and `writer::input::build` (WC2 "Input").

### Read first
Master plan §4 Settings, §5 WC2, §9 · `crates/research/src/{config.rs (ResearchSettings::from_slot, validate), service.rs (ResearchDeps, SettingsSource), stages.rs (descriptor, display), rules/value_map.rs}` · `crates/core/src/services/attributes.rs` (`layout_keys`, `builtin_definitions`) · W1's `writer::note`.

### Files
New: `crates/research/src/writer/{config.rs, defaults.rs, input.rs}`, `crates/research/tests/engine/w2_writer_run.rs`. Edited: `crates/research/src/{writer/mod.rs, service.rs}` (`ResearchDeps.writer_settings`), `crates/research/tests/engine/{harness.rs, main.rs}` (the harness supplies `writer_settings`), every construction of `ResearchDeps` in the crate and in `crates/integrations/src/lib.rs` (passing `Arc::new(|| None)` until W3; one line).

### Core mechanisms (the only functions with unit tests)
`WriterSettings::{from_slot, validate}` · `writer::input::visible_keys`.

### Ordered tasks
1. `WriterSettings` with the fields and defaults of master plan §4; `from_slot` tolerant as `ResearchSettings::from_slot`; `validate` rejects: an unknown language, a mode other than off/note/ask, `max_questions` outside 0–10, any length min above its max or a length of 0, a blank model.
2. `writer::defaults`: `INSTRUCTION_EN`, `INSTRUCTION_SV`, the title formulas and dimension lines, `EXAMPLES_EN` (A, B, C), `EXAMPLES_SV` (D), as master plan §9 states. `resolved_instruction(language)` and `resolved_examples(language)` return the saved value or the shipped one.
3. `visible_keys` (WC2 "Visible keys") and `input::build` (WC2 "Input") over the item's stored state and W1's note view; `requestable_texts` (WC4.1).
4. Tests for the rows.

### Criteria (4 criteria, 12 rows)
Boundary: the named function (pure); `input::build` on the engine harness (store with rows and feedback written through the store, a session with a committed research run).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W2·C1(a) | `WriterSettings::default()` and its JSON | — | equals master plan §4's list field by field (`auto` true, `language` "English", `model` "gpt-6.1-sol", `effort` "medium", the seven lengths, the five modes, `max_questions` 3, `shop_name` "", `condition_report_key` "custom.condition_report_description", `instructions`/`examples` with no saved text) | 08C §8 |
| W2·C1(b) | `resolved_instruction`, `resolved_examples` | default settings, language English, then Swedish; then a saved English text "X" | `INSTRUCTION_EN` and 3 examples named A, B, C; `INSTRUCTION_SV` and 1 example named D; then "X" | 08C §7 (rule 10) |
| W2·C2(a) | `WriterSettings::from_slot` | no slot; then `{"max_questions": "three", "shop_name": "Beyo"}` | the default; then the default with `shop_name` "Beyo" (the wrongly shaped field ignored) | 08C §8 |
| W2·C2(b) | `validate` | five values: language "German"; `safeguard.field_vs_field` "loud"; `max_questions` 11; `description_min_words` 130 with max 120; model "" | each refused with an error naming that field | 08C §8 |
| W2·C3(a) | `visible_keys` | layout: Details text `custom.designer`, Details money `custom.price`, Condition choice `custom.condition_scratch`, Condition media list `custom.damage_report`, Product `product.description`; no saved list | `custom.designer`, `custom.condition_scratch` | WC2 (visible keys) |
| W2·C3(b) | same | the same layout; saved `visible_keys` = [`custom.price`, `custom.gone`, `custom.designer`] | `custom.designer`, `custom.price` (layout order; the unknown key dropped) | WC2 |
| W2·C4(a) | `input::build` | visible: designer `ai` + `accepted` "Børge Mogensen", era `ai` + `suggested` low "1950s", wood `default` "Oak" | `fields` = designer {decision Accepted, fact true}, era {Unchecked, fact false}, wood {ShopDefault, fact true} | WC2, HW2 |
| W2·C4(b) | same | the latest research run read pages with stage 4 verdicts: p1 `similar`, p2 `different`, p3 `match`, and 9 more `similar` | `listings` = p3 first, then p1, then the next `similar` pages in stage 3 order, 8 in total; p2 absent | WC2 |
| W2·C4(c) | same | the item's `writer_findings`: one `answered` (older), one `dismissed`, one `answered` (newer) | `answers` = the two answered ones, older first, each with type, keys, question and answer | WC2, 08C §5.6 |
| W2·C4(d) | same | `product.description` row `ai` + `cleared` "Old text"; feedback `kept_previous` with suggested value "Other text" | `rejected` has description with exactly "Old text" and "Other text" | WC2, MC-A1 |
| W2·C4(e) | same | layout without `custom.condition_report_description` | `texts` = [`title`, `description`] | WC4.1 |
| W2·C4(f) | same | `shop_name` "" then "Beyo Vintage" | no `shop_name` key; then `shop_name` "Beyo Vintage" | WC2 |

### Delegated choices
The display text of a value (reuse `stages::display`) · the order of keys inside `fields` beyond layout order.

### Notes
- The research harness gets a `writer_settings` source like its `settings` source; nothing in the app reads the `writer` slot yet (W3).

### Unit test commands
`cargo test -p psm-research --lib writer::` · `cargo test -p psm-research --test engine w2_writer_run::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo check --workspace`

## Unit 2 — The post-check

**Unit id:** `W2·U2` · **Rows:** W2·C5–C11 · **Changes app files:** no

### Goal
`writer::postcheck::check` exactly as WC5 items 1–10 (item 11 is W6).

### Read first
Master plan §5 WC5, §9.4 · `crates/core/src/mechanisms/{name_key.rs, values.rs (is_severity_set)}` · evidence §7.4, §7.5.

### Files
New: `crates/research/src/writer/postcheck.rs`. Edited: `crates/research/src/writer/mod.rs`.

### Core mechanisms (the only functions with unit tests)
`writer::postcheck::check` (all rows of this unit are its unit tests).

### Ordered tasks
1. `check(texts, ctx)`; `ctx` carries the language, lengths, `banned_extra`, the `fact` values, the note texts the writer received, the visible Condition severity fields with values and labels, `damage_terms`, the detailed-condition key and value, the attribution names (choices of `attribution_keys` definitions and their stored values).
2. Unit tests for the rows. Each row's fixture is otherwise clean, so its one predicate is the only reason for its hit.

### Criteria (7 criteria, 36 rows)
Boundary: `check` (pure). "Clean" means a text that passes every check. Characters and words as WC5.1.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W2·C5(a) | `check` | title of 70 characters with "ø"; then 71 | none; then one `Length` on title | WC5.1 |
| W2·C5(b) | same | description of 59, 60, 120, 121 words | `Length`; none; none; `Length` | WC5.1 |
| W2·C5(c) | same | condition report of 45, then 46 words | none; then `Length` | WC5.1 |
| W2·C5(d) | same | search title of 60, then 61 characters | none; then `Length` | WC5.1 |
| W2·C5(e) | same | search description of 119, 120, 155, 156 characters | `Length`; none; none; `Length` | WC5.1 |
| W2·C6(a) | same | a description containing "Stunning" | one `Banned` naming "stunning" | WC5.2 |
| W2·C6(b) | same | "perfect for"; then "perfekt för"; then "showcases" | one `Banned` each | WC5.2 |
| W2·C6(c) | same | "iconic" in the title; then the same with a fact value "Iconic design" | `Banned`; then none | WC5.2 |
| W2·C6(d) | same | `banned_extra` ["nice"]; a text with "Nice" | `Banned` naming "nice" | WC5.2 |
| W2·C6(e) | same | "teak — oak"; "1950–60"; "teak – oak"; "teak - oak"; "re-oiled" | `Dash`; none; `Dash`; `Dash`; none | WC5.3 |
| W2·C6(f) | same | "A fine chair!"; "Ready?" | `Punctuation` each | WC5.4 |
| W2·C6(g) | same | ", highlighting the grain"; "serves as a desk"; "not just a chair but a sculpture"; "It is not a copy, it is the original" | `Pattern` each | WC5.5 |
| W2·C7(a) | same | an English description with one sentence containing " works at " | none | WC5.6 |
| W2·C7(b) | same | two sentences with " suits " and " use them " | one `UseSentences` | WC5.6 |
| W2·C7(c) | same | a Swedish description with " passar " in two sentences | one `UseSentences` | WC5.6 |
| W2·C8(a) | same | field "Condition.Scratches" = Minor (fact); report "Good. Light scratches on the top. Stable." | none | WC5.7 |
| W2·C8(b) | same | "Condition.Veneer Damage" = Moderate; report with "a 1 cm veneer chip" | none | WC5.7 |
| W2·C8(c) | same | "Condition.Stains/Discoloration" = Minor; report naming no stain | one `Damage` naming "Condition.Stains/Discoloration" | WC5.7 |
| W2·C8(d) | same | "Condition.Scratches" = None; then = N/A; report naming nothing | none; none | WC5.7 |
| W2·C8(e) | same | Swedish; "Condition.Scratches" = Minor; report "Gott skick. Lätta repor på sitsen. Stadig." | none | WC5.7 |
| W2·C8(f) | same | `damage_terms` {`custom.condition_odor`: {English: ["smell"]}}; "Condition.Odor" = Minor; report with "a faint smell" | none | WC5.7 |
| W2·C8(g) | same | "Condition.Cracks" = Major but not a fact (unchecked, low); report naming no crack | none | WC5.7, WC2 |
| W2·C9(a) | same | detailed condition fact "Very Good - This vintage item has no defects…"; report "Very good, restored. … Stable." | none | WC5.8 |
| W2·C9(b) | same | the same fact; report "Good. … Stable." | one `Grade` | WC5.8 |
| W2·C9(c) | same | Swedish; the same fact; report "Mycket gott renoverat skick. … Stadig." | none | WC5.8 |
| W2·C9(d) | same | no detailed-condition fact; report "Light wear. Stable." | none | WC5.8 |
| W2·C9(e) | same | a report with none of the stability words; then one ending "All three stand firm." | `Stability`; then none | WC5.9 |
| W2·C10(a) | same | attribution choices include "Hans J. Wegner" and "Børge Mogensen"; fact designer "Børge Mogensen"; title naming Wegner's full name | one `Names` naming "Hans J. Wegner" | WC5.10, HW2 |
| W2·C10(b) | same | the same; title naming "Børge Mogensen" | none | WC5.10 |
| W2·C10(c) | same | fact era "1960s"; description "made in 1958" | one `Years` naming 1958 | WC5.10 |
| W2·C10(d) | same | fact era "1960s"; title "…, 1960s" | none | WC5.10 |
| W2·C10(e) | same | fact model "J39"; title "J39 Chair"; then a title "GE 290 Chair" | none; then one `Models` naming "GE 290" | WC5.10 |
| W2·C10(f) | same | a description whose last line is "Dimensions: W 49 × D 46 × H 78 cm, seat height 45 cm." and no model fact | none | WC5.10 |
| W2·C10(g) | same | a note text received "Stamp reads FD 164"; title "France & Søn FD 164 Chair" with maker "France & Søn" a fact | none | WC5.10, HW2 |
| W2·C11(a) | same | each shipped example (A, B, C in English, D in Swedish) as texts, its fields line as the only fact value, no Condition field, default lengths | no hit for any of the four | 08C §7 (rule 10) |
| W2·C11(b) | same | example A's condition report with its first word replaced by "Fair" and a detailed-condition fact "Very Good - …" | one `Grade` | WC5.8 |

### Delegated choices
Internal structure of `check` (one pass per text, or one per check) · the wording of each hit's detail (it must name the word, field label, year or token).

### Notes
- Rule 2 companion: every row's text is otherwise clean (right length, a stability word in reports, no banned word), so its predicate is the only cause.

### Unit test commands
`cargo test -p psm-research --lib writer::postcheck` · `cargo clippy -p psm-research --all-targets -- -D warnings`

## Unit 3 — The run's life

**Unit id:** `W2·U3` · **Rows:** W2·C12–C15 · **Changes app files:** no

### Goal
WC3 (request, refusals, once-per-session, repeat presses, start order, `may_start`) and the snapshot of WC7; `RunTrigger::Write`, `Stage::Write`, the writer schema.

### Read first
Master plan §5 WC3, WC4.1, WC7 · `crates/research/src/{runs.rs (insert_queued, tick, start, may_start_run, missing_price, key_set, first_failing), rules/run_rules.rs, rules/cost.rs (next_call), domain.rs (Stage, RunTrigger), adapters/openai/schema.rs, job.rs, stages.rs (execute)}`.

### Files
New: `crates/research/src/writer/{run.rs, schema.rs}`. Edited: `crates/research/src/{domain.rs, runs.rs, rules/run_rules.rs, rules/cost.rs, adapters/openai/schema.rs, job.rs (dispatch by trigger), writer/api.rs}`, `crates/research/tests/engine/w2_writer_run.rs`.

### Core mechanisms (the only functions with unit tests)
`rules::run_rules::start_order` (the tier) · `writer::run::request_decision` (the pure part of WC3: inputs → outcome).

### Ordered tasks
1. `RunTrigger::Write` (`"write"`); `Stage::Write` (8, `"write"`, not in `ALL`; `parse` resolves it); `next_call` treats `Write` like `Decide`.
2. `writer::run::request(session, origin)` (WC3 1–6) with the writer prerequisites and their sentences (`WriterBlock`); `WriterApi::request`.
3. Start order tier and `may_start` for `write` (WC3 "Start"). The job handler executes a `write` run through `writer::run::execute` (U4 fills it; here it commits nothing and ends `finished` so the life can be tested).
4. The start transaction for a `write` run: settings snapshot with `writer` (resolved settings, the instruction and examples actually sent) and `stages.write`; `input_snapshot` = `input::build`.
5. `writer::schema::schema_for_write(texts)`: `{texts: {<name>: {paragraphs | text, facts: [string], reason: string}}, findings: [{type: enum of the five, keys: [string], text}], questions: [{keys: [string], text}]}` — `description` answers `paragraphs: [string]`, the others `text: string`; all required, `additionalProperties: false`. `schema_for` returns it for `Stage::Write`.
6. Tests for the rows.

### Criteria (4 criteria, 16 rows)
Boundary: `WriterApi::request` and `ResearchService::tick` on the engine harness (outcome = run rows and the answer); `schema_for` and `start_order` (pure).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W2·C12(a) | `request` (`write_texts`) | the session has a `queued` `manual` research run | `queued: false`, `ResearchBusy`; no `write` run | WC3.2 |
| W2·C12(b) | same | the session has a `running` `look` run | `queued: false`, `ResearchBusy` | WC3.2 |
| W2·C12(c) | same | no integration and no model key; then the integration only; then both and no price row for `gpt-6.1-sol` | `NoIntegration`; `NoModelKey`; `PriceMissing` naming the provider and `gpt-6.1-sol`; never a run | WC3.3, HW7 |
| W2·C12(h) | same | the Unassigned session, with a `queued` research run of another session present | `queued: false`, `NothingToWrite`; no run | WC3.1 |
| W2·C12(d) | same | `writer.auto` false; origin `next_to_review`; then origin `write_texts` | no run and no reason; then one `queued` `write` run | WC3.4 |
| W2·C12(e) | same (`all_checked`) | the session has a `finished` `write` run; then (another session) only a `failed` one | no new run; then one `queued` `write` run | WC3.4, A19 |
| W2·C12(f) | same (`write_texts`, twice) | a `queued` `write` run exists | `queued: true` both times; still exactly one `write` run | WC3.5 |
| W2·C12(g) | same (`write_texts`, twice) | a `running` `write` run exists | after the first: one `queued` `write` run beside the running one; after the second: still one queued | WC3.5, WC3.6 |
| W2·C13(a) | `start_order` | queued (trigger, queued_at, id): (arranged, 1, 1), (write, 5, 2), (manual, 7, 3), (look, 6, 4) | ids 2, 4, 3, 1 | WC3 (start) |
| W2·C13(b) | `tick` | a `queued` `write` run; the session's photos changed 0 ms ago; no run running | the run starts (`running`, a `jobs` row of kind `research`) | WC3 (start) |
| W2·C13(c) | `tick` | a `queued` `write` run; today's spend at the day cap | the run stays `queued`, no `jobs` row | WC3, MA5 |
| W2·C14(a) | `Stage` | `Stage::parse("write")`, `number`, `Stage::ALL`; `ResearchSettings::default().stages` | `Some(Write)`, 8; `ALL` has seven stages without `Write`; the settings' stages map has the seven research stage keys only | WC7 |
| W2·C14(b) | `schema_for` | a `Write` request with texts [`title`, `description`] | top-level `texts`, `findings`, `questions`; `texts` has exactly `title` (`text`, `facts`, `reason`) and `description` (`paragraphs`, `facts`, `reason`); `findings.items.type` enumerates the five finding types | WC4.1, WC6 |
| W2·C14(c) | `next_call` | `Write` with run cost 499 999 and cap 500 000 and reserve 50 000; then cost 500 000 | `Call`; then `EndAtCap` | WC7 (stop rule) |
| W2·C15(a) | the start transaction of a `write` run | saved writer settings with English instruction text "My guide", model "gpt-6.1-terra", effort "low" | the run's settings snapshot has `writer.instruction` "My guide", the English examples A, B, C, and `stages.write` = {gpt-6.1-terra, low}; `input_snapshot` equals `input::build` for the session at that instant | WC7, WC2 |
| W2·C15(b) | same | the saved instruction is changed after the run started | the snapshot still holds "My guide" | WC7 (snapshot) |

### Delegated choices
The pure/impure split of `request` · the representation of the origin in the snapshot.

### Notes
- Hazard: the per-session partial unique indexes allow one `running` and one `queued` run per session across all triggers; WC3's order is what keeps a `write` run from ever colliding with a research or look run (C12(a), (b), (g)).
- `title` is always requestable (WC4.1), so for a session with an item `NothingToWrite` cannot fail; it is reached through the Unassigned session (C12(h)). The prerequisite stays in the order for a future layout without a title.

### Unit test commands
`cargo test -p psm-research --lib rules::run_rules rules::cost writer::` · `cargo test -p psm-research --test engine w2_writer_run:: lifecycle::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo check --workspace`

## Unit 4 — The call, the rewrite, the writes

**Unit id:** `W2·U4` · **Rows:** W2·C16–C20 · **Changes app files:** no

### Goal
`writer::run::execute`: units `texts` (the call), `retry` (the rewrite, WC5 "Regenerate once") and `result` (writes, steps and findings in one transaction, WC4.6); `writer::writes` (WC4.2–5).

### Read first
Master plan §5 WC4, WC5 (regenerate), WC7 · `crates/research/src/{writes.rs (suggest_in, WriteOutcome), rules/write_table.rs, gateway.rs (call), stages.rs (Ctx::unit, commit, ask), decide.rs (committed, finish), fakes.rs}`.

### Files
New: `crates/research/src/writer/writes.rs`, `crates/research/tests/engine/w2_writer_writes.rs`. Edited: `crates/research/src/{writer/run.rs, writes.rs (a shared helper only if needed)}`, `crates/research/tests/engine/main.rs`.

### Core mechanisms (the only functions with unit tests)
`writer::writes::stored_state` (WC4.3).

### Ordered tasks
1. The request: instructions = resolved instruction + "\n\n" + title formula + dimensions line + the examples (each "Example <name>\nFields: …\nTitle: …" and so on); input = the run's `input_snapshot`; no images; model and effort from `stages.write`.
2. Post-check (U2) on the answer; with hits a `retry` call asking for the texts with hits only, naming each hit, plus the previous texts; the stop rule applies (WC7).
3. `writer::writes`: requestable keys only; `stored_state`; `decide_write`; the row of WC4.5; the same-content check.
4. The `result` unit: writes, one step per text (`concluded` with the reason and facts, or `not_determined` with the refusal reason), the post-check hits per attempt, findings (U5 fills the filter), committed together. Resume continues at the first uncommitted unit.
5. Tests for the rows.

### Criteria (5 criteria, 17 rows)
Boundary: a `write` run on the engine harness with `FakeModel` replying for stage `write` (outcome = requests the fake received, attribute rows, steps, unit outputs, call rows).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W2·C16(a) | a `write` run | default settings, a layout with the three text keys, an answer with three clean texts | the fake received exactly one `write` request; its instructions begin with `INSTRUCTION_EN`, contain the title formula and "Example A"; its input equals the run's `input_snapshot`; it carries no image | WC7, HW4 |
| W2·C16(b) | same | the same | title, description and condition report rows are `ai` + `suggested` at `medium`; each `ai_reason` ends with "Facts: " and the labels of the answer's `facts`; `ai_sources` empty; the description value is the paragraphs joined by a blank line | WC4.5 |
| W2·C17(a) | a `write` run | first answer: description with "stunning", title and report clean; retry answer: a clean description | two requests; the second names the hit "stunning" and asks for `description` only; the description row is the retry's at `medium`; the title and report rows are the first answer's | WC5 (regenerate) |
| W2·C17(b) | same | both answers' descriptions contain "stunning" | the description row is the retry's text at `low`, its reason contains "Check:" and "stunning"; the run's `result` output lists the hits of both attempts | WC5, WC4.5 |
| W2·C17(c) | same | `max_cost_per_run` set so the first call's cost reaches it; first answer's description has a hit | one request only; the description is written at `low` with a `NotRechecked` hit "not checked again: run cost limit" in its reason | WC7 (stop rule), WC5 |
| W2·C18(a) | a `write` run | no `product.description` row | written | WC4.4 (none) |
| W2·C18(b) | same | `product.description` `integration` "Shop text" | written; its `ai_prior` = {"Shop text", valid, integration} | WC4.4 |
| W2·C18(c) | same | `product.description` `human` "My text" | row unchanged; the text's step is `not_determined` with "the photographer wrote this value" | WC4.4, HW1 |
| W2·C18(d) | same | `product.description` `ai` + `accepted` | row unchanged; step names "the photographer accepted this value" | WC4.4, HW1 |
| W2·C18(e) | same | `product.description` `ai` + `suggested` (an earlier writer text) with an `ai_prior` | replaced by the new text; `ai_prior` unchanged | WC4.4 |
| W2·C18(f) | same | `product.description` `ai` + `cleared` with value "Old"; the answer's description "old" (same `N`); then, in a second run, "New text" | first: unchanged, step "matches a text you dismissed"; second: written | WC4.4, HW1, MA10 |
| W2·C18(g) | same | `items.title` blank, no `product.title` row | a `product.title` row `ai` + `suggested`; `items.title` still blank | WC4.3, WC4.5 |
| W2·C18(h) | same | `items.title` "Chair" typed (no link) | no `product.title` row; step "the photographer wrote this value"; `items.title` "Chair" | WC4.3, HW1 |
| W2·C18(i) | same | `items.title` "Shop chair" equal to the item link's title base | a `product.title` row whose `ai_prior` = {"Shop chair", valid, integration}; `items.title` unchanged | WC4.3 |
| W2·C19(a) | a `write` run | the layout lacks the condition report key; the answer (a fake ignoring the schema) also carries `condition_report` and `seo_title` | no row for either key; the result lists them as not requested | WC4.2 |
| W2·C20(a) | a `write` run killed after the `texts` unit committed and before `result` | resume (job runs again) | no new request; the same rows as an uninterrupted run with the same answer; each text has one step | WC4.6, H4 (MC-A7) |
| W2·C20(b) | the `result` writes applied twice (resume after commit, as stage 7's repeat) | — | no row changes on the second application (`updated_at` unchanged) | WC4.5 (same content), H4 |

### Delegated choices
How the examples are rendered into the instructions (the exact separators) · the step wording beyond the strings named.

### Notes
- Hazard: `writes::suggest_in` refuses every Product field through `never_written`; the writer's write path must not route through research's `never_written` (C18(a) would stay unwritten) and must not widen it either (research still refuses them).
- Hazard: the title rows never touch `items.title`; C18(g)–(i) assert it.

### Unit test commands
`cargo test -p psm-research --lib writer::writes` · `cargo test -p psm-research --test engine w2_writer_writes:: resume::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo check --workspace`

## Unit 5 — Safeguard, findings, cost, record, export

**Unit id:** `W2·U5` · **Rows:** W2·C21–C25 · **Changes app files:** yes (`crates/infra/src/sqlite/session_delete.rs`, `crates/testkit/tests/suite/p01_storage_identity.rs`)

### Goal
WC6 (filter, cap, storage, answer, dismiss, counts), the findings that come from the post-check (WC5), WC7's record and export; the `writer_findings` table; session delete removes it.

### Read first
Master plan §4 Storage, §5 WC6, WC7 · `crates/research/src/{schema.rs, store.rs, export.rs, gateway.rs}` · `crates/infra/src/sqlite/session_delete.rs` · `crates/testkit/tests/suite/{p01_storage_identity.rs (the table list), a17_delete_session.rs}`.

### Files
New: `crates/research/src/writer/safeguard.rs`, `crates/research/tests/engine/w2_safeguard.rs`. Edited: `crates/research/src/{schema.rs, store.rs, export.rs, writer/run.rs, writer/api.rs}`, `crates/research/tests/engine/main.rs`, `crates/infra/src/sqlite/session_delete.rs`, `crates/testkit/tests/suite/{p01_storage_identity.rs, a17_delete_session.rs}`.

### Core mechanisms (the only functions with unit tests)
`writer::safeguard::filter`.

### Ordered tasks
1. `writer_findings` in `schema::ensure` (master plan §4).
2. `filter` (WC6 1–6) with the decisions of W1 (`reviewed`, undecided); post-check hits to findings (WC5); storage and superseding in the `result` transaction; `WriterApi::{answer, dismiss}` (answer → `request(session, Answer)`); a counts query per type.
3. `export`: `writes.json` for `write` runs (WC7); `look` runs null.
4. Session delete removes the session's `writer_findings`; the p01 table list gains the table.
5. Tests for the rows.

### Criteria (5 criteria, 15 rows)
Boundary: `filter` (pure); a `write` run on the engine harness (outcome = `writer_findings` rows, run rows, call and outbound rows, the export folder); the wired app for the delete row.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W2·C21(a) | `filter` | a `field_vs_field` finding with mode off | dropped | WC6.1 |
| W2·C21(b) | same | `field_vs_photos` on `custom.wood` `Accepted`; then on `custom.wood` `Typed` "Teak" with `research_saw` "oak" | dropped; then kept (mode ask) | WC6.2, WC6.3, HW3 |
| W2·C21(c) | same | `missing_in_text` with text key `custom.condition_report_description` whose row is `ai` + `accepted`; then the row `ai` + `suggested` | dropped; then kept | WC6.2, HW3 |
| W2·C21(d) | same | a `field_vs_listings` finding whose identity equals an `answered` row's; then one equal to a `dismissed` row's | dropped; dropped | WC6.5, HW3 |
| W2·C21(e) | same | `max_questions` 3; four ask findings and two questions | three findings stored as ask, the fourth as note, no question stored; the step lists the two dropped questions | WC6.6 |
| W2·C21(f) | same | `field_vs_field` naming only `custom.gone` (not in the layout) | dropped | WC6.4 |
| W2·C22(a) | a `write` run | the answer carries two findings that pass; the item has one `open` row from an earlier run | two new `open` rows (kind, type, keys sorted, identity, mode); the earlier row is `superseded` | WC6 (storage) |
| W2·C22(b) | `WriterApi::answer`, then one tick | an open ask; answer "It is teak" | the row is `answered` with the answer and `resolved_at`; a `write` run is queued; the next run's input `answers` contains it | WC6 (answer), 08C §5.6 |
| W2·C22(c) | `WriterApi::dismiss`; the counts query | an open note | `dismissed`, no run queued; the counts give `field_vs_field` dismissed 1 | WC6 (dismiss, counts) |
| W2·C23(a) | a `write` run | both answers omit a Minor "Condition.Stains/Discoloration" from the report; `missing_in_text` mode ask | one `open` row, type `missing_in_text`, keys [`custom.condition_stains_discoloration`], text field the report key | WC5 → WC6 |
| W2·C23(b) | same | both answers name a year absent from the facts; `text_vs_facts` mode off | no row; the text is flagged (`low`) | WC5 → WC6.1 |
| W2·C24(a) | a `write` run with usage reported | two calls (a rewrite) | two `research_calls` rows, stage `write`, model `gpt-6.1-sol`, each cost the MC-A8 integer over its usage; the run's cost is their sum; today's spend (as `day_limit_reached` reads it) includes them | WC7, HW6, MA15 |
| W2·C24(b) | same | a session with Main and Condition photos | no `research_outbound` row has the run's id | WC7, HW4, MA12 |
| W2·C24(c) | `ResearchApi::export` | a date range holding a research run, a `look` run and a `write` run | the export has a folder per run; the `write` run's `run.json` has trigger "write" and its `writes.json` holds `texts`, `hits`, `writes`, `findings`, `questions`; the `look` run's `writes.json` is null | WC7, HW6, MA9 |
| W2·C25(a) | the wired app, `delete_session` (A17) | a session with a `write` run and two `writer_findings` rows; p01's schema list | after the delete no `writer_findings` row names the session; p01's table check passes with the new table | 08C §10, A17 |

### Delegated choices
The counts query's return shape · whether superseding is an UPDATE of state or a column.

### Notes
- Hazard: the identity of a question includes `N(text)` (WC6.5); without it, the first answered question with no keys would block every later one.
- This unit changes two app files; the Closing gate below is the full gate for that reason.

### Unit test commands
`cargo test -p psm-research --lib writer::safeguard` · `cargo test -p psm-research --test engine w2_safeguard:: export::` · `cargo test -p psm-testkit --test suite p01_storage_identity:: a17_delete_session::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo check --workspace`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check`. No TypeScript changed: `npm` checks are not needed. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-research` · `cargo test -p psm-testkit --test suite p01_storage_identity:: a17_delete_session::` · clippy, fmt. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: the writer's write path cannot reach a key outside the requestable texts; no writer request builder can attach an image; `filter` runs before any `writer_findings` insert. Notes go to `docs/implementation/backlog_writer/W2_writer_engine.md`.

## Manual try-out (the narrative is built from this)
Nothing in the app runs the writer yet. The narrative says so and lists what is proven: the shipped writing guide and examples, the check of every text, the once-per-session rule, the never-ask-again rule, and that the writer's cost and calls sit in research's record and export. The owner can read the shipped guide in `master_plan_writer.md` §9.

## Review log

### 2026-11-19 — implement r1 (Claude Opus, `pipeline-implementer`)
**Built.** U1 `writer::config` (`WriterSettings`, `from_slot`, `validate`, `WriterSnapshot`), `writer::defaults` (§9 verbatim: both instructions, title formulas, dimensions lines, examples A–D with §9.4's changes), `writer::input` (`visible_keys`, `requestable_texts`, `build`), `ResearchDeps.writer_settings` (the app passes `Arc::new(|| None)`). U2 `writer::postcheck::check` (WC5 1–10). U3 `RunTrigger::Write`, `Stage::Write` (8, not in `ALL`, `parse` resolves it), `next_call` treats it like Decide, start tier 0 and no source wait for `write`, `writer::run::{request_decision, writer_request}` (WC3), the start transaction's writer snapshot and input, `writer::schema::schema_for_write`, `WriterApi::request`. U4 `writer::run::execute` (units `texts`, `retry`, `result`; one rewrite; flagged texts `low`; `NotRechecked` at the run cost limit), `writer::writes` (`stored_state`, `write_text` over MC-A1's unchanged table; never `items.title`). U5 `writer::safeguard::filter` (WC6 1–6), storage and superseding in the `result` transaction, `WriterApi::{answer, dismiss}`, `writer_counts`, `writes.json` for writer runs, session delete removes `writer_findings`, p01 lists the table, guide `02_architecture.md` updated.
**Assumptions** (handoff lists all): findings and questions come from the first answer only; a run whose rewrite the cost limit stopped ends `stopped_at_cap`; the origin waits in the queued run's settings snapshot until the start moves it into `writer.origin`; a model `text_vs_facts` finding's text field is the first requested text key among its keys.
**Deviations.** C18(f): the row's input "old" against a dismissed "Old" cannot match under WC4.4 / 08B MC-A1 (the built-in description is `rich_text`, whose sameness is canonical equality, not `N`); the contract is implemented as written and the test uses a canonically equal "Old". The `writer_findings` table is created from U1 (C4(c) reads it) in `store::TABLES`, which `schema::ensure` runs. The writer run is dispatched in `stages::execute`, not `job.rs`.
**Stamp.** `cargo test --workspace` (exit 0, 0 failed) · `cargo clippy --workspace --all-targets -- -D warnings` (exit 0) · `cargo fmt --all -- --check` (exit 0).

