---
plan: R11_research_on_arranged
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-15
actor: opus
---
# Re-review — R11 Research starts when the session is arranged (review r2, after fix 1)

**Summary.** F1, the only blocking finding of r1, is resolved as its correction said. The fix changed only the permitted test. All checks are green. No blocking item remains, so there is nothing to triage. The 8 r1 notes stand in the backlog. Verdict `APPROVED_WITH_NOTES`. The tracker is set to `APPROVED` and the narrative is written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
r1 (`3b93141`) settled U1–U6: engine trigger, settings, derived condition and look-alike rule, app seams, and UI. It also settled H1–H8 and the retired-tests table. All Review-mode commands were green. It found one blocking item (F1, B2): A25·C23(a) did not observe `session_arrange_next` reaching the engine. It wrote 8 notes. This round covers only the F1 delta.

**Gate.** The tracker showed `IMPLEMENTED` with the fix note. Checkpoint `80adfc6` exists. The tree was clean at `8421c5d`. The commits after `80adfc6` change only pipeline docs: the fix handoff SHA line and this round's prompt.

## Perimeter check
`git show --stat 80adfc6`:
| Path | Allowed by | Verdict |
|---|---|---|
| `apps/desktop/src-tauri/src/lib.rs` (+15/−3, inside `a25_c23a_new_commands_reach_wired_app_and_bindings` only) | F1 "Files the fix may touch" | ok |
| `docs/implementation/plans/R11_research_on_arranged.md` (Tracker line, Review log entry) | fix role's own rows | ok |
| `docs/implementation/handoffs/implementer/handoff_R11_research_on_arranged_fix_r2.md` | fix handoff | ok |
Nothing outside the perimeter. No production file changed.

## Findings resolved
**F1 (B2): resolved as corrected.** The test now does the following:
- classifies its photo `Main`;
- calls `session_arrange_next` with the test prices intact;
- asserts `next_session_id == null`, `research_queued == true` and `arranged_at` set;
- asserts that `store::runs(…, Some(s))` holds exactly one run, with state `Queued` and trigger `"arranged"`.

Only `ResearchApi::arranged` creates run rows. A command that marked through core and never called the engine would now fail on both `research_queued` and the run count. The fixer probed the test with a changed input (photo classified Condition) and it failed at `research_queued`. That is the right variation: the engine refuses for lack of a Main photo.

Changed-seam reading:
- The emptied price table is now written with `Settings::open` + `Settings::set` after the arrange call. These are the two calls `TestApp::set_research_settings` makes (`crates/testkit/src/test_app.rs:357`). `settings_reset` reads live settings at call time (`api_impl.rs:76`). The reset half keeps the meaning it had in r1.
- `AppState::new` starts no worker thread. The `Queued` assertion cannot race a background tick.
- The bindings checks are unchanged.

The declared judgment call (writing the file directly) is sound.

**Checks (run once each on `8421c5d`, one cargo command at a time):**
| Command | Result |
|---|---|
| `cargo test -p psm-desktop` | green, 25 |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `cd apps/desktop && npm run typecheck` | green |
F1 named no `psm-testkit` suite stem. The fix touched no other crate. No full suite was run: the change is test-only, and the implementer's full-suite stamp from r1 stands.

## Triage
| Item | Class | Reason |
|---|---|---|
| none | — | no blocking item remains after fix 1 |
No MUST, no DROP. No new item is added to the backlog.

## Remaining findings
None blocking. Nothing was seen wrong in passing on the changed seam.

**Carry-forward dispositions** (the r1 notes in `docs/implementation/backlog_research/R11_research_on_arranged.md`):
| r1 note | Destination |
|---|---|
| 1. C22(a) status clause cannot fail (`"price_missing"` vs `"PriceMissing"`) | next touch of `r11_arranged.rs` (R12); plan lesson for the orchestrator |
| 2. C24(d) Unassigned half checks before the list loads | backlog, next touch of `r11_arranged_button.test.tsx` |
| 3. C33(a) saved-value half asserts the fixture | backlog, next touch of that test |
| 4. C7(a) partial default check | backlog, next touch of that test |
| 5. C9(c) sets the shop host directly | backlog, next touch of that test |
| 6. Stage 7 decides a derived-only field past the cap | orchestrator: fold into 08B MC-A6/MC-A7 (named in the narrative's choices) |
| 7. Publishing marks before the preview can fail | orchestrator: state the order in 08B / the publish chapter |
| 8. Declared file lists differ from commits | process lesson for implementer handoffs |

## Lessons for the plans
- None new. Both r1 lessons stand: say which provider ids a wired-app "no PriceMissing" row is priced against, and name the observable that proves a command reached the engine. This round confirms the second lesson: the corrected test is exactly "response says queued, and a run row exists".

## Probe declaration
No probe. No production or test file was touched, and no database or settings state.

## Files written by this review (full write perimeter)
- `docs/implementation/handoffs/reviewer/handoff_R11_research_on_arranged_review_r2.md` (this file)
- `docs/implementation/plans/R11_research_on_arranged.md`: Tracker line → `APPROVED`, and the r2 Review log entry
- `docs/implementation/narratives/R11_research_on_arranged.md` (new)

No backlog change. No archgraph delta. No commits.
