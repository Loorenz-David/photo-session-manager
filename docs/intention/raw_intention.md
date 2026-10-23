# Furniture Photography Pipeline — Project Brief

Handoff brief for the coding agent. Read fully before writing code. Full feasibility study (with sources): https://claude.ai/code/artifact/8f7a26ae-7158-4136-9f35-5afef7793e99

## Goal

A photography workstation for vintage furniture where the photographer only positions, shoots and approves. Everything else is automated:

Camera → Capture One (auto-styled) → review → Session Manager (classify + order) → Shopify (gallery + condition-report metafield).

## Equipment and software

- Camera: Sony ZV-E10 (original, not Mark II), USB-C tethered
- Mac
- Capture One Pro (confirm edition/version; Express cannot tether)
- Shopify (Admin GraphQL API, version 2026-10)

## Verified facts (from feasibility study)

- ZV-E10 tethers natively in Capture One (wired + wireless). Imaging Edge must NOT run at the same time.
- ZV-E10 is NOT in Sony's Camera Remote SDK (only ZV-E10 II). Do not build a custom tether.
- Capture One Next Capture Adjustments apply a style, AI masks (recalculated per image) and AI Crop on arrival.
- Capture One is scriptable via AppleScript/JXA (Mac, Pro): styles, adjustments, layers, color tags/ratings, process recipes.
- `CO_CaptureDone(rawFilePath)` in a Capture One background script fires per capture (works, but undocumented).
- Community MCP exists: capture-one-mcp (byjustinjones), tested on C1 16.8.5. Evaluate before writing our own AppleScript layer.
- NOT scriptable: brush strokes, spot healing, AI Eraser → manual (or future external retouch).
- Shopify: `stagedUploadsCreate` → upload → attach (`productUpdate` media / `fileCreate`); `productCreateMedia` is deprecated. Media processing is async → poll until `READY`/`FAILED`. `productReorderMedia` (moves, zero-based) returns a Job to poll.
- Condition images: upload as Files, then `metafieldsSet` on `custom.condition_images` (type `list.file_reference`). Product-media IDs are rejected there.

## Architecture

The Session Manager is an application over native folders/files. Capture One is the editor on the same files. SQLite stores everything files cannot: product, group, order, status, Shopify IDs.

### Stack

- Backend: Python + FastAPI + SQLite
- UI: local web app (localhost), optionally wrapped with pywebview. Not Tkinter.
- Folder watching: `watchdog` (FSEvents)
- Capture One control: `osascript` (AppleScript/JXA) or capture-one-mcp

### Components

| Component | Responsibility |
| --- | --- |
| Capture Adapter | Receives capture events (hook + folder watcher fallback) |
| Session Manager (UI) | Main interface: current product, product cards, classify, drag-reorder, publish |
| Review Controller | Accept/reject state, read from Capture One color tags/ratings |
| Capture One Adapter | All AppleScript commands, through ONE serial queue |
| Batch Manager | Groups, order, completion per product |
| Shopify Adapter | Lookup, create (Draft), upload, attach, reorder, metafield, verify, retry |
| Edit Engine (interface) | `apply_edit(image, command)`; v1 provider = Capture One. Future: external AI retouch, manual |

### Communication

Capture One → Session Manager:
1. `CO_CaptureDone` background AppleScript → `curl http://localhost:8765/capture` with the RAW path.
2. `watchdog` on Capture + Output folders (backup, and detects exported files).
3. Poll color tags/ratings via AppleScript every few seconds and when a product card opens (no tag-change event exists).

Session Manager → Capture One: `osascript` commands (set tag, apply style, select, process recipe) through a single serial queue.

Thumbnails: a small "preview" process recipe (~1600 px JPEG) runs after each capture; full export via a web recipe on approval.

```
Camera → Capture One ──hook/curl──▶ Session Manager API ──▶ SQLite
              ▲                         │         │
              └──── AppleScript queue ◀─┘         └──▶ Shopify API
         Output folder ──folder watcher──▶ Session Manager
```

## Product decisions

- **Lookup:** scan barcode or SKU → search Shopify. Not found → offer to create the product at publish, as Draft.
- **Add vs replace:** default adds images. Replace is an explicit per-product flag, shown on the confirmation screen.
- **Product card:** title, SKU, status, existing Shopify images, new images by group; opens to the needed Shopify data + link to Shopify admin.
- **Groups:** Gallery (product media), Condition report (metafield), Discard (local only). Condition photos are usually shot in the same set; split by drag-and-drop or quick keys.
- **Order:** owned by the Session Manager (SQLite), set by drag-and-drop. Capture One's sort order is not used. Applied at publish; also written into output filenames, e.g. `BEYO-1042_01_gallery.jpg`.
- **Retouch timing:** not fixed. Products stay queued until published.
- **Publish:** per product or several in a batch. Confirmation screen before any write: "Add X gallery, add Y condition, replace nothing".
- **Human gate:** nothing reaches Shopify without final approval.

## Data model (starting point)

- `products`: id, sku, barcode, shopify_product_id (nullable), title, create_on_publish, replace_flag, status
- `images`: id, product_id, raw_path, preview_path, output_path, review_status (pending/accepted/rejected/approved), group (gallery/condition/discard), position, file_hash, shopify_media_id / shopify_file_id, upload_status
- `edits`: id, image_id, provider, command (JSON), before_values (JSON), created_at — for undo and learning recurring edits
- `jobs`: id, type (preview/export/upload/reorder/metafield/retouch), image_id/product_id, status, attempts, error

Every step must be idempotent: a crash or restart resumes with no lost product associations and no duplicate uploads (check hash + stored IDs + existing product media before upload).

## Engineering principles

1. Reuse Capture One for all imaging; never build image processing.
2. Human approval stays essential.
3. Non-destructive: never modify RAWs.
4. Prefer native integrations; no fragile UI automation.
5. Small, independently testable milestones.
6. Verify capabilities before designing around them.
7. Build for later AI editing now (Edit Engine interface, lineage, command log, job queue), but do not ship AI editing in v1.

## Milestones

- **M0 – Tether PoC (no code):** camera → Capture One over USB; 20 shots, 0 lost; measure shot-to-preview time.
- **M1 – Auto-processing (no code):** house style + AI masks + AI Crop in Next Capture Adjustments.
- **M2 – Event hook:** `CO_CaptureDone` → local API, folder watcher fallback; every shot logged exactly once.
- **M3 – Orchestrator skeleton:** FastAPI + SQLite; select product; stamp captures; survive restarts.
- **M4 – Review loop:** read tags/ratings; accepted → preview/export recipes.
- **M5 – Product batch + E1:** groups, drag order, product cards.
- **M6 – Shopify read-only:** lookup by SKU/barcode, show existing media.
- **M7 – Shopify upload (dev store first) + E2:** create-as-Draft, staged upload, attach, reorder, condition metafield, verify, retries.
- **M8 – Hardening:** logs, alerts, state backup.
- Future: **E3** text edit commands, **E4** voice input, **E5** external AI retouch.

## Open questions

- Capture One edition/version in use?
- Does that version's AppleScript dictionary expose color tags/ratings and the capture folder per Session?
- Does `custom.condition_images` exist in Shopify, and does the theme render it?

## How to work

- Start at M2 only after M0/M1 are confirmed working by the photographer.
- Use a Shopify development store until M7 is proven.
- Propose the plan for each milestone before coding it.
