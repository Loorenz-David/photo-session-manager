---
plan: R11_research_on_arranged
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-15
actor: opus
---
# Review — R11 Research starts when the session is arranged (review r1)

**Gate:** the tracker showed `IMPLEMENTED`. All seven checkpoints exist in `git log`: U1 `ff2fdec`, U2 `a32391b`, U3 `fea02f4`, U4 `a692083`, U5 `1c09727`, U6 `5a7abe7`, closing `3b93141`. The working tree was clean.

## Verified correct

**Checks (plan "Review mode", run once each, on `3b93141`):**
| Command | Result |
|---|---|
| `cargo test -p psm-research` | green: 142 + 202 + 1 |
| `cargo test -p psm-testkit --test suite research::` | green: 77 |
| `cargo test -p psm-testkit --test suite p03_session_state_list::` | green: 28 |
| `cargo test -p psm-testkit --test suite p13_publish_runs::` | green: 36 |
| `cargo test -p psm-desktop` | green: 25 |
| `npx vitest run src/features/settings src/features/shell src/features/sessions src/features/publish src/features/form` | green: 211 |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `npm run typecheck` | green |
The implementer's full-suite stamp stands (`cargo test --workspace` 1414, `npm test` 509). Nothing contradicts it.

**Perimeter.** I read each unit's `git show --stat`. Every path is in the plan's Files list or is a forced fixture or test edit. The small differences are recorded as a backlog note (`r8_shop_facts.rs` one line in U1, the `ResearchSection.tsx` control removal and the UI fixtures in U4).

**Retired and changed tests (plan table).** I checked them by grep. `a13_c2a`, `a13_c2b`, `a13_c4a/b`, `a13_c5a/b/c`, `a13_c6a/b`, `a15_c7a`, `a19_c4a` and `a19_c5d` are gone, and each has its replacement row (A25·C1(b), C4(a)–(c), C21(a), C28(d)). `a13_c1a/b`, `a13_c7*`, `a13_c8*`, `a14_c3*`, `a14_c5a/b`, `a14_c6a`, `a24_c1a`, `a18_c3a/b` and `r7_u1_two_concurrent…` are present and re-pointed at `arranged`. `a14_c6a` now asserts a start with no quiet period. `a24_c1a` asserts no reason. No deleted test took a row that is still valid: the deleted rows are exactly those superseded by changelog 13.

**U1, the engine trigger.** `ResearchService::arranged` refuses the Unassigned session with the planned sentence. It answers no when any of the five creation-blocking prerequisites fails, when there is no Main photo, or when any run is `queued`. It answers yes when there is no run, or the latest run is `failed`/`cancelled`. Otherwise it compares the latest *started* run's snapshot with now (`arranged_needed`, MC-A7 trigger set). `session_changed` only moves the source-wait clock. `settings_changed` is a no-op. The follow-up path is deleted. `may_start` has no quiet period and keeps the source wait for non-manual triggers. Reuse still applies to every non-manual trigger (`stages.rs:335`), so `arranged` runs reuse units as MC-A7 now says. `cancel_legacy_queued` runs once when the service is built and touches only queued `auto`/`follow_up` runs. `auto` and `quiet_seconds` are gone from `ResearchSettings`, the DTO and the screen. A slot that still carries them loads, because `serde(default)` ignores unknown keys. Tests A25·C1(a)–(j), C2(a)–(c), C3(a)–(b), C4(a)–(d), C5(a), C6(a) and C7(a) observe the answer, the run rows and the status at `arranged`/`tick`/`status`, and would fail on a wrong outcome. (C7(a) is partial: see notes.)

**U2, the settings.** The excluded-keys check sits in `field_class` (first step, treated as never-written) and in `prerequisite_input` (the "writable field" count). The run's shop hosts are reported hosts plus `parse_entry` of each extra address, without repeats, in both the snapshot (`ResearchConfig::from_settings`) and the trusted-site refusal (`shop_hosts`). `validate` refuses a bad entry or one that overlaps a trusted site, and names it. `settings_reset` keeps the six named values and defaults the rest, writing nothing itself. C8(a)–(d), C9(a)–(e), C10(a)–(b) and C11(a) observe the outcome at their boundaries. C8(c) and C10(b) go through `arranged`/`status`.

