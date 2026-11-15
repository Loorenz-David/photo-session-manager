---
plan: R11_research_on_arranged
role: implement
round: 1
state: DONE
date: 2026-11-15
actor: codex (U1–U5) + claude-opus (U6, closing gate; resumed session)
---
# Handoff — R11 Research starts when the session is arranged (implement r1)

## What was built
- **Research starts when you say a session is arranged.** The open session gets an **Arranged — next** button
  (it reads **Arranged ✓ — next** once marked). Pressing it marks the session, queues its research run, and opens
  the next unarranged session (nearest created after this one, else the oldest; captures then go there). When every
  session is arranged it goes to the sessions list with "Every session is arranged". Marked sessions carry an
  **Arranged** tag in the list.
- **Photos arriving no longer start research.** The quiet period and the "Automatic research" switch are gone.
  Pressing Arranged again starts a new run only if the photos changed since the last run, or the last run
  failed or was cancelled. The form's research panel says "Research starts when you press Arranged — next."
  before any run. Runs queued by an older build are cancelled when the app starts.
- **Publishing marks first.** Opening the publish popover marks every unmarked session in it and queues its run.
  The item shows "Research is still running for this item. Its suggestions are not in this publish unless you
  wait." and refreshes when research ends. Confirm is never disabled.
- **Settings → Research:** a filterable picker of your shop's fields for **Designer and maker** (fields labelled
  Designer / Manufacturer are pre-ticked as "suggested — not saved yet" when none are saved) and for **Fields
  research leaves alone**; **Extra shop addresses**; a **Detailed condition** table (choose the field, then one
  grade per condition level, or "counts for nothing"); **Reset research settings to defaults**, which keeps
  trusted sites, the field choices, addresses, the table and language.
- **Engine rules:** fields left alone are never asked for, written or charged. Detailed condition is derived from
  the condition levels by the table (never read from photos), at `medium`, or `low` when a level is a low AI guess.
  A page of your own shop is never evidence for designer or maker. Width and visible fields still use it.

## Tests → criterion rows
99 rows → 99 tests, one per row, each named after its row (`a25_c<n><row>_…`). No test outside the map.
| Rows | Test file | Boundary |
|---|---|---|
| A25·C1(a)–(j), C2(a)–(c), C3(a)–(b), C4(a)–(d), C5(a), C6(a), C7(a) | `crates/research/tests/engine/arranged.rs` | `ResearchApi::arranged`, `tick`, `status`, service build, `from_slot` |
| A25·C8(a), C13(a)–(c), C15(a)–(g) | `crates/research/src/rules/confidence.rs` (unit, core mechanisms `field_class`, `decide_attribution`) | the rule |
| A25·C8(b)–(d), C9(a)–(e), C10(a)–(b), C11(a) | `crates/research/tests/engine/settings_r11.rs` | full runs, `validate_settings`, `site_add`, `settings_reset`, `arranged`/`status` |
| A25·C12(a)–(k) | `crates/research/src/rules/derived_condition.rs` (unit, core mechanism `derive`) | the rule |
| A25·C14(a)–(d), C16(a) | `crates/research/tests/engine/derived.rs` | full runs |
| A25·C17(a)–(d), C18(a)–(d), C19(a), C20(a)–(e), C21(a), C22(a) | `crates/testkit/tests/suite/research/r11_arranged.rs` | `psm_integrations::arranged`, `SessionsService`, `PublishService::confirm`, store, wired app |
| A25·C23(a) | `apps/desktop/src-tauri/src/lib.rs` `a25_c23a_new_commands_reach_wired_app_and_bindings` | desktop commands + `bindings.ts` |
| A25·C24(a)–(e) | `apps/desktop/src/features/shell/__tests__/r11_arranged_button.test.tsx` | Header |
| A25·C25(a) | `apps/desktop/src/features/sessions/__tests__/r11_arranged_tag.test.tsx` | sessions list row |
| A25·C26(a)–(b) | `apps/desktop/src/features/publish/__tests__/r11_publish_research.test.tsx` | publish popover |
| A25·C27(a) | `apps/desktop/src/features/form/__tests__/r11_panel_hint.test.tsx` | research panel |
| A25·C28(a)–(e), C29(a), C30(a)–(b), C31(a)–(b), C32(a)–(b), C33(a) | `apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx` | `ResearchSection` |

