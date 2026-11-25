# 10J — Live preview (human guidance now, machine guidance later)

**Status: COLLABORATING (shaper draft 2026-11-25, rebased 2026-11-26; not ratified, not planned; research phase).**
Source: the owner's chat of 2026-11-25/26 (answers folded below). First written as "10H" on a stale checkout;
renamed when the checkout was brought up to date (10G and 10H are Shooting templates and Archived items).
Changelog entries D3–D4. Builds on **10G Shooting templates** (containers, shooting order, Next, retake) and
**10E Direct camera adapter** (sidecar, transports, capture). Evidence owed: spikes LV-S1…LV-S4 (§Owed before
ratification), recorded in `evidence/sony_camera_lab.md` (live view section, to be added) and a new
`evidence/live_preview_lab.md`. The phone and desktop look come from the owner's Claude Design work.

## Why
The photographer handles the piece physically and must match each shot to the shooting template's framing. Live
preview shows what the camera sees, on the Mac and on the phone in their hand, next to the template shot they are
matching, and lets them take the picture from either.

Two purposes:
- **Human guidance (built first):** see live, compare with the template shot, take the picture.
- **Machine guidance (later, accounted for now):** an agent or logic reads live frames and acts (capture now; crane
  controllers later). Nothing of it is built in this chapter; the architecture must not need a rewrite for it.

