---
plan: direct-camera
role: plan
round: 1
state: DONE
owner_cards: 0
date: 2026-11-06
actor: Claude Fable 5.1 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# Direct camera adapter (book 10E) — planning handoff, round 1

**Summary.** Gate passed: `docs/intention/intention.md:3` reads **RATIFIED** (owner, 2026-10-24); 10E was ratified at `53d6ea4` (R42–R45, M37–M39,
changelog 31), inventoried at `41bef14` (R-I18…R-I38, M37–M40 decidable, changelog 32; `handoff_direct_camera_inventory_r1.md`), and the one open card CI1
was answered "always jump" during this session (R46 at `e0ce354`, relayed by the orchestrator; chapter 15 has no open card). I read both commits' diffs, 10E
in full, 01/03/05A/05B/10/10A/10C/11/12/13/14/15/16, the inventory handoff, the evidence (`sony_camera_lab.md`), flow.md, the master plan, the P39 plan as
the house template, the code at `41bef14` (every path and line cited in the plans was verified there), and the lab repo at `221aa49` (`sdk/` — CMake, the
15 sources, 5 headers, 17 mock-camera `TEST_CASE`s, the stub; `camlab/ftp/server.py` + `config.py` and the 18 receiver tests; `camlab/ledger.py` and
its 15 tests; `docs/session-manager-handoff.md` §4 protocol; `.gitignore` and `vendor/`). **Eleven plan files written** (P42–P52: 93 criteria / 263
rows in total, of which 27 `[rule]` and 28 `[manual]` — counts derived from the files), `phases.txt` appended (52 phases), the master plan amended
(§2, §3, §4, §5, §6.1–§6.9, **new §6.11** camera skeleton, §7.1, §7.2, §7.5, §9). Not committed (as instructed). **No owner card.**

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. CI1 was answered mid-session and is planned as the fixed rule (P49). Every other open point is either answered in the book, a
planner reading recorded in a plan's Notes (surfaced to the owner through the narratives' "Choices made for you"; the two that matter most are listed
under §Choices below), or a delegated choice. The two **owner-attended** phases (P47 spike, P52 acceptance) need the owner's time and the camera, not a
decision: the orchestrator schedules them.

## Write perimeter
- `docs/implementation/plans/P42_camera_runtime.md` · `P43_ftp_receiver.md` · `P44_capture_ledger.md` · `P45_sidecar_and_protocol.md` ·
  `P46_direct_camera_adapter.md` · `P47_support_package_spike.md` · `P48_camera_ownership.md` · `P49_capture_one_alongside.md` ·
  `P50_camera_settings_and_support.md` · `P51_camera_setup_ui.md` · `P52_hardware_acceptance.md` (all new).
- `docs/implementation/phases.txt` (eleven lines appended with `printf >>`, run order P42 → P52).
- `docs/implementation/master_plan.md` (§2 sources: 10E at `53d6ea4`/`41bef14`/`e0ce354`, M1–M40, the Sony evidence and the read-only lab repo; §3 trace
  ranges MC-1…MC-34 / M1–M40, `[manual]` rows, the camera hard rules; §4 eleven NOT_STARTED rows; §5 count 52, graph tail, `depends_on`, the direct-camera
  phasing note, P45/P47/P52 preconditions; §6.1 new modules incl. `native/camera-sidecar/`; §6.2 camera crates; §6.3 support folder, fingerprints,
  `.incoming`/`.duplicates`; §6.4 port changes; §6.5 migrations 006/007; §6.6 camera enums, error and health codes; §6.7 camera commands and events; §6.8
  the `camera` settings block and Keychain accounts; §6.9 fake sidecar / FTP client / C++ tests; **new §6.11**; §7.1 cmake/clang/SDK/camera facts; §7.2
  sidecar build commands; §7.5 harness; §9 log entry).
- `docs/implementation/handoffs/reviewer/handoff_direct_camera_plan_r1.md` (this file).
- **Not touched:** `docs/intention/**`, `STATUS.md`, `backlog.md`, narratives, other plans, any code, any tool-recorded state. No commit, nothing pushed, no
  pipeline run, nothing against the camera, Capture One or Shopify; no secret read or printed (the lab's `.env` was not opened; `.env.example` was read
  with its password line filtered out).

