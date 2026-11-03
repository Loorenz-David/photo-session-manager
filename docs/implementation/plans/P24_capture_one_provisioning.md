---
phase: P24_capture_one_provisioning
title: Capture One set up by the app — its own session document, launched and opened alongside the photographer's, recipes created and repaired, style applied, front document on activation
difficulty: high
projection: no
depends_on: [P23_shape_on_arrival]
---
# P24 — Capture One set up by the app: its own session document, launched and opened alongside the photographer's, recipes created and repaired, style applied, front document on activation

## Goal
Implement book chapter `10C_capture_one_setup_by_the_app.md` (owner decision 2026-11-03, R36, MC-26, invariant M32). The app no
longer needs a Capture One session prepared by hand. `document_id` missing or null means **the app's own document**
`~/Pictures/Session Manager/Session Manager.cosessiondb` (folder `~/Pictures/Session Manager`); an explicit value (the dev Mac
stores `/Users/davidloorenz/Pictures/session test 1`) keeps being honoured and is provisioned the same way. One **ensure**
operation, owned by the Capture One adapter, run on the serialised user lane under one lock and never on the UI thread, goes
step by step: Capture One running (launch + bounded wait) → the session document exists on disk (else `make new document`,
never over a folder that is not a session) → the document is open (else `open`, other documents never closed, saved, switched
or edited) → the two recipes exist with exactly the dependent properties the renders need (missing → created, drifted → set
back, everything else left alone, other recipes never touched) → the next-capture style when `style_name` is set and Capture
One lists it (else health "Style '<name>' not found in Capture One", capture still works) → hooks (MC-15, unchanged). The
first failing step stops the sequence with its own plain-worded health item; nothing blocks review, export or the rest of the
app (H6). The ensure runs at startup, after a settings save (the runtime rebuild), on Health "Retry checks" and before a
session is activated; activation then brings the app's document to the front **before** the MC-18 capture-folder switch,
because tethered captures land in the foreground document; a front that cannot be verified blocks capture like an
unverified capture folder. Settings shows "Capture One session: Session Manager (created by the app)" with "Use another
session…" (a `.cosessiondb` picker) and "Use the app's own session" to return to the default. **Task 0 is a live grounding
probe** (VERIFY-13/14) in a scratch folder, alongside whatever the owner has open, and the phase **stops with an owner card if
Capture One cannot keep two documents open**. **Not in this phase:** creating styles (a creative act of the photographer),
closing or saving any document the app did not create, migrating photos between documents, the direct camera adapter, the
`render()`-time `recipe_exists` round trip (backlog 33), continuous watching of which document is frontmost (the front is
re-asserted at activation and at every ensure while a session is active, not polled), Keychain, the work-Mac move itself.

## Read first
Book: `10C_capture_one_setup_by_the_app.md` (whole chapter: MC-26, M32, VERIFY-13/14, the stop rule) · `01` (H2, H5: only the
dictionary, H6, H7: core never names Capture One, H8) · `14` (M32; M22, M23, M25 for the rows this phase must not break) · `10`
§Editing provider, §Capture One adapter (the Startup bullet now points at 10C) · `10A` MC-15 (hooks: application-wide
properties, install after compile, read-back by `canon`), MC-16 (recipes: `existing files = overwrite`, `output sub folder =
_session_manager`, `root folder type` not scriptable, the adapter never computes output paths), MC-18 (save → mkdir → set →
read back; `osascript` cwd = data dir), MC-19 (document `id` = the document **folder** path; `current document` only for
`capture`; the serialised lane; 30 s timeout) · `10B` MC-25 (the P23 startup ratio check stays where it is) · `05A` MC-2 (capture
root `<document id>/Capture` must exist) · `12` §Settings, §Health screen · `15` R36 · `16` entry 24 · evidence
`capture_one_16.8.6.md` §Document (session) properties (`kind` r/o, `next capture settings` r/w), §Document identity (id = folder
path; `path` = parent), VERIFY-5 (recipes: `make` works for recipes; a **disabled** recipe processes; `root folder type` writes are
ignored; settable: `output name format`, `output sub folder`, `existing files`, `scaling method` (`Long_Edge`), `scaling unit`,
`primary scaling value`, `JPEG quality`), VERIFY-7 (`documents.byId(<folder path>)`; two open documents **not probed**), §Read-only
smoke (`running()` does not launch; `currentDocument().kind()` etc.), §Transport, §Crop by script (`PSM Preview` 1600×1067,
`PSM Web` 2048×1365 are the live recipes) · `docs/implementation/real_run_2026-10-26.md` "Setup done by the orchestrator" (the two
recipes: JPEG, sRGB, long edge 1600 q80 / 2048 q90, both disabled, created by JXA in `session test 1`) · dictionary
`docs/reference/capture_one_16.8.6.sdef`: Standard Suite `open` `:7–11` (direct parameter: file list), `close` `:13–16`, `make`
`:42–56` (`new` class, `at`, `with properties`), `window` `:82–95` (`index` "ordered front to back", writable), Capture One
`application` `:205–310` (`documents` element `:208`, `current document` `:225` "(PRO Only) The currently frontmost document" —
no `access` attribute, so read-write by the sdef DTD default, **unverified as a setter**; the PRO-only hook properties `:234–258`;
`available styles` `:297–299` r/o list of text), `documentType` `:339–342`, `document` `:357–420` (`name` r/o, `id` r/o, `recipes`
element `:375`, `template` `:391` "during 'make new' only", `path` `:394` r/o "The folder item containing the document", `folder`
`:399` r/o, `kind` `:400` r/o), `next capture settings` `:486`, `save` `:727`, `recipeFileFormat` `:798` (`JPEG`), `scalingType`
`:818` (`Long_Edge`), `measurementUnit` `:828` (`pixels`), `recipeRootType` `:836`, `existing files behavior` `:861` (`overwrite`),
`recipe` `:867–940` (`name` rw, `enabled` rw, `output format`, `JPEG quality`, `color profile` text, `scaling method`, `scaling unit`,
`primary scaling value`, `root folder type` (no `access`, but VERIFY-5 observed writes ignored), `output sub folder`, `existing
files`), `next capture options` `:1710–1718` (`apply styles` list of text) · master plan §3 (standing rules), §5 (preconditions),
§6.3 (`AppPaths`), §6.4 (ports; `CaptureOneScripting`), §6.6 (error and health codes), §6.7 (commands), §6.8 (`document_id`),
§6.9 (test layout), §7.2 (commands), §7.5 (`TestApp`, `FakeCaptureOneScripting`) · process `flow.md` §4 (one behaviour test per
row), §6 (no questions; `BLOCKED` only for an impossible task), §6A (the suite never needs Capture One) · plan
`P23_shape_on_arrival.md` (the previous phase: Node harness shape, live row shape, Review log lessons: untouched reads `null`,
rectangles decode as `x+width`) · code (line numbers at `7874391`, 2026-11-03): `crates/core/src/ports/editing.rs`
(`EditingProvider`: `inspect_document :24`, `set_capture_folder :25`, `install_hooks :26`, `check_recipes :27`),
`crates/core/src/domain/editing.rs` (`DocumentState`, `ShapeOutcome`), `crates/core/src/error.rs` (`DocumentNotOpen`,
`DocumentNotSession`, `RecipeMissing`, `HooksNotInstalled`, `CaptureFolderNotSet`), `crates/core/src/services/editing_startup.rs`
(`EditingStartup::run`: inspect `:15`, hooks `:40`, recipes `:46`, ratio check `:58`, capture folder `:74`),
`crates/core/src/services/sessions.rs` (`with_editing :36`, `activate :46`: inspect → set folder → store), `crates/core/src/
services/camera.rs` (`status`/`capture` gated on `capture_folder_not_set`), `crates/core/src/services/health.rs` (`set` keeps
several `recipe_missing` rows `:78`), `crates/core/src/services/settings.rs` (allowed keys `:38–49`, `export_folder` absolute
rule `:54`), `crates/infra/src/paths.rs` (`AppPaths { root }`, `for_dir`, `from_env_or_default`), `crates/infra/src/settings_file.rs`
(`Settings.document_id: Option<String>`, `Default`), `crates/infra/src/osascript.rs` (`OsaRunner::run`: stderr → `Error::Provider`,
timeout → `Error::Timeout`), `crates/integrations/src/editing/capture_one/scripting.rs` (`CaptureOneScripting`, `DocumentInfo`),
`jxa.rs` (`SCRIPTS :14` — 16 entries; `call`; every script = `lib.js` + script), `jxa/document_info.js` (`documents.byId(id).exists()`
→ throw "document not open"), `jxa/recipe_exists.js`, `jxa/set_recipe_props.js` (replaced by this phase), `gate.rs`
(`GatedScripting::pair`, every method forwarded), `adapter.rs` (`CaptureOneAdapter::new :38`, `inspect_document :131`,
`install_hooks :170`, `check_recipes :191`, `render :211` keeps its `recipe_exists`), `crates/integrations/src/camera/
via_capture_one.rs` (`document_id`), `crates/integrations/src/lib.rs` (`AppHandles :103`, `health_check :184` — runs
`EditingStartup` only when `document_id` is non-empty, `App::build :333`: `document_id = settings.document_id.unwrap_or_default()
:394`, the `"No Capture One session configured"` item `:396`, adapter and camera construction `:398–410`, the startup thread
`:518–535`, `startup() :651`), `apps/desktop/src-tauri/src/commands/settings.rs` (`SettingsDto`, `SettingsPatchDto`),
`apps/desktop/src-tauri/src/commands/sessions.rs` (`session_activate :393`, `session_deactivate :410`), `apps/desktop/src-tauri/src/
commands/items.rs` (`CommandError` code match `:79–97`, exhaustive), `apps/desktop/src/services/settings.ts` (`pickFolder` over
`@tauri-apps/plugin-dialog` `open`), `apps/desktop/src/features/settings/SettingsPage.tsx` (`text(...)` builder `:64`, the Capture One
section `:88–92`), `apps/desktop/src/features/health/HealthPage.tsx` (`names :9`) · tests to reuse: `crates/testkit/tests/
p04_capture_one_adapter.rs` (`app`, `session`, `photo`, `ops`, `capture` helpers `:12–50`; `c3a :157`, `c3b :170` change in this
phase; `c8a :413` must still hold), `p16_operations.rs` (`stored :21`; `c3a :175` fixture changes in this phase),
`p17_real_run_fixes.rs` (`report_item :44`; `c1a :55`, `c1b :74` change in this phase; `c2b :437`, `c3e :488`: the **Node harness**),
`p21_backend_robustness.rs` (`c1a` uses `TestApp::new()` + `health_check()` — must keep passing with the default document),
`crates/testkit/src/fake_capture_one.rs` (`FakeState`, `check_id :112`, `recipes: HashMap<String,(String,String)> :25`),
`crates/testkit/src/test_app.rs` (`create :90`: the temp document folder and `Capture/`; `sessions() :46`),
`apps/desktop/src/features/settings/__tests__/p16_settings.test.tsx` and `p23_shape_on_arrival.test.tsx` (`shown` fixtures; the
`@/services/settings` mock factories enumerate exports).

