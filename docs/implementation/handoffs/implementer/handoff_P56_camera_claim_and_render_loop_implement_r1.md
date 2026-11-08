---
plan: P56_camera_claim_and_render_loop
role: implement
round: 1
state: DONE
date: 2026-11-08
actor: claude (pipeline-implementer, Opus 5.5; relaunch after the Codex r1 block, see the `_blocked` handoff)
---

# P56 implement r1 — the helper takes the cable at plug-in and never sits idle; the export loop is gone

## What was built

**Camera helper (F13, F14; orchestrator rulings 1 and 2).**
- **Plug-in = move to the cable, at once.** When the cable is plugged in and the camera is not busy, the helper closes its
  Wi-Fi session and starts the first USB connect on the next step (≤ 100 ms in the sidecar's loop; the old 2.5 s settle
  wait is gone). Failed attempts repeat every 200 ms until 2.5 s after the plug-in (the "claim window"). If an attempt
  shows the interface is held by another program (IOKit `UsbExclusiveOwner` names someone else), or the window ends without
  a USB session, Wi-Fi reconnects at once and USB waits (`usb.fallback_to_wifi` with the reason). The window's attempts never
  count towards the three-failure rule. A busy camera still defers the switch (`switch.deferred`); the claim starts the moment
  busy clears.
- **Retry on release, never by probing.** While the camera is plugged in and not on a USB session, the helper reads the
  interface's holder every 500 ms (no privileges). When another program lets go (held → not held), it emits `usb.released`
  and opens a new claim window under the same rule (close Wi-Fi, try, back to Wi-Fi at once on failure). While Wi-Fi is
  connected no USB attempt is made otherwise — no Wi-Fi drop to "check" the cable.
- **Never idle.** With USB waiting for its holder and Wi-Fi unreachable, the helper alternates Wi-Fi and USB attempts on the
  normal backoff (max 30 s) for as long as it takes; there is no terminal idle state. A holder release also leaves the
  ERROR state (as a re-plug already did).

**The export loop (F12).** Cause, found in the owner's database and log (read-only):
1. **Capture One's "add suffix" output naming.** In the owner's Capture One the recipe wrote `DSC04979 14.tif`, but the
   processing-done hook reported `DSC04979.tif` — this morning's file (875 TIFF completions on 2026-11-08 over only 47
   distinct reported paths; 0 reported paths carry a suffix). The app checked the reported file's age, found it old, ignored
   the completion, waited 60 s, re-issued the render, waited 60 s more, failed it — and the next trigger started over. The
   "PSM Master" TIFFs in `Output/` were **preview** renders (the jobs table has no `render_master` row at all; every TIFF
   completion matched a preview job): the "PSM Preview" recipe in Capture One had become a TIFF/`Output`/add-suffix recipe
   around 11:52 UTC, when a `recipe_missing: PSM Preview` burst appears in the log (probably during the P52 probes, the
   `psm_probe_sub` / `psmprobe_` files). The app never re-checks recipe properties after startup, so it kept ordering.
2. **Capture One's own sidecar rewrites taken for edits.** The `.cos` sidecars of DSC04766/67/68/70 were all rewritten at
   16:31:03, five minutes after their last export (16:26), with no edit. The fallback rule (sidecar newer than the preview →
   one adjusted event) turned every such rewrite — Capture One processing the export the app ordered, a document save —
   into a new generation and a new export at the next window focus.

Fixes:
- **A finished render is recognised under the name Capture One really used.** Completion still matches by job id; the output
  is the reported path when it was written after the request, otherwise the newest `<stem> <n>.<ext>` beside it written after
  the request (one render per base name is in flight, MC-16). Applies to previews and masters.
- **An export is never taken for an edit.** The app keeps a fingerprint of the adjustments in each RAW's sidecar (the
  `<DL>` lists; Capture One's processing counters sit outside them), recorded when a preview is rendered and at each check.
  On focus and at startup a RAW gets an adjusted event only when its adjustments changed **and** the old time rules hold
  (newer than the preview, outside the own-write window). A real edit made while the hook was missed still re-renders once.
- **A render that cannot complete stops** after 2 `process` calls (issue + the one re-issue, MC-16) and the Health page
  shows "Previews — Capture One produced no preview for DSC….ARW" until the photo renders or is adjusted again.
- **Masters pause on a nearly full disk.** Before ordering a master (full-size TIFF) the adapter checks the free space on
  the disk holding the Capture One session; below 5 GB it orders nothing, the export fails with "not enough disk space", and
  Health shows "Disk space — Only 3.0 GB free on the startup disk. Full-size renders for export are paused until 5 GB are
  free." Nothing is ever deleted.
