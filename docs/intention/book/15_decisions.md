# 15 — Decisions

## Resolved (owner, unless marked shaper)
| ID | Decision | Date |
|---|---|---|
| R1 | ~~Native macOS app, uv-managed Python; no Docker.~~ Superseded by R21. | 2026-10-23 |
| R2 | ~~FastAPI, industry-standard layout.~~ Superseded by R21. The owner's contract set is still not applied. | 2026-10-23 |
| R3 | React + TypeScript + Vite + shadcn/ui; Node on the work Mac is acceptable. | 2026-10-23 |
| R4 | Shopify through a custom dev-app token; lookup fields barcode and SKU (design adds title). | 2026-10-23 |
| R5 | Public GitHub repo, commercially open source for now (C1). | 2026-10-24 |
| R6 | A Session is one item's photography job (C2). | 2026-10-24 |
| R7 | Attribute schema and provenance in the MVP; the AI agent is the first post-MVP phase (C3). | 2026-10-24 |
| R8 | ~~One transport, HTTP + WebSocket, under Tauri and on the phone; Tauri is lifecycle only (C4).~~ Reopened as card A1 by R21. | 2026-10-24 |
| R9 | Capture One is the only editing provider in the MVP; no pixel processing in the repo (C5). | 2026-10-24 |
| R10 | ~~Tauri desktop shell with Python sidecar.~~ Superseded by R21. | 2026-10-24 |
| R11 (shaper) | `capture-one-mcp` not adopted: extra Node/MCP runtime designed for LLM clients; the dictionary covers every command. | 2026-10-23 |
| R12 (shaper) | Review and group collapse into one classification state (design: approve / condition / delete). | 2026-10-24 |
| R13 (shaper, from evidence) | Condition photos map to `custom.damage_report`; the brief's `custom.condition_images` does not exist in the shop. Theme rendering unverified. | 2026-10-24 |
| R14 (shaper) | "Replace" flag removed; re-publishing is the dirty-state flow from the design (N1 → R18). | 2026-10-24 |
| R15 (shaper) | Shopify API version pinned in settings, defaulting to 2026-10. | 2026-10-24 |
| R16 | **D1 yes.** The app sets Capture One's capture folder to `Capture/<SKU>/` when a session becomes active. | 2026-10-24 |
| R17 | **D2 tags only**, behind the editing provider's `review.mirror` capability: main green, condition blue, deleted red, unreviewed none; ratings ignored. Writing tags ships; reading them back is only-if-cheap, because the owner expects to sort only in the app. (Shaper interpretation of "mirror, but coloring won't be needed" — confirm at ratification.) | 2026-10-24 |
| R18 | **N1 mirror.** On every publish the platform gallery becomes exactly the session's Main list; platform photos the session does not own are removed, and the removal count is shown on the confirmation popover before confirm. | 2026-10-24 |
| R19 | **N2 exclude.** Unreviewed photos never publish or export; the popover says "n unreviewed photos will not be published". (Owner answered "yes" to the recommendation.) | 2026-10-24 |
| R20 | **N3 after.** The phone remote is the first phase after the desktop MVP. | 2026-10-24 |
| R21 | **Rust is the backend.** Tauri + Rust carry application services, domain, persistence, jobs, orchestration and native operations; no FastAPI, no uv. Python is optional, introduced only for a demonstrated need, as a sidecar. | 2026-10-24 |
| R22 | **Camera connectivity is an integration**: camera service → camera provider interface → device adapter with declared capabilities. Sony ZV-E10 first; no manufacturer in core. Direct device control is not assumed feasible until researched. | 2026-10-24 |
| R23 | **Editing software is an integration**: editing service → editing provider interface → Capture One adapter. Communication mechanism (AppleScript, filesystem) is the adapter's concern. | 2026-10-24 |
| R24 | **Tauri commands are thin.** Business logic never depends on Tauri-specific commands or events (H8 revised). Layers: frontend, core domain, application services, integration layer, infrastructure. | 2026-10-24 |
| R25 | Concrete MVP integrations are exactly: Sony ZV-E10 (camera), Capture One (editing), Shopify (commerce). No speculative adapters or plugin system. | 2026-10-24 |

## ⚠ OWNER DECISIONS REQUIRED (3)

**D4 — When does Capture One render the web-size files?**
Question: The files we upload are rendered by Capture One with the 3000 px web recipe (not our
code). Should that render run when you confirm Publish or Export, or earlier, each time you
approve a photo? (This is separate from the live preview, which chapter 10 now defines.)
Story: Seven frames approved at 10:00, two retouched in Capture One at 14:00, published at
16:00. Render-at-publish uploads the retouched versions and costs about a minute at 16:00.
Render-at-approve would upload the 10:00 versions unless re-rendered.
Branches: **At publish** — always the latest edit; the design's "Preparing images" step is this.
**At approve** — faster publish, stale risk.
Recommendation: At publish.
On silence: gate holds.
Trace: chapter 07 steps, chapter 10, M3.

**A1 — How does the desktop window talk to the Rust backend?**
Question: Typed Tauri commands and events in-process, adding an HTTP + WebSocket server inside the
same Rust process only when the phone remote arrives; or run that server from day one and have
the desktop window use it too (the former "one transport" decision R8)?
Story: With Tauri commands the MVP has no open port, no auth and no serialization layer to
maintain; the phone later gets a server that calls the same services. With a server from day one,
desktop and phone share one code path, at the price of a port, a token and a second type pipeline
before anyone needs them.
Branches: **Tauri commands now, server later** — less MVP surface; H8 keeps the services
transport-agnostic. **Server for both from day one** — one path, more MVP work.
Recommendation: Tauri commands now; services must be callable without Tauri so the server is an
addition.
On silence: gate holds.
Trace: chapter 11, H8, M4.

**A2 — Which camera provider does the MVP ship?**
Question: Capture One holds the USB tether to the ZV-E10, and only one program can hold it.
Should the MVP's camera adapter work *through Capture One* (status from the document's camera
property, shutter via the `capture` command, files via the capture folder), with a direct Sony
adapter as a later provider that would take the tether over from Capture One; or should we
research and build a direct Sony adapter (Camera Remote SDK or PTP) now?
Story: Through Capture One, the first stage needs no SDK licence, no device driver and no change
to the S0 tether checklist, and the header dot and capture button still work. A direct adapter
means Capture One can no longer tether; files would have to be handed to it by import, and its
Next Capture Adjustments would not apply on arrival.
Branches: **Through Capture One (MVP)** — camera interface defined, one adapter, direct adapter
deferred. **Direct Sony adapter now** — research SDK availability for the ZV-E10 and macOS first;
reworks capture arrival.
Recommendation: Through Capture One for the MVP; the camera provider interface and capability
list are defined now so the direct adapter is a drop-in later.
On silence: gate holds.
Trace: chapter 10, chapter 13, M1.
