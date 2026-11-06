---
plan: P45_sidecar_and_protocol
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-06
actor: opus (pipeline-reviewer)
---
# Review — P45 camera sidecar and protocol, round 1

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. The production code satisfies all 21 rows, every row's test observes its
outcome at the row's boundary and would fail on a wrong outcome, the suite is green (one unrelated frontend flake, re-run
green), and no assumption contradicts the book. 7 notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
Tracker row `IMPLEMENTED`; checkpoint `db3cdae` (`CHECKPOINT (not approved): P45_sidecar_and_protocol implement r1`) in `git log`.
Reviewed tree `c4bdab2`, which equals `db3cdae` plus runner files (`STATUS.md`, the implementer handoff, the review prompt). No code drift.

## Verified correct
- **Perimeter.** `git show --stat db3cdae` = 44 paths = the plan's 42 (24 native, 2 tools, 4 repo, 4+3 integrations, 2+3 testkit) +
  tracker + plan Review log, which equals the handoff's declaration. No extra or missing path.
- **No vendor file (hard rule).** `git ls-files native/` = exactly the 24 expected sources. `git ls-files` shows no
  `CrSDK*`/`CameraRemote*`/`sdk_shim`/`vendor/`/`*.dylib`/`*.fingerprint`/`.env*`. `diff -r` against the lab's `sdk/` at `221aa49`: only
  `tests/sdk_shim/` absent and `README.md` added. Every other difference is one of the C++ edits below.
- **C++ edits 1–7 (read against the lab and 10E R-I19–R-I23):**
  1. Credentials: `std::getline(std::cin)` before `SdkCameraAdapter adapter(log, base, …)`. EOF → `return 0`. `credentials ` →
     `percent_decode` of `user`/`password` into `base`, then `cmd.received {cmd}` only. Any other first line is pushed as the first
     queued command. `CAMERA_SSH_*` reads and usage text are gone. `percent_decode` (`json_log.cpp`, in `crlab_core`) guards with
     `i + 2 < size`, so it never reads past the end.
  2. `usb.initial {present: present_count() > 0}` is emitted right after a successful `mon.start`.
  3. `usb_retry()` is a queued `Input::UsbRetry` that resets `usb_failures_`, `usb_retry_at_` and `failures_`, sets
     `next_attempt_ms_ = now` and emits `usb.retry_requested`. It is accepted before the `!connected` rejection.
  4. One `--usb-product-id` comma list, default `0x0E0C`, passed to the `UsbMonitor` constructor. The filter in `on_added`/`on_removed`
     runs before `present` changes. `usb_exclusive_owners`/`report_usb_claim` use the same filter.
  5. `after_connect` runs on both transports after `r.ok`: `configure_save(save_dir, "")`, then
     `set_store_destination(usb ? "pc+camera" : "camera", 3000)`, which returns `{verified, read, value}`. `try_connect` appends `model`,
     plus `store_destination`/`store_value` when non-empty. `restore_store_destination`, its call in `disconnect()` and the `"pc"` mapping
     are removed.
  6. `before_voluntary_stop(Transport)` is a default no-op on `CameraAdapter`. It is called only when `active_ == Usb`, after the
     deferred-stop wait and before `intentional_disconnect`. The SDK adapter does one `"camera"` write with a 1000 ms limit.
  7. Exposure and download handlers are set on `CameraSession`. A USB exposure → `FileTransfer` + a 10 s deadline. `None` after
     `run_capture` only when no USB exposure was seen. The 100 ms loop clears the deadline. A stop is deferred while busy, for at most
     `stop_wait_ms`, measured from the draining step with `>=`. It returns early (no connect or switch), and a cable loss ends the wait
     as `session_lost`.
- **Packaging.** `build.rs` builds the stub only when `binaries/camera-sidecar-<TARGET>` is missing, and checks for cmake first.
  `externalBin` is set, and so is `NSLocalNetworkUsageDescription`. `.gitignore` has all six entries. CMake: stub by default, the SDK
  build needs `CAMERA_SIDECAR_WITH_SDK=ON` with `CRSDK_ROOT` set, dylib copies go to the build dir only, ctest registers one test.
