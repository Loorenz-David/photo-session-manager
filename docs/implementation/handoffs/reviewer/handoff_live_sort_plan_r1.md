---
plan: live-sort
role: plan
round: 1
state: DONE
date: 2026-11-27
actor: opus (Claude Opus 5.5, pipeline-planner)
---
# Handoff — plan "sort and arrange on the phone" (book 10J LV-D20) → LV8_phone_sort_and_arrange

**Summary.** Gate passed: book 10J is RATIFIED (D18) and LV-D20 is an owner decision recorded in changelog D20 (commit
`f7d41079`). I read the phone code (Dock, Preview, Move drawer, PhotoImage, state, api, gestures, the test helpers), the
phone routes, `template_placement::move_to`, `CurrentShots`, `ReviewService::reorder` and book chapters 06 and 10G §6,
then wrote **one milestone**, `LV8_phone_sort_and_arrange` (`implementer: fable`, `difficulty: high`, projection off):
4 units, 15 criteria, 70 rows. The small server part is **U1 of the same milestone**, not a separate `LV8a`: it is one
core function (`classify_to`) and two move variants beside `move_to`, one route and two DTO variants, no camera code
and no schema; `HandAction::finish` already makes the PH8 announcements (reason recorded in the plan and master plan
§3). `master_plan_live.md` gets dated v1.2 amendments: **PH4a** (the classify route, the insert moves) and **PH9** (the
phone's swipe, lift-and-drag state machine, drop targets, cancels, the drop shown at once and its three ends), plus
§2/§3/§4.2/§4.3/§4.5/§4.6/§4.7/§6/§9. `phases_live.txt` gets LV8 appended. No code changed; nothing committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None — nothing needs the owner before LV8 runs. What LV-D20 leaves open is settled from the book or as the smallest
consistent choice, each listed in the plan's "Points recorded by planning" (P1–P12) for the narrative's "Choices made
for you". The two the owner is most likely to want to change, each a one-sentence answer: **P6** (a template shot's
photo sent to Main makes its shot current, as Deny and Condition do; LV-D20 says so only for those two) and **P7**
(across groups: onto a template shot swaps as the Move drawer does, card LV8; into the other group goes to its end,
book 06; a group with no photos cannot be dropped into, the swipe does it).

## Write perimeter (complete)
| File | Change |
|---|---|
| `docs/implementation/plans/LV8_phone_sort_and_arrange.md` | new (Tracker `NOT_STARTED`, empty Review log) |
| `docs/implementation/master_plan_live.md` | v1.2 header note; §2 rows LV7 (recorded) and LV8, totals by grep; §3 LV8 implementer and server-unit reason; §4.2, §4.3, §4.5 names; §4.6 `by=classify`; §4.7 stems and changed tests; §5 PH4a, PH9; §6 texts and haptics; §9 sequencing. Nothing earlier superseded. |
| `docs/implementation/phases_live.txt` | appended a comment line and `LV8_phone_sort_and_arrange` |
| `docs/implementation/handoffs/reviewer/handoff_live_sort_plan_r1.md` | this file |
Not touched: code, the book, `STATUS.md`, `STATUS_live.md`, other plans' Tracker lines, other roles' prompts and
handoffs. No archgraph in this repo. No commit.

## Counts (derived by `grep`/`awk` from the plan file)
| Unit | Criteria | Rows | What |
|---|---|---|---|
| U1 | 4 (C1–C4) | 14 | the Mac: `POST /photos/<id>/classify`, move kinds `extra_insert`, `condition_insert` (Rust, route boundary through `TestPhone`) |
| U2 | 4 (C5–C8) | 22 | the Preview: swipe vs hold vs tap, outcomes, the confirmation, the strip, no Delete |
| U3 | 5 (C9–C13) | 22 | lift, drag, drop within and across groups, auto-scroll, the Preview's strip |
| U4 | 2 (C14–C15) | 12 | cancels; the drop shown at once; rollback on refusal and after 8 s |
| **LV8** | **15** | **70** | max 5 criteria per unit |
Master plan §2 totals recomputed by the same grep over all eight live plans: 131 criteria, 402 rows (LV3's own table
says 85 rows and LV6's 13; grep finds 83 and 14 — the plan files are the truth; recorded, not corrected, outside my
lane).

