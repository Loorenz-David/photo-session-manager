# 13 — Scope ladder

## Must ship (MVP)
Stages S2–S8 as re-expressed in this book: Tauri + Rust packaging spike (chapter 11); camera
provider interface with the camera-through-Capture-One adapter (A2); editing provider interface
with the Capture One adapter, capture hook + watcher, live adjusted preview; sessions, items,
photos with review, undo, order, Original/Adjusted; color-tag write mirror; unassigned, move,
copy, import; attribute schema, presets, form with provenance UI; local export; Shopify adapter
with the capability set in chapter 09; publish runs with resume and retry; notifications; health;
settings screen; packaged app (ad-hoc signed).

## Only if cheap
Color-tag read-back from Capture One (D2: owner expects to sort only in the app). Capture trigger
from the UI (`camera.capture`). Rating display (read-only). Keychain for tokens. Phone remote
**layout** once a design exists; services already support it. Dark mode (tokens exist).

## Explicitly deferred (non-goals for the MVP)
The AI research agent and description writing (chapter 08 ships its data model and UI states).
Direct camera adapter (Sony Camera Remote SDK or PTP) and any other camera make. Second editing
provider. Second commerce integration. Python component. Plugin system. Licensing,
subscriptions, auto-update, notarised installer. Multi-user, cloud, remote access beyond the LAN.
Catalog documents. Video. Theme work on the customer's shop.
