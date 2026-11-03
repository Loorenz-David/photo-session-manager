---
plan: P24_capture_one_provisioning
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-03
actor: opus
---
# Review P24_capture_one_provisioning, round 1: CHANGES_REQUESTED

Gate: the tracker showed `IMPLEMENTED`, and checkpoint `1a4d277` exists. `git show --stat 1a4d277` lists exactly
the 45 paths the handoff declares. No path is extra or missing.

## Verified correct
- **Build:** `cargo test --workspace` passed 445 tests; `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo fmt --all -- --check`, `npm run typecheck` and `npm test` (129 tests) all pass. No B4.
- **Ensure sequence (`provision.rs`).** It runs on the user lane under `provision_lock`, in this order:
  editor → document → open → recipes → style → hooks. Each step's failure sets its own plain-worded item and clears
  the ok and failure items of the later steps. A missing style reports and continues; the other failures stop.
  `classify_editor_error` is the only unit-tested function, as the plan's Core mechanisms list says.
  Waits are bounded by `inputs.wait`, polled every `min(500 ms, wait/4)`.
- **Callers.** `EditingStartup::run` = ensure → inspect → ratio check → front (only while a session is active)
  → capture folder. `SessionsService::activate` = ensure → front → inspect → `set_capture_folder` → store (MC-18
  order kept). `CameraService` gates `status`/`capture` on `document_not_front`. Tauri commands stay thin and call
  through `spawn_blocking` (H8). Core gains no Capture One names (H7).
- **Settings.** `document_id` `""` → stored `null`; a relative value → `RelativePath`; `effective_document_id`
  falls back to `$HOME/Pictures/Session Manager`; `SettingsDto` and the bindings carry it. The
  "No Capture One session configured" item is gone.
- **Rows.** C1(a)–C1(f), C2(a)–C2(c), C3(a), C4(a), C5(a) and C5(b) each have a test at the row's boundary
  that fails when the row's main outcome is wrong. Clauses that are not asserted are listed in a backlog note;
  none blocks.
- **Implementer deviations, accepted.** C4(b): `make` without `kind`, as VERIFY-13 and the plan's task-0 section
  require. C4(a): eight `changed` names on creation, not nine; a fresh recipe is already `enabled false`, and the
  live run below confirmed eight.
- **Assumptions.** All four are consistent with book 10C and the plan: `home` from `$HOME`; the folder-named
  `.cosessiondb` first, then name order; the throwaway open window; an empty style leaves Capture One's setting
  alone. None is B1.
- **Hard constraints H1–H8.** No violation in production code. H2: the only deletion is the app's own throwaway
  folder under `hooks/jxa/`. H5: only the dictionary, plus `open -g -a`. H4: the second ensure writes nothing
  (C1(a)). H7 and H8 hold. The suite's reach into live Capture One is F1.
- **C6(a) live (reviewer, Capture One 16.8.6.25, scripts built from `lib.js` + each script, run from a temp dir
  outside the repo).**
  1. `make_document.js` returned `D`, about 3 s; on disk: `.cosessiondb`, `Capture`, `Selects`, `Output`, `Trash`.
  2. The owner's `instagram pictures new camaera` stayed open beside it.
  3. `ensure_recipe`:
     - existing `PSM Preview`/`PSM Web` (application-wide): `created false`, `changed []`, props equal to the spec,
       `rootFolderType 'output location'`;
     - a scratch `PSM Live Probe`: `created true`, eight `changed`, props equal to the spec; second run
       `changed []`; then deleted, and no `Untitled Recipe` was left.
  4. `front_document.js` switched the front both ways and read it back.
  5. `ensure_capture_style`: `Bright Contrast` → `listed true`, `applied ['Bright Contrast']`; a repeat wrote
     nothing; `PSM Nope` → `listed false`, nothing changed.
  6. Extra open-alongside check: closed `D`, made a `PSM Open Window` throwaway, ran `open_document.js` on `D`'s
     file. `D` reopened beside both other documents and the throwaway was replaced. `currentDocument()` reads
     `null` after `open` and after closing the front document.
  7. Restored: front, `D` closed, scratch folder and scripts removed.

## Findings

### F1 (blocking, B3): `cargo test` drives the real Capture One, `route: production`
- **What is wrong.** Two tests in `crates/testkit/tests/p00_spike.rs`, `c1b_second_open_preserves_settings_and_migration`
  (two calls) and `c4d_open_drains_file_left_while_closed`, call `App::open(AppPaths::for_dir(tmp))`. That uses the
  real `JxaScripting` with `run_startup = true`, and `paths.home` is the real `$HOME`. Until P24 the startup thread
  ran only when `document_id` was set. This phase removed that guard, as planned, so these tests now run the full
  ensure against the live Capture One.
- **What happened in the reviewer's run.** It created `~/Pictures/Session Manager` (born 19:19:12, opened and
  put in front in Capture One). It ensured the app-wide recipes. It set Capture One's three application-wide hook
  properties to `/var/folders/…/T/.tmpD9ZBah/hooks/*.scpt`, a temp dir deleted when the test ended. With
  Capture One quit, the run would also launch it (`open -g -a`).
