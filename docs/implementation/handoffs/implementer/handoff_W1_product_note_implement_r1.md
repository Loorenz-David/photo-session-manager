---
plan: W1_product_note
role: implement
round: 1
state: DONE
date: 2026-11-19
actor: claude-opus-5.5 (pipeline-implementer)
---
# W1_product_note — implement r1

Three units built in order, each with its checkpoint; closing gate stamp below.

⚠ OWNER DECISIONS REQUIRED (0) — none.

## What was built
- **The product note.** Research's two photo looks (stage 1 on the Main photos, stage 6 on the Condition photos)
  now also write a short factual note, each observation tagged with the Details/Condition fields it bears on (stage
  6 also with its photo). It is stored inside each look's unit output and reused with it. The instructions changed,
  so every stage is looked at again once (`INSTRUCTIONS_VERSION` 3).
- **Your decisions win.** For each tagged field, the owner's standing (accepted, corrected, kept previous,
  dismissed, typed; or unchecked, shop default, imported, empty) is derived from the stored row and the latest
  review feedback. An observation on a decided field is shown as "Wood: Teak (owner's decision)" (or "Wood: not
  stated (the owner dismissed Oak)"); the original words are kept aside (`research_saw`) only for fields the owner
  has not decided. Condition observations are numbered "condition photo n" among the photos the damage report
  keeps; one whose photo was taken out is marked "photo no longer in the condition photos" and left out of what the
  writer will get.
- **Staleness and Look again.** The note knows when the photos it looked at differ from the photos research would
  look at now (another photo among the first ones, a recompose, other condition photos; a photo beyond the limit is
  not a change), or when Main photos were never looked at. **Look again** queues a `look` run that runs stages 1 and
  6 only (unchanged units copied, not paid again), writes no field, and ends finished. It is refused, with a
  sentence, while anything else runs on the session, when nothing changed, or when the shop / model key / prices
  are missing.
- **Research views unchanged.** Look (and W2's future write) runs are invisible to the research panel's status,
  the sessions overview, the "Arranged" check and a research run's reuse; Run again (and Arranged) cancels a queued
  look run with "research started again".
- **App.** `AppHandles::writer` (the same engine), commands `writer_note` and `writer_look_again` (answer
  `research_unavailable` without the engine), bindings regenerated, and the **"What research saw"** block under the
  research panel's steps: one line per observation with its photo, a "Your decision" mark, the banner "Photos
  changed since they were looked at" / "No product note yet" with **Look again**, a refusal's sentence.

## Tests → criterion rows
| Row | Test | Shape as the row specifies |
|---|---|---|
| W1·C1(a) | `psm-research --lib adapters::openai::schema::tests::w1_c1a_observe_with_photos_answers_a_note_of_texts_and_keys` | yes: top-level keys and required, item properties, types, `additionalProperties: false` |
| W1·C1(b) | `…schema::tests::w1_c1b_condition_answers_a_note_naming_its_photo` | yes |
| W1·C2(a) | engine `w1_note::w1_c2a_stage_1_asks_for_a_note_over_the_details_and_condition_fields` | yes: exact `note_fields` (keys, labels, order), `fields` = wood only, instructions end with the §9.3 sentence (from the call record) |
| W1·C2(b) | `w1_note::w1_c2b_a_stage_1_unit_of_instructions_version_2_is_not_reused` | yes: one stage 1 request, not copied, fingerprint = identity with version 3 |
| W1·C3(a) | `w1_note::w1_c3a_note_keys_are_kept_only_when_asked_for_once_each` | yes, exact stored note |
| W1·C3(b) | `w1_note::w1_c3b_blank_observations_are_dropped_and_long_ones_cut_at_300_characters` | yes; the prefix is two-byte letters so a byte cut would fail |
| W1·C3(c) | `w1_note::w1_c3c_at_most_20_observations_in_reply_order` | yes, exact 20 in order |
| W1·C3(d) | `w1_note::w1_c3d_a_reply_without_note_stores_an_empty_note` | yes: `[]` and run finished |
| W1·C4(a) | `w1_note::w1_c4a_stage_6_numbers_its_photos_and_asks_over_the_condition_fields` | yes: `photos` exact, images in that order, `note_fields` exact |
| W1·C4(b) | `w1_note::w1_c4b_a_note_photo_index_counts_only_the_photos_sent` | yes: photo deleted after the first attempt, the retry drops it (gateway), notes 1/2 → first/third photo |
| W1·C4(c) | `w1_note::w1_c4c_a_note_photo_out_of_range_has_no_photo` | yes |
| W1·C5(a) | `w1_note::w1_c5a_an_unchanged_stage_1_unit_is_copied_with_its_note` | yes: no stage 1 call, `reused_from_run_id` = run 1, same note |
| W1·C6(a)–(k) | `psm-research --lib writer::note::tests::w1_c6a_…` … `w1_c6k_…` (11 tests) | yes, one exact `Decision` each; (f) also with the rows given in reverse order |
| W1·C7(a)–(d) | `writer::note::tests::w1_c7a_…` … `w1_c7d_…` | yes |
| W1·C7(e) | engine `w1_note::w1_c7e_keep_previous_is_reviewed_until_it_is_undone` | yes: Keep previous through `AttributesService`, undo through `HistoryService` |
| W1·C8(a)–(f) | `w1_note::w1_c8a_…` … `w1_c8f_…` (6 tests, through `WriterApi::note`) | yes: exact texts, `owner`, exact `research_saw`; (f) exact photo labels and `in_writer_input` |
| W1·C9(a)–(b) | `w1_note::w1_c9a_…`, `w1_c9b_…` | yes (the browse run also holds a note, so it is excluded by rule, not by absence) |
| W1·C10(a)–(e) | `w1_note::w1_c10a_…` … `w1_c10e_…` | yes |
| W1·C11(a) | `w1_note::w1_c11a_look_again_looks_at_the_changed_photos_only` | yes: queued, one look run, model requests = [Observe], no search/reader request, stage 6 copied from run 1, finished, attribute rows equal, `stale_main` false after |
| W1·C11(b)–(d) | `w1_note::w1_c11b_…`, `w1_c11c_…`, `w1_c11d_…` | yes, exact reason codes; (b) no new run |
| W1·C12(a)–(c) | `w1_note::w1_c12a_…`, `w1_c12b_…`, `w1_c12c_…` | yes |
| W1·C13(a)–(b) | `psm-testkit --test suite research::w1_note_seam::w1_c13a_…`, `w1_c13b_…` | yes, through `AppHandles::writer` (the handle the commands call) on the wired app |
| W1·C13(c) | `psm-desktop` `w1_writer_command_tests::w1_c13c_writer_commands_without_the_engine_answer_research_unavailable` | yes: both commands through Tauri IPC answer `research_unavailable`; bindings contain both |
| W1·C14(a)–(c) | `apps/desktop/src/features/form/__tests__/w1_product_note.test.tsx` (3 tests) | yes |

Reverse direction: every test in the new files maps to a row above, plus two unit tables on core mechanisms the
plan names: `writer::note::tests::w1_c10_stale_rules` (`stale`) and `stages::tests::w1_parse_note_rules` (the note
parser). Existing tests edited only to follow the new shape (listed under Deviations). Red baseline not captured
(the tests were written beside the code, unit by unit).

## Assumptions
1. A stage 1 or 6 output committed before W1 (no `note` key) is never reused: stage 6's identity has no
   instructions version, so without this Look again would copy the note-less unit forever. Every stage 6 is looked
   at again once (the plan's "every stage is looked at again once").
2. A photo look that could not read the photos stores `note: []`.
3. `research_saw` holds the original words for each key whose decision is not Accepted, Corrected, Kept previous or
   Dismissed (so Typed and Empty count; row C8(e) needs Empty).
4. A condition observation with no photo (`photo_id: null`) is shown without a photo label and kept for the writer.
5. "Selected condition photos" = the first media-list field's selection in the form (the damage report), else every
   Condition photo by (position, id).
6. Staleness rebuilds the note unit's identity with today's photos under that run's own settings and form snapshot
   and compares fingerprints (identities are not stored); the photo limit is the live setting.
7. Look again checks, in order: Busy, NotStale, NoIntegration, NoModelKey, PriceMissing; the Unassigned session is
   an error (as Run again).
8. A queued look run is blocked only by its own prerequisites (shop, model key, observe/condition prices), not by
   research's visual-search key or chosen fields, and it starts like a manual run (no source wait).
9. "Arranged" with a queued look run cancels it like Run again (one queued run per session is allowed).
10. A look run ended by the cost stop rule ends `stopped_at_cap`.
11. `product.title`'s stored state (WC4.3) is read in `note.rs` (W2 will own it as `writes::stored_state`).
12. Refusal sentences: Busy "Research is still working on this session"; NotStale "The photos have not changed
    since they were looked at"; the others reuse research's sentences.
13. The note block re-reads with the panel's research status (which the form re-reads on `research_changed`), so
    there is one listener; it sits under the steps and is not collapsible.
14. Actor: the prompt template says `codex`; this session is Claude Opus 5.5 (`PSM_IMPLEMENTER=claude`), so the
    tracker names Claude.

## Deviations
- `crates/research/src/decide.rs` (`end` → `pub(crate)`) and `crates/research/src/writes.rs` (`cleared_set` →
  `pub(crate)`): visibility only, outside U2's list.
- Existing engine tests adjusted to the new shape: `adapter_openai.rs` (a15_c4b Condition schema keys; a15_c6a
  replies carry `note: []` because the strict schema now requires it), `photo_look.rs` (a26_c6b version 3; a26_c7a
  schema keys and reply), `stages.rs` (identity literal version 3).
- `apps/desktop/src/features/form/FormDialog.tsx`: one prop (`sessionId`) to the panel; without it the note block
  never appears in the app (rule 10).
- `docs/guide/02_architecture.md`: the new trigger, module and commands (flow §7A).
- `commands/writer.rs` has its own `unavailable()` instead of making `commands/research.rs`'s public; `state.rs`
  unchanged (the handle comes through `AppHandles`).
- The full suite ran detached (the agent tool's 10-minute foreground limit; first run after a relink), once.

## Files changed (write perimeter)
U1 `853cf91`: `crates/research/src/{stages.rs, config.rs, adapters/openai/schema.rs}`,
`crates/research/tests/engine/{main.rs, w1_note.rs, adapter_openai.rs, photo_look.rs, stages.rs}`.
U2 `c6279fb`: `crates/research/src/{api_impl.rs, decide.rs, domain.rs, lib.rs, rules/run_rules.rs, runs.rs,
stages.rs, store.rs, writes.rs, writer/mod.rs, writer/api.rs, writer/note.rs}`,
`crates/research/tests/engine/w1_note.rs`.
U3 `06463ae`: `crates/integrations/src/lib.rs`, `apps/desktop/src-tauri/src/{commands/mod.rs, commands/writer.rs,
lib.rs}`, `apps/desktop/src/services/{bindings.ts (generated), writer.ts}`,
`apps/desktop/src/features/form/{ResearchPanel.tsx, ProductNote.tsx, FormDialog.tsx,
__tests__/w1_product_note.test.tsx}`, `crates/testkit/tests/suite/research/{mod.rs, w1_note_seam.rs}`,
`docs/guide/02_architecture.md`.
Closing: `docs/implementation/plans/W1_product_note.md` (Tracker line, Review log), this handoff.
Mutation-probe files: none (no mutation regime, flow §4). Tool-recorded state: none (no archgraph). Untracked
`.pipeline/runs/w1_closing_suite.log` (not committed).

## Checkpoint SHA
U1 `853cf91` · U2 `c6279fb` · U3 `06463ae` · closing (tracker + Review log) `44680c6`.

## Stamp
Tree: the code tree of `06463ae` (the closing commit `44680c6` changes only the plan file); `git status --porcelain`
clean apart from untracked `.pipeline/` and this handoff (committed by the runner).
- `cargo test --workspace` (once, detached; log `.pipeline/runs/w1_closing_suite.log`): **exit 0, 1696 passed,
  0 failed**. No pre-existing failure appeared (the known intermittent tests passed).
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `cd apps/desktop && npm run typecheck`: clean.
- `npm test`: 101 files, 636 tests passed.
Per-unit commands run as the plan lists them: U1 `w1_note::`, `--lib adapters::openai::schema`, `cargo test -p
psm-research` (263 + 175 + 1), clippy, `cargo check --workspace`; U2 `--lib writer::note` (16), `w1_note::` (31),
the whole research crate in place of the three stems (284 + 191 + 1; U2 changed shared run code), clippy, check;
U3 `research::w1_note_seam::` (2), `cargo test -p psm-desktop` (51; the C13(c) bindings check failed once on the
run that regenerated `bindings.ts`, as expected for an `include_str!` check, and passed on the re-run of that one
test), `npx vitest run src/features/form` (86), typecheck.

## Manual try-out
On scratch data (`cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`), research keys set
and a few fields chosen (including a Condition field for condition observations):
1. Shoot or import a session with Main and Condition photos, press "Arranged — next", let research finish.
2. Open Details → research panel: under the steps, "What research saw" lists short observations; condition ones
   carry "condition photo n".
3. In AI review, accept or correct one value (for example the wood). The matching observation now reads
   "Wood: Teak (owner's decision)" with a "Your decision" mark.
4. Recompose a Main photo: the banner "Photos changed since they were looked at" appears. Press **Look again**:
   only the photo looks run (no listing search, no new suggestions; the cost grows by a few cents at most), and the
   banner goes when it finishes. Pressing it again while it runs shows "Research is still working on this session".
