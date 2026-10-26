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
| R26 | **A1 Tauri commands now.** Desktop UI uses typed Tauri commands and events; an HTTP + WebSocket server is embedded in the same Rust process only when the phone remote arrives. Services stay callable without Tauri (H8). Closes R8. | 2026-10-24 |
| R28 | **D4 at publish.** Web-size files are rendered by the editing provider only inside a publish or export run, never at approve. | 2026-10-24 |
| R29 | Adjusted-preview refresh: a Capture One edit marks the photo stale; re-render 15 s after the last edit (setting 10–30 s) or at once when the app window becomes visible/focused. | 2026-10-24 |
| R30 | **I1 Yes.** A Capture One retouch of a published photo makes the session need publishing again; the next publish replaces that photo on Shopify in the same gallery place (07B MC-21). | 2026-10-25 |
| R31 | **Card 2: generate, editable; SKU editable after photos.** New local items get `<prefix>-<next number>` proposed, editable before confirming; an item's SKU can change at any time; the capture folder keeps the key it had at the first photo (05B MC-22). | 2026-10-25 |
| R33 | **Tag merge base (orchestrator from MC-13's intent, 2026-10-26).** The three-way tag merge uses the app's own last-published tag list as base, refreshed after every successful update; admin-added tags are never removed, app removals reach the shop. (07A MC-13) | 2026-10-26 |
| R32 | **Shopify access and condition field (owner 2026-10-26).** The app authenticates as a Dev Dashboard app (client ID + secret, client credentials grant, token in memory only); the probe and development run against the owner's test store and the real store is a settings change; condition photos go to the one metafield named by the setting `condition_photos_key` (initial `custom.damage_report`). (09A MC-23, MC-24) | 2026-10-26 |
| R27 | **A2 camera through Capture One** for the MVP. How the camera physically connects (USB tether today, wireless under exploration) stays open and must not disturb the implementation: everything camera-specific sits behind the camera provider interface, and capture arrival is defined by the editing provider's hook and the capture folder, not by the link. | 2026-10-24 |

## Open owner cards
None. I1 (retouch after publishing) answered Yes on 2026-10-25: R30.

