# 10E — Direct camera adapter: camera sidecar, file paths, capture ledger, camera ownership

**Status: RATIFIED amendment (owner 2026-11-06, changelog 31; decisions R42–R45).** Evidence: `evidence/sony_camera_lab.md`
(lab finding ids H-n, F-n, constraints C1–C10, and Session Manager's own checks V1–V6).

**Mechanism inventory (2026-11-06, changelog 32).** The "Inventory additions" under each contract make it contract-grade
against the lab code and this repo. Items marked **(ratify R-In)** settle a gap or a contradiction without changing what the
photographer gets; they are listed for the owner in `implementation/handoffs/reviewer/handoff_direct_camera_inventory_r1.md`.
Owner card CI1 (§Capture One alongside) was answered "always jump" on 2026-11-06 (R46).

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

**Inventory additions (2026-11-06) (ratify R-I36).**
- **What is verified.** The app carries, for each supported SDK version, the list of runtime files and their SHA-256. The manifest is
  generated on the build machine from its SDK and compiled in; it holds hashes only, no Sony bytes. The files are `libCr_Core.dylib`,
  `libmonitor_protocol.dylib`, `libmonitor_protocol_pf.dylib` and the `CrAdapter/` folder (`libCr_PTP_USB`, `libCr_PTP_IP`, `libssh2`,
  `libusb-1.0.0`), with exact names taken from the manifest. A missing file or a hash mismatch refuses the install and names the file.
  After install, the sidecar's `version` must report an SDK version on the supported list. Settings → Camera shows the installed
  version.
- **Where it lives.** The app extracts the archive itself, so no quarantine attribute reaches the copies. The destination is
  `~/Library/Application Support/com.loorenz.sessionmanager/camera/sony/<sdk version>/`, and it holds the runtime **and a copy of the
  bundled sidecar**: the SDK finds `CrAdapter/` next to the executable, and a signed app bundle cannot receive files (lab handoff §5.3).
  The sidecar runs from there. After copying, the folder is signed ad-hoc locally. At each start, the copy's SHA-256 is compared with
  the bundled sidecar's, and on a mismatch (an app update) the sidecar is copied and signed again.
- **Unverified; a spike before the package phase is planned** (it extends §Owed before planning):
  - S1: the copied sidecar loads the runtime and connects on USB and Wi-Fi from a packaged, ad-hoc-signed app, on a second user
    account.
  - S2: whether hardened-runtime library validation refuses the separately signed Sony dylibs; if it does, the sidecar is signed with
    `com.apple.security.cs.disable-library-validation`.
  - S3: no Gatekeeper prompt or removal.
  - S4: macOS Local Network privacy for the sidecar's Wi-Fi connect and the receiver's inbound connections from the packaged app. The
    lab ran from Terminal, which held that permission. The app's `Info.plist` gains `NSLocalNetworkUsageDescription` (it has none
    today).
  - S5: reading which processes have ImageCaptureCore loaded.
  - S6: the firewall and Keychain prompts after an update of an ad-hoc-signed build.

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
- **Status values and precedence (ratify R-I37).** `BusyReason` = `Exposure | Transfer | Switching`. `NeedsActionReason`, in
  precedence order: `AuthenticationRefused`, `FingerprintMismatch`, `FingerprintUnconfirmed`, `PhoneAppConnected`,
  `HeldByOtherProgram { holder }`, `StoreDestinationUnverified`, `FilesNotReceived { count }`, `SupportNotInstalled`. The status is
  the first that holds of: any `NeedsAction` (the first reason in that order; it wins even while connected on the other link, as MC-34
  requires) › `Busy` › `Connected` › `Connecting` (sidecar running, no session, and the sidecar reports it is connecting on a link; a running sidecar that reports no connecting attempt is `Disconnected`) › `Disconnected`. The camera dot shows that one; the
  health screen lists every reason that holds.
- **Camera independent of Capture One's health (ratify R-I33).** With the direct adapter, the camera status and `capture()` never
  depend on Capture One's health items. Today `CameraService` reports `Disconnected` and refuses capture while
  `capture_folder_not_set` or `document_not_front` is set; those gates stay only for the Capture One camera adapter.
