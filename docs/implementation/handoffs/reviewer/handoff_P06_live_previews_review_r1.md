---
plan: P06_live_previews
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-25
actor: opus
---
# P06 live previews — review r1

**Verdict: CHANGES_REQUESTED.** Two blocking findings, both `route: production`, both small: F1 (B1) a photo row
added later for an already-rendered RAW never gets the adjusted preview; F2 (B3, H7) the Capture One sidecar layout
is hard-coded in a core service. All 18 criterion rows hold at their boundaries; suite, lint and typecheck are green.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Gate and evidence

- Tracker showed `IMPLEMENTED`; checkpoint `4d3aef271f90c514e42cd2fd12f229c1820806e7` is in `git log`.
- `git show --stat 4d3aef2`: 23 paths, equal to the handoff's declared list. Plan-listed
  `apps/desktop/src-tauri/src/events.rs` is unchanged (event emission sits in `src-tauri/src/lib.rs`); the extras
  (`integrations/src/lib.rs`, `core/src/events.rs`, `infra/src/settings_file.rs`, `integrations/Cargo.toml`,
  `Cargo.lock`, module files) are declared deviations. No finding.
- Stamp, tree `235d34d` (clean `git status --porcelain`; code identical to `4d3aef2`): `cargo test --workspace` exit 0
  (P06 18/18), `cargo clippy --workspace --all-targets -- -D warnings` 0, `cargo fmt --all -- --check` 0,
  `npm run typecheck` 0, `npm test` 0.

## Verified correct

- **MC-16 issuing (C6(a)).** Adapter job map `job id → {purpose, raw, generation, issued_at ms + mono, tracking job}`;
  one render in flight per base name, compared on the lower-cased stem; a second waits in a queue and is issued on
  completion or on final timeout of the first.
- **MC-16 completion (C1(a)–(d)).** Matched by job id (plus a canon(raw) cross-check); `accept_output` = file exists
  and mtime ≥ issued_at − 2 s; temp + rename copy to `previews/<pk[0..2]>/<pk>/adjusted-<g>.jpg` (§6.3 layout, seen
  in the probe output); path set on every row of the RAW and `rendered_gen = max(rendered_gen, g)` in one
  transaction; previous file deleted after commit; one `photo_changed` per row; unknown job logged and ignored.
- **Timeouts (C1(e)).** 60 s preview / 120 s export: re-issue once, then the tracking job is `failed` with
  `timeout`, slot released, no path change.
- **MC-17 staleness (C2(a)–(e), C3(a)).** `adjusted_gen > rendered_gen`; delay clamped 10–30 s, missing or
  non-numeric → 15 s (settings parse made tolerant, which also closes the P00 backlog note for this key); delay
  measured from the last event; an event during a render keeps the RAW stale and the delay rule re-renders it;
  unknown RAW ignored; focus renders every stale RAW immediately, active session first in ch06 order
  (main/condition position, then capture time).
- **Sidecar fallback (C4(a)–(c)).** Newest `Settings*` sidecar, mtime later than the last completion and than own
  write + 30 s, one event per new mtime; at focus and at startup.
- **MC-21 retouch (C5(a)–(c)).** Outside the own-write window: `retouch_gen += 1` and `change_seq += 1` on every
  session with a non-deleted main/condition row of the RAW, one transaction, `session_changed` per session. Inside
  the window: stale only, re-rendered by the delay rule.
- **Tests.** One per row, each at the row's boundary (hook delivery, `tick`, `focus`, stored state, the fake's
  `process` calls) and each able to fail: C2(b) checks at 19.999 s, which separates "15 s after the last event"
  from "after the first"; C3(a)'s fixture separates active-first from position-only ordering; C1(b) uses
  issued_at − 5 s. Unit tests exist only for the plan's two named mechanisms. No over-engineered tests.
- **Assumptions.** Background tick as the delay timer; display order; newest `Settings*`; export callback for
  P09; 2 s retry when the provider fails before a job id. None contradicts the book (the retry cadence is a
  backlog note).
- **Hard constraints.** H1 no external writes. H2 RAWs and sidecars only read; deletes limited to the app's own
  previous preview file. H3 byte copy only. H4 tracking jobs left `running` are requeued at startup and re-request
  the render. H5 AppleScript `process` only. H6 n/a. H8 the Tauri layer only forwards `Focused(true)` to
  `PreviewService::on_focus` and maps events. H7: see F2.

## Findings

### F1 — B1 · route: production — a later row of an already-rendered RAW never gets the adjusted preview

- **What is wrong.** `PreviewService::request_initial` (run by the `render_preview` job queued for each new row)
  returns early when the RAW is not stale and `preview_state.adjusted_preview_path` is set. That path is read from
  the RAW's *first* row. A row added later for the same RAW (import of the same RAW into a second session; P05's
  import dedupes per session) is left with `adjusted_preview_path = NULL` until the next edit in Capture One.
