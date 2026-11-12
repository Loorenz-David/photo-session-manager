---
plan: P70_fast_test_suite
role: implement
round: 1
state: DONE
date: 2026-11-12
actor: claude (opus, pipeline-implementer; the runner's Codex launch exited at once)
---
# P70 fast test suite — implement r1 handoff

## What was built
The Rust test suite is faster to run, and the app behaves exactly as before.

- **Where the hour went (measured first).** On this Mac, macOS checks every freshly built program the first
  time it starts (`syspolicyd`; the process sits in `dyld_start`). With 63 test programs that check cost
  **47.6 min** of the 65 min suite (45–170 s per program), while the tests themselves ran 17.8 min, 7.4 of
  which were p00's file-watcher test waiting for the system's file-event service and p43's camera tests
  waiting for fresh copies of the fake camera helper to pass the same check.
- **One integration test program.** The 56 behaviour-test files moved unchanged to
  `crates/testkit/tests/suite/` as modules of one test target `suite` (`autotests = false`, one `[[test]]`).
  One program is checked once per build instead of 56 times. One phase's tests:
  `cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::`.
- **What had relied on being its own process, now per test:**
  - the fake camera helper is no longer copied per test (every copy was a new program for macOS to check,
    and dozens starting at once queued for minutes); each camera test links to one shared copy (named by its
    content hash, beside the build output), and the fake still records into the test's own folder;
  - the two tests that install a process-wide log subscriber (p43 C5(c), p53 C3(a)) and p00 C6(a) (the
    only test that waits for a real file-system event) run in their own child process of the suite
    (`psm_testkit::isolation::in_own_process`), exactly as when their file was its own program;
  - thread-scoped log captures (p16, p34, p61, p62, p65, p66, p69) first install a do-nothing global
    subscriber (`psm_testkit::logs`) so that `tracing`'s per-site interest cache never drops a captured line
    while other test threads hit the same log site;
  - two start-order races that only show under the suite's parallel load were closed in the tests
    (see Deviations: p50 C6(b), p63 C1(a)).
- **No real-time wait on a production timing.** The only test that waited seconds for a production timing
  was p42 C2(b) (the camera helper's 3 s quit grace). That grace is now `SidecarTimings { quit_grace }` with
  `Default` = 3 s, read by the runtime at every quit; the test sets 500 ms through the new test seam
  `CameraRuntime::set_sidecar_timings`. Production never calls the seam. The other timings the plan listed
  already run on the test clock (restart backoff, take-back timers, watchdog, storage batch pause) or are
  delays the tests choose themselves (fake upload/call delays).
- **Optimised dependencies:** `[profile.dev.package."*"] opt-level = 2` — kept: the suite program's own time went from 188 s to 115 s (−39 %), the warm `cargo test --workspace` from 596 s to 524 s. One-time cost: the first build after the change recompiled every dependency (about an hour today, most of it macOS checking the rebuilt macro libraries after an XProtect update).
- **UI tests:** plain `npm test` ran 21 s, 466/466, with no load timeout, so the vitest config is unchanged (the plan's step 5 applies only if timeouts recur). A second run hit the known intermittent p33 C4(d) stage-bar failure (backlog since P35: "Main 6 of 6"); that file passed 3/3 alone.

## Tests → criterion rows
| Row | Boundary | Evidence |
|---|---|---|
| C1(a) | suite time | before 65 min (table in the plan's Review log); after: **8 min 43 s** (B2: 523.5 s, second of two back-to-back runs, optimised dependencies; without them 596.1 s) |
| C2(a) | same tests | `--list` of the 56 old programs (812 integration tests) vs the `suite` list: 812 = 812, no name missing, none extra (module prefix aside); no `#[ignore]` added, no test deleted; assertion changes only the p42 C2(b) timing value (3 s → the test's 500 ms grace, same bounds: ≥ grace, < grace + 1 s). Lists: `.pipeline/p70/before_names.txt`, `after_names.txt` (script `compare_lists.sh`) |
| C3(a) | production timings | core-mechanism unit test `camera::sidecar::tests::production_timings_unchanged` (psm-integrations): `SidecarTimings::default()` = `{ quit_grace: 3000 ms }`; the only `set_sidecar_timings` caller is p42 C2(b) |
| C4(a) | per-phase command | `cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::` → `6 passed; 0 failed; … 806 filtered out` (exactly p66's 6 tests) |
| C4(b) | docs | master plan §6.9 + §7.2, flow.md §4A, templates implement/fix/review/rereview name `--test suite <stem>::`; `grep -- '--test <'` finds no per-file form left |
| C5(a) | existing suites | `cargo test --workspace` twice in a row (B1, B2): 946/946 both times (812 suite + 134 unit); clippy `-D warnings` exit 0; fmt exit 0; typecheck exit 0; plain `npm test` 466/466 (one later run hit the known p33 C4(d) flake, named below); the sidecar stub C++ tests run inside the suite (p45 C2) and passed |

## Assumptions
- **A1** "Measure first" timed each of the 63 old test programs once from the built binaries (the plan's
  method), recording separately the first start (`--list`, which carries macOS's check) and the run.
- **A2** A test that installs a process-wide subscriber, or that needs the system's file-event service to
  itself, is made "per test" by running it in its own child process of the same suite binary (the smallest
  way to keep its body and assertions unchanged); the child is selected by an environment variable
  (`PSM_TESTKIT_ISOLATED_TEST`) set only on that child.
- **A3** The fake camera helper is reached through a symbolic link to one shared copy instead of a copy per
  test. Production only ever renames over or deletes that path (never writes through it), so no test can
  change the shared copy; the fake derives its recording folder from the path it was started by (the link).
- **A4** Only timings a test actually waited for in real seconds were made settable (p42's quit grace);
  the plan's list was the expected set, the measurement showed the rest are already on the test clock.
- **A5** C1's "warm build (second run)" is read as the second of two back-to-back `cargo test --workspace`
  runs (the first one absorbs macOS's first-start check of every relinked program).
- **A6** Step 5 (vitest workers) is conditional on load timeouts; none occurred, so the vitest config is
  unchanged. The one UI failure seen is the known p33 C4(d) race, not a timeout.
- **A7** Long cargo runs were started with the tool's background mode only because its foreground limit is
  10 minutes; never two cargo commands at once, never piped into `head`.

## Deviations
- **p50 C6(b)** now waits for the helper's first start before reading its process ids (it read them as soon
  as the receiver was up; under parallel load the helper starts a moment later and the "no restart"
  assertion compared against an empty list). The assertion is unchanged.
- **p63 C1(a)** does not tick the app until the take-back has passed its preconditions (its `save` reached
  the fake). Under load the test's first tick started the pending preview render before the spawned
  take-back thread checked its preconditions, and the take-back refused ("a preview is rendering") —
  diagnosed with a temporary print, then removed. Assertions unchanged.
- **p00 C6(a)** ends its own process without stopping the native watch (`mem::forget`): on this Mac the
  file-event service took 40 s–20 min to stop a stream while the suite's other apps start theirs. Its
  assertion (the event carries the absolute path) is unchanged. Observation for the owner: every app
  open starts two FSEvents watches; in the suite ~800 of them wait on `fseventsd` without blocking any test.
- `crates/integrations/src/camera/{runtime.rs, sidecar/mod.rs, sidecar/supervisor.rs}`: the only
  production change — the quit grace read from `SidecarTimings` (default 3 s) instead of the constant
  directly.
- **Not fixed (outside the perimeter, noted):** every `cargo test --workspace` rebuilds `psm-desktop`,
  because the p45 test re-runs `tools/sidecar/build.sh --stub`, which rewrites
  `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` (tauri-build watches it); the two
  relinked desktop test programs then pay macOS's first-start check again. A copy-only-if-changed in
  `build.sh` would remove ~1–2 min from every run.

## Files changed
All in checkpoint `41e01c7` (73 paths: 56 renames + edits):
- `Cargo.toml` (workspace: `[profile.dev.package."*"] opt-level = 2` only)
- `crates/testkit/Cargo.toml` (`autotests = false`, `[[test]] suite`)
- `crates/testkit/src/lib.rs`, `crates/testkit/src/isolation.rs` (new), `crates/testkit/src/logs.rs` (new),
  `crates/testkit/src/test_app.rs` (fake helper as a link to `shared_fake_sidecar()`)
- `crates/testkit/tests/*.rs` → `crates/testkit/tests/suite/*.rs` (56 files moved; edited: fixture `include_str!` paths one
  level deeper in 11 files; p00 C6(a), p43 C5(c), p53 C3(a) wrapped in `in_own_process`; p16/p34/p61/p62/p65/p66/p69 one
  `keep_log_sites_interested()` line before their capture; p42 C2(b) grace; p50 C6(b), p63 C1(a) start-order waits),
  `crates/testkit/tests/suite/main.rs` (new)
- `crates/integrations/src/camera/sidecar/mod.rs` (`SidecarTimings` + core-mechanism test),
  `crates/integrations/src/camera/sidecar/supervisor.rs` (`quit_and_wait(grace)`),
  `crates/integrations/src/camera/runtime.rs` (holds `SidecarTimings`, seam `set_sidecar_timings`, passes the grace)
- `docs/implementation/master_plan.md` (§6.9 test layout, §7.2 command, tracker row), `docs/implementation/plans/P70_fast_test_suite.md`
  (Review log), `docs/process/flow.md` (§4A, log v1.5), `tools/pipeline/templates/{implement,fix,review,rereview}.md`
- This handoff (committed by the runner).
- Generated outside git: `apps/desktop/src-tauri/target/debug/psm-testkit/camera-sidecar-<hash>` (the shared fake helper copy).

Scratch evidence (not committed, `.pipeline/` is ignored): `.pipeline/p70/before/` (per-program lists,
outputs, `times.tsv`), `before_names.txt`, `after_names.txt`, `suite_list.txt`, `suite_run2.json`
(per-test times), `after_noopt_run{1,2}.log`, `after_opt_run{1,2}.log`, `measure.sh`, `compare_lists.sh`.

## Checkpoint SHA
`41e01c7` — `CHECKPOINT (not approved): implement P70 fast test suite …`

## Stamp
| Command | Result |
|---|---|
| per-program measurement before (63 programs from the built binaries) | 945 tests, 1 failure (p43 C7(b), macOS check of a fresh helper copy > 15 s); run 1066.8 s + first start 2856.8 s = 65.4 min |
| `cargo test --workspace` A1 / A2 (unoptimised deps) | 946/946 / 946/946; 3050 s (25 min compiling while macOS re-checked macro libraries) / 596.1 s |
| `cargo test --workspace` B1 / B2 (opt-level 2 deps, final configuration) | 946/946 / 946/946; 1029.9 s / **523.5 s**; suite program 113.7 s / 116.4 s |
| `cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::` | 6 passed, 806 filtered out (498 s: the single-package build resolves features differently and rebuilt some dependencies once) |
| `cargo fmt --all -- --check` | first exit 1 (line wrapping of my `quit_and_wait(…)` calls and the wrapped test bodies) → `cargo fmt --all` (no new files touched) → exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings (on the formatted final tree) |
| `npm run typecheck` | exit 0 |
| `npm test` | run 1: 70 files, 466 passed (21.3 s); run 2: 465 passed, 1 failed — pre-existing intermittent `p33_split_grid.test.tsx` C4(d) (backlog P35/P37/P45 rows) → that file alone 3×: 7/7 |

Pre-existing failures named: `p33_split_grid` C4(d) (UI, intermittent). The B runs were on the tree before `cargo fmt`
re-wrapped lines (whitespace only); clippy compiled the final tree.

## Manual try-out
1. In the repo, run `cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::` —
   only that phase's 6 tests run (`806 filtered out`).
2. Run `time cargo test --workspace` twice; the second run is the suite's normal time on this Mac.
3. Watch Activity Monitor during the first run after a rebuild: one `suite-…` process waits about a
   minute before printing anything — that is macOS checking the new program, once instead of 56 times.
4. The app itself is unchanged: open it and use it as before.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
