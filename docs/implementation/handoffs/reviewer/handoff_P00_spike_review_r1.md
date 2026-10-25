---
plan: P00_spike
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-25
actor: opus
---
# Review P00_spike, round 1: APPROVED_WITH_NOTES

All 16 criterion rows hold at their boundaries, and each test would fail on a wrong outcome. Suite,
clippy, fmt, typecheck and the frontend test command are green. No B1–B4 finding. Ten notes are in
the backlog. The live spike items still open (bundle Automation prompt, work Mac, real capture-folder
watch) are recorded, not blocking: plan task 11 assigns (c) to the orchestrator and owner after the phase.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker row `P00_spike` = `IMPLEMENTED` (2026-10-25, codex). Checkpoint `87b2826d9405d69c01e4ca22c3e986b24cf5a055` exists in `git log`.
- Tree at review: HEAD `85063e7`, `git status --porcelain` empty before and after the run; delta from the checkpoint is two runner docs (handoff, review prompt).

## Verified correct
**Perimeter.** `git show --stat 87b2826` = exactly the 6 declared files (`crates/integrations/src/lib.rs`, `HealthPage.tsx`, `globals.css`, `packaging_spike.md`, `master_plan.md`, `plans/P00_spike.md`). The whole-phase diff `24b27a7^..87b2826` (code, excluding docs) matches the plan's file list plus the extras declared in earlier continuations (`Cargo.lock`, shadcn `components/ui/button.tsx`, `lib/utils.ts`, iconset PNGs under `icons/*`). `tools/pipeline/run_phase.sh` was changed by the orchestrator's commit `48238fb`, not the implementer. The `cn` npm dependency is shadcn's own package (`github.com/shadcn-ui/cn`), not a stray.

