---
plan: P53_camera_upload_and_log
role: implement
round: 1
state: DONE
date: 2026-11-08
actor: codex
---
# P53 implementation handoff

## What was built
The camera gets its FTP completion reply as soon as the clean transfer has a completed marker on disk. Hashing and filing continue afterward; a receiver restart recovers waiting files. A gap in camera file numbers raises the existing not-received notice after 60 seconds, with a camera-side remedy, and clears when the missing files arrive. The app log now records redacted helper events and each FTP upload stage at INFO; repetitive provider-call lines are DEBUG.

## Tests → criterion rows
| Row | Boundary test / result |
|---|---|
| C1(a) | `c1a_reply_precedes_filing`: 226 while sink blocked; then one delivery with exact size/hash |
| C1(b) | `c1b_completed_marker_recovered_at_start`: stop first receiver after 226, marker survives; second receiver delivers once with correct hash |
| C1(c) | `c1c_waiting_file_is_present_to_camera`: SIZE returns 213 and exact size while filing blocked |
| C2(a) | `c2a_gap_becomes_header_notice_after_a_minute`: absent at 59 s, count 2 and remedy at 61 s |
| C2(b) | `c2b_gap_closes_on_missing_files`: one missing file leaves count 1; both clear state |
| C2(c) | `c2c_counter_reset_has_no_gap`: 9998 → 0002 never raises |
| C3(a) | `c3a_camera_events_and_uploads_are_logged_without_secrets`: INFO event/upload lines and secret absent |
| C4(a) | Closing stamp below: workspace and UI suites, clippy, fmt, typecheck all green |

## Assumptions
- `.incoming/.<name>.done` is the completed marker; the receiver recovers markers in the current capture root when it starts.
- The later numbered file's completion time starts the gap's 60-second clock. A lower number below all earlier ones with the same prefix is a counter reset; a late file within a known gap is not.
- The notice count is the larger of pending exposures and missing file numbers, so a shot represented by both is counted once.
- Helper log redaction masks the named protocol fields and registered secrets.

## Deviations
The existing P16, P43, P44, and P46 tests changed where the new reply timing or sentence changed their asserted outcomes. No files outside the plan perimeter were changed. An optional process-status diagnostic, `ps -eo pid,etime,command | rg 'cargo test --workspace|p[0-9][0-9]_.*-' | tail -8`, returned `zsh:1: operation not permitted: ps`; it was not required for implementation or verification, and no alternative process inspection was attempted.

## Files changed
- `crates/core/src/mechanisms/{camera_status,capture_ledger}.rs`
- `crates/core/src/services/camera_ledger.rs`
- `crates/infra/src/ftp/{server,session}.rs`, `crates/infra/src/osascript.rs`
- `crates/integrations/src/camera/sidecar/client.rs`
- `crates/testkit/tests/{p16_operations,p43_ftp_receiver,p44_capture_ledger,p46_direct_camera_adapter,p53_camera_upload_and_log}.rs`
- `docs/implementation/{master_plan.md,plans/P53_camera_upload_and_log.md,handoffs/implementer/handoff_P53_camera_upload_and_log_implement_r1.md}`

## Checkpoint SHA
The commit containing this handoff is the checkpoint; resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P53_camera_upload_and_log_implement_r1.md` (a literal SHA here would change that commit's SHA).

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | green, final run |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `npm run typecheck` (apps/desktop) | green |
| `npm test` (apps/desktop) | green, 404 tests |

## Manual try-out
1. With the camera on Wi-Fi and the app open, take 20 photos about eight seconds apart. Confirm one upload per shot and no new file in `.duplicates/`; inspect each upload's `reply_ms` in the app log.
2. Temporarily make the receiver unreachable using the P52 A10 procedure, take a shot, then restore reachability. After about a minute, confirm the header says the camera may have stopped sending and instructs switching FTP Function off and on. Toggle it on the camera and take another shot; confirm the photo arrives.
3. Inspect the app log for `camera.event`, `ftp.upload completed`, and `ftp.upload filed` INFO lines without a credential or fingerprint.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