Assertion shape: each test asserts the row's exact outcome. Known weaker spot, reported and not changed:
**A25·C23(a)** proves `session_arrange_next` marks the session through the command, but runs with prices emptied
and an unreviewed photo, so it does not show that the command queued a run (the reset half does reach the
engine). Left as committed under the resume note's "keep U1–U5 unless the gate shows a defect".

Plan table "Tests that pin what this plan retires or changes": verified by grep. Deleted: `a13_c2a`, `a13_c2b`,
`a13_c4a/b`, `a13_c5a/b/c`, `a13_c6a/b`, `a15_c7a`, `a19_c4a`, `a19_c5d`. Kept or amended: `a13_c1a/b`, `a13_c7a/b`,
`a14_c3a`, `a14_c5a/b`, `a14_c6a`, `a24_c1a`, `a18_c3a`. The harness and testkit settings no longer carry
`auto`/`quiet_seconds`.

## Assumptions
- U1: `arranged_needed` compares the latest started run's input snapshot with the snapshot of now. A leftover
  `auto`/`follow_up` queued run is cancelled silently, with a warning only if the cancel fails.
- U4: `SessionChanged` is emitted only when the mark is newly set. `researching` = the session's latest run is
  `queued` or `running` (from `ResearchApi::status`). The reset is saved through the existing `save_research` path
  (validation, then `settings_changed`). The "next" rule is the plan's: nearest unmarked eligible session created
  after the open one, else the oldest. The Unassigned session is never next.
- U6: "the shop's fields" = every definition whose integration is not `core` (built-in Product fields left out,
  no adapter name in UI code). The picker has a filter box and a scrolling list. With no shop (or no shop fields) it
  says "Connect a shop to choose fields" and still lists already-chosen keys so they can be removed. The detailed
  field select offers the shop's `choice` fields. Level rows = choices of the shop's `condition`-group choice fields
  in their own order (the chosen field excluded). Changing the field clears the table. A repeated extra address is
  ignored. "suggested — not saved yet" shows while the suggested field stays ticked and clears on Save.
- The owner's card (detailed-condition table) was answered on 2026-11-15 with A (08B changelog 15). R12 writes it
  as the default. R11 ships the table empty, as planned, so the owner fills it in Settings for now.

## Deviations
- The closing gate found three defects in committed U4/U5 work, fixed in the closing checkpoint:
  1. clippy `collapsible_if` in `crates/core/src/services/sessions.rs` (same behaviour);
  2. `p32_reframe.test.tsx` failed because the compose test harness mock of `@/services/events` lacked
     `onResearchChanged`, which the publish popover now subscribes to. Added to
     `apps/desktop/src/features/compose/__tests__/harness.tsx` (outside U5's listed files; a mock forced by the
     new subscription);
  3. clippy `unnecessary_get_then_check` in `r11_arranged.rs`.
- Strengthened while on that file: **A25·C20(e)** could not fail before (its run was held at its first model call
  with nothing scripted, so nothing could ever be frozen). It now scripts a sure designer read and holds only the
  first model call. After release it asserts the run did write the value and the frozen publish item does not hold it.
  (Fake hooks cleared during a call are put back by `fakes.rs::arrive`, so a hook that held every call deadlocked;
  worth knowing for later tests.) **A25·C18(d)** now gives the Unassigned session a photo, so its exclusion is
  exercised.
- U6 deleted `a19_c5d` and its unused helper from `r6_research_settings.test.tsx`, per the plan's table.

