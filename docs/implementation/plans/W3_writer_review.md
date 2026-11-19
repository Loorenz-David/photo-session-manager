---
phase: W3_writer_review
title: Texts in AI review — after "Next to review" (or leaving a session with everything checked) and from "Write texts" in the Product step, the writer drafts the title, description and condition report in the background; they arrive in AI review like research's suggestions, with the writer's questions and notes above them; a title suggestion changes the item's title only when you accept it; the research panel shows what the writer used, why it wrote each text, what the check found and what it cost
difficulty: high
projection: no
depends_on: [W2_writer_engine]
blocked_on: none
---
# W3 — Texts in AI review

**Tracker:** NOT_STARTED

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `08C` §5.1, §5.5, §5.6, §6 (surfaces), §11 by WC3, WC4.7–8 and WC6 of `master_plan_writer.md` §5. (U1) The title suggestion in core and the publish partition with the built-in fields. (U2) The `writer` settings slot, the wiring, the commands. (U3) The triggers, the AI review tab's writer block, the text rows, the research panel's writer block, the guide. **Not here:** Settings → Writer (W4: the writer runs on its defaults, and a hand-edited `writer` slot in `settings.json` is honoured); SEO fields (W5); Restoration (W6).

## How to work this plan (flow.md §10, master_plan_writer.md §3)
One runner phase of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `W3·C<n>(<row>)`; tests named after their row. Open details: the smallest option consistent with 08C, A11, A18, A19 and the contracts, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_writer/W3_writer_review.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The title suggestion; publish partition with the built-in fields | 3 | 9 | **yes** (core) |
| U2 | The `writer` slot, wiring, commands | 3 | 9 | **yes** |
| U3 | Triggers, the writer block in AI review, text rows, the panel block, the guide | 4 | 12 | **yes** (TypeScript, guide) |
| **Total** | | **10** | **30** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §7A, §10) · `master_plan_writer.md` (all; **§5 WC3, WC4.7–8, WC6**) · `master_plan_research.md` §5 Seams, §6.6 · book `08C` §5, §6, §11 · `08B` MC-A10, MC-A12 · changelog `16_changelog.md` A11, A18, A19 · the W2 plan · `docs/guide/02_architecture.md` §Research (review queue), §Publish · `docs/guide/04_change_impact.md` rows "Tauri command/DTO", "settings field", "DB change".

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **A title suggestion becomes the item's title only when you accept it;** until then the shop keeps the old title, even if you publish.
- **A writer description that replaces the shop's description is a conflict at publish,** like research's values: the item waits until you choose.
- **The writer starts by itself when you press "Next to review", or when you leave a session whose AI review says "All checked"; once per session.** "Write texts" in the Product step writes again whenever you like.
- **Answering a question rewrites the texts** (no photos, one or two calls) and the answer is kept for the item.
- **A dismissed note never comes back,** and the counts of answered and dismissed notes are kept per type for Settings → Writer.
- **Until Settings → Writer exists (W4) the writer uses its shipped defaults.**

## Unit 1 — The title suggestion; publish partition with the built-in fields

**Unit id:** `W3·U1` · **Rows:** W3·C1–C3 · **Changes app files:** yes (core)

### Goal
WC4.7 (the form, Accept, Keep previous, Dismiss, human edit, undo for `product.title`) and WC4.8 (partition definitions and the title rows).

### Read first
Master plan §5 WC4.3, WC4.7, WC4.8 · `crates/core/src/services/attributes.rs` (`form_get`, `title_value`, `write_title`, `accept`, `accept_all`, the keep-previous and conflict actions, `note_action`) · `crates/core/src/services/history.rs` (`FieldState::Title`, the conditional undo of MC-A12) · `crates/core/src/services/publish.rs` (`partition_in`, `confirm`) · `crates/core/src/mechanisms/publish_partition.rs` · `crates/testkit/tests/suite/research/{r5_feedback_and_undo.rs, r5_publish_hold.rs}`.

