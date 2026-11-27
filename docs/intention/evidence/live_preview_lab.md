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

## Wi-Fi, phone tap-to-focus, sharp photo, several viewers (Experiment 11, 2026-11-26, run `results/20261126T071828Z_11-wifi-phones`)
| Question | Answer |
|---|---|
| Tap-to-focus from a phone over Wi-Fi | **10/10 focused**, tap → focused 0.38–0.83 s (Android phone; same as USB). |
| Photo after a tap sharp where tapped? | **Yes**, both photos: full-size crops show the tapped object sharp and the other blurred (tape measure in front / table at the back). Button → exposed 0.60–0.77 s. |
| Several viewers at once | Mac + Android: each **19.4 fps** (all frames the camera gave over Wi-Fi), 0 skipped, no viewer slowed the other; Mac 6 ms, Android 31 ms behind. Dips during the photo downloads over the same Wi-Fi. |
| iPhone | Could not reach the Mac that session (no request arrived, by name or address); it worked through the same server and page the day before. Three viewers at once not shown; the owner chose to skip. |

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

## Hardware check (LV4, 2026-11-26/27, Session Manager build of `feature/live-preview`, owner-attended)
Packaged app (SDK 2.02 helper), a7R V (Manual, AF-S, Flexible Spot S), studio Wi-Fi, owner's iPhone and Android. First
run on the LV1–LV3 build (2026-11-26); fixes LV5/LV6 built from what it found; the rest on the fixed build
(`3c403e05`, 2026-11-27). Values from the app log (`live.*`, `phone.*`, `focus.*`, `current_shot.*`) and photos of the
lab clock (`experiments/07-clock.html`) taken with a second phone.

| Row | Check | Measured | Result |
|---|---|---|---|
| C1 | Live on the iPhone over **USB** | 23–25 fps to the phone (`sent_10s` 216–250); glass → phone **107–150 ms** (4 photos, two during captures) | PASS |
| C2 | Live on the phones over **Wi-Fi**, RAW upload during it | source 17–21 fps; ~15–18 fps while a 42 MB RAW uploads (2.4 s), no stall text; glass → Android **150–190 ms** (3 photos) | PASS |
| C3 | No stale frame on a peek after 20 s on the template | first picture current (owner) | PASS |
| C4 | Phones on the template view receive nothing | `sent_10s=0` for both, many lines | PASS |
| C5 | Several viewers; edge of Wi-Fi | iPhone + Android locked together ~20 fps each, skipped 0–9/10 s; far end lagged, Wi-Fi off froze, back on resumed live by itself (fixed build) | PASS |
| C6 | Reconnect after a 1-min phone lock | back within seconds, no reload (iPhone; Android shows the template until the first fresh frame) | PASS |
| C7 | iPhone reach and install | first QR (`.local`) opened; Add to Home Screen → full screen, paired | PASS |
| C8 | Android Full screen | one tap hid the bars (button shown once a session showed) | PASS |
| C9 | Capture from the phone | Wi-Fi request → exposed 0.50–0.73 s; USB 0.44–0.47 s; one photo each; on USB the Auto FTP copy of the same file was filed `duplicate` | PASS |
| C10 | Mixed buttons, late files | camera button, iPhone, Android: each bound to the shot current at its shutter, shooting order 7→10→8→9→Condition; **first build: the next shot could briefly aim at a shot whose file was on the way (F4), fixed in LV5** | PASS after LV5 |
| C11 | Tap a shot, shoot with the camera button | landed in the tapped shot; Mac "Next" followed | PASS |
| C12 | Retake, undo, retake | red at once on the Mac, Cmd+Z restores, new photo fills, advance | PASS |
| C13 | After the last shot | Condition, appended | PASS |
| C14 | Remove a phone on live | `reason=revoked`, removed text, other phone unaffected; installed app must be reinstalled (owner accepts, LV-D19 (2)) | PASS |
| C15 | Tap-to-focus over Wi-Fi | **20/20 focused**, tap → focused 0.39–0.97 s (median ≈ 0.63 s) | PASS |
| C16 | Sharp where tapped | owner at 100 % | PASS |
| C17 | × recentres | `focus x=320 y=240 af=false`, point set ≈ 0.21 s; camera square at the centre | PASS |
| C18 | Focus Area Wide | first build: no message (F5); fixed build: "Set the camera's Focus Area to a movable spot…", camera unchanged | PASS after LV5 |
| C19 | Capture settings unchanged | owner: file format, exposure mode, Focus Area as always; helper wrote only `store_destination` and the focus point | PASS |
| C20 | Clean stop, re-plug | quit on live → reopen: one `connect failed`, connected 7 s later by itself; unplug → Wi-Fi live in 10 s; re-plug → USB after ~40 s with a click on Capture One's window (known hand-back limitation) | PASS |

**M-LV6 targets (owner, 2026-11-27):** USB ≥ 23 fps and glass → phone ≤ 0.20 s; Wi-Fi ≥ 15 fps (short dips while a photo
uploads allowed) and ≤ 0.25 s.
**Fixed after the first run (LV5, LV6):** F1 exposure readout format, F2 Wi-Fi stall closed the socket, F3 Mac actions
not reaching phones, F4 next shot aimed at a shot with a file on the way, F5 Focus Area Wide not reported, F7 tile
pictures stuck broken; LV-D19 dock and lock swipe.
