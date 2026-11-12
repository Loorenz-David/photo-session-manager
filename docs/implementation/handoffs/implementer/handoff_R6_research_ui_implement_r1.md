---
plan: R6_research_ui
role: implement
round: 1
state: DONE
date: 2026-11-12
actor: claude (pipeline-implementer, Opus 5.5)
---
# R6_research_ui — implement round 1

## What was built
- **Research panel (U1).** In a product form, the Research panel now shows the session's research as steps: Looked,
  Searched, Read (with a link to the page), Compared, Concluded, Not determined, in order. While research runs, a
  "Researching…" indicator shows and new steps appear on their own. When it ends: "Finished · 3 fields filled · 2 to
  review" (plus "stopped at the cost limit" when it hit the limit; "no fields changed" when a re-run only confirmed
  values). A failed run shows the stage and the error with **Run again**. When research cannot run, the panel says why
  ("Day limit reached", "Waiting for Capture One", …), and **Research** starts a run when one is allowed. Opening a
  field's AI popover and choosing See research dims the steps that did not lead to that field. The form header shows
  "n to review" and opens the panel. Until R7 wires the engine, the panel shows no state line and no button.
- **Settings → Research (U2).** Keys (typed, never shown again, "set"/"not set"), trusted sites (your shop first and
  fixed, add, remove, move up/down, refusals shown as written), model/effort/image size per stage, cost limits, price
  rows, language, Automatic research, quiet period, record days, attribution keys with a warning under a key your shop
  does not have.
- **Publish review and conflicts (U3).** The publish review says "n low-confidence AI suggestions will not be
  published" under the item concerned, and lists each conflict (field, previous value and where it came from, the
  suggestion and its confidence) with **Keep previous / Use suggestion / Edit** and "This item waits until you resolve n
  conflicts". Publish stays available. The confirmation carries exactly what was shown. If suggestions change while
  confirming, the review says so and rebuilds. A waiting session row reads "Waiting for you · n conflicts". In the
  form, a field in conflict is marked and its popover offers the same three actions. Edit saves once, when you press
  Enter, leave the field or Save & close. Resolving the last conflict updates the session row without a refresh.

## Tests → criterion rows
| Row | Test | Shape |
|---|---|---|
| A18·C2(a) | `form/__tests__/r6_research_panel.test.tsx` `a18_c2a_steps_in_seq_order_with_verbs_and_read_link` | exact order of kinds and verbs; link href |
| A18·C2(b) | `a18_c2b_running_run_appends_the_new_step_on_research_changed` | via FormDialog + event handler; 1 → 2 steps, live indicator |
| A18·C2(c) | `a18_c2c_finished_line_counts_filled_and_to_review` | exact line; also the 0-filled wording and the capped line (carried R4 note) |
| A18·C2(d) | `a18_c2d_failed_run_shows_stage_error_and_run_again` | exact text; Run again button |
| A18·C3(a) | `a18_c3a_research_button_starts_a_run_for_the_session` | `runResearch(4)` once |
| A18·C3(b) | `a18_c3b_reason_sentences_and_no_button_at_the_day_limit` | both sentences; no button; plus "Waiting for Capture One" held until the wait ends (carried R4 note) |
| A18·C4(a) | `a18_c4a_focus_from_a_field_popover_dims_other_steps` | dimmed flags `[true,false,true]` → all false |
| A18·C5(a) | `a18_c5a_header_to_review_opens_the_panel_and_hides_at_zero` | "2 to review", opens panel, gone at 0 |
| A19·C5(a) | `settings/__tests__/r6_research_settings.test.tsx` `a19_c5a_…` | order `[shop, pamono.com, 1stdibs.com]`; remove controls |
| A19·C5(b) | `a19_c5b_refused_add_shows_the_refusal_and_keeps_the_list` | refusal text; list unchanged |
| A19·C5(c) | `a19_c5c_keys_are_never_shown_and_read_set_after_saving` | empty password inputs; "set" after save; one call with exactly the typed key |
| A19·C5(d) | `a19_c5d_warning_only_under_an_attribution_key_the_shop_does_not_have` | warning under `custom.maker` only |
| A20·C1(a) | `publish/__tests__/r6_research_publish.test.tsx` `a20_c1a_…` | exact line once, inside session 1's block |
| A20·C2(a) | `a20_c2a_…` | label, prior, origin, suggestion, confidence, 3 buttons, waits line; Confirm enabled |
| A20·C2(b) | `a20_c2b_prior_from_the_shop_reads_from_shopify` | "from Shopify" |
| A20·C3(a) | `a20_c3a_keep_previous_resolves_and_rereads_the_preview` | `resolveConflict(11,'designer','keep_previous')`; preview read twice; block gone |
| A20·C3(b) | `a20_c3b_confirm_receives_exactly_the_shown_set_and_the_shown_lines` | exact shown set and confirmed text (N8) |
| A20·C3(c) | `a20_c3c_changed_suggestions_refusal_is_shown_and_the_preview_rebuilt` | sentence; preview re-read; popover open; no toast |
| A20·C4(a) | `a20_c4a_waiting_for_you_row_label` | "Waiting for you · 2 conflicts" (and singular) |
| A20·C5(a) | `form/__tests__/r6_conflicts_in_form.test.tsx` `a20_c5a_…` | mark, popover actions + prior/origin, Edit focuses; typing "Walnut" never saves on the debounce; Enter → `resolveConflict(7,'designer','edit','Walnut')` exactly once (N1 row) |
| A20·C5(b) | `a20_c5b_header_counts_plain_suggestions_and_conflicts` | "3 to review" |
| Carried (R5 review): release emits events | `apps/desktop/src-tauri/src/lib.rs` `r6_command_tests::r6_a20_release_emits_session_changed_and_publish_progress` | resolving 1 of 2 → no events; the last → `session_changed`, `publish_progress(prepare)` for the session |

