# 10J — Live preview (human guidance now, machine guidance later)

**Status: RATIFIED (owner, 2026-11-26: "i ratify"; changelog D18; not planned yet). All owner cards
answered and MC-LV3 confirmed (2026-11-25); all spikes LV-S1…LV-S6 done (2026-11-26). The owner's phone design (Shoot Companion) is
folded in (LV-D15…LV-D18) and its six conflicts answered (LV5–LV10). The design folder is local to the owner's
Mac and git-ignored (owner 2026-11-26: design folders are not committed).**
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
| LV-D15 | **The phone app is the owner's "Shoot Companion" design** (`claude_design/live_preview/design_handoff_shoot_companion/`, 2026-11-26, high fidelity: colours, type, spacing, gesture thresholds, haptics and timings are final). Minimal, gesture-driven: hold for live, slide up to lock, double-tap to unlock, tap to focus while locked, slide sideways to set the template overlay; shot dock, captured-image preview, Move drawer, Item drawer. | LV-D6 (overlay style as a Settings choice) → replaced by the overlay **opacity gesture** (0–100 %, default 35 %, shared by live view and preview); no outline mode |
| LV-D16 | **Split preview on live**: a split shot (10D Split, one 2:1 composition → two squares) shows a centre guide and the two halves on live view and in the preview, so it can be framed as it will be published. The design labels the halves A/B; the book calls them Left/Right. | — |
| LV-D17 | **Item drawer** on the phone: item info (name, SKU, article number, current shot, angle, image counts; more fields later) and an action **"Fill item properties"** that opens an **empty full-page container** for now; phone inputs come later (e.g. the 10I tape's target dimension, properties easier to fill phone in hand). | — |
| LV-D18 | **The phone follows the Mac's active session** ("reactive and submissive"): it shows and shoots for whatever session is open in Session Manager. **Later** (not built now, but the phone API must allow it): completing a session from the phone (triggering its agentic flows) and opening a new session. | 11 §Phone remote (the phone app is a separate mobile entry of the same frontend, QR + pairing token as planned) |
| LV-D19 | **Hardware-check answers (owner 2026-11-26, changelog D19).** (1) **Shot dock without hold gestures:** "remove the preview interaction … where the user taps and drags … uncomfortable, i prefer to increase the container sizes and make the function to be scrollable, single tap opens that preview or selects the shot template". Larger tiles; the dock scrolls sideways; a **single tap** on a filled tile opens its Preview, on an empty template tile makes it the current shot (Free shot tile as before); **no hold-to-magnify and no hold-and-drag-up**. (2) **A removed phone reinstalls:** after Remove in Settings → Phone, the phone's installed app (iPhone home-screen app, Android shortcut) stays unpaired; the owner accepts deleting and installing it again from the new QR code ("i will go for accept reinstalling for now"); the removed screen says so. | LV-D15 (dock gestures), card LV3 consequences |

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
- **Retake** (LV-D10, card LV6): press Retake on a captured shot; its photo is denied at once (undoable) and the shot
  becomes current; the next capture fills it, then the current shot moves on (MC-LV3).
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

## The phone design (Shoot Companion, 2026-11-26) — what it adds
Read against this book; the handoff README is the authority for look and gestures. Beyond LV-D15…LV-D18:
- **Exposure readout** on live (shutter · aperture · ISO, read-only from the camera).
- **Landscape shots rotate the view 90°** so the phone is held sideways; gestures follow the image.
- **Shot dock**: template shots, then extra Main images, a **Free shot** tile, then Condition images; tap opens or
  selects (**amended LV-D19:** larger tiles, sideways scroll, single tap only; no hold-to-magnify, no drag-up). **Preview** of a captured image with template overlay on hold;
  actions Back, Retake (template shots), Delete (extras, condition), **Move image** (to any Main position, a new
  extra, or the Condition group).
- **Capture** feedback is immediate (flash, haptic) and reconciled with the camera's confirmation (C3: an
  acknowledgement is not an exposure, an exposure is not a file).
- **Haptics** only where the browser has them (Android Chrome); iPhone gets the visual cues.
- **Templates in black and white** by the phone's own display filter, as 10G already shows them (H3: no image
  processing in the app's backend).