- **Evidence.** Reviewer scratch test (temporary file under `crates/testkit/tests/`, deleted after the run; no
  production code touched): import RAW into S1 → render completes → import the same RAW into S2 → ticks past 60 s →
  rows `[(S1, …/adjusted-1.jpg), (S2, None)]`.
- **Consequence.** S2 can never derive Ready (`mechanisms/status.rs` requires zero non-deleted rows without an
  adjusted preview), and the photo shows no adjusted preview there.
- **Authority.** Plan Task 2 and row C1(a) ("every photo row of the RAW has `adjusted_preview_path`"); book ch10
  Capture One adapter, "per capture … receive processing-done, record the adjusted path"; ch07 Ready rule.
- **Correction.** When the RAW is not stale and has a recorded adjusted preview, give every row of that RAW that
  lacks one the current path and emit `photo_changed` for those rows (or issue a fresh render). Add one boundary
  test: same RAW imported into a second session after the first render → the second row has the path.
- **Files the fix may touch.** `crates/core/src/services/previews.rs`, `crates/core/src/ports/store.rs`,
  `crates/infra/src/sqlite/store.rs`, `crates/testkit/tests/p06_live_previews.rs`.

### F2 — B3 (H7) · route: production — Capture One's sidecar layout lives in a core service

- **What is wrong.** `PreviewService::check_sidecars` in `crates/core/src/services/previews.rs` builds
  `<RAW folder>/CaptureOne/Settings*/<RAW name>.cos` itself. That file layout is a Capture One concept inside a core
  module.
- **Authority.** H7 (book ch01: application-specific concepts never appear in core modules; they live in
  adapters); book ch10 lists the sidecar fallback under the Capture One adapter. The plan delegated "the sidecar
  glob implementation" and lists `ports/editing.rs` and the adapter among the files, so this placement was not
  plan-directed (unlike P05's hook payload note).
- **Correction.** Add a provider-neutral `EditingProvider` method that returns the newest settings-change time for
  a RAW (e.g. `settings_changed_at(raw) -> Option<SystemTime>`), implemented in the Capture One adapter (newest
  `CaptureOne/Settings*/<name>.cos`). `PreviewService` keeps only the comparison rule (later than the last
  completion and than own write + 30 s, once per mtime). C4(a)–(c) stay unchanged and green.
- **Files the fix may touch.** `crates/core/src/ports/editing.rs`, `crates/core/src/services/previews.rs`,
  `crates/integrations/src/editing/capture_one/adapter.rs`, `crates/integrations/src/editing/capture_one/mod.rs`,
  `crates/integrations/src/editing/capture_one/renders.rs` or a new `crates/integrations/src/editing/capture_one/sidecar.rs`.

## Notes written to backlog

Nine rows appended to `docs/implementation/backlog.md`:
1. P06 → P16: unbounded 2 s retry when the provider fails before a job id, one `failed` jobs row per attempt.
2. P06 → P08: focus handler runs renders on the main thread under the app lock (UI freeze per stale RAW).
3. P06 → P16: a twice-timed-out render is not retried until the next edit or relaunch; no health item.
4. P06: crash between commit and deleting the previous generation leaks a file.
5. P06: tick/focus race can leave an orphan `running` tracking row until relaunch (self-healing).
6. P06 → P09: `RenderCompletion` lacks the publish item MC-16 stores for exports.
7. P06: sidecar compared with the preview file's mtime; a RAW without an adjusted preview compares with 1970.
8. P06 → book 07B/10A: MC-21's "retouch inside the own-write window is caught at the next focus" does not hold
   when autosave lands inside the window.
9. P05 → P16 (H7): core arrival skips folders named `CaptureOne` by literal name.

## Lessons for the plans

- A plan that sets a per-row value "on every row of the RAW" at completion should also state what a row created
  *after* the completion gets (arrival path vs completion path). F1 lives in that gap.
- When a plan touches a provider file layout, its file list should name the adapter seam (port method + adapter
  file) rather than leave it under "delegated choices" next to a core service task.
- Book fold-back: MC-21 / MC-17 retouch-inside-window claim (backlog note 8).

## Write perimeter

- `docs/implementation/master_plan.md` (P06 tracker row only → `CHANGES_REQUESTED`).
- `docs/implementation/plans/P06_live_previews.md` (Review log entry appended).
- `docs/implementation/backlog.md` (nine rows appended).
- This handoff.
- Probe: one temporary test file `crates/testkit/tests/zz_reviewer_probe_p06.rs`, created, run, deleted;
  `git status --porcelain` clean afterwards (before these document edits). Its build artefacts live only in
  `target/`. No production code was edited. No database outside test temp dirs was touched.
- No commits.
