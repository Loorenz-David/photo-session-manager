---
phase: P53_camera_upload_and_log
title: Camera uploads after P52 — the receiver answers "transfer complete" as soon as the file is safely on disk (no more double uploads), the photographer is told when the camera has stopped sending, and the helper's and receiver's events are in the app log
difficulty: normal
projection: no
depends_on: [P51b_fingerprint_exact_confirm]
---
# P53 — Upload reply, "camera stopped sending", camera log lines (P52 findings F9, F8, F2)

Written by the orchestrator on 2026-11-08 from P52 run 1 (evidence `docs/intention/evidence/sony_camera_lab.md` §Acceptance (P52, 2026-11-08)). The owner:
"it is better to fix what we have gathered and later when fixed we finish the tests".

## Goal
1. **F9: no double uploads.** In a 20-shot Wi-Fi session, 16 of 18 photos were uploaded twice by the camera (byte-identical second copies in `.duplicates/`).
   The receiver sends `226 Transfer complete` only after it has hashed the 45 MB file and the ledger has filed it (`crates/infra/src/ftp/session.rs`:
   `upload` → `sha256_file` → `sink.delivery` → `226`). Under load that takes seconds and the camera re-sends. After this phase the reply follows the data
   connection's clean close at once; hashing and filing happen afterwards, and a crash between the two loses nothing.
2. **F8: the camera stopped sending.** After one failed upload (receiver unreachable), the A7R V puts its FTP function in an error state and sends nothing
   more, new shots included, until FTP Function is switched off and on. It does **not** re-send by itself (10E's A10 expectation is superseded by the
   measurement). The app logged `camera.sequence_gap` at INFO every second and showed nothing. After this phase a gap in the camera's file numbers that is
   still open 60 s later is shown in the header like the existing "not received" state, with what to do on the camera.
3. **F2: readable logs.** Every helper event (stdout protocol line) and every receiver upload is one INFO line in the app log, with no secret; the
   per-call `osascript` provider lines move to DEBUG (they are ~66 000 lines a day and bury everything else); a repeated identical `camera.sequence_gap` is
   logged once per change, not every second.