- `exposure observed.at` and `ArrivalSource.completed_at` are Mac-clock UTC milliseconds (05A MC-4): the sidecar envelope's
  `timestamp`, and the time the receiver saw the data connection close.

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
  sidecar as R-I21 specifies (its first stdin line, never its environment), never logged, never in settings.json or the repo. Pinned SSH host-key fingerprints live
  under the app's support folder.
- **Fingerprint.** On the first Wi-Fi connect the app shows the camera's fingerprint and asks the photographer to compare it with the
  camera's menu; it is pinned only on their confirmation. A mismatch later is `NeedsAction`. **An SSH authentication failure is never
  retried** (camera lockout risk).
- **Capture.** Half-press, AF wait up to 3 s, release (H-1). `capture.acknowledged` means accepted, `capture.exposed` that the camera
  reported an exposure; **neither means a file exists** (C3).
- **Camera identity.** One configured camera holding both ids (Wi-Fi MAC, USB serial) (C9).

**Inventory additions (2026-11-06).**
- **One camera runtime per app process (ratify R-I18).** The sidecar, the FTP receiver and the ledger are created once per app
  process and live outside the runtime that a settings save rebuilds. Today every settings save that changes more than the theme runs
  `App::build` and builds the new runtime **before** shutting the old one down: a second sidecar would compete with the first for the
  USB interface, and a second FTP listener would fail to bind while the first holds the port. A settings save hands the camera runtime to
  the new runtime unchanged. A camera setting that changed is applied in place: receiver port, passive range or password → the listener is
  closed, then reopened; camera credentials → used at the sidecar's next spawn; the sidecar is restarted for them only while it holds no
  USB session. Never two sidecar processes, never two listeners.
- **Connect sequence (ratify R-I19).** In `auto` mode the sidecar does three things on every new session (USB and Wi-Fi), before it
  reports `connection.ready`: (1) it sets the Save-to-PC folder (`SetSaveInfo`) to the run's staging folder (below), so a download on
  either link lands there and never in the sidecar's working directory; (2) it writes the store destination for that link and reads it
  back (≤ 3 s): USB *Host PC + Memory Card*, Wi-Fi *Memory Card*; (3) it emits `connection.ready` with `store_destination: verified | failed`
  and the value read. The write is persistent. The lab's `configure_save` path restores the previous value on every disconnect, which
  would block a cable-pull recovery for up to 3 s while it writes to a dead session, so it is not used. **The app never writes
  *Host PC* alone.** Both values keep the card copy, so a photo still reaches the card and Auto FTP in two cases: an exposure during a
  write, and a cable pull that leaves *Host PC + Memory Card* set until the next Wi-Fi session corrects it. Downloads made over Wi-Fi in
  the meantime land in staging and are ordinary deliveries (`via: Wireless`). `failed` → `NeedsAction { StoreDestinationUnverified }`,
  and the session is kept. On a clean `quit` during a USB session the sidecar first writes *Memory Card* (one attempt, ≤ 1 s).
- **Save-to-PC staging and completion (ratify R-I20).** The staging folder is `<capture root>/.incoming/sdk-<sidecar run id>/`, new for
  each sidecar process, so a file left by a crashed run never meets a new download of the same name. A file is handed to the ledger
  only on `sdk.download.completed` with `size_bytes > 0`. The adapter then re-reads the size, hashes the file itself (SHA-256, the same
  Rust code as the receiver) and reports `via: Cable`. A file in `.incoming/` without a completion event (session lost mid-download) is
  never promoted and never deleted; the card copy arrives by Auto FTP. While a download is expected (from `capture.exposed` on a USB
  session to the matching completion, at most 10 s), the sidecar holds busy `Transfer`. That busy state only defers a voluntary stop
  (app quit, a credentials restart), because a USB session is never left voluntarily.
- **Busy, precisely (ratify R-I22).** The busy reasons are `Exposure` (an app capture, from the command until it returns) and `Transfer`
  (above); the adapter reports `Switching` during a transport change. Auto FTP uploads do **not** defer a switch: they run over Wi-Fi
  whatever link the SDK session uses (lab H-14, H-18), and deferring on them would keep a fast-shooting Wi-Fi session from ever moving to
  the cable. "Never during a file transfer" in §What the photographer gets means a Save-to-PC download.
