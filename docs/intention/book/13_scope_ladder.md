# 13 — Scope ladder

## Must ship (MVP)
Stages S2–S8 as re-expressed in this book: Tauri + Rust packaging spike (chapter 11); camera
provider interface with the direct camera adapter (chapter 10E: sidecar, FTP receiver, capture ledger, camera ownership guard,
camera support package for Sony); editing provider interface
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
Any camera make other than Sony; FTPS for the receiver; live view; camera settings beyond the store destination; capture
groups, bracketing and Pixel Shift (chapter 10E §Out of scope). Second editing
provider. Second commerce integration. Python component. Plugin system. Licensing,
subscriptions, auto-update, notarised installer. Multi-user, cloud, remote access beyond the LAN.
Catalog documents. Video. Theme work on the customer's shop.
