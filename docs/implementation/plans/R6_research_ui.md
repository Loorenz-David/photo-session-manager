---
phase: R6_research_ui
title: Research on screen — the panel shows the steps as research runs and why it is not running when it is not, Settings gets a Research section for keys, trusted sites, models and limits, and the publish review shows left-out weak suggestions and conflicts with Keep previous / Use suggestion / Edit
difficulty: normal
projection: no
depends_on: [R5_research_app_seams]
---
# R6 — UI: the live research panel, Settings → Research, the publish review

**Tracker:** IMPLEMENTED · 2026-11-12 · codex · 3 units (U1 f6ad84b, U2 d0a56e2, U3 f365188); 21 UI rows + 1 Rust row; full gate: suite 861+1 flake re-run green, clippy, fmt, typecheck, vitest 487 green; Rust deviations in settings/attribute commands.

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
Frontend only, against the bindings frozen in R2 and extended in R5, with the services mocked in tests (`vi.mock`): the research panel and the form header, the Research section of Settings, the publish popover's withheld line and conflict block, the "Waiting for you" row label, conflict marks in the form. **Not here:** any Rust change (one found necessary is a Deviation and makes the closing gate the full one), the only-if-cheap items of 08A §12.

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 3 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R6_research_ui.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The live research panel and "n to review" | 4 | 8 | yes |
| U2 | Settings → Research | 1 | 4 | yes |
| U3 | Publish review and form: withheld line, conflicts, "Waiting for you" | 5 | 9 | yes |
| **Total** | | **10** | **21** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`. Also `docs/intention/evidence/design_handoff.md` and `claude_design/NOTES.md` §Screens 7, §AI agent rules.

## Carried from earlier reviews (orchestrator, 2026-11-12; wins over older wording below)
- **U3, the "Waiting for you · n conflicts" pill (R2 review).** `StatusPill.tsx` has no `waiting_for_you` branch today, so a held session row shows an empty status cell. Add it (08B MC-A10) with its activity popover.
- **U1, "n fields filled" (R4 review).** `derived_counts` counts only stage 7 writes with outcome `written`; a follow-up that confirms the same values shows "0 fields filled · 3 to review". The panel must not read as if research did nothing on a fully suggested form: show the fields the latest run supports (written or unchanged), or word the line so 0 is not misleading.
- **U1, "Waiting for Capture One" timing (R4 review).** `run_rules::may_start` returns `WaitingForEditor` as soon as no eligible photo has a source, before the 120 s source wait ends; MC-A6 says the panel says it "when the wait ends". Show the sentence only once the wait has ended (the start behaviour itself is correct).
- **U3, Edit on a conflict commits once (R5 projection N1).** On a held item, the first save of a pending key resolves it, and a
  step that resolved is never coalesced into. So a conflict field's **Edit** saves once, on explicit commit (Enter,
  blur or Save), never on the form's 300 ms typing debounce; otherwise the shop gets the first typed prefix. Add a
  UI row: typing "Walnut" into a conflict's Edit calls the resolve command once, with "Walnut".
- **U3, the confirmed text (R5 projection N8).** If the popover shows the withheld and "waiting for you" lines, the
  `summary` text passed to confirm carries them (07A MC-10: the exact text shown).
- **U3, Looks right only on suggestions (R5 projection P1).** R5 made the popover offer Looks right only on `ai` +
  `suggested`; keep that when U3 edits `AiPopover.tsx` for conflicts.
- **U3, the confirm refusal (R5 projection P5).** A changed-suggestions refusal arrives as command code
  `suggestions_changed` (`Error::SuggestionsChanged`); rebuild the preview on that code only, never by matching text.
- **U3, held item whose field left the form (R5 projection S4).** Pending is decided at confirmation; if the
  photographer later removes a pending field from the form, the held label's activity popover keeps Cancel (R5
  gives `can_cancel` for held items), which is the way out.
- **U3, release emits events (R5 review).** `resolve_pending_in` → `StoreTx::release_publish_item` emits nothing, so the
  session row keeps "Waiting for you · n conflicts" until the next list read. After an attribute action that released
  a held item, the command layer emits `session_changed` and `publish_progress` (after the action returns). Add a row:
  resolving the last conflict flips the row to "Publishing" without a manual refresh.
- **U3, undo after release (R5 review; contract-conformant).** Undoing the resolution after the item was released
  restores the conflict in the form while the run already sent the resolved value (MC-A10 removes the entry only while
  held). In the form, such a field's conflict actions stay available, and the activity popover or field hint says the
  publish already used the value you chose; a next publish holds on it again. Keep it to wording; no new backend rule.
- **Tests (R5 review).** `p15_publish_ui.test.tsx` C5(a): give the preview mock sessions 1 and 2 so the shown-set
  assertion reflects the queue, not the session-7 fixture, when U3 touches that file.

## Unit 1 — The live research panel and "n to review"

**Unit id:** `R6·U1` · **Replaces:** A18 criteria C2–C5 · **Changes app files:** yes

### Goal
The research panel rebuilt from stored steps: live while a run is `running`, with a Finished state ("n fields filled · n to review"), the reason when research cannot run, and **Research** / **Run again**; the form header shows "n to review" and opens the panel. Frontend only. **Not in this unit:** the only-if-cheap items of 08A §12 (cancel a run, research one field again, show cost).

### What changed from the superseded plan (this wins over any older wording below)
- Commands, the event and `services/research.ts` exist since R2·U2; the backend rows (A18·C1, A18·C6) are in R4·U6. This unit is the panel and the form header only: tasks 3 and 4 of the older list.
- While the engine is not wired (before R7) `research_status` fails with `research_unavailable`: the panel then shows no state line and no button, and the form works as before.

### Read first
`master_plan.md` §3, §6.7, §6.9 (frontend tests) · `master_plan_research.md` §6.6 (commands, event, sentences) ·
book `08A` §1, §5 (Not determined), §7, §8 (`research_steps`; derived values), `08B` MC-A6 (prerequisites, panel
sentences), MC-A10 ("n to review") · evidence `design_handoff.md` (AI pill, popover, panel rules), `claude_design/
NOTES.md` §Screens 7 and §AI agent rules · code `apps/desktop/src/features/form/{ResearchPanel.tsx, FormDialog.tsx
(lines 34, 323), AiPopover.tsx, AiPill.tsx}`, `apps/desktop/src/services/{attributes.ts, events.ts, bindings.ts}`,
`apps/desktop/src-tauri/src/{commands/attributes.rs, commands/mod.rs, lib.rs (collect_commands), state.rs}`,
`crates/research/src/{mod.rs, runs.rs}` (`status`, `run_again`).

### Files
New: `apps/desktop/src/features/form/__tests__/r6_research_panel.test.tsx`. Edited: `apps/desktop/src/features/form/{ResearchPanel.tsx, FormDialog.tsx}`, `apps/desktop/src/services/{research.ts, events.ts}`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `research_status(session_id) -> ResearchStatus {reason: Option<code + sentence>, run: Option<{id, state, trigger,
   stage_reached, error}>, steps: [{seq, stage, kind, summary, detail, source_url, source_title, photo_ids,
   field_keys}], fields_filled, to_review, can_run}` and `research_run(session_id)`. Derived, never stored:
   `fields_filled` = keys the latest run wrote; `to_review` = live `ai` + `suggested` rows whose key is in the
   current layout, at every confidence.
2. Forward `AppEvent::ResearchChanged` as the Tauri event `research_changed {session_id}`.
3. Panel: the steps of the latest run in order, each with its kind's verb (Looked, Searched, Read with the source
   link, Compared, Concluded, Not determined) and detail; while `running`, a live indicator and new steps appended
   on `research_changed`; when `finished` or `stopped_at_cap`: `Finished · n fields filled · n to review` (for a
   capped run the line also says the run stopped at its cost limit); when `failed`: the stage and error and **Run
   again**; when there is no run and research can run: **Research**; when it cannot: the sentence. Focusing a field
   from its popover dims the steps whose field keys do not include it (existing behaviour, now from `field_keys`).
   Accept all stays, unchanged.
4. Form header: `n to review` (hidden at 0), opening the panel.

### Criteria (4 criteria, 8 rows, 7 `[rule]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A18·C2(a) | panel (UI test) | status with five steps of kinds looked, searched, read, compared, concluded | five items in `seq` order with their verbs; the read step links its source URL | `[rule]` 08A §8 |
| A18·C2(b) | panel | run `running`; a `research_changed` event arrives and the status now has one more step | the new step is shown without reopening; the live indicator is present | `[rule]` 08A §1 |
| A18·C2(c) | panel | run `finished`, 3 filled, 2 to review | `Finished · 3 fields filled · 2 to review` | `[rule]` 08A §3 (design mock-up) |
| A18·C2(d) | panel | run `failed` with stage `read` and an error | both shown; a **Run again** button | `[rule]` 08A §10 |
| A18·C3(a) | panel | no run, `can_run` true; click **Research** | `research_run` called with the session id | `[rule]` 08A §5 |
| A18·C3(b) | panel | reason `DayLimit`; reason `WaitingForEditor` | `Day limit reached`; `Waiting for Capture One`; no Research button for the first | MA7 |
| A18·C4(a) | panel | focus on field `designer` from its popover | steps whose field keys lack `designer` are dimmed; clearing the focus undims | `[rule]` design_handoff §AI |
| A18·C5(a) | form header | `to_review` 2; then 0 | `2 to review`, opening the panel on click; then no label | `[rule]` 08A §7 |