**Stamp (L4, tree `85063e7`, clean; cargo from `~/.cargo/bin`, Node 22 via the §7.1 PATH prefix).**
| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0: `p00_spike` 16/16; `mc15_parse_processing_payload`, `mc15_json_escape_preserves_value`, `mc15_template_fills_spool_path_and_port`; `export_bindings` |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` | exit 0 |
| `npm test` (`vitest run --passWithNoTests`) | exit 0, no frontend test files (none required by P00) |
The committed `bindings.ts` equals the regenerated one (tree clean after `export_bindings`).

**Rows (production read, then test).**
| Row | Production that makes it true | Test verdict |
|---|---|---|
| C1(a) | `AppPaths::create` makes the 6 dirs. `Settings::open` writes defaults with `create_new` + `mode(0o600)` only when absent. `sku_prefix` is `skip_serializing_if = None`; `document_id`/`integration` serialize as `null`. `001_init.sql` `INSERT OR IGNORE mechanisms_version '1'`. | Observes files, mode, every value and the DB row. Weak only on absent-vs-null for two keys (note 8). |
| C1(b) | Settings are re-read, never rewritten. `rusqlite_migration` is at `user_version` 1. | Byte-equal file with `20` (a rewrite would restore `15`); version 1 and `user_version` 1. Bites. |
| C2(a) | `Command::current_dir(data_dir)` | A different cwd (crate dir) fails the canonical compare. Bites. |
| C2(b) | Poll `try_wait`; on timeout `kill` + `wait`, `Err(Timeout)` | Elapsed < 2 s and `pgrep -f <script>` empty. Without the kill, `delay(3)` would still be running. Bites. |
| C3(a) | `compile_all` renders 3 templates → `osacompile -o` | Non-empty `.scpt` × 3. |
| C3(b)/(c)/(d) | Hook body in `try`: build JSON, write `.json.tmp`, `/bin/mv` to `.json`, background `curl -m 1 … &`. `processing_done` maps argv 3+ to `outputs`. | Runs the compiled hook via `osascript` against port 9 (no listener). Exact JSON equality; quote/backslash path round-trips. Bites. |
| C4(a) | `drain`: sort, parse, `record_hook` in one transaction with an `INSERT OR IGNORE` name key, then `remove_file` | Exact `(source, hook, payload)` row and file gone. |
| C4(b) | Parse error → `rename` to `spool/bad/<name>` | |
| C4(c) | `files.sort()` | Checked by variation: on this APFS volume, raw `readdir` returns `0002-later.json` before `0001-earlier.json` in both creation orders, so a missing sort fails the test. Bites. |
| C4(d) | `App::open` drains before starting the listener | |
| C5(a) | `/nudge` handler drains synchronously, then returns 204 | The row's 1 s bound is met trivially (consumed before the response). The test's 2 s poll is harmless. |
| C5(b) | `any` route ignores the body; `TcpListener::bind("127.0.0.1:0")` | Zero rows and IP `127.0.0.1`. |
| C6(a) | `notify` FSEvents (polling fallback), root + one level, absolute paths | Equality with the absolute path. Several events are acceptable: MC-2 lets hook and watcher race, and the downstream transaction dedupes. |
| C7(a) | `specta_builder` with both commands; `export_bindings` | `healthGet`/`appPathsGet` in bindings (invoked as `health_get`/`app_paths_get`); typecheck green. |

**Hard constraints H1–H8.** H1/H6 nothing external is written. H2 no RAW is touched (hooks write only to `spool/`). H3 no pixel work. H4 crash between commit and delete is safe (name key in the same transaction), and hook writes are tmp → rename. H5 the hooks use only dictionary four-char codes, each checked in the 16.8.6 sdef on the right class: `COCp` last captured file on `document`, `Cudo` current document, `cprv` primary variant on `application`, `ID  `, `ipth`, `vpar`. No UI scripting. H7 `psm-core` has no Capture One or Tauri types. H8 logic lives outside Tauri; one thinness note (note 2). Secrets: none. No live services: the suite runs `osascript`/`osacompile` locally, never addresses Capture One, and its `curl` goes to the closed port 9.

**Assumptions (handoff + plan Review log).** None contradicts the book. Node 22 in `apps/desktop` (environment). Processed spool names kept in `settings`: ch05 defines `settings` as key/value app state, so this is allowed, but it grows without bound (note 4). Physical cwd compare (test-only). Watcher polling fallback. Probe on load and on the button, matching the plan's manual try-out ("or wait for the startup check"). Local check time. Placeholder icon. All are in the narrative.

## Findings
None blocking.

## Notes written to backlog
Carry-forward dispositions (each row is in `docs/implementation/backlog.md`):
| # | Note | Destination |
|---|---|---|
| 1 | Sync `health_get` runs the 30 s Capture One probe inline while holding `Mutex<App>`; the probe also launches Capture One when it is closed. | P04, P16 |
| 2 | `capture_one` health item is assembled in the Tauri command, not `HealthService` (H8 thinness). | P16 |
| 3 | Non-numeric `preview_delay_s` or malformed `settings.json` → `Settings::open` error → startup panic; MC-17 says not-a-number → 15 s. | P06, P16 |
| 4 | Dedupe keys `spool:<name>` accumulate in `settings` forever. | P01/P05 (new migration) |
| 5 | Hook `jsonEscape` misses control characters other than CR/LF/tab (MC-15). | P05 |
| 6 | Watcher start failure also kills the 5 s interval drain. | P05 |
| 7 | Logging initialised after `App::open`. | P16 |
| 8 | `c1a` cannot tell an absent key from `null` for `document_id`/`integration`. | P00 test (low) |
| 9 | Spike items open: (c) bundle Automation prompt with the real-event probe + remembered permission; (f) work Mac; (e) real capture-folder FSEvents. | owner before P07; (e) P05 |
| 10 | Unused `icons/SessionManager.iconset/*`. | P00 (cleanup) |

## Lessons for the plans
- Commands that call providers (P04 onward) should be specified as `async` / off the IPC thread, and must not hold the app-wide lock across an `osascript` call. A plan line in P04 prevents note 1 from multiplying.
- MC-17's "not a number → 15 s" needs a criterion row where settings are read (P06 or P16). P00's §6.8 skeleton only said "read clamped".
- C1(a)-style rows that assert "key present with value null" need a presence check (`settings.get(k)`), since JSON indexing returns null for missing keys.

## Write perimeter
`docs/implementation/backlog.md` (10 rows) · `docs/implementation/plans/P00_spike.md` (Review log entry) · `docs/implementation/master_plan.md` (tracker row P00 → APPROVED only) · `docs/implementation/narratives/P00_spike.md` (new) · this handoff. No production or test file was edited. No mutation probes (flow §4). `cargo test` regenerated `bindings.ts` byte-identically (tree clean). Temporary directories created by `mktemp` for the APFS order check were removed. Nothing committed.
