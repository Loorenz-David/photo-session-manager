---
phase: W4_writer_settings
title: Settings → Writer — choose the model and effort, the language and whether the writer starts by itself; edit the instruction with "Preview what is sent", earlier versions and "Reset to default"; pick the fields the writer may see; edit the style examples per language, extra banned words and target lengths; set each safeguard type to off, note or ask and see how often each was answered or dismissed; the number of questions and the shop name for search titles
difficulty: medium
projection: no
depends_on: [W3_writer_review]
blocked_on: none
---
# W4 — Settings → Writer

**Tracker:** IMPLEMENTED (2026-11-19, claude opus (`pipeline-implementer`), implement r1: 2 units, checkpoints 23a9668 · 82da541; 21 rows → 9 command tests + 2 `push_version` unit tests + 11 UI tests; closing gate green: `cargo test --workspace` 1829 passed, clippy, fmt, typecheck, `npm test` 659 passed)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `08C` §8 over the `writer` slot of `master_plan_writer.md` §4. (U1) The commands: read and save the slot, versions kept on save, reset an instruction, preview the exact request, safeguard counts. (U2) The Settings → Writer section. **Not here:** the SEO fields (W5), Restoration (W6), learning from edits (deferred, 08C §12).

## How to work this plan (flow.md §10, master_plan_writer.md §3)
One runner phase of 2 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `W4·C<n>(<row>)`; tests named after their row. Open details: the smallest option consistent with 08C §8, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_writer/W4_writer_settings.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Commands: get, save with versions, reset, preview, counts | 5 | 10 | **yes** |
| U2 | The Settings → Writer section | 5 | 11 | **yes** (TypeScript) |
| **Total** | | **10** | **21** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `master_plan_writer.md` §4 (Settings, API, Commands), §5 WC2 (visible keys), WC6 (modes, counts), WC7 (snapshot), §9 · book `08C` §7, §8 · the W2 plan, Unit 1 (`WriterSettings`) and Unit 3 (the request) · `docs/guide/02_architecture.md` §Frontend · `docs/guide/04_change_impact.md` row "settings field" · `apps/desktop/src/features/settings/{SettingsNav.tsx, ResearchSection.tsx, ShopFieldPicker.tsx}`.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Settings → Writer saves on Save, like Settings → Research;** the writer reads the saved values at its next run.
- **Every save that changes an instruction keeps the text it replaced;** the last 20 per language are listed with their date, and choosing one puts it back in the editor.
- **"Reset to default" returns to the shipped instruction and keeps your text among the earlier versions;** future builds' improvements to the shipped text reach you while you have not edited it.
- **"Preview what is sent" shows the exact request for a built-in example item** (example A's fields), so it costs nothing and needs no item.
- **The models offered are those with a price in Settings → Research.**
- **The shop name is typed once here;** it is added to a search title only when the title still fits in 60 characters.

## Unit 1 — Commands: get, save with versions, reset, preview, counts

**Unit id:** `W4·U1` · **Rows:** W4·C1–C5 · **Changes app files:** yes

### Goal
`writer_settings_get`, `writer_settings_save(value)`, `writer_instruction_reset(language)`, `writer_preview(language)`; `WriterApi::{settings_validate, instruction_default, preview}`; the counts in the get.

### Read first
Master plan §4 · `crates/research/src/writer/{config.rs, defaults.rs, run.rs (the request builder), safeguard.rs (counts)}` · `apps/desktop/src-tauri/src/commands/{writer.rs, research.rs (save_research), settings.rs}` · `crates/testkit/tests/suite/research/w3_writer_wiring.rs`.

### Files
Edited: `crates/research/src/writer/{api.rs, config.rs, run.rs}`, `apps/desktop/src-tauri/src/{commands/writer.rs, lib.rs}`, `apps/desktop/src/services/{bindings.ts (generated), writer.ts}`, `crates/testkit/tests/suite/research/w3_writer_wiring.rs` (or a new `w4_writer_settings.rs` with its `mod` line).

### Core mechanisms (the only functions with unit tests)
`WriterSettings::push_version` (the versions rule: newest first, at most 20, only when the text changes).

### Ordered tasks
1. `writer_settings_get -> WriterSettingsDto` = the slot read by `from_slot` with every field present, each language's `effective_instruction` (saved or shipped) and `is_default`, and `counts: {<type>: {answered, dismissed}}`.
2. `writer_settings_save(value)`: validate; for each language whose instruction text differs from the stored one, push the stored effective text to `versions` (master plan §4: newest first, at most 20); save through `settings_set({writer: …})`; return the saved DTO.
3. `writer_instruction_reset(language)`: push the current saved text (if any) to versions; set `text` to none; save; return.
4. `writer_preview(language) -> {instructions, input, schema}` built by the same code as a run's request (W2 U4 task 1) for a built-in example item whose fields are example A's fields line split into labelled fields, with the current saved settings; no provider call, no run row.
5. Tests for the rows.

### Criteria (5 criteria, 10 rows)
Boundary: the wired app through the command layer (outcome = the answer, the saved file, run and call rows, the fake model's requests); `push_version` (pure).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W4·C1(a) | `writer_settings_get` | no `writer` slot | every field of master plan §4 at its default; English `effective_instruction` = `INSTRUCTION_EN`, `is_default` true; counts all zero | 08C §8 |
| W4·C1(b) | same | two answered and one dismissed `field_vs_field` findings stored | counts `field_vs_field` {answered 2, dismissed 1}, the other types zero | 08C §6 (counts) |
| W4·C2(a) | `writer_settings_save` | English text "Guide 2" over a stored "Guide 1" | saved text "Guide 2"; English versions begin with "Guide 1" and its time | 08C §8 (versions) |
| W4·C2(b) | `push_version` | 20 stored versions; a new text | 20 versions, the new previous text first, the oldest gone | 08C §8 |
| W4·C2(c) | `writer_settings_save` | the same text saved again; then an invalid value (`language` "German") | no new version; then refused naming `language`, the file unchanged | 08C §8 |
| W4·C3(a) | `writer_instruction_reset` | English saved text "Guide 2" | `text` none, `effective_instruction` = `INSTRUCTION_EN`, `is_default` true; versions begin with "Guide 2" | 08C §8 (reset) |
| W4·C4(a) | `writer_preview` | language English; saved text "Guide 2"; `shop_name` "Beyo Vintage" | `instructions` begin with "Guide 2" and contain "Example A"; `input` has `texts`, `fields` (example A's values), `shop_name` "Beyo Vintage"; `schema` has `texts`, `findings`, `questions` | 08C §8 (preview) |
| W4·C4(b) | same | — | no run row was created, no call row, and the fake model received no request | 08C §8, HW6 |
| W4·C5(a) | `writer_settings_save`, then a write run | `visible_keys` = [`custom.designer`] only; `max_questions` 1 | the run's input `fields` hold only the designer; at most one ask is stored from an answer with three | WC2, WC6.6 (rule 10) |
| W4·C5(b) | same | `model` "gpt-6.1-terra", `effort` "low" | the run's call row has model `gpt-6.1-terra`, effort `low` | WC7 |

### Delegated choices
DTO field names beyond those listed · how the example item's fields are labelled.

### Notes
- Hazard: `settings_set` must receive the whole `writer` value; a partial save would drop fields (the research DTO avoided this by a full slot). The save sends the full slot.

### Unit test commands
`cargo test -p psm-research --lib writer::config` · `cargo test -p psm-testkit --test suite research::w3_writer_wiring::` (and `research::w4_writer_settings::` if new) · `cargo test -p psm-desktop` · `cargo clippy -p psm-research -p psm-desktop --all-targets -- -D warnings`

## Unit 2 — The Settings → Writer section

**Unit id:** `W4·U2` · **Rows:** W4·C6–C10 · **Changes app files:** yes (TypeScript)

### Goal
`WriterSection.tsx` (with `WriterInstruction.tsx`, `WriterExamples.tsx`) and its nav entry, over the U1 commands.

### Read first
`apps/desktop/src/features/settings/{SettingsNav.tsx, SettingsPage.tsx, ResearchSection.tsx, ShopFieldPicker.tsx}`, `__tests__/{r6_research_settings.test.tsx, r13_research_fills.test.tsx, t1_settings_drawer.test.tsx}` · `apps/desktop/src/services/{writer.ts, attributes.ts (listDefinitions)}` · `docs/intention/evidence/design_handoff.md`.

### Files
New: `apps/desktop/src/features/settings/{WriterSection.tsx, WriterInstruction.tsx, WriterExamples.tsx}`, `apps/desktop/src/features/settings/__tests__/w4_writer_settings.test.tsx`. Edited: `apps/desktop/src/features/settings/{SettingsNav.tsx, SettingsPage.tsx}`.

### Core mechanisms
None.

### Ordered tasks
1. Nav entry "Writer" after "Research". Sections in order: Model and effort (models = the distinct models of `research.prices` for the research model provider); Language (English, Swedish); Automatic writing on "Next to review" (switch); Instruction (editor for the chosen language, **Preview what is sent** in a dialog showing instructions, input and schema, an "Earlier versions" list with date and **Use this**, **Reset to default**); Fields the writer may see (a checklist over the Details and Condition definitions, the default rule's set checked when no list is saved, prices unchecked); Style examples per language (list with add, edit, remove); Extra banned words; Target lengths; Safeguard (per type a select off / note / ask with its answered and dismissed counts; Questions per write); Shop name for search titles.
2. **Save** sends the whole slot through `writerSettingsSave`; errors name the field.
3. Tests for the rows.

### Criteria (5 criteria, 11 rows)
Boundary: `WriterSection` rendered in a UI test with `services/writer.ts`, `services/settings.ts` and `services/attributes.ts` mocked (outcome = what is shown and the service calls with their arguments).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W4·C6(a) | `SettingsNav`, `WriterSection` | default settings; prices for `gpt-6.1-sol`, `gpt-6.1-terra`, `gpt-6-luna` | a "Writer" entry; the model select offers exactly those three with `gpt-6.1-sol` chosen; effort `medium`; language English; automatic writing on | `[rule]` 08C §8 |
| W4·C6(b) | same | choose Swedish, turn automatic writing off, Save | `writerSettingsSave` receives the full slot with `language` "Swedish" and `auto` false | `[rule]` 08C §8 |
| W4·C7(a) | `WriterInstruction` | edit the English text, Save | the save carries the edited English text | `[rule]` 08C §8 |
| W4·C7(b) | same | click **Preview what is sent** | `writerPreview('English')` called once; the dialog shows its instructions, input and schema | `[rule]` 08C §8 |
| W4·C7(c) | same | versions [{"Guide 1", date}]; click **Use this** | the editor holds "Guide 1" (not saved until Save) | `[rule]` 08C §8 |
| W4·C7(d) | same | click **Reset to default** | `writerInstructionReset('English')` called once; the editor shows the returned default text | `[rule]` 08C §8 |
| W4·C8(a) | visible fields | definitions: Details designer, Details price (money), Condition scratch; no saved list | designer and scratch checked, price unchecked | `[rule]` WC2 (visible keys) |
| W4·C8(b) | same | uncheck scratch, Save | the save carries `visible_keys` = [designer] | `[rule]` WC2 |
| W4·C9(a) | `WriterExamples` | Swedish examples [D]; add one, fill its title, Save | the save carries Swedish examples [D, the new one] | `[rule]` 08C §8 |
| W4·C10(a) | safeguard | counts `field_vs_photos` {answered 3, dismissed 5} | the type's row shows its mode "ask" and "answered 3 · dismissed 5" | `[rule]` 08C §6 |
| W4·C10(b) | same | set `field_vs_field` to off, questions per write 2, shop name "Beyo Vintage", Save | the save carries `safeguard.field_vs_field` "off", `max_questions` 2, `shop_name` "Beyo Vintage" | `[rule]` 08C §8 |

### Delegated choices
Layout and grouping inside the section (design handoff) · how a long instruction is edited (textarea height, a word count).

### Notes
- Hazard (rule 10): the model select must show the saved model even when the price table no longer lists it (marked "no price"), so a save does not silently replace it.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/settings` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-research --lib writer::` · `cargo test -p psm-testkit --test suite research::w3_writer_wiring::` (and `w4_…` if present) · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/settings` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: the preview and a run build their request through one function; a save never sends a partial `writer` slot. Notes go to `docs/implementation/backlog_writer/W4_writer_settings.md`.

## Manual try-out (the narrative is built from this)
1. Open Settings → Writer. Change the instruction (for example ask for shorter descriptions) and press **Preview what is sent**: the exact request shows. Save.
2. In a session press **Write texts**: the new texts follow your instruction. Back in Settings, "Earlier versions" lists the shipped text; **Reset to default** brings it back.
3. Switch the language to Swedish and write texts again: they arrive in Swedish.
4. Set "Field vs field" to off; uncheck a field you do not want used; save and write again.

## Review log

### 2026-11-19 · implement r1 (claude opus, `pipeline-implementer`)
**Built.** U1 (`23a9668`): `WriterSettings::push_version` (newest first, at most 20, only when the text changes),
`saved_over` (the stored versions stay authoritative; a changed instruction keeps the text it replaced, with the save
time; a text or example list equal to the shipped one is stored as the shipped default) and `reset_instruction`;
`WriterSnapshot::resolve` (used by a run's start and by the preview); `writer::input::assemble` (the input object, used
by a run's input and the preview's); `writer::run::request_of` (instructions, input, schema: a run's call and the
preview are built by it) and `preview_input` (example A's fields line as twelve labelled `typed` facts, every text
requested); `WriterApi::{settings_validate, instruction_default, preview, counts}`; the commands
`writer_settings_get`, `writer_settings_save(value)`, `writer_instruction_reset(language)`, `writer_preview(language)`
over a typed `WriterSettingsDto` (every field, both languages' instructions resolved, examples resolved, counts per
type); bindings and `services/writer.ts`. U2 (`82da541`): `WriterSection` (loaded when first opened), `WriterInstruction`
(editor with word count, Preview dialog, Earlier versions with Use this, Reset to default merged into the unsaved
form), `WriterExamples` (both languages, add / edit / remove), the "Writer" nav entry after "Research" (shown with it);
the guide (01, 02).

**Assumptions.** Preview uses the saved settings (task 4), so an unsaved edit shows after Save · the preview requests
all five texts (no item, so no layout) · the example item's keys are `example.<name>` · examples are shown for both
languages at once, the instruction for the chosen language · effort is a text field (as in Settings → Research) ·
extra banned words are one per line · a saved model without a price is offered as "<model> (no price)" · Reset saves
at once and replaces only that language's instruction in the form, other unsaved edits stay.

**Deviations.** Files outside the U1 list, each needed by the plan's own tasks: `crates/research/src/writer/input.rs`
(the shared `assemble`), `crates/research/src/runs.rs` (one call to `WriterSnapshot::resolve`),
`crates/research/src/api_impl.rs` (the trait methods). `WriterApi::counts` added beside the three named methods (the
get needs the counts through the handle). U1 tests are desktop command tests in `apps/desktop/src-tauri/src/lib.rs`
(`w4_tests`, the command layer itself), not a testkit stem. U2 also edited `__tests__/t1_settings_drawer.test.tsx` (its
section list gains "Writer") and `docs/guide/{01_product,02_architecture}.md`.