### Files
Edited: `crates/core/src/services/{attributes.rs, history.rs, publish.rs}`, `crates/core/src/mechanisms/publish_partition.rs`. New: `crates/testkit/tests/suite/research/w3_title.rs`; `crates/testkit/tests/suite/research/mod.rs` gains its line.

### Core mechanisms
`mechanisms::publish_partition::partition` (the `product.title` rule).

### Ordered tasks
1. `form_get`: the `product.title` field's value per WC4.7.
2. Accept / Use suggestion / Accept all, Keep previous, Dismiss (clear), human edit and undo/redo for `product.title` per WC4.7, each one undo step with its feedback row.
3. `partition` skips `product.title` rows; `partition_in` passes `builtin_definitions()` ∪ the shop's definitions.
4. Tests for the rows (suggestions are written through `psm_research::writer::writes` or the store, as the R5 tests do).

### Criteria (3 criteria, 9 rows)
Boundary: `AttributesService` and `PublishService` through `TestApp` (outcome = the form, `items.title`, rows, feedback rows, undo results, the publish preview and items, the calls the fake commerce adapter received).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W3·C1(a) | `form_get` | `items.title` "Shop chair" from the link; a `product.title` row `ai` + `suggested` "Børge Mogensen J39 Chair in Beech, 1960s" with prior "Shop chair" | the title field's value is the suggestion row (provenance ai, status suggested, `ai_prior` "Shop chair") | WC4.7 |
| W3·C1(b) | same | after the owner types "My title" over it | the title field is "My title", provenance human; no `product.title` row exists; one feedback row `edited` | WC4.7, MC-A12 |
| W3·C2(a) | `accept`, then undo, then redo | the suggestion of C1(a) | after accept: `items.title` = the suggestion, the row `ai` + `accepted`, one feedback `accepted`; after undo: `items.title` "Shop chair", the row `ai` + `suggested`, no feedback row; after redo: as after accept | WC4.7, MA18 |
| W3·C2(b) | keep previous | the same | the row is gone; `items.title` "Shop chair"; feedback `kept_previous` with the suggestion; a later writer text equal to it is not written (cleared set) | WC4.7, WC4.4 |
| W3·C2(c) | dismiss (clear) | `items.title` blank; a suggestion row with no prior | the row `ai` + `cleared`; `items.title` blank; feedback `cleared` | WC4.7 |
| W3·C2(d) | `accept_all` | a `product.title` suggestion at `medium` (no prior), a description suggestion at `low` | `items.title` = the title suggestion; the description stays `suggested` | WC4.7, MC-A12 |
| W3·C3(a) | `publish_preview`, `publish_confirm`, then the run | a linked item; `product.description` `ai` + `suggested` `medium` with prior {"Shop text", integration} | the preview lists `product.description` as a conflict; the item is `held`; no product update reaches the fake adapter before it is resolved | WC4.8, MA17 |
| W3·C3(b) | same | `items.title` "Shop chair"; a `product.title` suggestion `medium` with no prior; nothing else pending | the preview's withheld count and conflicts do not name `product.title`; the item runs; the fake adapter receives no title update (remote title "Shop chair" equals `items.title`) | WC4.8 |
| W3·C3(c) | same | the title suggestion accepted before confirming | the fake adapter receives one title update with the accepted text | WC4.8, M20 |

### Delegated choices
Where the title rule lives in `attributes.rs` (inside `write_title`, or a helper both actions call) · the undo step's label.

### Notes
- Hazard: `history.rs` restores `FieldState::Title` and attribute rows separately; Accept must record both in one step, or undo restores `items.title` and leaves the row `accepted` (C2(a)).
- Hazard: adding the built-in definitions to `partition_in` changes nothing for research rows (research never writes Product fields); the R5 publish-hold tests must stay green.

