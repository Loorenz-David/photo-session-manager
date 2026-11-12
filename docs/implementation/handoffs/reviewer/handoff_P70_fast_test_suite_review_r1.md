---
plan: P70_fast_test_suite
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-12
actor: opus (pipeline-reviewer)
---
# P70 fast test suite — review r1

**Verdict: APPROVED_WITH_NOTES.** Gate passed (tracker IMPLEMENTED, checkpoint `41e01c7` in `git log`). Every row holds at its
boundary; no B1–B4 finding. Four non-blocking notes written to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
Tree for all runs: clean `e6ad922` (code tree = checkpoint `41e01c7`; later commits are runner prompt/handoff files only).

| Check | Result |
|---|---|
| Perimeter | `git show --stat 41e01c7` = 73 paths: 56 renames `crates/testkit/tests/*.rs → tests/suite/*.rs` + 17 edits (workspace `Cargo.toml`, testkit `Cargo.toml`, testkit `src/{lib,isolation,logs,test_app}.rs`, `tests/suite/main.rs`, integrations `camera/{runtime.rs, sidecar/mod.rs, sidecar/supervisor.rs}`, master plan, plan, flow.md, 4 templates) = handoff "Files changed" exactly. No extra or missing path. |
| Build | `cargo test --workspace --no-run` 51 s |
| C1(a) | `time cargo test --workspace` run 1 (first after relink, nothing compiled): **16 min 50 s**, 946/946. Run 2 (warm, authorised before it: C1(a) is defined on the warm second run and only a whole-suite run measures it): **7 min 59 s**, 946/946 (suite 812 in 114 s; unit 67+20+10+37). ≤ 10 min holds. |
| C2(a) | `.pipeline/p70/before_names.txt` and `after_names.txt` byte-identical (812 names each). Independent cross-check: `git grep '#[test]'` in `crates/testkit/tests` = 812 at `a00b7ee` and at `41e01c7`; `#[ignore]` = 0 and 0. Every `-`/`+` `assert` line in the diff is a re-indent inside `in_own_process`, except p42 C2(b) (3 s → the test's 500 ms grace; same shape ≥ grace, < grace + 1 s). The p50 C6(b) and p63 C1(a) start-order waits only delay the first read/tick; the asserted outcome and the window it guards are unchanged (p63's ticks still run through the take-back's closed window). |
| C3(a) | `SidecarTimings::default()` = `{ quit_grace: Duration::from_millis(QUIT_GRACE_MS = 3000) }`; `production_timings_unchanged` asserts the literal 3000 ms, so a changed default fails it. All four `Supervisor::quit_and_wait` call sites in `runtime.rs` pass `self.sidecar_timings().quit_grace`; the field is initialised to `Default`; the only `set_sidecar_timings` caller in the repo is `tests/suite/p42_camera_runtime.rs:104`. No other production code changed. |
| C4(a) | `cargo test -p psm-testkit --test suite p66_approve_on_arrival_and_session_order::` → 6 passed, 806 filtered out (exactly p66's six tests). 2 min 35 s, mostly recompiling integrations/testkit under package-level features (backlog note). |
| C4(b) | master plan §6.9 + §7.2, flow.md §4A + log v1.5, templates implement/fix/review/rereview name `--test suite <stem>::`; `grep -E -- '--test [a-z<]'` over those files and `tools/pipeline/*.sh` finds no other form. |
| C5(a) | Workspace suite green twice (above); `cargo fmt --all -- --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `npm run typecheck` exit 0. Implementer's `npm test` stamp (466/466; the known intermittent p33 C4(d) on a second run, backlog since P35) stands per Review mode. Sidecar stub tests run inside the suite (p45). |
| Isolation | `in_own_process` cannot pass vacuously: the parent requires child exit 0 **and** `1 passed` for the exact test name (a wrong name gives `0 passed` → red). |
| A3 (symlinked fake helper) | Production never writes through `<support>/camera-sidecar`: `sony::install` and `refresh_sidecar_copy` write/chmod/sign only temp copies and rename over the path. Holds. |
| H1–H8 | Hold. Only adapter-side code (`psm-integrations` camera) changed; no core, Tauri, pixel or RAW path touched; no live services; no secrets. |
| Assumptions | A1–A7 read; none contradicts the intention (10E MC-31's "wait up to 3 s, then kill" is still 3 s in production). A7 (background cargo runs) is a process deviation, already in the backlog (P68); recurred in this review (run 1 was moved to the background by the tool at its 10-min limit; nothing else ran on cargo meanwhile). |

## Findings
None blocking (B1–B4: none).

## Notes written to backlog
1. With `autotests = false`, a test file not listed as `mod <stem>;` in `tests/suite/main.rs` (or put in `tests/` as before) compiles nowhere and never runs, with no warning. Suggest one folder-vs-module check.
2. `-p psm-testkit` and `--workspace` resolve features differently, so alternating them recompiles integrations/testkit, and every relink pays the macOS first-start check again (measured: 2 min 35 s for a 1.7 s phase run; 16 min 50 s first workspace run vs 7 min 59 s warm). Combined with the p45 sidecar rewrite noted by the implementer.
3. Process recurrence: the first workspace run after a relink exceeds the agent tool's 10-minute foreground limit.
4. Shared fake helper copies accumulate in `target/debug/psm-testkit/` (one per fake build).

## Lessons for the plans
- C1(a)'s "warm build (second run)" was the right wording: the first run after a relink is about twice as long on this Mac. Future speed criteria should name the run condition the same way.
- Review mode's "full suite once" cannot measure a warm-run criterion when the reviewer's first run follows a relink; a plan with a timed whole-suite row should budget two runs for the reviewer.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P70_fast_test_suite_review_r1.md` (this file, new)
- `docs/implementation/narratives/P70_fast_test_suite.md` (new)
- `docs/implementation/plans/P70_fast_test_suite.md` (Review log entry appended)
- `docs/implementation/master_plan.md` (P70 tracker row only → APPROVED)
- `docs/implementation/backlog.md` (4 notes appended)
- Outside git: `.pipeline/p70/review/` (run logs). Side effects of running the suite: `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` rewritten by p45's `build.sh --stub` (gitignored/unchanged in `git status`), build output under `apps/desktop/src-tauri/target/`.
- No production or test code touched; no probes applied; no commits made.