Every test in the new files maps to a row above. No unit tests (no core mechanisms in this plan). No baseline red run
was recorded before the first production edit; the tests were written alongside each unit.

## Assumptions
1. "n to review" in the form header is computed from the form's own fields (live `ai` + `suggested` in the layout, at
   every confidence), which is the MC-A10 definition. It works before R7 and updates at once after Looks right. The
   panel's Finished line uses the status counts.
2. A finished run that wrote 0 fields reads "no fields changed" (R4 review: 0 must not mislead).
3. "Waiting for Capture One" is shown only once the latest photo's capture time + 120 s (the default source wait) has
   passed. The UI cannot see other session changes, such as reclassification.
4. The form tells a conflict by MC-A1 (suggested, with a prior, not the same). "Same" uses a TypeScript approximation
   of the name key: NFKC, lower case, punctuation/symbols/spaces collapsed. Lists compare as sets, anything else by
   its JSON text.
5. Edit in the publish review is an inline text field (Enter or Save commits); Edit in the form focuses the field
   itself. An Edit left unchanged is dropped, not saved.
6. Undo after a release (R5 review) is handled by wording only: the form's conflict popover says a publish that already
   went ahead keeps the value chosen for it, and that the next publish waits on this choice again.
7. The publish review gets the item id through the session view (the preview carries session ids only).
8. Confirmed text = preview summary + the withheld and waiting lines shown. With several sessions, each line is
   prefixed "Session n:".
9. Settings → Research saves the whole `research` block with its own button. Site add/remove/reorder use the site
   commands, which return the new list (they save it from R7·U1 on).

## Deviations
1. **Rust, `apps/desktop/src-tauri/src/commands/settings.rs`.** `SettingsDto`/`SettingsPatchDto` gain a typed
   `research` block (read with every field filled from `ResearchSettings::from_slot`). The section had no other data
   source or save path. Bindings regenerated (96 added lines, nothing removed). **Saving research settings still
   refuses** ("invalid value: research") until R7·U1 makes core `SettingsService::set` accept the `research` key behind
   `ResearchApi::validate_settings` + `settings_changed`. **Fold into R7·U1 task 4:** add `"research"` to the
   accepted keys in `crates/core/src/services/settings.rs`.
