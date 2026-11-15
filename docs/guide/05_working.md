# 05 — Working: run, test, debug, ship, pipeline, keep the guide current

Exact commands come from master plan §7.2 and `docs/process/flow.md` §4A; this file repeats the
current form and adds the practical recipes. Repo root unless noted. Node 22 / npm 10 (master plan
§7.1: a Homebrew Node 23 in `apps/desktop` broke `npm install`; prefix the Node 22 bin dir if needed).

## Run the app in development
```
cd apps/desktop && npm ci                       # once
cd apps/desktop && npm run tauri -- dev         # Vite on :1420 inside the Tauri window
PSM_DATA_DIR=/tmp/psm-dev npm run tauri -- dev  # scratch data dir (from apps/desktop)
```
First run: `build.rs` builds the stub camera sidecar (needs CMake and Apple clang). Without a support
package installed the camera goes through Capture One; Settings → Camera installs Sony support from
a zip. The Capture One document defaults to `<home>/Pictures/Session Manager` and is created by the
app. Hash routes: `#/`, `#/session/<id>`, `#/health`, `#/settings`.

## Tests, lint, typecheck (flow §4A; one cargo command at a time, never two, never piped into `head`)
| Purpose | Command |
|---|---|
| One phase's behaviour tests | `cargo test -p psm-testkit --test suite <stem>::` (e.g. `p66_approve_on_arrival_and_session_order::`); one test: `… <stem>::<test_name>` |
| One core mechanism | `cargo test -p psm-core <module>::tests` |
| One crate compiles | `cargo check -p <crate>` (also `--tests`) |
| Full Rust suite (the closing gate, once) | `cargo test --workspace` (≈ 8 min warm; 15–17 min right after a relink) |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` |
| Bindings | `cargo test -p psm-desktop export_bindings` (writes `apps/desktop/src/services/bindings.ts`) |
| Frontend | `cd apps/desktop && npm run typecheck` · `npm test` (vitest, ≈ 20 s) · one file: `npx vitest run src/features/<f>/__tests__/<file>` |
| Sidecar (C++) | `tools/sidecar/build.sh --stub --test` (ctest; build dir `target/camera-sidecar/stub`) |
| Timings of a runner session | `tools/pipeline/timeline.sh <.pipeline/runs/…log>` |
"Green" for a phase (flow §5 B4) = suite, clippy, fmt, typecheck, `npm test` all exit 0.
Test naming: `c<criterion><row>_<slug>` per criterion row; harness `psm_testkit::TestApp` (fakes for
Capture One, commerce, secrets, processes, sleep, direct camera; `ManualClock`, `tick(ms)`). The
suite never needs the camera, Capture One, Shopify or the network.

## Where the logs are and what the key lines mean
Log file: `~/Library/Application Support/com.loorenz.sessionmanager/logs/psm.<yyyy-mm-dd>.log`
(or `<PSM_DATA_DIR>/logs/`). Secrets are redacted. Useful greps:
| Line | Meaning |
|---|---|
| `publish.timing item=<sku> masters_ms= outputs_ms= upload_ms= shopify_ms= total_ms= masters_reused=n/m` | one per finished publish item: wait for Capture One masters, the app's own render, upload, Shopify calls (P69) |
| `capture_one.sync …` / `capture_one.variant_seen … after_ms=` | the app asked Capture One to look at the folder; how long until the variant existed (P65) |
| `preview render deferred … attempts=n` / `variant poll deferred` | Capture One not ready yet; retrying with backoff |
| `export failed` / `export stopped waiting for Capture One` | an export (or the export step of a publish) failed or gave up after the 10 min wait |
| `master asked for in advance` / `master prepared in advance` | background master preparation (P69) |
| `ftp.upload started|completed|filed|failed name=… reason=transfer|hash|delivery` (target `psm::camera`) | the FTP receiver's view of one camera upload |
| `camera.event <name> {fields}` | every sidecar event, redacted |
| `camera helper restarted for the new Capture One session` / `camera helper waits: the Capture One session is missing` | P63 document handling |
| `storage.cleared rule=<rule> path=… bytes=…` / `storage.item_cleared item=<sku> …` / `storage.not_cleared item=<sku> missing=n` | 10F clean-up, retention clearing, Shopify verification failure |
| `recipe repaired before rendering` / `render refused: recipe missing; repairing once` | 10F §5 self-repair of `PSM Preview`/`PSM Master` |
| `settings.json invalid; using defaults` / `integration settings invalid; using Export mode` | the file did not parse; health `settings_invalid` |
| `osascript` debug lines with `duration_ms` | every Capture One script call and its time (30 s timeout) |
The Health page (`#/health`) lists every health item with its code and detail; the header dots show
the first failing one. `camera_helper_log` shows the sidecar's recent stderr.