### Unit test commands
`cargo test -p psm-core mechanisms::publish_partition` · `cargo test -p psm-testkit --test suite research::w3_title:: research::r5_feedback_and_undo:: research::r5_publish_hold:: p10_attributes:: p39_undo_history_core:: p40_undo_history_values:: p13_publish_runs:: p14_publish_update::` · `cargo clippy -p psm-core --all-targets -- -D warnings`

## Unit 2 — The `writer` slot, wiring, commands

**Unit id:** `W3·U2` · **Rows:** W3·C4–C6 · **Changes app files:** yes

### Goal
`psm_infra::Settings.writer`, the rebuild comparison, `ResearchDeps.writer_settings` from the slot, `WriterApi::status`, the commands `writer_status`, `writer_request`, `writer_finding_answer`, `writer_finding_dismiss`.

### Read first
Master plan §4 Settings, API, Commands · `crates/infra/src/settings_file.rs` · `crates/integrations/src/lib.rs` (`build_research`, the settings prepare with its rebuild comparison) · `apps/desktop/src-tauri/src/commands/{writer.rs, research.rs}` · `apps/desktop/src-tauri/src/lib.rs` (command tests) · `crates/testkit/tests/suite/research/r7_wiring.rs`.

### Files
Edited: `crates/infra/src/settings_file.rs`, `crates/integrations/src/lib.rs`, `crates/research/src/writer/api.rs` (`status`), `apps/desktop/src-tauri/src/{commands/writer.rs, lib.rs}`, `apps/desktop/src/services/{bindings.ts (generated), writer.ts}`. New: `crates/testkit/tests/suite/research/w3_writer_wiring.rs`.

### Core mechanisms
None.

### Ordered tasks
1. `Settings.writer: Option<Json>`; the rebuild comparison nulls `writer` (beside `theme`); a saved `writer` slot is validated with `WriterSettings::validate` as the research slot is.
2. `build_research` passes `writer_settings` reading the live slot.
3. `WriterApi::status(session) -> WriterStatus {reason, run, texts: [{key, label, flagged, hits}], findings: [open findings], steps, cost_micro_usd, inputs: {fields, note, listings}, stale_main, stale_condition, can_write}`; the four commands with their DTOs; regenerate the bindings.
4. Tests for the rows.

