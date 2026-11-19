---
plan: W3_writer_review
role: implement
round: 1
state: DONE
date: 2026-11-19
actor: claude opus (pipeline-implementer)
---
# Handoff — W3_writer_review, implement r1

Gate: intention RATIFIED; `W2_writer_engine` Tracker APPROVED; W3 Tracker was NOT_STARTED. Passed.

## What was built
- **Texts arrive in AI review.** Pressing **Next to review** (in the AI review tab or the session header), or closing
  Details / leaving while AI review shows **All checked ✓**, asks the writer for the session's title, description and
  condition report in the background: once per session, never with a toast, never holding up the move to the next
  session. **Write texts** in the Product step asks again whenever the owner likes and shows the refusal sentence as a
  toast ("Research is still working on this session; write texts when it finishes", "Add the model key in
  Settings → Research", …).
- **Questions and notes above the suggestions.** A writer block at the top of AI review: "Writing texts…" while the
  writer works; each open question with an answer box and **Answer** (the texts are written again with the answer);
  each open note with **Dismiss**. The tab's empty reason names the writer when it is the cause.
- **Text rows.** A description or condition report shows the previous text and the suggestion as paragraphs; a text
  the post-check still flags shows its low badge and its "Check: …" line; the title row shows "Shop chair → new
  title" with Accept, Keep previous and Edit.
- **The title.** A title suggestion is an attribute row: the form shows it, but `items.title` (and so the shop)
  changes only when the owner accepts it; undo brings the old title back; Keep previous deletes the suggestion (the
  writer never writes it again); typing a title removes it.
- **Publish.** The partition knows the built-in fields, so a writer description over the shop's description is a
  real conflict (the item waits); `product.title` rows are never frozen, withheld or pending.
- **Research panel.** A Writer block: what the latest write used (fields, observations, listing titles), each text's
  reason and facts, the post-check per attempt, "Rewritten once", findings and questions, the cost ("$0.02").
- **Wiring.** `settings.json` gains the `writer` slot (validated, merged per field, never rebuilds the App), read live
  by the engine; `WriterApi::status`; commands `writer_status`, `writer_request`, `writer_finding_answer`,
  `writer_finding_dismiss` (all answer `research_unavailable` without the engine); bindings regenerated;
  `services/writer.ts`. Guide 01–05 updated (flow §7A).

## Tests → criterion rows
| Row | Test | Shape |
|---|---|---|
| W3·C1(a) | `research::w3_title::w3_c1a_the_form_shows_the_title_suggestion_over_the_shops_title` | exact (provenance, status, prior) |
| W3·C1(b) | `…::w3_c1b_typing_over_the_suggestion_removes_it_with_one_edited_feedback` | exact |
| W3·C2(a) | `…::w3_c2a_accept_sets_the_items_title_and_undo_redo_move_both_together` | exact (accept, undo, redo) |
| W3·C2(b) | `…::w3_c2b_keep_previous_deletes_the_suggestion_and_the_writer_never_writes_it_again` | exact; real writer run proves the cleared set (its description is written, the title is not) |
| W3·C2(c) | `…::w3_c2c_dismissing_a_title_suggestion_on_a_blank_title_clears_it` | exact |
| W3·C2(d) | `…::w3_c2d_accept_all_takes_the_title_and_skips_a_low_description` | exact |
| W3·C3(a) | `…::w3_c3a_a_writer_description_over_the_shops_is_a_conflict_that_holds_the_item` | exact (conflict key, held, no mutation) |
| W3·C3(b) | `…::w3_c3b_an_unaccepted_title_suggestion_neither_holds_nor_reaches_the_shop` | exact (no title in partition, Done, 0 `update_title`, remote title unchanged) |
| W3·C3(c) | `…::w3_c3c_an_accepted_title_suggestion_is_sent_once` | exact (1 `update_title`, remote = suggestion) |
| Core mechanism | `psm-core mechanisms::publish_partition::tests::w3_partition_skips_every_product_title_row` | unit, input → output |
| W3·C4(a) | `research::w3_writer_wiring::w3_c4a_saving_the_writer_slot_never_rebuilds_and_auto_off_queues_nothing` | exact (`PreparedSettings::Simple`, file, `queued:false, reason:None`, no run) |
| W3·C4(b) | `…::w3_c4b_an_invalid_writer_slot_is_refused_and_nothing_is_saved` | exact (error names `max_questions`, file bytes equal) |
| W3·C5(a) | `…::w3_c5a_write_texts_queues_a_run_whose_texts_reach_the_form` | exact (queued, finished, title and description suggested in the form, `ResearchChanged` for the session) |
| W3·C5(b) | `…::w3_c5b_write_texts_while_research_runs_is_refused_with_its_sentence` | exact sentence |
| W3·C5(c) | `…::w3_c5c_status_shows_the_run_its_flagged_text_the_ask_the_cost_and_the_inputs` | exact (run, flagged text + hit, ask type/keys/mode, cost = run cost, inputs 2 observations, the match listing title, fields = snapshot) |
| W3·C5(d) | `…::w3_c5d_an_answer_writes_the_texts_again_with_the_answer_in_the_input` | exact (`answered`, second run finished, `answers` = the answer) |
| W3·C5(e) | `…::w3_c5e_dismissing_a_note_queues_nothing` | exact |
| W3·C6(a) | `psm-desktop w1_writer_command_tests::w3_c6a_writer_commands_without_the_engine_answer_research_unavailable` | exact (4 commands, 8 binding names) |
| W3·C6(b) | `research::w3_writer_wiring::w3_c6b_without_the_model_key_the_writer_says_why_and_publish_still_works` | exact sentence; the session publishes to Done |
| W3·C7(a) | `features/sessions/__tests__/w3_writer_triggers.test.tsx` `w3_c7a_…` | exact (once, args, before `onNext`; still moves on when the request fails) |
| W3·C7(b) | `w3_c7b_closing_details_while_all_checked_asks_once_and_not_with_a_suggestion_left` | exact (once `all_checked`; not called with one unchecked) |
| W3·C7(c) | `w3_c7c_write_texts_in_the_product_step_asks_once_and_shows_a_refusal_as_a_toast` | exact |
| W3·C7(d) | `w3_c7d_the_headers_next_to_review_asks_the_writer_before_the_queue_moves_on` | exact (order) |
| W3·C8(a) | `features/form/__tests__/w3_writer_review.test.tsx` `w3_c8a_…` | exact (question before the suggestions list; `writerFindingAnswer(11, 'Teak')` once) |
| W3·C8(b) | `w3_c8b_a_dismissed_note_leaves_after_the_re_read` | exact |
| W3·C8(c) | `w3_c8c_while_the_writer_writes_the_tab_says_so` | exact (+ the empty reason names the writer) |
| W3·C9(a) | `w3_c9a_…` | exact (previous text, both paragraphs, `acceptSuggestion(7, 'product.description')` once) |
| W3·C9(b) | `w3_c9b_…` | exact (low badge, Check line) |
| W3·C9(c) | `w3_c9c_…` | exact (prior before suggestion; Accept, Keep previous, Edit) |
| W3·C10(a) | `features/form/__tests__/w3_writer_panel.test.tsx` `w3_c10a_…` | exact (counts, titles, reasons, facts, attempt 1 hits, "Rewritten once", note, "$0.02") |
| W3·C10(b) | `w3_c10b_without_a_write_run_there_is_no_writer_block` | exact |

Every test in the five new test files maps to a row above (no orphan). No red baseline was captured before the
production edits (stated plainly; flow §4 has no mutation regime).

## Assumptions
1. Clear on the title: a shown suggestion → only its row becomes `cleared` (`items.title` kept); otherwise the
   item's title is emptied and a live accepted row is cleared with it, in one undo step.
2. Accept all still skips a title suggestion that has a prior (an MC-A1 conflict like any other); C2(d) has none.
3. A title edit that removes a suggestion is one undo step; further typing merges into it and updates the `edited`
   feedback's final value.
4. Undo/redo of a step that holds a `product.title` row and the title is refused ("that title was changed") when the
   item's title is no longer the one the step left; a plain title edit restores as before.
5. `status` reads the latest `write` run (delegated); findings are the item's `open` ones; `reason` / `can_write` say
   what a "Write texts" press would meet now.
6. `writer_request` refuses an unknown origin and `answer` (internal) with `invalid_value`.
7. "Leaving the session page" = the Details dialog unmounting while AI review shows "All checked ✓"; "All checked" =
   no suggestion left, some AI value on the item, research not queued or running.
8. On the writer's texts (Product step, long texts) Accept is the plain accept (`attribute_ai_accept`), the same
   backend `accept` as Use suggestion; research's conflicts keep Use suggestion (the A11 test is unchanged). Without
   this, C9(a) could not hold: a description with a prior is a backend conflict.
9. Money: no research-panel money helper existed; `WriterPanel.money` shows USD with two decimals.
10. Write texts sits at the top of the Product step with one explaining line; a queued answer shows "Writing texts.
    They arrive in AI review.".

## Deviations
- `crates/core/src/services/settings.rs` (outside the U2 list): `writer` added to the patch allowlist, merged per
  field like `research`; without it `settings_set({"writer": …})` is refused before validation (C4).
- `crates/research/src/writer/run.rs` (outside the lists): `WriterBlock::ResearchBusy`'s sentence extended to the
  plan's "…; write texts when it finishes" (C5(b), C7(c)).
- `crates/research/src/api_impl.rs` (outside the lists): the one-line `status` delegation of the trait impl.
- `apps/desktop/src/App.tsx` listed but unchanged: the calls live in the components the rows name.

## Files changed
U1 (`79eb78f`): `crates/core/src/services/attributes.rs`, `crates/core/src/services/history.rs`,
`crates/core/src/services/publish.rs`, `crates/core/src/mechanisms/publish_partition.rs`,
`crates/testkit/tests/suite/research/w3_title.rs` (new), `crates/testkit/tests/suite/research/mod.rs`.
U2 (`e4f7e3e`): `crates/infra/src/settings_file.rs`, `crates/integrations/src/lib.rs`, `crates/core/src/services/settings.rs`,
`crates/research/src/writer/api.rs`, `crates/research/src/writer/run.rs`, `crates/research/src/api_impl.rs`,
`apps/desktop/src-tauri/src/commands/writer.rs`, `apps/desktop/src-tauri/src/lib.rs`,
`apps/desktop/src/services/bindings.ts` (generated), `apps/desktop/src/services/writer.ts`,
`crates/testkit/tests/suite/research/w3_writer_wiring.rs` (new), `crates/testkit/tests/suite/research/mod.rs`.
U3 (`f2e18ac`): `apps/desktop/src/features/form/{WriterBlock.tsx, WriterPanel.tsx}` (new),
`apps/desktop/src/features/form/__tests__/{w3_writer_review.test.tsx, w3_writer_panel.test.tsx}` (new),
`apps/desktop/src/features/sessions/__tests__/w3_writer_triggers.test.tsx` (new),
`apps/desktop/src/features/{form/AiReviewTab.tsx, form/FormDialog.tsx, form/ResearchPanel.tsx, sessions/reviewQueue.ts, shell/Header.tsx}`,
`docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md, 05_working.md}`.
Closing (`CHECKPOINT` closing commit): `docs/implementation/plans/W3_writer_review.md` (Tracker line, Review log),
this handoff. Mutation-probe files: none (no probes; flow §4).
No architecture graph in this repo.

## Checkpoint SHA
1. W3·U1 — `79eb78f`
2. W3·U2 — `e4f7e3e`
3. W3·U3 — `f2e18ac`
4. Closing (tracker, Review log, handoff) — the `CHECKPOINT (not approved): W3_writer_review closing` commit after these.

## Stamp
Once, on tree `f2e18ac` (all units committed, clean for code):
- `cargo test --workspace` → exit 0, 1818 passed, 0 failed (testkit suite 1008 in 506 s)
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0
- `cargo fmt --all -- --check` → exit 0
- `cd apps/desktop && npm run typecheck` → exit 0
- `npm test` → exit 0, 104 files, 648 tests
No pre-existing failures.

## Manual try-out
On scratch data (`PSM_DATA_DIR=$HOME/psm-research-dev`), research keys set and fields chosen in Settings → Research
(the shipped prices already cover the writer's model `gpt-6.1-sol`):
1. Let research finish on a session; open Details → AI review; check every suggestion; press **Next to review →**.
   The next session opens.
2. Go back to the first session after a minute: AI review lists a title, a description and a condition report, each
   with why and which facts it used; a question may sit above them.
3. Answer the question: the texts are written again within a minute.
4. Accept the title: the item's title changes; ⌘Z brings the old one back.
5. In the Product step press **Write texts**: new texts arrive; a text you dismissed does not come back word for
   word. While research still runs, the button says why in a toast.
6. Open the research panel: the Writer block shows what it used, its reasons, any check it failed and the cost.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