## The phase map
| Phase | Criteria / rows | Camera | Projection | Difficulty | One line |
|---|---|---|---|---|---|
| P42_camera_runtime | 8 / 26 | no | **yes** | high | the process-wide `CameraRuntime` (R-I18) carried across settings rebuilds; sidecar supervisor over a fake sidecar binary (first-line credentials, backoff 1-2-4-8-16-30, quit→3 s→kill); `SecretStore` + Keychain impl + fake; new `CameraStatus` and its precedence (R-I37, 11 adjacent pairs); `CameraService` gates only for the Capture One adapter (R-I33); new `camera_status` DTO |
| P43_ftp_receiver | 8 / 27 | no | no | high | hand-written passive-only FTP receiver over tokio (MC-32, R-I24): staging `.part`, reset = failure (RST via `SO_LINGER 0` in the testkit client), `REST`/`APPE` resume, virtual namespace, listings from completed names, refusals with the fixed codes, private peers, 2 concurrent uploads; the `camera` settings block applied in place (one listener) |
| P44_capture_ledger | 8 / 33 | no | **yes** | high | the lab ledger as a core mechanism with the inventory's fixes (file key, association order with the 300 s window and the candidate-set session rule, global duplicates, `renamex_np(RENAME_EXCL)`, target folder at rename time, freeze), migration 006, the one worker with crash-ordered rows, MC-2 rule 0 / `captured_at`, watchdog counts; the receiver's sink becomes the ledger (one upload → one photo) |
| P45_sidecar_and_protocol | 7 / 18 | no | **yes** | high | the lab `sdk/` imported under `native/camera-sidecar/` without `sdk_shim` or any Sony file; CMake stub build from `build.rs`, `ctest` (17 + 2 new); the R-I19/R-I20/R-I21/R-I23 sidecar changes; `SidecarEvent`/`Command`/`SidecarClient` proven against the fake replaying the 13 real lab lines + emit-site-cited lines; `externalBin`, `NSLocalNetworkUsageDescription` |
| P46_direct_camera_adapter | 8 / 25 | no | **yes** | high | the direct `CameraProvider`: status from events, capture ids, exposures (app/camera), Save-to-PC → cable deliveries, store-destination outcome, busy, Sony `_N` normalisation, `ftp-status` on the watchdog's first raise, reconnect on Wi-Fi only, first copy wins end to end through the real receiver and ledger, adapter selection by installed support |
| **P47_support_package_spike** | 7 / 10 `[manual]` | **yes** | no | normal | owner-attended: S1–S6 (runtime loaded from Application Support on a second account, library validation, Gatekeeper, Local Network, ImageCaptureCore readers, update prompts) + S7 (the camera's FTP command sequence; `cmd.received credentials` has no fields); answers folded into P50 |
| P48_camera_ownership | 8 / 38 | no | **yes** | high | holder naming (fixed list, Capture One first), start order (R-I31, five conditions, 15 s), the editor-launch gate over every launching script (M40), "Take the camera back" step by step with bounds and aborts (never killed), activation stores the session first (R-I33), idle-sleep assertion, `camera_test_connection` |
| P49_capture_one_alongside | 5 / 19 | no | no | high | **R46**: current collection on every arrival while Capture One shows another collection and on every activation; variant wait 1 s × 15 then backoff, `variant_not_found` never terminal (task 0 reproduces V4); style once per RAW (migration 007); Import fixed by copying into the session's capture folder (`import.js` retired) |
| P50_camera_settings_and_support | 8 / 23 | no | no | normal | support-package install from the Sony archive (manifest of hashes, no quarantine, ad-hoc signing + S2's entitlement if needed, sidecar copy refreshed on update, version check), damaged folder → health; Keychain secrets with R-I18's restart rule; generated FTP password shown once, receiver reopened; fingerprint pending/confirm/reject (+ `accept-fingerprint` sidecar command) |
| P51_camera_setup_ui | 8 / 26 `[rule]` | no | no | normal | the dot's five states and sentences, Take the camera back / Reconnect / Dismiss on the card, Health names and actions, Settings → Camera (add Sony support, FTP password shown once with the camera steps, ports, Access Authentication, fingerprint card, Test camera connection, the store-destination agreement line) |
| **P52_hardware_acceptance** | 18 / 18 `[manual]` | **yes** | no | normal | owner-attended: 10E's acceptance table on the packaged build with support installed through the app; results in the evidence; findings routed by the orchestrator |

Order in `phases.txt`: P42 → P43 → P44 → P45 → P46 → **P47** → P48 → P49 → P50 → P51 → **P52**. `depends_on`: P42[P41] · P43[P42] · P44[P43] · P45[P44] ·
P46[P45] · P47[P46] · P48[P46] · P49[P48] · P50[P47,P49] · P51[P50] · P52[P51]. P48 and P49 do not depend on the spike, so the orchestrator may run them
before P47 by reordering `phases.txt` when the owner cannot spare the camera that day; P50 waits for both.

## Deviations from the orchestrator's phasing (latitude: "your call, justified in the handoff")
1. **Eleven phases, not ten.** The orchestrator's step 8 (settings and setup UI) became P50 (backend: support package, secrets, fingerprint, health,
   commands — 23 rows) and P51 (frontend — 26 rows): together they would have exceeded 8 criteria and mixed Rust and React in one phase.
2. **The spike moved before the support-package phase** (P47 right after P46), as the guidance asks ("before any phase that depends on its answer"):
   P50's signing/entitlement and quarantine rules take S2/S3's answers, and S7 (the camera's FTP command sequence) de-risks P43's hand-written receiver
   before P52. The spike needs only P46's runtime and a hand-installed support folder; its plan says how (one `security add-generic-password` per Keychain
   item with the interactive prompt, no secret on a command line).
3. **Protocol client folded into the sidecar import (P45)** rather than a separate phase: the client's fixtures are the lab's event lines, which the
   import brings in; P46 then owns the semantics.
4. **CI1 planned as the fixed rule** after the owner's mid-session answer (R46): no policy constant, no either-way allowance (the earlier draft of P49 had
   one; it was removed before this handoff).

