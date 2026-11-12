---
phase: P70_fast_test_suite
title: The test suite runs in minutes instead of over an hour — the integration tests become one test program, real-time waits in tests shrink to milliseconds through test settings, and slow image work in tests runs optimised; the same tests run and the app behaves exactly as before
difficulty: high
projection: no
depends_on: [P69_faster_export_and_publish]
---
# P70 — Fast test suite (owner 2026-11-11)

Written by the orchestrator (Opus) on 2026-11-11. Owner: "why are these builds / corrections taking so long? … I will continue
developing the app and changes can't take this long." Measured on P66: implementer 4 h 20 min wall-clock, 17 min model time;
review 58 min, 5 min model time. A single `cargo test --workspace` on the dev Mac ran over 1 h 15 min (P67, 2026-11-11): the
54 files in `crates/testkit/tests/` are 54 separate test programs, each linked against the whole workspace and run one after
the other, and several of them wait in real time (camera helper backoffs, 10 s busy windows, settle loops, the 2 s purge
check) — one program alone (`p46_direct_camera_adapter`, `p50_camera_settings_and_support`) ran for minutes. This phase
changes **how the tests run, never what the app does.**

## Goal
1. **Measure first.** Before any change, record per test program its wall-clock time (build excluded): build with
   `cargo test --workspace --no-run`, then run each test program once and time it. Put the table (top 15 + total) in the
   Review log. Repeat after the change.
2. **One integration test program.** The 54 (+ P66–P69) files in `crates/testkit/tests/` become modules of one test target
   (e.g. `crates/testkit/tests/suite/main.rs` with `mod p46_direct_camera_adapter;` …, `autotests = false` and one `[[test]]`
   in `crates/testkit/Cargo.toml`). File contents move unchanged except where two files clash (duplicate helper names,
   shared statics, environment variables, fixed ports, fixed temp paths) — make those per-test. Tests in one program run in
   parallel threads, so anything that relied on being a separate process (process-wide env vars, the current directory,
   global loggers, fixed ports) is fixed to be per-test. A per-phase run becomes
   `cargo test -p psm-testkit --test suite <stem>::` — update the master plan's environment table (§ test commands), the
   implement/fix/review templates' examples (`tools/pipeline/templates/*.md`) and flow.md §4A.
3. **No real-time waits in tests.** Every test that waits wall-clock seconds for a production timing (camera helper restart
   backoff ≤ 30 s, busy-transfer window 10 s, take-back timers, the deny purge check every 2 s, retention/storage timers,
   Capture One waiting/polling intervals, FTP timeouts) gets that timing from a test-settable value (the existing settings /
   service config / an injected clock where one already exists; otherwise a `Timings` struct with the production values as
   `Default` passed to the service) so tests use milliseconds. **Production values do not change**; one unit test asserts the
   `Default` values equal today's numbers (Core mechanism below). Polling loops in test helpers (`settle`, `settle_absence`)
   keep their 10 s ceiling as a failure bound but return as soon as the condition holds.
4. **Optimised dependencies in tests.** Add to the workspace `Cargo.toml`:
   `[profile.dev.package."*"] opt-level = 2` (dependencies such as image codecs, SQLite and hashing run optimised in debug and
   test builds; our own crates stay unoptimised for fast rebuilds). Keep it only if the measurement shows a gain; record the
   numbers either way.
5. **UI tests.** Record `npm test` time. If plain `npm test` still hits load timeouts (backlog 2026-11-10), set the worker
   count in the vitest config (`maxWorkers` / `minWorkers` from the CPU count, as `--maxWorkers=4` showed green) so plain
   `npm test` is reliable.

## Read first
`docs/process/flow.md` §4 and §4A; master plan environment table (test commands); `crates/testkit/Cargo.toml`, `crates/testkit/src/*`
(TestApp, fakes); the slowest files from step 1 (expected: p42–p63 camera/FTP/take-back files, p06 live previews, p16 operations,
p61/p62 storage, p66 purge); the services whose timings they wait on (`crates/core/src/services/{camera*.rs, arrival.rs,
storage.rs, review.rs, jobs*.rs}`, `crates/integrations/src/camera/*`, `crates/infra/src/ftp/*`); `apps/desktop/vite.config.ts`
or `vitest.config.ts`.

## Files expected to change (write perimeter)
`crates/testkit/Cargo.toml`; `crates/testkit/tests/**` (moved into the suite module tree); `crates/testkit/src/*` (shared helpers);
workspace `Cargo.toml` (profile only); the services/adapters whose timings become injectable (constructor/config only, no
behaviour change); the vitest config; `docs/implementation/master_plan.md` (test commands); `docs/process/flow.md` §4A;
`tools/pipeline/templates/{implement,fix,review,rereview}.md` (command examples). No product feature changes.

## Core mechanisms (unit tests allowed)
- **Production timings unchanged:** the `Default` of every timing made injectable equals today's value (one test listing them).