## Debugging recipes
| Symptom | Where to look | What it usually is |
|---|---|---|
| Publish failed `variant_not_found` | log `capture_one.variant_seen`; Capture One shows the photo? | Since P65 the publish waits ("Waiting for Capture One…", up to 10 min) instead of failing; a hard failure now means Capture One never registered the variant: the document is not the one in Settings, or the folder was not synced. Check Health `document_*` items; Retry |
| `export failed: timeout` / publish stuck at Preparing | `publish.timing masters_ms`, `export stopped waiting` | Capture One took too long to render masters (P69 pipelines batches one item ahead and prepares masters when Ready). Free space < `storage.min_free_gb` also blocks master renders. Retry re-runs the failed step only |
| Publish `needs_review` | the row's "Re-frame first: Main 02 …" | a photo's frame changed in Capture One and its composition no longer fits; open Compose, re-frame |
| Camera dot red / `camera_held` | Health detail names the holder; `ownership.rs` | Capture One (through `ptpcamerad`) holds the cable: "Take the camera back" or plug in with auto take-back on; Imaging Edge/Lightroom must be quit |
| Camera never connects over Wi-Fi | Health `camera_auth_refused`, `camera_fingerprint_unconfirmed`/`_mismatch`, `camera_support_not_installed`, `camera_phone_connected` | wrong Access Authentication login (Settings → Camera, saved to Keychain with Save settings); fingerprint pending (confirm in the header card); support not installed; a phone app connected |
| Photos taken but not received (`camera_files_not_received`) | `ftp.upload …` lines; Health detail with `camera FTP: …` | the camera's FTP server points at the wrong host (P68: Settings → Camera step 3 shows this Mac's address; "This Mac's address changed to … — set Host Name on the camera"), wrong password (Show password re-reads the Keychain), user must be `sessionmanager`, port 2121, passive 50000–50009 reachable |
| Fingerprint card keeps coming back | `camera_fingerprint_reject` only declines to pin; the sidecar retries (backlog P51) | confirm it if the camera is yours; rejection is not a block |
| `The shop domain looks wrong: …` / old `http: builder error` | Settings → Shopify | P68 normalises `https://<shop>.myshopify.com/admin/` and `admin.shopify.com/store/<x>` to `<shop>.myshopify.com`; re-save |
| `The Shopify client ID or secret is wrong` / health `integration_token_invalid` | Settings → Shopify; the custom app's credentials | client-credentials grant refused (`09A` MC-23); stray spaces are trimmed on save (P68) |
| Products created without stock tracking | log `item created without stock tracking`, `STOCK_ACCESS_MISSING` text | the custom app lacks `read_locations`/`write_inventory`; add them and reinstall the app |
| `condition_field_missing` | Health detail names the key | the shop has no `custom.damage_report` file-list metafield definition, or the schema was not read yet |
| App starts but nothing arrives | Health `document_not_front`, `capture_folder_not_set`, `hooks_not_installed`; `spool/bad/` | Capture One shows another document; hooks not compiled; a hook payload failed to parse |
| Files deleted by hand in Finder | `RAW missing` / `RAW back` lines | P60: the photo leaves the views and comes back when the file does; nothing breaks |
| Item shows "Archived in Shopify" | `storage.item_cleared` | retention cleared its files (default 4 days after publish); read-only by design (`10F`); pin "Keep files" beforehand to prevent it |

## Release build with the SDK sidecar and distribution to the other Mac
From STATUS entries 2026-11-09 … 2026-11-12. Nothing from the SDK is ever committed or bundled.
1. Build from a clean tree, ideally a **detached worktree with its own target dir** so the running
   pipeline's tree and `binaries/` are untouched (the suite's `p45` test would put the stub back).