- **Authority.** Master plan §3 ("the automated suite never needs the camera, Capture One, Shopify or the
  network", charter rule 9); flow §6A ("The automated suite never depends on the camera or on Capture One").
  B3, no-live-services rule.
- **Correction.** Build the app in those tests over the fake, so no test reaches Capture One. For example:
  `App::open_with(paths, Arc::new(FakeCaptureOneScripting::new(..)), Arc::new(SystemClock), false)`, or the
  `TestApp` equivalent. The spool drain and settings and migration checks they make do not need the startup
  thread. Optionally, add a guard so a test cannot reach real Capture One again.
- **Files the fix may touch.** `crates/testkit/tests/p00_spike.rs`; `crates/testkit/src/test_app.rs` only if a
  helper is wanted.
- **State restored by the reviewer** (a side effect of the review's own suite run):
  - Hooks set back to `~/Library/Application Support/com.loorenz.sessionmanager/hooks/{capture_done,processing_done,variant_adjusted}.scpt`
    and read back. Those are the real app's files, written 2026-11-02 21:52; no test could reach these
    properties before P24.
  - The suite-made `Session Manager` document closed in Capture One; its empty folder `~/Pictures/Session Manager`
    removed (it held no images).
  - `instagram pictures new camaera` set as the front document. It is now the only open document, as at
    planning time.

### F2 (blocking, B2): the open-alongside form cannot be caught by C4(b), `route: production`
- **What is wrong.** Plan §"Task 0 already run", `open_document.js` bullet: never a bare `open` while another
  document is in front, because it replaces that document. The form is "`make` a throwaway document named
  `PSM Open Window` … then `a.open(Path(file))`", and "C4(b) and C6(a) assert this form".
  The implementation splits it: `JxaScripting::open_document` in Rust runs `make_document` for the throwaway, then
  `open_document.js` does a bare `open`. C4(b)'s harness therefore sees exactly one `open` and nothing else. The
  Rust half cannot run without Capture One. So if the throwaway step were dropped, the photographer's window
  would be replaced and every test would still pass. Today's code is correct; the live check above confirmed it.
  The criterion's test cannot fail on the wrong outcome.
- **Authority.** Plan P24 §"Task 0 already run" (`open_document.js` bullet) and row C4(b); book 10C MC-26 step 3
  ("other documents never closed"); VERIFY-14 result.
- **Correction.** `open_document.js` takes the scratch parent as a second argv. It runs `make` (new `document`,
  name `PSM Open Window`, `path` = scratch) and then `open`. Its error path closes the throwaway, guarding
  against a null `currentDocument()`. Rust keeps creating and deleting the scratch folder. C4(b) asserts the
  harness saw the throwaway `make` before the single `open` of the file.
- **Files the fix may touch.** `crates/integrations/src/editing/capture_one/jxa/open_document.js`,
  `crates/integrations/src/editing/capture_one/jxa.rs`, `crates/testkit/tests/p24_capture_one_provisioning.rs`.

## Notes written to backlog
1. P24 (test): row clauses that are not asserted, in C1(a), C1(b), C1(e)(3), C3(a) and p17 `c1b`.
2. P24: `document_not_front` stays set after a refused activation until an activation succeeds.
3. P24: `open_document.js`'s error path dereferences a `currentDocument()` that can be `null` (seen live).
4. P24: the `make_document` id is compared with `document_id` by string, not `canon` (symlinked paths).
5. P24 (UI): the Settings detail line shows the literal `~/Pictures/Session Manager` after "Use the app's own
   session", until Save.
6. P24 → plan template: row cells C4(a)/C4(b) were older than the task-0 section; C6(a) has no open step.

## Lessons for the plans
- A phase that removes a "nothing configured → do nothing" guard must list every test that builds the real
  `App` (`App::open`). Those tests silently start reaching real services. Planner check: `grep App::open` in
  `crates/testkit/tests`.
- When task 0 runs after planning, amend the criterion cells it changes. A separate overriding section left
  C4(a)/C4(b) contradicting the binding text, and the implementer had to declare deviations.
- "Row X asserts this form" only works if the form lives where row X's harness can see it. Say which file
  carries it.

## Probe declaration
- No repository file was edited by a probe; no production code was touched.
- Live scratch: `/private/tmp/psm_p24_live/20261103-1922/` (`PSM Live Session`, `open-window-probe`) and
  `/private/tmp/psm_p24_live_scripts/`, both removed.
- Capture One state, before the live row (already after the suite run): documents `instagram pictures new camaera`
  and `Session Manager`, front `Session Manager`. At the end, after the F1 restore: `instagram pictures new camaera`
  only, in front.
- The application-wide hook properties are restored as described under F1.
- The app-wide recipes `PSM Preview`/`PSM Web` are at the spec (the suite's ensure repaired them; the live row
  wrote nothing to them). The scratch recipe `PSM Live Probe` was created and deleted.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
