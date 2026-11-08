---
phase: P47_support_package_spike
title: Camera support package spike (owner-attended, camera needed) — the packaged, ad-hoc-signed app loads the Sony SDK runtime from a hand-installed Application Support folder, connects on USB and Wi-Fi, passes Local Network and Gatekeeper, and the camera's FTP command sequence is recorded; answers S1–S7 for P50
difficulty: normal
projection: no
depends_on: [P46_direct_camera_adapter]
---
# P47 — Camera support package spike (10E §Camera support packages, S1–S6; owner-attended)

## Goal
Answer, on real hardware and with the real Sony runtime, the questions 10E §Camera support packages (inventory additions R-I36) lists as unverified
**before** the support-package phase (P50) is built: S1 the copied sidecar loads the runtime from `~/Library/Application Support/com.loorenz.sessionmanager/
camera/sony/<version>/` and connects on USB and Wi-Fi from a packaged, ad-hoc-signed app, on a second user account; S2 whether hardened-runtime library
validation refuses the separately signed Sony dylibs (fallback: `com.apple.security.cs.disable-library-validation` on the sidecar); S3 no Gatekeeper
prompt or removal; S4 macOS Local Network privacy for the sidecar's Wi-Fi connect and the receiver's inbound connections (`NSLocalNetworkUsageDescription`
is in place since P45); S5 whether the processes with ImageCaptureCore loaded can be read (best effort, for P48's holder naming); S6 the firewall and
Keychain prompts after an update of an ad-hoc-signed build. Plus **S7 (planner)**: the exact FTP command sequence the camera sends to the Rust receiver
for one upload (so P43's command subset is confirmed before P52), and the sidecar's `cmd.received` for `credentials` carries no fields (P45 change 1).
**Not in this phase:** any code change (findings are recorded; code changes they imply go to P50 or a backlog note), the acceptance table (P52),
"Take the camera back" (P48), running the suite.

## Read first
Book: `10E_direct_camera_adapter.md` §Camera support packages (whole, incl. R-I36 and S1–S6), MC-31 §Secrets (R-I21: Keychain items, fingerprints
folder), §Owed before planning · `11_architecture_and_packaging.md` · evidence `sony_camera_lab.md` C8 (Gatekeeper deleted a dylib once), §V6 (store
destination over USB verified) · master plan §3 (`[manual]` rows; camera hard rules), §6.3 (support folder layout), §6.11 (Support packages; Sidecar
`SidecarSpec` env `CRLAB_FINGERPRINT_DIR`), §7.1 (SDK location on this Mac), §7.2 (`tools/sidecar/build.sh --sdk`).
Lab (read-only): `docs/session-manager-handoff.md` §5 (runtime layout: `libCr_Core`, `libmonitor_protocol*` next to the executable, `CrAdapter/` in
`<exe dir>/CrAdapter/` and `<exe dir>/Contents/Frameworks/CrAdapter/`; signing), `docs/setup.md` (camera menus), `camlab/experiments/camera_steps.py`
(the camera's FTP and remote-shooting settings; **nothing on the camera is changed except the FTP user and password when the owner chooses to, P50**).
Code: P45's `tools/sidecar/build.sh`, P46's runtime (adapter selection by `camera/sony/<version>/camera-sidecar`), P43's receiver (the FTP password from
the Keychain item `ftp-password`), P42's `KeychainSecrets`.

## Files expected to change
`docs/intention/evidence/sony_camera_lab.md` (new section **§Spike S1–S7 (P47, <date>)** with one result row per question), this plan's Review log,
`docs/implementation/backlog.md` (notes for findings that imply code changes outside P50). **No code.** Nothing of the Sony runtime, the fingerprints,
the Keychain or `.env`-like material is ever written into the repo.

## Core mechanisms
None (no code).

## Ordered tasks (the orchestrator with the owner; the camera and the second user account are needed from step 3)
0. **If launched by `run_phase.sh`:** report `STATE: BLOCKED`, reason `owner-attended phase (P47): run by the orchestrator with the owner`, change nothing.
1. **Build.** On the dev Mac: `tools/sidecar/build.sh --sdk /Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk` (places the SDK
   build of `camera-sidecar` at the externalBin path, uncommitted), then `cd apps/desktop && npm run tauri -- build --bundles app` → `Session Manager.app`
   (ad-hoc signed, `signingIdentity: "-"`). Record the sidecar's `version` output (`sdk_version`).
2. **Hand-install the support folder** (what P50 will automate): under `~/Library/Application Support/com.loorenz.sessionmanager/camera/sony/<sdk_version>/`
   copy from the lab's `vendor/sony-sdk` the runtime `libCr_Core.dylib`, `libmonitor_protocol.dylib`, `libmonitor_protocol_pf.dylib` and the `CrAdapter/`
   folder (also as `Contents/Frameworks/CrAdapter/`, lab §5.3), and a copy of the built `camera-sidecar`; `codesign -s - --force` every Mach-O; record
   `xattr -l` of the copies (expect no `com.apple.quarantine`). Add the Keychain items with `security add-generic-password -s com.loorenz.sessionmanager.
   camera -a access-auth-user -w` (interactive prompt — the value is never on the command line), likewise `access-auth-password` and `ftp-password`
   (the lab's camera FTP password until P50 generates one; the camera's FTP user stays the lab's `sony` for this spike — P43's `FTP_USERNAME` is
   `sessionmanager`, so for S7 the owner temporarily sets the camera's FTP user to `sessionmanager`, or the orchestrator runs the receiver standalone with
   a `--user sony` override from a scratch binary; **record which**). Nothing of this is committed.
3. **S1 — load and connect (dev account).** Launch the `.app` (double-click). Expect: the header dot goes through Connecting to Connected (cable) within
   ~8 s with the cable in; pull the cable → Wi-Fi within ~6 s (first time: the fingerprint card — confirm it by comparing with the camera menu; until P50 the
   confirmation is `touch`ing the pinned file the sidecar wrote? No: the sidecar refuses unpinned fingerprints without `--accept-fingerprint`; for the spike
   the orchestrator pins it once by running the sidecar from Terminal with `--accept-fingerprint` into `camera/fingerprints/`, as the lab did; **recorded**).
   Record the helper log lines (`connection.ready` with `store_destination`), the time to connect on each link, and `cmd.received {cmd:"credentials"}`
   having no other field (S7b).
4. **S1 on a second user account.** Log into another macOS user, copy the `.app` and repeat step 2's install (the support folder is per user) and step 3.
   Record Gatekeeper's behaviour at first launch (right-click Open) and whether any dylib was removed (S3).
5. **S2 — library validation.** If the sidecar fails to load a dylib (`dyld` error in the helper log / Console: "code signature in … not valid for use in
   process"), sign the sidecar copy with an entitlements plist carrying `com.apple.security.cs.disable-library-validation` and retry; record which signing
   made it load. If it loaded without it, record that too.
6. **S4 — Local Network.** From the packaged app: does macOS show the Local Network prompt (text from `NSLocalNetworkUsageDescription`)? Does the Wi-Fi
   connect work before/after accepting? Does the camera's FTP upload reach the receiver (System Settings → Network → Firewall state recorded)? Record the
   firewall prompt for the app, if any (S6a).
7. **S5 — ImageCaptureCore readers.** With Capture One running, run `lsof +c 0 | grep -i ImageCaptureCore | awk '{print $1}' | sort -u` (read-only) and
   record whether the process list is readable without privileges and how long it takes; also `pgrep -x ptpcamerad`. Nothing is killed.
8. **S6 — update prompts.** Rebuild the app (a new ad-hoc code identity), relaunch: record whether the Keychain asks again for the three items and whether
   the firewall asks again.
9. **S7 — FTP command sequence.** With the receiver's log at debug (`PSM_LOG=debug` env for the app, or the standalone receiver), let the camera Auto-FTP one
   shot: record the full command sequence (`USER`, `PASS`, `TYPE`, `PASV`/`EPSV`, `CWD`/`MKD`, `STOR`, `LIST`/`SIZE`…) and every reply code the receiver gave,
   and whether the file arrived (one photo). A command answered `502` is a finding for P50/P52.
10. Write the evidence section and the Review log entry; fold answers into P50's Notes (the orchestrator edits P50's plan before it is dispatched: entitlement
    yes/no, quarantine behaviour, the FTP command subset).

## Criteria (7 criteria, 10 rows; all `[manual]`)
Every row is recorded by the orchestrator in `evidence/sony_camera_lab.md §Spike` with the observed value; "pass" is what P50 then assumes.

| Row | Boundary | Input | Pass | Trace |
|---|---|---|---|---|
| S1(a) | packaged app, dev account, cable | step 3 | `Connected · Cable` ≤ ~8 s after launch with the cable in; helper log `connection.ready {transport:"usb", store_destination:"verified"}` | `[manual]` R-I36 S1; MC-31 R-I19; M39 |
| S1(b) | packaged app, Wi-Fi | step 3, cable out | `Connected · Wi-Fi` ≤ ~6 s after the cable event; no SSH auth retry in the log | `[manual]` R-I36 S1; MC-31 §Fingerprint |
| S1(c) | second user account | step 4 | both links connect from the copy installed under that user's Application Support | `[manual]` R-I36 S1 ("on a second user account") |
| S2(a) | library validation | step 5 | recorded: loads as signed ad hoc **or** needs `disable-library-validation` (the entitlement P50 then applies) | `[manual]` R-I36 S2 |
| S3(a) | Gatekeeper | steps 3–4 | no dylib removed, no "Move to Trash"; the first-launch right-click Open recorded | `[manual]` R-I36 S3; C8 |
| S4(a) | Local Network privacy | step 6 | the prompt text is the P45 string; Wi-Fi connect and FTP uploads work after accepting; firewall state recorded | `[manual]` R-I36 S4; ch11 |
| S5(a) | ImageCaptureCore readers | step 7 | recorded: readable (list + duration) or not; Capture One appears when running | `[manual]` R-I36 S5; MC-34 R-I30 (best effort) |
| S6(a) | prompts after an update | step 8 | recorded: Keychain re-prompt yes/no per item; firewall re-prompt yes/no | `[manual]` R-I36 S6; R-I21 |
| S7(a) | the camera's FTP commands | step 9 | the full sequence and reply codes recorded; the file arrived as one photo; no `502` — or each `502` listed as a finding | `[manual]` MC-32 R-I24; M37 |
| S7(b) | credentials line | step 3 log | `cmd.received {cmd:"credentials"}` has no other field; no secret in the helper log | `[manual]` R-I21; M39 |

## Delegated choices
How the orchestrator captures the helper log (the Health page's helper log if P51 is not there yet: `PSM_LOG=debug` and the daily log file under the data dir);
the order of S1 and S7; whether the standalone receiver is used for S7.

## Manual try-out
This phase **is** a try-out; the narrative is the evidence section in plain words: what connected, how fast, what macOS asked, what the camera sent.

## Notes
- **Hard rules (owner):** nothing of the Sony SDK, the fingerprints or the Keychain goes into the repo (the evidence records facts, not files or hashes of
  secrets); no Python; the camera's network/FTP settings are not changed by the app — if the owner changes the FTP user for S7, that is the owner's hand on the
  camera, recorded; no camera file deleted; no process killed (S5 only reads); live view out of scope.
- **Owner-attended.** `run_phase.sh` must not dispatch this phase (task 0); the orchestrator runs it with the owner at a time of their choosing. P48 and P49 do
  not depend on it; P50 does (its install/sign rules take S2/S3's answer, its FTP subset S7's).
- **Fingerprint pinning for the spike** is done the lab's way (one sidecar run from Terminal with `--accept-fingerprint` after the owner compared the
  fingerprint with the camera menu); P50 replaces this with the in-app confirmation.
- Registry: none. Intention: the evidence file gains a section; no chapter text changes (facts only). If S2 needs the entitlement, P50's task list says so.

## Review log

- **2026-11-07 evening, orchestrator prep (steps 1–2, no camera, owner away).**
  - **Step 1.** `build.sh --sdk` → `sdk_version 2.02.00`. `npm run tauri -- build --bundles app` → `apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app` (31 MiB). The app and its bundled sidecar are signed ad hoc **with hardened runtime** (`flags=0x10002(adhoc,runtime)`). Afterwards the stub sidecar was restored at the externalBin path (`sdk_compiled_in:false`), and `git status` is clean.
  - **Step 2.** Under `~/Library/Application Support/com.loorenz.sessionmanager/camera/sony/2.02.00/`:
    - the three runtime dylibs, `CrAdapter/` and `Contents/Frameworks/CrAdapter/` copied from the lab SDK;
    - every dylib re-signed ad hoc with `codesign -s - --force`;
    - xattrs: only `com.apple.provenance`, **no `com.apple.quarantine`** on any copy (S3 input);
    - `spctl -a` rejects the sidecar and the dylibs (expected for ad hoc; nothing was removed).
  - **S2, answered offline (the camera is not needed to load the libraries).** The sidecar copy with hardened runtime and an ad hoc signature **does not start**. dyld refuses `libCr_Core.dylib` with "code signature … not valid for use in process: mapping process and mapped file (non-platform) have different Team IDs" (exit 134). Two variants load and print `version`:
    - (A) hardened runtime plus `com.apple.security.cs.disable-library-validation`;
    - (B) ad hoc without hardened runtime.

    Variant A is what P50 should apply: the entitlement on the sidecar copy, and hardened runtime kept. The copy in the support folder is signed as A. The CrAdapter dylibs are loaded only at connect, so their loading is confirmed at step 3.
  - **Finding for P50.** The SDK build embeds a second `LC_RPATH` holding the build machine's absolute SDK path (`~/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk/…/crsdk`) next to `@executable_path`. It is harmless at runtime, because `@executable_path` comes first, but it leaks a home path into a shipped binary. P50 should drop it (CMake `BUILD_RPATH`/`INSTALL_RPATH`, or `install_name_tool -delete_rpath`).
  - **Parked.** While a runnable `camera-sidecar` sits in that folder, the owner's everyday app picks the direct adapter at start (`support::installed`). The copy is therefore renamed **`camera-sidecar.parked`**, and the app still uses the Capture One adapter. **At the spike:** `mv camera-sidecar.parked camera-sidecar` in that folder, then add the three Keychain items (interactive, owner), then step 3. To remove everything, delete `…/camera/sony/2.02.00/`.
- **2026-11-08, spike run with the owner (06:00–06:45 local).** Results in evidence `sony_camera_lab.md` §Spike S1–S7.
  - **Passed:** S1(a) cable (third launch), S1(b) Wi-Fi (after the owner fixed the network and Remote Shooting and confirmed a new fingerprint), S2 (entitlement needed), S4, S5 (Capture One not visible to the scan), S6 (3–4 Keychain re-prompts after an update), S7 partial (PASV/STOR, no 502; Wi-Fi shot to app about 10 s).
  - **Not run:** S1(c) and S3 on a second account → P52 on the work Mac.
  - **Main finding:** the Keychain reads block the camera start (→ P50 Notes (d)).
  - **Owner's hand on the camera:** Remote Shooting on; FTP Server 2 = `sessionmanager` at the Mac's IP, port 2121, selected. Lab Server 1 is untouched.
  - **Cleanup:** after the spike the support-folder sidecar is parked again (`camera-sidecar.parked`). The `.app` bundle's version was bumped to `0.1.0.1` and re-signed for S6; it is a build artefact and is rebuilt by P50.
  - **State: DONE (manual).**
