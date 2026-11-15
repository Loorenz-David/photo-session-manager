---
plan: templates
role: plan
round: 1
state: OWNER_DECISIONS_PENDING
date: 2026-11-16
actor: Claude Fable 5.2 (pipeline-planner, implementation-planner doctrine; orchestrated)
---
# Shooting templates — plans for T1–T4 (book 10G)

**Summary.** Four milestone plans in the format of R13, a short master plan and the phase list. Gate: the intention map is RATIFIED (2026-10-24) and lists 10G; 10G is RATIFIED (owner, 2026-11-16 00:20); the mechanism-inventory gate and projection are waived for this chapter by its own §10 (owner). The one silent-failure mechanism (shot order) has its contract in `master_plan_templates.md` §5 and an enumerated row table (T2·C1). Documents only: no code, no cargo, no external call, no commit. One owner card; it blocks nothing (built as recommended on silence, as the launch instruction asks).

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1 — dragging a placed photo onto another filled container**

**Question** — When you drag a photo that already sits in a container onto another container that is filled, should the two photos swap places (A), or should the photo that was there be deleted (B)?

**Story** — On a chair you grab the split photo in place 3 and drop it on place 2 by mistake. With B the good photo in place 2 turns red, and Back to template only brings the split photo home; you also have to undo to get the other one back. With A both photos get a small mark, and Back to template puts both back.

**Branches** — **A**, swap: nothing is deleted by a slip; a photo dragged in from Condition (a real retake) still replaces the old one, which is deleted. **B**, always delete: every drop on a filled container deletes what was there.

**Recommendation** — A, because your own story expects Back to template alone to repair the slip.

**On silence** — A is built tonight (you opened the process for this chapter); changing to B afterwards is one rule in one place.

**Trace** — 10G §6 rule 3 against §8 A6; T3·C2(b), T3·C2(d), T4·C1.

## Write perimeter (complete)
| File | Change |
|---|---|
| `docs/implementation/master_plan_templates.md` | new |
| `docs/implementation/phases_templates.txt` | new |
| `docs/implementation/plans/T1_templates_and_settings.md` | new |
| `docs/implementation/plans/T2_template_placement.md` | new |
| `docs/implementation/plans/T3_template_hand_changes.md` | new |
| `docs/implementation/plans/T4_template_retake_and_shift.md` | new |
| `docs/implementation/handoffs/reviewer/handoff_templates_plan_r1.md` | this file |
No code, no test, no book chapter, no `STATUS.md`, no `master_plan.md`, no tracker line of another plan, no archgraph state, no commit. Nothing read or written outside the worktree except the doctrine files under `~/agent-skills/`.

## Counts (computed from the plan files: rows = lines starting `| T<n>·C` in the criteria tables)
| Phase | Unit | Criteria | Rows |
|---|---|---|---|
| T1 | U1 tables, service, commands | 4 | 14 |
| T1 | U2 Settings as a drawer | 1 | 4 |
| T1 | U3 the templates editor | 1 | 8 |
| **T1** | | **6** | **26** |
| T2 | U1 shot order and the other pure rules | 2 | 13 |
| T2 | U2 placement on arrival, presets, view, hand changes kept | 5 | 20 |
| T2 | U3 the session view | 1 | 5 |
| **T2** | | **8** | **38** |
| T3 | U1 differs; order after an undo | 1 | 7 |
| T3 | U2 drop onto a container | 1 | 6 |
| T3 | U3 Back to template; commands | 1 | 7 |
| T3 | U4 warning, marks, drop in the view | 1 | 4 |
| **T3** | | **4** | **24** |
| T4 | U1 the retake | 1 | 4 |
| T4 | U2 Deny; Shift the rest back | 1 | 7 |
| T4 | U3 button, story test, guide | 2 | 3 |
| **T4** | | **4** | **14** |
| **All** | 13 units | **22** | **102** |
T2 is above the 15–30 rows asked for: 10 of its rows are the ordering table (one line each, pure function), kept because ordering is the one place a wrong rule would be silent.

