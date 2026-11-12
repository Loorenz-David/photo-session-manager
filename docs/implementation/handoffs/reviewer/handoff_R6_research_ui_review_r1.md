---
plan: R6_research_ui
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-12
actor: opus (pipeline-reviewer, Opus 5.5)
---
# R6_research_ui — review round 1: APPROVED_WITH_NOTES

All 21 UI criterion rows and the carried release-event row hold at their boundaries. Each test observes the row's
outcome and would fail on a wrong one. No B1–B4. The checks are green. The two Rust deviations are necessary and
covered by the implementer's full-gate stamp. Nine notes went to the research backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Evidence
| What | Scope | Command | Tree | Result |
|---|---|---|---|---|
| Gate | — | tracker line; `git log` | `115812c` | IMPLEMENTED; `f6ad84b` `d0a56e2` `f365188` `4b3e09e` present |
| Perimeter | — | `git show --stat` per checkpoint; `git diff --stat f6ad84b^ 4b3e09e -- '*.rs'` | same | = handoff's per-unit lists (7 · 7 · 12 · 3); Rust: `commands/settings.rs`, `commands/attributes.rs`, `lib.rs` (declared Deviations 1–2) |
| UI checks (plan Review mode) | L2 | `npx vitest run src/features/form src/features/settings src/features/publish src/features/sessions` | clean `115812c` | 27 files, 159/159 |
| Typecheck | — | `npm run typecheck` | clean `115812c` | clean |
| Rust suite, clippy, fmt | L4 | consumed: implementer stamp (handoff §Stamp) | Rust files unchanged since `f365188` | 861/862 + p68 flake green on re-run; desktop lib incl. `r6_command_tests` green; clippy, fmt clean. Not re-run (tree match). |

## Verified correct
- **A18·C2(a)** `ResearchPanel` sorts by `seq`; `STEP_VERB` per kind; link only for `read` with `source_url`. The test feeds steps out of order and asserts the kind order, verbs and href, so it fails on a missing sort or a wrong verb.
- **A18·C2(b)** `useResearchStatus` re-reads on `onResearchChanged` for its session. `lib.rs:829` forwards `AppEvent::ResearchChanged` as `research_changed {session_id}`. The test goes through `FormDialog` and the captured handler: 1 → 2 steps, and the live indicator is present.
- **A18·C2(c)** `finishedLine` exact text. The test also covers 0 filled ("no fields changed", carried R4 note) and `stopped_at_cap`.
- **A18·C2(d)** Exact "Stopped at read: …" and a Run again button (`ended` → label).
- **A18·C3(a)** `runResearch(4)` called once, from the panel button inside the form.
- **A18·C3(b)** The backend sentences are shown as given. The code string `WaitingForEditor` matches `psm_research::domain.rs:292`. With fake timers, the sentence is held until `waitEndsAt`. Capture times are ms (`arrival.rs` `as_millis`), so the real wiring holds for live captures. For imports, see the notes.
- **A18·C4(a)** Dimming comes from `field_keys`, via the real AI popover → See research path. Clearing the focus chip undims.
- **A18·C5(a)** `toReview` counts live `ai`+`suggested` at every confidence (MC-A10 definition). The label opens the panel and disappears after Looks right reloads the form.
- **A19·C5(a–d)** The shop domain row has no remove control, and entries follow in order. On a refused add, the text is shown and the list is unchanged. Keys use password inputs that are never filled. There is one `setResearchSecrets` call with only the typed key, then "set" shows, and the key text is absent from the DOM. The warning appears only under `custom.maker`. The `SettingsPage` main save never sends `research`, so existing saves are unaffected by the new read-only block (`to_dto` fills it with `ResearchSettings::from_slot`).
- **A20·C1(a)** The line appears exactly once, inside `data-session="1"`.
- **A20·C2(a), C2(b)** Label, prior with "preset default" / "from Shopify", suggestion with "medium confidence", three enabled buttons, waits line singular. Confirm is enabled.
- **A20·C3(a)** `resolveConflict(11,'designer','keep_previous')` uses the item id from `getSessionView(7)`. The preview is read twice and the block disappears.
- **A20·C3(b)** `publishConfirm([7], summary+shown lines, [{7,1,['designer']}])`. The extra text is safe: `PublishService::confirm` stores `summary_text` in the run and never compares it.
- **A20·C3(c)** Matching is on `code === 'suggestions_changed'` (`ServiceError.code`; `items.rs:78`), never on text. The sentence is shown, the preview is re-read, the dialog stays open and no toast appears.
- **A20·C4(a)** The `waiting_for_you` pill, plural and singular, sits inside `PublishActivityPopover`, so Cancel is kept (S4). Serde `kind`/`value` matches `RowLabelResponse`.
- **A20·C5(a)** Conflict mark; popover shows "Replaces Unknown · preset default" and the three actions, with no Looks right or Clear (P1 kept for plain suggestions). Edit focuses the field. Every typed value of a conflict field goes to `conflictDrafts`, never to the debounce. Enter commits `resolveConflict(7,'designer','edit','Walnut')`, and a later blur does not commit again (N1 row).
- **A20·C5(b)** "3 to review" from two plain suggestions and one conflict, with exactly one conflict mark.
- **Carried release row** `emit_if_released` compares the held item before and after the action. The Rust test asserts no events after the first of two resolutions, then `session_changed` + `publish_progress(prepare)` for the session. The session list listens to both (`events.ts:7`).
- **Constraints.** H1: nothing new reaches Shopify without confirmation, and resolutions act on an item already confirmed. H2–H7: not touched. H8: logic stays in services, and the command only notifies (see note 9). No secrets in tests (`sk-test-0001` is fake). No live services.
- **Assumptions 1–9** stay within 08A/08B and the plan's carried items. None contradicts the book. The narrative lists them in plain words.

