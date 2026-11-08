---
plan: P55_camera_setup_polish
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-08
actor: opus
---
# P55 review, round 1: APPROVED_WITH_NOTES

Gate: tracker row was `IMPLEMENTED`; the checkpoint `12ffac6` ("CHECKPOINT (not approved): P55 camera setup polish") exists and contains the implementer handoff.
Process authority: `docs/process/flow.md` (no mutation probes; blocking only B1–B4). I edited no production code.

## Verified correct
- **Perimeter.** `git show --stat 12ffac6` lists 16 paths, equal to the handoff's Files changed. The two paths outside the plan's expected list (`crates/testkit/tests/p42_camera_runtime.rs`, `apps/desktop/src-tauri/src/lib.rs`) are declared. They only update earlier assertions for the new restart timing and error wording.
- **Suite (reviewer run, clean tree `c5b9c76`, code identical to `12ffac6`).** `cargo test --workspace` 836 passed / 0 failed. `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `npm run typecheck` and `tools/sidecar/build.sh --stub --test` green. `npm test` 410/411: the one failure is `p33_split_grid` C4(d), the intermittent that plan C5(a) names. It passes 7/7 when run alone, and no P55 path touches the workspace grid. Not B4.
- **C1(a).** `manifest::verify` first matches against the outer archive. It then copies each nested `.zip` once to an anonymous temp file and matches against it, by path suffix as before. Members are read into memory only when they match. Nested names still pass the `..` and absolute-path refusal. `c1a_sony_download_installs_seven_verified_files` installs a zip whose only entry is `RemoteCli.zip` (made-up bytes under `external/crsdk/`) and compares all seven installed files byte for byte. It fails if nested archives are not read.
  - **Variation evidence, at reality:** a read-only probe (a scratch crate in `/tmp/p55probe` with a path dependency, nothing added to the repo) ran `manifest::verify` with the compiled Sony manifest on the owner's real `~/Downloads/CrSDK_v2.02.00_20260610a_Mac.zip`. All 7 files verified in 0.27 s. Every entry in the outer and nested archives is deflate or stored, which matters because the crate is built with deflate only. SimpleCli.zip comes before RemoteCli.zip and carries byte-identical copies.
- **C1(b).** An absent file gives `missing <path>`, a hash mismatch `changed <path>` (`c1b_missing_and_changed_are_distinct`, exact strings). The P51 Settings case asserts both toast sentences exactly: "The archive is missing CrAdapter/libssh2.dylib" and "The archive has changed CrAdapter/libssh2.dylib".
- **C2(a).** The step reads `MENU › Network › [FTP Transfer] › [FTP Transfer Func.] › [Auto FTP Trnsfr] = [On]; …`, after FTP Function. The lab file uses the menu path but spells it `[Auto FTP Transfer]`; the code uses the camera's own label `Auto FTP Trnsfr`, as the plan row asks. The test renders Settings → Camera and asserts the full text. It fails if the step is absent or worded differently.
- **C3(a).** `Take photo` in `CameraCard` (the header dot's popover, also used on Health) is `disabled` unless the status is `connected` (and while an action is running), and it calls `cameraCapture` once. The command-to-helper leg is covered by the existing `c8b_camera_capture_answers_the_capture_id`: `camera_capture` answers the id, the exposure is recorded and the fake helper reports `cmd.done`. Across both tests the row's outcome is observed at its boundaries.
- **C3(b).** A rejected capture produces `toast.error(message)`, and the test asserts the exact sentence.
- **C4(a).** `Supervisor::tick`: the first exit waits 0 s; from the second consecutive exit `RESTART_BACKOFF_S` applies (1, 2, 4 … then `RESTART_STEADY_S`). The 60 s healthy-run reset is unchanged, and an explicit quit sets `stopped`, so it never restarts. The production background loop ticks every 50 ms, so "at once" holds in the app, not only in the test. `c4a` would fail on a 1 s first wait (it never advances the fake clock) and on a 0 s second wait (it asserts `pids == 2` before `tick(1000)`).
- **Assumptions** (one nested level; Auto FTP next to the FTP steps; Take photo first and enabled only when Connected; "unexpected" means any child exit while the supervisor runs). None contradicts the intention. They are in the narrative in plain words.
- **H1–H8.** All changes are in integrations (camera support and supervisor) and the frontend. Core is untouched (H7). The command stays thin (H8). Nothing touches RAWs (H2) and no other app is driven (H5). Test archives use made-up bytes: no Sony file, secret or `.env` is committed.

## Findings
None blocking (B1–B4).

**Lesson for the coordinator (plan/book, not blocking):** 10E MC-31 §Process still reads "A crash is followed by a restart with bounded backoff (1, 2, 4, 8, 16 s, then every 30 s)". P55 Goal 4 and C4(a) make the first restart immediate. That is consistent with 10E's own hardware acceptance row ("restarted sidecar reconnects in ~1 s"), which a 1 s first wait plus startup cannot meet. Align the book's wording at the next owner amendment. The owner can reverse the choice via the narrative.

## Notes written to backlog
1. Every nested `.zip` is copied to a temp file before matching, about 196 MB for Sony's download (sources, API reference). The plan's hazard note asks for less; it costs 0.27 s today.
2. The first suffix match wins, and a mismatch fails at once even when a later copy (for example in RemoteCli.zip) would match. Safe today because the copies are identical.
3. Other install errors produce garbled sentences ("The archive is CrSDK.zip" for a non-zip). `data.file` now carries a reason prefix; a separate `reason` key would be cleaner.
4. The MC-31 book wording differs from the code (the lesson above).
5. The FTP steps, including Auto FTP, appear only after Generate password, which replaces the camera's FTP password.

## Write perimeter (this session)
- `docs/implementation/master_plan.md`: P55 tracker row only (→ APPROVED).
- `docs/implementation/plans/P55_camera_setup_polish.md`: Review log entry appended.
- `docs/implementation/backlog.md`: 5 notes appended.
- `docs/implementation/narratives/P55_camera_setup_polish.md`: new.
- This handoff.
- Outside the repo: `/tmp/p55rev/` (suite logs), `/tmp/p55probe/` (read-only probe crate and its build), `/tmp/p55sony.Um8k/` (nested zips extracted from the Sony download for listing and hashing. Removing it was denied by the sandbox; it is outside the repo and holds Sony files, so delete it by hand: `rm -rf /tmp/p55sony.Um8k`).
- No production file touched. No mutation probes (flow §4). No database or app state changed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