- **When a USB session may close (ratify R-I23).** It closes only when the cable is pulled (an IOKit or SDK report), when the camera ends
  it (power off, auto power-off, Mac sleep), on `quit` (app quit, and also end-of-file on stdin), or when the sidecar dies. It never
  closes for idleness, for a property or command error (the lab's `auto` has no such path; keep it that way), for the watchdog's
  reconnect (Wi-Fi only, MC-33) or for a settings save (R-I18). While a USB session is open, the app holds a macOS idle-sleep assertion,
  because a Mac sleep while Capture One runs loses the camera (V6). The setup guide asks for the camera's auto power-off to be off. A
  session the camera ends while Capture One runs is followed by MC-34's claim notice. The sidecar's IOKit match names the camera (vendor
  and product id), not every Sony device: the lab counts any device with vendor id 0x054C as camera presence. Protocol additions: the
  event `usb.initial { present }`, emitted once after the monitor starts; the command `usb-retry`, which clears the three-failure fallback
  and is used by MC-34.
- **Secrets (ratify R-I21).** The secrets are Keychain generic-password items under service `com.loorenz.sessionmanager.camera`, with
  accounts `access-auth-user`, `access-auth-password` and `ftp-password`. Only the app process reads them; the sidecar never touches the
  Keychain. Credentials reach the sidecar as its **first stdin line** (`credentials user=<…> password=<…>`, percent-encoded), not through
  its environment, because another process of the same user can read a child's environment (`ps -E`). Neither side ever logs that line
  (its `cmd.received` carries no fields). Every secret is registered with the log redactor (`logging::add_secret`) before the sidecar is
  spawned or the receiver starts, and the sidecar's stderr reaches the log only through that redactor. The app generates the FTP password
  (32 random characters from `[A-Za-z0-9]`) and shows it once in camera setup, for the photographer to enter on the camera; it can be
  changed there. No secret appears in argv, settings.json or the repo. Pinned fingerprints (not secret) live in
  `<app support>/camera/fingerprints/`, passed as `CRLAB_FINGERPRINT_DIR`. A known cost, checked in spike S6: with ad-hoc signing, every
  new build has a new code identity, so macOS asks again for Keychain access after each update.

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

**Inventory additions (2026-11-06) (ratify R-I24).**
- **Namespace.** The receiver's file system is virtual. In the lab setup the camera uses *Directory Hierarchy: Standard* and
  *Same File Name: Does Not Overwrite*. `MKD` and `CWD` of any path succeed and are ignored; a file is identified by its base name.
  In every directory, `LIST`, `NLST`, `SIZE` and `MDTM` answer for the names completed in the last 24 h, taken from the ledger's
  deliveries so a restart keeps them, as the lab's real directory did. If the camera then re-sends a file or renames it (`_1`), the
  duplicate rule (MC-33) handles it; hardware acceptance checks that it does not. `DELE`, `RMD` and `RNFR`/`RNTO` are refused (550/553):
  every rename, not only overwriting ones.
- **Upload lifecycle.** `STOR <name>` writes `<capture root>/.incoming/.<name>.part`; a name that is hidden or looks like a staging file
  is refused (553). One upload per name at a time: a second `STOR` of a name already in flight gets 450. `REST` + `STOR`/`APPE` resumes
  only onto an existing `.part` at least as long as the offset (truncated to it), else 554; a new `STOR` of the same name truncates its
  `.part`. `.part` files belong to the receiver, are never deleted, and fall outside M37's "never overwritten", which covers files under
  candidate names and in `.duplicates/`. **Completed** means the data connection closed with no socket error; a reset, an abort, a timeout
  or a pending `SO_ERROR` is a failure (F-1). On completion the receiver hashes the `.part` (outside the transfer timing) and hands
  `{camera_filename, written_path = the .part, sha256, size, via: Wireless, completed_at}` to the ledger, which renames it to its
  destination. A failure leaves the `.part` in place for a resume.
- **Network.** The control port and the passive range are settings (defaults 2121 and 50000–50009). Accepted peers: IPv4 10/8,
  172.16/12, 192.168/16, 169.254/16; IPv6 fc00::/7, fe80::/10. Any other peer gets 421 at connect.
- **Capture root without Capture One.** The receiver and the ledger compute the capture root from the configured document path with
  05A MC-2's formula (`canon(<document folder>/Capture)`) and never ask Capture One, which is quit during "Take the camera back"
  (MC-34). If that folder does not exist, `STOR` gets 451, and the camera keeps the file queued and retries.

## MC-33 — Capture ledger, duplicates, target folder, watchdog
Port of the lab ledger's semantics (its 15 tests become unit tests of this mechanism), plus where files go:
- **Association** by camera filename when known; otherwise by file-stem order to the oldest exposure still expecting a file, marked
  provisional. RAW and JPEG with the same stem are one shot; one exposure may produce several files. A shot is complete only when an
  expected count is known and met; elapsed time never completes a shot. `uncertain` after a session loss during a protected operation;
  `missing_at_deadline` is an observation, never proof of loss. Camera-initiated exposures are shots, not orphans. Gaps in camera file
  numbering are reported.
- **First complete copy wins.** The key is the camera filename without server or retry suffixes (`__dupN`, `_N`) plus SHA-256. The first
  complete copy of a camera file, by either path, is moved into the target folder; a later identical copy is moved to
  `<capture root>/.duplicates/` (ignored by MC-2), never counted, never a photo; deleted 24 h after verification (10F, owner 2026-11-09). A same-name file with a different hash
  (camera counter reused) is a new photo, renamed with a `__dupN` suffix.
- **Target folder.** `<capture root>/<folder key>/` (MC-2's per-session folders, MC-8 key, `_Unassigned` when no session) of the session
  **active at the matched exposure**, or, if no exposure matches, of the session active when the file completed. From there MC-2 applies
  unchanged (V3: one photo per file, about 2 s after the rename).
- **Watchdog.** Exposures with no file older than 60 s while no transfer is active → one notice and a camera FTP-status read shown on the
  health screen. An SDK reconnect at a safe point is offered as a button, never done automatically (H-11).
- The ledger persists across restarts so a late file still finds its exposure. It never creates photos; MC-2 does, from files.

**Inventory additions (2026-11-06).**
- **Persistence (ratify R-I25).** Two SQLite tables, added in one migration and written by the same process as `photos`:
  `camera_exposures(id, capture_id NULL, exposed_at, initiated app|camera, session_id NULL, status expecting|received|missing_at_deadline|uncertain|dismissed, expected_files NULL)`:
  one row when the adapter receives `exposure observed`; `session_id` is the active session read at that moment (null = Unassigned).
  `camera_deliveries(id, exposure_id NULL, camera_filename, file_key, sha256, size, via cable|wireless, completed_at, association filename|stem|fifo|none, outcome photo|duplicate|renamed_photo, session_id NULL, final_path, state pending|done)`.
  Rows are kept (a few thousand a day). A delivery row is written `pending`, with its final path, **before** the rename and set `done`
  after it. At startup, a `pending` row whose staging file still exists is renamed again; one whose final file exists is set `done`.
- **File key (ratify R-I26).** `file_key` is the adapter's normalised camera file name, compared case-insensitively, extension
  included. Core strips only the app's own collision suffix `__dupN`. Camera re-send suffixes are camera-specific, so the adapter strips
  them in `normalize_camera_filename` (Sony: `_<1–2 digits>` after the stem's trailing number, so `DSC01234_1.ARW` → `DSC01234.ARW`).
  Core never strips `_N`, because `IMG_1234.CR3` would collapse (H7). The lab's `file_key` strips only `__dupN`; its `_N` rule exists only
  for gap reporting.
- **Association, in a fixed order (ratify R-I26).** A delivered file gets its exposure from the first rule that matches:
  (1) a delivery with the same `file_key` **and the same SHA-256** exists (an identical copy) → that delivery's exposure; (2) a delivery
  with the same normalised stem, completed within 300 s of this one, exists → its exposure (RAW + JPEG); (3) **FIFO in a window**: the oldest exposure with status `expecting`, `exposed_at ≤ completed_at` and
  `completed_at − exposed_at ≤ 300 s` → `fifo`, provisional; (4) no exposure.
  The whole FIFO candidate set, not only the pick, decides the session. If every candidate in the window has the same `session_id`, that
  is the file's session, whichever exposure was picked. If they differ, the pick decides, and the health screen lists the photo ("may
  belong to <other session>"), so a lost or reordered file is visible instead of shifting every later file. The window keeps one lost file
  (an exposure that never yields a file) from capturing every later file forever. The lab's FIFO has no window, and the ledger now
  persists across restarts.
  *Clarification (orchestrator, 2026-11-06, from line 28 and R-I27):* the camera's counter wraps after DSC09999, so a name alone never
  ties a new file to an old exposure; a reused name with other bytes goes through rule (3) and lands in the session open at its exposure.
- **Duplicates (ratify R-I27).** A delivery is a duplicate iff an earlier delivery has the same `file_key` and the same SHA-256, whatever
  its exposure (or none) and however long ago. The lab compares only inside one capture and never for unassociated files, so a
  camera-initiated shot delivered twice would have become two photos. Same key with a different hash → a new photo, named with the lowest
  free `__dupN`. One ledger worker makes every decision, one file at a time: decide → write the `pending` row → rename → `done`. When two
  copies finish together, the order in which they reach the worker decides. The rename refuses to replace an existing file
  (`renamex_np(RENAME_EXCL)` on macOS). `std::fs::rename` silently replaces, so it is never used for camera files. On `EEXIST` the
  existing file's SHA-256 is compared: equal → the new copy is a duplicate; different → the next `__dupN`. `.duplicates/` follows the
  same rule.
- **Target folder and attribution (ratify R-I28).** A non-duplicate goes to `<capture root>/<stored folder_key of the session's item>/`,
  or `_Unassigned` when the session is null or no longer exists. The folder key is read **at the moment of the rename**, and the folder is
  created if missing. `items.folder_key` is never empty (05B MC-22 computes it at item creation), but it follows the SKU until frozen. A
  committed delivery **freezes it like a first photo row** (05B MC-22 amended), so a SKU edit between the rename and the photo row cannot
  move the folder. 05A MC-2 gains rule 0: a file whose canonical path is a delivery's `final_path` is attributed to that delivery's
  session, and its `captured_at` is the matched exposure's `exposed_at`, else `completed_at` (05A MC-4 amended: a received file's mtime
  is its arrival time). So the ledger's decision reaches `photos` even when a folder name matches no single session. With the direct
  adapter, `ArrivalService` takes the capture root from the same formula as the receiver (MC-32 additions), so `final_path` and MC-2
  compare like with like.
- **Watchdog, decidable (ratify R-I29).** `pending_count` = exposures with status `expecting` and `exposed_at` older than 60 s but
  newer than 30 min. The notice `FilesNotReceived { count }` shows iff `pending_count > 0` and no upload or download is in progress. It
  clears when the count reaches 0 or the photographer dismisses it (those exposures become `dismissed`). When it is first raised, the
  adapter sends `ftp-status` and the health screen shows the result. The "reconnect" button is offered only while the SDK session is on
  Wi-Fi, because a reconnect on USB gives the camera to Capture One (V6). Gaps in a camera file-numbering prefix are logged
  (`camera.sequence_gap`) and counted on the same health item; they never block anything.

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

**Inventory additions (2026-11-06).**
- **Naming the holder (ratify R-I30).** `holder` is the process name in the interface's `UsbExclusiveOwner`. If that is `ptpcamerad`,
  the programs named are the running ones from a fixed list (Capture One, Imaging Edge Desktop, Image Capture, Photos, Adobe Lightroom
  Classic, Adobe Lightroom), with Capture One first when it runs (V6: it is the trigger), plus any other user process that has
  ImageCaptureCore loaded, when that can be read (best effort, spike S5). macOS does not say which program asked `ptpcamerad` for the
  camera, so the sentence reads "programs that can take the camera: …". (The lab's `competing_processes` is a `pgrep` of fixed names.)
- **Start order (ratify R-I31).** At app start, with Sony support installed, the sidecar is spawned before 10C's ensure may launch
  Capture One. Ensure step 1 waits for the first of: `connection.ready` on USB; `usb.initial { present: false }`; the three-failure
  fallback; the sidecar being unavailable or crashing; 15 s. If Capture One is already running at app start, nothing is waited for, and a
  claim is reported as above. A consequence of V6, told to the photographer in setup: a camera switched on, woken or plugged in **after**
  Capture One started is taken by the system camera service and needs "Take the camera back".
- **One gate for every Capture One launch (ratify R-I32).** Every path that can start Capture One goes through one editor-launch gate.
  The paths are `EditingProvider::ensure` (called at startup, by every health check, by Retry checks, after a settings save and by
  session activation; the Health page and Settings run a health check when they load) and every script that does not check `running()`
  first (`front_document`, `make_document`, `open_document`, `ensure_recipe`, `ensure_capture_style`), because addressing Capture One by
  script launches it. While "Take the camera back" holds the gate, these return "deferred" without running a script.
- **"Take the camera back", step by step (ratify R-I32).** The button is enabled only when no upload or download is in progress, no
  exposure younger than 10 s is still expecting a file, no export or publish run is active and no render is in flight; otherwise it is
  disabled and says why. Each step is bounded. A failure stops the sequence, releases the gate and says which step failed.
  1. Hold the gate.
  2. `save` the document. On failure, stop with Capture One untouched.
  3. Ask Capture One to quit (AppleScript `quit`) and wait ≤ 20 s until it is not running. If it is still running (a dialog is open),
     stop; it is never killed.
  4. Wait ≤ 5 s for the interface to be free (`usb.claimed_by` cleared), send `usb-retry`, and wait ≤ 15 s for `connection.ready` on
     USB.
  5. Whatever step 4 gave, launch Capture One, run 10C's ensure (document open in its own window, in front when a session is active),
     then run MC-18 for the active session.
  6. Release the gate.

  If Capture One starts during steps 3–4 and the app did not start it (the photographer opened it), stop and report. Total ≈ 15–20 s.
  Wi-Fi delivery continues throughout; the ledger and the receiver never depend on Capture One.
- **Session switching while Capture One is away (ratify R-I33).** With the direct adapter, activating a session stores the active
  session first (the ledger's target) and then applies MC-18 to Capture One. An MC-18 failure becomes a health item ("Capture One will not
  show this session's new photos until it is reachable"), not a capture block, and is retried at the next ensure. Today activation stores
  the session only after Capture One accepts the folder.

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

**Inventory additions (2026-11-06).**
- **Current collection on arrival: owner card CI1, answered "always jump" (owner 2026-11-06, R46).** The switch is made on every
  arrival while Capture One shows another collection, whether or not Capture One is the frontmost app, and on every session activation.
- **Waiting for the variant (ratify R-I35).** After an arrival, the adjusted-preview render and the style apply wait until Capture One
  finds the variant: `find_image` is polled every 1 s for up to 15 s, then the normal render backoff applies (1, 2, 4 … 60 s) for as long
  as the photo exists and is stale. `variant_not_found` never ends the retries and never marks a generation failed; a current-collection
  switch triggers an immediate retry. `find_image` searches only the current collection (evidence), which is why the collection rule
  matters. The style is applied at most once per RAW (recorded in SQLite), and never again after a restart or on a retouched variant.

## Out of scope (later chapters)
Live view (its own frame channel); camera settings beyond the store destination (capture intents and presets); bracketing, Pixel Shift
and capture groups (one result from many files: the owner's research intent, tracks C–G); FTPS; the ZV-E10 on the direct adapter
(untested); a second camera.

## Owed before planning
- ~~V6~~ done 2026-11-06 (evidence §V6): Capture One is the trigger; its tether checkboxes do not help; SDK-first holds; release loses
  the camera within 1 s. ~~Store-destination write over USB~~ verified both ways (evidence §V6). Cable re-plug with Capture One running: lost,
  `ptpcamerad` holds the interface within 28 ms of the plug-in (evidence §V6); covered by "Take the camera back".
- **Camera support package spike:** install the SDK runtime through the flow above on a second macOS user account, confirm the sidecar
  loads it and connects on USB and Wi-Fi from the packaged app. Its questions are S1–S6 in §Camera support packages (inventory).

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
| No FTP re-send of a completed file (inventory) | over a 20-shot Wi-Fi session, FTP deliveries = exposures; no `_1` names, nothing in `.duplicates` |
| Settings saved while on USB with Capture One running (inventory) | USB session unbroken; the receiver still accepts uploads; one sidecar process |
| Take the camera back (inventory) | after a cable re-plug with Capture One running: one click, USB session within ~20 s, Capture One back with its document and the session folder; opening Health meanwhile does not relaunch Capture One early; no photo lost or doubled |

## Invariants (registered in chapter 14)
Wording made decidable by the inventory **(ratify R-I38)**; chapter 14 holds the same text.
- **M37 (MC-31, MC-32).** Every completed camera transfer (completion as MC-31 and MC-32 define it) yields exactly one file,
  byte-identical (SHA-256) to what was sent: under a candidate name if it is the first copy of its (file key, SHA-256), otherwise in
  `.duplicates/`. An aborted transfer yields none under a candidate name. No file under a candidate name or in `.duplicates/` is ever
  overwritten or deleted.
- **M38 (MC-33, MC-2).** Every distinct camera file (file key + SHA-256) becomes exactly one `photos` row, in the session that MC-33's
  association rule gives (the session active at the matched exposure, else at completion), whichever path delivered it first. An identical
  later delivery becomes none; a same-name file with other bytes becomes its own row.
- **M39 (MC-31, MC-34).** At most one SDK session, one sidecar process and one FTP listener per app process at any time, settings saves
  included. No transport switch starts while the camera is busy (MC-31's busy reasons). A USB session closes only for MC-31's listed
  reasons. Every store-destination write is verified, and none is *Host PC* alone. A secret never appears in a log, argv, a settings file or
  the repo. No other process is ever killed. Capture One is quit only on the photographer's click.
- **M40 (MC-34, 10C MC-26).** While "Take the camera back" runs, nothing but its own step 5 launches Capture One. At app start with the
  camera on the cable, Capture One is launched only after one of MC-34's start-order conditions holds.

## Chapters this amended (2026-11-06)
10 (research item resolved; Session Manager controls the camera; the Capture One camera adapter retired after acceptance), 03 (camera:
A7R V), 05A MC-2 (`.incoming` and `.duplicates` ignored; arrival via MC-33's target folder), 10A MC-17 (current collection follows the
session), 10C MC-26 (camera connected before launching Capture One; style applied by script to arrivals), 11 (camera sidecar, camera
support packages, C++ via CMake, no vendor SDK in the repo), 12 (Keychain for camera and FTP secrets; health items: camera holder,
fingerprint, refused password, backlog, store destination), 13 (direct adapter moves from deferred to must-ship; FTPS, live view,
capture groups stay deferred), 14 (M37–M39), 15 (R42–), 16 (changelog).
Mechanism inventory (2026-11-06, changelog 32): 05A MC-2 (rule 0) and MC-4 (captured_at of received files), 05B MC-22 (freeze at a
delivery), 10A MC-18 (step 5 with the direct adapter), 10C MC-26 (launch gate, start order), 11 (camera runtime outside the settings
rebuild; `NSLocalNetworkUsageDescription`), 12 (Keychain item names, health items), 14 (M37–M39 reworded, M40), 15 (card CI1), 16.

## Owner answers (2026-11-06)
D1 USB has priority, Wi-Fi is the fallback, and Session Manager must keep control of the camera against other programs → MC-34.
D2 Session Manager controls the camera (live view and settings later) → Capture One tether retired. D3 a place compatible with the app
→ the session capture folder, with V1/V2's two rules. D4 on USB files come over the cable, the duplicate rule handles the FTP copy →
Save-to-PC on USB. D5 plain FTP now, FTPS later. D6 Keychain now. D7 the app owns its camera layer and is portable; users add a make's
SDK through the app → camera support packages. D8 (orchestrator's call): the lab's hand-written SDK stand-in headers are not copied into
this public repo; CI builds the stub sidecar.
