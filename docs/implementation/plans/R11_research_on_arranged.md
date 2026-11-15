---
phase: R11_research_on_arranged
title: Research starts when the session is arranged — the Arranged, next button marks the open session, starts its research run and opens the next session; photos arriving no longer start runs; Settings → Research gets pickers for the designer and maker fields, the fields research leaves alone and extra shop addresses, a detailed-condition table and a reset to defaults; a look-alike piece in your own shop no longer names a designer or maker
difficulty: high
projection: no
depends_on: [R10_research_defaults]
blocked_on: none (one owner card, which the plan does not wait for: the detailed-condition table is a setting)
---
# R11 — Research starts when the session is arranged

**Tracker:** NOT_STARTED · 2026-11-15 · planner · plan written (33 criteria, 99 rows, 6 units); one owner card on the detailed-condition table, not blocking

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
Implement the owner's decisions of 08B changelog 12 (a), (b), the look-alike finding, and changelog 13. (U1) The engine starts a run when it is told a session is arranged, and never because a photo arrived; the quiet-period trigger and its two settings are retired. (U2) Three new settings in the engine: fields research leaves alone, extra shop addresses, and a reset of the settings that keeps the person's own lists. (U3) Detailed condition is derived from the condition levels by a rule that takes its table from settings; a page of the shop that is another product is never evidence for designer or maker. (U4) The session mark, the "Arranged — next" command and "publishing marks first", in the app. (U5) The button, the list tag, the publish line and the panel hint. (U6) The Settings → Research pickers and the reset button. **Not here:** photo choice by shoot template, the local page parser, dropping non-matches before compare, the second measurement (all R12); a shoot-template concept; any change to the publish hold (MC-A10).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 6 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids of this plan are `A25·C<n>(<row>)` (the next free plan number in `docs/implementation/research_row_mapping.md`); criteria are numbered once across the units. Name each test after its row (`a25_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R11_research_on_arranged.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The arranged trigger in the engine; the quiet-period trigger retired | 7 | 22 | tests only (three testkit files) |
| U2 | Fields research leaves alone, extra shop addresses, reset to defaults | 4 | 12 | no |
| U3 | Detailed condition derived; no designer or maker from another product of the shop | 5 | 26 | no |
| U4 | The session mark, "Arranged — next", publishing marks first (app) | 7 | 17 | **yes** |
| U5 | The button, the list tag, the publish line, the panel hint (UI) | 4 | 9 | **yes** (TypeScript) |
| U6 | Settings → Research: the pickers, the detailed-condition table, reset (UI) | 6 | 13 | **yes** (TypeScript) |
| **Total** | | **33** | **99** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all; §6.8 holds the names this plan adds; §9 v9 and v10) · book `01`, `14` §Research agent, `08A`, `08B` (**changelog 12 and 13 are the owner's decisions this plan implements; they win over older text in 08A and 08B**) · evidence `docs/intention/evidence/research_providers.md` ("Owner answers", "Acceptance measurement") · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`.

## What this plan supersedes in the book (the orchestrator folds these into 08A and 08B; until then this section is the rule)
| Book text | Replaced by |
|---|---|
| 08A §5 "Start" and "Quiet period"; 08B MC-A6 event table rows "Session reaches ≥ 1 Main photo …" and "Photo-derived input change …", the paragraph "Quiet period", and the row "`research.auto` switched off (R-IA25)"; MC-A6's eighth prerequisite ("`research.auto` off") | U1: runs are created only by "Arranged" (trigger `arranged`) and by Run again (`manual`). A photo change creates and cancels nothing. There is no quiet period and no `auto` setting. Authority: changelog 13 (a), (b), (d), (e). |
| 08A §5 "New photos after a run (progressive)"; 08B MC-A7 "Follow-up trigger set" (when a follow-up is queued) | The same comparison (MC-A7's trigger set against the latest started run's input snapshot) now decides whether an "Arranged" press creates a run. It is never evaluated on a photo change. Changelog 13 (e). |
| 08B MC-A7 "Reuse": "In an `auto` or `follow_up` run" | also in an `arranged` run. A `manual` run still reuses nothing. |
| 08B MC-A6 invariant MA13, clause "No `auto` or `follow_up` run starts within the quiet period" | dropped with the quiet period; the other two clauses stand. |
| 08B changelog 12 (c) "the condition step waits until the person … leaves the item or publishes it" | covered by changelog 13: the whole run starts at "Arranged". No separate deferral of stage 6 is built. |
| 08A §6 table, row `high`: "**or** a `match` (stage 4) with a listing on the owner's own shop"; 08B MC-A4 `high(v)` "or a `match` listing item on site `shop`", `medium(v)` and "Support" where they count the `shop` identity, enumerated rows 2 and 3 — **for attribution fields only** | U3: for an attribution field a listing item on site `shop` counts at no level. Measured and visible fields keep MC-A4 as written. Authority: changelog 12 (measurement pieces 6 and 33). |
| 08B MC-A4 "Field class" (five classes) | U2 and U3: seven steps, first match wins: never-written (MC-A1) · left alone (`excluded_keys`), treated exactly as never-written · derived (`detailed_condition.key`) · condition · attribution · measured · visible. Changelog 12 (a), (b). |
| 08A §6 "Other (visible) fields", for the detailed-condition field once the setting names it | derived from the condition levels, never read from a photo or a listing. Changelog 12 (b). |
| 08B MC-A3 "Shop hosts" (reported by the commerce port only) | plus the setting `extra_shop_hosts` (changelog 11 (c) said a follow-up would add the other addresses; `read_markets` stays declined). |

## Points recorded by planning (decided here; each is one sentence in the narrative's "Choices made for you")
- **Where the mark lives.** One nullable column, `sessions.arranged_at` (UTC ms of the first press), added by the idempotent step `psm_infra::sqlite::ensure` and never by a numbered migration (`master_plan_research.md` §0, §6.4: this track does not edit `SqliteStore::migrations()`). The mark is not an undo step, does not bump `change_seq`, and is never removed.
- **Which session is "next".** Among item sessions that are not marked, not published, not archived and hold at least one photo that is not deleted: the nearest one created after the open session; when there is none, the oldest one. So the person walks the day's sessions in shooting order from wherever they start, and the walk wraps. No artifact states an order; this is the smallest rule that visits every session once.
- **Opening the next session routes captures to it.** `App.tsx` activates whichever session is open (existing behaviour of opening any session). The button changes nothing about that.
- **Publishing marks first, at the moment the publish popover asks for its preview.** Every unmarked session in the selection is marked and its research run is queued; the popover says research is running and re-reads itself when it ends. **Confirm is never disabled** (08B MC-A10, chapter 08 "the form never blocks publishing", HA4): a person who confirms while research is running publishes without that run's suggestions (08A §7, values are frozen at confirmation). Research before publish is therefore reached by waiting in the popover, by the person's choice.
- **`auto` and `quiet_seconds` are removed, not kept inert.** They leave `ResearchSettings`, the Settings screen and the command DTO. A saved slot that still carries them loads without error and the two values are ignored and not written back. `RunTrigger::{Auto, FollowUp}` stay as names so runs stored by an earlier build still read; no code creates them.
- **Reset keeps the person's lists.** "Reset research settings to defaults" replaces models, efforts, image sizes, prices, the two cost limits, the page limit, waits, timeouts, record days and provider ids, and keeps trusted sites, designer and maker fields, fields left alone, extra shop addresses, the detailed-condition table and the language.
- **This item's own listing stays unread (R-IA9).** The scope sentence "an own-shop listing counts as evidence only when it is this item's own listing" meets 08B MC-A3, which excludes that page from reading and from evidence. Both hold together as: for designer and maker, no page of the shop is evidence. Nothing observable is lost: the item's own listing can only state what the form already holds from the shop.
- **There is no field picker to copy.** The scope names "the existing condition-photos field picker"; in the code that setting is a text box (`SettingsPage.tsx`, "Condition photos key"). U6 builds one picker component over `listDefinitions()` and uses it three times. The condition-photos text box is not changed.
- **`www.` of the primary address is already covered** by the subdomain rule (`rules::sites`, a host matches a shop host or any of its subdomains). A25·C9(b) pins it.

## ⚠ Needs you (1)
**Question** — Which grade should research write in "Detailed condition" for each worst condition level: None → Very Good, Minor → Very Good, Moderate → Good, Major → Worn, and never Excellent?

**Story** — A teak sideboard has Scratch: Minor, Veneer damage: Moderate, the rest None. Today research guesses the grade from the photos and was wrong on 19 of 42 pieces. Your own listings do not settle a rule: with Minor as the worst level you wrote Very Good 17 times and Good 5 times; with Moderate, each twice.

**Branches** — **A**, the table above: agrees with 21 of your 28 graded listings. **B**, stricter (Minor → Good, Moderate → Worn): agrees with 7 of 28. **C**, your own table, entered in Settings → Research.

**Recommendation** — A, because it agrees with most of what you already published.

**On silence** — nothing is derived: the table ships empty and Detailed condition is filled as it is today until you enter one.

**Trace** — R11·U3 rows A25·C12–C14, U6 row A25·C31; 08B changelog 12 (b).

## Tests that pin what this plan retires or changes (found by grep; each is handled in the unit named)
| Test | File | Pins | What happens, where |
|---|---|---|---|
| `a13_c1a_each_prerequisite_alone`, `a13_c1b_the_first_failing_is_named` | `crates/research/tests/engine/lifecycle.rs` | eight prerequisites, `AutoOff` last | U1: amended to seven; rows A13·C1(a), (b) keep their ids |
| `a13_c2b_one_run_while_queued` | same | an `auto` run queued by a photo | U1: deleted; A25·C1(b) replaces it |
| `a13_c4a_auto_off_cancels_queued_runs_and_lets_the_running_one_finish`, `a13_c4b_auto_off_then_run_again` | same | switching `auto` off | U1: deleted (rows A13·C4(a), (b) superseded by changelog 13) |
| `a13_c5a_quiet_period_after_the_last_change`, `a13_c5b_quiet_period_after_launch`, `a13_c5c_manual_skips_the_quiet_period` | same | the quiet period | U1: deleted; A25·C4(a) replaces them |
| `a13_c6a_waits_for_a_missing_source_then_starts_without_it`, `a13_c6b_no_source_at_all_waits_for_capture_one`, `a15_c7a_wait_ends_at_for_a_run_waiting_for_the_editor` | same | readiness of an `auto` run | U1: deleted; A25·C4(b), (c) replace them |
| `a13_c7a_one_run_at_a_time`, `a13_c7b_manual_starts_before_an_earlier_auto`, `a13_c7c_the_start_transaction_snapshots_settings_and_form`, `a13_c8a_day_limit_waits_for_the_next_local_date`, `a13_c8b_offline_holds_the_run_and_retries_within_a_minute` | same | runs started by a photo change | U1: kept, the run is started by `arranged` (same assertions; A13·C7(b) reads "before an earlier `arranged`") |
| `a14_c3a` … `a14_c3e` (follow-up reuse), `a14_c6a_changes_during_a_run_queue_one_follow_up` | `crates/research/tests/engine/resume.rs` | a follow-up queued by a photo change | U1: kept, the second run is started by a second `arranged` and its trigger is `arranged` (rows A14·C3(a)–(e), C6(a) keep their outcomes) |
| `a14_c5a_each_photo_derived_change_queues_one_follow_up`, `a14_c5b_retouch_settings_and_non_cover_swap_queue_nothing` | same | the follow-up trigger set on a photo change | U1: re-pointed at `arranged` (each change makes the next press create one run; a retouch, a settings change and a non-cover swap make it create none) |
| `a19_c4a_switching_automatic_research_off_cancels_the_queued_run` | `crates/research/tests/engine/api.rs` | `auto` off | U1: deleted |
| `a24_c1a_shipped_defaults_start_a_requested_run` | `crates/research/tests/engine/defaults.rs` | reason `AutoOff` with an empty sentence | U1: amended, the status names no reason at all |
| harness `test_settings()` (`"auto": true`, `"quiet_seconds": 1`), `Harness::chair`, `Harness::changed` | `crates/research/tests/engine/harness.rs` | how every engine test gets its first run | U1: the two keys go; the helper that starts a run calls `arranged` |
| `research_test_settings()` (`"auto": true`, `"quiet_seconds": 1`) | `crates/testkit/src/test_app.rs` | how app tests get a run | U1: the two keys go |
| `a13_c2a_first_main_photo_queues_an_auto_run`; the helpers of `a13_c8c`, `a13_c8d`, `r7_u1_two_concurrent_wake_ups_never_run_two` | `crates/testkit/tests/suite/research/r7_wiring.rs` | a photo change queues a run in the wired app | U1: `a13_c2a` deleted (A25·C21(a) replaces it in U4); the helpers start runs through `ResearchApi::arranged` |
| the run-producing helper | `crates/testkit/tests/suite/research/r7_export_schedule.rs` | same | U1: starts runs through `ResearchApi::arranged` |
| the engine-driving tests | `crates/research-tool/tests/tool/u3_report_and_replay.rs` | runs started on the engine harness | U1: follow the harness helper |
| `a19_c5d_warning_only_under_an_attribution_key_the_shop_does_not_have`; the fixture line `auto: true, quiet_seconds: 1` | `apps/desktop/src/features/settings/__tests__/r6_research_settings.test.tsx`, `r7_research_export.test.tsx` | attribution keys as text boxes; the two retired settings | U6: `a19_c5d` is replaced by A25·C28(d); the fixtures lose the two keys |
| `a18_c3a`, `a18_c3b` and the fixture `trigger: 'auto'` | `apps/desktop/src/features/form/__tests__/r6_research_panel.test.tsx` | the Research button and the reason sentences | U5: kept; a reason code for `auto` off no longer exists, the fixture's trigger becomes `arranged` |
No test pins a default value this plan changes: the three new settings are new, and `attribution_keys` stays empty (A24·C2(b) stands).

## Unit 1 — The arranged trigger in the engine

**Unit id:** `R11·U1` · **Rows:** A25·C1–C7 · **Changes app files:** tests only (`crates/testkit/src/test_app.rs`, `crates/testkit/tests/suite/research/{r7_wiring.rs, r7_export_schedule.rs}`)

### Goal
One new way for a run to exist: the engine is told a session is arranged. Everything that created or cancelled a run because a photo or a setting changed is removed. **Not in this unit:** the session mark (U4), any screen.

### Read first
`master_plan_research.md` §6.3, §6.5, §6.6, §6.8 · book `08B` MC-A6 (all), MC-A7 (Reuse, Follow-up trigger set), changelog 7 and 13 · code `crates/research/src/{runs.rs, rules/run_rules.rs, domain.rs, config.rs, api.rs, api_impl.rs, stages.rs (reuse), service.rs, store.rs}`, `crates/research/tests/engine/{harness.rs, lifecycle.rs, resume.rs, api.rs, defaults.rs}`.

### Files
Edited: `crates/research/src/{domain.rs, config.rs, runs.rs, rules/run_rules.rs, api.rs, api_impl.rs, stages.rs, service.rs}`; `crates/research/tests/engine/{harness.rs, lifecycle.rs, resume.rs, api.rs, defaults.rs, main.rs}`; `crates/testkit/src/test_app.rs`; `crates/testkit/tests/suite/research/{r7_wiring.rs, r7_export_schedule.rs}`; `crates/research-tool/tests/tool/u3_report_and_replay.rs` (only if it names the retired helper). New: `crates/research/tests/engine/arranged.rs`.

### Core mechanisms (the only functions with unit tests)
`rules::run_rules::arranged_needed` (whether a press creates a run, from the session's runs and the photo-derived inputs of now) · `rules::run_rules::may_start` (the `arranged` trigger: no quiet period, the source wait of an automatic run).

### Ordered tasks
1. `RunTrigger` gains `Arranged` (stored as `arranged`). `Prerequisite::AutoOff` and its code are removed; the prerequisite list is the first seven of MC-A6.
2. `ResearchSettings` loses `auto` and `quiet_seconds`; reading a slot that carries them ignores them; `validate` no longer looks at them.
3. `ResearchApi::arranged(session_id)` answers whether a run was queued. It queues one run with trigger `arranged` when the session has at least one Main photo, none of the five creation-blocking prerequisites fails, no run of the session is queued, and one of these holds: the session has no run; the latest run ended `failed` or `cancelled`; the photo-derived inputs of now differ from the input snapshot of the latest run that started (MC-A7's trigger set). Otherwise it queues nothing and answers no. The Unassigned session is refused with the existing sentence.
4. `session_changed` records the session's last change (the source wait still needs it) and creates no run. `settings_changed` cancels nothing. The follow-up path is deleted.
5. Start: an `arranged` run has no quiet period; it waits for missing sources like an automatic run did (MC-A6 Readiness) and shows "Waiting for Capture One" the same way. Start order is unchanged (`manual` first, then `queued_at`, then id). Reuse (MC-A7) applies to an `arranged` run.
6. When the service is built, every `queued` run whose trigger is `auto` or `follow_up` is set `cancelled` (left by an earlier build; nothing would ever start it).
7. The tests of the table above; the harness and testkit helpers start runs through `arranged`.

### Criteria (7 criteria, 22 rows)
Boundary: `ResearchApi::arranged` and `ResearchService::{tick, status}` on the research harness (`Harness`, manual clock, the crate's fakes); outcome = the returned answer, the `research_runs` rows, the status.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A25·C1(a) | `arranged` | a session with one Main photo with a source, no run, every prerequisite passing | answers yes; exactly one run, `queued`, trigger `arranged` | MC-A6 (changelog 13 b) |
| A25·C1(b) | `arranged` | the same session again while that run is `queued` | answers no; still exactly one run, trigger `arranged` | MA13 |
| A25·C1(c) | `arranged` | a `queued` run with trigger `manual` exists | answers no; one run, trigger still `manual` | MC-A6 |
| A25·C1(d) | `arranged` | the latest run `finished`; no photo changed since it started | answers no; no new run | MC-A7 (changelog 13 e) |
| A25·C1(e) | `arranged` | the latest run `stopped_at_cap`; no photo changed | answers no; no new run | MC-A7 |
| A25·C1(f) | `arranged` | the latest run `failed`; no photo changed | answers yes; one new `queued` run, trigger `arranged` | MC-A6 |
| A25·C1(g) | `arranged` | the latest run `cancelled`; no photo changed | answers yes; one new `queued` run, trigger `arranged` | MC-A6 |
| A25·C1(h) | `arranged` | the latest run `finished`; a Main photo was added since | answers yes; one new `queued` run, trigger `arranged` | MC-A7 (changelog 13 e) |
| A25·C1(i) | `arranged` | a run is `running`; a Condition photo was added after it started; pressed twice | the first answers yes and one `queued` run exists beside the running one; the second answers no and there is still one | MA13 |
| A25·C1(j) | `arranged` | a run is `running`; no photo changed since it started | answers no; no queued run | MC-A7 |
| A25·C2(a) | `arranged` | a session whose photos are one Condition and one unreviewed photo, no Main | answers no; no run row | MC-A6 |
| A25·C2(b) | `arranged`, `status` | each alone: no integration; no model key; no visual-search key; a stage model missing from the price table; a layout with no field research may write | each: answers no; no run row; `status` names that prerequisite with its sentence (`master_plan_research.md` §6.6) | MA7 (HA4) |
| A25·C2(c) | `arranged` | the Unassigned session | refused with "the Unassigned session is never researched"; no run row | MC-A6 (R-IA23) |
| A25·C3(a) | `session_changed`, `tick` | a saved slot carrying `"auto": true, "quiet_seconds": 1`; a session gets its first Main photo with a source; ten minutes pass on the manual clock | no run row | MC-A6 (changelog 13 d) |
| A25·C3(b) | `session_changed`, `tick` | after a `finished` arranged run: a Main photo added, another recomposed, a third moved to Condition; ten minutes pass | no new run row | MC-A7 (changelog 13 e) |
| A25·C4(a) | `tick` | `arranged` answered yes; `tick` with no time passed since the session's last change | the run is `running` | MC-A6 (changelog 13 d) |
| A25·C4(b) | `tick` | two Main photos, one without a source; `arranged`; `tick` 1 s before `source_wait_seconds` have passed since the session's last change, then at that moment | first `queued`; then `running`, and its input snapshot names only the photo with a source | MC-A6 (R-IA14) |
| A25·C4(c) | `tick`, `status` | one Main photo without a source; `arranged`; the source wait passes | the run stays `queued`; `status` names `WaitingForEditor` ("Waiting for Capture One") and `wait_ends_at` equals the last change plus the source wait | MC-A6 |
| A25·C4(d) | `tick` | sessions A then B arranged, then Run again for session C, nothing running | the runs start in the order C, A, B, never two at once | MA13 (R-IA15) |
| A25·C5(a) | `status` | every prerequisite passing, no run | no reason; `can_run` yes | MA7 |
| A25·C6(a) | building the service | the database holds a `queued` run with trigger `auto`, a `queued` one with `follow_up`, a `queued` `manual` and a `running` `auto` | the first two are `cancelled`; the `manual` run is still `queued`; the running run is still `running` | MA13 |
| A25·C7(a) | `ResearchSettings::from_slot`, serialising | a slot `{"auto": true, "quiet_seconds": 5, "max_pages": 7}` | `max_pages` is 7 and every other field is its default; the value written back has neither `auto` nor `quiet_seconds` | MC-A6 (changelog 13 d) |

### Delegated choices
Whether `arranged_needed` takes the run rows or a small summary of them · the text of the log line for a cancelled leftover run.

### Notes
- Hazard (P70): no real-time waits; every wait is on the manual clock.
- Hazard (R4 review, R7): one start app-wide. `tick` stays the only place a run starts; `arranged` only inserts a `queued` row.
- The app's wiring still calls `session_changed` on photo events (R7·U1 observer); that stays, it now only moves the source wait.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib rules::run_rules` · `cargo test -p psm-research --test engine arranged::` · `cargo test -p psm-research --test engine lifecycle::` · `cargo test -p psm-research --test engine resume::` · `cargo test -p psm-research --test engine defaults::` · `cargo test -p psm-research --test engine api::` · `cargo test -p psm-research-tool` · `cargo test -p psm-testkit --test suite research::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 2 — Fields research leaves alone, extra shop addresses, reset to defaults

**Unit id:** `R11·U2` · **Rows:** A25·C8–C11 · **Changes app files:** no

### Goal
Three settings in the engine and one function. `excluded_keys`: the fields research never asks for, suggests or spends on. `extra_shop_hosts`: addresses that count as the owner's own shop beside the ones the shop reports. `settings_reset`: the value to save when the person resets research settings. **Not in this unit:** the screens (U6); the command (U4).

### Read first
`master_plan_research.md` §6.6, §6.8 · book `08B` MC-A1 (fields never written), MC-A3 (trusted entry, shop hosts, own listing), MC-A4 (field class), MC-A6 (prerequisite 5), changelog 11 (c), 12 (a) · evidence "Owner answers" 3 (the owner's addresses) · `backlog_research/R10_research_defaults.md` (the reset note) · code `crates/research/src/{config.rs, rules/confidence.rs, rules/sites.rs, rules/write_table.rs, stages.rs (class_of, fields_of), decide.rs, runs.rs (prerequisite_input), api.rs, api_impl.rs, shop.rs}`.

### Files
Edited: `crates/research/src/{config.rs, rules/confidence.rs, rules/sites.rs, stages.rs, decide.rs, runs.rs, api.rs, api_impl.rs}`; `crates/research/tests/engine/{main.rs, stages.rs (helpers only), api.rs}`. New: `crates/research/tests/engine/settings_r11.rs`.

### Core mechanisms (the only functions with unit tests)
`rules::confidence::field_class` (the left-alone step) · `rules::sites::parse_entry` reused for an extra shop address.

### Ordered tasks
1. `ResearchSettings` and the run's settings snapshot gain `excluded_keys` (a list of field keys, default empty) and `extra_shop_hosts` (a list, default empty).
2. A key in `excluded_keys` is treated exactly as a never-written field of MC-A1: it is in no request of any stage, no value is written for it, the final step names it "not researched", it does not count as a writable field for the "nothing to research" prerequisite, and an earlier suggestion in it is left as it is.
3. Shop hosts of a run = the hosts the commerce port reports, then each extra address in its stored form (`H` with one leading `www.` removed, MC-A3 Trusted entry), without repeats. Everything that uses shop hosts (site class, own listing, reading order, the trusted-site refusal) uses this list.
4. `validate` refuses an extra shop address that has no dot, is an IP literal or is empty, and one that equals, is a parent of, or is a subdomain of a trusted site; the text names the entry. `site_add` refuses a trusted site covered by an extra shop address, as it does for a reported shop host.
5. `ResearchApi::settings_reset()` returns the `research` value to save: `ResearchSettings::default()` with `trusted_sites`, `attribution_keys`, `excluded_keys`, `extra_shop_hosts`, `detailed_condition` and `language` taken from the live value. It writes nothing itself (like the three site methods).

### Criteria (4 criteria, 12 rows)
Boundary: the named rule; `ResearchService` runs on the research harness with scripted fakes; `ResearchApi::{validate_settings, site_add, settings_reset}`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A25·C8(a) | `rules::confidence::field_class` | a Details text field, a dimension field and a Condition choice field, each with its key in `excluded_keys`; the same three without | with: all three never-written; without: visible, measured, condition | MC-A4 (changelog 12 a) |
| A25·C8(b) | a full run | layout: wood (visible), width (dimension), designer (attribution), scratch (condition); `excluded_keys` = width and scratch; a read page states a width; a Condition photo exists | no request of Observe, Read, Labels or Condition lists width or scratch among its fields; no value is written for either; the final step lists both as "not researched"; wood and designer are decided as without the setting | MA10, MA6 |
| A25·C8(c) | `arranged`, `status` | every field of the layout that research may write is in `excluded_keys` | answers no; no run row; `status` names `NothingToResearch` | MC-A6 |
| A25·C8(d) | a full run | width holds an `ai` + `suggested` value at `medium` from an earlier run; width is then added to `excluded_keys`; a new run finds nothing for it | the row is unchanged (value, confidence, reason, sources) | MA10 |
| A25·C9(a) | a run's settings snapshot; `rules::sites::site_of` | the shop reports `x.se` and `x.myshopify.com`; `extra_shop_hosts` = `www.x.com`, `y.myshopify.com`, `x.se` | snapshot shop hosts are `x.se`, `x.myshopify.com`, `x.com`, `y.myshopify.com` in that order; a page on `www.x.com` has site class `shop` | MA11 (MC-A3) |
| A25·C9(b) | `rules::sites::site_of` | shop hosts `x.se` only; a page on `www.x.se` | site class `shop` | MA11 |
| A25·C9(c) | a full run | the item's own listing has the path tail `products/p`; extra address `x.com`; the search returns `https://x.com/products/p` and one trusted page | the first is set apart as "this item's own listing" and never read; the trusted page is read | MA11 (R-IA9) |
| A25·C9(d) | `ResearchApi::validate_settings` | `extra_shop_hosts` with, each alone: `localhost`; `10.0.0.1`; an empty text; `shop.pamono.com` while `pamono.com` is a trusted site; `x.com` | the first four are refused and the text names the entry (the fourth says it is already a trusted site); the last is accepted | MC-A3 |
| A25·C9(e) | `ResearchApi::site_add` | extra address `x.com`; add the trusted site `shop.x.com` | refused, "already covered by x.com"; the trusted list is unchanged | MC-A3 |
| A25·C10(a) | `ResearchApi::settings_reset` | a live slot with `auto` true, no prices, empty stage models, `max_pages` 3, `language` "Svenska", trusted sites e1 and e2, `attribution_keys` one key, `excluded_keys` one key, `extra_shop_hosts` one host, a filled `detailed_condition` | the returned value has the stages, prices, both cost limits, `max_pages`, `source_wait_seconds`, `record_days`, `call_timeout_seconds` and `providers` of `ResearchSettings::default()`; the five lists and the language are as in the live slot; it has neither `auto` nor `quiet_seconds` | MC-A6 (prerequisite 4, the price table) |
| A25·C10(b) | `arranged`, `status` (rule 10) | that returned value saved, the keys set, a session with a Main photo with a source | `status` names no reason; `arranged` answers yes | MA7 |
| A25·C11(a) | `ResearchSettings::default()` | the default value | `excluded_keys` and `extra_shop_hosts` are empty; `detailed_condition` has an empty key and no levels | MC-A4 (changelog 12 a, b) |

### Delegated choices
Whether the left-alone step is a parameter of `field_class` or of `never_written` · the exact refusal sentences beyond what the rows fix.

### Notes
- The settings snapshot is what a run executes from (MC-A3): a change of either list applies to runs that start later.
- `detailed_condition` is declared in this unit's settings shape (so reset and defaults can name it) and used in U3.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib rules::confidence` · `cargo test -p psm-research --lib rules::sites` · `cargo test -p psm-research --test engine settings_r11::` · `cargo test -p psm-research --test engine api::` · `cargo test -p psm-research --test engine stages::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 3 — Detailed condition derived; no designer or maker from another product of the shop

**Unit id:** `R11·U3` · **Rows:** A25·C12–C16 · **Changes app files:** no

### Goal
(1) The field named by the setting `detailed_condition.key` is never read from a photo or a listing. Stage 7 derives it from the levels in the item's condition level fields, by a table the person enters. (2) For an attribution field, a page of the owner's shop counts at no level. **Not in this unit:** the table's screen (U6).

### Read first
`master_plan_research.md` §6.5, §6.8 · book `08A` §6, `08B` MC-A1 (write table, prior and conflict), MC-A2 (choice mapping), MC-A4 (all), MC-A7 (stage 7 identity), changelog 12 (b) and its measurement finding · evidence "Acceptance measurement" (pieces 6 and 33; detailed condition 21 right, 19 wrong) · `claude_design/reference/SHOPIFY_PRODUCT_METAFIELDS.md` (the shop's `custom.detailed_condition` choices Excellent, Very Good, Good, Worn; the twelve `custom.condition_*` fields with N/A, None, Minor, Moderate, Major) · code `crates/research/src/{decide.rs, rules/confidence.rs, stages.rs (class_of, fields_of), writes.rs, domain.rs}`, `crates/testkit/src/fixtures/definitions.rs`.

### The rule (this is the contract of the new mechanism; charter rule 5)
- **Setting.** `detailed_condition` = a field key and an ordered list of levels, best first. Each level names a condition value (the text of a choice of the condition level fields, such as "Minor") and either the choice of the detailed-condition field to write, or "counts for nothing" (for a value such as "N/A").
- **Condition level fields** of an item: the fields of its layout whose definition group is `condition` and whose core type is `choice`, in layout order. A level field in `excluded_keys` still counts here (the person fills it by hand).
- **The value of a level field** is the live value its row holds in stage 7's writing transaction, after this run's condition writes: a row that is not cleared, of any provenance.
- **Derivation,** first match wins: (1) the layout has no condition level field → not determined, "no condition levels in this form". (2) a level field has no value → not determined, "condition levels are incomplete: <labels>". (3) a value is not in the setting's list → not determined, "no grade is set for <value>". (4) every value counts for nothing → not determined, "no condition level applies". (5) otherwise the worst value is the one latest in the list; its choice is mapped by MC-A2's choice rule against the field's current choices; no such choice → not determined, "the setting names a choice the field does not have". (6) the mapped choice is written at `low` when any level field's value is an `ai` value at `low`, else at `medium`. It is never `high`.
- **Reason** (rule-written, not sent to the model): "From the condition levels: the worst is <value> (<labels of the fields holding it>)." **Sources:** none.
- **The write** goes through MC-A1 like every other (a human value is never replaced; over a default or a shop value the prior is kept and the field is a conflict).
- **Unset.** With an empty key the field has no special class and is treated as before.

### Files
New: `crates/research/src/rules/derived_condition.rs`, `crates/research/tests/engine/derived.rs`. Edited: `crates/research/src/{rules/mod.rs, rules/confidence.rs, domain.rs (FieldClass::Derived), config.rs, stages.rs, decide.rs}`, `crates/research/tests/engine/{main.rs, stages.rs (helpers only)}`.

### Core mechanisms (the only functions with unit tests)
`rules::derived_condition::derive` · `rules::confidence::field_class` (the derived step) · `rules::confidence::decide_attribution` as stage 7 calls it for an attribution field (shop items left out).

### Ordered tasks
1. `rules::derived_condition::derive`: the derivation above as a pure function over the setting's levels, the level fields' values (value, label, whether an `ai` value at `low`) and the detailed-condition definition.
2. `FieldClass::Derived`; the class order of "What this plan supersedes". A derived field is in no request of stages 1, 3, 5 and 6.
3. Stage 7 decides the derived field after the condition fields, in the same writing transaction, with the rule's reason; it is not part of the Decide model call. Its `concluded` or `not_determined` step names the key (MA6). `detailed_condition` and `excluded_keys` join the stage 7 unit identity.
4. For a field of class attribution, stage 7 leaves listing items of site class `shop` out before the attribution decision. Measured and visible fields pass every item as before. The read step of such a page gets the detail line "another product of your shop: not used for designer and maker" when the page states an attribution field.
5. **Carried from the R10 review:** a test about attribution sets `attribution_keys` itself; none relies on a helper's default.

### Criteria (5 criteria, 26 rows)
Boundary: the named rule; full runs on the research harness with scripted fakes. In C12 the setting is: None → A, Minor → B, Moderate → C, Major → D, N/A → counts for nothing; the detailed-condition field's choices are A, B, C, D.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A25·C12(a) | `derive` | level values None, None | A, `medium` | MC-A4 (changelog 12 b) |
| A25·C12(b) | `derive` | None, Minor | B, `medium` | MC-A4 |
| A25·C12(c) | `derive` | Minor, Moderate, None | C, `medium` | MC-A4 |
| A25·C12(d) | `derive` | Moderate, Major | D, `medium` | MC-A4 |
| A25·C12(e) | `derive` | N/A, Minor | B, `medium` | MC-A4 |
| A25·C12(f) | `derive` | N/A, N/A | not determined, "no condition level applies" | MC-A4 |
| A25·C12(g) | `derive` | Minor, and one level field with no value | not determined; the reason names that field's label | MC-A4 |
| A25·C12(h) | `derive` | None, and Minor held as an `ai` value at `low` | B, `low` | MC-A4 |
| A25·C12(i) | `derive` | the setting's list without Major; values Minor, Major | not determined, "no grade is set for Major" | MC-A4 |
| A25·C12(j) | `derive` | a layout with no condition level field | not determined, "no condition levels in this form" | MC-A4 |
| A25·C12(k) | `derive` | values Minor; the setting maps Minor to a text that is none of the field's choices | not determined, "the setting names a choice the field does not have" | MA11 (MC-A2) |
| A25·C13(a) | `field_class` | a Details choice field whose key is `detailed_condition.key` | derived | MC-A4 |
| A25·C13(b) | `field_class` | the same key also in `excluded_keys` | never-written | MC-A4 (changelog 12 a) |
| A25·C13(c) | `field_class` | the same field with an empty `detailed_condition.key` | visible | MC-A4 |
| A25·C14(a) | a full run | two Condition photos; stage 6 reads scratch Minor (sure), veneer Moderate (sure), every other level field None (sure); the setting of C12 | the detailed-condition row is C, provenance `ai`, status `suggested`, confidence `medium`, reason "From the condition levels: the worst is Moderate (<veneer's label>).", no sources; no request of Observe, Read, Labels or Condition lists the key; one `concluded` step names the key | MA6, MA11 |
| A25·C14(b) | a full run | no Condition photo and no stored level | no row is written for the key; the `not_determined` step names it with "condition levels are incomplete" | MA6 |
| A25·C14(c) | a full run | no Condition photo; every level field holds a `human` value, the worst Minor | the row is B at `medium`; the level fields are unchanged | MA10, MA11 |
| A25·C14(d) | a full run | `detailed_condition.key` empty; the field is in the layout | the Observe request lists the field (it is researched as a visible field, as before) | MC-A4 |
| A25·C15(a) | attribution decision as stage 7 makes it | a shop `match` states A, nothing else | not determined | MA2 (changelog 12) |
| A25·C15(b) | same | a shop `similar` states A, nothing else | not determined | MA2 |
| A25·C15(c) | same | a shop `match` states A; a `match` on trusted e1 states A | A, `low` | MA2 |
| A25·C15(d) | same | a shop `match` states A; `match` on trusted e1 and e2 state B | B, `medium`; A is not named among other values | MA2 |
| A25·C15(e) | same | a label names A; a shop `match` states B | A, `high`; B is not named | MA2 |
| A25·C15(f) | measured decision | a shop `match` states a width of 120 cm, nothing else | 120 cm, `medium` (unchanged) | MA11 |
| A25·C15(g) | visible decision | a sure photo read of Teak; a shop `match` states Teak | Teak, `high` (unchanged) | MA11 |
| A25·C16(a) | a full run | `attribution_keys` set to the designer key by the test; the search returns one page on the shop that is not the item's own listing and one untrusted page; Compare answers `match` for the shop page and `different` for the other; the shop page states designer X | designer is not determined and no row is written; the shop page's read step carries "another product of your shop: not used for designer and maker" | MA2 |

### Delegated choices
Where the shop items are dropped (inside `decide_attribution` behind a parameter, or before the call) · how the level fields' values are read inside the writing transaction.

### Notes
- Hazard: `decide_measured` and `decide_visible` call the attribution function over listings; the drop must not reach them (rows C15(f), (g)).
- The twelve `custom.condition_*` fields and `custom.detailed_condition` are in `crates/testkit/src/fixtures/definitions.rs`; the detailed-condition field is group Details there, which is why it is a visible field today.
- The Decide model call still writes the reasons of every other decided field.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib rules::derived_condition` · `cargo test -p psm-research --lib rules::confidence` · `cargo test -p psm-research --test engine derived::` · `cargo test -p psm-research --test engine stages::` · `cargo test -p psm-research --test engine writes::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 4 — The session mark, "Arranged — next", publishing marks first

**Unit id:** `R11·U4` · **Rows:** A25·C17–C23 · **Changes app files:** yes

### Goal
The app side of changelog 13: a session can be marked arranged; marking tells the engine; the app knows which session is next; asking for a publish preview marks the selection first. Two new commands, three widened shapes, bindings regenerated once. **Not in this unit:** any screen.

### Read first
`master_plan.md` §6 (sessions, commands, bindings), §7 (bindings regeneration) · `master_plan_research.md` §3 (Isolation, Scratch data only), §5 (seams), §6.4, §6.8 · book `01` (H6, H8), `07A` MC-10, `08A` §7, `08B` MC-A10, changelog 13 · code `crates/core/src/{domain/session.rs, ports/store.rs, services/sessions.rs (list, ready_session_ids), services/publish.rs (preview, confirm), services/retention.rs (cleared_sessions)}`, `crates/infra/src/sqlite/{ensure.rs, store.rs}`, `crates/integrations/src/lib.rs` (`AppHandles`, `ResearchWiring`, `SettingsSetContext`), `apps/desktop/src-tauri/src/{commands/sessions.rs, commands/publish.rs, commands/research.rs, commands/settings.rs, lib.rs}`, `crates/testkit/src/test_app.rs`, `crates/testkit/tests/suite/research/r7_wiring.rs`.

### Files
Edited: `crates/core/src/{domain/session.rs, ports/store.rs, services/sessions.rs}`; `crates/infra/src/sqlite/{ensure.rs, store.rs}`; `crates/integrations/src/lib.rs`; `apps/desktop/src-tauri/src/{commands/sessions.rs, commands/publish.rs, commands/research.rs, commands/settings.rs, lib.rs}`; `apps/desktop/src/services/{bindings.ts (generated), sessions.ts, research.ts, publish.ts}`; `crates/testkit/tests/suite/research/mod.rs`; fixtures or tests that list the columns of `sessions`. New: `crates/integrations/src/arranged.rs`, `crates/testkit/tests/suite/research/r11_arranged.rs`.

### Core mechanisms (the only functions with unit tests)
None (the "next" rule is proven at the service in C18).

### Ordered tasks
1. `sessions.arranged_at` (nullable integer) in `psm_infra::sqlite::ensure`; `Session.arranged_at`; a store method that sets it once.
2. `SessionsService::arrange(session_id)`: sets `arranged_at` to now when it is empty, emits `SessionChanged`, and says whether it was newly set. The Unassigned session is refused. `SessionsService::next_unarranged(after)`: the rule of "Points recorded by planning".
3. `crates/integrations/src/arranged.rs`, two functions that only connect services (H8). **Arrange and next:** mark through core; tell the engine (`ResearchApi::arranged`) when research exists, and never fail because research cannot run (HA4: a refusal or an error is logged); return the next session and whether a run was queued. **Preview for publish:** for each session of the selection that is not marked, do the same marking; then `PublishService::preview`; then, per item, whether a research run of its session is `queued` or `running`.
4. Commands: `session_arrange_next(session_id)` → the next session's id or none, and whether research was queued. `research_settings_reset()` → the reset value, saved through the same path as a research settings save (validation, then `settings_changed`), returned as saved. `publish_preview` calls the preview function of task 3; each preview item gains `researching`. `publish_confirm` is unchanged.
5. Shapes: the session view and the list row carry the mark (`arranged_at`; `arranged` on the row). The research settings DTO gains `excluded_keys`, `extra_shop_hosts`, `detailed_condition` and loses `auto` and `quiet_seconds`. Regenerate `bindings.ts` (this unit is the only one in R11 and R12 that does).
6. The tests. An app test about attribution sets `attribution_keys` in its own settings (carried from the R10 review); none of this unit's rows needs it.

### Criteria (7 criteria, 17 rows)
Boundary: `SessionsService`, the two functions of `psm_integrations::arranged` and `PublishService::confirm` on `TestApp::with_research()` (the wired app with the crate's fakes), `TestApp::tick`; outcome = stored state, returned values, events, runs.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A25·C17(a) | arrange and next | an item session with one Main photo with a preview, not marked | `arranged_at` equals the clock's now; one research run of that session is `queued` with trigger `arranged`; `SessionChanged` for it was emitted; "research queued" is yes | MC-A6 (changelog 13 a, b) |
| A25·C17(b) | arrange and next | the same session again, an hour later, after that run finished and no photo changed | `arranged_at` is still the first time; no second run; "research queued" is no | MC-A7 (changelog 13 e) |
| A25·C17(c) | `SessionsService::arrange` | the Unassigned session | refused; nothing stored | MC-A6 (R-IA23) |
| A25·C17(d) | store | a database whose `sessions` table has no `arranged_at` column and holds two sessions is opened twice; one session is then marked; the app is restarted | after the first open the column exists and both sessions are unmarked; after the restart the marked session reads back the same time, the other none | H4 |
| A25·C18(a) | arrange and next | item sessions A, B, C, D in creation order, each with a photo, none marked; called for B | next is C | MC-A6 (changelog 13 a) |
| A25·C18(b) | arrange and next | B and C marked, A not; called for D | next is A | MC-A6 |
| A25·C18(c) | arrange and next | every other session is marked | next is none | MC-A6 |
| A25·C18(d) | arrange and next | the only unmarked sessions are: a published one, an archived one (files cleared), one whose only photo is deleted, and the Unassigned session | next is none | MC-A6 |
| A25·C19(a) | arrange and next | the wired app with no integration and no key; a session with a Main photo, and a second unmarked session | the session is marked; next is the second session; "research queued" is no; no run row; no request reached a fake provider | MA7 (HA4) |
| A25·C20(a) | preview for publish | one unmarked session with a Main photo with a preview; research can run | the session is marked; one run `queued` with trigger `arranged`; the item's `researching` is yes | MC-A6 (changelog 13 e) |
| A25·C20(b) | preview for publish | then `TestApp::tick` until the run is `finished`, the fakes scripted so one field gets a `low` suggestion; preview again | `researching` is no; `n_withheld` is 1; no second run | MA8, MA17 |
| A25·C20(c) | preview for publish | a session marked earlier whose run finished and whose photos did not change | no new run; `researching` is no | MC-A7 |
| A25·C20(d) | preview for publish | the wired app with no model key; an unmarked session | the session is marked; no run; `researching` is no; every other field of the preview equals `PublishService::preview` for the same selection | MA7 (HA4) |
| A25·C20(e) | `PublishService::confirm` | a preview whose item is `researching` (the run blocked in a fake call), confirmed with what was shown | the confirmation succeeds; the publish item's frozen attributes hold no value written by that run | MA8 (MC-A10) |
| A25·C21(a) | the wired app | a session's first photo is classified Main through `ReviewService`; `TestApp::tick`; ten minutes pass; tick | no research run row | MC-A6 (changelog 13 d) |
| A25·C22(a) | reset through the settings path | `settings.json` holds a `research` slot with no prices, empty stage models and two trusted sites | after the reset the file's slot has the default stages and prices and the same two trusted sites; a following `status` names no `PriceMissing` | MC-A6 (prerequisite 4, the price table) |
| A25·C23(a) | generated bindings (`cargo test -p psm-desktop export_bindings`) and the desktop command tests | the two new commands | `bindings.ts` exports `sessionArrangeNext` and `researchSettingsReset`; the session row type has `arranged`, the preview item type has `researching`, the research settings type has `excluded_keys`, `extra_shop_hosts`, `detailed_condition` and neither `auto` nor `quiet_seconds`; both commands reach the wired engine in the desktop crate's command test | MC-A6 (changelog 13 a; charter rule 10: the command surface reaches the engine) |

### Delegated choices
Names inside `arranged.rs` · whether `researching` is computed from `ResearchApi::status` or from a small query the research crate offers through its API (the Isolation rule of `master_plan_research.md` §3 holds either way) · how the reset is saved (through `SettingsSetContext` or the existing `save_research` helper of `commands/research.rs`).

### Notes
- Hazard (R7 lesson): commands stay thin; the composition is in `crates/integrations`. No research rule is written there (H8).
- Hazard: the preview is read often (the popover reloads). Marking is idempotent and `arranged` answers no when nothing changed, so a reload queues nothing (row C20(c)).
- `PublishService` and `psm-core` learn nothing about research: the mark is a session fact; the engine is told by the integration layer.
- Scratch data only (`master_plan_research.md` §3) now also covers the new column.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-testkit --test suite research::r11_arranged::` · `cargo test -p psm-testkit --test suite research::r7_wiring::` · `cargo test -p psm-testkit --test suite p01_storage_identity::` · `cargo test -p psm-testkit --test suite p03_session_state_list::` · `cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::` · `cargo test -p psm-testkit --test suite p13_publish_runs::` · `cargo test -p psm-testkit --test suite p16_operations::` · `cargo test -p psm-desktop` · `cd apps/desktop && npm run typecheck`

## Unit 5 — The button, the list tag, the publish line, the panel hint

**Unit id:** `R11·U5` · **Rows:** A25·C24–C27 · **Changes app files:** yes (TypeScript only)

### Goal
What the photographer sees of changelog 13: the "Arranged — next" button on the open session, an "Arranged" tag in the sessions list, one line in the publish popover while research runs, and one sentence in the research panel before any run. **Not in this unit:** Settings (U6).

### Read first
`docs/intention/evidence/design_handoff.md` (rules) · `master_plan_research.md` §6.8 · book `08B` changelog 13 · code `apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/features/sessions/SessionRow.tsx`, `apps/desktop/src/features/publish/{PublishPopover.tsx, usePublish.ts}`, `apps/desktop/src/features/form/{ResearchPanel.tsx, FormDialog.tsx}`, `apps/desktop/src/services/{sessions.ts, publish.ts, research.ts, events.ts}`, `apps/desktop/src/App.tsx` (routes; opening a session activates it).

### Files
Edited: `apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/features/sessions/SessionRow.tsx`, `apps/desktop/src/features/publish/PublishPopover.tsx`, `apps/desktop/src/features/form/ResearchPanel.tsx`, `apps/desktop/src/services/{sessions.ts, publish.ts}`, `apps/desktop/src/features/form/__tests__/r6_research_panel.test.tsx` (fixture). New: `apps/desktop/src/features/shell/__tests__/r11_arranged_button.test.tsx`, `apps/desktop/src/features/sessions/__tests__/r11_arranged_tag.test.tsx`, `apps/desktop/src/features/publish/__tests__/r11_publish_research.test.tsx`, `apps/desktop/src/features/form/__tests__/r11_panel_hint.test.tsx`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. Header, an item session open and not archived: the button "Arranged — next" beside Details; "Arranged ✓ — next" when the session is marked. A click calls the command once, then opens the returned session by the route the sessions list uses, or goes to the sessions list with the toast "Every session is arranged" when there is none. A failure is a toast and the route stays.
2. Sessions list row: the tag "Arranged" when the row is marked.
3. Publish popover: under an item whose `researching` is yes, the line "Research is still running for this item. Its suggestions are not in this publish unless you wait." Confirm stays enabled. The popover reloads its preview on the `research_changed` event.
4. Research panel, when there is no run, no reason and research can run: the sentence "Research starts when you press Arranged — next." above the Research button.

### Criteria (4 criteria, 9 rows, all `[rule]`)
Boundary: the component rendered in a UI test with the services mocked; outcome = what is shown and which service call was made.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A25·C24(a) | Header | an unmarked item session open; click "Arranged — next"; the command answers session 7 | the command was called once with the open session's id; the route is `#/session/7` | `[rule]` 08B changelog 13 (a) |
| A25·C24(b) | Header | the command answers no next session | the route is `#/`; the toast "Every session is arranged" is shown | `[rule]` 08B changelog 13 (a) |
| A25·C24(c) | Header | the open session is marked | the button reads "Arranged ✓ — next" | `[rule]` 08B changelog 13 (a) |
| A25·C24(d) | Header | the Unassigned session open; an archived session open | no such button in either | `[rule]` 08B changelog 13 (a) |
| A25·C24(e) | Header | the command fails with a message | a toast shows the message; the route is unchanged | `[rule]` 08B changelog 13 (a) |
| A25·C25(a) | sessions list row | a row with `arranged` yes; a row with no | the first shows the tag "Arranged"; the second shows none | `[rule]` 08B changelog 13 (a) |
| A25·C26(a) | publish popover | a preview whose item has `researching` yes | the line of task 3 is shown under that item; the Confirm button is enabled | `[rule]` 08B MC-A10, changelog 13 (e) |
| A25·C26(b) | publish popover | then a `research_changed` event; the preview now answers `researching` no and one withheld suggestion | the preview was read again; the line is gone; the withheld line is shown | `[rule]` 08B changelog 13 (e) |
| A25·C27(a) | research panel | a status with no run, no reason, `can_run` yes | the sentence of task 4 and the Research button are shown | `[rule]` 08B changelog 13 (b) |

### Delegated choices
Icon and placement details within the design handoff's rules · whether the tag is a pill or a check mark with a tooltip.

### Notes
- Opening the next session switches where captures go (existing `App.tsx` behaviour). The narrative says so.
- The form's Research / Run again button is unchanged (changelog 13 d: "the manual Run research stays").

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cd apps/desktop && npx vitest run src/features/shell src/features/sessions src/features/publish src/features/form` · `npm run typecheck`

## Unit 6 — Settings → Research: the pickers, the detailed-condition table, reset

**Unit id:** `R11·U6` · **Rows:** A25·C28–C33 · **Changes app files:** yes (TypeScript only)

### Goal
Settings → Research lets the person point research at their own shop: which fields name the designer and the maker, which fields research leaves alone, which other addresses are their shop, how the detailed condition follows from the levels, and a reset. The two retired controls go. **Not in this unit:** the condition-photos key (it stays a text box).

### Read first
`docs/intention/evidence/design_handoff.md` · `master_plan_research.md` §6.6, §6.8 · book `08B` changelog 11 (c), 12 (a), (b) · R10 plan, "Carried from earlier reviews" (the picker note of 2026-11-13) · code `apps/desktop/src/features/settings/{ResearchSection.tsx, SettingsPage.tsx}`, `apps/desktop/src/services/{attributes.ts (listDefinitions), settings.ts, research.ts}`, `apps/desktop/src/features/settings/__tests__/{r6_research_settings.test.tsx, r7_research_export.test.tsx}`.

### Files
New: `apps/desktop/src/features/settings/ShopFieldPicker.tsx`, `apps/desktop/src/features/settings/__tests__/r11_research_pickers.test.tsx`. Edited: `apps/desktop/src/features/settings/ResearchSection.tsx`, `apps/desktop/src/services/research.ts`, `apps/desktop/src/features/settings/__tests__/{r6_research_settings.test.tsx, r7_research_export.test.tsx}`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `ShopFieldPicker`: the linked shop's product fields from `listDefinitions()`, each shown by label and key, with a check box; a chosen key that is not one of the shop's fields is still listed, marked "Not one of your shop's fields", so it can be removed. With no shop linked it says "Connect a shop to choose fields".
2. **Designer and maker** (replaces the "Attribution keys" text boxes): the picker over `attribution_keys`. When the saved list is empty and the shop is linked, the fields whose label is "Designer" or "Manufacturer" (compared without case, surrounding spaces ignored) are preselected and marked "suggested — not saved yet"; nothing is saved until the person saves. When nothing is chosen the section shows "No designer or maker field is chosen: research will not suggest a designer or a maker."
3. **Fields research leaves alone:** the picker over `excluded_keys`, nothing preselected.
4. **Extra shop addresses:** a list with add and remove over `extra_shop_hosts`; a refusal from saving is shown as it is.
5. **Detailed condition:** a single choice of field among the shop's choice fields; then one row per condition level, taken from the choices of the shop's Condition-step choice fields in their own order, each with a select of the chosen field's choices, "counts for nothing", or unset. Saved as `detailed_condition` in row order, unset rows left out. With no field chosen: "Not set: Detailed condition is filled like any other field."
6. **Reset research settings to defaults:** a button with a confirmation that lists what is kept (trusted sites, designer and maker fields, fields left alone, extra shop addresses, the detailed-condition table, language) and says the keys are untouched; on confirm it calls the reset command and shows the returned values.
7. Remove the "Automatic research" check box and the "Quiet period" field.

### Criteria (6 criteria, 13 rows, all `[rule]`)
Boundary: `ResearchSection` rendered in a UI test with `listDefinitions`, the settings save and the reset command mocked; outcome = what is shown and what the save or the command received.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A25·C28(a) | Designer and maker | a shop with five fields; `attribution_keys` holds two of them | all five are listed by label and key; exactly those two are checked | `[rule]` 08A §6 |
| A25·C28(b) | Designer and maker | `attribution_keys` empty; the shop has fields labelled "Designer" and "MANUFACTURER" and three others; then Save | both are checked and marked "suggested — not saved yet"; the save receives exactly their two keys | `[rule]` 08A §6 (owner 2026-11-13) |
| A25·C28(c) | Designer and maker | `attribution_keys` empty; no field has such a label; and, separately, a saved key unchecked | nothing is checked; the sentence "No designer or maker field is chosen: research will not suggest a designer or a maker." is shown in both cases | `[rule]` 08A §6 |
| A25·C28(d) | Designer and maker | `attribution_keys` holds one shop field and one key the shop does not have; the shop also has a field labelled "Designer" that is not chosen | the unknown key is listed with "Not one of your shop's fields" and can be unchecked; the "Designer" field is not preselected | `[rule]` 08A §6 |
| A25·C28(e) | Designer and maker | no shop linked (no definitions) | "Connect a shop to choose fields"; nothing preselected | `[rule]` 08A §2 (HA4) |
| A25·C29(a) | Fields research leaves alone | `excluded_keys` empty; check two fields; Save | none was checked at first; the save receives `excluded_keys` with exactly the two keys | `[rule]` 08B changelog 12 (a) |
| A25·C30(a) | Extra shop addresses | add `beyovintage.com`, add `x.myshopify.com`, remove the first; Save | the list shows the second only; the save receives `extra_shop_hosts` with that one entry | `[rule]` 08B changelog 11 (c) |
| A25·C30(b) | Extra shop addresses | Save is refused with a text naming an entry | the text is shown; the list is unchanged | `[rule]` 08B MC-A3 |
| A25·C31(a) | Detailed condition | the shop's Condition choice fields have the choices N/A, None, Minor, Moderate, Major; choose the field "Detailed condition"; set None and Minor to its second choice, Moderate to its third, N/A to "counts for nothing", leave Major unset; Save | five rows in that order; the save receives `detailed_condition` with the field's key and four levels in row order (N/A counting for nothing, no entry for Major) | `[rule]` 08B changelog 12 (b) |
| A25·C31(b) | Detailed condition | no field chosen | "Not set: Detailed condition is filled like any other field." | `[rule]` 08B changelog 12 (b) |
| A25·C32(a) | Reset | click the button; confirm; the command answers a value | the confirmation named the six kept things; the command was called once; the section shows the returned stages and prices | `[rule]` R10 review (carried) |
| A25·C32(b) | Reset | click the button; cancel | the command was not called; the section is unchanged | `[rule]` R10 review (carried) |
| A25·C33(a) | Behaviour section | any research settings; Save | there is no "Automatic research" check box and no "Quiet period" field; the saved value has neither `auto` nor `quiet_seconds` | `[rule]` 08B changelog 13 (d) |

### Delegated choices
Layout of the picker (a list with a filter box when the shop has many fields) · wording of helper lines the rows do not fix.

### Notes
- The owner's shop has 68 product fields: the picker needs a filter or a scroll area, not 68 loose check boxes.
- For the owner's shop the narrative lists what to choose: Designer and Manufacturer; left alone: total width, depth, height, seat height, weight definition, damage details; extra addresses from the evidence doc "Owner answers" 3; the detailed-condition table from the card's answer.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cd apps/desktop && npx vitest run src/features/settings` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
This milestone changed app files, so the full gate runs once: `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test` · `cargo test -p psm-research`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-research` · `cargo test -p psm-testkit --test suite research::` · `cargo test -p psm-testkit --test suite p03_session_state_list::` · `cargo test -p psm-testkit --test suite p13_publish_runs::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/settings src/features/shell src/features/sessions src/features/publish src/features/form` · clippy, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. The reviewer also checks that every test of "Tests that pin what this plan retires or changes" was handled as the table says, and that no deleted test took a still-valid row with it. Notes go to `docs/implementation/backlog_research/R11_research_on_arranged.md`.

## Manual try-out (the narrative is built from this)
On scratch data (`PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`), shop connected, keys entered:
1. Settings → Research. If this folder was used before R10, press **Reset research settings to defaults** and confirm: models and prices appear, your trusted sites are still there.
2. Under **Designer and maker** the fields named Designer and Manufacturer are ticked as a suggestion; save. Under **Fields research leaves alone** tick the dimensions, weight and damage details. Under **Extra shop addresses** add your other addresses. Under **Detailed condition** choose the field and fill the table (see "Needs you").
3. Create two sessions and shoot a few photos in each. Nothing is researched while photos arrive: the form's research panel says "Research starts when you press Arranged — next."
4. In the first session sort the photos and mark the condition photos, then press **Arranged — next**. The second session opens (captures now go to it). Open the first session's form: steps appear; no width, weight or damage details are suggested; Detailed condition follows the levels.
5. Change a photo in the first session: nothing starts. Press **Arranged — next** there again: a new run starts and only looks again at what changed.
6. In the second session press **Publish** without arranging it: the popover says research is still running; wait until the line goes, or confirm at once to publish without it.

## Review log
