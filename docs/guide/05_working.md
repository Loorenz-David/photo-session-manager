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

Research in the dev app: connect a shop, enter the two keys in Settings → Research (they go to the
Keychain), tick fields under "Fields research fills" (empty on a fresh install: research does nothing
until then), then press "Arranged — next" in a session with a Main photo. Runs spend real money up
to the Settings → Research cost limits. The research narratives' try-outs use a scratch folder,
`PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`. That was a **rule only until research
was merged into `main`** (research master plan §3 "Scratch data only"; merged 2026-11-15, PR #1), so
it is no longer required; its reason still holds for any build made *before* the merge: such a build
does not know the publish step `held` or the research columns, so never point it at a data dir a
research build has written. A scratch folder stays the sensible choice for try-outs.

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
| Research rules (inline unit tests) | `cargo test -p psm-research --lib rules::<module>` (e.g. `rules::confidence`) |
| Research engine, one test file | `cargo test -p psm-research --test engine <stem>::` (stems = files in `crates/research/tests/engine/`, e.g. `chosen_fields::`, `stages::`, `resume::`) |
| Whole research crate | `cargo test -p psm-research` |
| Research tool crate | `cargo test -p psm-research-tool` (one file: `… <stem>::`, e.g. `u4_accept::`; stems in `crates/research-tool/tests/tool/`) |
| Research app seams | `cargo test -p psm-testkit --test suite research::<stem>::` (stems in `crates/testkit/tests/suite/research/`, e.g. `research::r11_arranged::`); harness `TestApp::with_research()` + `research_test_settings()`; W3 writer seams `research::w3_title::`, `research::w3_writer_wiring::` (the writer's default model `gpt-6.1-sol` needs a price for `FakeModel::ID` added to the research settings) |
| Research UI | `npx vitest run src/features/form/__tests__/r6_research_panel.test.tsx` · `src/features/settings/__tests__/r13_research_fills.test.tsx` · `src/features/publish/__tests__/r6_research_publish.test.tsx` (from `apps/desktop`) |
| Research-only lint | `cargo clippy -p psm-research --all-targets -- -D warnings` (same with `-p psm-research-tool`) · `cargo check --workspace` |
| Sidecar (C++) | `tools/sidecar/build.sh --stub --test` (ctest; build dir `target/camera-sidecar/stub`) |
| Timings of a runner session | `tools/pipeline/timeline.sh <.pipeline/runs/…log>` |
"Green" for a phase (flow §5 B4) = suite, clippy, fmt, typecheck, `npm test` all exit 0.
Test naming: `c<criterion><row>_<slug>` per criterion row; harness `psm_testkit::TestApp` (fakes for
Capture One, commerce, secrets, processes, sleep, direct camera; `ManualClock`, `tick(ms)`). The
suite never needs the camera, Capture One, Shopify or the network.
Research (flow §10; research master plan §7.2): a change inside `crates/research` or the tool crate
only runs that crate's tests, its lint and `cargo check --workspace`, **not** the app suite; a change
to app files runs the touched `research::` / `p…::` stems and the full gate once at the end. Research
tests are named after their criterion row (`a27_c3a_<slug>`); no research test reaches a provider
(engine: `psm_research::fakes`; tool: the `Stub` HTTP seam in `tests/tool/main.rs`).

## Where the logs are and what the key lines mean
Log file: `~/Library/Application Support/com.loorenz.sessionmanager/logs/psm.<yyyy-mm-dd>.log`
(or `<PSM_DATA_DIR>/logs/`). Secrets are redacted. Useful greps:
| Line | Meaning |
|---|---|
| `publish.timing item=<sku> masters_ms= outputs_ms= upload_ms= shopify_ms= total_ms= masters_reused=n/m` | one per finished publish item: wait for Capture One masters, the app's own render, upload, Shopify calls (P69) |
| `publish.seo item=<publish item id> written=<bool>` | the publish's search-texts step (W5): `written=false` when the shop already held the frozen texts (or none were frozen, then no line); `verified=false` added when the read-back differed |
| `capture_one.sync …` / `capture_one.variant_seen … after_ms=` | the app asked Capture One to look at the folder; how long until the variant existed (P65) |
| `preview render deferred … attempts=n next_s=` / `variant poll deferred` | Capture One not ready yet; retrying with backoff. A16: logged on attempt 1, 2, 4, 8…; a photo Capture One does not list is asked again after up to 10 minutes |
| `export failed` / `export stopped waiting for Capture One` | an export (or the export step of a publish) failed or gave up after the 10 min wait |
| `master asked for in advance` / `master prepared in advance` | background master preparation (P69) |
| `ftp.upload started|completed|filed|failed name=… reason=transfer|hash|delivery` (target `psm::camera`) | the FTP receiver's view of one camera upload |
| `arrival.photo_added file= folder= session= photo= source=capture|import filed_to_row_ms=` (target `psm::arrival`, A10) | a file became a photo; ms from the file's last write (≈ "filed") to the photo row. More than a few seconds means arrival waited |
| `arrival.preview_ready file= folder= session= photo= row_to_preview_ms=` (`psm::arrival`, A10) | the photo's first adjusted preview from Capture One; ms since its row. Long values: Capture One had no variant (wrong collection, busy) |
| `arrival.given_up reason=stability_timeout event_id=` (warn, `psm::arrival`, A10) | a seen file kept changing or stayed missing for 120 looks; it is not added (a launch scan may add it later) |
| `arrival.rescan reason=watcher_dropped_events` (`psm::arrival`, A10) | macOS dropped folder events; the capture folders were scanned instead |
| `capture_one.switch reason=open_session|leave_session|photo_arrived from= to= folder= ms= result=ok|error: …` (`psm::capture_one`, A10) | one Capture One collection switch (opening a session, back to the list, a photo arriving in another folder) and its outcome |
| `capture_one.switch_step reason= to= folder= step=ensure|bring_document_to_front|inspect_document|set_capture_folder|set_current_collection|collection_switched ms= result=` (`psm::capture_one`, A10) | each Capture One call of a switch; the failing step and its error text |
| `capture_one.switch_scripts reason= to= scripts=` (`psm::capture_one`, A20) | how many scripts one switch sent to Capture One (interactive lane); it should stay flat however many sessions exist (~9 once verified) |
| `capture_one.ensure checks=cached|full instance=` (`psm::capture_one`, A20) | whether this ensure skipped the recipe and hook checks (already verified for this Capture One process) or ran them all |
| `capture_one.switch_skipped request= reason=a_newer_request_follows` (`psm::capture_one`, A10) | a switch the person had already overtaken by opening another session; not applied |
| `arrival.target session= folder= from=` (`psm::arrival`, A16) | where the camera's photos are filed changed (a session opened or left); written at the request, before Capture One switches |
| `capture_one.switch_follow_up session= photos= waited_ms= ms= result=` (`psm::capture_one`, A16) | after a switch, the opened session's previews looked at again on the background loop: how many photos, how long after the switch, how long it took |
| `capture_one.style applied\|skipped\|failed raw=<folder>/<file> style= reason=` (`psm::capture_one`, A16) | the Capture One Style for a new photo: applied, skipped (`style_not_listed`) or failed (the error) |
| `photos.moved|copied file= from_session= from_folder= to_session= to_folder= files=follow|shared` (`psm::capture_one`, A10, A21) | one photo moved or copied between sessions (the rows); a moved photo's files follow (next line), a copy shares the RAW, which stays |
| `photos.files_moved file= session= from_folder= to_folder= sidecars= caches=` (`psm::capture_one`, A21) | a photo's RAW, Capture One's files of it (settings, masks, proxies, thumbnails) and the app's caches of it moved into its session's folder (after a move, undo/redo, Assign product, or "Fix photo files location") |
| `photos.files_not_moved file= session= to_folder= reason=` (warn, `psm::capture_one`, A21) | a photo left where it is: a same-named file in the target, a file that could not be moved (what moved was moved back), its record not updated |
| `photos.files_fixed session= moved= sidecars= left=` (`psm::capture_one`, A21) | "Fix photo files location" ran for a session |
| `capture_one.folder_synced folder= reason=photo_files_moved|photo_not_listed result=` (`psm::capture_one`, A21) | Capture One asked to re-read a folder: files moved in or out, or a photo on disk it did not list (its list of a folder it is not showing is stale; at most once a minute per folder) |
| `session.kept_as_published session= denied_late_photos=` (`psm::arrival`, A10) | "Keep as published" on a session that changed after publishing |
| `session.deleted session= sku= photos= files_trashed= files_missing= folder_trashed= research_runs= capture_one_result=left_session\|not_needed\|not_reached (…)` (`psm::session`, A17) | "Delete session": files moved to the macOS Trash, files already gone, whether the capture folder went too, and what Capture One did first. `session.delete … kept` lines say why a RAW or folder stayed (another session uses it, outside the capture folder, other files in the folder) |
| `search_window.opened session= photo=` (`psm::search`, A12) | the "Search this photo" window opened |
| `search_window.lens_route step=start\|photo_posted …` / `result=consent\|uploading\|ok\|fallback why=` (`psm::search`, A12) | the Google Lens route: photo prepared, posted from Google's page, and its outcome; `fallback` = the window showed its own list (why: no results in 30 s, Google's page did not open, a "confirm you are a person" page) |
| `search_window.navigation session= host= kind=google_consent\|google_sorry\|google_results\|lens_upload\|google_other\|other` (`psm::search`, A13) | every page the window's browser finished loading, by host and kind only (no path details, query or text) |
| `search_window.lens_route step=photo_posted_again attempt=2 session=` (`psm::search`, A13) | Google lost the first post (cookie page, home page) and the photo was posted once more; `why="Google showed a page without results for the photo"` = it was lost twice |
| `search_window.navigation_refused session= scheme= host= why=empty\|local_address\|not_a_web_page` (`psm::search`, A12; host and why A14) | the page webview was kept from loading something that is not a public web page |
| `search_window.new_window session= host= result=opened_here\|ignored why=` (`psm::search`, A14) | a page asked for a new window (a link or form aimed at one that the page script did not catch); `opened_here` = loaded in the same browser area. Pop-ups opened from script never reach the app: the page script loads them in place, so they show as an ordinary `search_window.navigation` |
| `search_window.text_to_field_undone session= key=` (`psm::search`, A14) | the toast's Undo after "Put in" gave the field back what it held |
| `search_window.page_saved item= host=` / `search_window.text_to_field session= key= chars=` (`psm::search`, A12) | a page saved; selected text put in a field (no text in the log) |
| `search_window.ask_ai session= run= result=ok suggested=n\|failed cost_micro_usd=` / `search_window.serpapi_search …` (`psm::search`, A12) | Ask AI on selected text, or the window's own visual search, with its cost; the call itself is in the research record (`browse` run) |
| `writer.requested session= origin=next_to_review\|all_checked\|write_texts\|answer queued= reason=` (`psm::writer`, W2/W3) | a request for the session's texts and whether it queued a `write` run (reason = the `WriterBlock` code when refused; empty when an automatic origin did nothing) |
| `writer.started run= session=` / `writer.post_check run= attempt= hits=` / `writer.finished run= written= flagged= regenerated= findings= questions= cost_micro_usd=` (`psm::writer`) | one writer run: its start, each post-check, its end with what it wrote and what it cost |
| `camera.event <name> {fields}` | every sidecar event, redacted |
| `camera helper restarted for the new Capture One session` / `camera helper waits: the Capture One session is missing` | P63 document handling |
| `storage.cleared rule=<rule> path=… bytes=…` / `storage.item_cleared item=<sku> …` / `storage.not_cleared item=<sku> missing=n` | 10F clean-up, retention clearing, Shopify verification failure |
| `recipe repaired before rendering` / `render refused: recipe missing; repairing once` | 10F §5 self-repair of `PSM Preview`/`PSM Master` |
| `settings.json invalid; using defaults` / `integration settings invalid; using Export mode` | the file did not parse; health `settings_invalid` |
| `osascript` debug lines with `duration_ms` | every Capture One script call and its time (30 s timeout) |
| `research call failed run_id= stage= class= attempt= error=` | one failed provider call of a run (class: offline, transient, key, quota, request, schema, refusal); the text is redacted |
| `research not available` / `research turn deferred` | the engine could not be built at start (the app runs without research, commands answer `research_unavailable`) / one runner turn failed and is retried |
| `a secret value was found in a research call record and redacted` (error) | a key value appeared in a provider exchange; the record has `redaction_hit` set |
The research record itself is not in the log: runs, steps and calls are rows (`research_runs`,
`research_steps`, `research_calls`), shown in the form's research panel, and leave the app through
Settings → Research → "Export research" (a zip the tool's `report` and `replay` read).
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
| Research does not run | the research panel's sentence (`rules::run_rules::first_failing`, `runs.rs::status`) | first failing prerequisite: no shop connected · model key or visual-search key missing · "No price is set for …" (a stage model without a price row) · no fillable field · **"Choose the fields research fills in Settings → Research"** (`fill_keys` empty: the fresh-install state) · day limit · offline; "Waiting for Capture One" = adjusted previews not there yet. "Arranged — next" also queues nothing when the session has no Main photo or nothing changed since the last run |
| Research ran but a field stayed empty | the panel's steps (`not_determined`), the field's class | the key is not in "Fields research fills", is a never-written type (Product step, money, rich text, …), the person cleared that value before, or the evidence did not reach a decision |
| Item stuck at "Waiting for you · n conflicts" | the publish popover's conflict list | a confirmed item with unresolved conflicts is `held` by design (MC-A10): resolve each (Keep previous / Use suggestion / Edit) or cancel |

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

## Research tool (`crates/research-tool`; usage text: `USAGE` in `crates/research-tool/src/lib.rs`)
```
cargo run -p psm-research-tool -- <command> …            # or the built binary:
apps/desktop/src-tauri/target/debug/psm-research <command> …
```
| Purpose | Command (after `--`) |
|---|---|
| Acceptance measurement: the real engine and real providers over the test pieces, each run exported | `accept --pieces docs/test-items/beyo_test_samples_1.json --data <scratch dir outside the repo> --ceiling-usd <n> [--only 6,12,28,33] [--settings docs/test-items/accept_settings_owner.json]` |
| Score those runs against the answer key | `score-accept --data <the same dir>` → `<data>/scores.json` (`--pieces <file>` for another pieces file) |
| Summarise an export (from `accept`'s `<data>/exports/`, or the app's "Export research" zip) | `report <export folder or zip> [--json]` |
| Re-send one recorded stage of one run, optionally changed | `replay <export> --run <id> --stage <stage> [--model <m>] [--effort <e>] [--instructions <file>] [--out <dir>]` |
| R1's probes and benchmarks | `probe <id>`, `bench-search`, `bench-models` (each needs `--out <dir> --ceiling-usd <n>`); `score-search`, `score-models`, `recost`, `fixtures <dest>` (each needs `--out <dir>`) |
- **`accept`, `replay`, `probe` and the two `bench-*` commands spend money on the owner's accounts**
  (`report`, `score-*`, `recost`, `fixtures` send nothing). Run them only
  when a plan or the owner says so, with the ceiling the plan names (research master plan §3
  "Spending ceilings"). `accept` refuses without `--ceiling-usd`; its ceiling counts per data folder;
  **`replay` has no ceiling**.
- **Keys.** The tool reads `OPENAI_API_KEY` and `SERPAPI_API_KEY` from its environment only, hands
  them to an in-memory store (never the Keychain); recordings and exports are scanned for them
  before they are written. They come from
  the owner's env file outside the repo (research master plan §7.6), which a launching script loads
  into the environment; a session never reads that file, prints a value or writes one anywhere.
  `report` and `score-accept` need no key.
- `--settings <file>` replaces keys of the tool's scratch research settings
  (`accept::scratch_settings`, which chooses every field of its stand-in shop);
  `docs/test-items/accept_settings_owner.json` is the owner's set-up (22 `fill_keys`, extra shop
  addresses, the detailed-condition table). `--only` takes piece numbers.
- `accept` writes only under `--data` and never to a shop (its shop is a read-only stand-in). Past
  runs and their findings: `docs/intention/evidence/research_providers.md`; how they were launched:
  `docs/implementation/STATUS_research.md`.

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

## Research phases (authority: `docs/process/flow.md` §10; `docs/implementation/master_plan_research.md` §0, §5, §7)
- A research **milestone is one runner phase**: `tools/pipeline/run_phase.sh phase R13_research_chosen_fields`
  (ids in `docs/implementation/phases_research.txt`, R1–R13, all approved). `run_phase.sh all` reads
  `phases.txt` only, so research milestones are started one by one.
- Runner switches (`tools/pipeline/run_phase.sh` header): `PSM_IMPLEMENTER=claude` (Claude instead of
  the default Codex implementer), `RESUME=1` (skip the implement and first review sessions whose
  handoffs already say `state: DONE`, e.g. after a usage limit cut a later session), `DRY_RUN=1`.
- The plan `docs/implementation/plans/R<n>_<slug>.md` is made of `## Unit` sections worked in order,
  one `CHECKPOINT (not approved):` commit per unit; its **Tracker line** is the milestone's state and
  its **Closing gate** names the commands (they replace the template's). One review, one fix budget,
  one narrative (`narratives/R<n>_<slug>.md`) per milestone.
- Criterion rows keep the id of the plan they were first written in (`A07·C1(a)`; R11 `A25·…`, R12
  `A26·…`, R13 `A27·…`); `docs/implementation/research_row_mapping.md` says where each row is.
- Reviewer notes go to `docs/implementation/backlog_research/<milestone id>.md`; the stop log is
  `docs/implementation/STATUS_research.md`. Semantics decided after ratification are recorded in the
  08B changelog and, until the chapters are rewritten, in the plan's "What this plan supersedes in
  the book" section: a new research plan that changes behaviour needs both.
- `[manual]` measurement rows (paid, real providers) are the orchestrator's or the owner's, never an
  implementer session's and never a suite's.

## Keeping this guide current
- A phase that changes architecture (a crate, module, flow, table, event, settings key), a rule, or a
  coupling updates the matching file in `docs/guide/` **in the same change**: new flow or module →
  `02_architecture.md`; new or amended rule → `03_rules_and_principles.md`; new coupling or fragile
  spot → `04_change_impact.md`; new command, log line or recipe → `05_working.md`; a product decision
  → `01_product.md` (and its "Book pending updates" table when the book lags).
- Research changes update the research sections: `01_product.md` §Research assistant, `02_architecture.md`
  §Research, `03_rules_and_principles.md` §Research hard rules, `04_change_impact.md` §Research,
  this file's §Research tool and §Research phases. When 08A/08B are rewritten to absorb the
  changelog and the "supersedes" sections, delete the three research rows from `01_product.md`
  §Book pending updates and the "later wins" wording in `README.md`.
- Every path written in the guide must exist; prefer pointing to the authority over copying it.
- When the book catches up with the code, delete the row from `01_product.md` §Book pending updates.
- The reviewer checks the guide like the registry: a phase that moves something the guide names
  without updating the guide gets a backlog note.
