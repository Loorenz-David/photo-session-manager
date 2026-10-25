---
plan: P00_spike
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: codex
---

# P00 packaging spike — implementer handoff

## What was built

Session Manager now has a signed macOS app with a Health page, local data storage, compiled
Capture One hooks, a spool that keeps events while the app is closed, and a local event listener.
The Capture One Health check now asks for the open document's name through a real Apple event.
The button shows “Testing…” while it runs; the result row shows the name or error and a check time.
The probe script is written once at app startup. Earlier live checks outside the sandbox proved
that the bundle opens and that real processing and capture hooks reach the app's database.

## Tests → criterion rows

All 16 rows have one boundary test in `crates/testkit/tests/p00_spike.rs`:

| Row | Test | Boundary outcome asserted |
|---|---|---|
| C1(a) | `c1a_first_open_creates_layout_and_defaults` | layout, settings mode/defaults, mechanism version |
| C1(b) | `c1b_second_open_preserves_settings_and_migration` | edited settings unchanged, migration/version unchanged |
| C2(a) | `c2a_osascript_uses_data_directory_as_cwd` | stdout resolves to data directory |
| C2(b) | `c2b_osascript_times_out_and_reaps_child` | timeout within two seconds, no remaining child |
| C3(a) | `c3a_compile_all_produces_three_scripts` | three nonempty compiled hooks |
| C3(b) | `c3b_processing_hook_returns_and_spools_without_listener` | fast exit and exact processing payload |
| C3(c) | `c3c_capture_hook_spools_argv` | exact capture argv payload |
| C3(d) | `c3d_processing_hook_escapes_quote_and_backslash` | escaped path round trips |
| C4(a) | `c4a_valid_file_becomes_one_event_and_is_deleted` | one exact event, file removed |
| C4(b) | `c4b_malformed_file_goes_to_bad` | file in bad spool, no event |
| C4(c) | `c4c_drain_uses_filename_order` | event payloads follow filename order |
| C4(d) | `c4d_open_drains_file_left_while_closed` | startup records event and removes file |
| C5(a) | `c5a_nudge_consumes_spool_file` | HTTP 204 and recorded event within deadline |
| C5(b) | `c5b_nudge_body_cannot_inject_event` | HTTP 204, no event, loopback address |
| C6(a) | `c6a_watcher_emits_absolute_path` | absolute created path received within deadline |
| C7(a) | `c7a_generated_bindings_expose_commands` | generated bindings name both commands; TypeScript typecheck passed |

The three allowed core-mechanism unit tests cover hook template filling, JSON escaping,
and spool payload parsing. The Tauri bindings export test also passed.

## Assumptions

- The check time uses the Mac's local clock. The startup check and button use the same probe.
- Earlier P00 choices remain recorded in the plan's Review log: Node 22 in `apps/desktop/`,
  processed spool filenames in `settings`, physical cwd comparison on macOS, watcher polling
  fallback, and placeholder icon.

## Deviations

None in this continuation. The corrected Automation prompt and remembered permission need an
interactive bundle run outside the Codex sandbox, as the amended plan permits. The work Mac was
not available; the real capture-folder watcher check was not rerun. These limits are in
`docs/intention/evidence/packaging_spike.md`.

## Files changed

- `crates/integrations/src/lib.rs`
- `apps/desktop/src/features/health/HealthPage.tsx`
- `apps/desktop/src/styles/globals.css`
- `docs/intention/evidence/packaging_spike.md`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P00_spike.md`
- This handoff, for the runner to commit.

No mutation probes were used.

## Checkpoint SHA

`87b2826d9405d69c01e4ca22c3e986b24cf5a055` — `CHECKPOINT (not approved): finish P00 Capture One probe and spike record`.
The handoff is written after the checkpoint so it can name that SHA; the runner commits handoffs.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS: 16 P00 boundary tests, three core-mechanism tests, bindings export |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run typecheck` in `apps/desktop` | PASS |
| `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm test` in `apps/desktop` | PASS; no frontend test files in P00 |
| `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run tauri -- build --bundles app` in `apps/desktop` | PASS; ad-hoc-signed `.app` produced |
| `codesign --verify --deep --strict 'apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app'` | PASS |

The stamp covered the checkpoint's code tree; the checkpoint then added only tracker and evidence
text. No pre-existing failures were observed.

## Manual try-out

1. In `apps/desktop`, run `npm ci && npm run tauri -- dev`. Open a Capture One session document.
2. On the Health page, press **Test Capture One**. The button should show “Testing…”; allow
   macOS Automation access if prompted. The Capture One row should show the document name and
   “checked hh:mm:ss”.
3. Quit and relaunch, then check that macOS remembers the permission and the document name returns.
4. Run `npm run tauri -- build --bundles app`, then right-click Open
   `apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app`.
5. Inspect `~/Library/Application Support/com.loorenz.sessionmanager/` for the database,
   mode-0600 settings, hooks, spool, and logs. To verify bundle-attributed Automation, perform
   step 2 from the bundled app as well.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