## Owner decisions recorded (2026-11-25/26)
| # | Decision | Amends |
|---|---|---|
| LV-D1 | **Live view is in scope now.** | 10E §Out of scope, 13 (live view deferred), the camera hard rule "live view is out of scope" |
| LV-D2 | **The app may change camera settings that live view requires.** Network and FTP settings stay untouched (10E MC-31); the exact list comes from spike LV-S1. | camera hard rule "the camera's own settings are never changed" (narrowed) |
| LV-D3 | **One frame source in Rust, any number of viewers** (shaper's proposal, accepted; the owner notes Sony's own tools behave this way). | — |
| LV-D4 | **Live view over USB and over Wi-Fi**, like photo delivery: whichever transport the camera session is on (10E MC-31). | — |
| LV-D5 | **Live view sleeps when nobody is looking**, to save the camera's battery, the Mac and the network, if the camera allows it (spike LV-S2). Frame rate and quality are tuned by measurement, not fixed here. | — |
| LV-D6 | **Overlay style is a setting**: how the template shot appears together with live (flip, ghost with opacity, outline). | — |
| LV-D7 | **The phone app is installable (PWA)**: full screen, home-screen icon, feels like an app; served by the app's own server (chapter 11 §Phone remote). | 11 §Phone remote (adds install), 13 (phone remote moves up) |
| LV-D8 | **The template is 10G's shooting template** (per item type; containers in published order plus a shooting order). Placing photos by shooting order is what guarantees each photo lands in the right container despite transfer delays; the photographer can always drag a photo to another container. Templates are made in Settings → Shooting templates (10G §3). Answers cards L1, L7, L9. | — |
| LV-D9 | **Taking a picture moves to the next shot** (10G's Next). The photographer can go back to retake a shot and can view what was taken for it. Answers card L2. | — |
| LV-D10 | **A retake replaces:** the new photo takes the container, the old one is denied (undoable, as 10G §6 rule 3 already does for a dropped photo). Answers card L8. | — |
| LV-D11 | **Camera settings changed for live view are not restored**, on the condition that they do not slow the workflow and do not change how photos are captured. Live shooting becomes the normal way of shooting. Answers card L4. | — |
| LV-D12 | **Any paired phone may watch and shoot, several at once.** Answers card L5. | — |
| LV-D13 | **Image analysis for machine guidance is a recorded future exception to H3**, decided with the machine-guidance chapter; human guidance needs none. Answers card L6. | — |

## The working loop (owner's description, the acceptance story)
1. The phone (or the Mac) shows the **template shot for Next** (10G §5), frozen, full opacity.
2. The photographer places the piece, then **looks at live** (peek), sees the difference.
3. They put the phone down and **adjust the piece**; nobody is looking, so live view sleeps.
4. They peek again; when it matches, they press **Capture** (on the phone or the Mac).
5. The photo goes to that container by 10G's placement; **Next moves on** and its template shot appears; repeat.

Physical correction takes far longer than looking, so live view is awake only for short glances.

## What the photographer gets
- **Two views, one switch:** *Template* (the example of the Next container, with its presets' crop, 10G §3) and
  *Live*. Hold to peek at live, release to go back to the template; **Lock** keeps live on, unlock returns to the
  template.
- **Overlay modes** (LV-D6), chosen in Settings: plain flip; ghost (template over live, opacity setting); outline
  (template edges over live). Drawn by the UI only, never written to any file.
- **The live frame shows the area the container will keep** (its presets: shape, split, zoom; 10D values), so the
  match is the published framing.
- **Shot strip:** the containers in shooting order, each with what was taken for it (the photo's preview) or empty;
  Next marked. Tapping a filled container shows the taken photo beside its template shot.
- **Retake** (LV-D10): pick a filled container, press Retake, capture; the new photo takes that container, the old
  one is denied. Next then returns to where it was (MC-LV3).
- **Capture** on both views, phone and Mac, through the existing capture command (10E). The photo arrives exactly
  as a shutter press on the camera would (MC-32/MC-33 unchanged) and is placed by 10G.
- **Status:** "Live", "Waking camera…", "Live paused", "Camera busy", "Live unavailable on this camera"; the camera's
  own status (10E) is unchanged.
- An item type **without a template** gets live view and Capture only, no template view.
- The Mac shows the same views on the stage; the phone's PWA shows them full screen with a large Capture button.

## Architecture
```
camera ──USB or Wi-Fi──▶ camera sidecar (owns the SDK session; pulls live frames only while wanted)
                                │ frame channel (binary, separate from the JSON control line)
                                ▼
                      live view hub (Rust): newest frame only + viewer demand
                         ├──▶ Session Manager window (desktop)    ┐ same delivery path: the embedded
                         ├──▶ phone PWA(s) on the studio Wi-Fi    ┘ HTTP + WebSocket server (R26)
                         └──▶ (later) machine-guidance subscribers, in Rust
```
- **Sidecar.** It already owns the camera; it also pulls live frames. The installed Sony SDK (v2.02) is a **pull
  API**: `GetLiveViewImageInfo` then `GetLiveViewImage` return one JPEG per call, `CrWarning_Frame_NotUpdated` when no
  new frame exists; properties `LiveViewStatus` (NotSupport / Disable / Enable), `LiveView_Image_Quality`
  (Low / High), `LiveViewProtocol` (Main / Alt) (headers in the lab's SDK copy; behaviour on the a7R V to verify in
  LV-S1). Frames go over their **own channel** (10E reserved "its own frame channel"), never as JSON on the control
  line. "Sleep" means the sidecar stops pulling; whether the camera then saves power is LV-S2.
- **Hub.** Keeps exactly the newest frame; every viewer gets the latest; a slow viewer skips frames, never queues.
  It turns viewer demand into sidecar start/stop with a short grace period (MC-LV2).
- **Delivery.** One path for desktop and phone: the embedded HTTP + WebSocket server pushes frames as binary JPEG
  messages, drawn on a canvas. No re-encoding. Video encoding (WebRTC/H.264) only if measured latency or Wi-Fi load
  demands it (LV-S3).
- **Core vocabulary (H7):** live view is a **camera capability** declared by the adapter (`live_view`); core knows
  "live frame", "viewer", "demand"; nothing Sony-specific.
- **H3 holds:** frames are passed through as the camera made them; overlays are UI drawing. Machine-guidance pixel
  analysis is a future exception (LV-D13).
- **H8 holds:** the hub, demand and retake aim are services; Tauri and the HTTP server are thin.

## Mechanism contracts (proposed; deepened by the mechanism inventory)

### MC-LV1 — Live frame
`{ seq: u64 (per live-view start, strictly increasing), captured_at: monotonic + wall time (stamped when the frame
reaches the app), width, height, jpeg: bytes, transport: usb | wifi, focus: optional (only what the SDK supplies) }`.
A frame older than the hub's newest is dropped. A viewer never receives a frame with a lower `seq` than one it has.

### MC-LV2 — Viewer demand and sleep
A viewer **wants live** while it is in Live view (peek held or Lock on) and visible (window focused / phone screen
on). The hub starts live view when the first viewer wants it and stops it after a grace period (setting, initial
value from LV-S2) once no viewer wants it. Start, stop and grace never affect capture, file delivery or the camera
session. Rules for a viewer that disappears (phone locked, Wi-Fi drop) come from LV-S3.

### MC-LV3 — Retake aim (extends 10G §4/§6)
10G places by shutter order; a retake from the phone needs a chosen container. Pressing Retake on filled container
*c* sets the session's **aim** to *c*. The **next shutter** (from any device, including the camera's own button)
takes the aim: the aim is bound to that exposure when the camera reports it (10E, `exposed_at`), so a late file still
goes to *c*. When that photo arrives it is placed exactly as 10G §6 rule 3 places a photo dropped on a filled
container (old photo denied and released, the new one takes *c*'s shot key and presets), then the aim clears and
Next is computed by 10G as before. Cancelling Retake clears the aim. At most one aim per session; it never survives a
session switch or a relaunch. Placement for every other shot is 10G unchanged.

## Machine guidance — what this chapter guarantees for it
- A Rust subscriber interface on the hub: a subscriber takes frames at its own rate, never slows human viewers.
- Its actions go through the same services as a human's (capture, retake aim now; crane commands later through their
  own integration family).
- Frames carry MC-LV1 metadata so a later analysis can relate a frame to a capture and a template container.

## Measurement ledger entries (proposed, enter chapter 14 on ratification)
| ID | Contract | Observable outcome | Defect family guarded |
|---|---|---|---|
| **M-LV1** | MC-LV1 | Every viewer shows only frames newer than the last it showed; a slow viewer never delays another or the capture path. | stale frames shown as live; one phone freezing the Mac |
| **M-LV2** | MC-LV2 | With no viewer wanting live, the sidecar pulls no frames after the grace period; a peek after sleep shows a live frame within the measured wake budget (from LV-S2). | live view running all day; peeks that feel broken |
| **M-LV3** | 10E MC-32/33, 10G §4 | A capture from the phone or the Mac yields exactly one photo in the active session, placed by 10G exactly as a camera-button capture, with live view on or off, on USB and Wi-Fi. | lost, duplicated or misplaced photos when shooting from the phone |
| **M-LV4** | MC-LV3 | A retake aimed at container *c* puts the next shot's photo in *c* and denies the old one, even when its file arrives after later shots; with no aim, placement equals 10G; the aim never applies to a second shot. | a retake landing in the wrong container or eating the next shot |
| **M-LV5** | LV-D2, LV-D11 | Only the camera settings listed for live view are ever written, network and FTP settings never; a photo captured with live view on has the same capture settings as one without (file type, quality, store destination, exposure mode). | live view silently changing how the camera shoots or uploads |
| **M-LV6** | MC-LV1 (operational) | Shot-to-screen latency and frame rate on USB and Wi-Fi, Mac and phone, recorded at hardware acceptance against targets the owner sets after LV-S1…LV-S3. | a design that passes tests and is unusable by eye |

## Owed before ratification (research spikes, in the camera lab `Camera-Connection-Tests`)
| Spike | Question | Where it is recorded |
|---|---|---|
| LV-S1 | Sony SDK live view on the a7R V, USB and Wi-Fi: frame size, frames per second, latency, CPU, at Low and High quality; do capture and Save-to-PC / Auto FTP keep working while it runs; which properties must be set. | `evidence/sony_camera_lab.md` |
| LV-S2 | Can live view really sleep: does stopping the pull reduce camera battery use, or does the camera keep its sensor live anyway (monitor/EVF on)? Wake time after a pause. | same |
| LV-S3 | Delivery: frame channel sidecar → Rust, WebSocket to the desktop webview and to iPhone Safari/PWA; latency, Wi-Fi load with a RAW uploading over FTP at the same time; viewer disappearing. | `evidence/live_preview_lab.md` |
| LV-S4 | PWA on iPhone from a LAN address: install, full screen, screen staying awake, which features need HTTPS (card LV3). | same |

## Non-goals (this chapter)
Machine guidance itself; crane controllers; recording video; focus/exposure control from the phone (beyond what live
view needs); access from outside the studio network; a native iOS app; changes to 10G's placement other than the
retake aim (MC-LV3).

## ⚠ OWNER DECISIONS REQUIRED (1)
Answered 2026-11-26: L1, L7, L9 → LV-D8; L2 → LV-D9; L8 → LV-D10; L4 → LV-D11; L5 → LV-D12; L6 → LV-D13.
To confirm at ratification (shaper reading of LV-D9/LV-D10): the retake aim, MC-LV3.

**Card LV3 — Secure connection for the phone app?**
- **Story:** iPhones only give a web app some features (staying awake, offline start) over a secure connection. The
  app's address on your Wi-Fi is not secure by default, so the screen may dim mid-shoot.
- **Branches:** (a) the app makes its own certificate and the phone installs a trust profile once; (b) plain
  connection, accept the limits; (c) decide after spike LV-S4.
- **Recommendation:** (c), then (a) if the limits bite.
- **On silence:** the gate holds.
- **Trace:** LV-D7, LV-S4.