- An untemplated session: live, Free shot only; each capture is Main at the end of the list (P66), as today.
- **After the last template shot (card LV7):** captures are **Condition** (10G §4 rule 4), not extra Main images as
  the prototype's Free shot shows; the person can move any of them to Main (Move image). The Free shot tile therefore
  adds Condition images in a templated session; extra Main images exist only by Move.
- Six points changed existing rules; the owner answered them on 2026-11-26 (cards LV5–LV10 below).

## Architecture
```
camera ──USB or Wi-Fi──▶ camera sidecar (owns the SDK session; pulls live frames; focus and capture on
                                │            their own threads so pulling never waits)
                                │ frame channel (binary, separate from the JSON control line)
                                ▼
                      live view hub (Rust): newest frame only + who is watching
                         ├──▶ Session Manager window (desktop): Tauri channel, raw JPEG bytes (LV-S3)
                         ├──▶ phone app(s) on the studio Wi-Fi: embedded HTTP + WebSocket server (R26)
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
- **Delivery.** Two thin outlets over the one hub, both binary JPEG drawn on a canvas, no re-encoding:
  **desktop through a Tauri channel** (raw bytes; in the lab every frame at ~16 ms p50 / 22 ms p95 even with the
  window hidden, where a WebSocket in the same window dropped frames with ~110 ms tails; LV-S3), **phones through
  the embedded server's WebSocket** (25 fps, ~15–30 ms Mac → phone). The window could open `ws://` (loopback and LAN
  name) from `tauri://localhost`; the channel is chosen for robustness, not because WebSocket is blocked. Video
  encoding (WebRTC/H.264) only if a later measurement demands it.
