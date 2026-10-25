---
plan: P04_capture_one_adapter
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-10-25
actor: Claude Opus 5.5 (pipeline-projectionist, plan-projection doctrine under docs/process/flow.md)
---
# Projection of P04_capture_one_adapter (round 0)

The Capture One phase can be built, but the plan is not ready to hand to the implementer yet. I found
25 places where the implementer would have to guess, and several of the guesses decide what
you would actually see: what the camera dot shows when the capture folder could not be switched, whether
undoing a delete puts the right colour back in Capture One, whether the capture button really
autofocuses before it fires, and whether opening Session Manager starts Capture One without being asked.
Five of the 21 test rows, as written, would pass even if the code were wrong (C4(a), C4(c), C5(a), C5(d), C5(e)). None of this needs you. The
book or the evidence answers every point, so the orchestrator can fix the plan text directly and then
start the implementer.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. Every ledger row below is a plan amendment or an explicit delegation the coordinator can apply.

## Gate check
- Intention status header: **RATIFIED** (owner, 2026-10-24). Pass.
- No upstream gate handoff for P04 in `OWNER_DECISIONS_PENDING` (handoffs/reviewer: inventory and master-plan
  cards are answered per STATUS.md). Pass.
- `depends_on` [P00, P01]: both APPROVED in the master-plan tracker. Pass.
- Phase size: 8 criteria, 21 rows, 2 `[rule]` rows (counted from the table: C1 4, C2 2, C3 2, C4 3, C5 5, C6 1,
  C7 3, C8 1). Matches the plan's own count.

## Decision ledger
Classification: **PG** plan gap (amend the plan) · **IG** intention gap (route upstream; none found) ·
**FC** free choice (record an explicit delegation).

