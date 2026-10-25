---
phase: P06_live_previews
title: Live adjusted previews — render completion, staleness, focus, sidecar fallback, retouch
difficulty: high
projection: no
depends_on: [P05_capture_arrival]
---
# P06 — Live adjusted previews: render completion, staleness, focus, sidecar fallback, retouch

## Goal
Make the adjusted preview live: match `processing_done` to the render job that caused it and record the file
only when it is the right one (MC-16, M23), track per-RAW staleness and re-render after the delay or on window
focus (MC-17, M24), fall back to the settings sidecar at startup and focus, and treat an adjusted event outside
the own-write window as a retouch that dirties published sessions (MC-21 first half, M28/M13). **Not in this
phase:** export renders' staging and naming (P09; the completion path is shared), UI crossfade (P08), replacement
at publish (P14).

## Read first
master plan §6.3 (previews layout), §6.5 (`raw_state`), §6.7 (events) · book `01`, `14`, `06` §Original / Adjusted,
`10` §Capture One adapter (live adjusted preview bullet), `07B` · contracts `10A` MC-16 (Issuing, Completion), MC-17
(all), MC-19 (`last_own_write`), MC-21 (Retouch event, Dirty); `05A` MC-6 · evidence VERIFY-1 (processing-done argv
`[job id, RAW, [outputs]]`, l.22–27), VERIFY-5 (`overwrite` vs suffix, l.116–120), VERIFY-9 (adjusted hook once per
change 130–165 ms, no args; sidecar path and autosave timing, l.165–174), VERIFY-10 (render 0.52–0.72 s).

## Files expected to change
`crates/core/src/services/{previews.rs (new: PreviewService), editing_events.rs (P05 stubs replaced)}`,
`crates/core/src/mechanisms/staleness.rs` (new), `crates/integrations/src/editing/capture_one/{adapter.rs (job map,
in-flight per base name, re-issue/timeout), renders.rs (new)}`, `crates/infra/src/previews_store.rs` (new: temp+rename
copy, previous generation delete), `crates/core/src/ports/editing.rs` (`RenderCompletion` type), Store trait + impl
(`raw_state` gens, `adjusted_preview_path` on all rows of a RAW, `change_seq` bump across sessions),
`apps/desktop/src-tauri/src/{lib.rs (window focus → `PreviewService::on_focus`), events.rs}`, `crates/testkit/src/test_app.rs`
(`complete_render(job_id)`: fake writes the output JPEG and the test delivers the hook; `focus()`; `write_sidecar(raw, mtime)`),
`crates/testkit/tests/p06_live_previews.rs` (new).

## Core mechanisms (unit tests allowed)
- `mechanisms::staleness::{is_stale(adjusted_gen, rendered_gen), due_at(last_adjusted, delay), clamp_delay(setting) -> Duration}` (MC-17).
- `editing::capture_one::renders::accept_output(output_path, issued_at) -> bool` (exists and mtime ≥ issued_at − 2 s).

## Ordered tasks
1. Job map in the adapter: `job_id → RenderEntry{purpose, raw, issued_at (mono + ms), generation, publish_item?}`;
   at most one render in flight per output base name compared case-insensitively (others wait in a per-name queue).
2. `PreviewService::on_processing_done(job, raw, outputs)`: unknown job → log + ignore; `accept_output` false →
   ignore (the timeout path handles it); else copy `outputs[0]` temp+rename to `previews/…/adjusted-<g>.jpg`, set
   `adjusted_preview_path` on every photo row of the RAW, `rendered_gen = max(rendered_gen, g)`, delete the previous
   generation file after commit, emit `photo_changed` per row; release the base-name slot; if still stale → schedule
   by the normal rule. Export purpose → hand the file to the export staging callback (P09 implements it; P06 ships
   the hook point).
3. Timeouts: no completion within 60 s (preview) / 120 s (export) → re-issue once; a second timeout → job `failed`
   with reason; slot released.
4. `on_variant_adjusted(variant_id, raw)`: known RAW only; `adjusted_gen += 1`, `last_adjusted = mono now`; if
   `now_ms > last_own_write_at + 30 000` → **retouch**: `retouch_gen += 1` and `change_seq += 1` on every session
   holding a non-deleted `main`/`condition` row of that RAW (emit `session_changed`). Scheduling: render when
   `now − last_adjusted ≥ delay` (`clamp_delay(settings.preview_delay_s)`), none in flight.
5. `on_focus()`: render every stale RAW with no render in flight, active session's rows first in display order;
   then sidecar check: for each known RAW, newest `<RAW folder>/CaptureOne/Settings*/<RAW name>.cos` with mtime later
   than the last render completion **and** later than `last_own_write_at + 30 s` → one adjusted event. Same check at
   startup.
6. Tauri: subscribe to the main window's `Focused(true)` event → `on_focus()`. Tests.