## Findings
None blocking. No should-fix (flow §5: everything outside B1–B4 is a note).

## Notes written to backlog
Written to `docs/implementation/backlog_research/R6_research_ui.md`, the plan's Review mode location. A pointer line was added to `docs/implementation/backlog.md`.
1. Settings → Research save is refused until R7·U1 accepts the `research` key (→ R7·U1 task 4).
2. Undo/redo that releases a held item emits no event.
3. The panel's "Waiting for Capture One" clock (latest `captured_at` + fixed 120 s) differs from the engine's (last change + `source_wait_seconds`). For imported photos the sentence can show early (→ R7·U1: the status should carry the wait end).
4. The form's conflict test is a TS copy of MC-A1 (lowercase rather than case fold; measurements compared as text). It can mark a field the backend does not treat as a conflict. The DTO could carry `conflict: bool`.
5. The publish popover's Edit sends text for every field type, so non-text conflicts are refused there.
6. Panel "Accept all (n)" counts conflicts that accept-all skips.
7. "Your shop" shows the `*.myshopify.com` host, not the primary domain.
8. Lesson for plans: Goal and Review mode said "no Rust" while a carried item required Rust. The tracker said `codex`, but the implementer was Claude.
9. Release detection reads the store inside the command layer. A service result would be thinner and would also cover undo/redo.

## Lessons for the plans
- A TypeScript-only gate must not carry a task that lives in the command layer. Name such tasks as Rust and put the full gate in the plan from the start (note 8).
- Rules with a Rust mechanism (MC-A1 sameness) should reach the UI as a computed field, not be re-implemented in TS (note 4).

## Write perimeter
- `docs/implementation/plans/R6_research_ui.md`: Tracker line (APPROVED) and a Review log entry.
- `docs/implementation/backlog_research/R6_research_ui.md` (new).
- `docs/implementation/backlog.md`: one pointer line appended.
- `docs/implementation/narratives/R6_research_ui.md` (new).
- This handoff.
No production or test file touched. No probes. No database or state side effects. No archgraph in this project. Nothing committed (the runner commits).
