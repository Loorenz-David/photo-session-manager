
## Build decisions (owner, 2026-10-23)

- **Runtime:** native macOS app managed by `uv` (pinned Python + `uv.lock`). No Docker. One process on localhost.
- **Launcher:** double-click `.app` built from AppleScript source in `launcher/`; it installs/syncs the environment and opens the UI. Installing git, uv and Node on the work Mac is acceptable.
- **Backend:** FastAPI, industry-standard layout. Do not apply the owner's `Application_contracts` set to this repo.
- **UI:** React + TypeScript + Vite with shadcn/ui. Design comes from Claude Design, adapted by the owner to the component structure we define.
- **Capture One:** Pro 16.8.6.25 on the dev Mac. Dictionary dump at `docs/reference/capture_one_16.8.6.sdef`.
- **Shopify:** custom dev app token (not legacy/embedded), supplied by the owner when M6 starts. Search fields for lookup: barcode and SKU.
- **Repo:** public GitHub repo `photo-session-manager`.
- **Process:** pipeline skills in order — intention-shaper → mechanism-inventory → implementation-planner → pipeline-coordinator. Intention lives at `docs/intention/intention.md`.
