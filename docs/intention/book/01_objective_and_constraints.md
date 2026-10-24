# 01 — Objective and hard constraints

**Outcome.** A photographer positions the piece, presses the shutter, and sorts. Session Manager
turns that into a completed photography session: adjusted photos in the chosen order, split into
main images and condition images, with the item's information filled in, ready to export locally
or to send to the connected platform. For the first customer the platform is Shopify, and the
finished product page shows the gallery in the chosen order and the condition photos in the
condition metafield.

**Hard constraints (never traded for scope).**

| # | Constraint | Why |
|---|---|---|
| H1 | Nothing is written to an external platform without an explicit, recorded confirmation by the photographer. | A wrong image on a live listing is a customer-facing defect. |
| H2 | RAW files are never modified, moved or deleted by Session Manager. "Delete" in the UI hides a photo; bytes stay on disk. | Non-destructive; the capture folder belongs to the editing provider. |
| H3 | No pixel processing in this repo. Styling, crop, masks and rendering are the editing provider's job (Capture One in the MVP). Copying an embedded JPEG out of a RAW is allowed; decoding is not. | Reuse; keeps the codebase small. |
| H4 | Every step is idempotent. After a crash at any point, resuming yields the same end state: no lost associations, no duplicate uploads. | Single operator, no engineer on site. |
| H5 | No UI automation of other apps. External applications are driven only through interfaces they publish (Capture One: its AppleScript dictionary). | Breaks on every update otherwise. |
| H6 | The core photography workflow runs with no commerce integration configured. Publishing is optional and last. | Different customers have different systems after photography. |
| H7 | Device-, application- and platform-specific concepts (a camera protocol, Capture One variant ids, Shopify products, metafields, media IDs) never appear in core modules or core tables. They live in adapters and link tables. | A second customer, camera or editor must not require a core rewrite. |
| H8 | Business logic is independent of Tauri, of any HTTP server and of the UI. Tauri commands and events, and any later server, are thin interfaces to application services. | Commercial packaging and the phone remote must not force a rewrite. |