- **Leftover `running` render rows** (274 in the owner's database) already recover at a real start (requeued, then handled
  by `request_initial`, which orders nothing for a current preview); with completions recognised they no longer pile up.
  C3(c) proves both.

## Tests → criterion rows

| Row | Test | Boundary / what it asserts |
|---|---|---|
| C1(a) | C++ `P56 C1(a) a plug-in closes Wi-Fi, claims USB at once, repeats through the window, then Wi-Fi at once` | mock camera call log with fake time (production timings, 2.5 s window): `disconnect` then `discover:usb` ≤ 300 ms after the arrival; USB attempt starts ≤ 500 ms apart, none after t0+2.5 s, ≥ 5 of them (> 3, yet no fallback until the window ended, with `usb_failures` = the window's count); `discover:wifi` ≤ t0+2.8 s; Wi-Fi then kept for 30 s with no USB probe. Held case: one USB connect, Wi-Fi discover ≤ 300 ms after the refusal. Success case: USB session on the 3rd attempt, Wi-Fi not reconnected. |
| C1(b) | C++ `P56 C1(b) a plug-in during a protected operation waits for busy to clear, then claims at once` | busy `exposure` at the plug-in: Wi-Fi kept, `switch.deferred` with `busy:exposure`, no USB call, no session closed for 6 s; busy cleared → `disconnect` then `discover:usb` ≤ 300 ms, USB session. |
| C2(a) | C++ `P56 C2(a) held cable: Wi-Fi for 30 minutes without USB probes, USB within 1 s of the release` | every USB connect refused "held": Wi-Fi connect starts ≤ 1 s, one fallback; 30 simulated minutes (holder reported each minute): no `connect:usb`/`discover:usb`, no `session.closed`/`session.lost`, no new `connection.ready`; holder-released input → `disconnect` then `discover:usb` ≤ 1 s later, `usb.released`, USB session. A release refused again → Wi-Fi discover ≤ 300 ms after the refusal. |
| C2(b) | C++ `P56 C2(b) never idle: a lost Wi-Fi session is retried for as long as it takes` | after C2(a)'s fallback, Wi-Fi lost and unreachable for 30 min: ≥ 10 Wi-Fi and ≥ 10 USB attempts in the last 10 minutes, never ERROR, no `transport.idle`; Wi-Fi back → `CONNECTED_WIFI`, one session. |
| C3(a) | `p56…::c3a_suffixed_export_is_recorded_once_and_never_ordered_again` | fake Capture One with add-suffix naming and sidecar rewrite on processing; this morning's `DSC04767.jpg` present: `DSC04767 1.jpg` written, `DSC04767.jpg` reported → recorded once (gen 1/1, preview bytes = that file, one `render_preview` row done, none running); an hour of focus every minute + document saves every 10 min → the fake received exactly **one** `process`; control: a real adjustment change → exactly one more. |
| C3(b) | `…::c3b_render_that_never_produces_output_stops_after_two_attempts` | no output ever: 2nd `process` at 61 s, failure at 122 s with Health detail exactly `Capture One produced no preview for DSC04800.ARW`; an hour of focus → still exactly 2; no row left running. **N = 2.** |
| C3(c) | `…::c3c_restart_with_every_preview_current_orders_nothing` | two current previews, a leftover `running` render row, sidecars rewritten while closed; restart with startup, ticks and focus → no `process` call, no `running` row, generations unchanged. |
| C3(d) | `…::c3d_master_is_rendered_once_and_not_at_all_on_a_nearly_full_disk` | master written as `DSC04979 1.tif` and reported as this morning's `DSC04979.tif` → export completes; 10 min of ticks + a second export → exactly one `PSM Master` `process`. At 3 GB free: export refused, zero `PSM Master` `process` for the new photo, Health detail names the disk exactly; existing export and Output files still there. |
| core mechanism | `renders::tests::p56_completion_resolves_capture_ones_suffixed_copy` (unit) | completion matching: newest fresh `<stem> <n>.<ext>`; never another stem, another format, `.tmp`, non-numeric suffix; fresh reported file wins; nothing fresh → `None`; reported file missing → suffixed copy. |
| core mechanism | C++ `P56 holder report: only another process counts as holding the cable` (unit) | `UsbExclusiveOwner` values: own pid not held; other pid / unparsable held. |

## Assumptions

1. **Claim window retry interval 200 ms** (delegated, ≤ 500 ms); the sidecar loop's 100 ms tick puts attempt starts ≤ ~410 ms
   apart when a refusal is fast (~10 ms, evidence). Window length = the existing `usb_settle_ms` (2.5 s).
2. **The window's end parks USB whether or not a holder was seen** (Wi-Fi at once, then USB only on re-plug, holder release
   or `usb-retry`). Book changelog 37 says USB is "never [retried] by blind probing that would drop a working Wi-Fi session";
   regular counted attempts after the window would do exactly that. The three-failure rule stays for USB attempts outside a
   window (a USB session lost with the cable in, a camera found on USB by discovery). Cost: a camera whose interface needs
   more than 2.5 s to come up and that nobody holds stays on Wi-Fi until re-plug or Retry (the app's Retry sends `usb-retry`).
3. **The helper itself watches the holder** (delegated: helper or app): IOKit `UsbExclusiveOwner` read every 500 ms while
   plugged in and not on USB; "held by another" = any owner whose pid is not the helper's. A release is acted on only after a
   hold was seen (from the poll or from a refused connect).
4. **While USB waits and Wi-Fi is unreachable** the two links alternate on the normal backoff ("USB attempts continue on the
   existing backoff", ruling 2); these USB attempts do not re-raise the fallback event.
5. **Completion matching** (delegated): by job id as before, the output by path — reported path if fresh, else Capture One's
   newest fresh suffixed copy.
6. **N = 2** for C3(b) (MC-16's issue + one re-issue; ≤ 3).
7. **"Not an adjustment" = same adjustment lists.** The sidecar fingerprint is SHA-256 over its `<DL>…</DL>` lists (whole
   text if none). It is recorded at each preview completion (`none` when no sidecar exists) and at each check. A RAW with no
   fingerprint yet (rendered before this phase) adopts its current sidecar silently: no re-render wave at the first start
   after the update, at the price of missing an edit made while the app was closed **before** this update.
8. **Disk guard** measures the disk holding the Capture One session folder (Capture One's output root cannot be read by
   script, MC-16; it defaults to the session's `Output/`). 5 GB = 5 000 000 000 bytes. The system volume is named "the
   startup disk", another volume by its name. An unreadable disk never blocks a render. Only masters are guarded (previews
   are ~0.5 MB JPEGs).
9. **New health codes** `preview_not_rendered` and `disk_space_low` (Health page names "Previews", "Disk space"). For the
   master plan's §6.6 registry (orchestrator).

## Deviations

- **Files outside the listed perimeter** (all small, each needed by a row):
  `crates/infra/src/disk.rs` (new, the `statfs` free-space query) + `crates/infra/src/lib.rs` (module line);
  `crates/infra/src/sqlite/migrations/008_settings_fingerprint.sql` (new) + `crates/infra/src/sqlite/store.rs` and
  `crates/core/src/ports/store.rs` (the fingerprint column, read with the preview state, one setter);
  `crates/core/src/ports/editing.rs` (`accept_render_output → resolve_render_output`, `settings_changed_at → settings_sidecar`;
  the Capture One adapter is the only implementation); `crates/integrations/src/lib.rs` (`Lanes.disk`, `App::disk_space()`,
  preview health wiring); `crates/testkit/src/test_app.rs` (renders written through the fake's naming; a roomy disk by default
  so no test depends on the host's free space); `apps/desktop/src/features/health/HealthPage.tsx` (two labels).
- **Old assertions changed** (rulings / schema):
  - `test_transport_manager.cpp` "repeated usb connect failure falls back to wifi": exact `connect:usb == 3` → one fallback
    and no USB connect afterwards (claim-window attempts are not the three counted ones).
  - "P54 C3(a) …": `usb_connects == usb_failures_before_wifi_fallback` removed (same reason); the rest unchanged and passing.
  - "replugging usb after fallback …": first phase now fails every USB connect until the fallback (with 3 failures the window
    itself would reach USB), then the re-plug succeeds.
  - "usb flapping within settle window …": `session.closed == 0` removed (ruling 1 closes Wi-Fi at every plug-in); still ends
    on Wi-Fi with one session.
  - "usb-retry gives usb a fresh chance …": counts made relative (`before + 1`, control unchanged over 60 s).
  - Mock camera gained `usb_held`, `usb_fail_cost_ms`, a timed call log; `Rig(true)` = production timings.
  - `p01_storage_identity::c1a_open_creates_all_domain_columns`: `raw_state` lists the new `settings_fingerprint` column.
  - `p27_retire_shape_on_arrival::c4a_…`: setup also drops `settings_fingerprint` before rewinding `user_version` (as it does
    for 007's column); no assertion changed.
- `seen_sidecars` (in-memory) removed from `PreviewService`: the recorded fingerprint replaces it and survives a restart.

## Files changed

Production: `native/camera-sidecar/include/crlab/transport.hpp`, `native/camera-sidecar/src/transport_manager.cpp`,
`native/camera-sidecar/src/sdk_adapter.cpp`, `native/camera-sidecar/src/main.cpp`, `crates/core/src/ports/editing.rs`,
`crates/core/src/ports/store.rs`, `crates/core/src/services/previews.rs`, `crates/infra/src/disk.rs` (new),
`crates/infra/src/lib.rs`, `crates/infra/src/sqlite/migrations/008_settings_fingerprint.sql` (new),
`crates/infra/src/sqlite/store.rs`, `crates/integrations/src/editing/capture_one/adapter.rs`,
`crates/integrations/src/editing/capture_one/renders.rs`, `crates/integrations/src/lib.rs`,
`apps/desktop/src/features/health/HealthPage.tsx`.
Tests and harness: `native/camera-sidecar/tests/test_transport_manager.cpp`, `crates/testkit/src/fake_capture_one.rs`,
`crates/testkit/src/test_app.rs`, `crates/testkit/tests/p56_camera_claim_and_render_loop.rs` (new),
`crates/testkit/tests/p01_storage_identity.rs`, `crates/testkit/tests/p27_retire_shape_on_arrival.rs`.
Docs: `docs/implementation/master_plan.md` (P56 tracker row only), `docs/implementation/plans/P56_camera_claim_and_render_loop.md`
(Review log), this handoff.
No owner data was written: the owner's database, log, `settings.json` and Capture One folders were only read (`sqlite3 -readonly`,
`stat`, `ls`, `grep`).

## Checkpoint SHA

The commit containing this handoff is the checkpoint (a file cannot hold its own commit's SHA):
`git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P56_camera_claim_and_render_loop_implement_r1.md`.
The full SHA is reported in the session's final message.

## Stamp

Run on the final tree (2026-11-08, dev Mac):
- `cargo test --workspace` → exit 0, **841 passed, 0 failed** (incl. `p56_camera_claim_and_render_loop` 4/4 and
  `renders::tests::p56_completion_resolves_capture_ones_suffixed_copy`).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0. `cargo fmt --all -- --check` → exit 0.
- `cd apps/desktop && npm run typecheck` → exit 0; `npm test` → **411 passed / 411** (56 files; the known intermittent
  `p33_split_grid` did not occur).
- `tools/sidecar/build.sh --stub --test` → **27 tests, 0 failed** (22 before + 5 P56 cases).
- SDK build: `tools/sidecar/build.sh --sdk /Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk` compiled
  every object fresh with no error or warning; the stub was rebuilt afterwards (`--stub`); nothing from either build is committed.
- Discrimination check (done by the implementer, reverted): disabling the suffixed-output resolution fails C3(a), C3(c),
  C3(d); disabling the fingerprint comparison fails C3(a), C3(b), C3(c).
- No pre-existing failure.

## Manual try-out

1. Build and install as usual (the helper needs the SDK build: `tools/sidecar/build.sh --sdk <sdk root>`, then package).
2. **Plug-in with Capture One running:** camera on Wi-Fi (dot `Wi-Fi`), plug the cable in. Within about a second the dot
   reads `Cable`; `ioreg -l -r -n "ILCE-7RM5" | grep UsbExclusiveOwner` names `camera-sidecar`. No "Take the camera back".
3. **If another program got there first** (the dot names it): the dot is back on `Wi-Fi` within about a second and stays
   there. Quit that program (or let it release) — within about a second the dot switches to `Cable`.
4. **Leave it an hour** with the cable in and Capture One open: the dot is `Cable` or `Wi-Fi` the whole time, never
   disconnected while the camera is on.
5. **Export loop:** work normally for a while, switch between Capture One and Session Manager often. Capture One shows its
   export window only when a photo was actually edited; `Output/_session_manager/` gets one new file per edit, no growing
   `name 1.jpg, name 2.jpg …` series.
6. **Disk guard** (optional): with less than 5 GB free on the session's disk, press Export: it refuses and Health shows
   "Disk space — Only … GB free on …".

## ⚠ OWNER DECISIONS REQUIRED (0)

None. Two things the orchestrator should know (no decision needed): the owner's Capture One "PSM Preview" recipe was in a TIFF /
`Output` / add-suffix state on 2026-11-08 (the app checks and repairs recipe properties only when it provisions Capture One at start; a restart of
Session Manager should re-apply JPEG / `_session_manager` / overwrite); and a camera interface that is not ready within 2.5 s and not held by anyone now waits
on Wi-Fi until re-plug or Retry (Assumption 2).