## Files expected to change
Backend: `crates/core/src/domain/editing.rs` (`ProvisionInputs`, `ProvisionReport`, `StyleOutcome`), `crates/core/src/error.rs`
(`DocumentNotFront`), `crates/core/src/ports/editing.rs` (`ensure`, `bring_document_to_front`; `check_recipes` removed),
`crates/core/src/services/editing_startup.rs` (ensure → front when a session is active → capture folder), `crates/core/src/
services/sessions.rs` (`with_editing` third argument; `activate` = ensure → front → folder), `crates/core/src/services/camera.rs`
(`document_not_front` gates status and capture), `crates/core/src/services/settings.rs` (`document_id`: `""` clears, non-empty
must be absolute), `crates/infra/src/paths.rs` (`home`), `crates/infra/src/settings_file.rs` (`effective_document_id`),
`crates/integrations/src/editing/capture_one/{scripting.rs (7 methods + 4 types, `set_recipe_props` removed), gate.rs
(forwarding), jxa.rs (`SCRIPTS` 16 → 21; `launch` via `open -g -a`), jxa/editor_info.js (new), jxa/make_document.js (new),
jxa/open_document.js (new), jxa/front_document.js (new), jxa/ensure_recipe.js (new), jxa/ensure_capture_style.js (new),
jxa/set_recipe_props.js (deleted), recipes.rs (new: `RecipeSpec`, `SRGB_PROFILE`, dependent props), provision.rs (new: the
ensure sequence, `classify_editor_error`, `EDITOR_WAIT_POLL`), adapter.rs (`ensure`, `bring_document_to_front`, `provision_lock`;
`check_recipes` removed)}`, `crates/integrations/src/editing/capture_one/mod.rs` (`pub mod provision; pub mod recipes;`),
`crates/integrations/src/lib.rs` (`EDITOR_WAIT`, effective document id, `App.provision` + `AppHandles.provision`, the
`"No Capture One session configured"` branch removed, startup/health always run), `crates/testkit/src/fake_capture_one.rs`
(editor state, documents, styles, `RecipeProps`, faults, ops), `crates/testkit/src/test_app.rs` (`home`, the `.cosessiondb` marker,
`sessions()`), `apps/desktop/src-tauri/src/commands/settings.rs` (`effective_document_id` in `SettingsDto`), `apps/desktop/
src-tauri/src/commands/sessions.rs` (`with_editing(.., &app.provision)`), `apps/desktop/src-tauri/src/commands/items.rs`
(`document_not_front`), `apps/desktop/src/services/bindings.ts` (regenerated).
Frontend: `apps/desktop/src/services/settings.ts` (`pickSessionFile`), `apps/desktop/src/features/settings/SettingsPage.tsx`
(the Capture One session block), `apps/desktop/src/features/health/HealthPage.tsx` (`names` for the new codes).
Tests: `crates/testkit/tests/p24_capture_one_provisioning.rs` (new: C1–C4), `apps/desktop/src/features/settings/__tests__/
p24_capture_one_provisioning.test.tsx` (new: C5(a), C5(b)); updated for the new behaviour, each change named in the handoff:
`crates/testkit/tests/p04_capture_one_adapter.rs` (`c3a`: `check_recipes()` → `ensure(&inputs)`, props asserted against
`RecipeSpec`; `c3b`: a recipe that **cannot be created** (`fail_make_recipe`) blocks only its render), `p16_operations.rs` (`c3a`:
"not open" is now the fake's `open_documents` cleared + `fail_open_document`), `p17_real_run_fixes.rs` (`c1a`: the op is
`ensure_recipe:PSM Preview`; `c1b`: a renamed recipe is **created**, not reported missing, and `PSM Preview` stays untouched);
frontend `Settings`-typed fixtures gain `effective_document_id` (`p16_settings.test.tsx`, `p23_shape_on_arrival.test.tsx`,
`p18_pending_state.test.tsx`, `p19_navigation.test.tsx`, `p22_shell_tidy.test.tsx`) and the five `@/services/settings` mock
factories gain `pickSessionFile: vi.fn()`.
Evidence (implementer, task 0): `docs/intention/evidence/capture_one_16.8.6.md` gains "VERIFY-13 result", "VERIFY-14 result" and
"Recipe property read-back" sections (append only; nothing else in `docs/intention` changes).

## Core mechanisms (unit tests allowed)
`psm_integrations::editing::capture_one::provision::classify_editor_error(message: &str) -> Option<EditorFailure>` — the
only pure classifier (not installed / Automation denied) whose inputs are macOS strings the suite cannot produce live.