### Delegated choices
Visual details within the design handoff's tokens and components · how reused units of a follow-up are shown (one
line "n earlier steps reused") · polling fallback if an event is missed (re-read on panel open). Refine at prompt
time against `claude_design/`.

### Notes
- No `implementer: fable` (flow log v1.3).

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cd apps/desktop && npx vitest run src/features/form` · `npm run typecheck`

## Unit 2 — Settings → Research

**Unit id:** `R6·U2` · **Replaces:** A19 criterion C5 · **Changes app files:** yes

### Goal
The Research section of Settings: keys (never read back), the trusted-sites list with add, remove and reorder (the shop's own domain first and fixed), model, effort and image size per stage, cost limits, price rows, language, the automatic switch, quiet period, record days, attribution keys. Frontend only, calling the commands frozen in R2·U2. **Not in this unit:** the Export button (R7·U2).

### What changed from the superseded plan (this wins over any older wording below)
- The commands exist since R2·U2 and their behaviour is R4·U6 (rows A19·C1–C4). This unit is the section only: task 5 of the older list, calling the frozen commands.

### Read first
`master_plan.md` §6.7, §6.8 · `master_plan_research.md` §3 (keys), §6.6 (accounts, settings keys, commands) · book
`08A` §5, §6 (Trusted sites), §9, `08B` MC-A3 (Trusted entry, Shop hosts), MC-A6 (eligibility re-evaluated on every
save of a research setting or key; `auto` off), MC-A8 (decimal strings, at most 6 fractional digits), MC-A11
(`record_days` minimum 2) · code `apps/desktop/src/features/settings/{SettingsPage.tsx, CameraSection.tsx,
StorageSection.tsx}` (section pattern, secrets entry for the camera login), `apps/desktop/src-tauri/src/commands/
{settings.rs, camera.rs (camera_secrets_set)}`, `crates/infra/src/{settings_file.rs, keychain.rs}`,
`crates/research/src/rules/sites.rs`, `crates/research/src/runs.rs` (`settings_changed`).

### Files
New: `apps/desktop/src/features/settings/{ResearchSection.tsx, __tests__/r6_research_settings.test.tsx}`. Edited: `apps/desktop/src/features/settings/SettingsPage.tsx`, `apps/desktop/src/services/{research.ts, settings.ts}`.

### Core mechanisms (the only functions with unit tests)
None new.

### Ordered tasks
1. `research_secrets_set({model_key?, search_key?, reader_key?})` writes the given keys through the secrets port;
   `research_secrets_state()` returns only which are set. No command ever returns a key.
2. `research_site_add(text)`, `research_site_remove(entry)`, `research_sites_reorder(entries)`: `TrustedSites` (R3·U2)
   with the shop hosts from `public_listing`; a refusal's text is shown as is.
3. The other fields save through `settings_set` with a `research` patch; the service refuses: a cap or price that is
   not a decimal string with at most 6 fractional digits, `record_days` below 2, a negative `quiet_seconds`, a long
   edge that is not a positive integer.
4. Every successful save or key change calls `settings_changed()` (R4·U4).
5. The section: keys (masked, "set" / "not set"); trusted sites (the shop's domain first, labelled always trusted,
   not removable; the list with drag or up/down reorder and remove; an add field); one row per stage (model,
   effort, image long edge); caps; price rows; language; Automatic research switch; quiet period; record days;
   attribution keys with a warning line under a key that is not one of the shop's field keys.

### Criteria (1 criteria, 4 rows, 3 `[rule]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A19·C5(a) | section (UI test) | load with two entries and the shop domain | the shop domain is first, has no remove control; the two entries follow in order with remove controls | `[rule]` 08A §6 |
| A19·C5(b) | section | add refused by the service | the refusal text is shown; the list is unchanged | `[rule]` 08A §6 |
| A19·C5(c) | section | the key fields | never show a stored key; show "set" after saving; the save call carries the typed value once | MA16 |
| A19·C5(d) | section | attribution key `custom.designer` exists in the definitions, `custom.maker` does not | a warning under `custom.maker` only | `[rule]` master_plan_research §8 |

