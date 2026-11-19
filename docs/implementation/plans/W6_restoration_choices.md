---
phase: W6_restoration_choices
title: Restoration as fixed choices — a "Restoration done" field with fixed choices (sanded, oiled, lacquered, new foam, webbing, papercord, fabric and more) at the end of the Condition step of every item type; kept in Session Manager only, never sent to Shopify; the writer states each chosen step exactly, and the check makes sure none is left out; the free-text Restoration field stays for anything extra
difficulty: medium
projection: no
depends_on: [W5_seo_publish]
blocked_on: none (owner card 1 answered A on 2026-11-19, see Review log)
---
# W6 — Restoration as fixed choices

**Tracker:** NOT_STARTED

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there; the owner card above is a second gate.)

## Goal
Book `08C` §13 ("Restoration becomes fixed choices … The free-text Restoration field stays") by WC8-R and WC5.11 of `master_plan_writer.md` §5, built as owner card 1's recommended default: an app-local built-in field. One unit. **Not here:** a Shopify metafield for the choices (the card's other branch would replace this plan).

## How to work this plan (flow.md §10, master_plan_writer.md §3)
One runner phase of 1 unit; after it, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit. Row ids `W6·C<n>(<row>)`; tests named after their row. Reviewer notes: `docs/implementation/backlog_writer/W6_restoration_choices.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The field, never published or researched, used and checked by the writer | 5 | 10 | **yes** |
| **Total** | | **5** | **10** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `master_plan_writer.md` §4, §5 WC2, WC5.11, **WC8-R** · book `08C` §7, §13 · the W5 plan, Unit 1 (the always-present built-ins) · `crates/core/src/services/{attributes.rs, publish.rs (write_set)}` · `crates/research/src/{rules/write_table.rs (never_written), writer/{input.rs, postcheck.rs}}`.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **"Restoration done" lives only in Session Manager:** it is saved with the item and in its export, never sent to Shopify, so the shop needs no new field.
- **The choices are fixed in this build** (cleaned, sanded, oiled, waxed, lacquered, joints re-glued, veneer repaired, new foam, new webbing, new springs, new papercord, new fabric, new leather, rewired); more can be added in a later build.
- **Research never fills it;** it is the workshop's knowledge, not something in the photos.

## Unit 1 — The field, never published or researched, used and checked by the writer

**Unit id:** `W6·U1` · **Rows:** W6·C1–C5 · **Changes app files:** yes

### Goal
`local.restoration` (WC8-R) as an always-present built-in of the Condition step; `never_written` names it; the writer sees it as a fact and the post-check enforces WC5.11.

### Read first
As above, plus `crates/testkit/tests/suite/{p09_local_export.rs, research/w5_seo_publish.rs}` and the W5 always-present list.

### Files
Edited: `crates/core/src/services/attributes.rs` (the definition, the always-present list), `crates/research/src/rules/write_table.rs`, `crates/research/src/writer/{postcheck.rs, defaults.rs (the phrases)}`. New: `crates/testkit/tests/suite/research/w6_restoration.rs` (its `mod` line). Tests enumerating built-ins follow.

### Core mechanisms (the only functions with unit tests)
`writer::postcheck::check` (the `Restoration` rule) · `rules::write_table::never_written` (the new key).

### Ordered tasks
1. `builtin_definitions()` gains `local.restoration` (MultiChoice, "Restoration done", group Condition, the 14 choices of WC8-R in that order); it joins the always-present list for the Condition step.
2. `never_written` returns true for `local.restoration`.
3. The writer: the field is visible by default (Condition step, WC2); `postcheck` gains rule 11 with the phrases of WC8-R per language.
4. Tests for the rows.

### Criteria (5 criteria, 10 rows)
Boundary: `AttributesService` and `PublishService` through `TestApp` with the fake commerce adapter; a research run and a `write` run on the wired app with the fakes; `check` and `never_written` (pure).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W6·C1(a) | `form_get` | an item type with a stored Condition preset that lacks the field | the Condition step ends with "Restoration done", multi-choice, the 14 choices in WC8-R order | WC8-R |
| W6·C1(b) | `set_value` | choose Sanded and Oiled | stored canonically as a set of the two choice strings, provenance human | WC8-R |
| W6·C2(a) | a publish of a linked item | `local.restoration` = {Sanded, Oiled}; the shop has no definition with that key | the fake adapter's attribute writes contain no `local.restoration`; the item reaches `done` | WC8-R, H1 |
| W6·C2(b) | local export (07 §Completion) | the same item | `item.json` holds `local.restoration` with the two choices | WC8-R, M7 |
| W6·C3(a) | `never_written` | the `local.restoration` definition in the Condition step | true | WC8-R |
| W6·C3(b) | a research run | `fill_keys` lists `local.restoration`; the fakes propose Oiled for it | no row written; the final step names it "not researched" | WC8-R, MA10 |
| W6·C4(a) | a `write` run | `local.restoration` = {New foam} (human) | the input's `fields` hold it with decision `Typed`, fact true, value "New foam" | WC2 |
| W6·C5(a) | `check` | English; restoration {Sanded, Oiled}; condition report "Very Good condition. Frame sanded and oiled. Stable."; description silent on it | no `Restoration` hit | WC5.11 |
| W6·C5(b) | same | the same restoration; the report mentions sanding only, the description says "sanded and oiled" | one `Restoration` hit naming "Oiled" (the description does not count) | WC5.11 |
| W6·C5(c) | same | Swedish; restoration {New fabric}; report "Mycket gott renoverat skick. Sits och rygg nyklädda. Stadig." | no hit | WC5.11 |

### Delegated choices
How the choices render in the form (chips or a list, design handoff).

### Notes
- Hazard: an always-present Condition built-in appears in every item type's form, so the form tests that count Condition fields change; update their expected lists, never their assertions' strength.

### Unit test commands
`cargo test -p psm-research --lib writer::postcheck rules::write_table` · `cargo test -p psm-testkit --test suite research::w6_restoration:: p09_local_export:: p10_attributes::` · `cargo clippy -p psm-core -p psm-research --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer runs, once: `cargo test -p psm-research --lib writer::postcheck rules::write_table` · `cargo test -p psm-testkit --test suite research::w6_restoration:: p09_local_export:: p10_attributes::` · clippy, fmt. The implementer's full-suite stamp stands. Only B1–B4 block. Notes go to `docs/implementation/backlog_writer/W6_restoration_choices.md`.

## Manual try-out (the narrative is built from this)
1. Open a session's Condition step: "Restoration done" is at the end. Tick Sanded and Oiled.
2. Press **Write texts**: the description and the condition report say the frame was sanded and oiled.
3. Publish: Shopify's product has no new field; the export's `item.json` has the choices.

## Review log

- **2026-11-19, orchestrator: owner card 1 answered A.** Owner: the restoration shown to customers lives in the condition report description. So `local.restoration` stays app-local (never sent to Shopify), the writer states each chosen step in the condition report, and rule 11 checks the condition report only (master plan §5 rule 11 amended; rows C5(a)/(b) amended).
