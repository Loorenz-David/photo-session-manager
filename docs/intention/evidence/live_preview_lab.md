# Evidence — Live preview delivery and the phone app (chapter 10J, spikes LV-S3 / LV-S4)

Source: the owner's camera lab `~/Desktop/Developer/Camera-Connection-Tests`, Experiment 08 (`experiments/08-phone-app.md`,
server `camlab/pwa`), run `results/20261125T114537Z_08-phone-app` (local to the lab Mac). Owner-attended, 2026-11-25.
Hardware: Sony a7R V on USB-C (live view 25 fps, 1024×680 JPEG), Apple Silicon Mac, owner's iPhone on iOS 18.7, one
studio Wi-Fi. The camera-side facts (frame rate, latency to the Mac, FTP interplay, battery) are in
`sony_camera_lab.md` §Live view. This file records facts; what the product does with them is chapter 10J. [TEST-HW]

## Answers

| Question | Answer |
|---|---|
| Does an installed web app (Add to Home Screen) open full screen on plain HTTP? | **iPhone: yes**, on plain HTTP and on HTTPS. Android: no, see §Android. |
| Can it keep the phone's screen on? | **Only on HTTPS.** The Screen Wake Lock API is missing on plain HTTP; with Auto-Lock 30 s the phone dimmed at ~50 s, locked at ~61 s and the stream closed. On HTTPS, after one tap the lock was held and the screen stayed on, streaming, for > 3 min untouched. |
| Can it open with no connection (service worker)? | Only on HTTPS (registered); not available on plain HTTP. Offline start itself not yet tried. |
| What does the phone need for HTTPS on the studio network? | A certificate for the Mac's `.local` name and address, signed by an authority the phone trusts. In the test: a lab CA **name-constrained** to `.local` names and private address ranges, installed once as a profile and trusted in Settings → General → About → Certificate Trust Settings. No warning afterwards. |
| Live frames on the phone | WebSocket with binary JPEG, newest frame only: **~25 fps** on the phone (all the camera gives over USB), Mac → phone screen **~15–30 ms** typical, ≤ ~42 ms slow; round trip 8–43 ms. Same on `ws://` and `wss://`. **Glass → phone ≈ 145 ms** (USB). |
| Mixed content | A secure page cannot open a plain `ws://` stream (blocked), so a secure app needs `wss://` from the same server. |
| Mac cost | Pulling 25 fps ≈ 5–6.5 % of one core (lab probe, including wasted repeat pulls); serving one phone ≈ 5–7 %; memory 23–36 MB. |

## Android (2026-11-25, owner's Android phone, Chrome, plain HTTP; run `results/20261125T121749Z_08-android`)
| Question | Answer |
|---|---|
| Install as an app over HTTP? | **No.** Chrome: "This app cannot be installed"; only "Create shortcut — Shortcuts open in Chrome" (a Chrome tab with the address bar). Chrome installs only secure sites. |
| Full screen over HTTP? | **Yes, by a button:** the Fullscreen API works on HTTP; one tap hides all browser bars. Needs the tap each time the app opens; the back gesture or an app switch leaves it. (iPhone has no Fullscreen API, but installs full screen on HTTP.) |
| Live frames | 24–25 fps; Mac → phone 21–35 ms typical; `.local` name resolved. |

## Desktop window (Experiment 09, 2026-11-26, run `results/20261126T070627Z_09-desktop-webview-usb`)
Tauri 2.12.5 window configured as Session Manager (`csp: null`, page on `tauri://localhost` = secure context), camera
25 fps on USB, 40 s per path; the window was unfocused and mostly hidden behind other windows.

| Path | Allowed? | fps | Delay p50 / p95 | Dropped |
|---|---|---|---|---|
| WebSocket `ws://127.0.0.1` | yes | 5–17 | 18 / ~112 ms | 293–621 |
| WebSocket `ws://<host>.local` | **yes** (not blocked, unlike the secure iPhone page) | 17.6 | 17 / ~110 ms | 271 |
| **Tauri channel, raw bytes** | yes | **25.0** | **16 / 22 ms** | **0** |

Decode + draw ≈ 9–10 ms per frame. CPU: app 3.6 %, WebKit helpers ≈ 26 % in total, crlab 7.5 %, lab Python server 10 %.
**Desktop delivery: Tauri channel; phones: WebSocket.**

## Constraints found
- **P1 — Wake lock needs a user gesture** each time the app opens (requested on a button press; the product can tie
  it to the first Live peek or Capture tap).
- **P2 — The address must be stable.** The certificate names the Mac's `.local` name and its current address; the
  Mac's DHCP address has changed before (`sony_camera_lab.md`, final hands-on check). Use the `.local` name.
- **P3 — A trusted root on the phone is powerful.** Name constraints limit it to local names and private addresses;
  it must be removable and short-lived enough to be rotated.

## Not covered yet
The desktop window focused and in front (channel expected no worse); Android over HTTPS; offline start in airplane mode; wake lock after switching apps; several phones at once; Wi-Fi camera (≈ 20 fps) to the
phone; delivery to the desktop webview inside Session Manager; behaviour when the phone sleeps and wakes (reconnect).