| # | Decision point the artifacts do not determine | Class | Proposed routing (amendment text in short) |
|---|---|---|---|
| L1 | C1(b) "`camera_status() == Disconnected`-equivalent red state" is not one outcome. `CameraStatus` has only `Connected(name)` and `Disconnected` (§6.6), and Task 6 derives status from `document_info.camera` alone, so a tethered camera would read `Connected` while the folder is not set. | PG | Pick one value. Recommend: while health holds `capture_folder_not_set`, camera status = `Disconnected` (no separate `camera_disconnected` item is added). Otherwise, status comes from the document's camera. C1(b) and C1(c) then assert `== Disconnected`. |
| L2 | Where the "capture refused while `capture_folder_not_set`" rule and the camera status live, and the boundary of C7 and of C1(b)'s camera half. `camera_status()` / `camera_capture()` are Tauri commands, which testkit cannot call. The rule is business logic (H8), so it belongs neither in a command nor in the integrations adapter reading core health state. No camera service is in the perimeter. | PG | Add `crates/core/src/services/camera.rs` (new): `CameraService{status, retry, capture}` over `CameraProvider` + health. The commands stay thin. Rows C1(b), C7(a–c) name `CameraService::status/capture` as the boundary. |
| L3 | `activate(s)` on failure: its return value and `active_session_id` are not stated (C1(b), C1(c)). `deactivate()`'s effect on `active_session_id` is not stated (C1(d)). | PG | Recommend: failure → `Err(capture_folder_not_set)` and `active_session_id` unchanged. A frame shot meanwhile lands in the previous folder and is attributed to that session by MC-2, which is consistent. `deactivate()` → `active_session_id` = null. Add both to the rows' outcome cells. |
| L4 | What "health has X" / "health has no X" means. `HealthItem{code, ok, detail}` can express it two ways: an item with `ok=false`, or the item being absent. `recipe_missing{name}` has no field for `name`. `HealthService` today is a stateless `report(data_dir, hooks_compiled, nudge)` (crates/core/src/services/health.rs:6), so it has nowhere to hold startup results. | PG + FC | Define: "has X" ⇔ the report contains an item with `code == X` and `ok == false`. "Has no X" ⇔ no such item. `recipe_missing` carries the recipe name in `detail`, one item per missing recipe. Delegate where the health state is held (core service with interior mutability, or App-owned) and whether positive items are listed (the Manual try-out shows them). |
| L5 | Job states contradict each other. C4(c) expects `status queued` after a failed attempt. C5(b) expects "job `failed` with `tag_verification`". Task 4 says "`done`/`failed` with `attempts += 1`". Yet the poller selects only `queued`, so a `failed` row would never be retried, against MC-19's "stay queued and retry with backoff". | PG | Define: a retryable failure → `status = queued`, `attempts += 1`, `error = <code>`, `next_attempt_at = now + backoff(attempts)`. `failed` is terminal and is not reached by any P04 kind. Rewrite C5(b): `status queued, attempts 1, error "tag_verification", next_attempt_at == now + 1000`. |
| L6 | `backoff(attempts)` indexing ("1, 2, 4 … capped 60 s"): is it `backoff(0)` or `backoff(1)` that equals 1 s, and is the argument read before or after the increment? C4(c)'s `next_attempt_at > now` accepts any backoff (rule 2). | PG | `backoff(n) = min(2^(n−1), 60) s` for n ≥ 1, where n is `attempts` after the increment. The unit test enumerates n = 1…7, so the cap sits between 6 (32 s) and 7 (60 s). C4(c): `next_attempt_at == now + 1000`. |
| L7 | "Copy rows: most recently changed row wins" (Task 4, MC-19). `photos` has no per-row change time (002_domain.sql:35–52; only `created_at`, and copies inherit it, per backlog P02). No migration is in the perimeter, so the implementer would have to invent a column or a heuristic (highest id, session `last_activity_at`). | PG | Recommend no migration. The coalesced `MirrorTag` job's payload is `{photo_id}`, and each enqueue overwrites it, so the latest-enqueuing row wins. The handler writes that row's **current** classification. State this in Task 4. |
| L8 | Which review operations enqueue `MirrorTag`. Only "review.rs (enqueue MirrorTag)" is given. If `undo` does not enqueue, a restored photo keeps its red tag (M26), and no row would notice. | PG | Enqueue in the same transaction for `classify` (any target, including `unreviewed`), `delete` and `undo`. Not for `reorder`, `move` or `copy`, which leave the classification unchanged (copy_photo keeps it: crates/infra/src/sqlite/store.rs:209–216). |
| L9 | Coalescing against a job that is already `running`. If a classify that arrives during the write is absorbed into the running job, the tag stays stale (M26). | PG | Coalesce only into a `queued` row for the same `target`. A classify while the RAW's job is `running` inserts a new `queued` row. Delegate (FC) whether re-enqueueing onto a backed-off queued row resets `next_attempt_at` to now (recommend yes). |
| L10 | How "a timed-out write is followed by a read before any retry" (MC-19, C6(a)) is achieved, and when `last_own_write_at` is recorded. A timed-out write may have been applied and fired the adjusted hook (VERIFY-4). If the own-write time is not recorded, P06 would read that hook as a retouch and dirty the session (M28: own tag writes never dirty a session). | PG + FC | Record `last_own_write_at = now` whenever a write is **issued**, whatever its outcome. Delegate the read-first mechanism: (a) remember `error = timeout` and read first on the next attempt, or (b) every attempt reads first and skips the write when the tag already matches. Both satisfy C6(a). |
| L11 | Scripted capture. Evidence VERIFY-12 (capture_one_16.8.6.md:198–207) shows that a bare `capture` writes **nothing** with the owner's camera settings (AF, focus priority), yet the command still succeeds. Its recipe "for … the camera adapter's trigger" is: `autofocusing = true`, wait 2 s, require `focusIndicator == "focused"`, then `capture()`. The plan's `capture()` op and C7(c) never mention it, so the shutter button would silently do nothing. | PG | Task 1/6: `capture.js` implements the VERIFY-12 recipe. Not focused → `Err(Provider("camera did not focus"))`, so no new registry code is needed. Cite VERIFY-12 in Read-first. The trait op stays `capture()`, so C7(c) is unchanged. |
| L12 | The result envelope of `process` (rule 17). The sdef says it returns "the batch job identifier string **or an error message prefixed with the string 'ERROR'**" (docs/reference/capture_one_16.8.6.sdef:1864), so a failure arrives on the success path. Unhandled, an error text is stored as a job id and P06 waits 60 s for a completion that never comes. | PG | Map a result starting with `ERROR` to `Err(Provider(text))`. Recommend a pure `parse_process_result(&str) -> Result<JobId>` added to **Core mechanisms**, with a two-case table test (UUID → Ok; `ERROR…` → Err). This uses no criterion budget. |
| L13 | JXA `Application("Capture One")` launches Capture One when it is not running (backlog row "P00 → P04, P16"). Startup and every health/camera call would start the editor unasked, while MC-19 plans for "not running → jobs stay queued and retry". | PG | Every script checks `Application("Capture One").running()` first and returns a not-running error without launching. Startup then reports `document_not_open`. |
| L14 | The backlog item addressed to P04 is not folded in. Provider calls run inside sync commands on the IPC thread while holding `Mutex<App>` (state.rs:4; commands/health.rs:29–35). P04 adds startup (≈6 osascript calls at up to 30 s each), activate and camera commands. | PG | Task 5/7: `EditingStartup::run` runs off the main/IPC thread. Commands that reach the provider are `async`, or do not hold the App lock across osascript. Mechanism delegated (FC). |
| L15 | The `DocumentInfo` field set, and H7 placement. C1 needs the capture root, which MC-2 defines as the `file` of collection id `capture` (05A MC-2; evidence l.71–73), but no trait op returns it and the DTO fields are not listed. `crates/core/src/domain/editing.rs` would invite Capture One concepts (`session`/`catalog` kind, variant ids, recipe `existing files`) into core, an H7 violation, which is blocking (B3). | PG | `document_info` returns at least `kind`, `captures`, `capture_root`, `camera: Option<name>`. Core-side types are provider-neutral (for example `DocumentState{open, usable, capture_root, capture_folder}`). Capture One kinds, variant ids and recipe enums stay in `integrations`. Names stay delegated. |
| L16 | Test wiring. `App::open(paths)` takes no providers and uses `SystemClock` internally (crates/integrations/src/lib.rs:34–76). If startup runs inside `App::open` with `JxaScripting`, every existing P00–P03 test would reach osascript, breaking rule 9. C8(a) needs the fake set to `catalog` **before** startup runs. TestApp's `settings.json` has `document_id = null` by default, so it would never match the fake's id. | PG + FC | `App::open` takes the scripting implementation and the clock (production: `JxaScripting`, `SystemClock`; TestApp: the fake, `ManualClock`). TestApp gains a way to configure the fake and `document_id` before startup, or C8(a) calls `EditingStartup::run` directly (its named boundary). API shape delegated. `crates/integrations/src/lib.rs` joins the perimeter (L22). |
| L17 | `check_recipes` when only one recipe exists. Task 3 says "both names exist → set `overwrite` …; else `recipe_missing`". Read literally, `PSM Preview` stays on `add suffix` when `PSM Web` is missing, yet C3(b) requires Preview renders to keep issuing, and MC-16 says completions are then mis-recorded. | PG | Per recipe: each existing recipe is fixed, and each missing one produces its own `recipe_missing`. C3(b) adds "`PSM Preview` is `overwrite` / `_session_manager`" and the exact error `Err(recipe_missing{name: "PSM Web"})`. |
| L18 | Startup with no active session. Task 5 says "re-apply the active session's folder" and is silent when there is none. | PG (book answers) | MC-2: "When no session is active the app sets the capture folder to `<capture root>/_Unassigned/`". Say so in Task 5. |
| L19 | "Job-map entry recorded for P06" (Task 3 `render`). P06 Task 1 creates the job map (`job_id → RenderEntry{…}`), and no P04 row observes it, so two phases claim one structure. | PG | Strike it from P04. `render` returns the job id, and P06 owns the map. |
| L20 | The fake's fidelity to observed shapes (rule 17). (a) VERIFY-3 (l.65–70): a set to a missing folder fails and leaves `captures` as `<cwd>/Capture`. The fake should do this natively, not only under a fault, so C1(a)'s "folder exists" ordering is enforced by the fake. (b) The `/private` read-back observed is the resolved real path (`/tmp/x` → `/private/tmp/x`, l.41–42), not a blind string prefix. A prefix on a non-symlinked path yields `/private/Users/…`, a shape Capture One never produces. (c) Faults are one-shot or sticky: only the `*_next_*` ones are named. (d) C1(c)'s fixture (set succeeds, read-back = `<cwd>/Capture`) is a synthetic combination. The real observation pairs that read-back with a failed set, so it is acceptable only as a single-cause isolation of step 4. | PG + FC | Amend §7.5 / Task 2: (a) native VERIFY-3 behaviour; (b) `private_prefix` ⇒ read-backs return `fs::canonicalize(path)`; (c) state one-shot vs sticky for `readback_garbage`, `drop_tag_writes`, `hooks_readback_null` (C5(b) needs sticky drop); (d) C1(c)'s cell says "synthetic: isolates step 4". The rest of the fake's API is delegated. |
| L21 | None of the 15 JXA scripts runs in the suite, and no task probes them live. Shapes that live only inside the `.js` are unprobed: `document.camera()` with no camera tethered (null vs error; producible **now**, since the camera is away per STATUS.md), the camera `name` display string, VERIFY-11 (the folder-collection fallback C4(b) relies on, which the plan's Notes omit), the JXA spelling of `existing files = overwrite`, and the `Path` → text conversion of the `get_hook`/`get_captures` read-backs. | PG | Add an optional task: a live read-only smoke against `session test 1` (`document_info`, `get_captures`, `get_hook`, `recipe_exists`, `find_image`) plus one captures set/restore round trip. Results go to the Review log. UNPROBED is acceptable if the Codex sandbox refuses Apple Events (P00 precedent). Add VERIFY-11 to the Notes' "Unprobed" list. |
| L22 | The perimeter omits files that must change, and the master plan §3 allows "touches only the files its plan lists". Missing: `crates/core/src/ports/store.rs` + `crates/infra/src/sqlite/store.rs` (job enqueue/poll/update, `raw_state.last_own_write_at`, `settings.active_session_id`); `crates/core/src/error.rs` (`capture_folder_not_set`, `hooks_not_installed`, `recipe_missing{name}`, `tag_verification`, `variant_not_found`; none exist, error.rs:1–25); `crates/core/src/events.rs` (`camera_status_changed`); `crates/core/src/{ports,services,domain}/mod.rs`; `crates/infra/src/lib.rs`; `crates/integrations/src/lib.rs` (App) + `editing/capture_one/mod.rs` + `Cargo.toml` (if uuid/tokio are needed); `apps/desktop/src-tauri/src/{lib.rs, commands/mod.rs, commands/health.rs, state.rs}`; `apps/desktop/src/services/bindings.ts`; `crates/testkit/src/lib.rs`. Listed as if existing but actually new: `crates/core/src/domain/{health.rs, editing.rs}` (HealthItem is in `domain/mod.rs:8–18`), `crates/core/src/ports/{editing.rs, camera.rs}`, `apps/desktop/src-tauri/src/commands/camera.rs`, plus `services/camera.rs` from L2. | PG | Amend "Files expected to change": add the missing paths and mark the new ones `(new)`. |
| L23 | Additional undelegated choices the implementer will hit: how JXA output is encoded (recommend `JSON.stringify` from `run(argv)`, with a non-zero exit meaning `Error::Provider(stderr)`); where the MirrorTag handler lives, given `psm-infra` cannot depend on `psm-integrations` (JobRunner takes handlers as trait objects registered by App); and whether `health variant_not_found` clears when the RAW is later found. | FC | Add these to "Delegated choices". Each is the smallest option consistent with MC-19, recorded under Assumptions. |
| L24 | C7(a) and C7(c) are traced `[rule]` to ch10. Master plan §3 allows `[rule]` only for UI-rendering, "not contracted" operations, and packaging rows, and camera status is none of these. | PG (trace) | Retrace C7(c) to M25. It is the positive half: capture is allowed once the folder reads back, and C1(b) is the blocked half. Either keep C7(a) as a recorded exception in the master plan §3, or retrace it to MC-18 step 5 (the red state's source). |
| L25 | Stale evidence citations. The file grew after the plan was written: VERIFY-4 owner confirmation and VERIFY-1/12 orchestrator entries. | PG (reality) | Fix the line numbers (table below) and add VERIFY-4's mapping confirmation (l.105–106) to C5(e) and VERIFY-12 (l.198–207) to Read-first. |

Decidability amendments (rule 2 companion: each fixture must make its own predicate the only cause):

| # | Row | Defect | Amendment |
|---|---|---|---|
| D1 | C5(a) | The sequence `main, condition, main` starts and ends on the same value, so an implementation that writes the **first** enqueue's classification also yields 4. Coalescing ("one queued job per raw_path") is never observed. | Use `condition, deleted, main` (all distinct; expected 4), and also assert that the fake received exactly one `set_color_tag` for that variant (a call crossing to the external system, which flow §4 allows). |
| D2 | C5(d) | Row B (the copy) is changed last **and** has the higher id **and** sits in the later-touched session, so "highest id" passes too. | Reverse the order: classify the copy row B `condition` at t1, then the original row A `main` at t2. Expected tag 4. |
| D3 | C5(e) | The `unreviewed → 0` case is vacuous if the fake's variant starts at tag 0 and no job runs. | The fake's variants start at tag 3 (unmapped). The unreviewed row gets there through `classify(main)` then `classify(unreviewed)`, so every expected value needs a write. |
| D4 | C4(a), C4(b) | With one variant, "variant at position 1" cannot be told apart from "any variant". C4(b)'s outcome "found; `process` issued" is not exact. | The fixture image has two variants, and position 1's id is not `"1"` (ids are per-document integers, not positions: evidence l.77). C4(b): the fake receives `process(<position-1 variant of the folder-collection image>, "PSM Preview")`, and `render` returns its job id. In C4(a), the image is **only** in the current collection. |
| D5 | C4(c), C5(b) | `next_attempt_at > now` and "retried with backoff" accept any delay. | `== now + 1000` (L6). C5(b) uses the exact four-field outcome of L5. |
| D6 | C1(b), C1(c), C1(d) | Outcome cells lack the return value, the camera value and `active_session_id` (L1, L3). | Add them as in L1/L3. |

## Criteria decidability and producibility, per row
"Writable now" = a test can be written from the artifacts with one exact outcome at the named boundary.
"Producible" = the Capture One-owned shape in the fixture is one the real application produces (rule 17).
The suite only ever sees the fake's DTOs. Wherever the external shape is confined to a `.js` script, the
producibility question moves to L21's live smoke.

| Row | Writable now? | Producible? (evidence) |
|---|---|---|
| C1(a) | Yes, once L16 wiring exists. | Yes: the `/private` real-path read-back (l.41–42), but model it as realpath (L20b). The capture root comes from collection `capture` (l.71–73), which needs L15. |
| C1(b) | **No**: camera value ambiguous, return value and `active_session_id` missing (L1, L3). | Yes: error text l.66. |
| C1(c) | **No** (same as C1(b)). | Partly: the read-back value is grounded (l.67–69), the combination with a successful set is synthetic (L20d). |
| C1(d) | **No**: `active_session_id` outcome missing (L3). | Yes. |
| C2(a) | Yes (paths: the compiled `.scpt` files under the TestApp data dir; `hooks::compile_all` runs real `osacompile`, no Capture One). | Yes: l.41–42. |
| C2(b) | Yes, given L4's definition of "has". | The null shape is grounded (l.42–43; clearing reads back null, l.213–214). Null after a non-empty set is a synthetic trigger for the mismatch branch, which is acceptable. |
| C3(a) | Yes. | Yes: l.118–123 (settable props; why `overwrite`). |
| C3(b) | **No**: partial-recipe behaviour (L17) and the exact error value are missing. | Yes. |
| C4(a) | Only weakly (D4). | Yes: `images whose path` l.79; UUID job id l.115. The `ERROR` envelope (L12) is not modelled. |
| C4(b) | **No**: outcome not exact (D4). | **Unprobed**: VERIFY-11 (l.183–187) is the probe for this fallback. Our adapter's ordering is testable, the JXA op is not (L21). |
| C4(c) | **No**: weak assertion (D5), status semantics (L5). | Yes (our behaviour). |
| C5(a) | **No as written**: fixture cannot discriminate (D1). | Yes: integer `color tag` l.97–98. |
| C5(b) | **No**: "job `failed`" contradicts C4(c) (L5). | Synthetic fault (dropped write), acceptable: it isolates the read-back. |
| C5(c) | Yes. Note the raw_state row may not exist for `TestApp::add_photo`, so the handler upserts it. | n/a (our state). |
| C5(d) | **No as written**: two causes (D2) and no defined "most recent" (L7). | Yes. |
| C5(e) | **No as written**: the unreviewed case is vacuous (D3). | Yes: mapping confirmed by the owner l.105–106. |
| C6(a) | Yes, once L10's delegation is recorded. | The timed-out-but-applied write is modelled from MC-19's rule, not observed. Acceptable: the observable is our read-first behaviour. |
| C7(a) | Yes, at `CameraService::status` (L2). | The fake's name string is self-consistent. The real `name` (displayName, sdef l.574) string is unprobed (L21). |
| C7(b) | Yes, at `CameraService::status` (L2). | **Unprobed**: what `document.camera()` returns with no camera (L21). It is producible now. |
| C7(c) | Yes, at `CameraService::capture` (L2). | The real op needs the VERIFY-12 recipe (L11), which lives inside `capture.js` and not in the row. |
| C8(a) | Yes, if the test calls `EditingStartup::run` with the fake configured as `catalog` (L16). | Yes: `kind` enum session/catalog (evidence l.52). |

## Reality checks
- **Evidence line citations** (docs/intention/evidence/capture_one_16.8.6.md, current file):
  VERIFY-2 l.39–46 ✓ · VERIFY-3 l.65–73 ✓ · document identity l.75–79 ✓ · VERIFY-4 l.97–103 ✓ (confirmation now at
  l.105–106) · **VERIFY-5 cited 110–121 → actual 113–124** · **C3(a) "l.116–120" → actual 118–123** ·
  **C4(a) "l.111" → actual 115** · **VERIFY-7 cited 218–222 → actual 239–243** · **Transport cited 212–214 →
  actual 233–237** · C1(b) l.66 ✓ · C1(c) l.67–69 ✓ · C2(a) l.41 ✓ · C2(b) l.42 ✓ · C4(a) l.78 ✓ (79).
- **Book/master-plan sections cited** all resolve and say what the plan claims: §6.1, §6.4, §6.6, §7.5, ch01,
  ch10, ch14, 10A MC-15/16/18/19, 05A MC-2/MC-8. The "MC-11 startup rule" (Task 4) resolves to 07A MC-11 l.51, which the
  Read-first list does not name. Add it, or quote the one sentence in Task 4.
- **Prior-phase outputs, verified in code:** `OsaRunner` exists with cwd = data dir and `Error::Timeout`
  (crates/infra/src/osascript.rs). Hook templates and `compile_all` exist (P00). `raw_state.last_own_write_at`,
  `jobs` and `settings` tables exist (002_domain.sql). `Settings.document_id/preview_recipe/web_recipe` exist.
  Not present and not in the perimeter: Store methods for jobs, `last_own_write_at`, `active_session_id`; the error
  variants; the event variant; provider injection in `App::open` (see L16, L22). `TestApp` has no `tick`,
  `open()` or fake accessor yet (expected, since P04 adds them).
- **Downstream overlap:** P05's perimeter also says it adds "capture root in the fake document" to test_app.rs,
  but P04's C1 already needs it. P04 should own it, and P05's line becomes a no-op (coordinator's call). P06 Task 1
  owns the job map (L19).
- **Manual try-out step 3** ("Window → Scripts … shows the three hook properties") is not grounded in the evidence. The
  hook properties are scripting properties, not a menu. Replace it with a one-line read-back the owner can paste,
  e.g. `osascript -l JavaScript -e 'Application("Capture One").captureDoneScript().toString()'`.
- **Core-mechanism location:** §6.9 puts unit tests in `crates/core/src/mechanisms/`, while `tag_for` (integrations) and
  `backoff` (infra) live elsewhere. P00's `hooks.rs` inline tests are precedent. Acceptable, but the coordinator
  should say so in one line so the reviewer does not read it as an orphan.

## Trace verification (both directions)
- Row → ledger: C1 → M25 ✓ (MC-18 invariant), C1(d) → MC-2 `_Unassigned` ✓. C2(a) → "M22 (installation)" is weak,
  because M22's invariant is about dispatch and installation is only its precondition. MC-15 is the accurate trace, and C2(b)
  already uses it. C3 → MC-16 ✓. C4 → MC-19/MC-16 ✓. C5 → M26/M2/MC-19 ✓. C6 → MC-19 ✓. C8 → MC-19 ✓. C7(a)/(c) `[rule]`:
  see L24.
- Ledger → rows: every entry the plan claims (M2, M25, M26, MC-15 install, MC-16 recipes/issuing, MC-18, MC-19,
  MC-2 `_Unassigned`) is served by at least one row ✓. Tasks without rows: the JobRunner startup reset
  (`running → queued`, MC-11), `import`, `capabilities()`, `camera_retry`. They have no row, so the reviewer judges them only
  for B1. That is acceptable, but the coordinator may add `running → queued` to an existing row's fixture at no cost.

## Write perimeter of this session
- Created: `docs/implementation/handoffs/reviewer/handoff_P04_capture_one_adapter_projection_r0.md` (this file).
- Nothing else: no plan, intention, master plan, code or test edits; no commits; no archgraph (none in this repo).
- Read-only commands run: `git log/status`, `cat`/`sed`/`grep` on docs, sources and the sdef.
- No non-authoritative skeleton is attached. The ledger's recommendations are routing proposals for the
  coordinator, not guidance for the implementer.
