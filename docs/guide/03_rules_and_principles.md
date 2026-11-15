# 03 — Rules and principles (never break these)

Each rule names its source. When a rule here and the source disagree, the source wins; fix this file.

## Hard constraints H1–H8 (`docs/intention/book/01_objective_and_constraints.md`; a violation is a blocking B3 finding, `docs/process/flow.md` §5)
| # | Rule | Where it bites in code |
|---|---|---|
| H1 | Nothing is written to an external platform without an explicit, recorded confirmation | every mutating `CommerceAdapter` method takes a `ConfirmedRun`; `publish_confirm` writes `publish_runs`/`publish_items` first |
| H2 | RAW files are never modified, moved or deleted. **Exceptions the owner made:** published, clean items are cleared after retention (`10F`, M41: only after Shopify is verified to hold every published image); a **denied** photo's RAW is deleted when its undo window closes (changelog 46, P66). "Delete" before P66 only hid the photo | `services/storage.rs`, `mechanisms/retention.rs`, `photo_denials`; the capture ledger never replaces a file (`camera_files::rename_exclusive`) |
| H3 | No pixel processing except the composition renderer (decode master, turn, flip, straighten, crop, resize, encode once, thumbnails); copying an embedded JPEG out of a RAW is allowed, decoding a RAW is not | `crates/infra/src/imaging/` is the only crate that decodes pixels |
| H4 | Every step is idempotent; resuming after a crash yields the same end state, no duplicate uploads | publish steps, `photo_links` upload states, export staging + swap, ledger `pending`/`done` rows |
| H5 | No UI automation of other apps; Capture One only through its AppleScript dictionary (`docs/reference/capture_one_16.8.6.sdef`) | all Capture One access goes through `CaptureOneScripting` → `jxa/*.js` → `osascript`; `launch()` uses `open -g -a`, `quit` is the dictionary's quit |
| H6 | The core workflow runs with no commerce integration configured; publishing is optional and last | `App.commerce: Option<…>`; header action "Export" without an integration |
| H7 | Device-, app- and platform-specific concepts never appear in core modules or core tables; they live in adapters and link tables | `item_links`, `photo_links`, `attribute_definition_links`; known debts listed in `backlog.md` ("H7 family": `"shopify"` literals, Capture One error-number heuristics, Sony sentences in core) — do not add more |
| H8 | Business logic is independent of Tauri, of any HTTP server and of the UI; commands and events are thin | `psm-core` has no Tauri dependency; commands call services inside `spawn_blocking`; the phone remote (post-MVP) reuses the services |

## Public repository: what must never be committed (`CLAUDE.md`; master plan §3 "Camera hard rules" and "Secrets"; `native/camera-sidecar/README.md` §Never commit; `.gitignore`)
- Any Sony SDK file: headers, `*.dylib`, `CrAdapter/`, the `CrSDK_*` archive or folder, the lab's
  hand-written `sdk_shim` headers. The SDK lives only on the build machine, outside the repo.
- `.env` files, credentials, tokens, Keychain contents, the Shopify client id/secret, the shop
  domain, IP addresses, camera serials/ids, SSH/camera fingerprints (`*.fingerprint`).
- Runtime data: databases, logs, previews, photos (`data/`, `*.db`, `*.log`).
- Generated artefacts: `apps/desktop/src-tauri/binaries/` (the sidecar), `target/`, `node_modules`,
  `.pipeline/`, `vendor/`, `CrSDK*`.
- Secrets never appear in logs, fixtures, handoffs or narratives either (master plan §3; `12`
  rule 8). Register every new secret with `psm_infra::logging::add_secret` so the log redacts it.
  Use placeholders such as `<shop>.myshopify.com` in docs.

## Camera hard rules (owner 2026-11-06; master plan §3; book `10E`; narratives P52–P57)
- Never change the camera's network or FTP settings; the app only reads/verifies the store
  destination the sidecar sets for a session (`10E` MC-31).
- Never delete a file on the camera.
- Never kill `ptpcamerad` or any other process. `infra/processes.rs` is read-only by design (M39).
- Capture One is quit only by the take-back sequence (`direct/take_back.rs`: ask it to quit through
  the dictionary, wait, never kill), and only when the preconditions hold. Nothing else quits it.
- Live view is out of scope.
- The automated suite never needs the camera, Capture One, Shopify or the network (flow §6A;
  master plan §3 "Hardware"); the sidecar in tests is `testkit/src/bin/fake_sidecar.rs`.
- No `implementer: fable` on camera phases (owner).

