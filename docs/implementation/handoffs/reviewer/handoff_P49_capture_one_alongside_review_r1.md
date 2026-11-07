---
plan: P49_capture_one_alongside
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-07
actor: opus
---

# P49 review r1: CHANGES_REQUESTED (1 blocking finding)

**Gate.** Tracker showed `IMPLEMENTED`. Checkpoint `df50d3d19e1fc053c5f3ac41211dab5af544a7bb` is in `git log`.

**Perimeter.** `git show --stat df50d3d` lists 30 paths, and they equal the handoff's declared list: the plan's list plus 3 disclosed deviations, `apps/desktop/src-tauri/src/commands/sessions.rs` (thin wiring, H8 holds) and the `p01`/`p27` test fixtures. The planned change to `crates/core/src/domain/editing.rs` was not needed because `StyleOutcome` was reused as is. Nothing is missing and nothing is extra.

**Suite (reviewer run, clean tree `bfa3b61`; production code = `df50d3d`).** All green:
- `cargo test --workspace`, including p49 19/19
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `npm run typecheck`
- `npm test`: 369/369 with default parallelism; the P33 flake did not occur

## Verified correct
- **C1(a)–(c), MC-18 step 6.** `SessionsService::activate`/`deactivate` and `EditingStartup::run` call `set_current_collection(target)` after a successful `set_capture_folder`. With the direct adapter a failure goes to `editor_not_following` and activation stays `Ok`. With the Capture One camera adapter it returns `CaptureFolderNotSet` and sets `capture_folder_not_set`, the same as a folder failure. Take-back step 5 inherits this through `EditingStartup::run`. Each test fails if the op, its order or the health item is wrong.
- **C2(a)–(d), R46.** Every arrival calls the setter. `set_current_collection.js` compares Capture One's *live* current collection and does nothing when it already matches. This is implementer assumption 1, and it is the faithful reading of "on every arrival while Capture One shows another collection". The plan's `folder != current` state would have failed C2(c). The fake mirrors the script: an op is recorded only on a real switch. The follow service checks `editor_info().running` before any script. Both new scripts check `running()` first and go through the lane-only `GatedScripting::call`, never `launching`, so P48's launch-gate list rightly stays at six (M40). C2(d) asserts that no switch, find, launch or style op happens and that the photo row exists.
- **C3(a)–(d), R-I35.**
  - While the follow service polls `find_variant` every 1 s for 15 s, `variant_waiting` blocks both `request` and `schedule_due`.
  - When the wait expires, `variant_ready` hands over to the preview backoff. C3(c) pins the exact poll seconds: 1…15, then 15, 16, 18, 22, 30, 46.
  - `VariantNotFound` never sets `failed_generation`. Only `expire_renders` does, and nothing is in flight here.
  - `collection_switched` clears `retry_after` and reschedules. C3(d) would panic without the immediate retry, because `issued` would be empty.
- **C4(a)–(e).**
  - The style is applied before `process`.
  - `mark_style_applied` upserts with `COALESCE`: first mark wins, and it works before any `raw_state` row exists.
  - It is not reapplied after a restart.
  - "Nope" gives the P24 health text and the render is still issued.
  - An empty style does nothing.
  - Each variant gets the style once.
  - `apply_style.js` returns early when the style is already listed, so a crash between apply and mark cannot apply it twice (H4).
- **C5(a)–(c), 06 §Import.** The copy goes to a `create_new` hidden temp, then an exclusive `hard_link` into place, with `__dupN` on a clash. Nothing is ever overwritten and the source is untouched (H2). The row is `source = import` with the canonical raw path. `_Unassigned` is used when no session is active, and Capture One follows to that folder. Capture One's `import` is gone from the port, the adapter, the gate, `jxa.rs` and the fake.
- **Dictionary checked against the sdef:** document `current collection` (settable), `available styles`, variant `styles`, `apply style <layer> named`, and collection `file` (hidden, deprecated) all exist.
- **Hard constraints H1–H8 hold.** No live service in the suite; no secrets.
- **Assumptions.** None contradicts the book. (1) Live collection compare: correct under R46. (2) Hard-link publication: same no-overwrite result as the planned exclusive rename. (3) The wait applies to direct deliveries and imports only, and tethered captures keep their render path: consistent with 10E, which scopes the wait to the direct adapter. But see F1: the before-render style step does reach tethered captures.

## Findings