- **Focus from the live picture (LV-D14, LV-S5).** A click/tap gives a point in the frame (0..1); the adapter maps
  it to the camera's grid (Sony: `AF_Area_Position`, x<<16|y on 640×480), then half-presses and reports
  focused / not focused. The camera's focus frame (Sony: live-view property, centre + size on the same grid, state
  focused/not) is drawn over live view. Measured on the a7R V with the owner's settings unchanged (AF-S, Flexible
  Spot S): **9/9 focused, click → focused ≈ 0.4 s (0.39–0.83)**, live view kept running at 21–25 fps when focus ran
  on its own thread (on the pull thread it froze ~0.5 s). Needs a movable Focus Area (Flexible Spot / Tracking);
  if the camera is set otherwise, **the app reports it and does not change it** (owner confirmed 2026-11-26). Exposure mode is never touched (Sony's sample switches to P; the owner shoots Manual).
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

### MC-LV3 — Current shot (replaces the retake aim; amends 10G §4 rule 2 and §6) — owner 2026-11-26 (cards LV5, LV6)
In a templated session there is **one current shot**: a container (or none). Default: 10G's "Next" — the first empty
container in **shooting order**, after any reported shutters whose files are still on the way. The person changes it
by tapping an empty shot (phone dock, later the Mac). **Every exposure** — phone, Mac or the camera's own button — is
bound to the current shot **as it was when the camera reported the exposure** (10E `exposed_at`; a file that arrives
late still goes there). After an exposure is bound, the current shot moves to the next empty container after it in
shooting order, else the first empty one, else **none**. With no current shot, shots are **Condition** (10G §4 rule 4,
appended; the person may move them to Main). **Retake** on a filled container = **Deny** the photo there at once
(red, undoable, released as in 10G §6) and make that container current. Filling a container applies its presets
(10G §4 rule 3). A photo placed through the current shot counts as **placed by template**. The current shot is per
session, never survives a session switch or relaunch (it is recomputed from the default), and an untemplated session
has none (P66: Main, end of list). This one rule absorbs 10G's shot-index placement, its refill and the earlier
retake aim.

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
| **M-LV4** | MC-LV3 | Each exposure lands in the shot that was current when the camera fired, even when its file arrives after later shots; the current shot then moves to the next empty one in shooting order; Retake denies the old photo at once (undoable) and the next exposure fills that shot; with no current shot photos go to Condition. | a photo landing in the wrong container; a retake eating the next shot; late Wi-Fi files misplaced |
| **M-LV5** | LV-D2, LV-D11 | Only the camera settings listed for live view are ever written, network and FTP settings never; a photo captured with live view on has the same capture settings as one without (file type, quality, store destination, exposure mode). | live view silently changing how the camera shoots or uploads |
| **M-LV6** | MC-LV1 (operational) | Shot-to-screen latency and frame rate on USB and Wi-Fi, Mac and phone, recorded at hardware acceptance against targets the owner sets after LV-S1…LV-S3. | a design that passes tests and is unusable by eye |

## Owed before ratification (research spikes, in the camera lab `Camera-Connection-Tests`)
| Spike | Question | Where it is recorded |
|---|---|---|
| LV-S1 | Sony SDK live view on the a7R V, USB and Wi-Fi: frame size, frames per second, latency, CPU, at Low and High quality; do capture and Save-to-PC / Auto FTP keep working while it runs; which properties must be set. | **Done 2026-11-25** (CPU not measured; moves to LV-S3) — `evidence/sony_camera_lab.md` §Live view |
| LV-S2 | Can live view really sleep: does stopping the pull reduce camera battery use, or does the camera keep its sensor live anyway (monitor/EVF on)? Wake time after a pause. | **Done 2026-11-25**: camera keeps live view on; no battery saving; wake 0 ms — same |
| LV-S3 | Delivery: frame channel sidecar → Rust, WebSocket to the desktop webview and to iPhone Safari/PWA; latency end to end; Mac CPU; viewer disappearing. | **Done 2026-11-26**: phones via WebSocket (Experiment 08); desktop window via Tauri channel (Experiment 09: 25 fps, ~16 ms, 0 dropped; WebSocket in the window allowed but degraded when hidden). Viewer reconnect/several phones at hardware acceptance — `evidence/live_preview_lab.md` |
| LV-S4 | PWA on iPhone from a LAN address: install, full screen, screen staying awake, which features need HTTPS (card LV3). | **Done 2026-11-25**: install + full screen on both; wake lock and service worker only on HTTPS — `evidence/live_preview_lab.md` |
| LV-S5 | Focus from the live picture on the a7R V: set the point from a click, focus, confirm, draw the camera's focus frame; settings needed; latency; USB and Wi-Fi; does a capture keep the point. | **Done on USB 2026-11-26** (Experiment 10): 9/9 focused ≈ 0.4 s, settings unchanged, focus on its own thread keeps live view moving. Wi-Fi and "photo sharp where clicked" at hardware acceptance — `evidence/sony_camera_lab.md` §Live view |
| LV-S6 | Owner-requested before implementation (2026-11-26): focus by tap from a phone over Wi-Fi; a photo after a tap sharp where tapped; several phones at once. | **Done 2026-11-26** (Experiment 11): from the Android phone over Wi-Fi 10/10 focused in 0.38–0.83 s; both photos sharp exactly where tapped (other object blurred); Mac + Android together each 19.4 fps (all camera frames), none skipped. The iPhone could not reach the Mac that session (worked in Experiment 08); three viewers at once not shown — `evidence/live_preview_lab.md` |


## Non-goals (this chapter)
The Mac's live view screen (card LV10: the owner designs it later; the phone comes first — the desktop delivery
finding, a Tauri channel, stays recorded for then); machine guidance itself; crane controllers; recording video; exposure control from the phone; focus control beyond choosing the focus point (LV-D14); access from outside the studio network; a native iOS app; changes to 10G's placement other than the
retake aim (MC-LV3).

## ⚠ OWNER DECISIONS REQUIRED (none open; all answered 2026-11-26)
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

**Card LV5 — Where does the next photo go?** — **ANSWERED 2026-11-26: (a)** "correct" → MC-LV3 rewritten (current shot, shooting order, bound at exposure).
- **Story:** in 10G each shot goes to the container of its place in the **shooting order** (shot 3 → third
  container in shooting order). The design gives the phone a **current shot**: tapping an empty shot makes it current,
  each capture fills the current shot, then it moves on to the next empty one. The design's shots run in one order
  only (its list order, which is the final/published order); it has no shooting order.
- **Branches:** (a) **one current shot per session**, shown on the phone and as "Next" on the Mac; every exposure,
  from any button including the camera's own, goes to the current shot **as it was when the camera fired** (so a late
  Wi-Fi file still lands right); then current moves to the next empty shot **in shooting order**; tapping a shot
  changes it. The phone dock lists shots in shooting order with their final numbers. (b) Keep 10G's rule; the phone
  only shows Next and cannot choose. (c) Current shot, but advancing in final order (as the prototype).