## Criteria (6 criteria, 18 rows; 0 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `deliver_hook(processing_done{job, raw, [out]})` | `job` issued for `raw` (Preview, g = 1); `out` exists, mtime ≥ issued_at − 2 s | file `previews/<pk…>/adjusted-1.jpg` bytes == `out`; every photo row of the RAW has `adjusted_preview_path` = that path; `rendered_gen == 1`; one `photo_changed` per row | M23 |
| C1(b) | same | `out` mtime = issued_at − 5 s | nothing recorded; `adjusted_preview_path` unchanged | M23 (stale output rejected) |
| C1(c) | same | unknown job id | nothing recorded; no event | MC-16 (unknown job) |
| C1(d) | second render completes | g = 2 | path now `adjusted-2.jpg`; `adjusted-1.jpg` no longer exists | M23 (newer render changes the path) |
| C1(e) | `tick` with no completion | preview issued at t; `tick(60 s)`; then `tick(60 s)` | after the first 60 s the fake received a second `process` for the RAW; after 120 s the job is `failed` with a timeout reason, no path change | MC-16 (re-issue once, then fail) |
| C2(a) | `deliver_hook(variant_adjusted{raw})` + `tick` | known RAW, `preview_delay_s = 15`, idle | stale; no `process` before 15 s; `process` issued at `tick` reaching 15 s | M24 |
| C2(b) | two adjusted events 5 s apart + `tick` | — | exactly one `process`, at 15 s after the **second** event | M24 |
| C2(c) | adjusted event during an in-flight render; then completion | g in flight = 1, event → gen 2 | after completion `rendered_gen == 1`, still stale; a second `process` is issued by the delay rule | M24 (event during render never lost) |
| C2(d) | `clamp_delay` via service | settings `5`, `40`, missing, `"abc"` | effective delay 10 s, 30 s, 15 s, 15 s | MC-17 (clamp) |
| C2(e) | `deliver_hook(variant_adjusted{raw})` | RAW with no photo row | no `raw_state` row, no `process` | MC-17 (unknown RAW ignored) |
| C3(a) | `focus()` | three stale RAWs (two in the active session at positions 2 and 1, one elsewhere), one not stale | `process` issued immediately for the three stale RAWs in order: active position 1, active position 2, other; none for the fresh one | M24 (focus renders all stale, active first) |
| C4(a) | `focus()` after `write_sidecar(raw, mtime = last completion + 20 s)` | `last_own_write_at` older than 30 s | one adjusted event → stale, `adjusted_gen` +1 | MC-17 (sidecar fallback) |
| C4(b) | `focus()` | sidecar mtime earlier than the last completion | no event | MC-17 |
| C4(c) | `focus()` | sidecar mtime 10 s after a tag write (`last_own_write_at`) | no event | MC-17/MC-21 (own-write window) |
| C5(a) | `variant_adjusted` outside the own-write window | RAW has a `main` row in published session S1 (`published_seq == change_seq`) and a `deleted` row in S2 | `retouch_gen == 1`; S1 `change_seq` +1 → `dirty = true`; S2 `change_seq` unchanged; `session_changed{S1}` emitted | M28, M13 |
| C5(b) | `variant_adjusted` 10 s after the app's own tag write | published S1 | `retouch_gen == 0`; S1 not dirty; RAW still stale and re-rendered by the delay rule | M28 (own tag writes never dirty) |
| C5(c) | `variant_adjusted` | copy rows in S1 and S2, both `main`, both published | both dirty | MC-21 (every session holding the RAW) |
| C6(a) | two renders | `<root>/K1/X.ARW` and `<root>/K2/x.arw` requested back to back | fake receives one `process` until the first completes (`complete_render`), then the second | MC-16 (one in flight per base name) |

## Delegated choices
Where the render timer lives (job runner tick vs. tokio timers; tests drive it through `tick`), the exact display
order source for C3(a) (position, then unreviewed by `captured_at`), the sidecar glob implementation, how the
export completion hook point is shaped for P09.

## Manual try-out
1. Shoot one frame; within ~1 s the Health page's "last events" list (or `capture_events`) shows the capture; within
   a few seconds `adjusted_preview_path` is set (`sqlite3 … 'select adjusted_preview_path from photos;'`).
2. In Capture One, move the Exposure slider on that photo; wait 15 s → the path changes to `adjusted-2.jpg` and the
   old file is gone. Move it again and click back into Session Manager immediately → re-rendered at once.
3. `select retouch_gen from raw_state;` increased by the edits; `select change_seq from sessions` changed only if
   the session was published (none yet).

## Notes
- Our own tag write fires the adjusted hook (evidence VERIFY-4); the extra render is accepted (MC-17) and the
  own-write window keeps it from being a retouch (C5(b)).
- Estimated size: ~9 files, ~1 500 lines.

