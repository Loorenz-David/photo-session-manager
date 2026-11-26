# 10J — Live preview (human guidance now, machine guidance later)

**Status: COLLABORATING (shaper draft 2026-11-25, rebased 2026-11-26; not ratified, not planned). All owner cards
answered and MC-LV3 confirmed (2026-11-25); spikes LV-S1, LV-S2, LV-S4 done, LV-S3 phone half done. Left before
READY_FOR_RATIFICATION: LV-S3 desktop-webview half and LV-S5 focus from the live picture (§Owed before
ratification); the owner's Claude Design prototype for the phone app is folded in first.**
Source: the owner's chat of 2026-11-25/26 (answers folded below). First written as "10H" on a stale checkout;
renamed when the checkout was brought up to date (10G and 10H are Shooting templates and Archived items).
Changelog entries D3–D4. Builds on **10G Shooting templates** (containers, shooting order, Next, retake) and
**10E Direct camera adapter** (sidecar, transports, capture). Evidence: spikes LV-S1 and LV-S2 **done** (2026-11-25,
`evidence/sony_camera_lab.md` §Live view — Experiment 07, folded in below, changelog D5); LV-S3 and LV-S4 owed
(§Owed before ratification, a new `evidence/live_preview_lab.md`). The phone and desktop look come from the owner's
Claude Design work.

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
| LV-D2 | **The app may change camera settings that live view requires.** Network and FTP settings stay untouched (10E MC-31); the exact list comes from spike LV-S1. **LV-S1: on the a7R V none is required** (live view is enabled at connect); `LiveView_Image_Quality` (Low/High) is the only candidate, if the owner wants Low. | camera hard rule "the camera's own settings are never changed" (narrowed) |
| LV-D3 | **One frame source in Rust, any number of viewers** (shaper's proposal, accepted; the owner notes Sony's own tools behave this way). | — |
| LV-D4 | **Live view over USB and over Wi-Fi**, like photo delivery: whichever transport the camera session is on (10E MC-31). | — |
| LV-D5 | **Live view sleeps when nobody is looking**, to save the camera's battery, the Mac and the network, if the camera allows it (spike LV-S2). Frame rate and quality are tuned by measurement, not fixed here. **LV-S2: the camera keeps its live view running whether or not the app pulls; pausing saves the Mac, the Wi-Fi and the phone, not camera battery** (≈ 27–33 %/h over Wi-Fi either way). What sleep should mean now: card LV4. **Revised 2026-11-25 (owner, card LV4 → branch c): no sleep feature.** Pausing does not rest the camera, so it is not worth building; live frames flow whenever the camera session is up. Revisit only if LV-S3 shows a real Mac or Wi-Fi cost. | — |
| LV-D6 | **Overlay style is a setting**: how the template shot appears together with live (flip, ghost with opacity, outline). | — |
| LV-D7 | **The phone app is installable (PWA)**: full screen, home-screen icon, feels like an app; served by the app's own server (chapter 11 §Phone remote). **Served over plain HTTP now, HTTPS later** (card LV3): iPhone installs full screen; Android uses a shortcut plus a Full screen button. | 11 §Phone remote (adds install), 13 (phone remote moves up) |
| LV-D8 | **The template is 10G's shooting template** (per item type; containers in published order plus a shooting order). Placing photos by shooting order is what guarantees each photo lands in the right container despite transfer delays; the photographer can always drag a photo to another container. Templates are made in Settings → Shooting templates (10G §3). Answers cards L1, L7, L9. | — |
| LV-D9 | **Taking a picture moves to the next shot** (10G's Next). The photographer can go back to retake a shot and can view what was taken for it. Answers card L2. | — |
| LV-D10 | **A retake replaces:** the new photo takes the container, the old one is denied (undoable, as 10G §6 rule 3 already does for a dropped photo). Answers card L8. | — |
| LV-D11 | **Camera settings changed for live view are not restored**, on the condition that they do not slow the workflow and do not change how photos are captured. Live shooting becomes the normal way of shooting. Answers card L4. | — |
| LV-D12 | **Any paired phone may watch and shoot, several at once.** Answers card L5. | — |
| LV-D13 | **Image analysis for machine guidance is a recorded future exception to H3**, decided with the machine-guidance chapter; human guidance needs none. Answers card L6. | — |
| LV-D14 | **Focus from the live picture** (owner 2026-11-26, "the ability to direct the focus with this live stream which the IEDT does … once i have changed the furniture positioning the focus should also change"): clicking or tapping a point on the live view, on the Mac or the phone, moves the camera's focus point there and focuses; the live view shows where the camera is focusing. Mechanism and required camera settings from spike LV-S5. | 10J non-goals (focus control from the phone, narrowed); LV-D2 (Focus Area may need a movable setting) |

## The working loop (owner's description, the acceptance story)
1. The phone (or the Mac) shows the **template shot for Next** (10G §5), frozen, full opacity.
2. The photographer places the piece, then **looks at live** (peek), sees the difference.
3. They put the phone down and **adjust the piece**; the phone shows the template again.
4. They peek again; when it matches, they press **Capture** (on the phone or the Mac).
5. The photo goes to that container by 10G's placement; **Next moves on** and its template shot appears; repeat.

Physical correction takes far longer than looking. Live view keeps running in the background meanwhile (LV-D5
revised): pausing it would not save camera battery (LV-S2).

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
  as a shutter press on the camera would (MC-32/MC-33 unchanged) and is placed by 10G. The command must keep 10E's
  half-press-then-release: in the lab a release without half-press was acknowledged and **silently not taken**
  (camera in AF). The live picture freezes ~0.3–0.45 s around each capture and resumes by itself.
- **Phone UI on Android:** a **Full screen** button (Chrome does not install the app over HTTP; card LV3).
- **Status:** "Live", "Camera busy", "Live unavailable on this camera"; the camera's
  own status (10E) is unchanged. The camera takes one remote session: if another Mac's Session Manager holds it, the
  lab saw only a generic connection error (evidence C12); 10E should name it ("camera in use by another computer").
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
  API**: `GetLiveViewImageInfo` then `GetLiveViewImage` return one JPEG per call with the camera's frame number;
  properties `LiveViewStatus` (NotSupport / Disable / Enable), `LiveView_Image_Quality` (Low / High),
  `LiveViewProtocol` (Main / Alt). **Measured on the a7R V (LV-S1):** 1024×680 (High) or 640×424 (Low) JPEG, 25 fps
  on USB, ~20 fps on Wi-Fi, no dropped frames, glass → Mac ≈ 120 ms on both. The camera **never** answers "no new
  frame" (`CrWarning_Frame_NotUpdated`); it returns the last frame again, so the sidecar **paces its pulls to the
  frame interval and forwards a frame only when its frame number is new**. Frames go over their **own channel** (10E
  reserved "its own frame channel"), never as JSON on the control line. The sidecar pulls while the camera session is up
  (no sleep: LV-D5 revised); the camera keeps its own live view running anyway (LV-S2).
- **Shared Wi-Fi.** A RAW upload by Auto FTP (≈ 4.3 s for 55 MB) drops Wi-Fi live view to ~13 fps (lowest 9) while
  it runs; live view never slows the upload (LV-S1). The UI must tolerate this dip without calling live "stalled".
- **Hub.** Keeps exactly the newest frame; every viewer gets the latest; a slow viewer skips frames, never queues.
  A viewer receives frames only while it shows Live (MC-LV2).
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
The adapter maps its own frame counter to `seq` (Sony: `GetFrameNo`, which restarts at 1 per connection); a pulled
frame whose counter has not advanced is a repeat and is never forwarded (LV-S1).

### MC-LV2 — Who receives frames
Live frames are pulled whenever the camera session is up and the camera declares `live_view` (no sleep, LV-D5
revised). A viewer **receives** frames only while it shows Live (peek held or Lock on) and is visible (window focused /
phone screen on); otherwise it is sent nothing. Pulling and sending never affect capture, file delivery or the camera
session. Rules for a viewer that disappears (phone locked, Wi-Fi drop) come from LV-S3.

### MC-LV3 — Retake aim (extends 10G §4/§6) — confirmed by the owner 2026-11-25
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
| **M-LV2** | MC-LV2 | A viewer not showing Live receives no frames; a viewer switching to Live shows a frame newer than the switch within the measured delivery budget (LV-S3); live frames never delay capture or file delivery. | phones on the template view still downloading video; a switch to Live showing a stale frame |
| **M-LV3** | 10E MC-32/33, 10G §4 | A capture from the phone or the Mac yields exactly one photo in the active session, placed by 10G exactly as a camera-button capture, with live view on or off, on USB and Wi-Fi. | lost, duplicated or misplaced photos when shooting from the phone |
| **M-LV4** | MC-LV3 | A retake aimed at container *c* puts the next shot's photo in *c* and denies the old one, even when its file arrives after later shots; with no aim, placement equals 10G; the aim never applies to a second shot. | a retake landing in the wrong container or eating the next shot |
| **M-LV5** | LV-D2, LV-D11 | Only the camera settings listed for live view are ever written, network and FTP settings never; a photo captured with live view on has the same capture settings as one without (file type, quality, store destination, exposure mode). | live view silently changing how the camera shoots or uploads |
| **M-LV6** | MC-LV1 (operational) | Shot-to-screen latency and frame rate on USB and Wi-Fi, Mac and phone, recorded at hardware acceptance against targets the owner sets after LV-S1…LV-S3. | a design that passes tests and is unusable by eye |

## Owed before ratification (research spikes, in the camera lab `Camera-Connection-Tests`)
| Spike | Question | Where it is recorded |
|---|---|---|
| LV-S1 | Sony SDK live view on the a7R V, USB and Wi-Fi: frame size, frames per second, latency, CPU, at Low and High quality; do capture and Save-to-PC / Auto FTP keep working while it runs; which properties must be set. | **Done 2026-11-25** (CPU not measured; moves to LV-S3) — `evidence/sony_camera_lab.md` §Live view |
| LV-S2 | Can live view really sleep: does stopping the pull reduce camera battery use, or does the camera keep its sensor live anyway (monitor/EVF on)? Wake time after a pause. | **Done 2026-11-25**: camera keeps live view on; no battery saving; wake 0 ms — same |
| LV-S3 | Delivery: frame channel sidecar → Rust, WebSocket to the desktop webview and to iPhone Safari/PWA; latency end to end; Mac CPU; viewer disappearing. (Camera-side FTP load already measured in LV-S1.) | **Phone half done 2026-11-25** (25 fps, Mac → phone ~15–30 ms, glass → phone ≈ 145 ms, Mac ≈ 5–7 % of a core per side); desktop webview, Wi-Fi camera to phone, several phones, reconnect still owed — `evidence/live_preview_lab.md` |
| LV-S4 | PWA on iPhone from a LAN address: install, full screen, screen staying awake, which features need HTTPS (card LV3). | **Done 2026-11-25**: install + full screen on both; wake lock and service worker only on HTTPS — `evidence/live_preview_lab.md` |
| LV-S5 | Focus from the live picture on the a7R V: set the focus point from a click (`AF_Area_Position`, 640×480 grid, or `RemoteTouchOperation`), focus (half-press), confirm focus, read the camera's focus frame for drawing (`GetLiveViewProperties`); which Focus Area / Focus Mode it needs; latency click → focused; USB and Wi-Fi; does a capture keep that point. | lab Experiment 10 → `evidence/sony_camera_lab.md` §Live view |

## Non-goals (this chapter)
Machine guidance itself; crane controllers; recording video; exposure control from the phone; focus control beyond choosing the focus point (LV-D14); access from outside the studio network; a native iOS app; changes to 10G's placement other than the
retake aim (MC-LV3).

## ⚠ OWNER DECISIONS REQUIRED (none open)
Answered 2026-11-26: L1, L7, L9 → LV-D8; L2 → LV-D9; L8 → LV-D10; L4 → LV-D11; L5 → LV-D12; L6 → LV-D13.
**MC-LV3 (retake aim) confirmed 2026-11-25** as stated: the next photo from any button takes the aimed container
and denies the old photo, bound at exposure so a late file still lands there; Next returns to where it was; one
photo only; cancel, session switch or relaunch clears it. LV4 (opened and answered
2026-11-25): no sleep feature → LV-D5 revised.

**Card LV3 — Secure connection for the phone app?** — **FINAL 2026-11-25 (owner): plain HTTP now, HTTPS later.**
"because this implementation for https increases the complexity maybe we can go and develop for http then later in
the future we can develop the https, for now on iphone im comfortable with the friction of the one time set up and on
android the full screen action capability." Consequences: iPhone: Add to Home Screen once, opens full screen (tested);
screen lock is the phone's Auto-Lock setting. Android: a home-screen shortcut opening in Chrome plus a **Full screen
button** in the phone UI (tested on HTTP; one tap per open). The phone app uses **no secure-context API**, and the
server and pairing are built so HTTPS can be added later without changing the phone app (same paths, `wss://` from an
`https://` page, `ws://` from `http://`). HTTPS later = the owner-domain route below (Route 53 subdomain, publicly
trusted certificate, no phone setup), lab Experiment 09, deferred. Unencrypted studio Wi-Fi traffic accepted until
then.
- Superseded the same day: **HTTPS
with a publicly trusted certificate on an owner-controlled domain** (a subdomain of the owner's Route 53 domain,
pointing at the Mac's local address); **nothing is installed on phones**. Reason: Android Chrome refuses to install
the app over plain HTTP (evidence §Android), and the full app experience matters. Session Manager keeps the DNS record
on the Mac's current address and renews the certificate itself (DNS challenge through Route 53, least-privilege
credentials). The domain name stays out of the public repo. Open: router acceptance of a public name resolving to a
private address, and the end-to-end check on both phones (lab Experiment 09). The earlier plain-HTTP answer follows
for the record.
- Earlier answer, superseded: **plain HTTP (branch b).** "If there
is actually no need for the https then there is no need of it, the lock screen is user phone configuration."
Everything the phone app needs works on HTTP (live frames, capture, iPhone home-screen app full screen, tested); what
HTTP lacks is not needed (wake lock → the user's Auto-Lock; service worker → no use without the Mac; push, camera,
Web Crypto unused). Recorded consequences: (1) **Android full screen unverified**: Chrome installs a standalone app
only for HTTPS, so on HTTP the home-screen icon likely opens inside Chrome; test when an Android phone is available;
the fallback is a publicly trusted certificate on an owner-controlled domain pointing at the Mac (no install on any
phone), not a profile; (2) **studio Wi-Fi traffic is unencrypted**: live frames and the paired phone's token are
visible to others on that Wi-Fi; accepted for a private studio network; pairing tokens are per phone and revocable.
The phone app must not depend on any secure-context API.
- Original card:
- **Story:** iPhones only give a web app some features (staying awake, offline start) over a secure connection. The
  app's address on your Wi-Fi is not secure by default, so the screen may dim mid-shoot.
- **Branches:** (a) the app makes its own certificate and the phone installs a trust profile once; (b) plain
  connection, accept the limits; (c) decide after spike LV-S4.
- **Evidence (LV-S4, 2026-11-25, owner's iPhone, `evidence/live_preview_lab.md`):** on plain HTTP the installed app
  opens full screen and streams at 25 fps, but cannot keep the screen on (with Auto-Lock 30 s it locked at ~61 s and
  the stream closed) and cannot open offline. On HTTPS, after a one-time trusted profile (a test authority limited to
  local names and private addresses), the screen stayed on > 3 min, streaming, after one tap.
- **Recommendation:** **(a)**: the limits bite (a phone that locks while the photographer handles the piece). Shape:
  Session Manager creates its own authority, limited to `.local` names and private addresses, plus a certificate for
  the Mac's `.local` name; the phone installs the profile once during pairing (a QR code on the Mac opens it);
  re-issued before expiry; removable from Settings.
- **On silence:** the gate holds.
- **Trace:** LV-D7, LV-S4, evidence P1–P3.

**Card LV4 — What should "sleep" mean, now that pausing does not rest the camera?** — **ANSWERED 2026-11-25:**
"if sleep has no influence over resources then is not worth implementing" → branch (c), no sleep feature (LV-D5
revised, MC-LV2 rewritten). Shaper note given with the answer: pausing would still save Mac and Wi-Fi work (≈ 2–3.5
MB/s), not camera battery; LV-S3 measures the Mac cost, and only a real cost reopens this.
- Original card:
- **Story:** the lab showed the camera keeps its live view running whenever the app is connected, pulled or not.
  Pausing still stops the Mac decoding, the Wi-Fi carrying ~2–3.5 MB/s and the phone drawing, but the camera's
  battery drains the same (≈ 27–33 % an hour over Wi-Fi, about three hours a charge; on the USB cable the camera is
  powered and does not drain).
- **Branches:** (a) keep sleep as designed (stop pulling when nobody looks; instant wake), accept the camera's
  battery use, and recommend the USB cable or a mains adapter for long sessions; (b) also research a real camera-side
  off (e.g. whether live view can be disabled through the SDK without ending the session, and what it saves), as a
  later lab spike; (c) drop sleep entirely and pull whenever connected.
- **Recommendation:** (a) now; (b) only if Wi-Fi battery life turns out to limit real shoots.
- **On silence:** (a) holds as the working assumption; the gate does not wait on this card.
- **Trace:** LV-D5, MC-LV2, LV-S2, evidence C11.
