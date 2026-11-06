# 10E — Direct camera adapter: camera sidecar, file paths, capture ledger, camera ownership

**Status: RATIFIED amendment (owner 2026-11-06, changelog 31; decisions R42–R45).** Evidence: `evidence/sony_camera_lab.md`
(lab finding ids H-n, F-n, constraints C1–C10, and Session Manager's own checks V1–V6).

Resolves chapter 10's research item ("A direct Sony adapter (Camera Remote SDK or PTP) is a later provider … its feasibility is a
research item"): **feasible on the A7R V over Wi-Fi and USB-C.** Session Manager becomes the camera's controller; Capture One stays
the editing provider and no longer holds a tether (owner 2026-11-06). Core stays manufacturer-neutral (H7): Sony, the SDK, FTP and
IOKit are named only in this chapter, in adapter and infrastructure code, and in evidence.

## What the photographer gets
- The camera connects by itself when it is on. **USB-C has priority** whenever the cable is in and the connection is free; Wi-Fi is
  the fallback. Switching needs no button press (about 7 s to USB, 6 s back to Wi-Fi) and never happens during an exposure, a
  multi-frame operation or a file transfer (owner 2026-11-06: "the whole point … is to transfer files faster through the usbc").
- **On USB, photos come over the cable** (about 2 s after the shutter for a 51 MB RAW); on Wi-Fi they come by the camera's Auto FTP
  (about 4 s). Auto FTP keeps running on both as a safety net; the second copy of a photo is recognised and never becomes a second photo.
- **Session Manager keeps control of the camera.** If another program holds or grabs the camera, Session Manager says which one
  and how to fix it, instead of silently failing (owner 2026-11-06: "tomorrow the photographer installs some program and it happens
  to read the camera, the session manager stops working and nobody knows why"). Shooting continues over Wi-Fi meanwhile.
- The header's camera dot shows the camera, the link (cable or wireless) and, when something needs the photographer, one plain
  sentence ("Capture One is holding the camera's USB connection — turn off its USB tethering: Preferences → Capture"; "Confirm the
  camera's fingerprint"; "The camera refused the password"; "3 photos taken but not received for 60 s").
- The capture button and the camera's own shutter both work. Every photo lands in the session that was open when the shutter fired
  and then behaves exactly as a capture does today (chapter 06): review, composition, Capture One previews and retouch, export, publish.

## Shape
```
camera ── USB-C ── SDK session (control + Save-to-PC) ─┐
   │                                                   ├─▶ camera sidecar (C++) ── stdio JSON ──▶ direct camera adapter ─┐
   ├────── Wi-Fi ── SDK session (control) ─────────────┘                                        (integrations)            │
   └────── Wi-Fi ── Auto FTP (always on) ──────────────▶ FTP receiver (infra) ──────────────── file arrived ──┐            │
                                                                                                          capture ledger (core)
                                                                    session capture folder ◀── one file per shot ──┘
                                                                              │
                                                                ArrivalService (MC-2, unchanged) → Capture One scan + style (MC-34)
```
- **Two file paths, one result.** USB: SDK Save-to-PC, fastest. Always: camera Auto FTP over Wi-Fi. The ledger keeps the first
  complete copy of each camera file and sets later identical copies aside (MC-33). The app never changes the camera's network or FTP
  settings; the photographer configures those once.
- **Camera sidecar.** The lab's `crlab`, copied into this repo without any Sony file, extended with the lab's Save-to-PC and
  store-destination commands in `auto` mode, built with CMake, shipped inside the app as a Tauri `externalBin`. This is the
  "demonstrated need" chapter 11 asks of a sidecar: the SDK is C++, needs single-thread Init/Release and crashes on re-Init (C8).
  No Python anywhere.
- **FTP receiver** in `crates/infra` (Rust, e.g. `libunftp` with a custom storage back-end), owned by the app process.
- **Capture ledger** in `crates/core/src/mechanisms`: pure logic, no I/O, manufacturer-neutral.

## Camera support packages (owner 2026-11-06)
The app owns its camera layer and installs anywhere: each supported camera make is an **adapter shipped inside the app** (for Sony:
the sidecar plus the Rust adapter). The make's vendor SDK runtime is **installed by the user from the app**: Settings → Camera →
"Add camera support → Sony" explains where to download Sony's Camera Remote SDK (the user accepts Sony's terms there), takes the
downloaded archive, checks it is a version the adapter supports (2.02 today), and copies only the runtime libraries (`libCr_Core`,
`libmonitor_protocol*`, the `CrAdapter/` folder) into the app's support folder, where the sidecar loads them. The app signs them for
local use so Gatekeeper never removes them (C8); quarantine is never stripped from anything the user did not install through this
flow. A later make (another vendor SDK) is another adapter plus another support package; core does not change. Nothing from any vendor
SDK is ever committed to the repo; the sidecar is compiled on a build machine that has the SDK headers, and CI builds the stub sidecar.

## Port changes (core, neutral)
- `CameraStatus` grows from `Connected(name) | Disconnected` to `Connected { name, link } | Connecting { link } | Busy { reason } |
  NeedsAction { reason, holder? } | Disconnected`, where `link` is `Cable | Wireless`, `reason` is a neutral enum and `holder` a
  display name the adapter supplies (the program holding the camera).
- `CameraProvider` gains an event stream (status changes and **exposure observed** `{ capture_id?, at, initiated: App|Camera,
  provisional }`); `capture()` returns a `capture_id`. Capabilities of the direct adapter: `camera.status`, `camera.capture`,
  `camera.transfer`, `camera.events.exposure`, `camera.settings.read`, and `camera.settings.write` limited in this chapter to the store
  destination (MC-31). The port is shaped so later work adds settings writes for capture intents and a live-view frame channel
  without changing core's photo model.
- An **arrival source** port (`ArrivalSource`) separate from the camera: the FTP receiver and the adapter's Save-to-PC downloads
  implement it; each reports `{ camera_filename, written_path, sha256, size, via: Cable|Wireless, completed_at }`.
- The Capture One camera adapter (A2/R27) stops being used when this chapter ships; its code stays until the hardware acceptance below
  passes on the work Mac, then it is removed.

## MC-31 — Camera sidecar protocol and lifecycle
- **Process.** One instance, started by the adapter while the app runs and Sony support is installed; stdin one command per line,
  stdout one JSON event per line (unknown fields and events ignored), stderr diagnostics only (logged, never parsed). Shutdown: `quit`,
  wait up to 3 s, then kill. A crash is followed by a restart with bounded backoff (1, 2, 4, 8, 16 s, then every 30 s); the SDK is never
  re-initialised in a running process.
- **Transport policy.** USB when the cable is in and the connection is free; Wi-Fi otherwise. A switch waits while the camera is busy
  (exposure, multi-frame operation, Save-to-PC transfer in progress) and happens at the next safe point (lab H-18). After three failed USB
  connects the sidecar stays on Wi-Fi until the cable is re-plugged or the holder is gone (no periodic USB probing, F-3).
- **Store destination** (the only camera setting the app writes in this chapter). On a USB session the adapter sets [Still Img. Save
  Dest.] to *Host PC + Memory Card*; on a Wi-Fi session to *Memory Card only* (Auto FTP delivers). Every write is read back and verified
  (C10, H-3); a failed verification is `NeedsAction`. The photographer agrees to this once in camera setup. The card always keeps a copy.
- **Save-to-PC downloads** are written by the sidecar into a staging folder (`<capture root>/.incoming/`, a dot folder MC-2 ignores) and
  handed to the ledger, which moves the file into the target session folder by atomic rename.
- **Secrets.** The camera's Access Authentication user and password and the FTP password are kept in the macOS Keychain, passed to the
  sidecar only through its environment at spawn, never logged, never in settings.json or the repo. Pinned SSH host-key fingerprints live
  under the app's support folder.
- **Fingerprint.** On the first Wi-Fi connect the app shows the camera's fingerprint and asks the photographer to compare it with the
  camera's menu; it is pinned only on their confirmation. A mismatch later is `NeedsAction`. **An SSH authentication failure is never
  retried** (camera lockout risk).
- **Capture.** Half-press, AF wait up to 3 s, release (H-1). `capture.acknowledged` means accepted, `capture.exposed` that the camera
  reported an exposure; **neither means a file exists** (C3).
- **Camera identity.** One configured camera holding both ids (Wi-Fi MAC, USB serial) (C9).

## MC-32 — FTP receiver
Guarantees (each a hardware-free acceptance test ported from the lab's 18 receiver tests):
1. A reset or aborted data connection is a **failed** upload, never a completed file (F-1).
2. Stage to `<capture root>/.incoming/.<name>.part`, then hand the finished file to the ledger; a partial file is never visible under a
   candidate name.
3. Never overwrite; never delete; refuse delete and overwriting rename commands.
4. SHA-256 after the transfer, outside its timing.
5. Private-network peers only, never a public interface; authentication with a strong random password from the Keychain; passive mode
   only on a fixed port range; at least 2 concurrent uploads (H-11). **Plain FTP on the private network** in this chapter; FTPS (explicit
   TLS, camera CA import) is a later step before shooting on any network the photographer does not own (owner 2026-11-06).

## MC-33 — Capture ledger, duplicates, target folder, watchdog
Port of the lab ledger's semantics (its 15 tests become unit tests of this mechanism), plus where files go:
- **Association** by camera filename when known; otherwise by file-stem order to the oldest exposure still expecting a file, marked
  provisional. RAW and JPEG with the same stem are one shot; one exposure may produce several files. A shot is complete only when an
  expected count is known and met; elapsed time never completes a shot. `uncertain` after a session loss during a protected operation;
  `missing_at_deadline` is an observation, never proof of loss. Camera-initiated exposures are shots, not orphans. Gaps in camera file
  numbering are reported.
- **First complete copy wins.** The key is the camera filename without server or retry suffixes (`__dupN`, `_N`) plus SHA-256. The first
  complete copy of a camera file, by either path, is moved into the target folder; a later identical copy is moved to
  `<capture root>/.duplicates/` (ignored by MC-2), never counted, never a photo, never deleted. A same-name file with a different hash
  (camera counter reused) is a new photo, renamed with a `__dupN` suffix.
- **Target folder.** `<capture root>/<folder key>/` (MC-2's per-session folders, MC-8 key, `_Unassigned` when no session) of the session
  **active at the matched exposure**, or, if no exposure matches, of the session active when the file completed. From there MC-2 applies
  unchanged (V3: one photo per file, about 2 s after the rename).
- **Watchdog.** Exposures with no file older than 60 s while no transfer is active → one notice and a camera FTP-status read shown on the
  health screen. An SDK reconnect at a safe point is offered as a button, never done automatically (H-11).
- The ledger persists across restarts so a late file still finds its exposure. It never creates photos; MC-2 does, from files.

## MC-34 — Camera ownership guard (owner 2026-11-06)
Session Manager must hold the camera; when it cannot, it names who does and how to fix it. It never kills, forces or reconfigures another
program or system service (`ptpcamerad` included).
- **Detect.** Before and after every USB connect attempt, the adapter reads the exclusive owner of the camera's USB interface (lab
  `usb.claimed_by`). When the holder is the system camera service (`ptpcamerad`), it names the programs behind it: running applications
  that use macOS camera services (seen on the dev Mac 2026-11-06: Capture One, Imaging Edge Desktop, a Photos widget). On Wi-Fi, a camera
  that refuses PC control because a phone app is connected is reported the same way.
- **Get there first and never let go (V6).** Capture One triggers the system camera service's claim even with its Sony tethering
  switched off, but the claim fails while our SDK session already holds the interface. So: at app start the app opens the USB session
  **before** it launches Capture One (it launches Capture One itself, 10C MC-26); while Capture One runs, the sidecar **keeps the USB
  session open without a break** (no disconnect for idle, no periodic reconnect; a transport switch away from USB happens only when the
  cable is pulled). The sidecar is restarted only when it has died.
- **Take it back from Capture One.** If the USB interface is held by the system camera service on Capture One's behalf (after a sidecar
  crash, or a cable re-plug the service wins), the app offers **"Take the camera back"**: when no file is in transit and no exposure is
  pending it saves the Capture One document, quits Capture One, opens the USB session, relaunches Capture One and reopens the document
  (about 15–20 s; Wi-Fi shooting continues until then). **It runs only on the photographer's click** (owner 2026-11-06: "at first is
  click, right when i see it working … we will move to automatic"); the notice appears as soon as the claim is seen, e.g. right after a
  cable re-plug, which the system camera service always wins while Capture One runs (V6: within 28 ms). An automatic mode is a later
  owner decision. It never touches any other program.
- **Guide.** For other programs the notice says what to change: Imaging Edge Desktop → quit it and stop it opening at login; Photos / Image
  Capture → "Connecting this camera opens: No application"; unknown programs: "<name> is using the camera: close it or stop it from using
  cameras". The health screen keeps the item until the holder is gone; the camera dot shows `NeedsAction` with the holder's name; Wi-Fi
  continues.
- **Setup check.** Settings → Camera runs the same check on demand ("Test camera connection") and lists every program that could take
  the camera.

## Capture One alongside (V1–V5)
- With the direct adapter, Capture One holds no tether; the app still sets the capture folder per session (MC-17) and installs the hooks;
  the capture-done hook does not fire, and the watcher path of MC-2 carries arrival (V3).
- **Making Capture One see new files (V1).** Capture One sees a file written by another process only when it scans the folder, which it
  does when that folder becomes its **current collection** and then about every 2 s while it stays current. So the app sets Capture One's
  current collection to the active session's capture folder when the session opens and whenever a photo arrives while Capture One shows
  another collection; the preview render waits for the variant (bounded wait, then the normal deferred retry) instead of failing
  (V4: renders failed with `variant_not_found` and stopped retrying).
- **Style on arrival (V2).** Next Capture Adjustments do not apply to such files; when `style_name` is set and listed, the app applies it to
  each new variant with `apply style` (verified by script).
- Not used: `import` (copies the file into the session root, V1), Hot Folder (not scriptable, V5).

## Out of scope (later chapters)
Live view (its own frame channel); camera settings beyond the store destination (capture intents and presets); bracketing, Pixel Shift
and capture groups (one result from many files: the owner's research intent, tracks C–G); FTPS; the ZV-E10 on the direct adapter
(untested); a second camera.

## Owed before planning
- ~~V6~~ done 2026-11-06 (evidence §V6): Capture One is the trigger; its tether checkboxes do not help; SDK-first holds; release loses
  the camera within 1 s. ~~Store-destination write over USB~~ verified both ways (evidence §V6). Cable re-plug with Capture One running: lost,
  `ptpcamerad` holds the interface within 28 ms of the plug-in (evidence §V6); covered by "Take the camera back".
- **Camera support package spike:** install the SDK runtime through the flow above on a second macOS user account, confirm the sidecar
  loads it and connects on USB and Wi-Fi from the packaged app.

## Hardware acceptance (packaged build; lab handoff §10, adapted)
| Test | Pass |
|---|---|
| Wi-Fi connect (SSH, pinned fingerprint) | connected ≤ ~6 s from start |
| USB connect | ≤ ~4 s from start |
| Capture | acknowledgement ≤ 50 ms; exposure ≤ ~0.6 s after the command (AF lock) |
| RAW over USB (Save-to-PC) | on the Mac ≤ ~2.5 s after the command, intact (ARW header, SHA-256), one photo in the right session; the FTP copy lands in `.duplicates`, identical |
| RAW by Auto FTP during a Wi-Fi session | every exposure arrives during the session, ~4–4.5 s after exposure, intact, one photo in the right session |
| Cable in | USB session ≤ ~8 s after the cable event, no camera interaction |
| Cable out | Wi-Fi session ≤ ~6 s after the cable event |
| Plug-in during a protected operation | switch deferred; done within ~1.5 s after busy clears |
| Never two sessions | at most one SDK session over the whole test |
| FTP receiver restarted | the camera re-sends the pending shot within seconds; one photo |
| Upload cut mid-transfer | recorded failed; full file re-sent; no partial file visible; one photo |
| Sidecar killed | restarted sidecar reconnects in ~1 s, no `SessionAlreadyOpened` |
| Another program holds the USB connection | the header names it with the fix; shooting continues over Wi-Fi; USB resumes once it is released |
| New photo while Capture One shows another collection | Capture One shows it and the adjusted preview arrives |

## Invariants (registered in chapter 14)
- **M37 (MC-32, MC-31).** Every completed camera transfer yields exactly one visible file under a candidate name, byte-identical to what
  the camera sent; an aborted transfer yields none; nothing is ever overwritten or deleted.
- **M38 (MC-33, MC-2).** Every camera file becomes exactly one `photos` row, in the session active at its exposure (or at completion when no
  exposure matches), whichever path delivered it first; an identical second delivery becomes none.
- **M39 (MC-31, MC-34).** At most one SDK session at any time; no transport switch starts while the camera is busy; a store-destination
  write is always verified; a secret never appears in a log, a settings file or the repo; no other process is ever killed.

## Chapters this amended (2026-11-06)
10 (research item resolved; Session Manager controls the camera; the Capture One camera adapter retired after acceptance), 03 (camera:
A7R V), 05A MC-2 (`.incoming` and `.duplicates` ignored; arrival via MC-33's target folder), 10A MC-17 (current collection follows the
session), 10C MC-26 (camera connected before launching Capture One; style applied by script to arrivals), 11 (camera sidecar, camera
support packages, C++ via CMake, no vendor SDK in the repo), 12 (Keychain for camera and FTP secrets; health items: camera holder,
fingerprint, refused password, backlog, store destination), 13 (direct adapter moves from deferred to must-ship; FTPS, live view,
capture groups stay deferred), 14 (M37–M39), 15 (R42–), 16 (changelog).

## Owner answers (2026-11-06)
D1 USB has priority, Wi-Fi is the fallback, and Session Manager must keep control of the camera against other programs → MC-34.
D2 Session Manager controls the camera (live view and settings later) → Capture One tether retired. D3 a place compatible with the app
→ the session capture folder, with V1/V2's two rules. D4 on USB files come over the cable, the duplicate rule handles the FTP copy →
Save-to-PC on USB. D5 plain FTP now, FTPS later. D6 Keychain now. D7 the app owns its camera layer and is portable; users add a make's
SDK through the app → camera support packages. D8 (orchestrator's call): the lab's hand-written SDK stand-in headers are not copied into
this public repo; CI builds the stub sidecar.