**U3, derived condition and the look-alike rule.** `derive` follows the contract's six steps in order. It takes the worst among levels that have a choice, maps through MC-A2's `map_output`, gives `low` when any level is an `ai` low value and `medium` otherwise, and never `high`. Stage 7 reads the level rows inside its own writing transaction, after this run's condition writes, skipping cleared rows. A level field in `excluded_keys` still counts. The write goes through `writes::suggest_in` (MC-A1), and the `concluded`/`not_determined` steps name the key. `detailed_condition` and `excluded_keys` join the stage 7 identity. `decide_attribution_without_shop` drops site-class `shop` items only for the attribution class. Measured and visible fields still call the unfiltered function (C15(f), (g)). The read step gets the "another product of your shop" detail when a shop page states an attribution key. The C12 unit tests, C13 and C15 on the real functions, and C14 and C16 as full runs discriminate. For example, C15(a)/(c) would read `high`/`medium` without the drop.

**U4, the app seams.** `sessions.arranged_at` is added by `ensure` (no numbered migration). `arrange_session_once` is one guarded `UPDATE … WHERE arranged_at IS NULL AND item_id IS NOT NULL`: no `change_seq` bump, no undo step. `SessionsService::arrange` emits `SessionChanged` only when it newly sets the mark. `next_unarranged` skips the open session, marked, published and cleared sessions, sessions with no non-deleted photo, and the Unassigned session (left out by `session_list_records`). It picks the nearest after the open one by `(created_at, id)` and otherwise wraps to the oldest. `psm_integrations::arranged` only composes services (H8). An engine refusal or error is logged and never fails the mark (HA4). `publish_confirm` is unchanged. Confirm is never disabled. C17(a)–(d), C18(a)–(d), C19(a), C20(a)–(e) and C21(a) observe stored state, returned values, events and runs. The strengthened C20(e) now proves that the run's value lands after confirmation and stays out of the frozen item.

**U5 and U6, the UI.** The Header button sits in the non-Unassigned branch, is hidden for archived sessions, calls the command once, routes or toasts, and keeps the route on failure. There is a list tag. The publish line is shown under its item, the popover reloads on `research_changed`, and Confirm stays enabled. The panel hint appears only with no run, no reason and `can_run`. `ShopFieldPicker` is filterable and scrolling, lists unknown chosen keys with the warning, and says "Connect a shop to choose fields" when there is no shop. Designer and Manufacturer are preselected only when the saved list is empty, and "suggested — not saved yet" clears on Save. The extra-address list ignores repeats and shows a save refusal unchanged. The detailed-condition table has rows from condition-group choice fields in their own order and saves in row order, leaving out unset rows. The reset dialog names the six kept things, and Cancel calls nothing. The C24–C33 tests observe what is shown and what the save or command received. A probe confirmed that C24(d)'s archived half fails on an unarchived input.

**Assumptions (handoff).** None contradicts the intention. U1 compares against the latest started run's snapshot, as the plan says. U4 emits `SessionChanged` only on a new mark, takes `researching` from the run state, and saves the reset through `save_research`. U6 defines "shop fields" as integration ≠ `core`, builds level rows from the condition-group choice fields, clears the table when the field changes, and ignores a repeated address. The owner's card (table A) is answered. R12 writes it as the default, and R11 ships the table empty, as planned.

**Hard constraints H1–H8.** H1: preview-time marking writes only local research suggestions, and publishing still needs Confirm. H2/H3: no file or pixel work. H4: `ensure` and the mark are idempotent, and the legacy cancel is too. H5: n/a. H6: arranging works with no integration (C19(a)). H7: `arranged_at` is a generic session fact, and the UI names no adapter. H8: commands stay thin, the composition lives in `crates/integrations`, and the "next" rule lives in core. Secrets and live services: every test uses fakes.