- **Recommendation:** (a): it keeps your shooting order, lets you choose freely, and absorbs today's retake aim
  (MC-LV3) and 10G's refill into one rule.
- **On silence:** the gate holds. **Trace:** 10G §4–§6, MC-LV3, design "current".

**Card LV6 — Retake: when does the old photo go?** — **ANSWERED 2026-11-26: (a) design way** → denied at once, undoable.
- **Story:** the design's Retake **removes the photo at once** ("Shot 02 reopened") and makes that shot current. The
  rule you confirmed yesterday (MC-LV3) removes the old photo only **when the new one arrives**.
- **Branches:** (a) as the design: Retake = Deny the old photo now (red, undoable) + make the shot current; (b) as
  MC-LV3: keep the old photo until the new one arrives.
- **Recommendation:** (a): what you see is what happens, undo brings it back, and with LV5 (a) no special aim rule
  is needed. **On silence:** the gate holds. **Trace:** MC-LV3, 10G §6 rules 3–4.

**Card LV7 — Photos after the last template shot: extra Main or Condition?** — **ANSWERED 2026-11-26 (b):** "photos after the template shot are condition, i can make them main if i want to" → 10G §4 rule 4 stays; the design's Free shot is adapted.
- **Story:** 10G classifies every shot after the last container as **Condition** automatically. The design's
  **Free shot** adds each capture as an **extra Main image**; photos reach Condition only by "Move image → Condition".
- **Branches:** (a) as the design: extra Main; Condition by Move; (b) as 10G: automatically Condition (the Free shot
  tile would then fill Condition); (c) as the design, plus a quick way to send the next captures to Condition.
- **Recommendation:** (a) if condition photos are few per item; (b) if you usually shoot several damage close-ups
  after the angles (moving each by hand is slow). **On silence:** the gate holds. **Trace:** 10G §4 rule 4, 06.

**Card LV8 — Moving a photo onto an occupied place: swap or replace?** — **ANSWERED 2026-11-26: (a)** phone swaps, the Mac keeps replace-by-drop.
- **Story:** the design's Move **swaps** the two photos (nothing is lost); a move into Condition takes a chosen
  position. On the Mac today, dropping a photo onto a filled container **replaces** it and deletes the old one (10G
  retake by drop), and a move between Main and Condition goes to the end (06).
- **Branches:** (a) phone swaps (as designed), the Mac keeps replace-by-drop; (b) both swap; (c) both replace.
- **Recommendation:** (a): each fits its gesture (an explicit "Move" picker on the phone, a drop on the Mac);
  revisit if it confuses. **On silence:** the gate holds. **Trace:** 10G §6 rule 3, 06 Order.

**Card LV9 — Clearing focus ("AF LOCKED ×")** — **ANSWERED 2026-11-26: (a)** × also returns the camera's focus point to the centre.
- **Story:** after a tap the phone shows "AF LOCKED"; × clears it. On the camera the focus point stays where you
  tapped and each capture refocuses there (measured: the photo is sharp where tapped).
- **Branches:** (a) × also moves the camera's focus point back to the centre; (b) × only removes the marker; the
  camera keeps the point.
- **Recommendation:** (a): otherwise the screen says "no focus point" while the camera still focuses at the old
  spot. **On silence:** (a). **Trace:** LV-D14, Experiment 10/11.

**Card LV10 — The Mac's live view** — **ANSWERED 2026-11-26: wait**; "i don't think i will be using that view at the moment on the mac, the phone is the most important" → not built now (non-goals).
- **Story:** the design covers the phone. The chapter says the Mac shows the same views on its stage.
- **Branches:** (a) the Mac reuses the Shoot Companion view inside Session Manager's window (mouse for gestures);
  (b) you design the Mac view in Claude Design first, the phone is built first meanwhile; (c) the Mac shows live and
  template with buttons only, no gestures.
- **Recommendation:** (b), as you prefer to design before building; nothing in the phone build waits for it.
- **On silence:** (b). **Trace:** LV-D3, design scope.
