# Evidence — Sony Camera Lab (direct camera adapter research)

Source: the owner's lab repo `~/Desktop/Developer/Camera-Connection-Tests` at commit `221aa49` (local only, read-only for
this project). Documents read: `docs/session-manager-handoff.md`, `docs/findings.md`, `docs/session-manager-recommendations.md`;
code read: `sdk/` (C++ `crlab`), `camlab/ftp/server.py`, `camlab/ledger.py` and their tests. Finding IDs (H-n, F-n) are the
lab's. Hardware: one Sony A7R V (ILCE-7RM5), one Apple Silicon Mac on macOS 26.5, one Wi-Fi network, Sony Camera Remote SDK
2.02.00. Runs dated 2026-11-05. Run folders (`results/`) are local to the lab Mac and not copied.

This file records facts. What the product does with them is chapter 10E.

## Answers

| Question | Answer | Lab findings |
|---|---|---|
| Can the SDK control the A7R V over Wi-Fi and USB-C? | Yes, both. Wi-Fi: SSH-authenticated connect 0.8–1.0 s. USB: connect 93–134 ms. Command → exposure about 0.5 s on both, with a half-press (S1) and AF wait | H-1, H-2, H-8, H-9, H-14 |
| Does camera Auto FTP deliver RAWs while an SDK session controls the camera? | Yes, on Wi-Fi sessions (6/6) and USB sessions (7/7). About 51 MB lands 4.0–4.4 s after exposure (17–25 MB/s). No effect on shutter latency | H-11, H-13, H-14, H-18 |
| Can Wi-Fi ↔ USB-C switching run with no camera interaction? | Yes: 3 cycles, 20/20 steps. Cable in → USB session 7.3–7.4 s; cable out → Wi-Fi session 5.8 s; a switch is deferred while the camera is marked busy | H-15, H-17, H-18 |
| Recovery | 6/6 run scenarios recovered with no manual step: power cycle (Wi-Fi back 16.3 s after the loss), FTP server down (re-send 0.9 s after restart), upload cut mid-transfer (recorded failed, full file re-sent), USB cycle, sidecar killed (reconnect 979 ms, no `SessionAlreadyOpened`), 30 s playback mode. Wi-Fi outage not run | H-19 |

## Constraints found

- **C1 — Capture One and USB (H-16, H-17).** While Capture One was running, macOS `ptpcamerad` took the camera's USB interface
  exclusively on plug-in (`UsbExclusiveOwner = "pid …, ptpcamerad"`), and every SDK USB connect failed in about 10 ms with
  `CrError_Connect_TimeOut` (0x8208), even with Capture One's tether camera set to "none". With Capture One quit, USB worked
  every time. Capture One links ImageCaptureCore; Session Manager links neither ImageCaptureCore nor IOKit. Single controlled
  comparison, not repeated. **Wi-Fi control and Auto FTP worked with Capture One open (H-13).**
- **C2 — One transport at a time (H-8, H-15).** With the cable in, the camera is listed only on USB and drops its Wi-Fi remote
  session itself; the SDK reports that loss about 17 s later, so the switch is driven by the USB hot-plug event (2.5 s settle).
- **C3 — An SDK acknowledgement is not an exposure, and an exposure is not a file (H-1, H-7).** Only a file arrival proves a photo.
  Shots fired on the camera body are reported too (`association: none`) and arrive by FTP.
- **C4 — Duplicates (H-14).** With [Still Img. Save Dest.] including Host PC, a USB shot arrives twice, byte-identical (SDK
  Save-to-PC, then Auto FTP 1.9 s later). With Camera Only (the proposed setting) no duplicates occurred (H-18).
- **C5 — FTP stall (H-11).** Once, after a camera FTP misconfiguration with failed items queued and 8 % battery, two uploads
  stalled 8.5 min and were released 1.2 s after the SDK disconnected. Clean runs never stalled (H-13, H-18).
- **C6 — Truncated uploads (F-1).** A stock FTP server reported a reset upload as complete. The lab receiver treats a reset or
  aborted data connection as failed; on hardware the camera re-sent the full file under the same name (H-19).
- **C7 — Burst ordering (F-2).** Files are matched to shots by file-stem order, not by "latest capture before upload"; still a
  provisional heuristic because re-sends can be out of order.
