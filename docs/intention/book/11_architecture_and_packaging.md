# 11 — Architecture and packaging

## Processes
One Python process (FastAPI + in-process job worker + provider queue + watcher) and one UI. In
development the UI runs in a browser from Vite and the backend from `uv run`. In the packaged app
Tauri hosts the UI in its webview and starts/stops the Python backend as a **sidecar**.

## One transport (decision C4)
The UI talks to the backend over HTTP (commands, queries) and one WebSocket (state events) in
both hosts. Tauri does lifecycle and window only; no Tauri commands carry business operations.
The phone remote (below) is the same React app over the same transport.

## Layers in the backend (H8)
```
api/            FastAPI routers + WebSocket: parse, call a service, serialize. No logic.
core/           domain models, services (commands and queries), events, job queue
attributes/     schema, presets, values, provenance
integrations/   interface + capabilities; shopify/
imaging/        interface (capture adapter, edit engine); capture_one/ with jxa/ scripts
storage/        SQLite, migrations, app directories
```
Services receive plain inputs and return plain results; they are callable from tests without
FastAPI. Frontend: `services/` holds typed functions generated from the OpenAPI schema plus the
WebSocket event types; components never call `fetch` directly.

## Repository layout
```
backend/            Python package `session_manager` (layout above), tests/
apps/desktop/       React app (src/) and Tauri shell (src-tauri/)
docs/               intention book, evidence, checklists, plans
claude_design/      design handoff (reference, read-only)
```

## Tauri sidecar spike (early stage, before UI features)
Package a minimal Tauri app with the Python backend as a sidecar and verify: sidecar start/stop
with the window; `/health` reachable from the webview; macOS **Automation** permission prompt for
Apple Events to Capture One attributed to the app bundle and remembered; app data directory
writable; the Python sidecar built as a single executable (PyInstaller or equivalent) that still
finds `osascript` and the bundled JXA scripts. Outcome recorded in evidence before any UI phase.

## Phone remote (design pending)
Same app, served on the Mac's LAN address, opened on a phone via a QR code shown on the desktop,
added to the home screen. Needs a phone layout (not in the current handoff) and LAN binding with
a pairing token. Scope position: chapter 13.

## Not in the MVP
Signed installer, notarisation, auto-update, licensing, multi-window. Code signing is ad-hoc.
