# 10E — Direct camera adapter: camera sidecar, FTP receiver, capture ledger

**Status: DRAFT, proposed 2026-11-06, awaiting the owner.** Not ratified; nothing may be planned from it until the owner
accepts it and the decisions marked **[D1]…[D8]** are answered (they are listed at the end with recommendations). Evidence:
`evidence/sony_camera_lab.md` (lab finding ids H-n, F-n, constraints C1–C10).

Resolves chapter 10's research item ("A direct Sony adapter (Camera Remote SDK or PTP) is a later provider … its feasibility is a
research item"): **feasible on the A7R V over Wi-Fi and USB-C, with Capture One's USB claim as the one hard constraint (C1).**
Core stays manufacturer-neutral (H7): Sony, the SDK, FTP and IOKit are named only in this chapter, in adapter and infrastructure
code, and in evidence.

## What the photographer gets
- The camera connects by itself when it is on: over USB when the cable is in, over Wi-Fi otherwise, switching with no button press
  (about 7 s to USB, 6 s back to Wi-Fi). A switch never happens during an exposure or a protected multi-frame operation.
- The header's camera dot shows the camera, the link (cable or wireless) and, when something needs them, one plain sentence:
  "Capture One is holding the camera's USB connection: shooting continues over Wi-Fi" **[D1]**, "Confirm the camera's fingerprint",
  "The camera refused the password", "3 photos taken but not received for 60 s: check the camera's FTP status".
- The capture button and the camera's own shutter both work. Every photo arrives by the camera's own Auto FTP into the session
  that was open when the shutter fired, and then behaves exactly as a tethered capture does today (chapter 06).
- Nothing changes after arrival: review, composition, Capture One previews and retouch, export and publish are as today.

## Shape
```
camera ── Wi-Fi ── Auto FTP (always on) ──────────────▶ FTP receiver (infra) ── file arrived ──┐
   │  └── Wi-Fi SDK session ─┐                                                                  ├─▶ capture ledger (core mechanism)
   └───── USB SDK session ───┴─▶ camera sidecar (C++, SDK) ── stdio JSON ──▶ direct camera adapter ┘     │
                                                                             (integrations)            ▼
                                                                                         capture folder → ArrivalService (MC-2, unchanged)
```
- **Originals take one path: camera Auto FTP, on both transports.** The SDK is control and state only: shutter, properties read,
  capture notifications, FTP status. The app never changes camera network, FTP or store-destination settings; the photographer
  configures the camera once (C10, lab handoff §6).
- **Camera sidecar.** The lab's `crlab` (`auto` mode), copied into this repo without any Sony file, built with CMake against an SDK
  kept outside the repo, shipped as a Tauri `externalBin`. This is the "demonstrated need" chapter 11 asks of any sidecar: the SDK
  is C++, needs single-thread Init/Release and crashes on re-Init (C8). No Python anywhere.
- **FTP receiver** in `crates/infra` (Rust, e.g. `libunftp` with a custom storage back-end), owned by the app process.
- **Capture ledger** in `crates/core/src/mechanisms` — pure logic, no I/O, manufacturer-neutral.