## Criteria (5 criteria, 6 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | suite time | `cargo test --workspace` on the dev Mac, warm build (second run), nothing else building | finishes in **≤ 10 min**; before/after table in the Review log | `[rule]` owner 2026-11-11 |
| C2(a) | same tests | test names listed before (`cargo test --workspace -- --list` per program) vs after | every test name present after (module path prefix aside); none newly `#[ignore]`d, none deleted, none weakened (asserts unchanged except timing values) | `[rule]` flow.md §4 |
| C3(a) | production timings | the timing `Default`s | equal today's values (core-mechanism test); no production call site passes a test value | `[rule]` intention ch. 10/10E timings |
| C4(a) | per-phase command | `cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::` | runs exactly that file's tests, green | `[rule]` master plan test commands |
| C4(b) | docs | master plan, flow.md §4A, templates | name the new commands; no reference to the old per-file `--test <stem>` form remains | `[rule]` flow.md §4A |
| C5(a) | existing suites | everything | `cargo test --workspace` (run twice in a row to show no flakiness from parallelism), plain `npm test`, clippy, fmt, typecheck, sidecar stub tests green | `[rule]` master plan §7.2 |

## Review mode (owner 2026-11-10: fast; kept)
- **Implementer:** high effort. This phase is the exception to "full suite once": run the full suite before (measurement), after,
  and once more (C5(a) flakiness check); between them run only the moved/changed files. One cargo command at a time.
- **Reviewer (time-boxed, still correct):** run the full suite **once** with `time` (C1(a)), the C4(a) command, clippy, fmt,
  typecheck. Check C2(a) by comparing the before/after test lists the implementer saved (in the Review log or a scratch file
  named there), and C3(a) by reading every changed production constructor. Only B1–B4 block. Aim for about 20 minutes plus
  the suite run.

## Notes
- **Hard rules:** no behaviour change in the app; no test deleted or ignored to gain time; no real camera, Capture One or shop in
  tests (as always); no Python. Default implementer (Codex; Claude when out).
- If 10 min is not reachable without touching behaviour, stop at the best honest result and record the remaining slowest
  programs and why in the handoff (that is DONE with a deviation, not BLOCKED).

## Review log

### 2026-11-12 — implement r1 (claude opus; the runner's Codex launch exited at once)
**Built.** The 56 behaviour-test files are modules of one test program `suite` (`crates/testkit/tests/suite/main.rs`,
`autotests = false`); per-phase command `cargo test -p psm-testkit --test suite <stem>::` (master plan §6.9/§7.2, flow.md §4A,
templates). Clashes made per-test: the fake camera helper is a link to one shared copy (no copy per test), the two tests
that install a process-wide subscriber and p00 C6(a) run in their own child process (`psm_testkit::isolation`), the
thread-scoped log captures pin `tracing`'s site interest (`psm_testkit::logs`). The one real-time wait on a production timing
(p42 C2(b), the helper's 3 s quit grace) reads `SidecarTimings` (Default 3 s, core-mechanism test
`production_timings_unchanged`; test seam `CameraRuntime::set_sidecar_timings`). `[profile.dev.package."*"] opt-level = 2` kept.

