---
plan: P34_composition_polish
role: implement
round: 1
state: BLOCKED
date: 2026-11-05
actor: codex
---

P34 code is checkpointed at `a4f9b961e4192295099c2490144267291d748d44`. The automated suites are green. The phase cannot be marked IMPLEMENTED because browser control was denied before rendered motion and live performance measurements could be checked, and two criterion fixtures need plan correction.

## ⚠ OWNER DECISIONS REQUIRED (1)

**Question** — May a resumed P34 session use browser/computer control on this Mac to verify the editor and record the requested ZV-E10 timings?

**Story** — The new editor can show its frame meter, and exports write render durations to the log. This unattended run could compile and test those features, but the computer-control request was denied before it could operate the app. Without a live pass, the owner receives no measured drag, pinch, master, or output times for this Mac.

**Branches** — Allow control: rerun the live verification and record the numbers. Keep control denied: leave the phase pending and measure the steps manually.

**Recommendation** — Allow control for a resumed session so it can finish the specified check on the dev Mac.

**On silence** — The gate holds; P34 stays IMPLEMENTING.

**Trace** — P34 C5(a,b), C7(c); 10D §Performance targets.

## What was built

- Hold Z for a 240 px, pointer-following loupe; drag snapping to the frame centre or image edges with guides and ⌥ bypass.
- ⌥⌘C/V for an in-memory composition clipboard, plus one `composition_apply` command for selected grid photos, with per-photo re-fit, individual validation, partial success and a refusal toast.
- Unsaved dots on the active stage and tiles while an autosave is in flight or queued; reduced-motion durations for glide, browser slide and Original/Adjusted crossfade; a 1440 × 900 default window with 1100 × 720 minimum.
- ⌥⌘P frame meter, master completion timing and actual output render elapsed timing in the structured log.

## Tests → criterion rows

| Row | Boundary test and result | Limit |
|---|---|---|
| C1(a) | `snapping.test.ts` → exact centre/edge/unchanged outcomes, pass | — |
| C2(a) | `p34_loupe_snap.test.tsx` → loupe URL, width, pointer position, removal and no save, pass | Image-point translation asserted only as present, not its exact coordinate. |
| C2(b) | `p34_loupe_snap.test.tsx` → guide and ⌥ bypass, pass | Plan's first save expectation conflicts with the unchanged snapped entry; see Deviations. |
| C3(a) | `p34_clipboard.test.tsx` → copy, next photo, fitted paste, pass | Centre/area assertions are approximate. |
| C3(b) | `p34_clipboard.test.tsx` → one apply call, refusal toast and tile update, pass | — |
| C3(c) | `p34_composition_apply.rs` → two valid saves, one `off_shape` refusal, revisions and change sequence, pass | Application service boundary; the named Tauri command boundary is not directly tested. |
| C4(a) | `p34_motion_dot.test.tsx` → stage and tile pending dots clear on success and failure, pass | — |
| C5(a) | `p34_motion_dot.test.tsx` → crossfade 0 ms, pass | Computed glide/browser transitions need a browser; blocked. |
| C5(b) | `p34_motion_dot.test.tsx` → crossfade 700 ms, pass | Computed glide/browser transitions need a browser; blocked. |
| C6(a) | `p34_motion_dot.test.tsx` → exact Tauri window dimensions, pass | — |
| C7(a) | `p34_meter.test.tsx` → 10 × 16 ms + 48 ms gives avg 19 / worst 48 and toggle hides it, pass | — |
| C7(b) | `p34_composition_apply.rs` → master/output log fields with nonnegative integer `ms`, pass | — |
| C7(c) | No automated test by plan; live dev-Mac report still unmeasured | Blocked computer control. |

## Assumptions

- Clipboard values stay in process memory and are re-fitted with the existing centre/area rule on each target frame.
- A selected photo without a frame is refused in the toast and omitted from the backend request.
- The meter samples `requestAnimationFrame` only while the machine is in a gesture; it displays the latest one-second window.

## Deviations

- `Stage.tsx` and `PhotoGrid.tsx` carry the pending-dot and selection state to their UI boundaries; `harness.tsx` mocks the new service call; `machine.test.ts` uses ⌥ for an old free-drag expectation; `crates/integrations/src/render_output.rs` logs actual output job duration. These are outside the file list but inside P34 behaviour.
- C2(b) specifies a save of `x = 0.166601562` after snapping the default rect back to its unchanged entry. The existing machine's no-op save rule and 10D MC-29's unchanged-composition rule mean that first drag correctly emits no save. The Alt drag does save. This row needs a fixture with a non-centred starting rect or a corrected no-save expectation.
- C3(c) names a `psm-testkit` test at the Tauri command boundary. `psm-testkit` cannot depend on `psm-desktop` without reversing the workspace dependency. The new test exercises `CompositionService::apply`; a command-boundary test would need to live in `psm-desktop` instead.
- `await cua.getState();` was requested for rendered UI verification and returned exactly: `MCP tool call requires approval, but approval policy is never`. Per the launch wrapper, no alternate browser route was attempted. The phase is BLOCKED.
- No ZV-E10 drag/pinch, master, or per-photo output timings were observed in this session. The a7R/work-Mac run is also owed.

## Files changed

Checkpoint `a4f9b96` contains only the 38 paths shown by `git show --pretty='' --name-only a4f9b96`: the listed P34 frontend/editor/service, Rust composition/export/render, tests and binding files; `docs/implementation/master_plan.md`; and `docs/implementation/plans/P34_composition_polish.md`. This handoff is the sole post-checkpoint uncommitted path, for the runner to consume.

## Checkpoint SHA

`a4f9b961e4192295099c2490144267291d748d44` — `CHECKPOINT (not approved): P34 composition polish blocked on browser verification`.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | pass; includes 2 P34 Rust cases |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `cargo test -p psm-desktop export_bindings` | pass; binding regenerated |
| `cd apps/desktop && npm run typecheck` | pass |
| `cd apps/desktop && npm test` | pass, 39 files / 300 tests; first full run had a crossfade regression and a flaky P33 position failure, both were green in targeted reruns and the final full run |
| `git diff --check` | pass before checkpoint |
| `await cua.getState();` | BLOCKED: `MCP tool call requires approval, but approval policy is never` |

## Manual try-out

1. Run `cd apps/desktop && npm run tauri -- dev` on the dev Mac with a session open; hold Z over an adjusted preview, then release. Drag near the frame centre and image edges, then repeat with ⌥ held.
2. Press ⌥⌘C on a framed photo, `]`, then ⌥⌘V. Select photos in the grid and use **Apply composition to selected**; observe saved tiles and any refusal toast.
3. Drag and watch the unsaved dot on the tile and stage. Turn on macOS **Reduce motion** and check the trim glide, browser reveal and Original/Adjusted swap.
4. Relaunch and inspect the 1440 × 900 window and 1100 × 720 minimum.
5. Press ⌥⌘P, drag and pinch a 2560 px adjusted preview, and record the meter's average/worst frame times. Export ZV-E10 photos and read `master rendered` / `render_output done` durations from the app log. **No dev-Mac values were measured in this blocked session.** Repeat on the a7R/work Mac when available.