## Stack and boundaries (`CLAUDE.md`; book `11`; master plan §6.1)
- Rust is the backend. **No Python in the product** (`11` §Processes: a Python sidecar only for a
  demonstrated need; none exists). `python3` is used only by the pipeline runner's template rendering
  (`tools/pipeline/run_phase.sh`), never by a build or by the app.
- `psm-core` depends on nothing above it; services depend on ports only; `infra` and `integrations`
  implement ports; `psm-desktop` commands are thin.
- No Tauri `invoke` from components: `services/*.ts` over the generated `bindings.ts`, which is
  never hand-edited (master plan §6.1, §6.7).
- Capture One Pro 16.8.6.25 is the only tested version; property names come from the sdef and the
  VERIFY items in `docs/intention/evidence/capture_one_16.8.6.md`.

## Persistence rules (master plan §3, §6.5, §7.3)
- Migrations are never rewritten once a checkpoint commit contains them: add the next numbered file
  and its `M::up` line (`crates/infra/src/sqlite/store.rs`).
- Tests open SQLite only inside a `TempDir`; the configured database is never touched by tests.
- `settings.json` stays user configuration; app state goes to the `settings` table. Secrets other
  than the Shopify client pair are Keychain only (`12`; `10E` R-I21).
- Derived values (session status, counts, fill counts, the publish summary) are never stored
  except as frozen into a confirmed run (`05` §Facts vs derived).

## The naming registry is binding (master plan §6)
Crate, module, table, column, enum, error code, health code, command, event, settings key and test
names are those of master plan §6.4–§6.11. A new name is added to the registry in the same change
(fold-back rule, master plan §2: semantics → book + changelog; skeleton/environment → master plan;
phase-local → plan file). Known registry gaps are recorded in `backlog.md` ("book and registry drift").

## Tests (`docs/process/flow.md` §4, §4A; master plan §3, §6.9)
- One behaviour test per criterion row, at the boundary the row names (service call, command
  handler, produced file, stored state, calls a fake adapter received), asserting the exact outcome.
- Unit tests only for core mechanisms a plan lists by name, inline in
  `crates/core/src/mechanisms/<m>.rs`, named `mc<n>_<slug>`.
- Never: internal call counts, private helpers, generated SQL, mocks of internal collaborators,
  snapshots of internals, "tests for confidence". The over-engineering test: a test that must change
  for a reason other than the behaviour changing is the defect.
- No mutation regime; reviewers never edit production code to see a test fail.
- Behaviour tests live in `crates/testkit/tests/suite/<phase>_<slug>.rs` **and must be listed as
  `mod <stem>;` in `crates/testkit/tests/suite/main.rs`** (`autotests = false`): an unlisted file never
  runs and nothing warns (P70). Tests share one process: no environment variables, current
  directory, global subscriber, fixed ports or fixed temp paths; a test that needs process-wide state
  wraps its body in `psm_testkit::isolation::in_own_process`.
- Test runs: while working, only the phase module (`cargo test -p psm-testkit --test suite <stem>::`),
  the changed module's unit tests, the touched UI test files; the full gate **once** at the end.
  **One cargo command at a time, never two in parallel, never piped into `head`** (a cut pipe leaves
  cargo holding the build lock). The reviewer by default runs no full suite.
- `FakeEditing` is the real Capture One adapter over `FakeCaptureOneScripting`; never a second
  `EditingProvider` implementation (master plan §7.5).

## Commits and pushes (`docs/process/flow.md` §8; STATUS 2026-11-11 11:00 and 11:40)
- Implementer and fix sessions commit `CHECKPOINT (not approved):` of their perimeter only; the
  runner commits prompts, handoffs, narratives, state and the `APPROVED <phase>` commit.
- **Commit only named paths** (`git commit -- <paths>`); never `git add -A` or `git commit -a` while
  a phase runs (two orchestrator commits swept in-progress code and do not build).
- Nobody pushes but the orchestrator, at the end of a run. Never push from a phase session.
- Phase perimeter: touch only the files the plan lists (plus its named test files); a new file is
  declared under **Deviations** in the handoff (master plan §3).
- No questions during a phase: an open choice is the smallest option consistent with the cited
  chapter, recorded under **Assumptions**; `BLOCKED` only for a plan/book contradiction or an
  impossible task (flow §6).

## Design and UX rules (`docs/intention/evidence/design_handoff.md`; `claude_design/`)
Field kinds, timings (undo toast 4.5 s, crossfade 700 ms), keyboard map, component set and tokens
come from the handoff; `styles/tokens.css` is copied from `claude_design/tokens/tokens.css`. Owner
2026-11-04/05: frontend implementation is not run on Fable; the Tools panel is a left collapsible
panel, the browser stays on the right (`10D`).