**Measured before** (each of the 63 programs once from the built binaries, 2026-11-12 03:40–04:50; "first start" = `--list`,
which carries macOS's check of a newly built program). Totals: **run 1066.8 s + first start 2856.8 s = 3923.6 s (65.4 min)**,
945 tests; one failure (p43 C7(b): the fake helper's fresh copy waited > 15 s for macOS's check). Top 15 by run + first start:

| Program | Tests | Run s | First start s | Total s | Result |
|---|---|---|---|---|---|
| p00_spike | 16 | 445.2 | 52.8 | 498.0 | pass (C6(a) waited minutes for the file-event service) |
| p43_ftp_receiver | 26 | 230.5 | 0.0 | 230.5 | FAIL (C7(b), see above) |
| p65_capture_sync_waiting_jobs_and_inventory | 8 | 4.1 | 170.3 | 174.4 | pass |
| p61_storage_cleanup_and_budget | 7 | 11.7 | 145.8 | 157.6 | pass |
| p58_fast_compose_on_camera_preview | 6 | 3.0 | 152.2 | 155.3 | pass |
| p62_retention_after_publish | 12 | 8.8 | 136.7 | 145.5 | pass |
| p69_faster_export_and_publish | 7 | 62.5 | 73.9 | 136.4 | pass |
| p63_final_check_fixes | 4 | 3.7 | 127.7 | 131.4 | pass |
| p64_publish_speed_and_visibility | 8 | 32.5 | 83.2 | 115.8 | pass |
| p66_approve_on_arrival_and_session_order | 6 | 2.8 | 112.4 | 115.2 | pass |
| p57_camera_auto_take_back_and_run2_fixes | 8 | 13.7 | 97.0 | 110.8 | pass |
| p56_camera_claim_and_render_loop | 4 | 3.1 | 106.2 | 109.3 | pass |
| p29_master_and_output_render | 26 | 24.9 | 71.5 | 96.4 | pass |
| p30_outputs_in_export_and_publish | 21 | 15.5 | 79.9 | 95.4 | pass |
| p60_take_back_and_self_healing | 8 | 5.6 | 77.3 | 82.9 | pass |

**Measured after** (`cargo test --workspace`, back to back, nothing else running):

| Run | Dependencies | Wall clock | Suite program (812 tests) | Result |
|---|---|---|---|---|
| A1 (first after rebuild) | unoptimised | 3050 s (25 min of it compiling while macOS re-checked the compiler's macro libraries after an XProtect update) | 186.6 s | 946/946 green |
| A2 (warm) | unoptimised | **596.1 s** | 188.8 s | 946/946 green |
| B1 (first after rebuild) | opt-level 2 | 1029.9 s | 113.7 s | 946/946 green |
| B2 (warm) | opt-level 2 | **523.5 s (8 min 43 s)** | 116.4 s | 946/946 green |

opt-level 2 for dependencies: suite program 188 → 115 s (−39 %), warm run 596 → 524 s; kept. The remaining warm time is
mostly macOS's first-start check of the two desktop test programs, relinked on every run because p45 re-runs
`tools/sidecar/build.sh --stub` and tauri-build watches the binary it rewrites (outside this perimeter; noted in the handoff).
Slowest modules inside the suite (summed test time, unoptimised run): p69 229.9 s (its 7 tests serialised by its own lock),
p29 196.6, p30 157.3, p64 131.6, p28 101.8, p48 67.4, p40 57.4, p13 55.6, p46 54.0, p50 51.2.
C2(a): 812 integration test names before = 812 after (`.pipeline/p70/before_names.txt` vs `after_names.txt`, module prefix
aside), none ignored or deleted. Assumptions A1–A5 and deviations (p50 C6(b), p63 C1(a), p00 C6(a) start/stop races) in the
handoff.

### 2026-11-12 — review r1 (opus): APPROVED_WITH_NOTES
**Gate/perimeter.** Tracker IMPLEMENTED; `41e01c7` = 73 paths (56 renames `tests/*.rs → tests/suite/*.rs` + 17 edits) = the handoff's
declared list exactly. Tree for every run below: clean `e6ad922` (code tree = `41e01c7`).
**Runs (review mode).** `cargo test --workspace --no-run` 51 s; `time cargo test --workspace` ×2 — run 1 (first after relink, nothing
compiled) 16 min 50 s, 946/946; run 2 (warm) **7 min 59 s**, 946/946 (812 suite in 114 s + 134 unit). The second run was authorised
before it: run 1 is not C1(a)'s condition (warm second run) and only a whole-suite run measures it. C4(a)
`cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::` → 6 passed, 806 filtered out (2 min 35 s, of which
recompiling integrations/testkit under package features). `cargo fmt --all -- --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0;
`npm run typecheck` 0. Logs: `.pipeline/p70/review/`.
**Verified correct.**
- C1(a): warm 7 min 59 s ≤ 10 min (implementer 8 min 43 s); before/after tables above.
- C2(a): `.pipeline/p70/before_names.txt` = `after_names.txt` (812, byte-identical after prefixing); independently `git grep '#[test]'`
  over `crates/testkit/tests` = 812 at `a00b7ee` and at `41e01c7`, `#[ignore]` 0 and 0; every changed `assert` line in the diff is a
  re-indent inside `in_own_process` except p42 C2(b) (3 s → the test's 500 ms grace, same shape: ≥ grace, < grace + 1 s).
  The p50 C6(b) and p63 C1(a) start-order waits only delay the first read/tick; the assertions and the window they guard are unchanged.
- C3(a): `SidecarTimings::default()` = `{ quit_grace: QUIT_GRACE_MS = 3000 ms }`; `production_timings_unchanged` asserts the literal 3000 ms,
  so a changed default fails it; all four `quit_and_wait` call sites pass `self.sidecar_timings().quit_grace`, the field starts at
  `Default`, and the only `set_sidecar_timings` caller in the repo is p42 C2(b). No other production change.
- C4(a)/(b): command runs exactly p66's 6 tests; master plan §6.9/§7.2, flow.md §4A, the four templates name `--test suite <stem>::`;
  no per-file `--test <stem>` form left.
- C5(a): green as above; implementer's npm stamp (466/466, known p33 C4(d) flake on a second run) stands.
- Isolation helper cannot pass vacuously: the parent requires child exit 0 **and** `1 passed` for the exact name. A3 (symlinked fake
  helper) holds: production writes only to temp copies (`sony::install`, `refresh_sidecar_copy`) and renames over the path, never
  through it. H1–H8 hold (only adapter-side code changed; no core, no pixel, no RAW, no Tauri change). No live services.
**Notes → backlog** (4): unlisted `tests/suite` file never runs; `-p` vs `--workspace` feature rebuilds + first-start check cost;
first workspace run after a relink > the tool's 10-min limit; shared helper copies accumulate.