- **Rust.** `parse_line` follows the variant table: required fields are those written unconditionally at the emit site, unknown fields
  are ignored, and an unknown event becomes `Unknown`. `iso_to_ms` accepts the specified forms and validates days. `Command::encode`
  uses the RFC 3986 unreserved set. Client correlation works as in §The client: a rejection or error attaches to the most recent open
  `cmd.received`, `Done` never overwrites, and empty ids are never tracked. `send` returns the wire id. Staging is
  `<document>/Capture/.incoming/sdk-<run_id>` (`capture_root` holds the document dir, as `capture_root_fn`'s `join("Capture")` shows).
  `--usb-product-id 0x0E0C` is in argv. Unparsable lines go to the helper log through the redactor.
- **Fixtures.** `lab_auto_switching.jsonl` (and the testkit copy `lab_lines.txt`) matches lab handoff §4.2's 13 lines byte for byte.
  `emit_sites.jsonl` carries the C3(b) shapes (`connect.failed` uses `error`, both paths).
- **Rows and tests** (each read against its production code):
  - C1(a–d): the tests check the working tree with the row's exact patterns and the required files present.
  - C2(a): exit 0, the binary is executable, and the output contains `20 tests, 0 failed` and `100% tests passed out of 1`.
  - C2(b): `version` must print exactly one line with `crlab.version`/`sdk_compiled_in:false`. `auto` must print `sdk.unavailable`,
    exit 4, within 2 s.
  - C2(c): exact ordered path list, SHA-256 computed in the test, `README.txt` absent, tree walks before and after show `native/`
    unchanged.
  - C3(a–c) and C4(a–c): whole-variant equality and exact strings.
  - C5(a): the 13-event vector with nothing before it, and the first stdin line is the credentials line.
  - C5(b): the exact wire line appears once, the outcome is `Done`, and `CaptureExposed` carries the uuid.
  - C5(c): `Rejected` survives the later `cmd.done`.
  - C5(d): exactly one event and exactly one `unparsable` helper-log entry.
  - C6(a–c): the C++ `TEST_CASE`s match the rows' rigs and outcomes, including the no-retry control rig and both deferred-stop rigs.
    The Rust wrappers require the `[ OK ] <name>` line, which `mt_run_all` prints only when no CHECK threw.
  - C7(a): exact `externalBin` and a non-empty plist string.
- **Suite** (reviewer run, tree `c4bdab2`, clean status):
  - `cargo test --workspace`: exit 0, 693 passed, 0 failed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
  - `cargo fmt --all -- --check`: exit 0.
  - `npm run typecheck`: exit 0.
  - `npm test`: 368/369 on the first run. `p33_split_grid.test.tsx` C4(d) failed. No frontend file is in this perimeter. That file
    then passed 3/3 alone, and a full re-run passed 369/369. This is a flake, not B4 (backlog note).
- **H1–H8.** No external platform write. The camera's storage setting is the R-I19 write the photographer agrees to. No RAW is touched,
  and staging is only created. No pixel work. No UI automation, since the SDK is the camera's published interface. Core is untouched
  (H7). No Tauri dependency in integrations (H8).
- **Secrets.** No credential in argv or env, the first stdin line is built from the cached credentials, and no secret is in the repo.
- **No live services.** Tests use the fake sidecar and a local cmake stub build.
- **Assumptions 1–13** checked against 10E MC-31: none contradicts the book. Assumptions 5–9 and 11 are narrated to the owner.
  Assumptions 1–4 are signature choices, recorded as registry drift.
- **Mutation probes:** none, per `flow.md` §4. No file was edited for probing.

## Findings
None blocking.

## Notes written to backlog
1. P45 → P46: busy *Transfer* is cleared by any `sdk.download.completed`, not the matching one (R-I20 "matching completion"). This
   matters with two quick USB shots.
2. P45 → P46: an unknown value of an optional enum field (`store_destination`) or of `transport` drops the whole line as unparsable.
3. P45 dev loop (observed): `c2a` re-copies the binary on every run. tauri-build's `rerun-if-changed` on it then rebuilds
   `psm-desktop` on the next cargo build.
4. P45 → P47/P50: `build.rs` uses the `TARGET` suffix, while `build.sh` always writes the `aarch64-apple-darwin` name.
5. P45 → master plan §6.11: registry drift. `Stamped`, `poll(now, side)`, no `events` sender, `StoreDestinationResult`, 60 symbol
   probes.
6. P45 → P46: `SidecarClient.outcomes` is never pruned.
7. P33 suite flake: `p33_split_grid` C4(d), observed once.

## Lessons for the plans
- C2(a)'s `OnceLock` build writes into the repo's `binaries/` during the suite. That matches the plan, but its side effect on cargo's
  rebuild tracking (note 3) was not foreseen. A plan whose test writes an `externalBin` should say "copy only when changed".
- The plan's variant table had no column for unknown enum values. Note 2 follows from "required = written unconditionally", applied to
  values as well as presence. Future protocol tables should state the unknown-value rule per enum.

## Write perimeter (this session)
- `docs/implementation/backlog.md`: 7 rows appended after the last table row.
- `docs/implementation/master_plan.md`: the P45 tracker row only.
- `docs/implementation/plans/P45_sidecar_and_protocol.md`: Review log entry.
- `docs/implementation/narratives/P45_sidecar_and_protocol.md`: new.
- This handoff: new.
- Side effects of running the suite: `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` and
  `target/camera-sidecar/stub` were rebuilt by `c2a` (gitignored, stub as before). Logs are in `/tmp/p45rev/` (outside the repo).
- No production or test code edited, no commit, no archgraph or tool state.
