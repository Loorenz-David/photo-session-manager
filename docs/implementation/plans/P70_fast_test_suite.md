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
