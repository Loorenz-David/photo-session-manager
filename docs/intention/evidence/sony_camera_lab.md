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
