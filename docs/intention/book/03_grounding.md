# 03 — Grounding

## Repository
Docs only at shaping time: `CLAUDE.md`, this book, evidence, two photographer checklists in
`docs/checklists/`, the Capture One dictionary in `docs/reference/`, and the design handoff in
`claude_design/`. No code, no contract system, no architecture graph. The pipeline charter's
standing rules are the baseline.

## Stack (owner decisions 2026-10-23/24; backend revised 2026-10-24)
- **Application:** Tauri, macOS first. **Rust is the backend**: application services, domain,
  SQLite persistence, background jobs, provider orchestration, native macOS operations and
  lifecycle live in the Tauri process. No separate server process (chapter 11).
- **Python:** optional and absent from the MVP. Added as a sidecar only when a concrete need is
  demonstrated (local vision, image processing, a specialised model).
- **Frontend:** React + TypeScript + Vite + shadcn/ui, with dnd-kit, Motion and Sonner as the
  design handoff names them. Talks to Rust through typed Tauri commands and events (chapter 11).
- **Editing provider:** Capture One Pro 16.8.6.25, driven through its AppleScript dictionary by
  JXA via `osascript`, spawned from Rust on one serial queue. `capture-one-mcp` not used (R11).
- **Camera:** Sony ZV-E10 over USB-C, tethered by Capture One in the MVP (R27); the physical link may change.
- **Folder watching:** FSEvents from Rust, as fallback and reconciler only.
- **Commerce integration:** Shopify Admin GraphQL API, custom dev-app token, provided before S6.
  Metafield definitions were read from the shop on 2026-10-24 (`claude_design/reference/`).

## Machines
Dev Mac: camera, cable, Capture One Pro, Rust toolchain, Node, Xcode command line tools. The work
Mac receives the built `.app` (ad-hoc signed) or builds from source with the same toolchain.
Both macOS 26.5.

## Evidence
- Capture One dictionary: `docs/intention/evidence/capture_one_16.8.6.md`.
- Design handoff rules: `docs/intention/evidence/design_handoff.md`; pictures and prototype in
  `claude_design/`.
- Shopify: evidence doc to be written when the dev store is available.
