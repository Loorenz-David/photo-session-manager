---
plan: P25_tidy_up_after_provisioning
role: plan
round: 1
state: DONE
date: 2026-11-04
actor: Claude Fable 5.1 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# P25_tidy_up_after_provisioning + P26_frontend_leftovers — planning handoff, round 1

**Summary.** Gate passed: `docs/intention/intention.md:3` reads **RATIFIED** (owner, 2026-10-24); the mechanism inventory is done
(`handoffs/reviewer/handoff_intention_inventory_r1.md`); the owner's two decisions this planning rests on are in the book (10C step 5
"Default empty", changelog 26 — written by the orchestrator 2026-11-04, present in the working tree, not yet committed) and in the
owner's "go" of 2026-11-04 relayed by the orchestrator. I read flow.md, the master plan, STATUS.md, the P21/P22/P24 plans and the P24
narrative and Review log, backlog rows 11/15/69/106/107/133/157/164/167/168/173, book 10C, 05A MC-3, 07A MC-11, 10B MC-25 §Failures,
ch12, evidence VERIFY-14 and SV-2, and the code behind every item (cited `file:line` at `c90b95d` in the plans). **Every one of the ten
owner items is still present in the code; none was dropped as already fixed.** Two plan files written (P25: 8 criteria / 12 rows, 4
`[rule]`; P26: 2 criteria / 2 rows, 2 `[rule]`), `phases.txt` appended, master plan amended, backlog rows marked. Not committed (as
instructed). No owner card.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. Every item either follows a decision already in the book (10C step 5; MC-26 step 3; MC-3; MC-11; MC-25) or is a
defect fix with one consistent reading, recorded in the plans' Notes.