**Not in this phase:** the cable/take-back findings (P54), install/capture-button/restart findings (P55), preview speed (F11), any camera-setting change
(the app never changes the camera's FTP settings: the photographer toggles FTP Function by hand).

## Read first
Evidence `sony_camera_lab.md` §Acceptance (P52, 2026-11-08) rows A10, A15. Book `docs/intention/book/10E_direct_camera_adapter.md` MC-32 (§Upload lifecycle:
"Completed means the data connection closed with no socket error … On completion the receiver hashes the `.part` (outside the transfer timing) and hands … to the ledger")
and MC-33 (§First complete copy wins, §Watchdog, "Gaps in camera file numbering are reported"); `12_operations.md` §Logs. Code: `crates/infra/src/ftp/session.rs`
(`upload`, the `STOR`/`APPE` arm, `LIST`/`NLST`/`SIZE`/`MDTM` and `completed`), `crates/infra/src/ftp/` (receiver start), `crates/core/src/services/camera_ledger.rs`
(`FILES_NOT_RECEIVED`, the watchdog, where `camera.sequence_gap` is logged), `crates/core/src/mechanisms/capture_ledger.rs` (`sequence_gaps`),
`crates/core/src/domain/camera.rs` (`NeedsActionReason::FilesNotReceived`), `crates/core/src/mechanisms/camera_status.rs` (its sentence),
`crates/integrations/src/camera/runtime.rs` (stderr → `camera sidecar:` WARN, `redacted`), `crates/integrations/src/camera/sidecar/{client.rs, protocol.rs, supervisor.rs}`,
`crates/infra/src/osascript.rs` (the `provider call` line). Tests: `crates/testkit/tests/{p43_ftp_receiver.rs, p44_capture_ledger.rs, p46_direct_camera_adapter.rs}`.

## Files expected to change (write perimeter)
`crates/infra/src/ftp/session.rs` (+ the receiver's start/sweep file in `crates/infra/src/ftp/` if the startup recovery lives there),
`crates/core/src/services/camera_ledger.rs`, `crates/core/src/mechanisms/capture_ledger.rs` (only if the gap age needs a helper),
`crates/core/src/domain/camera.rs`, `crates/core/src/mechanisms/camera_status.rs`, `crates/integrations/src/camera/runtime.rs`,
`crates/integrations/src/camera/sidecar/{client.rs or supervisor.rs}` (where stdout lines are read), `crates/infra/src/osascript.rs`,
the frontend camera sentence map if the new reason needs one (`apps/desktop/src/features/**/camera*` — find it by the `camera_files_not_received` code),
`crates/testkit/tests/p53_camera_upload_and_log.rs` (new), and existing tests only where the contract below changes an asserted value (list each in the handoff).
Pipeline files as usual. Estimated ≈ 12 paths, ≈ 450 lines.

## Core mechanisms (unit tests allowed)
The gap-notice decision (which gaps are open and old enough) if it is a pure function.

## The contract
- **Reply first, then file.** When the data connection closes cleanly, the receiver makes the upload durable as *completed* with one cheap step (for example
  rename `.<name>.part` → `.<name>.done` in `.incoming/`, or an equivalent marker), replies `226`, and only then hashes and calls `sink.delivery` with the
  completed path. Failure paths are unchanged (`426`, the `.part` stays for a resume). If hashing or delivery fails after `226`, the file stays in
  `.incoming/` as completed and is retried at the next receiver start. **At receiver start, every completed-but-undelivered file in `.incoming/` is hashed
  and delivered once** (the ledger's first-copy-wins rule makes a repeat harmless). `.part`/completed files are never deleted (MC-32).
- While a completed file is waiting to be filed, `LIST`/`NLST`/`SIZE`/`MDTM` report it as present (the camera must not conclude it is missing); a new
  `STOR` of the same name while it waits is answered as today for a name in flight or completed (no truncation of the waiting file).
- **Camera stopped sending.** The ledger keeps the open gaps it already computes. A gap still open 60 s after it was first seen (`WATCHDOG_MIN_AGE_MS`)
  raises `FilesNotReceived { count }` (count = the missing numbers, plus any watchdog exposures, no double counting) — the same header state the watchdog
  uses, through the same path. The sentence (core `camera_status`, and the UI if it has its own) becomes:
  `{count} photos were taken but not received. The camera may have stopped sending: on the camera, switch FTP Function off and on.`
  A gap closes when the missing file arrives (the state clears by itself) or by the existing dismiss. Gaps older than `WATCHDOG_MAX_AGE_MS` and gaps across
  a camera counter reset (lower number after a higher one, or a different prefix) are not reported.
- **Log lines (INFO, target `psm::camera`):** `camera.event <name> k=v …` for every helper stdout event (all fields except any field named
  `password`, `user`, `pass`, `credentials` or `fingerprint`, which are written as `***`; the fingerprint value is never logged); receiver:
  `ftp.upload started name=<n>`, `ftp.upload completed name=<n> size=<bytes> ms=<transfer ms> reply_ms=<close→226 ms>`, `ftp.upload failed name=<n> reason=<r>`,
  `ftp.upload filed name=<n> outcome=<photo|duplicate|dup_renamed> hash_ms=<ms>`; `camera.sequence_gap` once per change of the open-gap set.
  `osascript` `provider call` lines: DEBUG (errors stay WARN).

## Ordered tasks
0. Receiver: reply-first, completed marker, startup recovery, listings. 1. Ledger gap notice + sentence. 2. Log lines. 3. Tests. 4. Closing gate:
`cargo test --workspace`, clippy `-D warnings`, fmt, `npm run typecheck`, `npm test`.

## Criteria (6 criteria, 8 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | receiver: reply before filing | a test sink whose `delivery` blocks until released; FTP client `STOR DSC00001.ARW` of 1 MB, closes the data connection | `226` arrives while `delivery` is still blocked (assert before releasing it); after release, exactly one delivery with the correct SHA-256 and size | `[rule]` MC-32 §Upload lifecycle; P52 A15 |
| C1(b) | receiver: crash between reply and filing | as C1(a), then stop the receiver before releasing; start a new receiver on the same capture root with a normal sink | the file is delivered exactly once by the new receiver (correct hash), no file deleted from `.incoming/` before delivery succeeded | `[rule]` MC-32 (".part files … never deleted"); M37 |
| C1(c) | receiver: waiting file is visible | during C1(a)'s blocked delivery: `SIZE DSC00001.ARW` | `213 <size>` (present) | `[rule]` MC-32 |
| C2(a) | ledger: open gap becomes a header state | files `DSC00010`, `DSC00013` delivered; clock +61 s | camera status needs-action `camera_files_not_received` with count 2 and the new sentence; at +59 s nothing raised | `[rule]` MC-33 (gaps reported); P52 A10 |
| C2(b) | ledger: gap closes itself | C2(a) raised; then `DSC00011`, `DSC00012` delivered | the state is cleared with no dismiss | `[rule]` MC-33 |
| C2(c) | ledger: counter reset is not a gap | `DSC09998`, then `DSC00002` (reset); clock +61 s | nothing raised | `[rule]` MC-33 |
| C3(a) | app log: helper events and uploads | fake helper emits `connection.ready transport=wifi …` and an event with `password=secret`; one FTP upload | the captured log has `camera.event connection.ready` at INFO with `transport=wifi`; no line contains `secret`; `ftp.upload completed name=DSC00001.ARW` and `ftp.upload filed … outcome=photo` at INFO | `[rule]` 12 §Logs; P52 F2 |
| C4(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (the known intermittent `p33_split_grid` case aside, recorded if red once and green on rerun), clippy, fmt, typecheck green | `[rule]` master plan §7.2 |

## Delegated choices
The completed marker's form (rename vs. a sidecar file) as long as C1(b) holds; where the startup recovery runs; how the log is captured in the test
(a `tracing` test subscriber is fine); the `camera.sequence_gap` de-duplication mechanism.

## Manual try-out
P52 run 2 reruns A15 (20 shots over Wi-Fi: nothing new in `.duplicates/`, and each upload's `reply_ms` is in the log) and A10 (port changed and back: within
about a minute the header says the camera may have stopped sending and to switch FTP Function off and on; after the toggle the next shot arrives).

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, a `.env`, a credential or an SSH fingerprint; no Python; never change the camera's network/FTP
  settings, delete camera files or kill any process; live view out of scope; the suite never needs the camera. Lab repo read-only. Default implementer.
- **Hazards:** the camera keeps its control connection open between uploads (evidence S7), so the reply must not wait on anything but the data close.
  Do not hold the `active` name reservation across the hashing in a way that answers `450` to a legitimate later `STOR` of a *different* name.
  The ledger's duplicate rule already makes a repeated delivery harmless; do not add a second dedupe path.
- Intention: this applies MC-32 as written ("hashes … outside the transfer timing"). The A10 expectation in 10E §Hardware acceptance is superseded by the
  P52 measurement (recorded in the evidence); no other book change.

## Review log

- **2026-11-08 implement r1 (codex).** The FTP receiver now renames a clean transfer to `.done` before `226`, then hashes/files asynchronously; startup retries completed markers and FTP listings include them. The ledger raises the existing not-received notice for aged numbering gaps, clears it when gaps close, and logs a changed gap set once. Sidecar stdout events and FTP upload stages have redacted INFO lines; provider calls are DEBUG and failures WARN. Seven P53 boundary tests pass; existing P16/P43/P44/P46 assertions were updated only where the reply timing or sentence changed. Full Rust/UI suites, clippy, fmt and typecheck green. Assumptions: a `.done` rename is the durable completion marker permitted by the plan; the later numbered file's completion time starts a gap's 60 s timer; a lower number below all currently seen numbers of a prefix means counter reset, while a late arrival inside a known gap does not; the greater of watchdog exposure count and missing file-number count avoids double counting when they refer to the same shots. No hardware was used in this session. An optional `ps -eo pid,etime,command | rg 'cargo test --workspace|p[0-9][0-9]_.*-' | tail -8` diagnostic was denied (`zsh:1: operation not permitted: ps`); it was not needed for the phase or used as a workaround.
