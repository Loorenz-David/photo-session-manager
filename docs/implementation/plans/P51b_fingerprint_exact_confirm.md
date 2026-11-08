---
phase: P51b_fingerprint_exact_confirm
title: Fingerprint confirmation pins exactly the fingerprint the photographer compared — Confirm carries the shown value through the app to the camera helper, which pins only that value; a changed key while Settings is open is titled "Fingerprint changed"
difficulty: normal
projection: no
depends_on: [P51_camera_setup_ui]
---
# P51b — Exact fingerprint confirmation (10E MC-31 §Fingerprint; backlog P50 r1 "hardening", P51 r2 N5)

Written by the orchestrator on 2026-11-08, on the owner's request ("if it is fixed we can move to the next stage"), before P52.

## Goal
Today **Confirm** arms a process-wide flag in the camera helper (`g_accept_fingerprint`, `native/camera-sidecar/src/camera_connection.cpp:21–26`) that pins
**whatever** fingerprint the next SSH connect presents, not the one the photographer compared. A `replace=1` confirmation also deletes the pinned file at the
next check (`:313`) whatever key is presented then. 10E MC-31 says a fingerprint is "pinned only on their confirmation". After this phase:
1. **Confirm sends the value shown.** The UI passes the exact fingerprint string it displayed; the app accepts the confirmation only when it equals the adapter's
   current pending fingerprint (otherwise error `fingerprint_changed`, and nothing is sent to the helper).
2. **The helper pins only that value.** `accept-fingerprint fp=<percent-encoded display form> [replace=1]` stores an expectation. At the next fingerprint
   check the helper pins (or, with `replace=1`, replaces the pinned file) **only if** the presented key's display form equals the expectation. A different
   key is treated exactly as today without a confirmation (`unknown` or `MISMATCH`, with its value, `success:false`), nothing is written or deleted, and the
   expectation is dropped. A matching key consumes the expectation.
3. **"Fingerprint changed" while Settings is open (N5).** `camera_fingerprint_pending` carries `replace: bool` (true for `MISMATCH`), so the card is titled
   `Fingerprint changed` whenever the pending fingerprint replaces a pinned one, also when it arrives after mount.

**Not in this phase:** N6 ("Not mine" stops retries), any other backlog item, hardware (P52).

## Read first
Book: `docs/intention/book/10E_direct_camera_adapter.md` MC-31 §Fingerprint. Backlog `docs/implementation/backlog.md` lines with "P50 reviewer r1, hardening"
and "P51 reviewer r2 (2026-11-08), by reading → frontend polish". Code: `native/camera-sidecar/src/camera_connection.cpp` (`accept_fingerprint_on_next_connect`,
`fingerprint_for_display`, `check_fingerprint`), `native/camera-sidecar/src/main.cpp:540–541` (command parsing; `c.get(..)` and the percent-decoding in
`crlab_core`), `native/camera-sidecar/CMakeLists.txt` (`crlab_core`, `camera_sidecar_tests`), `native/camera-sidecar/tests/test_transport_manager.cpp` +
`mini_test.hpp` (test style), `crates/integrations/src/camera/sidecar/protocol.rs` (`Command::AcceptFingerprint`, `percent_encode`),
`crates/integrations/src/camera/direct/adapter.rs` (`pending_fingerprint`, `confirm_fingerprint`, the `SshFingerprint` handling `:593–613`),
`crates/integrations/src/camera/runtime.rs:770–776` (`AppEvent::FingerprintPending`), `crates/core/src/error.rs` (`FingerprintNotPending`),
`apps/desktop/src-tauri/src/commands/{camera.rs, items.rs}` (command + error code mapping), `apps/desktop/src/services/{camera.ts, events.ts, bindings.ts}`,
`apps/desktop/src/features/settings/CameraSection.tsx`, tests `crates/testkit/tests/p50_camera_settings_and_support.rs` (C7(b)–(e)) and
`apps/desktop/src/features/settings/__tests__/p51_camera_section.test.tsx` (C6).