- **C8 — Packaging (H-6).** Sony's dylibs are ad-hoc signed and not notarized; Gatekeeper once deleted `libCr_Core.dylib`. The
  runtime needs `libCr_Core`, `libmonitor_protocol*` next to the executable and the `CrAdapter/` folder; SDK re-`Init` in one
  process is reported to crash, so the SDK runs in its own process.
- **C9 — Camera identity differs per transport.** Wi-Fi id is the MAC address, USB id the USB serial.
- **C10 — Settings writes (H-3).** `SetDeviceProperty` success means accepted; the value applies about 210 ms later. Nothing in
  the proposed design writes camera settings.

## Not covered by the lab
Wi-Fi outage; long sessions (hours, hundreds of shots, battery); FTPS with the camera; a second network or Mac; other firmware;
the **ZV-E10** (untested; its SDK support must be checked); bracketing and Pixel Shift progress; direct USB connect by serial
(possible 3.5 s faster switch); live view; **Capture One's handling of files written into its capture folder by another process**
(visibility, Next Capture Adjustments, scripting) — owed by Session Manager, chapter 10E §Verification.

## Session Manager's own checks (2026-11-06, Capture One 16.8.6, dev Mac, document "session test 1")
Probe files were copies of existing ARWs written as `.<name>.part` and then renamed, exactly as the FTP receiver will write.
Nothing was deleted (H2). Capture One's settings were restored afterwards: Next Capture style empty, Hot Folder off, browsed
collection back to the session root.
- **V1 — Visibility.** A file written into the current capture folder (`Capture/test-2-photo/`) by another process was **not seen
  by Capture One in 120 s** while Capture One was browsing another collection; `refresh` and re-setting the capture folder did not
  help. Selecting that folder as Capture One's **current collection** made it scan and add the files at once, and **while that
  folder is the current collection, a new file appears in about 2 s**. `synchronize` refuses session favorites ("Can only
  synchronize collections of kind 'folder'"). `import` with a text path **copied** the file into the session's root folder
  (a second file at another path), so it is unsuitable.
- **V2 — Next Capture Adjustments are not applied** to such files (Next Capture style set to "Bright Contrast": the new variants had
  no styles). The dictionary says the same ("applied when an image is captured from a tethered camera"). **`apply style` by script
  on the variant's layer works** (styles read back `["Bright Contrast"]`).
- **V3 — Session Manager's watcher** created exactly one `photos` row per file, about 2 s after the rename, in the session of the
  folder (photos 64–66, session "test-2"). No capture hook fires (none expected).
- **V4 — Preview render** works once Capture One knows the variant (photos 65 and 66 got adjusted previews). For the first probe,
  render jobs failed with `variant_not_found` while Capture One could not see the file and stopped retrying.
- **V5 — Hot Folder** (Camera menu; not in the dictionary, settable only by GUI scripting): inconclusive, the folder chooser could
  not be driven reliably; not pursued, since it cannot be set up by the app itself (chapter 10C principle).
- **Side finding:** the app's `jxa/import.js` (`source: paths.map(p => Path(p))`) fails in 16.8.6 with "Can't convert types"
  (-1700), so the existing Import feature's Capture One step does not work; and `import` copies files into the session root.
- **USB contenders running now:** Capture One (loads ImageCaptureCore; `ptpcamerad` started 4 s after it, as in H-16), Imaging Edge
  Desktop (loads ImageCaptureCore), a Photos widget. Capture One's Preferences → Capture has **"Enabled Tether Support: USB /
  Wireless"** checkboxes: the candidate configuration for V6.

## V6 — camera ownership with Capture One (2026-11-06, A7R V on USB-C, lab `crlab` built out of tree against SDK 2.02)
- Capture One 16.8.6 running, **its Sony USB and Wireless tether support already unticked** (Preferences → Capture → Enabled
  Tether Support), Imaging Edge Desktop running: `UsbExclusiveOwner = ptpcamerad`; SDK USB connect fails in 6 ms with 0x8208 and
  `usb.claimed_by ptpcamerad`. **The tether checkboxes do not prevent the claim.**
- Imaging Edge Desktop quit: still claimed. Capture One quit (Session Manager quit first, since it relaunches Capture One itself):
  `ptpcamerad` exits and the interface is free immediately. **Capture One is the trigger.**
- **SDK first:** SDK USB session opened (`UsbExclusiveOwner = crlab`), then Capture One launched: `ptpcamerad` started 6 s later and
  **could not take the interface** (still `crlab` 15 s later).
- **Release while Capture One runs:** when the SDK session ended, `ptpcamerad` took the interface within about 1 s and a new SDK
  connect failed (0x8208). Getting the camera back needs Capture One quit (or, untested, a cable re-plug race, which the SDK is
  expected to lose: `crlab auto` attempts its connect about 3.6 s after arrival because of discovery).
- **Store destination over USB:** set *Host PC + Memory Card* (0x3) verified in 208 ms; set *Camera only* (0x2) verified in 205 ms;
  left on Camera only. Camera FTP status read over USB: connection error (no receiver running), as expected.
- **Cable re-plug with Capture One running (owner re-plugged, 11:07 UTC):** cable out → interface free; cable in → IOKit arrival at
  40.862, **`ptpcamerad` held the interface by 40.890 (≤ 28 ms, the poll's resolution)**; the SDK's first attempt came 3.7 s later
  (discovery) and failed 0x8208, as did every retry. The re-plug race is lost by orders of magnitude; even a direct connect by serial
  could not beat it. Recovery while Capture One runs therefore needs Capture One quit and relaunched (10E MC-34).

## Spike S1–S7 (P47, 2026-11-08, owner-attended; A7R V; packaged ad-hoc build of `ec80bdd`; dev account)
Setup (orchestrator prep, 2026-11-07): `Session Manager.app` built with the SDK sidecar. The support folder `camera/sony/2.02.00/` was hand-installed under the app's Application Support (P47 steps 1–2; nothing in the repo). The sidecar copy was signed ad hoc with hardened runtime plus `disable-library-validation`, and the dylibs re-signed ad hoc. The three Keychain items were added by the owner with `security add-generic-password` (values never on a command line). The app ran against the owner's real data folder, document "session test 1".

| Q | Result |
|---|---|
| **S1(a) cable** | **Pass on the third launch.** Timeline: the sidecar spawned at 10.2 s while a Keychain prompt was open. It held the USB interface (`UsbExclusiveOwner = camera-sidecar`) by 16.5 s, and Capture One launched at about 16.5 s. `ptpcamerad` started at 20.5 s and **could not take the interface**, which matches V6 "SDK first". The header showed Connected (owner), and two cable shots arrived as photos. **Launches 1–2 lost the race:** the Keychain prompts held the sidecar spawn to 24 s and 15.4 s, past the 15 s start-order wait. Capture One was launched, `ptpcamerad` claimed the interface 4 s later, and the sidecar never got the cable. A re-plug did not help (V6). The `connection.ready` line and `store_destination` could not be read: the app does not log sidecar stdout, and the helper log is P51's. |
| **S1(b) Wi-Fi** | **Pass after the fingerprint was confirmed.** With the cable out and no Keychain prompt, the sidecar spawned at 1.3 s and the SSH session to the camera was established at 5.4 s after launch. The first Wi-Fi attempts failed for two reasons: the Mac was on another network, and the camera's Remote Shooting was off. The owner fixed both on the camera. The lab's pinned fingerprint (2026-11-05) **no longer matched**: the camera now presents another key. The sidecar refused with `MISMATCH`, did not retry, and the app showed "The camera's fingerprint changed: confirm it in Settings → Camera before shooting over Wi-Fi". The orchestrator removed the stale copy. The new fingerprint (sidecar stderr) was compared by the owner with Access Authen. Info and matched, and was then pinned by writing the file (stand-in for `--accept-fingerprint` / P50's confirm). The cable-out → Wi-Fi switch time was not measured separately. |
| S1(c) second account | **Not run** (no second macOS account at hand): moved to P52 on the work Mac, which is a fresh account and machine. |
| **S2 library validation** | Hardened runtime with ad hoc signing **refuses** Sony's dylibs: dyld reports "mapping process and mapped file (non-platform) have different Team IDs", exit 134. **`com.apple.security.cs.disable-library-validation` on the sidecar fixes it.** With it, the runtime and `CrAdapter/` loaded and connected on both USB and Wi-Fi. Ad hoc without hardened runtime also loads. |
| S3 Gatekeeper | Dev account: the copies carry only `com.apple.provenance`, no `com.apple.quarantine`. `open` launched the `.app` without a prompt, and no dylib was removed over six launches. `spctl -a` rejects the ad hoc sidecar and dylibs, as expected. A first launch on another account is moved to P52. |
| **S4 Local Network** | Session Manager is listed and on under Privacy & Security → Local Network (owner). When the prompt appeared was not recorded. Wi-Fi discovery, the SSH connect and the inbound FTP upload all work from the packaged app. The macOS firewall is off (`socketfilterfw` state 0), so there was no firewall prompt. |
| **S5 ImageCaptureCore readers** | `lsof +c 0` is readable without privileges and took 0.85 s. **With Capture One running, only `mscamerad-xpc` was listed; Capture One was not**, so the scan cannot name Capture One behind `ptpcamerad`. The fixed program list stays the source for naming the holder. |
| **S6 after an update** | The update was simulated by changing the bundle version and re-signing ad hoc (new cdhash, same identifier). **The Keychain asked again for every item: 3–4 windows (owner count), each needing the Mac password.** Everything camera-related waited for them: receiver at 74.6 s, sidecar at 80.8 s, Wi-Fi connected at 84.6 s. There was no firewall prompt (firewall off). |
| **S7 FTP commands** | Partial capture (control channel via `tcpdump`; PASS hidden at capture): `PASV` → `227 Entering Passive Mode (…)` → `STOR DSC04770.ARW` → `150 Opening data connection`. **No `502`.** The login had happened before capture: the camera logs in when its FTP function is switched on, and the connection stays open. The final `226` was not captured (stopped early). Shutter → file on the Mac took about 8 s and → photo in the app about 10 s (owner). Two earlier shots, queued while the receiver had no password, uploaded on their own once it started. For the FTP user, **the owner configured camera Server 2** (`sessionmanager`, Mac LAN IP, port 2121, Does Not Overwrite) and selected it; the lab's Server 1 is unchanged. |
| S7(b) credentials line | Not observable (sidecar stdout is not logged; P51's helper log will show it). No secret appeared in the app log. |

**Findings for P50 (and P48/P51):**
1. **The Keychain must not block the camera start.** `CameraRuntime::bind` reads the FTP password (`bind_receiver`) and the Access Authentication credentials synchronously before it spawns the sidecar. A Keychain prompt holds the cable connection past the 15 s start-order wait, and Capture One then takes the cable for the whole run. This happened twice. Prompts come on first use of items made outside the app, and **after every update** of an ad-hoc-signed build (S6). P50 should spawn the sidecar at once on USB, deliver the credentials when they are read, and start the receiver independently.
2. Every update re-asks for all three items, because the ad hoc identity changes. P50 decides between accepting this (and telling the owner) and another ACL strategy.
3. Sign the sidecar with `disable-library-validation` (S2), and drop the build machine's absolute `LC_RPATH` from the SDK build (prep).
4. Each launch leaves an empty `Capture/.incoming/sdk-<run>` staging folder in the session document.
5. A stale pinned fingerprint is a real case (the camera's key changed within three days). P50/P51's confirm flow is needed, and the mismatch handling (refuse, no retry, clear sentence) behaved correctly.
6. Logging the sidecar's `connection.ready` (transport, store destination) at INFO would let P52 read results without the UI.

## Acceptance (P52, 2026-11-08, dev Mac, partial; owner-attended)
Setup: packaged build of `3dfd581` with the SDK sidecar, installed in `/Applications`; the P47 hand-installed support folder moved aside first. Support installed **through
Settings → Camera** (see A0), FTP password generated by the app and typed on the camera by the owner (Server 2, user `sessionmanager`, port 2121), Access
Authentication login entered, fingerprint compared and confirmed in the app (P51b: exact value pinned). Capture One 16.8.6 running on document "session test 1",
app item `test-2`. A7R V. Timings come from an orchestrator stopwatch (`ps`/`lsof`/`ioreg` every 0.2 s) and file birth times, because the app logs neither the
helper's events nor the receiver's uploads (F2). The owner stopped the run after A15 to have the findings fixed first; the rest runs after the fixes, then on the work Mac.

| Row | Measured | Result |
|---|---|---|
| A0 install | Sony's download (`CrSDK_v2.02.00_…_Mac.zip`) is **refused**: the seven runtime files sit in a nested `RemoteCli.zip` (`external/crsdk/`), and the installer reads only top-level entries; the toast says the file is "missing **or changed**". Selecting the extracted `RemoteCli.zip` installs (sidecar signed with `disable-library-validation`, no quarantine). | **F1** |
| A1 Wi-Fi connect | SSH session up at +8.4 s from launch, but the dot read `Camera disconnected` for about 2–3 min, never `Connecting`; Retry had no visible effect; then `Connected`. | **fail, F3** (cause unknown: F2) |
| A2 USB at start | not run | — |
| A3 capture from the app | there is no capture button in the app | **not runnable, F4** |
| A4 RAW over USB | file ~1 s after exposure; one photo in `test-2`; the FTP copy in `.duplicates/`, byte-identical; ARW header | pass |
| A5 Auto FTP, Wi-Fi | 3 shots on the camera's shutter: 4 / 5 / 6 s exposure → file complete; one photo each. (Auto FTP Trnsfr was **off** on the camera at first; the setup steps do not mention it.) | pass |
| A6 cable in, Wi-Fi session, Capture One running | `ptpcamerad` took the cable (V6); the header named Capture One with "Take the camera back" (correct). **Wi-Fi did not resume for ~5 min** while the cable was held elsewhere. | detection pass; **F6** |
| A7 cable out | SSH up ~5 s after the unplug; dot changed quickly | pass |
| A8 plug-in during capture | not run | — |
| A9 one helper | one `camera-sidecar` pid over each stretch (until A12's kill) | pass so far |
| A10 receiver restarted (port change) | one upload failed while the port was wrong. **The camera's FTP went into an error state and stopped all automatic transfers**, new shots included, until the owner switched FTP Function off and on. The two shots taken meanwhile stay on the card; the next shot arrived <1 s after the toggle. The app logged `camera.sequence_gap … missing=2` at INFO every second and showed the photographer nothing. 10E's expectation ("the camera re-sends the pending shot within seconds") **does not hold on this camera.** | **fail, F8** |
| A11 upload cut | not run (after A10 it would freeze the camera's FTP the same way) | — |
| A12 helper killed (our own helper, the only kill) | new helper +2 s, SSH up +5 s (target ~1 s); dot recovered | pass with note (F5) |
| A13 Imaging Edge | not run | — |
| A14 Capture One shows new photos | the new photo appeared in Capture One's session folder (owner) | pass |
| A15 20-shot Wi-Fi session | 18 exposures (contiguous numbers, no gap), all arrived, ~8 s apart; no `_1` names. **16 of the 18 arrived a second time** ~15 s later, byte-identical, and went to `.duplicates/`. The receiver replies `226` only after hashing the 45 MB file and handing it to the ledger (`crates/infra/src/ftp/session.rs`, `upload` → `sink.delivery` → `226`); a camera timeout on that reply is the likely cause (not proven: F2). | **fail, F9** (no photo lost or doubled) |
| A16 settings save on USB | not run: the owner could not get the cable session (below) | — |
| A17 Take the camera back | the click is refused forever with "a preview is rendering": renders deferred on `variant_not_found` stay counted as in flight | **not runnable, F7** |
| A18 store destination | USB: `Host PC + Memory Card`; Wi-Fi: `Memory Card` (owner read the menu). Quit-on-USB not run. | pass so far |

Other observations:
- **Capture One relaunch fights the cable.** When the owner quits Capture One, the app relaunches it within ~0.5 s; `ptpcamerad` takes the cable again ~6 s later,
  so our helper never gets it (stopwatch 14:58: three quit/relaunch cycles, `ptpcamerad` each time). Once, `ptpcamerad` released on its own and the helper took
  the cable 2.5 s later. With Capture One's camera support switched off by the owner, `ptpcamerad` still took the cable while Capture One ran. (F10)
- Wi-Fi uploads continue whoever holds the cable: the camera's FTP is independent of the helper's session.
- Reframing a photo while others were arriving waited on "waiting for Capture One to render preview" (F11, performance; owner asks for a faster path).
- The app log is flooded by `osascript` provider-call lines (`editor_info`, `find_image` retried every ~1 s), ~66 000 lines in a day.

**Findings:** F1 install from Sony's download + wording · F2 no helper/receiver log · F3 no `Connecting`, slow Wi-Fi status · F4 no capture button ·
F5 restart ~5 s vs ~1 s · F6 no Wi-Fi while another program holds the cable · F7 take-back blocked by deferred renders · F8 camera FTP stops after a receiver
outage, undetected · F9 duplicate uploads (late `226`) · F10 Capture One relaunch hands the cable to `ptpcamerad` · F11 preview wait blocks reframing.
