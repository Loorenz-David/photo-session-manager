---
plan: P24_capture_one_provisioning
role: plan
round: 1
state: DONE
date: 2026-11-03
actor: Claude Fable 5.1 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# P24_capture_one_provisioning — planning handoff, round 1

**Summary.** Gate passed: `docs/intention/intention.md:3` reads **RATIFIED** (owner, 2026-10-24); chapter 10C carries the owner's
decision of 2026-11-03 as a contract (MC-26) with its ledger entry (M32), decision (R36) and changelog entry 24, and the map, ch10,
ch12, ch14, ch15 point at it. I read 10C in full, 01, 14, 10, 10A (MC-15/16/18/19), 10B, 12, 05A MC-2, 15, 16, the evidence file
(document properties, VERIFY-5/7, the read-only smoke, the crop probe), the sdef (Standard Suite `open`/`close`/`make`, `window`,
the Capture One `application` class with `documents`, `current document`, `available styles`, `document` with `template`/`path`/
`folder`/`kind`/`recipes`, `recipe`, `next capture options`, the four recipe enumerations), flow.md, the master plan, STATUS.md,
backlog rows 29/33/36/38/40, `real_run_2026-10-26.md` (the orchestrator's recipe setup), plans P21/P23 and their handoffs, and the
code behind every task (ports, domain, error, `EditingStartup`, `SessionsService`, `CameraService`, `HealthService`,
`SettingsService`, `AppPaths`, `Settings`, `OsaRunner`, the Capture One scripting trait, JXA runner and all 16 scripts, gate,
adapter, camera-via-Capture-One, `App::build`/`AppHandles`, the Tauri settings/health/sessions/items commands, `state.rs`, the
Settings and Health pages and services, the fake, `TestApp`, and the p04/p16/p17/p21/p23 tests this phase reuses or changes).
One plan file written (6 criteria / 15 rows, 3 `[rule]`, one optional live row), `phases.txt` appended in place, master plan
amended (§2, §3, §4, §5, §6.1, §6.3, §6.4, §6.6, §6.7, §6.8, §7.5, §9). No book hole that blocks; two readings recorded in the
plan's Notes. `docs/intention` untouched. One commit, not pushed.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner now. One **conditional** card is drafted in the plan's Notes ("STOP rule") for the case task 0 shows that
Capture One closes the photographer's open document when the app opens its own; the implementer (or the orchestrator running task
0) relays it only if that happens.

## Write perimeter
- `docs/implementation/plans/P24_capture_one_provisioning.md` (new) — committed.
- `docs/implementation/phases.txt` (one line appended in place with `printf >>`: `P24_capture_one_provisioning`) — committed.
- `docs/implementation/master_plan.md` (§2 contracts row + 10C, MC-1…MC-26, ledger M1–M32; §3 trace ranges; §4 one NOT_STARTED
  row; §5 count 24, graph tail `→ P23 → P24`, `P24[P23]`, the Apple-events precondition for task 0; §6.1 `provision.rs`/`recipes.rs`;
  §6.3 `AppPaths.home`; §6.4 `EditingProvider::{ensure, bring_document_to_front}` (`check_recipes` removed), `CaptureOneScripting`
  seven ops / `set_recipe_props` removed / `SCRIPTS` 21 / `RecipeProps` and the dependent recipe values; §6.6 `document_not_front`,
  seven health codes with details, new details of three existing codes; §6.7 `effective_document_id`, `document_id: ""`,
  `session_activate` order; §6.8 `document_id` default rule and the Settings wording; §7.5 fake and `TestApp` additions; §9 log
  entry) — committed. The diff was checked before staging: every hunk is mine.
- `docs/implementation/handoffs/reviewer/handoff_P24_capture_one_provisioning_plan_r1.md` (this file) — committed.
- Not touched: `docs/intention/**`, other plan files, `STATUS.md`, `backlog.md`, any code, any tool-recorded state. Nothing pushed;
  the pipeline was not run.

## The plan in one table
| Phase | Scope | Rows | depends_on |
|---|---|---|---|
| `P24_capture_one_provisioning` (difficulty high, projection no) | **task 0** live grounding probe (VERIFY-13/14 + recipe property read-back) in a scratch folder alongside the owner's open document, with the 10C stop rule; `document_id` missing/null/`""` → `<home>/Pictures/Session Manager` (`AppPaths.home`, `effective_document_id` in `settings_get`, `""` clears, explicit values honoured); the **ensure** sequence owned by the adapter (`provision.rs`, one lock, user lane): `editor_info`/`launch` (`open -g -a`) with a bounded wait → `make_document` never over a non-session folder → `open_document` (other documents never closed) → `ensure_recipe` ×2 in one JXA run each (create / repair only drifted dependent props: disabled, JPEG, sRGB, Long_Edge, pixels, 1600/80 and 2048/90, overwrite, `_session_manager`) → `ensure_capture_style` (non-blocking "Style not found") → hooks; seven plain-worded health codes, stale ok items cleared; `EditingStartup` = ensure → inspect → ratio → front when a session is active → folder; `activate` = ensure → `bring_document_to_front` → MC-18 switch, `document_not_front` blocks capture like an unverified folder; Settings block "Session Manager (created by the app)" + "Use another session…" (`.cosessiondb` picker, parent folder stored) + "Use the app's own session"; Health labels; fake gains editor/documents/front/styles/`RecipeProps`; Node-harness rows for the six scripts; one optional live row that provisions a scratch document beside the owner's and proves theirs stayed open, then restores and deletes | 6 criteria / 15 rows (3 `[rule]`): C1 ensure ×6 (default created once + other document untouched, drift repair, launch/not installed/timeout, denied/not Pro, non-session folder/open failure/open success, style), C2 activation ×3 (front before folder, front failure blocks capture, re-asserted at every ensure while active), C3 setting ×1, C4 scripts ×2, C5 UI ×2, C6 live ×1 | [P23_shape_on_arrival] |