## What was settled from the code (the six points of the task)
1. **Shot order key** (`master_plan_templates.md` §5 P2–P4). `photos.captured_at` is the matched exposure's `exposed_at` for a direct-camera file (`delivery_for_path`), the file's mtime for tether and import. The ledger matches a file to the oldest expecting exposure (`capture_ledger::associate`, FIFO), so two files that arrive swapped carry each other's times; only the camera counter (`capture_ledger::seq_of` on the delivery's `file_key`) tells them apart. Key: time gives the slots; among counter-bearing files within `FIFO_WINDOW_MS` of each other the counter assigns files to slots (restart rule for the wrap; RAW + JPEG share a stem and only the RAW is a photo); an `expecting` exposure of the session with no file yet, younger than the window, holds the slot the counter leaves open, else the last slot. Tether and imports: by `captured_at`. Edge cases are stated in §5 "Edges" and in T2's Points.
2. **Hook:** `ArrivalService::accept`, right after `arrival_insert` reports a new photo and before `photo_added`. Positions stay dense (MC-5 untouched); the container is a new column (`photo_places.slot`), Main is ordered by it. Presets are expressible: `CompositionKind::Split` and a fraction `Rect`; zoom = the default rect scaled about its centre. **Finding:** the frame is unknown when a photo arrives (the composition row has no rect until `adopt_frame`), so a zoom cannot be computed then; the plan stores `pending_zoom` and `adopt_frame_tx` applies it in its first branch (it already honours a stored kind and shape).
3. **Storage:** numbered migrations `014` (templates, T1) and `015` (`photo_places`, T2); next free number verified (013 is the last); research's ensure steps are outside the numbered list. SQLite, not `settings.json` (reasons in master plan §4). No per-session template link: looked up by item type at each use. Both files need `IF NOT EXISTS` (`p27` rewinds `user_version`).
4. **Settings UI:** one `<form>` with one Save plus three self-contained sections. The drawer keeps every pane mounted and hides the inactive ones by stylesheet, so the form state, the Save and the leave guard are untouched and the 16 existing settings tests should not need edits (vitest loads no stylesheet). Item types: `useItemTypes` / `ItemTypePicker` (the shop's product types as suggestions in a free text box; works without a shop). Bindings: `cargo test -p psm-desktop export_bindings`.
5. **Undo:** a new `StepKind::Template` (T3) carries, per photo, place, `photo_places` row, composition and denial; Back to template, a drop, a retake and Shift the rest back are each one step; arrival placement records nothing. `HistoryService::reverse` renumbers Main in templated sessions after any older step.
6. **Research:** nothing in `crates/research` changes. A session is templated only when a template exists for its type, and no existing test creates one, so every existing test keeps P66 placement.

## Where the code or the chapter needed a decision (no card; each is in a plan's Points)
- A denied photo keeps counting as a shot until Shift the rest back (10G §4.2's "not deleted by a hand action" read with §6.4's "later photos do not move"; otherwise the next arrival would land on a filled container).
- A retake takes the replaced shot's place in the count, so it matches the template and no warning follows it.
- Deny marks nothing as "differing"; Back to template never restores a denied photo.
- Place-marking and the Compose mark are built in T2, not T3, so that a build with T1–T2 only never undoes a hand change.
- A template changed mid-session applies at the next arrival to photos the template placed (10G §7 says a changed template applies to new arrivals; a late arrival re-places by the current template).
- Example images: copied as JPEG ≤ 1600 px through the existing `Imaging::thumbnail`; black and white is a display rule in one component; HEIC and RAW are refused with a sentence.

## For the orchestrator
- `tools/pipeline/run_phase.sh` needs no change: `phase <id>` resolves `plans/<id>.md`; frontmatter of the four plans parses with its reader (checked with the same split rule). All `difficulty: high`, `projection: no`, no `implementer:`.
- 10G §4.1 asks the planner to state the exact key: it is `master_plan_templates.md` §5. If you want it in the book, fold P1–P10 into 10G as a lettered section.
- Card 1: if the owner answers B before T4 starts, write it into T4's Points (the swap case of `drop_on_container` becomes the retake case for every source). After T4 it is a small fix phase.
- T4·U3 updates `docs/guide/`; T1–T3 reviews may note a stale guide row (a note, not blocking).
- **Deadline risks:** (1) four closing gates at 10–17 minutes each plus the reviewer's checks; (2) T3 and T4 extend `services/history.rs` (1060 lines; undo is the most intricate code touched), the likeliest place for a fix round; (3) T2 edits `adopt_frame_tx`, the 10D path `p28`/`p58` guard (those stems are in T2·U2's unit commands); (4) the camera's behaviour is proven only against the fake ledger: whether the Sony reports each shutter in Wi-Fi mode and whether its counter looks like `DSC0nnnn` on the work Mac's camera is seen at the first real shoot (if no shutter is reported, a swapped pair is right once both files are in); (5) the release build must come from a separate worktree with the SDK sidecar (`p45` rebuilds the stub); (6) `p33_split_grid` C4(d) is intermittently red under load.