### F1: blocking (B1), route: production
**What is wrong.** `CaptureOneFollow::prepare_render` is installed as `PreviewService::before_render` in `crates/integrations/src/lib.rs`. Before **every** preview render of **any** RAW whose `style_applied_at` is NULL, it runs `find_variant` and then `apply_style_once`. So the style is not limited to arriving variants. It also reaches:
- a photo that arrived while `style_name` was empty, once a style is configured and the photo is next retouched or re-rendered;
- every photo that predates P49 (its `raw_state` row has no mark after migration 007) whose variant does not already list the style;
- tethered Capture One captures.

**Probe.** A temporary test file was added under `crates/testkit/tests/`, run once and deleted; it is listed below. Steps:
1. Arrival with `style_name = ""`, render completed, no `apply_style`.
2. `settings_set({"style_name":"Bright Contrast"})`.
3. `on_variant_adjusted("vold", raw)`, which is a retouch.
4. `tick(16_000)`.

Result: `ops` = `… apply_style:vold:Bright Contrast, recipe_exists, find_image, process:vold:PSM Preview`, and `style_applied_at = Some(18000)`. The style was stacked onto the retouched variant.

**Authority.** 10E §Capture One alongside: "the app applies it to each **new** variant". R-I35: "applied at most once per RAW … and never again after a restart or **on a retouched variant**". 10C MC-26: an empty `style_name` leaves adjustments "as the photographer set them". Plan §The contract: `apply_style_once` runs when the follow service finds an arriving variant.

**Correction.** Apply the style only to a RAW that arrived through `ArrivalFollow::photo_arrived` and has not yet had its first render. Keep that eligibility so it survives a settings change, which rebuilds `App` and drops in-memory state. Two options:
- a pending marker written at arrival and cleared on apply or first render (migration 007 is new in this phase and may gain a column);
- an equivalent "no rendered generation yet" check.

Row C3(b) in the plan's order (style set after the arrival, before the variant is found) must still apply the style. Add one boundary test for this case: style set later, then a retouch, gives no `apply_style`. While there, the before-render path may log a style-script error and render anyway, as the plan's "Err → logged, not marked" says (backlog note).

**Files the fix may touch:**
- `crates/integrations/src/editing/capture_one/follow.rs`
- `crates/core/src/services/previews.rs`
- `crates/core/src/ports/store.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/infra/src/sqlite/migrations/007_style_applied.sql`
- `crates/integrations/src/lib.rs` (wiring only)
- `crates/testkit/tests/p49_capture_one_alongside.rs`
- `crates/testkit/src/fake_capture_one.rs` (only if needed)

No other blocking findings. Every row's test observes its outcome at the row's boundary and fails on a wrong outcome. Over-engineering test applied: no test is coupled to internals.

## Notes written to backlog
Eight entries headed "P49 reviewer r1":
1. Take-back step 5 passes `follow: None`, so no immediate retry happens after it. Destination: P51/P52.
2. A style-script error in the before-render step fails the render, so the preview stops while the error lasts. Destination: the P49 fix or P52.
3. `CaptureOneFollow.current` is written but never read.
4. The C3(b) test sets the style before the arrival, while the row sets it after.
5. Import of a file already in the target folder makes a `__dup1`, and a crash between copy and registration leaves an orphan copy. H4 edge.
6. The collection read-back uses the deprecated `file` property. Live-unverified; destination P52.
7. Task 0 did not reproduce V4 in the fake, so the live cause is still unnamed. Destination: P52.
8. The plan's `SCRIPTS` count (21 vs 22). Planner lesson.

## Lessons for the plans
- A once-only side effect such as the style needs an eligibility rule the row table can falsify ("only arriving RAWs, not RAWs that existed before"). C4 had no row for a RAW that existed before the style was set, so this gap passed all 19 rows.
- When a settings change rebuilds the app, in-memory wait state does not survive it. A row that changes a setting mid-wait (C3(b)) should say which state must persist.

## Probe declaration
- Created, run once, then deleted: `crates/testkit/tests/zz_review_probe_p49.rs` (copied from `/tmp/p49_review_probe.rs`). `git status` after deletion showed no stray file.
- No production file was edited.
- Files written by this review: `docs/implementation/master_plan.md` (P49 tracker row only), `docs/implementation/plans/P49_capture_one_alongside.md` (Review log entry), `docs/implementation/backlog.md` (8 appended lines), and this handoff.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