## Grounding worth knowing (orchestrator)
- **No FTP server crate.** P43's receiver is hand-written over the pinned tokio because R-I24 fixes exact reply codes and the reset/resume semantics; the
  camera's real command subset is recorded at P47 S7 before P52. Recorded as a planner reading in P43 Notes.
- **Tauri `externalBin` and `cargo test`.** `tauri-build` expects the external binary to exist at compile time, so P45's `build.rs` builds the **stub**
  sidecar with cmake when the file is absent (cmake 4.4.5 and clang 17 are on the dev Mac; §7.1). The SDK build runs only through `tools/sidecar/build.sh
  --sdk <lab>/vendor/sony-sdk` and is never committed; the Sony runtime is never bundled (the user installs it, P50).
- **Where the Sony SDK is on this Mac:** `/Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk` (gitignored in the lab), not
  `~/Library/Developer/SonyCrSDK`. P45's implementer should compile the R-I19/R-I20 C++ edits against it once (nothing committed); the suite never needs it.
- **Preconditions before dispatch** (master plan §5): P45 — the implementer must be able to read the lab repo (or the orchestrator copies `sdk/` minus
  `sdk_shim/` into `native/camera-sidecar/` first); P50 — P47's S2/S3 answers written into P50's task 0 note and the manifest generated with
  `tools/sidecar/manifest.sh` (hashes only) or shipped as the empty shape.
- **Crate versions verified on crates.io 2026-11-06:** `libc 0.2.191` (has `renamex_np`, `RENAME_EXCL`; **not** the 1.0.0-alpha line), `security-framework
  3.7.0`, `ipnet 2.12.2`, `zip 9.4.1`, `getrandom 0.4.3`.
- **Externally derived fixtures** (rule 17) are concentrated in P45 C3(b) (sidecar emit sites, each cited `file:line` in the lab) and P46 (event scripts);
  both carry `projection: yes`. The 13 real lines of handoff §4.2 are copied verbatim as the primary fixture; `1_793_888_519_760` is the epoch-ms of its
  second line (`2026-11-05T14:21:59.760Z`) for the timestamp row.
- **Discriminating rows to keep an eye on:** P42 C1(a) (one pid across a rebuild), P43 C2(a) (RST → 426, no delivery), P44 C3(f) (an existing file is never
  replaced), P44 C4(f) (rule 0 vs rule 3 with a deleted session), P46 C5(a)/(b) (first copy wins in both orders through the real receiver), P48 C3(g)
  (a health check during take-back never launches Capture One), P48 C5(c) (quit refused → never killed), P49 C2(d) (Capture One not running → no script).
- **Tests that change their expectation** are named in each plan's Read-first (P42: p04 `:404`, p25 `:127`, seven frontend mocks gain `link: 'cable'`;
  P43/P44: the P43 fixture rows read the ledger from P44 on; P49: p05 `c8a_import_active_and_provider_called`).
- **Owner-attended phases and the runner:** P47 and P52 carry `[manual]` rows and a task 0 that makes any dispatched implementer report `BLOCKED`; the
  orchestrator runs them with the owner and writes the evidence sections.

## Choices the owner may want to know (recorded in the plans' Notes; for the narratives)
1. **Import now copies the file into the session's capture folder** (P49): Capture One's own `import` is broken in 16.8.6 and, when it works, copies the
   file somewhere the app does not track; the copy is what the app tracks, the original is untouched.
2. The FTP user name on the camera becomes `sessionmanager` with a password the app generates and shows once (P43/P50); the owner re-enters both on the
   camera once.
3. Nothing installed is not an error: without a Sony support package the app keeps working through Capture One's tether; a damaged support folder is a
   Health item (P50).
4. The helper-health item appears after three consecutive sidecar crashes, not the first; the backoff resets after 60 s alive (P42).
5. "Take the camera back" waits for the interface by retrying `usb-retry` once a second for up to 5 s after Capture One quit, and relaunches Capture One
   even if the camera could not be taken (P48).

## Lessons for the planner
- When an owner card is answered mid-planning, replace the policy seam entirely rather than leaving a constant "in case": a constant that can only have one
  value is review surface with no row that can fail.
- A lab's test suite is a semantics source, not a row list: four of the lab's 33 tests had no equivalent in the 10E model (failed ack, JSON persistence,
  format check, FTPS) and were mapped to "not a case" explicitly so the reviewer does not look for them.
