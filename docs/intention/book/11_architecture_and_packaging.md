# 11 — Architecture and packaging

## Processes
One native process: the Tauri application. Its Rust side is the backend (application services,
domain, persistence, jobs, provider queues, watcher); its webview hosts the React UI. In
development the UI runs from Vite inside the Tauri dev window. There is no second backend
process. **Python is optional**: introduced only as a Tauri sidecar when a concrete need is
demonstrated (local vision, image processing, a specialised model); none exists in the MVP.

## Frontend ↔ backend (card A1)
The window calls typed Tauri commands and subscribes to Tauri events. Command and event types are
generated from the Rust signatures into TypeScript (tool chosen by the planner); components never
call `invoke` directly, they call `services/`. The phone remote (chapter 04 S-M, N3) is served
later by an HTTP + WebSocket server embedded in the same Rust process, exposing the same
application services; under H8 it is an addition, not a change. If A1 chooses otherwise, the
desktop window also uses that server from the start.

## Layers (owner memo, 2026-10-24) and the dependency rule
```
frontend         React/TS/shadcn; services/ (typed bindings); no business rules
core             domain: Item, Session, Photo, Attribute, schemas, states, publish/export jobs
services         session, photo, file, jobs, integration coordination, AI orchestration (later)
integrations     camera providers, editing providers, commerce/publishing adapters
infrastructure   Tauri commands/events, SQLite, filesystem, OS APIs, task execution, config
```
Rule: core depends on nothing above it. Services depend on core and on the *interfaces* (traits)
of integrations and infrastructure, never on Capture One, Shopify, SQLite or Tauri types.
Adapters and infrastructure implement those traits. Tauri commands are thin: parse, call a
service, serialize. Services are callable from tests with in-memory implementations. These are
boundaries, not a framework: the MVP implements only what it needs inside them.

## Repository layout (proposal; the planner fixes crate names)
```
apps/desktop/src/          React app
apps/desktop/src-tauri/    Tauri app: commands, events, lifecycle, config (infrastructure edge)
crates/core/               domain + services (no Tauri, no SQL, no HTTP)
crates/integrations/       camera/, editing/capture_one/ (with jxa/ scripts), commerce/shopify/
crates/infra/              SQLite storage + migrations, fs + watcher, job runner, osascript runner
docs/                      intention book, evidence, checklists, plans
claude_design/             design handoff (reference, read-only)
```
A Cargo workspace. Fewer crates are acceptable if module boundaries keep the dependency rule.

## Packaging spike (first stage, before UI features)
Build the Tauri app and verify on this Mac: macOS Automation permission for Apple Events to
Capture One, attributed to the bundle and remembered (`NSAppleEventsUsageDescription`); spawning
`osascript` from the bundle; FSEvents watch on the capture folder; SQLite in the app data
directory; the capture-done hook reaching the app (VERIFY-8); an ad-hoc-signed `.app` copied to
the work Mac launches (Gatekeeper right-click Open). Outcome recorded in evidence before any UI
phase.

## Phone remote (design pending; N3: after the desktop MVP)
Same React app with a phone layout, served on the LAN by the embedded server, opened via a QR
code, pairing token. Nothing in core or services changes.

## Not in the MVP
Signed or notarised installer, auto-update, licensing, multi-window, Python component.