### Criteria (3 criteria, 9 rows)
Boundary: the wired app (`TestApp`, research fakes) through the command layer; `settings_set`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W3·C4(a) | `settings_set` | a patch `{"writer": {"auto": false}}` | the saved file holds it; the App is not rebuilt (the prepared settings are the simple kind, as for `theme`); `writer_request(…, next_to_review)` then queues nothing | WC3.4, 08C §8 |
| W3·C4(b) | `settings_set` | a patch `{"writer": {"max_questions": 99}}` | refused with an error naming `max_questions`; the file is unchanged | 08C §8 |
| W3·C5(a) | `writer_request` (`write_texts`), then ticks | a researched session (fakes), the fake model scripted for stage `write` | answers `queued: true`; the run finishes; the form shows the three text suggestions; a `research_changed` event names the session | WC3, 08C §5.5 |
| W3·C5(b) | `writer_request` (`write_texts`) | the session's research run is `running` | `queued: false`, reason sentence "Research is still working on this session; write texts when it finishes" | WC3.2 |
| W3·C5(c) | `writer_status` | after C5(a) with one text flagged and one open ask | `run.state` finished, `texts` lists the flagged one with its hits, `findings` lists the ask with type, keys and mode, `cost_micro_usd` equals the run's cost, `inputs` counts the fields, observations and listings received | 08C §11 |
| W3·C5(d) | `writer_finding_answer`, then ticks | the open ask; answer "Teak" | the finding is answered; a new `write` run runs and its input's `answers` hold "Teak" | WC6, 08C §5.6 |
| W3·C5(e) | `writer_finding_dismiss` | an open note | state dismissed; no run queued | WC6 |
| W3·C6(a) | the desktop command test | an app without the research engine | the four commands answer `research_unavailable`; `bindings.ts` contains the four names | HW7, MA7 |
| W3·C6(b) | the wired app with no model key | `writer_request(write_texts)` | `queued: false` with "Add the model key in Settings → Research"; sessions, review and publish unaffected (an existing publish test's session publishes) | HW7, MA7 |

### Delegated choices
DTO field names beyond those listed · whether `status` reads the latest `write` run or the latest run of either writer kind.

### Notes
- Hazard: without nulling `writer` in the comparison every writer save rebuilds the App (C4(a)).

### Unit test commands
`cargo test -p psm-testkit --test suite research::w3_writer_wiring:: research::r7_wiring::` · `cargo test -p psm-desktop` · `cargo clippy -p psm-integrations -p psm-infra -p psm-desktop --all-targets -- -D warnings`

## Unit 3 — Triggers, the writer block in AI review, text rows, the panel block, the guide

**Unit id:** `W3·U3` · **Rows:** W3·C7–C10 · **Changes app files:** yes (TypeScript, guide)

### Goal
The frontend of WC3 ("Frontend calls"), 08C §5.5–5.6 (questions and notes above the suggestions; answers), the long-text suggestion rows, 08C §11 (the panel block), and the guide.

### Read first
Master plan §5 WC3 "Frontend calls" · `apps/desktop/src/{App.tsx (goNextToReview), features/sessions/reviewQueue.ts, features/shell/Header.tsx, features/form/{FormDialog.tsx, AiReviewTab.tsx, AiPopover.tsx, ResearchPanel.tsx, StepTabs.tsx}, services/{writer.ts, research.ts, events.ts}}` · the A11/A18/A19 UI tests in `features/form/__tests__/` and `features/sessions/__tests__/` · `docs/intention/evidence/design_handoff.md`.

### Files
New: `apps/desktop/src/features/form/{WriterBlock.tsx, WriterPanel.tsx}`, `apps/desktop/src/features/form/__tests__/{w3_writer_review.test.tsx, w3_writer_panel.test.tsx}`, `apps/desktop/src/features/sessions/__tests__/w3_writer_triggers.test.tsx`. Edited: `apps/desktop/src/{App.tsx, features/sessions/reviewQueue.ts, features/shell/Header.tsx, features/form/{FormDialog.tsx, AiReviewTab.tsx, ResearchPanel.tsx}}`; `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md, 05_working.md}` (the writer: what it is, the flow, HW1–HW7, the couplings of WC1.9 and WC4.7–8, the commands and log lines).

### Core mechanisms
None.

### Ordered tasks
1. Triggers (WC3): "Next to review →" in the AI review tab and the header's "Next to review" call `writerRequest(session, 'next_to_review')` and then move on as today; closing Details or leaving the session page while that item's AI review shows "All checked ✓" calls `writerRequest(session, 'all_checked')` once; a **Write texts** button in the Product step calls `writerRequest(session, 'write_texts')` and shows a refusal reason as a toast.
2. `WriterBlock` at the top of the AI review tab: "Writing texts…" while the latest `write` run is queued or running; each open ask with its text, a one-line answer box and **Answer**; each open note with **Dismiss**; the empty reason of the tab mentions the writer when a `write` run is the cause.
3. Text suggestion rows: a long text shows the previous text and the suggestion as paragraphs; a flagged text shows its "Check:" line; the title row's actions are the existing Accept / Keep previous / Dismiss / Edit.
4. `WriterPanel` in the research panel: its own block for the latest `write` run: inputs used (fields, observations, listing titles), per text the reason and facts, post-check hits per attempt and "rewritten once" when a retry happened, findings and questions, cost.
5. The guide (flow §7A).
6. Tests for the rows.

### Criteria (4 criteria, 12 rows)
Boundary: the named component rendered in a UI test with `services/writer.ts`, `services/research.ts` and `services/attributes.ts` mocked (outcome = what is shown and the service calls made).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| W3·C7(a) | `AiReviewTab` | all suggestions checked; click "Next to review →" | `writerRequest` called once with (the session, `next_to_review`), then the navigation callback | WC3 (frontend) |
| W3·C7(b) | `FormDialog` | AI review shows "All checked ✓"; close Details; then (another render) close with one suggestion still unchecked | first: `writerRequest(session, 'all_checked')` once; second: not called | WC3 (frontend) |
| W3·C7(c) | Product step | click **Write texts**; the mock answers `queued: false` with "Research is still working on this session; write texts when it finishes" | `writerRequest(session, 'write_texts')` once; a toast shows that sentence | WC3, HW7 |
| W3·C7(d) | `Header` | an arranged session with "Next to review (2 left)"; click it | `writerRequest(current session, 'next_to_review')` once before the queue moves on | WC3 (frontend) |
| W3·C8(a) | `WriterBlock` | status with one open ask ("Is the frame teak or oak?", keys wood); type "Teak", click **Answer** | the question shows above the suggestions; `writerFindingAnswer(id, "Teak")` called once | `[rule]` 08C §5.5, §5.6 |
| W3·C8(b) | same | one open note; click **Dismiss** | `writerFindingDismiss(id)` once; the note leaves after the re-read | `[rule]` 08C §6 |
| W3·C8(c) | same | the latest `write` run `running` | "Writing texts…" shows | `[rule]` 08C §5 |
| W3·C9(a) | AI review row | a description suggestion of two paragraphs with a prior of one paragraph | the previous text and both suggested paragraphs show; Accept calls `attributeAiAccept(item, 'product.description')` | `[rule]` 08C §5.5 |
| W3·C9(b) | same | a flagged condition report (confidence low, reason with "Check: does not name Condition.Stains/Discoloration") | the low badge and the "Check:" line show | `[rule]` WC5 |
| W3·C9(c) | same | a `product.title` suggestion with prior "Shop chair" | the row shows "Shop chair → <suggestion>" with Accept, Keep previous and Edit | `[rule]` WC4.7 |
| W3·C10(a) | `WriterPanel` | a finished `write` run: 12 fields, 5 observations, 3 listing titles, two attempts with hits in the first, one note, cost 21 000 µUSD | the block shows the three inputs counts, each text's reason and facts, the first attempt's hits and "rewritten once", the note, and the cost "$0.02" in the research panel's money format | `[rule]` 08C §11 |
| W3·C10(b) | same | no `write` run | no writer block | `[rule]` 08C §11 |

### Delegated choices
Placement and look of the Write texts button (design handoff) · how "leaving the session" is detected (route change or unmount of the session page) as long as C7(b) holds · the money format helper (reuse the research panel's).

### Notes
- Hazard (A19): "Next to review" must still move on when `writerRequest` fails or answers `queued: false` (an automatic origin never blocks the owner's flow).

### Unit test commands
`cd apps/desktop && npx vitest run src/features/form src/features/sessions src/features/shell` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time. A failure is fixed and only the failing files re-run.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-testkit --test suite research::` · `cargo test -p psm-core mechanisms::publish_partition` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/form src/features/sessions src/features/shell` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: no path other than the owner's actions writes `items.title` from a suggestion; the partition never freezes a `product.title` row; an automatic writer request cannot block "Next to review". Notes go to `docs/implementation/backlog_writer/W3_writer_review.md`.

## Manual try-out (the narrative is built from this)
On scratch data with research keys and chosen fields:
1. Let research finish on a session; check every suggestion in AI review; press **Next to review →**. The next session opens.
2. Go back to the first session after a minute: AI review lists a title, a description and a condition report, each with why and which facts it used. A question may sit above them.
3. Answer the question: the texts are written again within a minute.
4. Accept the title: the item's title changes; undo brings the old one back.
5. In the Product step press **Write texts**: new texts arrive; a text you dismissed does not come back word for word.
6. Open the research panel: the writer's block shows what it used, its reasons, any check it failed and the cost.

## Review log