## Findings

### F1: blocking (B2) · route: production
**What is wrong.** Row A25·C23(a) requires that "both commands reach the wired engine in the desktop crate's command test". `a25_c23a_new_commands_reach_wired_app_and_bindings` (`apps/desktop/src-tauri/src/lib.rs`) calls `session_arrange_next` with the price table emptied and only an unreviewed photo. It asserts only `next_session_id == null` and that `arranged_at` is set, never `research_queued`. The engine is asked and refuses (`PriceMissing`, no Main photo), and nothing observes it. If the command marked the session through core and never called `ResearchApi::arranged`, the test would still pass. That is the exact defect family charter rule 10 names (a command surface that does not reach the engine). The production code is correct today by reading (`session_arrange_next` → `psm_integrations::arranged::arrange_and_next`). The proof is missing. The implementer flagged this in the handoff.
**Authority.** Plan R11·U4 row A25·C23(a) (trace: charter rule 10); flow.md §5 B2.
**Correction.** In that test, call `session_arrange_next` *before* emptying the prices, on a session whose photo is classified Main (e.g. `a.review().classify(id, Classification::Main)`). Assert that the response's `research_queued` is `true` and that the session has one `queued` run with trigger `arranged`. Then empty the prices and keep the reset half as it is.
**Files the fix may touch.** `apps/desktop/src-tauri/src/lib.rs` (the `a25_c23a_…` test only).

No other blocking finding.

## Notes written to backlog
To `docs/implementation/backlog_research/R11_research_on_arranged.md` (the plan's Review mode names this file):
1. A25·C22(a)'s status clause can never fail (it compares against `"price_missing"`, but the code is `"PriceMissing"`). A probe with the right string shows the wired test app *does* report `PriceMissing` after a reset, because the fakes' provider ids are not in the default price table. The real app is fine, and A25·C10(b) proves it at the engine. This is a plan lesson as well as a test cleanup.
2. The Unassigned half of A25·C24(d) checks before the list loads. The code is safe by its `sessionId === 1` branch.
3. A25·C33(a)'s "saved value" half asserts the fixture.
4. A25·C7(a) does not check "every other field is its default".
5. A25·C9(c) sets the shop host directly, and C9(a) covers the setting path.
6. Judgment call: stage 7 no longer stops at the cap when only the derived field remains. Should 08B absorb this?
7. Publishing marks first, before the preview can fail.
8. Small differences between the declared file lists and the commits (all forced).

## Lessons for the plans
- A row that asks the wired app's `status` for "no PriceMissing" after a reset has to say which provider ids the price table is checked against. With the testkit fakes, the default table never covers them (note 1).
- A command-reach row (charter rule 10) should name the observable that proves the engine was reached, e.g. "the response says research was queued and a run row exists", not just "reaches the wired engine".

## Probe declaration
No production file was touched. Two test-only probes, both reverted:
- `crates/testkit/tests/suite/research/r11_arranged.rs`: the C22(a) assertion was set to `"PriceMissing"` (plus one `eprintln!`) and the test run. It failed with "No price is set for fake-model gpt-6.1-sol". The file was restored from a copy and verified byte-identical with `shasum -c`.
- `apps/desktop/src/features/shell/__tests__/zz_probe_c24d.test.tsx`: a temporary copy of the C24(d) steps with an unarchived input. It failed as expected and was deleted.
No database or settings state outside the tests' temporary folders was touched.

## Files written by this review
`docs/implementation/handoffs/reviewer/handoff_R11_research_on_arranged_review_r1.md` (this file) · `docs/implementation/backlog_research/R11_research_on_arranged.md` (new) · `docs/implementation/plans/R11_research_on_arranged.md` (Tracker line and the Review log entry only). No narrative (written at approval). No commits.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. Note 6 (derived grade at the cap) is a recommendation for the orchestrator to fold into 08B. It does not wait on the owner.
