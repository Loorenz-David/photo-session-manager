# 03 — Grounding

## Repository
Docs only at shaping time: `CLAUDE.md`, this book, evidence, two photographer checklists in
`docs/checklists/`, the Capture One dictionary in `docs/reference/`, and the design handoff in
`claude_design/`. No code, no contract system, no architecture graph. The pipeline charter's
standing rules are the baseline.

## Stack (owner decisions, 2026-10-23/24)
- **Backend:** Python managed by uv (pinned interpreter, lockfile), FastAPI as transport, SQLite.
  Business logic in plain Python services, independent of FastAPI.
- **Frontend:** React + TypeScript + Vite + shadcn/ui, with dnd-kit, Motion and Sonner as the
  design handoff names them. Talks to the backend through typed service interfaces over HTTP and
  WebSocket (see chapter 11).
- **Shell:** Tauri, macOS first, running the Python backend as a sidecar. Validated by an early spike.
- **Imaging provider:** Capture One Pro 16.8.6.25, driven by JXA through `osascript`, one serial
  queue. `capture-one-mcp` not used (decision log, chapter 15).
- **Folder watching:** watchdog on FSEvents, as fallback and reconciler only.
- **Integration:** Shopify Admin GraphQL API, custom dev-app token, provided by the owner before S6.
  Reference metafield definitions were read from the shop on 2026-10-24 (`claude_design/reference/`).

## Machines
Development and testing on the dev Mac (camera, cable, Capture One Pro present). The work Mac
clones the repo and runs the app. Both macOS 26.5. Installing git, uv and Node on the work Mac
is acceptable.

## Evidence
- Capture One dictionary: `docs/intention/evidence/capture_one_16.8.6.md`.
- Design handoff rules: `docs/intention/evidence/design_handoff.md`; pictures and prototype in
  `claude_design/`.
- Shopify: evidence doc to be written when the dev store is available.