2. **Rust, `apps/desktop/src-tauri/src/commands/attributes.rs` + test in `lib.rs`.** This is the carried R5 note,
   which asks the command layer to emit `session_changed` and `publish_progress` after an attribute action that
   released a held item. Covered: values_set, ai_clear, ai_accept, conflict_resolve. Not covered: undo/redo through
   the history commands can also release and still emit nothing (a note for the backlog).
3. **TS files outside the listed perimeter:** `features/workspace/WorkspacePage.tsx` and
   `features/publish/BatchQueue.tsx` now pass `sessionId` to the form, because the item carries no session id.
   `features/form/MetafieldInput.tsx` passes the conflict actions on. In the tests,
   `publish/__tests__/p15_publish_ui.test.tsx` (carried C5(a) fix: preview sessions 1 and 2) and
   `compose/__tests__/p32_reframe.test.tsx` add the new publish-service text helpers to their mocks.
4. Because Rust changed, the closing gate was the full gate, as the plan requires.

## Files changed
U1 (f6ad84b): `apps/desktop/src/features/form/{ResearchPanel.tsx, FormDialog.tsx, __tests__/r6_research_panel.test.tsx}`,
`apps/desktop/src/services/{research.ts, events.ts}`, `apps/desktop/src/features/workspace/WorkspacePage.tsx`,
`apps/desktop/src/features/publish/BatchQueue.tsx`.
U2 (d0a56e2): `apps/desktop/src/features/settings/{ResearchSection.tsx, SettingsPage.tsx,
__tests__/r6_research_settings.test.tsx}`, `apps/desktop/src/services/{research.ts, settings.ts, bindings.ts}`,
`apps/desktop/src-tauri/src/commands/settings.rs`.
U3 (f365188): `apps/desktop/src/features/form/{AiPopover.tsx, FormDialog.tsx, MetafieldInput.tsx,
__tests__/r6_conflicts_in_form.test.tsx}`, `apps/desktop/src/features/publish/{ConflictList.tsx, PublishPopover.tsx,
__tests__/r6_research_publish.test.tsx, __tests__/p15_publish_ui.test.tsx}`,
`apps/desktop/src/features/sessions/StatusPill.tsx`, `apps/desktop/src/services/publish.ts`,
`apps/desktop/src-tauri/src/commands/attributes.rs`, `apps/desktop/src-tauri/src/lib.rs`.
Closing commit: `apps/desktop/src/features/compose/__tests__/p32_reframe.test.tsx`,
`docs/implementation/plans/R6_research_ui.md` (tracker line, Review log), this handoff.
No mutation probes, no architecture-graph delta (none in this project).

## Checkpoint SHA
U1 `f6ad84b` · U2 `d0a56e2` · U3 `f365188` · closing commit: see `git log` (subject "CHECKPOINT (not approved): R6
closing gate").

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` (after `--no-run` build) | 861/862 in the app suite; the failure was `p68_setup_polish_address_password_shop::c2a_easy_password_generated_and_shown_again`, panicking at "receiver running" (the camera receiver had not started under full-suite load; not related to R6). `cargo test -p psm-testkit --test suite p68_setup_polish_address_password_shop::`: 3/3 green. Every other test program green (research engine 122, desktop lib 24, …) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | clean |
| `npm test` | 486/487; the failure was `p32_reframe` C8(b): its publish-service mock lacked the new helpers. Fixed; `npx vitest run src/features/compose/__tests__/p32_reframe.test.tsx` 6/6 |
| Unit commands | U1 form 56/56 · U2 settings 49/49 · U3 publish+form+sessions 110/110, typecheck clean after each |

## Manual try-out
Research runs from R7 on. Before that:
1. Open Settings and scroll to **Research**. The keys read "not set", your shop's domain is first in Trusted sites
   marked "always trusted", and the stages, limits and the other fields show their defaults. Adding a site, saving
   keys or saving the section shows "Research is not available" / "invalid value: research" until R7.
2. Open a session's product form. If a field holds an AI suggestion, the header shows "n to review" and **Research**,
   and both open the panel.
3. From R7 and R8 on: watch the steps appear while research runs, then add your sites in Settings. When a suggestion
   replaces a preset or Shopify value, the publish review shows it as a conflict. Resolve it there or in the form,
   and the session row changes from "Waiting for you" to "Publishing" by itself.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