2. `tools/sidecar/build.sh --sdk <path to the extracted Sony SDK on the build machine>` → the SDK
   sidecar lands in `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin`. Check the
   size (SDK build ≈ 350 KB, stub ≈ 220 KB on 2026-11-12).
3. `cd apps/desktop && npm ci && npm run tauri -- build --bundles app` →
   `apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app` (ad-hoc signed; the
   worktree's target dir if you changed it).
4. Install locally: move the previous build out of `/Applications`, copy the new one in, launch once.
   Expect Keychain prompts (first use after a re-sign, 3–4 times) and the Automation prompt for
   Capture One.
5. For the work Mac: zip the `.app` (`Session Manager.zip`), transfer it, unzip into `/Applications`.
   Gatekeeper: first launch via right-click Open or System Settings → Privacy & Security → "Open
   Anyway" (ad-hoc signature, not notarised).
6. On the work Mac, once: Settings → Camera → install Sony support from a zip that holds the seven
   runtime dylibs at top level (Sony's own download nests them in `RemoteCli.zip` and is refused);
   generate the FTP password and type it, the user `sessionmanager`, and this Mac's address into the
   camera's FTP server entry; confirm the camera's fingerprint when the card appears; enter the Access
   Authentication login; choose the Capture One session; enter the shop domain and client id/secret.
7. Afterwards put the repo back on the stub (`tools/sidecar/build.sh --stub`) if you built in the
   main tree. Never commit `binaries/`.

## The pipeline in ten lines (authority: `docs/process/flow.md`, `docs/process/orchestrator.md`)
1. The owner wakes an **orchestrator** Claude session (`docs/process/prompts/wake_orchestrator.md`);
   it reads `STATUS.md`, `flow.md`, `git status`, and never implements itself.
2. A phase is a plan file `docs/implementation/plans/P<nn>_<slug>.md` (frontmatter: phase, title,
   difficulty, projection, depends_on; Core mechanisms; criteria rows with boundary/input/outcome/trace).
3. `tools/pipeline/run_phase.sh phase <ID>` (background, never polled) renders the prompts from
   `tools/pipeline/templates/`, commits them, launches the sessions and reads the handoff frontmatter.
4. Sequence: implement (Codex `gpt-6-sol`, or Claude Opus with `PSM_IMPLEMENTER=claude`) → review
   (Claude Opus, `pipeline-reviewer`) → APPROVED, or CHANGES_REQUESTED → one fix → re-review →
   MUST items left → stop, orchestrator decides (`FIX2_AUTHORISED:` or an owner card).
5. Exit codes: 0 approved (the runner makes the `APPROVED <phase>` commit), 10 orchestrator must look,
   20 runner error. Logs in `.pipeline/runs/` (not committed), `.timeline` per session.
6. Blocking only B1–B4; everything else is a note in `docs/implementation/backlog.md`.
7. The approving reviewer writes `docs/implementation/narratives/<phase>.md` for the owner; the
   orchestrator relays it verbatim and logs the stop in `STATUS.md` (newest first).
8. No questions during a phase: assumptions are recorded in the handoff; `BLOCKED` only for a
   plan/book contradiction.
9. Fold-back: semantics → book chapter + changelog; skeleton → master plan; phase-local → plan file.
10. Sessions commit only their perimeter as `CHECKPOINT (not approved):`; only the orchestrator pushes.

## Keeping this guide current
- A phase that changes architecture (a crate, module, flow, table, event, settings key), a rule, or a
  coupling updates the matching file in `docs/guide/` **in the same change**: new flow or module →
  `02_architecture.md`; new or amended rule → `03_rules_and_principles.md`; new coupling or fragile
  spot → `04_change_impact.md`; new command, log line or recipe → `05_working.md`; a product decision
  → `01_product.md` (and its "Book pending updates" table when the book lags).
- Every path written in the guide must exist; prefer pointing to the authority over copying it.
- When the book catches up with the code, delete the row from `01_product.md` §Book pending updates.
- The reviewer checks the guide like the registry: a phase that moves something the guide names
  without updating the guide gets a backlog note.