## How LV-D20's asks map to rows (for the coordinator)
| LV-D20 | Rows |
|---|---|
| (1) swipe left Deny (template shot → Retake semantics, shot empty and current) | C6(a), C6(b), C5(f) rotated |
| (1) right = Main; Condition → Main at end; extra → nothing; template shot → confirmation | C6(c), C6(f), C7(a)–(c), C1(a)–(c) |
| (1) up = Condition; template shot → Condition, shot empty and current | C6(d), C6(e), C2(a)–(d) |
| (1) next photo after a swipe; thresholds; feedback; cancel when short; swipe vs hold | C5(a)–(e), C6(a)–(e), C8(d) |
| (2) the Preview shows the strip | C8(a)–(c), C8(f), C13(a)–(b) |
| (3) press-hold-lift, drag, drop; tap and scroll kept | C9(a)–(d), C12(d) |
| (3) swap between template shots | C10(a), C10(b) |
| (3) insert and shift among extras / Condition | C10(c)–(e), C3(a)–(d), C4(a) |
| (3) across groups (book 06, card LV8) | C11(a)–(g) |
| (3) auto-scroll at the edges | C12(a)–(c) |
| (3) reliability: state machine, cancels, optimistic + rollback | C14(a)–(f), C15(a)–(f); PH9.2–PH9.5 |
| (4) no Delete button; Retake stays | C8(e) |
| H8, PH guard, PH8 announcements | C1(a), C3(a), C4(b), C4(c); review by reading (order: commit → current → announce) |

## Findings and hazards recorded in the plan (for the implementer and reviewer)
1. **`ReviewService::reorder` is unusable for the phone's insert:** it marks the photo by hand with `slot = position`
   (`crates/core/src/services/review.rs:114`ff), i.e. it would put an extra into a container. The insert moves are new
   `MoveTarget` variants inside `move_to`'s phone model (PH4a).
2. **The orphaned-Preview effect** (`PhoneApp.tsx`: a Preview whose photo left the session closes) would close the
   Preview after a Deny if the next photo were shown only after an animation; rows say "at once" (U2 Notes).
3. **Measuring drop targets from live rectangles oscillates** in a real browser (`getBoundingClientRect` includes the
   slide-aside transforms); PH9.3 fixes homes at lift. The jsdom layout stub ignores transforms, so no test can catch
   this — the reviewer checks it by reading (Review mode).
4. **The Preview's hold must now wait 200 ms** (else every swipe flashes the template); `lv3_c21a`/`lv3_c21b` change
   accordingly; `lv3_c22c` (Delete button) and `lv6_c2c` (long press = tap) are removed; `lv6_c5a` re-scoped. All
   listed in the plan's Files sections and master plan §4.7.
5. Two test-harness additions are planned (helpers: `cancel`, a pointer id argument, the scroller's `scrollWidth` /
   `clientWidth`); each has callers in LV8's rows.

## Notes for the coordinator
- `depends_on: [LV7_tile_loading_cue]` (APPROVED 2026-11-27). Start: `tools/pipeline/run_phase.sh phase
  LV8_phone_sort_and_arrange`.
- No external dependency shape at any row's boundary (rule 17): the server rows use this repo's routes and store; the
  UI rows use the phone's own wire through the existing fake Mac; jsdom's missing `PointerEvent` is handled as LV3/LV6.
- Flow log v1.3 ("no new plan may carry `implementer: fable`") is superseded for this track by the owner's 2026-11-26
  rule recorded in master plan §3 and by this planning prompt.