## Files changed (whole milestone; this session's part marked ★)
- U1 `ff2fdec`: `crates/research/src/{domain.rs, config.rs, runs.rs, rules/run_rules.rs, api.rs, api_impl.rs, stages.rs, service.rs, store.rs}`, `crates/research/tests/engine/{harness.rs, lifecycle.rs, resume.rs, api.rs, defaults.rs, main.rs, arranged.rs}`, `crates/testkit/src/test_app.rs`, `crates/testkit/tests/suite/research/{r7_wiring.rs, r7_export_schedule.rs}` (see `git show --stat ff2fdec`)
- U2 `a32391b`, U3 `fea02f4`: `crates/research/src/**` and `crates/research/tests/engine/**` per the plan (see `git show --stat`)
- U4 `a692083`: 34 files (core session/store/services, infra ensure/store, `crates/integrations/src/arranged.rs`, desktop commands, `bindings.ts`, services, test fixtures listing session columns, `r11_arranged.rs`)
- U5 `1c09727`: Header, SessionRow, PublishPopover, ResearchPanel, services, four `r11_*` UI tests, `r6_research_panel` fixture
- ★ U6 `5a7abe7`: `apps/desktop/src/features/settings/{ShopFieldPicker.tsx (new), ResearchSection.tsx}`, `apps/desktop/src/features/settings/__tests__/{r11_research_pickers.test.tsx (new), r6_research_settings.test.tsx}`
- ★ Closing `3b93141`: `crates/core/src/services/sessions.rs`, `crates/testkit/tests/suite/research/r11_arranged.rs`, `apps/desktop/src/features/compose/__tests__/harness.tsx`, `docs/implementation/plans/R11_research_on_arranged.md` (tracker line, Review log)
- ★ This handoff (left for the runner to commit).
- No archgraph in this project, so no graph delta. No mutation probes (flow.md §4).

## Checkpoint SHA
U1 `ff2fdec` · U2 `a32391b` · U3 `fea02f4` · U4 `a692083` · U5 `1c09727` · U6 `5a7abe7` · closing `3b93141` (clean tree after it, except this handoff).

## Stamp
Closing gate, run once in order on the U6 tree (`5a7abe7`):
| Command | Result |
|---|---|
| `cargo test --workspace` | green: 1414 passed, 0 failed (3 min 8 s) |
| `cargo clippy --workspace --all-targets -- -D warnings` | red: 1 lint (U4 `sessions.rs`); after the fix, a second lint (U4 test `r11_arranged.rs`); both fixed, then green |
| `cargo fmt --all -- --check` | green |
| `cd apps/desktop && npm run typecheck` | green |
| `npm test` | 508 passed, 1 failed (`p32_reframe` C8(b), missing mock export); fixed |
| `cargo test -p psm-research` | green: 142 + 202 + 1 passed |
Re-runs after the fixes, failing files only: `npx vitest run src/features/compose/__tests__/p32_reframe.test.tsx` 6/6 ·
`cargo test -p psm-testkit --test suite research::r11_arranged::` 16/16 · workspace clippy green · fmt green.
U6 unit commands: `npx vitest run src/features/settings` 15 files / 62 tests green · `npm run typecheck` green.
No pre-existing failures.

## Manual try-out
On scratch data (`PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev` in `apps/desktop`), shop connected, keys entered:
1. Settings → Research. If this folder was used before R10, press **Reset research settings to defaults** and
   confirm. The models and prices appear, and your trusted sites are still there.
2. Under **Designer and maker**, Designer and Manufacturer are ticked as "suggested — not saved yet". Under
   **Fields research leaves alone**, tick total width, depth, height, seat height, weight and damage details. Add
   your other addresses under **Extra shop addresses**. Under **Detailed condition** choose the field and set
   None → Very Good, Minor → Very Good, Moderate → Good, Major → Worn, N/A → counts for nothing. Press **Save
   research settings**.
3. Create two sessions and shoot a few photos in each. Nothing is researched while photos arrive. The form's
   research panel says "Research starts when you press Arranged — next."
4. In the first session sort the photos, mark the condition photos, and press **Arranged — next**. The second
   session opens (captures now go to it). The first session's form shows research steps. Nothing is suggested for
   the fields left alone. Detailed condition follows the worst level.
5. Change a photo in the first session: nothing starts. Press **Arranged — next** there again: a new run starts.
6. In the second session press **Publish** without arranging it. The popover says research is still running. Wait
   for the line to go, or confirm at once to publish without it.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The plan's one card (detailed-condition table) was answered by the owner on 2026-11-15 (A; 08B changelog 15). R12 makes it the default.