## Write perimeter
- `docs/implementation/plans/P25_tidy_up_after_provisioning.md` (new).
- `docs/implementation/plans/P26_frontend_leftovers.md` (new).
- `docs/implementation/phases.txt` (two lines appended in place with `printf >>`: `P25_tidy_up_after_provisioning`, `P26_frontend_leftovers`).
- `docs/implementation/master_plan.md` (§4 two NOT_STARTED rows; §5 count 26, graph tail `→ P25 → P26`, `P25[P24] · P26[P25]`; §6.4
  `requeue_failed_jobs(kind, exclude_error, now_ms)`; §6.6 `Integrity` NULL-hash rule and the `document_not_front` clearing rule; §6.7
  `default_document_id`; §6.8 `style_name` default `""`; §7.5 the fake's `styles` default kept; §9 log entry).
- `docs/implementation/backlog.md` (first cell prefixed `→ P25 · ` on rows 11, 15, 69, 106, 133, 164, 167, 168, 173; `→ P26 · ` on row 157;
  row 107 marked `→ P25 (first clause: media id by filename; the P09 staging sweep stays open) · …`).
- `docs/implementation/handoffs/reviewer/handoff_P25_tidy_up_after_provisioning_plan_r1.md` (this file).
- **Not touched by me:** `docs/intention/**` (the two modified book files in `git status` — `10C_capture_one_setup_by_the_app.md`,
  `16_changelog.md` — are the orchestrator's uncommitted owner-decision edits of 2026-11-04 and predate this session), other plan files,
  `STATUS.md`, narratives, any code, any tool-recorded state. No commit, nothing pushed, no pipeline run.

## The plans in one table
| Phase | Scope | Rows | depends_on |
|---|---|---|---|
| `P25_tidy_up_after_provisioning` (difficulty high, projection no) | **C1** `open_document.js` closes the `PSM Open Window` throwaway and throws when Capture One's `open` returned without replacing it (name-prefix check on the front document; `null` front = success) — ensure then reports `document_not_open … : <error>`; **C2** `document_not_front` clears on any editing check while no session is active and on `deactivate` (a refused activation still sets it — P24 C2(b) stands); **C3** `settings_get`/`SettingsDto` gain `default_document_id` (= `<home>/Pictures/Session Manager`, constant); **C4** `Settings::default().style_name = ""`, explicit values kept; **C5** `IntegrityCheck::verify` hashes a NULL-hash photo through `set_raw_hash` and reports `Ok` (verify, publish prepare, export rows); **C6** `open_logged(paths, open)` in the desktop crate: `paths.create()` → `logging::init` → `App::open`, plus an `app open` log line at the top of `App::build`; **C7** `Store::requeue_failed_jobs(kind, exclude_error, now)`: a health check re-queues `crop_ratio_unknown` failures only when the ratio check cleared that item (order: count → editing checks → re-queue); **C8** `ShopifyAdapter::product_create_media` polls the `newest` media to `READY`, compares its CDN filename with the staged stem (MC-11 matcher shared with `PublishService::name_matches`), falls back to the gallery's first match, errors when none; **task 9** backfills the five P24/P17 assertion gaps of backlog 167 | 8 criteria / 12 rows (4 `[rule]`): C1 ×1 (Node harness, three modes), C2 ×2, C3 ×1, C4 ×2, C5 ×3, C6 ×1 (desktop crate, fake app), C7 ×1, C8 ×1 (four scripted runs over the captured SV-2 fixtures) | [P24_capture_one_provisioning] |
| `P26_frontend_leftovers` (difficulty high, projection no; thin, refine at prompt time) | **C1** the Settings detail line reads `settings.document_id \|\| settings.default_document_id` (no `~/` literal); **C2** `useWorkspaceState.run` decrements the in-flight counter first and reloads only when it reaches zero (the last of two overlapping drags reloads once) | 2 criteria / 2 rows (2 `[rule]`) | [P25_tidy_up_after_provisioning] |

## Why two phases
Ten items make 10 criteria in one file, over flow §9's "≤ 8 criteria" rule and the charter's sizing lever; the split follows P21/P22
(backend / frontend halves), and P26 needs P25's `default_document_id` binding, so it depends on P25.

## Grounding worth knowing (orchestrator)
- **Item 1 is grounded live, not by prose:** VERIFY-14 (open replaces the front window; the throwaway form) and the P24 Review log r1/r2
  (`currentDocument()` read `null` after a successful open; a missing file returned success with the throwaway in front). The harness
  models exactly those three post-`open` states. No live row: the fix is in the script's control flow, and flow §6A keeps the suite off
  Capture One. If the reviewer wants a live variation, the P24 C6(a) procedure (scratch session, restore) applies.
- **Item 9 (`product_create_media`) is grounded by SV-2:** the create reply carries no filename while `PROCESSING` (`image.url` and
  `originalSource.url` are `null`), so the adapter must poll before it can compare — one extra `MediaPoll` per new media (publish's own
  poll then returns at once). The gallery `alt` stays the item title (changing it to the filename would be an owner choice; noted, not
  made). The C8 fixtures are the captured SV-2 replies with ids/URLs taken from the captured three-media gallery (`sv2_item_get`), never
  invented values; `FakeCommerce` is unchanged, so P13/P14 rows are unaffected.
- **Item 5 (NULL hash):** `TestApp::add_photo` stores a fake hash (`hash-<raw>`), so the P25 rows insert `NewPhoto { raw_hash: None }`
  through the p01/p09/p13 helper shapes. `set_raw_hash` writes only NULL rows, so the later hash job is a no-op; p01 c4b/c4c/c4d, p09
  c5a and p13 c3d keep their meaning (they all store a real hash first).
- **Item 6 (logging):** `tracing` allows one global subscriber per process and `logging::init` ignores the error, so C6(a) must stay the
  only desktop-crate test that initialises logging; the plan says so and asks the Review log to record it. The row uses the p00 c1b
  fake-app construction (`App::open_with(.., FakeCaptureOneScripting, ManualClock, false)`), never `App::open` (P24 F1).
- **Item 8 (re-queue):** P23's c3c (`provider` failures re-queued on every check, counted first) and c3d (corrected setting → re-queued on
  the next check) must stay green; the plan's ordering (count → editing checks → re-queue with the exclusion) preserves both. A real
  start still re-queues everything once.
- **Tests that change their expectation** (to be named again by the implementer): `p00_spike.rs:78` (`""`), `p24 c1f` (saves
  `style_name: "PSM House"` first), `p12 c6c` (one `MediaPoll` reply added), the six frontend `Settings` fixtures (`default_document_id`).
  Task 9 only adds assertions.
- **The owner's own `settings.json` holds `"style_name": "PSM House"` explicitly** (written from the old default). P25 changes the default
  only; the Style row stays red on the dev Mac until the owner clears the field (P25 try-out step 1; the P24 narrative already said so).
- **Left out on purpose:** backlog 107's second clause (P09's stale `.partial-*`/`.old-*` sweep) — not trivial; the row stays open with
  the first clause marked. Which Health actions re-queue (`health_get` = "Retry checks") is unchanged.

## Choices the owner may want to know (recorded in the plans' Notes)
1. A refused activation still shows the red "Capture One document" row and blocks capture until the next check or until the session is
   closed; it no longer waits for a successful activation.
2. A photo nobody has hashed yet is hashed at the moment of export or publish (same algorithm, same stored value), never refused.
3. Publishing makes one more small Shopify read per new photo (the "is it ready?" poll moved earlier) so the app never trusts gallery
   position to know which media it just created.
4. A fresh install applies no Capture One style; naming one in Settings is the photographer's choice.

## Lessons for the planner
- A "clear" rule for a health item needs the same enumeration as a "set" rule: name every event that lapses it (check with none active,
  deactivation, success), or the item outlives its fact.
- When an external reply is captured in fixtures, scripted "variation" rows should reuse real sibling nodes of the same capture (here the
  gallery's three media) rather than minting ids — the shape stays one the vendor produced.
