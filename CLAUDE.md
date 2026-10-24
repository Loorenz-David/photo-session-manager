
## Build decisions (owner, updated 2026-10-24, Rust revision)

The authority for product semantics is the **intention book**: start at `docs/intention/intention.md`
(map + status header) and read only the chapters your task touches. Summary of standing decisions:

- **Product:** Session Manager, a photography-workflow desktop app. Three integration families, one
  adapter each in the MVP: camera (Sony ZV-E10, through Capture One's tether), editing (Capture One),
  commerce (Shopify). Core vocabulary: Item, Session, Photo, Attribute. Nothing device-, app- or
  platform-specific in core.
- **Runtime:** Tauri (macOS first) with **Rust as the backend**: services, domain, SQLite, jobs,
  provider orchestration, native operations. No FastAPI, no uv, no Docker. Python is optional and
  absent from the MVP (sidecar only for a demonstrated need).
- **Boundaries:** frontend / core domain / application services / integrations / infrastructure.
  Tauri commands and events are thin; business logic never depends on Tauri. The phone remote
  (post-MVP) is an embedded HTTP + WebSocket server over the same services.
- **Frontend:** React + TypeScript + Vite + shadcn/ui (+ dnd-kit, Motion, Sonner), typed bindings
  generated from Rust commands. Design handoff in `claude_design/`; rules in
  `docs/intention/evidence/design_handoff.md`.
- **Capture One:** Pro 16.8.6.25 on the dev Mac, driven only through its AppleScript dictionary
  (`docs/reference/capture_one_16.8.6.sdef`) via `osascript` from Rust.
- **Shopify:** custom dev-app token, provided by the owner before S6. Condition photos → `custom.damage_report`.
- **Repo:** public, `Loorenz-David/photo-session-manager`. The owner's `Application_contracts` are not applied.
- **Process:** `docs/process/flow.md` is the project's process authority (wins over the global pipeline
  doctrine where they differ): Rust/Tauri, no tester role, no mutation regime, behaviour tests at
  boundaries, one automatic fix then triage, owner narratives per phase. Sessions are launched by
  `tools/pipeline/run_phase.sh`. The intention is RATIFIED (2026-10-24); start at `docs/implementation/narratives/`.