## Task 0 already run by the orchestrator (2026-11-03) — read before task 1
Results are in `docs/intention/evidence/capture_one_16.8.6.md` ("VERIFY-13 result", "VERIFY-14 result", "Recipe property
read-back"). **Do not re-run task 0.** Binding consequences for tasks 1–5 and the rows:
- `make_document.js`: `withProperties: {name, path: Path(parent)}` — no `kind` (it throws). Returns `id()` (= session folder).
- `open_document.js`: **never a bare `open` while another document is front** (it replaces that document — the owner's
  Instagram session was closed this way during the probe). Form: `make` a throwaway document named
  `PSM Open Window` under the app's data dir (scratch subfolder), then `a.open(Path(file))`, wait until
  `documents.byId(folder).exists()`, then the Rust side deletes the throwaway folder. If the target is already open, do nothing.
  C4(b) and C6(a) assert this form (C6(a): the owner's documents are all still open after open).
- `front_document.js`: `a.currentDocument = a.documents.byId(id)`; read back `currentDocument().id()`.
- `ensure_recipe.js`: recipes are **application-wide**; check `recipes.byName(name).exists()` first; `make` only when absent,
  as `a.make({new:'recipe', at:doc, withProperties:{name}})`; if `make` throws, delete any `Untitled Recipe*` it left and
  return the error. `SRGB_PROFILE = "sRGB Color Space Profile"`. Spellings as in the evidence.
- `ensure_capture_style.js`: `doc.nextCaptureSettings.applyStyles` (list), read-back works.
- C6(a)'s step (1) restore and every live step must keep the owner's documents open; the "two documents" STOP condition did
  not trigger (two documents can be open).

## Ordered tasks
0. **Grounding probe, live, before any code (VERIFY-13/14; book 10C "Grounding to verify before relying on it").** Run as the
   implementer if the session can send Apple events; if the sandbox refuses them (P16's Codex attempt: `-2700`), the
   orchestrator runs this task before dispatch and the implementer reads the results (master plan §5 precondition). Rules: work
   **only** in a scratch folder `/private/tmp/psm_p24_probe/<yyyyMMdd-HHmm>/`; never close, save, switch the content of or edit any
   document you did not create; read-only on the owner's documents (`documents.name()`, `currentDocument().id()`); restore the
   front document you found; close and delete the scratch document at the end. Record, in the evidence file, with the exact JXA
   that worked and the exact values returned:
   (a) `before = {names: a.documents.name(), current: a.currentDocument().id()}` (the owner has `instagram pictures new camaera`
   open at planning time; whatever is open stays open).
   (b) **VERIFY-13 — make new document.** Try `a.make({new: 'document', withProperties: {kind: 'session', name: 'PSM Probe Session',
   path: Path('<scratch>')}})` (then `path` as text, then without `kind`, then with `template`), until one creates a session; record
   the form, what landed on disk (`<scratch>/PSM Probe Session/PSM Probe Session.cosessiondb` + `Capture`, `Selects`, `Output`,
   `Trash`?), whether the parent folder had to exist, the new document's `id()`, `name()`, `kind()`, whether it became
   `currentDocument`, and how long `documents.byId(<folder>).exists()` took to become true.
   (c) **VERIFY-14 — two documents.** After (b): `a.documents.name()` must contain every name of `before.names` **and** ours. If
   any of the owner's documents is gone, **stop here**: close nothing more, write the finding, finish with `STATE: BLOCKED` and
   the owner-card draft in Notes ("alongside" impossible). Otherwise record the count and names.
   (d) **Front document.** Record `a.currentDocument().id()`. Try, in order, until one makes the other document current and reads
   back: (1) `a.currentDocument = a.documents.byId(<id>)`; (2) `a.windows.whose({name: <document name>})()[0].index = 1`;
   (3) `a.activate()` then (1). Use it to bring **ours** to the front, read back, then bring `before.current` back to the front and
   read back. Record the working form and whether Capture One's window came in front of the Terminal (so the app knows whether the
   front switch steals focus).
   (e) **Recipes in the scratch document.** List `doc.recipes.name()` and, for the first built-in recipe, read `outputFormat`,
   `colorProfile`, `scalingMethod`, `scalingUnit`, `primaryScalingValue`, `jpegQuality`, `enabled`, `rootFolderType`, `existingFiles`,
   `outputSubFolder` (spellings as Capture One returns them). Create `PSM Preview` with `make` (`a.make({new: 'recipe', at: doc,
   withProperties: {name}})`, else `doc.recipes.push(a.Recipe({name}))`), set `enabled=false`, `outputFormat='JPEG'`,
   `colorProfile=<sRGB>` (try `'sRGB IEC61966-2.1'`, then the sRGB spelling the built-in recipe reported), `scalingMethod='Long_Edge'`,
   `scalingUnit='pixels'`, `primaryScalingValue=1600`, `jpegQuality=80`, `existingFiles='overwrite'`, `outputSubFolder='_session_manager'`;
   read every one back and record the exact strings. **The sRGB string that reads back is `SRGB_PROFILE` (task 4).** Also record
   `rootFolderType()` of the new recipe (the default a fresh recipe gets; MC-16 relies on it being the Output folder).
   (f) **Styles.** `a.availableStyles()` (list; is `PSM House` among them?), `doc.nextCaptureSettings.applyStyles()` on the scratch
   document; set it to `[<one listed style>]`, read back, set it back to what it was.
   (g) **Open.** `a.close(ours)` → `documents.name()` equals `before.names`; `a.open(Path(<file>))` → is ours back among the open
   documents **with the owner's still open**, which is current, how long until `documents.byId(<folder>).exists()`.
   (h) **Launch.** `open -g -a "Capture One"` while Capture One runs (must be a no-op; record whether it brought Capture One to
   the front). Launching from quit is **not** probed (it would quit the owner's Capture One) — owner try-out step 1.
   (i) **Capture routing** (`capture` lands in the foreground document, dictionary `:1907`) only if a camera is tethered
   (`currentDocument().camera()` not null) and the owner is present; otherwise record "owner try-out step 5".
   (j) Cleanup: `a.close(ours)`, `rm -rf` the scratch folder; final `documents.name()` equals `before.names` and
   `currentDocument().id()` equals `before.current`. Write the three evidence sections. Scripts in tasks 3–4 are written from (b),
   (d), (e), (f), (g) — names and argv are fixed below, bodies follow the probe.
1. **Registry first (master plan §6.3, §6.4, §6.6, §6.7, §6.8).** Domain `psm_core::domain::editing`: `ProvisionInputs { style_name:
   String, wait: Duration }`, `StyleOutcome { Applied, NotListed, NotRequested }`, `ProvisionReport { document_created: bool,
   recipes_created: Vec<String>, style: StyleOutcome }`. `Error::DocumentNotFront` (`document_not_front`). Health codes
   `editor_not_installed`, `editor_not_pro`, `editor_automation_denied`, `editor_not_running`, `document_not_created`,
   `document_not_front`, `style_not_found` (details in §6.6). Settings: `document_id` null/missing/`""` → the default; `settings_set`
   `document_id: ""` stores `null`; `settings_get` adds `effective_document_id`. Constants: `psm_integrations::EDITOR_WAIT`
   (`Duration`, production value **45 s**), `provision::EDITOR_WAIT_POLL = min(500 ms, wait / 4)`. Scripts: `editor_info`,
   `make_document`, `open_document`, `front_document`, `ensure_recipe`, `ensure_capture_style` (`SCRIPTS` 16 → 21, `set_recipe_props`
   deleted); `launch` is `open -g -a "Capture One"` from Rust, not a script.
2. **Default document and settings.** `AppPaths` gains `pub home: PathBuf` (`from_env_or_default` and `for_dir` both read `$HOME`;
   `TestApp` overwrites it with the temp dir). `Settings::effective_document_id(&self, paths: &AppPaths) -> PathBuf` = the
   non-empty `document_id`, else `paths.home/Pictures/Session Manager`; the session file of a folder `f` is `f/<basename of f>.cosessiondb`
   for creation and, for an existing folder, the single `*.cosessiondb` inside it (several → the one named after the folder,
   else the first in name order). `SettingsService::set`: `document_id` must be a string; `""` → stored `Value::Null`; non-empty
   and not absolute → `Err(RelativePath)`. `AppHandles::settings_get` adds `"effective_document_id": <string>` beside
   `effective_preview_delay_s`; `SettingsDto` gains `effective_document_id: String`. `App::build`: `document_id` =
   `effective_document_id(...)` as a string everywhere it was `unwrap_or_default()` (adapter, camera, scan, startup thread, the
   `AppHandles::health_check` guard — the guard becomes unconditional); the `"No Capture One session configured"` item is removed.
3. **Scripting: seven operations, one script each, race-free where it matters.** `CaptureOneScripting` loses `set_recipe_props`
   and gains: `fn editor_info(&self) -> Result<EditorInfo { running: bool, version: Option<String>, pro: Option<bool> }>`
   (`editor_info.js`, argv `[]`: `{running: a.running()}` and, only when running, `version: a.version()` and `pro` = whether reading
   `a.captureDoneScript()` succeeds — the hook properties are "(PRO Only)" in the dictionary; a thrown read → `pro: false`);
   `fn launch(&self) -> Result<()>` (Rust: `open -g -a "Capture One"`; a non-zero exit → `Err(Provider(stderr))`);
   `fn make_document(&self, parent: &str, name: &str) -> Result<String>` (`make_document.js`, argv `[parent, name]`, the form from
   task 0(b); returns the new document's `id()`); `fn open_document(&self, file: &str) -> Result<()>` (`open_document.js`, argv
   `[file]`: `a.open(Path(file))`); `fn front_document(&self, id: &str) -> Result<String>` (`front_document.js`, argv `[id]`, the form
   from task 0(d); returns `a.currentDocument().id()` after the attempt); `fn ensure_recipe(&self, id: &str, name: &str, want:
   &RecipeProps) -> Result<RecipeEnsure { created: bool, changed: Vec<String>, props: RecipeProps }>` (`ensure_recipe.js`, argv
   `[id, name, wantJson]`, **one** run: `r = doc.recipes.byName(name)`; `created = !r.exists()` → make per task 0(e); then for each
   dependent property in the fixed order `enabled, outputFormat, colorProfile, scalingMethod, scalingUnit, primaryScalingValue,
   jpegQuality, existingFiles, outputSubFolder`: read, write only if `!==` wanted, push the name to `changed`; read all back;
   `rootFolderType` is read into `props` and never written); `fn ensure_capture_style(&self, id: &str, name: &str) ->
   Result<StyleEnsure { listed: bool, applied: Vec<String> }>` (`ensure_capture_style.js`, argv `[id, name]`, one run: `listed =
   a.availableStyles().includes(name)`; `current = doc.nextCaptureSettings.applyStyles()`; if listed and `current` is not exactly
   `[name]` → `doc.nextCaptureSettings.applyStyles = [name]`; `applied` = read back). `RecipeProps { enabled: bool, output_format,
   color_profile, scaling_method, scaling_unit: String, primary_scaling_value: i64, jpeg_quality: i64, existing_files,
   output_sub_folder: String, root_folder_type: Option<String> }` (serde, JS camelCase). `GatedScripting` forwards all seven.
   `recipe_exists.js` stays (used by `render`).
4. **Adapter: the ensure sequence.** `recipes.rs`: `pub const SRGB_PROFILE: &str` (the string task 0(e) read back), `RecipeSpec
   { long_edge: i64, jpeg_quality: i64 }` with `PREVIEW = {1600, 80}`, `WEB = {2048, 90}` and `fn props(&self) -> RecipeProps`
   (`enabled false`, `JPEG`, `SRGB_PROFILE`, `Long_Edge`, `pixels`, long edge, quality, `overwrite`, `_session_manager`,
   `root_folder_type: None`). `provision.rs`: `EditorFailure { NotInstalled, AutomationDenied }`, `classify_editor_error(message)`
   (contains `Unable to find application` → `NotInstalled`; contains `-1743` or `Not authorized to send Apple events` →
   `AutomationDenied`; else `None`), and `CaptureOneAdapter::ensure(&self, inputs) -> Result<ProvisionReport>` under
   `provision_lock: Mutex<()>` (one sequence at a time; the startup thread and an activation never interleave), every call on the
   **user** lane, each step idempotent, the first failing step sets its item, clears the ok items of its own and later steps
   (`document`, `recipes`, `hooks`) and returns `Err`:
   1. `editor_info()`; on `Err(e)`: `AutomationDenied` → `editor_automation_denied`; else `editor_not_running` with the error. Not
      running → `launch()` (on `Err(e)`: `NotInstalled` → `editor_not_installed`, else `editor_not_running`), then poll `editor_info()`
      every `EDITOR_WAIT_POLL` (errors during the launch are swallowed and polled again) until `running` or `inputs.wait` has
      elapsed (→ `editor_not_running`, detail `Capture One did not start within {wait_s} s`). `pro == Some(false)` → `editor_not_pro`.
      Success clears the four `editor_*` codes.
   2. `folder = self.document_id`; `fs::metadata(folder)` missing → `make_document(parent(folder), basename(folder))`, then require
      `folder/<basename>.cosessiondb` on disk (else `document_not_created`, detail `Capture One could not create the session {file}:
      {error or "no session file appeared"}`); folder present without any `*.cosessiondb` → `document_not_session`, detail
      `{folder} exists but is not a Capture One session`, `Err(DocumentNotSession)`. Success clears both codes;
      `create_dir_all(folder/Capture)` if missing (MC-2 needs the root on disk; Capture One creates it for a new session).
   3. `document_info(folder)` ok → open; `Err` → `open_document(file)` then poll `document_info(folder)` every `EDITOR_WAIT_POLL` up
      to `inputs.wait` (→ `document_not_open`, detail `Capture One could not open {file}` + `: {error}` when `open` itself failed).
      Success clears `document_not_open`, `set_ok("document", "{file name} open")`.
   4. For `(name, spec)` in `[(preview_recipe, PREVIEW), (web_recipe, WEB)]`: `ensure_recipe(folder, name, &spec.props())`; `Err`,
      or `props` (ignoring `root_folder_type`) ≠ wanted after the call → `recipe_missing` detail `{name} could not be created or
      repaired: {error}` and stop (`Err(RecipeMissing{name})`). Success clears `recipe_missing`, `set_ok("recipes", "Ready")`;
      `created` names go to the report.
   5. `inputs.style_name.trim()` empty → clear `style_not_found`, `StyleOutcome::NotRequested`. Else `ensure_capture_style(folder,
      name)`: `listed == false` → `style_not_found`, detail `Style '{name}' not found in Capture One`, `NotListed`, **continue**;
      `Err(e)` → `style_not_found` detail `Style '{name}' could not be applied: {e}`, continue; `listed` and `applied == [name]` →
      clear, `Applied`; `listed` but `applied ≠ [name]` → treat as the `Err` case.
   6. `self.install_hooks()` (unchanged: read-back by `canon`, `hooks_not_installed` on mismatch); `Err` → stop; success
      `set_ok("hooks", "Installed")`.
   `CaptureOneAdapter::bring_document_to_front(&self) -> Result<()>`: `front_document(document_id)` → read-back compared by `canon`;
   mismatch or `Err` → `health.set("document_not_front", "Capture One is showing another session; captures would land there")`,
   `Err(DocumentNotFront)`; success clears it. `check_recipes` is removed from the adapter and the port.
5. **Core.** `EditingProvider` gains `fn ensure(&self, inputs: &ProvisionInputs) -> Result<ProvisionReport>` and
   `fn bring_document_to_front(&self) -> Result<()>`; `check_recipes` removed. `EditingStartup` gains `provision: &ProvisionInputs`;
   `run()` = `provider.ensure(provision)?` → `inspect_document()?` (kind and capture root as today; the hooks and recipes blocks
   are deleted — the adapter did them) → the P23 ratio check (unchanged) → **if a session is active**, `bring_document_to_front()?`
   → the capture folder as today. `SessionsService::with_editing(editing, health, provision: &'a ProvisionInputs)`;
   `activate(id)`: `editing.ensure(provision)?` → `editing.bring_document_to_front()?` → `inspect_document` → `set_capture_folder` →
   store (MC-18 order kept; the returned error now carries a health item, closing backlog row 29). `deactivate` unchanged.
   `CameraService::status` → `Disconnected` and `capture` → `Err(DocumentNotFront)` while `document_not_front` is set (beside the
   existing `capture_folder_not_set` gate).
6. **Wiring.** `lib.rs`: `pub const EDITOR_WAIT: Duration = Duration::from_secs(45)`; `App.provision: ProvisionInputs { style_name:
   settings.style_name.clone(), wait: EDITOR_WAIT }` and `AppHandles.provision`; `EditingStartup { .., provision: &self.provision }`
   at all three sites (`build` thread, `health_check`, `startup`). `commands/sessions.rs`: `with_editing(editing, health, &provision)`
   in `session_activate`/`session_deactivate` (clone `app.provision` out of the handles). `commands/items.rs`: `Error::DocumentNotFront
   => "document_not_front"`. `TestApp::sessions()` passes `&self.app.provision`.
7. **Fake (`FakeCaptureOneScripting`) and `TestApp`.** `FakeState` gains `running: bool` (true), `pro: bool` (true),
   `automation_denied: bool`, `launch_fails: Option<String>`, `running_after_polls: Option<u32>` (after a successful `launch`,
   `editor_info` reports `running` from the n-th call; `None` = never), `open_documents: Vec<String>` (initial `[document_id]`),
   `front_document: Option<String>` (initial `Some(document_id)`), `fail_make_document: Option<String>`, `fail_open_document:
   Option<String>`, `fail_front: bool`, `fail_make_recipe: Option<String>`, `styles: Vec<String>` (initial `["PSM House"]`),
   `next_capture_styles: Vec<String>` (initial `[]`); `recipes` becomes `HashMap<String, RecipeProps>` (initial `PSM Preview`/`PSM Web`
   with `existing_files "add suffix"`, `output_sub_folder ""`, the other fields at the spec values, `root_folder_type
   Some("output location")`). `check_id(id)` → `Err(DocumentNotOpen)` unless `open_documents` contains `id`. `document_info(id)` for
   an id that is open but not `document_id` answers `kind "session"`, `captures "<id>/Capture"`, no collections, no camera. Ops:
   `editor_info`, `launch`, `make_document:<folder>` (creates on disk `<folder>/<name>.cosessiondb`, `Capture`, `Selects`, `Output`,
   `Trash`; pushes the id to `open_documents`; sets `front_document`), `open_document:<file>` (requires the file on disk; pushes the
   folder; sets front), `front:<folder>` (requires open; sets front unless `fail_front`), `ensure_recipe:<name>` plus
   `make_recipe:<name>` when created and `set_recipe_props:<name>:<changed names, comma-separated, spec order>` when anything was
   written, `ensure_capture_style:<name>` plus `set_next_capture_styles:<name>` when written. `automation_denied` makes
   `editor_info` and `document_info` return `Err(Provider("execution error: Not authorized to send Apple events to Capture One.
   (-1743)"))`. `TestApp::create`: `paths.home = dir.path()`, writes `<document>/Capture One Session.cosessiondb` (empty marker) so
   the configured temp document is a session on disk.
8. **Settings screen and Health labels.** `services/settings.ts`: `pickSessionFile(title): Promise<string | null>` =
   `open({ multiple: false, directory: false, filters: [{ name: 'Capture One session', extensions: ['cosessiondb'] }], title })`.
   `SettingsPage`, Capture One section, replaces the `document_id` text field with: label `Capture One session`; a read-only line
   showing `Session Manager (created by the app)` when `settings.document_id` is null or empty, else the folder path; a detail line
   with `settings.effective_document_id`; button `Use another session…` → `pickSessionFile` → when a path comes back,
   `edit({ document_id: <parent folder of the file> })`; button `Use the app's own session` (shown only when `document_id` is set)
   → `edit({ document_id: null })`. The save patch carries `document_id: settings.document_id ?? ''` (empty = back to the default).
   `HealthPage.names` gains `editor_not_installed`, `editor_not_pro`, `editor_automation_denied`, `editor_not_running` →
   `Capture One`; `document_not_created`, `document_not_front` → `Capture One document`; `style_not_found` → `Style`.
9. Tests (§6.9: `crates/testkit/tests/p24_capture_one_provisioning.rs`; the frontend file; the named updates to p04/p16/p17 and the
   fixtures), `cargo test --workspace`, clippy, fmt, bindings regeneration (diff expected: `SettingsDto.effective_document_id`),
   `npm run typecheck`, `npm test`. The live row C6(a) is run by the implementer if its sandbox allows Apple events, else by the
   reviewer (P16/P17/P23 precedent); its outcome goes in the Review log.

## Criteria (6 criteria, 15 rows; 3 `[rule]` rows)
`inputs(wait)` below = `ProvisionInputs { style_name: <the app's setting>, wait }`; `fake` = `a.fake.state.lock().unwrap()`;
`home` = `a.dir.path()`; `DEFAULT` = `<home>/Pictures/Session Manager`; `DOC` = the fake's initial document
`<home>/Capture One Session` (the configured document under `with_fake_document()`; under `TestApp::new()`, where nothing is
configured, it stands for the photographer's own open document); `ops` as in p04, read after `fake.ops.clear()` where a row
says "cleared". Harness shape for
C4 (P17 `c2b`/`c3e`, P23 C1): scripts written by `JxaScripting::new(&AppPaths::for_dir(tempdir))` into `<dir>/hooks/jxa/`;
`global.Application = () => app`, `global.Path = v => ({toString: () => v})`; the harness records every call it receives
(`calls.push(...)`) and prints `JSON.stringify({result: JSON.parse(run(argv)), calls, writes})`, or `{error: String(e)}`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `TestApp::new()` (no `document_id`); `fake.recipes.clear()` then insert `Owner JPEG` with arbitrary props; `a.app.health_check()` twice; `a.app.settings_get()` | a fresh Mac: nothing at `DEFAULT`, the photographer's `DOC` open | after the first check: `DEFAULT/Session Manager.cosessiondb` and `DEFAULT/Capture` exist on disk; `ops` contain exactly one `make_document:DEFAULT` and no `open_document:`; `fake.open_documents` contains **both** `DEFAULT` and `DOC`; no op addresses `DOC` (no `save`, `set_captures:`, `ensure_recipe:` carrying `DOC`'s id); `fake.recipes` has `PSM Preview == RecipeSpec::PREVIEW.props()` and `PSM Web == WEB.props()` (ignoring `root_folder_type`) and `Owner JPEG` unchanged; `ops` contain `make_recipe:PSM Preview`, `make_recipe:PSM Web`; the report has `document` ok (detail `Session Manager.cosessiondb open`), `recipes` ok, `hooks` ok and none of `document_not_*`, `recipe_missing`, `editor_*`; after the second check: `ops` gained no `make_document:`, `open_document:`, `make_recipe:` or `set_recipe_props:`; `settings_get()["document_id"]` is null and `["effective_document_id"] == DEFAULT` | M32 / MC-26 steps 2–4 (default document created once, opened alongside, recipes created, nothing else touched); H6 |
| C1(b) | `TestApp::with_fake_document()`; `fake.recipes["PSM Web"]` set to `enabled true`, `jpeg_quality 70`, `output_sub_folder ""`, `root_folder_type Some("image folder")`, the rest at spec; `health_check()` | an existing, open document whose web recipe drifted | `ops` contain no `make_document:`/`open_document:`/`make_recipe:`; `ops` contain `set_recipe_props:PSM Web:enabled,jpegQuality,outputSubFolder` and no `set_recipe_props:PSM Preview:`; `fake.recipes["PSM Web"]` equals `WEB.props()` except `root_folder_type == Some("image folder")` (left alone); `PSM Preview` byte-identical to before | MC-26 step 4 (dependent properties set back, other properties left alone); M23 / MC-16 |
| C1(c) | `with_fake_document()`, ops cleared before each call: (1) `fake.running = false`, `running_after_polls = Some(2)`; `a.app.editing.ensure(&inputs(2 s))`; (2) fresh app, `running = false`, `launch_fails = Some("Unable to find application named 'Capture One'")`, `ensure(&inputs(2 s))`; (3) fresh app, `running = false`, `running_after_polls = None`, `ensure(&inputs(Duration::from_millis(50)))` | Capture One quit, then not installed, then never answering | (1) `Ok(report)`; `ops` start `editor_info, launch, editor_info, editor_info` and continue with `ensure_recipe:`; no `editor_*` item; (2) `Err`; `editor_not_installed`, `ok == false`, detail exactly `Capture One is not installed on this Mac`; `ops == [editor_info, launch]` (nothing else attempted); `a.app.health.report("", true, true)` has no `document`, `recipes` or `hooks` item at all; (3) `Err`; `editor_not_running` detail starts with `Capture One did not start`; no `make_document:`/`ensure_recipe:` op | MC-26 step 1 (launch, bounded wait, "not installed" named); H6 |
| C1(d) | `with_fake_document()`: (1) `fake.automation_denied = true`, `health_check()`; (2) fresh app, `fake.pro = false`, `health_check()` | macOS refused the Automation permission; an edition without scripting hooks | (1) `editor_automation_denied`, `ok == false`, detail exactly `macOS has not allowed Session Manager to control Capture One. Allow it under System Settings › Privacy & Security › Automation`; (2) `editor_not_pro`, detail exactly `Capture One Pro is required; the installed edition cannot be scripted`; in both: no `make_document:`, `open_document:`, `ensure_recipe:` or `set_hook:` op after the failing call, and no `document`/`recipes`/`hooks` ok item | MC-26 step 1 ("not Pro", "Automation denied" named in plain words); 10C §What the photographer still does |
| C1(e) | (1) `TestApp::new()`; `create_dir_all(DEFAULT)` and write `DEFAULT/notes.txt`; `health_check()`; (2) `with_fake_document()`; `fake.open_documents.clear()`, `fail_open_document = Some("busy")`; `health_check()`; (3) `with_fake_document()`, `open_documents.clear()`, `fail_open_document = None`; `health_check()` | a folder that is not a session; a closed document Capture One refuses to open; a closed document it opens | (1) `document_not_session`, detail exactly `DEFAULT exists but is not a Capture One session` (the real path); no `make_document:`/`open_document:` op; the folder still holds only `notes.txt`; (2) `ops` contain `open_document:<DOC>/Capture One Session.cosessiondb`; `document_not_open`, detail starts `Capture One could not open`; no `ensure_recipe:` op; (3) `ops` contain exactly one `open_document:`; `fake.open_documents` contains `DOC`; `document` ok with detail `Capture One Session.cosessiondb open`; `recipes` and `hooks` ok | MC-26 steps 2–3 (never create over a non-session folder; open when closed; a failing step stops the sequence) |
| C1(f) | `with_fake_document()`; `fake.styles = ["PSM House", "Clean"]`; `health_check()` twice; `settings_set({"style_name":"Moody"})`, `health_check()`; `settings_set({"style_name":""})`, ops cleared, `health_check()` | the default style listed; an unlisted name; no style | after the first check: `fake.next_capture_styles == ["PSM House"]`, one `set_next_capture_styles:PSM House` op, no `style_not_found`; after the second: still exactly one `set_next_capture_styles:` op (unchanged → no write); after `Moody`: `style_not_found`, `ok == false`, detail exactly `Style 'Moody' not found in Capture One`, `next_capture_styles` still `["PSM House"]`, and that report still has `hooks` ok (the sequence went on); after `""`: no `style_not_found`, no `ensure_capture_style:` op in the cleared `ops`, `next_capture_styles` still `["PSM House"]` | MC-26 step 5 (style applied when listed; "not found" named; empty leaves the photographer's setting) |
| C2(a) | `with_fake_document()`; p04 `session(&a, "K1")`; `fake.open_documents.push("/elsewhere/Instagram")`, `fake.front_document = Some("/elsewhere/Instagram")`; `a.sessions().activate(s)`; then (second app) same plus `fake.running = false`, `running_after_polls = Some(1)`; `activate(s)` | the photographer's other document in front; Capture One quit | `Ok`; `front_document == Some(DOC)`; in `ops`, `front:<DOC>` comes **before** the first `set_captures:`; `open_documents` still contains `/elsewhere/Instagram`; `active_session_id() == Some(s)`; `captures` reads `<root>/K1`; second app: `ops` contain `launch` before `front:` before `set_captures:` | M32 / MC-26 §Front document; M25 / MC-18 (front before the folder switch; other documents stay open) |
| C2(b) | `with_fake_document()`; `session(&a, "K1")`; `fake.fail_front = true`, `front_document = Some("/elsewhere/Instagram")`; `activate(s)`; then `CameraService::new(camera, health).status()` and `.capture()`; then `fake.fail_front = false`, `activate(s)` | the front cannot be verified, then can | `Err(Error::DocumentNotFront)`; `document_not_front`, `ok == false`, detail exactly `Capture One is showing another session; captures would land there`; no `set_captures:` op; `active_session_id() == None`; `status() == Disconnected`, `capture()` → `Err(Error::DocumentNotFront)`, the fake has no `capture` op; after the retry: `Ok`, no `document_not_front`, `active_session_id() == Some(s)` | M25 / MC-18 step 5 analogue (capture blocked rather than landing in another document); M32 |
| C2(c) | `with_fake_document()`; `activate(s)` (ours in front); `fake.front_document = Some("/elsewhere/Instagram")` (the photographer switched); `health_check()`; then `a.sessions().deactivate()`; `front_document = Some("/elsewhere/Instagram")`; `health_check()` | a re-check while a session is active, then with none active | first check: the count of `front:<DOC>` ops grew by one and `front_document == Some(DOC)`; second check: the count of `front:` ops is unchanged and `front_document` is still `/elsewhere/Instagram` | M32 ("frontmost while a session is active", re-asserted by every ensure); 10C §Front document |
| C3(a) | `TestApp::new()`: `settings_get()`; `settings_set({"document_id":"/Users/me/Pictures/Other"})`; `settings_set({"document_id":""})`; `settings_set({"document_id":"relative/x"})`; separately a settings file written as the defaults plus `"document_id":"/Users/davidloorenz/Pictures/session test 1"`, `kill_and_restart()`, `settings_get()` | missing, explicit, cleared, relative, pre-existing | `["document_id"]` null and `["effective_document_id"] == DEFAULT`; after the explicit save: stored `document_id == "/Users/me/Pictures/Other"` and `effective_document_id` the same; after `""`: stored `document_id` is `null` and `effective_document_id == DEFAULT`; the relative value → `Err(Error::RelativePath)` and the stored value unchanged; the pre-existing file: `effective_document_id == "/Users/davidloorenz/Pictures/session test 1"` and `document_id` the same (an explicit value is honoured, the file is not rewritten) | MC-26 §Setting (missing or null → the app's own document; "Use another session" → that document) |
| C4(a) | `node -e <harness> <jxa>/ensure_recipe.js`, three runs | (1) `doc.recipes.byName('PSM Web').exists()` false; `make` records the call and returns a recipe object; argv `[doc, 'PSM Web', WEB.props() as JSON]`; (2) existing `PSM Web` with `enabled true`, `jpegQuality 70`, others at spec, `rootFolderType 'image folder'`, `sharpening 'standard'`; (3) existing recipe exactly at spec | (1) `result.created == true`, `result.changed` equals the nine dependent names in task 3 order, `calls` hold exactly one `make` whose `new == 'recipe'` and `withProperties.name == 'PSM Web'`, the recipe object's fields equal the spec; (2) `created == false`, `changed == ['enabled','jpegQuality']`, `writes == [['enabled', false], ['jpegQuality', 90]]`, `rootFolderType` and `sharpening` unchanged, `result.props.rootFolderType == 'image folder'`; (3) `changed == []`, `writes == []` | MC-26 step 4 (create, repair only what drifted, one call); the spellings are the dictionary's (`:798`, `:818`, `:828`, `:861`) as read back in task 0(e) |
| C4(b) | same harness: `make_document.js`, `open_document.js`, `front_document.js`, `ensure_capture_style.js` (shapes per **VERIFY-13/14 as recorded by task 0**; the harness models that recorded form) | `make_document` argv `['/tmp/x', 'Session Manager']`; `open_document` argv `['/tmp/x/Session Manager/Session Manager.cosessiondb']`; `front_document` argv `['/tmp/x/Session Manager']` with `currentDocument` initially another id; `ensure_capture_style` with `availableStyles() == ['PSM House','Clean']`, `applyStyles()` initially `[]`: runs for `PSM House`, then `PSM House` again (now `['PSM House']`), then `Moody` | `make_document`: exactly one `make` call with `new == 'document'`, `withProperties.kind == 'session'`, `.name == 'Session Manager'`, `.path` stringifying to `/tmp/x`; prints the id the fake returned; `open_document`: exactly one `open` call whose argument stringifies to the file path; `front_document`: performs the recorded front form once and prints the id the harness then reports as current; `ensure_capture_style`: run 1 → `{listed: true, applied: ['PSM House']}` and `writes == [['PSM House']]`; run 2 → `listed true`, `writes == []`; run 3 → `{listed: false, applied: ['PSM House']}`, `writes == []` | MC-26 steps 2, 3, 5 and §Front document; VERIFY-13/14 (evidence) — fixtures grounded by task 0, never by prose |
| C5(a) | `<SettingsPage>` (p16 render shape) with `getSettings` → p16 `shown` + `document_id: null`, `effective_document_id: '/Users/me/Pictures/Session Manager'`; `pickSessionFile` → `/Users/me/Pictures/Other/Other.cosessiondb`; second render with `document_id: '/Users/me/Pictures/Other'`; third with `pickSessionFile` → `null` | — | first render: the text `Session Manager (created by the app)` and the text `/Users/me/Pictures/Session Manager` are shown, no `Use the app's own session` button; click `Use another session…` → the block shows `/Users/me/Pictures/Other`; click `Save settings` → the patch has `document_id === '/Users/me/Pictures/Other'`; second render: the path is shown, click `Use the app's own session` → `Session Manager (created by the app)` is shown; save → patch `document_id === ''`; third: after the cancelled picker the text is unchanged and no `role="alert"` | `[rule]` 10C §Setting ("Capture One session: Session Manager (created by the app)" with "Use another session…"); ch12 §Settings |
| C5(b) | `<HealthPage>` (p16 C3(b) render) | `getHealth` → the seven new codes, each `ok:false` with a distinct detail: `editor_not_installed` `Capture One is not installed on this Mac`, `editor_not_pro`, `editor_automation_denied`, `editor_not_running` `Capture One did not start within 45 s`, `document_not_created`, `document_not_front`, `style_not_found` `Style 'PSM House' not found in Capture One` | every detail is shown inside a `.health-row[data-ok="false"]`; the four `editor_*` rows' `.health-label` text is `Capture One`; `document_not_created` and `document_not_front` → `Capture One document`; `style_not_found` → `Style`; no raw code text appears | `[rule]` ch12 §Health screen; 10C §Ensure ("health item naming that") |
| C6(a) | **live (optional, run where Apple events are allowed; must restore):** scripts as `JxaScripting::new` writes them into a temp dir outside the repo; `S=/private/tmp/psm_p24_live/<stamp>`, `D="$S/PSM Live Session"`; before: `B_NAMES` = `osascript -l JavaScript -e 'JSON.stringify(Application("Capture One").documents.name())'`, `B_CUR` = `…currentDocument().id()` | Capture One 16.8.6 running with the owner's document(s) open (planning time: `instagram pictures new camaera`); the camera need not be tethered | (1) `make_document.js "$S" "PSM Live Session"` prints an id equal to `D`; `D/PSM Live Session.cosessiondb` exists; (2) `documents.name()` now equals `B_NAMES` plus `PSM Live Session.cosessiondb` (every owner document still open — **if not, stop: owner card, Notes**); (3) `ensure_recipe.js "$D" "PSM Preview" <PREVIEW json>` and `… "PSM Web" <WEB json>` print `created true` and props equal to the specs; a second run of each prints `changed []`; (4) `front_document.js "$D"` prints `D` and `currentDocument().id()` reads `D`; (5) `ensure_capture_style.js "$D" "<a name from availableStyles()>"` prints `listed true` and the read-back; `… "$D" "PSM Nope"` prints `listed false` and leaves `applyStyles()` as (5) left it; (6) restore: `front_document.js "$B_CUR"` prints `B_CUR`; `osascript … 'a.close(a.documents.byId("D"))'`; `rm -rf "$S"`; `documents.name()` equals `B_NAMES` and `currentDocument().id()` equals `B_CUR`; (7) the Review log records every observed shape (make form, front form, `applyStyles` read-back, open timing) and that no owner document was closed, saved or switched beyond the front restore | `[rule]` M32 live / VERIFY-13/14 (the suite never needs Capture One, flow §6A) |

## Delegated choices
How `AppPaths.home` is set in tests (field assignment after `for_dir`, or a `with_home` builder); whether `effective_document_id`
lives on `Settings` or as a free function in `lib.rs`; the `.cosessiondb` lookup helper's home (adapter or infra); the exact
`Provider` message texts beyond the required prefixes and exact details above; whether `ensure` returns early on
`document_not_session` with `Error::DocumentNotSession` or a `Provider` error carrying the same text (the health detail is what
the row asserts); whether `EDITOR_WAIT_POLL` is a function or a constant; how `make_document`'s `path` is passed (`Path(parent)`
or text) — whatever task 0(b) found; the JS form of the front switch — whatever task 0(d) found; whether `editor_info.js` reads
`version()`; whether the fake's `make_document` writes four folders or only `Capture`; whether `TestApp::sessions()` clones
`provision` or borrows it; the read-only line's element in the Settings block (`<p>` with an id, or a disabled input) as long as
the texts in C5(a) are findable; whether `pickSessionFile` lives beside `pickFolder` or replaces it for this field (`pickFolder`
stays for the export folder); test module split (one Rust file is the plan's default).

## Manual try-out
Fresh setup on this Mac, deleting or renaming nothing of yours. Capture One may have your Instagram session open throughout.
1. Quit Capture One. Open Session Manager → Settings. The Capture One block shows "Session Manager (created by the app)" and
   the path `~/Pictures/Session Manager` — if it still shows `session test 1`, press "Use the app's own session" and Save.
   Within about half a minute Capture One opens by itself, with a new session "Session Manager" among its documents and your
   Instagram session still open. In Capture One → Process Recipes you see `PSM Preview` and `PSM Web`, both unticked
   (disabled). Settings → Health: "Capture One document", "Recipes", "Installed hooks" are green; if you have no style called
   `PSM House`, "Style" is red with "Style 'PSM House' not found in Capture One" — clear the Style field in Settings and Save
   (or create the style in Capture One), press Retry checks, and it is gone.
2. In Capture One, click your Instagram session so it is in front. In Session Manager open (activate) a session: Capture One's
   Session Manager document comes to the front; your Instagram session stays open behind it.
3. Tether the camera and shoot one frame: it lands in `~/Pictures/Session Manager/Capture/<the item's folder>` and appears as a
   tile. (This is the capture-routing check the probe could not do without a camera.)
4. Quit Capture One while a session is open in the app, then press the capture button or Health → Retry checks: Capture One
   starts again, the Session Manager document is in front, and shooting works.
5. In Capture One, tick `PSM Web` (enable it) and change its quality to 70. Health → Retry checks: it is unticked and back at 90.
   Change a recipe of your own, or add one: the app never touches it.
6. Settings → "Use another session…", pick `~/Pictures/session test 1/session test 1.cosessiondb`, Save: the block shows that
   folder; Health is green; captures go to that session's folders. "Use the app's own session" + Save brings the default back.
7. Optional: System Settings → Privacy & Security → Automation → Session Manager → switch Capture One off. Health → Retry checks
   shows "Capture One — macOS has not allowed Session Manager to control Capture One…". Switch it on again, Retry checks: green.

## Notes
- **Reading of `document_id` (10C vs MC-19).** 10C writes the default as the file `…/Session Manager.cosessiondb`; MC-19 fixes the
  document `id` as the **folder** path, and every stored value so far is a folder. The plan keeps the folder in `document_id`
  (nothing stored changes shape), derives the file (`<folder>/<folder name>.cosessiondb`, or the one `.cosessiondb` inside an
  existing folder) and stores the picker's parent folder. No book change needed.
- **"The first failing step stops the sequence" and step 5.** Style not listed is reported but **does not stop** the sequence
  (10C: "capture still works"), so hooks are still installed after a missing style; everything else stops at its step and leaves
  no stale green item for the later steps.
- **M32's "frontmost while a session is active" versus MC-26's "when a session is activated".** The invariant reads stronger than
  the mechanism. The plan re-asserts the front at activation **and** at every ensure while a session is active (startup, settings
  save, Retry checks, C2(c)); it does not poll Capture One's front document while the photographer works. Worth a sentence in 10C
  if the owner wants the stronger reading enforced; not a contradiction, so not carded.
- **Why the whole sequence runs before an activation (not only "open + front").** Activation is the moment captures must be
  routable. If Capture One was quit and relaunched by the activation, the processing-done hook may be unset and every preview
  render would time out (MC-16); the hooks and recipes steps are cheap when nothing has drifted (one read-only script call each).
- **Why `ensure` lives in the adapter and core only calls it.** Core knows "ensure the editing side" and the health codes;
  launching an app, `make new document`, recipe property names and style lists are Capture One concepts (H7). The sequence holds
  one `provision_lock` so the startup thread and an activation cannot interleave (backlog row 36 named that race).
- **Bounded waits and tests.** `ProvisionInputs.wait` carries the bound so a row can prove the timeout path in 50 ms while
  production uses `EDITOR_WAIT` = 45 s (charter rule 13: rows assert the behaviour, never the 45). Opening a session document
  takes a few seconds; the same bound covers it.
- **Externally owned shapes (charter rule 17) and why projection stays off.** `make new document` (form, where files land,
  whether the parent must exist), the front switch (`current document` setter versus window `index`), `open` with another
  document open, the sRGB `color profile` string and the recipe enumerator read-backs are **not in the evidence yet**; task 0
  grounds every one of them live before a line of script is written, and C4's harness rows are written from that record. Known
  today: `recipeFileFormat` `JPEG`, `scalingType` `Long_Edge`, `measurementUnit` `pixels`, `existing files behavior` `overwrite`
  (sdef; VERIFY-5 observed `Long_Edge`, `overwrite` writes), `make` of a recipe (VERIFY-5), `documents.byId(<folder>)` (VERIFY-7),
  `running()` does not launch (smoke 2026-10-25). `/usr/bin/open` prints `Unable to find application named '<name>'` and exits 1
  for an unknown app (verified on the dev Mac 2026-11-03); the Automation refusal is macOS's `errAEEventNotPermitted` (-1743),
  printed by `osascript` as `execution error: Not authorized to send Apple events to Capture One. (-1743)` — not reproduced live
  (it would mean revoking the owner's permission); the classifier matches the code and the phrase, and try-out step 7 lets the
  owner see it once. "Not Pro" is detected by a PRO-only property read failing; the dev Mac runs Pro, so it is unverifiable here.
- **STOP rule and the owner-card draft (10C: "If Capture One cannot keep two documents open, the implementer stops and the owner
  decides").** If task 0(c) or C6(a)(2) shows an owner document closed by our `make`/`open`, the session ends `STATE: BLOCKED` with
  this card for the orchestrator to relay: *Question:* Capture One closes your open session when the app opens its own — which
  way? *Story:* you are retouching Instagram photos in Capture One; you start Session Manager to shoot a chair; Capture One
  switches to the Session Manager session and your Instagram work is closed (saved by Capture One, but gone from the screen).
  *Branches:* **A** the app opens its session only when no other document is open and otherwise shows "Close your other session
  in Capture One, then press Retry checks" (nothing of yours is ever closed) · **B** the app asks Capture One to switch anyway
  (your other session closes; Capture One saves it first) · **C** go back to one shared document: the app uses whatever session
  is open and only creates its own when none is. *Recommendation:* A — it keeps the "never touch other documents" promise and
  costs one click when it happens. *On silence:* the gate holds. *Trace:* 10C MC-26 steps 3 and §Grounding; M32; this plan C1(a),
  C1(e), C6(a).
- **Default style `PSM House` (master plan §6.8) on a fresh Mac.** A Mac without that style shows the red "Style" item until the
  photographer creates the style or clears the field (try-out step 1 says so). Kept as the registry default; the owner may prefer
  an empty default — one line in §6.8 and `Settings::default()` if so.
- **Health detail of `recipe_missing` changes** from the bare name to `<name> could not be created or repaired: <error>` (p04 c3b,
  p17 c1b updated). The Health label stays "Recipe missing".
- **Capture root for a new document.** `make new document` is expected to create `Capture/`; the adapter also `create_dir_all`s
  `<folder>/Capture` when missing so MC-2's "must exist on disk" holds for a session the photographer emptied by hand. Nothing
  else on disk is created by the app.
- **Fake documents.** The fake gains a list of open documents and a front document but keeps one modelled document for images,
  recipes and the capture folder; the photographer's "other" document is an entry that must still be there afterwards. `TestApp`
  writes an empty `.cosessiondb` marker beside the temp document's `Capture/` so the existence check sees a session.
- **Sandbox precondition.** Task 0 and C6(a) need Apple events. The Codex sandbox refused them in P16's attempt; if the implementer
  cannot run task 0, the orchestrator runs it before dispatch (master plan §5) — the plan's scripts cannot be written from prose.
- Registry additions made by the planner (master plan §6.3, §6.4, §6.6, §6.7, §6.8, §7.5). No intention-book change is needed; the
  two readings above are recorded here, `docs/intention` is untouched by this plan.
- Estimated size: ~32 files, ~1 500 lines (half tests and fake). One session; difficulty high.

## Review log
- 2026-11-03 (orchestrator): task 0 run live by the orchestrator (owner said "go now"); findings in evidence and the section above; book 10C steps 3–4 grounded (changelog 25).
- 2026-11-03 (codex, implement r1): Built default and selected Capture One session provisioning in the adapter, launch and bounded wait, safe create/open alongside, recipe creation/repair, next-capture style, hooks, front verification before capture routing, Settings picker and health labels. The fake and 12 Rust phase tests plus 2 UI phase tests cover C1–C5; full Rust/Node suites, clippy, fmt and typecheck passed. C6(a) live check was left for review as the plan permits when Apple events are unavailable to Codex. Assumptions: use `AppPaths.home` from HOME with a temp override in tests; choose the folder-named `.cosessiondb` first, then name order; the open adapter makes a uniquely parented `PSM Open Window` throwaway document and removes it after opening. Deviations in stale criterion wording: C4(b) expects `withProperties.kind == 'session'`, which the completed VERIFY-13 probe proved throws, so the harness asserts no `kind`; C4(a) expects all nine dependent recipe fields in `changed` on creation, while VERIFY-14 says a fresh recipe has `enabled=false`, so an unchanged `enabled` is correctly omitted (eight fields change). Existing P17 capture-root regression now asserts the P24 repair (recreate missing `Capture/`). No owner document was touched by the automated suite; the task 0 orchestrator probe recorded two documents open together and the front/open forms.
- 2026-11-03 (opus, review r1): **CHANGES_REQUESTED.** Perimeter `1a4d277` = the 45 declared paths. Suite 445 Rust / 129 UI, clippy, fmt, typecheck green. Rows C1(a)–C1(f), C2(a)–(c), C3(a), C4(a), C5(a), C5(b) hold at their boundaries; C4(a)'s eight `changed` names on creation (not nine) and C4(b)'s `make` without `kind` follow task 0 and are accepted. **F1 (B3, route production):** `crates/testkit/tests/p00_spike.rs` `c1b_second_open_preserves_settings_and_migration` and `c4d_open_drains_file_left_while_closed` call `App::open(AppPaths::for_dir(tmp))`: real `JxaScripting`, `run_startup = true`, `paths.home = $HOME`. P24 removed the `document_id` guard on the startup thread, so `cargo test` runs the ensure sequence against the live Capture One. In the reviewer's run it created `~/Pictures/Session Manager` (open and in front), repaired the app-wide recipes, and set the three application-wide hook properties to `/var/folders/…/.tmpD9ZBah/hooks/*.scpt`, a deleted temp dir. With Capture One quit, the suite would launch it. The reviewer restored the state (hooks back to `~/Library/Application Support/com.loorenz.sessionmanager/hooks/*.scpt`, the suite-made document closed, its empty folder removed, the owner's document in front). Fix: those tests build the app over the fake (`App::open_with(.., FakeCaptureOneScripting, .., false)` or equivalent); the suite must not reach Capture One (master plan §3, flow §6A). **F2 (B2, route production):** the plan's task-0 binding puts the open-alongside form (`make` a `PSM Open Window` throwaway, then `open`) in `open_document.js` and says C4(b) asserts it. The implementation does the throwaway `make` in Rust (`JxaScripting::open_document`), so C4(b) sees only a bare `open`, and nothing in the suite would catch a bare `open` replacing the photographer's window. Fix: `open_document.js` takes the scratch parent as a second argv and does `make` (throwaway) → `open`. Rust keeps creating and deleting the folder. C4(b) asserts `make` with `new 'document'`, name `PSM Open Window`, before the one `open`. **C6(a) live (reviewer, Capture One 16.8.6.25):** (1) `make_document.js` → id = `D`, on disk `.cosessiondb` + `Capture/Selects/Output/Trash`, ~3 s; (2) owner's `instagram pictures new camaera` still open beside it; (3) `ensure_recipe` on the existing app-wide `PSM Preview`/`PSM Web` → `created false`, `changed []`, props = spec, `rootFolderType 'output location'`; a scratch `PSM Live Probe` → `created true`, eight `changed`, props = spec; second run `changed []`; probe recipe deleted, no `Untitled Recipe` left; (4) `front_document.js` both ways reads back; (5) `ensure_capture_style` with `Bright Contrast` → `listed true, applied ['Bright Contrast']`, repeat unchanged, `PSM Nope` → `listed false`, unchanged; extra: `D` closed, throwaway made, `open_document.js` → `D` open beside both other documents, `currentDocument()` reads `null` after `open` and after closing the front document; (6) restored: front back, `D` closed, scratch removed, names/current as before. No owner document closed or saved. 6 notes in backlog.
- 2026-11-03 (codex, fix r2): **F1 and F2 resolved.** F1: `c1b_second_open_preserves_settings_and_migration` and `c4d_open_drains_file_left_while_closed` now use fake Capture One scripting with startup disabled, so their app construction cannot invoke real Capture One. F2: `open_document.js` now creates the `PSM Open Window` throwaway at the Rust-created scratch parent before opening the target; error cleanup tolerates a null `currentDocument()`. Rust still removes the scratch folder. C4(b) asserts the `make new document` form precedes exactly one target `open`. No additional choices or divergences. Targeted p00/P24 tests passed; full workspace tests (445 Rust, 129 UI), clippy, fmt and typecheck passed.
