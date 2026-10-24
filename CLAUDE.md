
## Build decisions (owner, updated 2026-10-24)

The authority for product semantics is the **intention book**: start at `docs/intention/intention.md`
(map + status header) and read only the chapters your task touches. Summary of standing decisions:

- **Product:** Session Manager, a photography-workflow desktop app; Shopify is the first *integration*,
  Capture One the first *imaging provider*. Core vocabulary: Item, Session, Photo, Attribute.
- **Runtime:** Python managed by `uv`, FastAPI as transport only, SQLite. No Docker.
- **Shell:** Tauri (macOS first) with the Python backend as a sidecar; validated by an early spike.
  One transport (HTTP + WebSocket) for the desktop window and the future phone remote.
- **Frontend:** React + TypeScript + Vite + shadcn/ui (+ dnd-kit, Motion, Sonner). Design handoff in
  `claude_design/`; rules extracted in `docs/intention/evidence/design_handoff.md`.
- **Backend layout:** industry-standard FastAPI; do not apply the owner's `Application_contracts`.
- **Capture One:** Pro 16.8.6.25 on the dev Mac; dictionary at `docs/reference/capture_one_16.8.6.sdef`.
- **Shopify:** custom dev-app token, provided by the owner before S6. Condition photos → `custom.damage_report`.
- **Repo:** public, `Loorenz-David/photo-session-manager`.
- **Process:** intention-shaper → mechanism-inventory → implementation-planner → pipeline-coordinator.
  Only a RATIFIED intention enters inventory or planning.