## Files expected to change (write perimeter)
Helper: `native/camera-sidecar/include/crlab/fingerprint_policy.hpp` (new), `native/camera-sidecar/src/fingerprint_policy.cpp` (new, in `crlab_core` so the
stub/test build compiles it), `native/camera-sidecar/src/camera_connection.cpp`, `native/camera-sidecar/src/main.cpp`, `native/camera-sidecar/CMakeLists.txt`,
`native/camera-sidecar/tests/test_fingerprint_policy.cpp` (new; or new cases in the existing test file). Rust: `crates/integrations/src/camera/sidecar/protocol.rs`,
`crates/integrations/src/camera/direct/adapter.rs`, `crates/integrations/src/camera/runtime.rs`, `crates/core/src/error.rs`, the `AppEvent` definition file
(`FingerprintPending` gains `replace`), `apps/desktop/src-tauri/src/commands/{camera.rs, items.rs}`, `crates/testkit/tests/p50_camera_settings_and_support.rs`
(C7(b)–(d) assertions on the `accept-fingerprint` line and the `confirm_fingerprint` call only), `crates/testkit/tests/p51b_fingerprint_exact_confirm.rs` (new),
the testkit fake sidecar only if needed to record lines (no behaviour change for other tests). Frontend: `apps/desktop/src/services/{camera.ts, events.ts,
bindings.ts}` (regenerated), `apps/desktop/src/features/settings/CameraSection.tsx`, `apps/desktop/src/features/settings/__tests__/p51_camera_section.test.tsx`
(C6 rows only), and any frontend test file whose `@/services/camera` or `@/services/events` mock must follow the changed signatures (mock lines only).
Pipeline files as usual (this plan's Review log, master plan tracker row, handoff). Estimated ≈ 18 paths, ≈ 450 lines.

## Core mechanisms (unit tests allowed)
The helper's fingerprint decision (`fingerprint_policy`): a pure function, tested in C++ with `camera_sidecar_tests` (`tools/sidecar/build.sh --stub --test`).

## The contract
- **Helper decision** `decide(pinned: optional<string>, presented: string /* display form */, expectation: optional<{fp: string, replace: bool}>) -> Outcome`:
  - pinned present and equal (raw compare as today) → `Match` (expectation untouched);
  - pinned present, differs: expectation with `replace` and `fp == presented` → `Replace` (consume); otherwise → `Mismatch` (drop expectation);
  - no pinned file: expectation with `fp == presented` (replace either way) → `Pin` (consume); otherwise → `Unknown` (drop expectation).
  The `--accept-fingerprint` CLI flag (diagnostics, operator at the terminal) keeps its meaning: pin whatever is presented when nothing is pinned.
  `check_fingerprint` writes or deletes the pin file only for `Pin`/`Replace`, and emits the same `ssh.fingerprint` events as today (`match`, `pinned`,
  `MISMATCH`, `unknown`, with `fingerprint` = display form). The pin file content stays the raw form the SDK gives (unchanged format).
- **Helper command:** `accept-fingerprint fp=<percent-encoded display form> [replace=1] [cmd_id=…]`. Missing `fp` → `cmd.rejected` with reason
  `fingerprint required`, nothing armed. The expectation is held until consumed or dropped as above; a new `accept-fingerprint` replaces it.
- **Rust:** `Command::AcceptFingerprint { fingerprint: String, replace: bool, cmd_id }` encodes as above (`percent_encode`). `DirectCameraAdapter::confirm_fingerprint(&self, shown: &str)`:
  no pending → `Error::FingerprintNotPending` (unchanged); pending value ≠ `shown` → new `Error::FingerprintChanged` (code `fingerprint_changed`),
  nothing sent; equal → send with the pending `replace`. The command `camera_fingerprint_confirm(fingerprint: String)`. `AppEvent::FingerprintPending { fingerprint, replace }`.
- **UI:** `cameraFingerprintConfirm(fingerprint)`; Confirm passes the displayed value. `fingerprint_changed` → toast `The camera shows a different fingerprint now. Compare the new one.`
  and the card stays open (the next event brings the new value). `onFingerprintPending` handler receives `{fingerprint, replace}`; `replace: true` → title
  `Fingerprint changed`, false → `Confirm camera fingerprint` (the mount-time status still sets the title until an event arrives).

## Ordered tasks
0. `fingerprint_policy` + C++ tests; wire into `check_fingerprint` and `main.cpp`. Build stub with tests; if the SDK is available locally
   (`~/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk`), also run `tools/sidecar/build.sh --sdk <that path>` to prove it compiles, then
   `tools/sidecar/build.sh --stub` to put the stub back. Never commit build output or any SDK file.
1. Rust protocol, adapter, error, event, command; p50 C7 assertions; p51b testkit file. 2. Bindings, services, `CameraSection`, C6 test rows.
3. Closing gate: `cargo test --workspace`, clippy, fmt, `npm run typecheck`, `npm test`, `tools/sidecar/build.sh --stub --test`.

## Criteria (5 criteria, 12 rows; all `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | helper decision: pin only the confirmed key | no pin; expectation `{fp:"aa:bb", replace:false}`; presented `aa:bb` | `Pin`; expectation consumed | `[rule]` MC-31 ("pinned only on their confirmation") |
| C1(b) | helper decision: a different key is not pinned | no pin; expectation `{fp:"aa:bb"}`; presented `cc:dd` | `Unknown`; expectation dropped (a following `aa:bb` with no new expectation → `Unknown`) | `[rule]` MC-31 |
| C1(c) | helper decision: replace only the confirmed key | pin `X` ≠ presented; expectation `{fp: presented, replace:true}` → `Replace`; expectation `{fp:"other", replace:true}` → `Mismatch` and expectation dropped | as stated | `[rule]` MC-31 §Fingerprint (changed key) |
| C1(d) | helper decision: unchanged cases | pin equal to presented, any expectation → `Match`; no pin, no expectation → `Unknown`; pin differs, no expectation → `Mismatch` | as stated | `[rule]` MC-31 |
| C2(a) | helper command parsing | `accept-fingerprint replace=1` with no `fp` | `cmd.rejected` reason `fingerprint required`; nothing armed (test at the parse/arm boundary the implementer exposes; C++ test) | `[rule]` MC-31 |
| C3(a) | app sends the shown value | pending `ab:cd:ef` (script `fingerprint_unknown.txt`); `confirm_fingerprint("ab:cd:ef")` | the fake helper's stdin has `accept-fingerprint fp=ab%3Acd%3Aef` (exact percent form of `percent_encode`) followed by ` cmd_id=`; no `replace=1` | `[rule]` MC-31 |
| C3(b) | app refuses a stale confirmation | pending `ab:cd:ef`; `confirm_fingerprint("11:22")` | `Err(FingerprintChanged)`; no `accept-fingerprint` line sent; still pending | `[rule]` MC-31 |
| C3(c) | mismatch confirmation | pending from `MISMATCH` `ef:12`; `confirm_fingerprint("ef:12")` | stdin line contains `fp=ef%3A12` and `replace=1` | `[rule]` MC-31 §Fingerprint (changed key) |
| C3(d) | event carries replace | `MISMATCH` `ef:12` / `unknown` `ab:cd:ef` | `camera_fingerprint_pending` emitted with `{fingerprint:"ef:12", replace:true}` / `{fingerprint:"ab:cd:ef", replace:false}` | `[rule]` 10E §What the photographer gets |
| C4(a) | UI passes the displayed value | event `{fingerprint:'ab:cd:ef', replace:false}`; click Confirm | `cameraFingerprintConfirm('ab:cd:ef')`; card closes | `[rule]` MC-31 |
| C4(b) | UI stale confirmation | `cameraFingerprintConfirm` rejects `{code:'fingerprint_changed'}` | toast `The camera shows a different fingerprint now. Compare the new one.`; the card stays open | `[rule]` MC-31 |
| C4(c) | "Fingerprint changed" after mount | mount with a connected status; then event `{fingerprint:'ef:12', replace:true}` | card title `Fingerprint changed`, value `ef:12`, Confirm and Not mine shown | `[rule]` 10E MC-31 §Fingerprint; backlog N5 |
| C5(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (the known intermittent `p33_split_grid` case aside, recorded if red once and green on rerun), sidecar tests, clippy, fmt, typecheck green | `[rule]` master plan §7.2 |

## Delegated choices
Where the expectation lives in the helper (a mutex-guarded optional replaces the two atomics); the exact name and file of the parse/arm seam used by C2(a);
whether C1 cases sit in a new test file or the existing one.

## Manual try-out
Covered by P52 (the Wi-Fi fingerprint step): with the pinned file removed, the card shows the camera's fingerprint; Confirm pins it; the next connect says
`match`. The orchestrator may also verify with the owner's camera before P52.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint (tests use made-up values
  like `ab:cd:ef`); no Python; never change the camera's network/FTP settings, delete camera files or kill any process; live view out of scope; the suite never
  needs the camera. Lab repo read-only. Default implementer (no `implementer: fable`).
- **Old test changes allowed:** only the `accept-fingerprint` line assertions and `confirm_fingerprint()` calls in p50 C7(b)–(d) (they now pass the pending
  value and expect `fp=`), the C6 rows of the p51 Settings test, and service-mock signatures. No other old assertion may change.
- **Hazards:** the display form contains `:` (and may contain other characters SDK-dependent), so it must be percent-encoded on the wire and decoded by the
  helper's existing parameter decoding; compare display form to display form. The raw pin file compare stays raw. Make sure the stub build still links
  (`fingerprint_policy.cpp` in `crlab_core`, no SDK types in it).
- No intention-book change: this enforces MC-31 as written.

## Review log