## Review log
- 2026-10-25, implement r1 (codex): Added the tracked Capture One render queue, completion and timeout handling, atomic adjusted preview copies, per-RAW stale generation scheduling, focus and startup sidecar checks, retouch dirty updates, and Tauri focus/photo/session events. All 18 criterion rows have one boundary test. Assumptions: the existing background tick drives the 15-second delay; display order is active session classification/position then capture time; sidecar selection uses newest `Settings*` file; export completion is a service callback for P09. A provider render request that fails before a job id is returned gets a two-second retry pause. Deviations from the expected file list were the necessary app wiring (`crates/integrations/src/lib.rs`), module exports, core event variants, tolerant settings parsing, and the integration crate's unit-test dependency. An inherited P03 test showed that its direct photo helper deliberately has no `raw_state` row; P06 fixtures now add their own state, while production arrival keeps the P05 path. No export staging or UI crossfade was added.
- 2026-10-25, review r1 (opus): **CHANGES_REQUESTED** (one B1, one B3; both route production). Gate: tracker `IMPLEMENTED`, checkpoint `4d3aef2` present; its 23 committed paths equal the handoff's declared list (plan-listed `src-tauri/src/events.rs` unchanged, emission lives in `src-tauri/src/lib.rs`; extras are declared deviations). Stamp on tree `235d34d` (clean; code = `4d3aef2`): `cargo test --workspace` 0 (P06 18/18), clippy `-D warnings` 0, `fmt --check` 0, `npm run typecheck` 0, `npm test` 0. Verified correct: job map + case-insensitive per-base-name queue (C6(a)); completion matched by job id, `accept_output` = exists ∧ mtime ≥ issued_at − 2 s; temp+rename copy to `previews/<pk[0..2]>/<pk>/adjusted-<g>.jpg` (layout §6.3, seen in probe output), path on every row in one transaction, `rendered_gen = max`, prior file deleted after commit, `photo_changed` per row; unknown job logged and ignored; 60 s re-issue once then `failed`/`timeout` (export 120 s); per-RAW gens, delay clamp 10–30/15 fallback (settings parse tolerant), delay restarts per event, event during render keeps RAW stale and re-renders; focus renders all stale RAWs active-session first (main/condition position, then capture time — ch06); sidecar rule (newest `Settings*`, > last completion and > own write + 30 s, once per mtime) at focus and startup; retouch outside the own-write window bumps `retouch_gen` and `change_seq` on sessions holding non-deleted main/condition rows in one transaction, emits `session_changed`; own tag echo stays a stale-only event. Every row's test asserts its exact outcome at the row's boundary and can fail (C2(b) at 19.999 s separates first-event from last-event timing; C3(a)'s order separates active-first and position). Assumptions consistent with the book (2 s provider retry noted). H1–H6, H8 hold. **F1 (B1, route production):** `request_initial` skips any RAW that is not stale and whose *first* row has a path, so a row added later for an already-rendered RAW (import of the same RAW into a second session) never gets `adjusted_preview_path`; reviewer scratch probe (import into S1 → render → import into S2 → ticks past 60 s): rows `[(S1, …/adjusted-1.jpg), (S2, None)]`; S2 can never derive Ready (status requires zero missing previews). Authority: plan Task 2 / C1(a) ("every photo row of the RAW"), ch10 Capture One adapter "per capture … record the adjusted path", ch07 Ready rule. Correction: when the RAW is not stale and has a recorded adjusted preview, give every row of that RAW lacking one the current path and emit `photo_changed` for those rows (or render anew); one boundary test for it. Files: `crates/core/src/services/previews.rs`, `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`, `crates/testkit/tests/p06_live_previews.rs`. **F2 (B3 H7, route production):** `PreviewService::check_sidecars` (core) hard-codes Capture One's sidecar layout (`CaptureOne/Settings*/<name>.cos`). Authority: H7 (ch01), ch10 places the sidecar fallback in the Capture One adapter, plan leaves the glob implementation delegated (not plan-directed). Correction: a provider-neutral `EditingProvider` method returning the newest settings-change time for a RAW, implemented in the Capture One adapter; the core keeps only the comparison rule; C4 tests unchanged and green. Files: `crates/core/src/ports/editing.rs`, `crates/core/src/services/previews.rs`, `crates/integrations/src/editing/capture_one/{adapter.rs, mod.rs, renders.rs or a new sidecar.rs}`. Notes: 9 rows in backlog. Lessons: plans should state where a new row for an already-rendered RAW gets its preview (arrival vs completion); plans touching provider file layouts should name the adapter seam in the file list; MC-21's "retouch inside the own-write window is caught at the next focus" does not hold for edits whose autosave lands inside the window (book fold-back).
- 2026-10-25, fix r2 (codex): Resolved F1 by having `request_initial` fill only rows missing an adjusted path from the existing clean RAW preview and emit `photo_changed` for each filled row; added the later-session boundary case. Resolved F2 by moving Capture One `Settings*/*.cos` discovery into `CaptureOneAdapter::settings_changed_at`; core now applies only the timestamp comparison rule. C4 tests are unchanged. Full workspace suite passed (P06 19/19); clippy, fmt check, frontend typecheck and npm test passed. No divergences or additional judgment calls.