## Adapters and selection [D2]
One camera adapter is active at a time, chosen in Settings → Camera: **Through Capture One** (today's A2/R27 adapter, kept unchanged)
or **Direct** (this chapter). Only one host can own the camera. Switching the setting rebuilds the services (as settings changes do
today) and never touches the camera's settings. With **Direct**, the editing provider is still Capture One for everything else
(previews, retouch, export); Capture One simply holds no tether.

## Port changes (core, neutral)
- `CameraStatus` grows from `Connected(name) | Disconnected` to `Connected { name, link } | Connecting { link } | Busy { reason } |
  NeedsAction { reason } | Disconnected`, where `link` is `Cable | Wireless | Other` and `reason` is a neutral enum plus a
  display string the adapter supplies (e.g. the name of the app holding the cable connection). The Capture One adapter reports
  `Connected { link: Other }` / `Disconnected` as today.
- `CameraProvider` gains an event stream (status changes and **exposure observed** `{ capture_id?, at, initiated: App|Camera,
  provisional }`) instead of being polled; `capture()` returns a `capture_id`. New capability `camera.events.exposure`. Capabilities
  of the direct adapter: `camera.status`, `camera.capture`, `camera.transfer`, `camera.events.exposure`, `camera.settings.read`
  (read-only; not used by the UI yet). `camera.connect` / `camera.discover` stay undeclared: connection is automatic.
- A **file-arrival source** port (`ArrivalSource`) separate from the camera: the FTP receiver implements it; it reports
  `{ camera_filename, written_path, sha256, size, started_at, completed_at }`.

## MC-31 — Camera sidecar protocol and lifecycle
- **Process.** Started by the adapter when Direct is selected and the app is running; one instance; stdin one command per line,
  stdout one JSON event per line (unknown fields and events ignored), stderr diagnostics only (logged, never parsed). Shutdown:
  `quit`, wait up to 3 s, then kill. A crash or kill is followed by a restart with bounded backoff (1, 2, 4, 8, 16 s, then every 30 s);
  the SDK is never re-initialised in a running process.
- **Secrets.** The camera's Access Authentication user and password and the FTP password are kept in the macOS Keychain **[D6]**,
  passed to the sidecar only through its environment at spawn, never logged, never written to settings.json or the repo. Pinned
  SSH host-key fingerprints live under the app's support directory, never in the repo.
- **Fingerprint.** On the first Wi-Fi connect the app shows the camera's fingerprint and asks the photographer to compare it with the
  camera's menu; it is pinned only on their confirmation. A mismatch later is `NeedsAction`, never auto-accepted. **An SSH
  authentication failure is never retried** (camera lockout risk): it becomes `NeedsAction` until the photographer re-enters the password.
- **Capture.** `capture id=<uuid> af=1`: half-press, AF wait up to 3 s, release (lab H-1). `capture.acknowledged` means the command was
  accepted, `capture.exposed` that the camera reported an exposure; **neither means a file exists** (C3). `capture.exposure_missing` and
  `capture.uncertain` are ledger inputs, not errors shown as failures.
- **USB claim.** `usb.claimed_by` (owner process named) becomes `NeedsAction` with the app's name and "shooting continues over Wi-Fi"
  **[D1]**. The app never kills `ptpcamerad` or any other process.
- **Camera identity.** The camera is one configured device holding both ids (Wi-Fi MAC, USB serial) (C9); events are never keyed on
  `camera_id` alone.

## MC-32 — FTP receiver
Guarantees (each a hardware-free acceptance test ported from the lab's 18 receiver tests):
1. A reset or aborted data connection is a **failed** upload, never a completed file (F-1).
2. Stage to `.<name>.part` in the target folder, then rename atomically; a partial file is never visible under a candidate name.
3. Never overwrite; never delete; refuse delete and overwriting rename commands.
4. SHA-256 after the rename, outside the transfer timing.
5. Private-network peers only, never a public interface; authentication with a strong random password from the Keychain; passive
   mode only on a fixed port range; at least 2 concurrent uploads (H-11). Plain FTP on the private network in the first phases;
   FTPS (explicit TLS, camera CA import) **[D5]**.
6. **Target folder.** The receiver writes each file into `<capture root>/<folder key>/` (MC-2's per-session capture folders, MC-8 folder
   key, `_Unassigned` when no session) of the session **active at the matched exposure time** from the ledger, or, if no exposure
   matches, of the session active when the upload started **[D3]**. From there MC-2 applies unchanged: the watcher sees a finished file
   (the atomic rename satisfies the stability rule), attributes it by its parent folder, creates the `photos` row.
7. **Duplicates** (camera filename without `__dupN`/`_N` suffix, plus SHA-256, C4): an identical second delivery is kept under
   `<capture root>/.duplicates/` (a dot folder, which MC-2's candidate filter ignores), never counted, never a photo, never deleted. A
   same-name file with a different hash (camera counter reused) is a new photo, written with a `__dupN` suffix.

## MC-33 — Capture ledger and backlog watchdog
Port of the lab ledger's semantics (its 15 tests become unit tests of this mechanism):
- Association by camera filename when known; otherwise by file-stem order to the oldest exposure still expecting a file, marked
  **provisional**. RAW and JPEG with the same stem are one shot. One exposure may produce several files.
- A shot is complete only when an expected file count is known and met; elapsed time never completes a shot.
- After a session loss during a protected operation: `uncertain`. At a deadline: `missing_at_deadline`, an observation, never proof of loss.
- Camera-initiated exposures (no app capture pending) are shots, not orphans. Gaps in camera file numbering are reported.
- **Watchdog:** exposures with no file older than 60 s while the receiver is idle → one notice and an `ftp-status` read shown on the
  health screen. An SDK reconnect at a safe point is offered as a button, never done automatically (it released a stall once, H-11: a
  hypothesis, not a fix).
- The ledger is the app's memory of shots; it does not create photos (MC-2 does, from files). It persists across restarts so a late
  file still finds its exposure.

## Capture One alongside [D1][D3]
- With **Direct**, the app keeps setting Capture One's capture folder per session (MC-17) so Capture One browses the folder the receiver
  writes into, and keeps the capture-done hook installed (it simply does not fire; the watcher path of MC-2 carries arrival).
- **Owed verification in Capture One 16.8.6 before planning** (§Verification): files written into the capture folder by another process
  must appear in the document and be addressable by our scripts (MC-19), take the preview render (MC-16) and the live-adjusted hook; if
  Next Capture Adjustments do not apply to them, the app applies the session's style through the editing provider's `style.apply` on
  arrival (only when `style_name` is set, 10C MC-26 step 5).

## Out of scope (later chapters)
Live view (needs its own frame channel); camera settings write and intent presets; bracketing, Pixel Shift and capture groups (one
result from many files; the owner's research intent tracks C–G); SDK Save-to-PC fast path **[D4]**; the ZV-E10 on the direct adapter
(untested); a second camera.

## Verification owed before planning
In Capture One 16.8.6 on the dev Mac (orchestrator, with the owner's go-ahead, scratch session only, files never deleted — H2):
- V1 An ARW copied into `Capture/<key>/` by another process appears in the document, and how fast; our scripts find its variant (MC-19).
- V2 Next Capture Adjustments (style, orientation, ICC) are applied to it, or not.
- V3 The `capture done script` does not fire for it (expected), the watcher path creates exactly one photo (M9).
- V4 `process` with the preview recipe renders it (MC-16); `primary variant adjusted script` fires when it is edited.
- V5 Capture One's Hot Folder (UI only, not in the dictionary) as an alternative arrival: does it import as a capture with NCA and keep
  the per-session folder?
In the lab (owner, one run): V6 `make exp-05` with Capture One open in a candidate configuration, and with the camera's USB session
opened **before** Capture One launches — does `ptpcamerad` still claim the interface?

## Hardware acceptance (packaged build; lab handoff §10)
| Test | Pass |
|---|---|
| Wi-Fi connect (SSH, pinned fingerprint) | connected ≤ ~6 s from start |
| USB connect | ≤ ~4 s from start |
| Capture | acknowledgement ≤ 50 ms; exposure ≤ ~0.6 s after the command (AF lock) |
| RAW by Auto FTP during a Wi-Fi session | every exposure arrives during the session, ~4–4.5 s after exposure, intact (ARW header, SHA-256), one photo in the right session |
| RAW by Auto FTP during a USB session | same |
| Cable in | USB session ≤ ~8 s after the cable event, no camera interaction |
| Cable out | Wi-Fi session ≤ ~6 s after the cable event |
| Plug-in during a protected operation | switch deferred; done within ~1.5 s after busy clears |
| Never two sessions | at most one SDK session over the whole test |
| FTP receiver restarted | the camera re-sends the pending shot within seconds; one photo |
| Upload cut mid-transfer | recorded failed; full file re-sent; no partial file visible; one photo |
| Sidecar killed | restarted sidecar reconnects in ~1 s, no `SessionAlreadyOpened` |
| Capture One running | USB works, or the header names Capture One as holding the cable connection and shooting continues over Wi-Fi |

## Invariants (to register in chapter 14 on acceptance)
- **M37 (MC-32).** Every completed camera upload yields exactly one visible file under a candidate name, byte-identical to what the
  camera sent; an aborted upload yields none; nothing is ever overwritten or deleted.
- **M38 (MC-33/MC-2).** Every RAW the camera delivers becomes exactly one `photos` row, in the session active at its exposure (or at
  upload start when no exposure matches); an identical duplicate delivery becomes none.
- **M39 (MC-31).** At most one SDK session exists at any time; no transport switch starts while the camera is busy; a secret never
  appears in a log, a settings file or the repo.

## Chapters this amends on acceptance
10 (research item resolved; two camera adapters and the setting), 03 (camera: A7R V; ZV-E10 kept for the Capture One adapter),
05A MC-2 (arrival via the receiver's target-folder rule; `.duplicates` ignored), 11 (camera sidecar, C++ via CMake, the SDK outside the
repo, `externalBin`, dylib re-signing), 12 (Keychain for camera and FTP secrets; health items: USB claimed, fingerprint, auth refused,
backlog), 13 (direct adapter moves from deferred to must-ship), 14 (M37–M39), 15 (decision rows), 16 (changelog entry).

## Decisions for the owner
Listed with recommendations in the orchestrator's brief of 2026-11-06 and recorded here once answered: [D1] Capture One and USB ·
[D2] which host owns the tether · [D3] the arrival seam · [D4] SDK Save-to-PC · [D5] FTPS · [D6] Keychain now · [D7] SDK build,
signing and licence · [D8] the hand-written SDK shim in a public repo.