### Delegated choices
Layout within the settings page's existing pattern · reorder control (drag with dnd-kit, or up/down buttons) · how
price rows are edited (a small table). Refine at prompt time against `claude_design/`.

### Notes
- No `implementer: fable`. Keys are typed into the app, never into a file in the repo.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cd apps/desktop && npx vitest run src/features/settings` · `npm run typecheck`

## Unit 3 — Publish review and form: withheld line, conflicts, "Waiting for you"

**Unit id:** `R6·U3` · **Replaces:** A20 (all rows) · **Changes app files:** yes

### Goal
Show what R5·U2 computes: in the confirmation popover the line "n low-confidence AI suggestions will not be
published" and, per item, its conflicts as a warning with the three actions; pass the shown set to the
confirmation and rebuild on refusal; the session row label "Waiting for you · n conflicts"; conflict marks and the
same three actions in the form. **Confirm is never disabled by a conflict.** **Not in this phase:** any change to
the rules (R5·U2), the panel (R6·U1).

### Read first
`master_plan_research.md` §6.6 · book `08A` §7, `08B` **MC-A10** (the popover's content and texts; what is shown and
re-checked; status and label), MC-A12 (actions) · evidence `design_handoff.md` · code
`apps/desktop/src/features/publish/{PublishPopover.tsx, usePublish.ts, BatchQueue.tsx}`,
`apps/desktop/src/features/sessions/StatusPill.tsx`, `apps/desktop/src/features/form/{FormDialog.tsx, AiPopover.tsx,
MetafieldInput.tsx}`, `apps/desktop/src/services/{publish.ts, attributes.ts, bindings.ts}`, R5·U2's preview fields.

### Files
New: `apps/desktop/src/features/publish/{ConflictList.tsx, __tests__/r6_research_publish.test.tsx}`, `apps/desktop/src/features/form/__tests__/r6_conflicts_in_form.test.tsx`. Edited: `apps/desktop/src/features/publish/{PublishPopover.tsx, usePublish.ts}`, `apps/desktop/src/features/sessions/StatusPill.tsx`, `apps/desktop/src/features/form/{FormDialog.tsx, AiPopover.tsx}`, `apps/desktop/src/services/{publish.ts, attributes.ts}`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. Popover: when an item's `n_withheld` > 0, the line `n low-confidence AI suggestions will not be published`.
2. Popover: for each item with conflicts, a warning block listing per conflict the field label, the previous value
   and where it came from (`preset default` / `from Shopify`), the suggested value and its confidence, the buttons
   **Keep previous**, **Use suggestion**, **Edit**, and the line `This item waits until you resolve n conflicts`.
   A resolution calls `attribute_conflict_resolve` and re-reads the preview.
3. Confirm passes the `n_withheld` and conflict keys shown for each item. On the refusal `AI suggestions changed
   while you were confirming` the popover shows that sentence and rebuilds from a new preview.
4. Row label `Waiting for you · n conflicts` for `RowLabel::WaitingForYou(n)`.
5. Form: a field in conflict is marked and its popover offers the same three actions in place of Looks right /
   Clear; resolving there has the same effect.

### Criteria (5 criteria, 9 rows)
Boundary: UI tests with the services mocked (`vi.mock`), as `master_plan.md` §6.9.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A20·C1(a) | popover | preview with `n_withheld` 2 for one item; 0 for another | `2 low-confidence AI suggestions will not be published` once, under the first item | MA8 |
| A20·C2(a) | popover | an item with one conflict: label `Designer`, prior `Unknown` from the preset, suggestion `Hans J. Wegner` at medium | the block shows the label, `Unknown`, `preset default`, `Hans J. Wegner`, medium confidence, the three buttons and `This item waits until you resolve 1 conflict`; Confirm is enabled | MA17 |
| A20·C2(b) | popover | a conflict whose prior came from the shop | `from Shopify` | MC-A10 |
| A20·C3(a) | resolve in the popover | click **Keep previous** | `attribute_conflict_resolve(item, key, keep_previous)` called; the preview is read again; the block is gone when the new preview has no conflict | MC-A12 |
| A20·C3(b) | confirm | preview showed `n_withheld` 1 and conflict keys `{designer}` | `publish_confirm` receives exactly that shown set for the session | MA17 (R-IA22) |
| A20·C3(c) | refusal | confirm rejects with `AI suggestions changed while you were confirming` | the sentence is shown; the preview is requested again; the popover stays open | MA17 |
| A20·C4(a) | session row | label `WaitingForYou(2)` | `Waiting for you · 2 conflicts` | M14 |
| A20·C5(a) | form | a field in conflict | marked; its popover shows Keep previous / Use suggestion / Edit and the previous value with its origin; **Edit** puts the field in edit mode and the typed value is saved as a human edit | MC-A12 |
| A20·C5(b) | form header | two plain suggestions and one conflict | `3 to review` | MC-A10 |

### Delegated choices
Visual treatment of the warning within the design tokens · singular/plural wording (`1 conflict` / `n conflicts`)
· where Edit opens (the form field, focused). Refine at prompt time against `claude_design/`.

### Notes
- No `implementer: fable`.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cd apps/desktop && npx vitest run src/features/publish src/features/form src/features/sessions` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cd apps/desktop && npm run typecheck` · `npm test` (the whole UI suite, once). **No cargo test**: no Rust file changed, so no Rust test can change (flow §10's reason for research-only units, applied to a TypeScript-only milestone; the planner's reading, recorded in `master_plan_research.md` §8). If a Rust file did change, run the full gate instead: `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`.

## Review mode
The reviewer runs `npx vitest run src/features/form src/features/settings src/features/publish src/features/sessions` and `npm run typecheck`, and checks with `git diff --stat` that no Rust file changed. Notes: `docs/implementation/backlog_research/R6_research_ui.md`.

## Manual try-out (the narrative is built from this)
(Research itself is wired in R7; until then the panel says nothing is available.) On scratch data: 1. Settings → Research: the section shows keys as "not set", an empty trusted-sites list with your shop's domain first. 2. Open a form: the research panel opens from the header. After R7 and R8 the steps of the superseded panel, settings and publish plans apply: watch steps appear, add your nine sites, see a conflict in the publish review and resolve it.

## Review log

### 2026-11-12 · implement r1 · claude (pipeline-implementer, Opus 5.5)
**Built.** U1: the research panel rebuilt from the session's stored steps (verbs per kind, read steps link their source,
`seq` order; live indicator while `running` and re-read on `research_changed`; `Finished · n fields filled · n to
review` with the cost-limit note for `stopped_at_cap`; failed stage + error + Run again; the reason sentence; Research /
Run again when `can_run`; focus dims steps whose `field_keys` lack the field; Accept all unchanged). The form header
shows `n to review` (hidden at 0) and opens the panel. Before R7 every `research_*` command fails, so the panel shows
no state line and no button (as planned). U2: Settings → Research (`ResearchSection.tsx`): keys (password inputs,
never read back, "set"/"not set"), trusted sites (shop domain first, "always trusted", no remove; up/down and remove;
add with the refusal shown as is), one row per stage, caps, prices table, language, Automatic research, quiet period,
record days, attribution keys with a warning under keys the shop lacks. U3: popover withheld line; per-item conflict
block (`ConflictList.tsx`) with Keep previous / Use suggestion / Edit and "This item waits until you resolve n
conflict(s)"; confirm passes the shown set and a text carrying the shown lines (N8); `suggestions_changed` (by code)
shows the sentence and rebuilds; StatusPill `Waiting for you · n conflicts` inside the activity popover (Cancel stays
there, S4); the form marks conflicts and offers the three actions in place of Looks right / Clear (P1 kept for
plain suggestions); Edit commits once on Enter / blur / Save & close, never on the debounce (N1).

**Deviations (Rust; the closing gate was the full one).**
1. `commands/settings.rs`: `SettingsDto` and `SettingsPatchDto` gain `research: ResearchSettingsDto` (read with every
   field filled by `ResearchSettings::from_slot`; a patch replaces the block). Without it the section had no data
   source and no save path. Bindings regenerated (additions only). **Saving still refuses until R7·U1**: core
   `SettingsService::set` does not accept the `research` key yet; R7·U1 owns accepting it behind
   `ResearchApi::validate_settings` + `settings_changed` (its task 4). Fold into R7·U1: add `"research"` to that list.
2. `commands/attributes.rs`: `attribute_values_set`, `attribute_ai_clear`, `attribute_ai_accept`,
   `attribute_conflict_resolve` read the item's held publish item before the action and, if it is gone after,
   emit `SessionChanged` and `PublishProgress {step: prepare}` through the app's event hub (R5 review carried
   note). Test `r6_command_tests::r6_a20_release_emits_session_changed_and_publish_progress` in `lib.rs`. Undo/redo
   (history commands) can also release and still emit nothing; left for a note.
3. Perimeter additions in TS: `WorkspacePage.tsx`, `BatchQueue.tsx` pass `sessionId` to the form (the item carries no
   session id); `MetafieldInput.tsx` forwards the conflict actions; `p15_publish_ui.test.tsx` and
   `p32_reframe.test.tsx` mocks gained the new publish-service text helpers (p15 C5(a) now previews sessions 1 and 2,
   as carried).

**Judgment calls.** "n to review" in the header is computed from the form's fields (live `ai` + `suggested` in the
layout, every confidence: the MC-A10 definition), so it works before R7 and updates at once after Looks right;
the panel's finished line uses the status counts. A finished run with 0 written fields reads "no fields changed".
"Waiting for Capture One" is held back until the latest photo's capture time + 120 s (the default source wait; the
UI cannot see other session changes). The form tells a conflict by the MC-A1 rule with a TS approximation of the
name key (NFKC, lower case, punctuation/symbols/spaces collapsed). The popover's Edit is an inline text field; the
form's Edit focuses the field itself. The "undo after release" case is wording only: the form's conflict popover
says a publish that already went ahead keeps the value chosen for it. Item id for a popover resolution comes from
`getSessionView(session).session.item_id` (the preview carries only session ids).

**Gate.** `cargo test --workspace`: 861/862, the one failure `p68…c2a_easy_password_generated_and_shown_again`
("receiver running", camera receiver start under load) passed on re-run; clippy and fmt clean; typecheck clean;
`npm test` 486/487, the failure was the p32 publish-service mock (fixed; file re-run green).