## Grounding worth knowing (orchestrator)
- **Task 0 needs Apple events.** The Codex sandbox refused them in P16 (`-2700`). If the implementer is Codex, run task 0 yourself
  before dispatch (the plan lists the steps (a)–(j); scratch folder only; owner documents read-only; restore the front; close and
  delete only the scratch document) and append the three evidence sections; the implementer then writes the five Capture One
  scripts from that record. Master plan §5 names this precondition.
- **The owner's Capture One currently has `instagram pictures new camaera` open** (`~/Pictures/instagram pictures new camaera/…
  .cosessiondb`). That is exactly the "alongside" case the probe and the live row must leave untouched. `~/Pictures/Session Manager`
  does not exist yet on the dev Mac; the dev Mac's settings still name `session test 1`, so try-out step 1 tells the owner how to
  switch to the default.
- **Externally owned shapes not yet in the evidence** (charter rule 17): the `make new document` form and where its files land; the
  front switch (`current document` has no `access` attribute in the sdef — read-write by default — but the setter is unverified;
  window `index` is the fallback); `open` with another document open; the sRGB `color profile` string; the recipe enumerator read-backs
  beyond VERIFY-5's `Long_Edge`/`overwrite`. All are grounded by task 0 before any script exists; C4's harness rows are written from
  that record ("per VERIFY-13/14 as recorded by task 0"). Verified today: `/usr/bin/open -g -a '<unknown>'` prints `Unable to find
  application named '<name>'` and exits 1. Not reproducible without harming the owner's setup: the `-1743` Automation refusal (format
  per osascript's standard error print; code per `MacErrors.h`) and "not Pro" (the dev Mac is Pro); both are classified by a pure
  function with a unit test and named in try-out step 7.
- **Tests that change their expectation** (declared in the plan's Files list, to be named again in the implementer's handoff): p04
  `c3a` (recipes asserted against `RecipeSpec`), p04 `c3b` (a recipe that *cannot be created* blocks only its render — the old
  "missing recipe" scenario no longer exists because missing recipes are created), p16 `c3a` ("not open" is now modelled by clearing
  the fake's `open_documents` and a failing open), p17 `c1a` (the op is `ensure_recipe:`), p17 `c1b` (a renamed recipe is created, not
  reported missing). Frontend `Settings` fixtures gain `effective_document_id`; the five `@/services/settings` mock factories gain
  `pickSessionFile`.
- **`TestApp::new()` now provisions a fake default document** (in the temp dir, via `AppPaths.home`) on every `health_check()`; P21
  C1(a) and friends keep passing because they assert settings items only. The fake's default style list is `["PSM House"]` so no
  existing test sees a spurious `style_not_found`.

## Choices the owner may want to know (recorded in the plan's Notes)
1. **`document_id` keeps storing the folder** (as every value so far does); the `.cosessiondb` is derived — the default is
   `~/Pictures/Session Manager/Session Manager.cosessiondb`, and "Use another session…" stores the picked file's folder.
2. **A missing style does not stop the setup**: the recipes and hooks still get done, Health shows "Style 'PSM House' not found in
   Capture One" until the photographer creates the style or clears the Style field. The default name `PSM House` therefore shows a
   red Health row on a fresh Mac until one of those happens; an empty default would avoid that (one line to change if preferred).
3. **The app brings its document to the front** when a session is opened and at every re-check (startup, settings save, Retry
   checks) while a session is active; it does not watch what the photographer clicks in between. If the front cannot be verified,
   capture is blocked with a clear message rather than risking a frame in another session.
4. **The whole setup also runs before every session activation** (not only "open and front"), so a Capture One that was quit in
   between comes back with its hooks and recipes — at the cost of one or two read-only calls when nothing changed, or up to 45 s
   when Capture One has to start.
5. **Capture One is launched in the background** (`open -g`), so the Session Manager window keeps the focus; whether the front
   switch later steals focus is recorded by the probe.
6. **"Folder exists but is not a session" is never overwritten**, not even when the folder is empty; the Health row names the folder.
7. **The app creates `Capture/` inside its own session if it is missing**, nothing else on disk.
8. **Projection stays off**: task 0 grounds the external shapes live before any code, which is stronger than a paper projection
   (P17/P23 precedent).

## Lessons for the planner
- When a chapter's default names a *file* and the contract's identity is a *folder*, settle the storage shape in the plan before
  the implementer meets both (here: folder stored, file derived) and say so in Notes rather than touching the book.
- An invariant worded as a continuous state ("frontmost while active") needs the plan to say at which events it is asserted; a
  mechanism that only acts at one event would otherwise be read as satisfying the invariant by accident.
