---
plan: P35_composed_display_and_chrome
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer)
---
# Re-review r2: P35 composed display and chrome (after fix 1)

**Summary.** F1, the only blocking finding from r1, is resolved as its correction said. The fix touched only `workspace.css`.
I measured it in WebKit against the committed stylesheet, and the composed image now fills its frame exactly. No blocking
item remains, so triage has 0 MUST. Verdict: **APPROVED_WITH_NOTES**.

**Attention, orchestrator (not a P35 defect, but it affects every later runner stamp).** The UI suite flake that r1 could
not identify now has a captured ID: `p33_split_grid.test.tsx > C4(d)`. In this session it failed **4 of 8** full `npm test`
/ `vitest run` runs on the P35 tree, and **1 of 6** on the pre-P35 tree (`c8d3715^`, the approved P34 code). So it predates
P35. It is a P33 test that P35 only touched with a `waitFor`. Details are under N1 below. The suite is hardware-free and Rust
is green, but a UI stamp can now go red at random. A harness fix should be scheduled; I recommend one line in P36's plan
(see N1).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
- **r1 (CHANGES_REQUESTED):** settled the perimeter, the core mechanism `composed.ts` (C2(a)–(c) and C3(c) re-derived by hand),
  the transform order (MC-27, MC-20), Compose availability, Split for any photo, Details, Sessions, Apply clearing Re-frame,
  all 18 rows at their boundaries, the assumptions, and H1–H8. One blocking finding: F1 (layout cascade, B1).
- **fix r2 (codex):** one CSS line.
- **This round:** perimeter, checks, F1, and an adversarial read of the changed seam. Settled areas were not re-verified.

## Perimeter check
- Gate: the tracker row is `IMPLEMENTED` with the fix-1 note. The fix checkpoint is `bd90002` ("CHECKPOINT (not approved): fix P35
  composed stage image positioning"), which I resolved from `git log` as the fix handoff directs.
- `git show --stat bd90002` lists 4 paths. Each is allowed:
  - `apps/desktop/src/features/workspace/workspace.css` (+1/−1). Named in F1's "Files the fix may touch".
  - `docs/implementation/master_plan.md` (P35 tracker row only).
  - `docs/implementation/plans/P35_composed_display_and_chrome.md` (Review log: one line appended).
  - `docs/implementation/handoffs/implementer/handoff_P35_composed_display_and_chrome_fix_r2.md` (new).
- `StageImage.tsx` (the optional file) is unchanged. Between `bd90002` and HEAD `4541438`, only the runner's re-review prompt changed.
  `git status --porcelain` was empty at entry.

## Checks (tree = HEAD `4541438`, code identical to `bd90002`, clean)
The fix changed a tracked file after the r1 stamp, so I ran one entry stamp:
- `cargo test --workspace`: every `test result: ok`, exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `npm run typecheck`: clean.
- `npm test` / `npx vitest run`: **8 full runs, 4 green (320/320) and 4 with 1 failure (319/320)**. The failing ID was the same
  every time: `p33_split_grid.test.tsx > C4 … > C4(d)`.
  - Scope: I ran the full suite more than once because the file alone passes (7/7). The failure appears only under full-suite
    load, so a narrower run could not capture the ID.
  - Comparison: the pre-P35 tree (`git archive c8d3715^ apps/desktop` into `/tmp/p35_base`, with a node_modules symlink) failed
    the same test in 1 of 6 runs. Separately, its `geometry.test.ts` fails to load in that copy, which is a copy artefact, not the
    repo. See N1.
- P35's own tests: `npx vitest run p35 composed.test` passes 19/19. The Rust guard `p35_split_any_photo` is in the green workspace run.

## Findings resolved
### F1 (r1: blocking, B1, route production): RESOLVED as its correction said
- **The change.** `.composed-frame img` now also sets `right: auto; bottom: auto; margin: 0`. That was the first of the two
  corrections r1 offered. No divergence was declared, and none exists.
- **Specificity.** `.composed-frame img` (0,1,1) beats `.stage-layer` (0,1,0) whatever the source order, so the inherited `inset: 0`
  and `margin: auto` are neutralised.
- **Rules that could still interfere.** I checked every rule in `apps/desktop/src` that matches the stage layers:
  - `.stage-original[data-quarter-turn]` (0,2,0) would beat it. But `StageImage` sets `data-quarter-turn` only when there is no
    composed layout (`!layout && …`, line 81), so the rule never applies inside the composed box.
  - `.crossfade` / `[data-visible]` set only `opacity`.
  - `.composed-frame img[data-swapped]` keeps its own width and height, and the new margin reset applies to it as well.
- **Measured in WebKit** (variation on r1's probe: the **committed** `workspace.css` with no inline override, plus a turned
  camera-original case that r1 did not measure). Probe files are in `/tmp/p35_probe_r2/`. The page is 1000 × 700 per stage box.
  | case | img rect (x, y, w, h) | frame rect | margin-top |
  |---|---|---|---|
  | C2(a) adjusted, square on 2560 × 1707 | −25, 0, 1049.8, 700 | −24.9, 0, 1049.8, 700 | 0px (r1: −175px) |
  | C2(b) adjusted, `S` | −30.3, 696.5, 1060.4, 707.1 | −30.2, 696.5, 1060.4, 707.1 | 0px (r1: −176.8px) |
  | camera original, `data-swapped`, `rotate(90deg)` on the turned frame | 150, 1225, 700, 1049.8 | 150, 1225.1, 700, 1049.8 | 0px |
  | tile (`.thumb-image`) | −12.5, 2100, 225, 150 | −12.5, 2100, 225, 150 | 0px (unchanged) |

  Every img rect equals its frame rect to within 0.1 px, which is subpixel rounding.
- **Tests.** No test was added, as F1 directed (jsdom does not lay out). The C2 rows' inline-style assertions are unchanged
  and still green.

## Changed seam, read adversarially
- **What the CSS line reaches.** It affects only images inside `.composed-frame`, which is rendered only by `ComposedPicture`. Its
  users are `StageImage` (`.stage-composed`) and `PhotoTile` (`.tile-composed`). The tile images had no inset or margin, so they
  do not change (measured above). The editor's `.compose-image` is not inside `.composed-frame`, so it is not reached.
- **Nothing else wrong seen in passing on the seam.**

## Triage (flow.md §5)
| item | from | class | reason |
|---|---|---|---|
| F1 | r1 blocking (B1) | resolved | fixed as corrected; WebKit measurement above |
| N1: flaky `p33_split_grid` C4(d) | seen this round | **not a P35 blocking item → backlog** | It is not P35's delta: it reproduces on the approved P34 tree, and the test is a P33 row whose product path (`PhotoTile.onClick → state.setActiveId`) is synchronous. No user-visible wrong outcome has been identified, and no hard constraint is involved. It fails the gate's B4 only intermittently and independently of this phase. One more P35 fix could not legitimately reach it within the phase's perimeter. Recorded with its ID and the comparison evidence so it is no longer "unreproduced". |

0 MUST, so the verdict is `APPROVED_WITH_NOTES`.

## Remaining findings
None blocking. Notes written to `docs/implementation/backlog.md`:
- **N1 (suite flake, captured).** `p33_split_grid.test.tsx` C4(d).
  - **Symptom.** After `openWorkspace` and `fireEvent.click(panelTile(3))`, the stage bar stays on the default active photo
    ("Main 6 of 6 · 1 to review", photo 4) until the 1 s `waitFor` times out (line 124).
  - **Rates.** P35 tree: 4 of 8 full runs. Pre-P35 tree: 1 of 6. The file alone passes.
  - **Cause.** Not proven. The click handler is synchronous, and `reload` keeps a visible active photo. A remount, or a leaked
    async from an earlier test, is the likely shape (harness: `compose/__tests__/harness.tsx` `openWorkspace` sets
    `location.hash`, and its async `hashchange` reaches `App`).
  - **Why it matters.** It can fail any runner stamp at random. P35's `waitFor` did not cure it.
  - **Recommendation.** Before or within P36: wait in `openWorkspace` for the default active photo to be on the stage, not only
    for the tile count, and capture the failing DOM once to confirm the cause.
  - This supersedes r1's backlog row "Unreproduced UI flake". I added a new row rather than editing that one.

## Lessons for the plans
- A flake must be captured by ID the first time it is seen. r1 lost the ID, which cost this round a diagnosis. Runner stamps and
  reviewers should save the failing log before any re-run.
- A WebKit measurement confirmed a CSS-only fix in seconds. P36's plan could list it as a manual-probe step for overlay, strip and
  grid geometry (repeats r1's lesson; now proven on a fix).

## Write perimeter (this session)
- **Documents:**
  - `docs/implementation/master_plan.md` (P35 tracker row only).
  - `docs/implementation/plans/P35_composed_display_and_chrome.md` (Review log entry appended).
  - `docs/implementation/backlog.md` (1 row appended).
  - `docs/implementation/narratives/P35_composed_display_and_chrome.md` (new).
  - This handoff.
- **Code:** none. No production or test file was opened for writing. No mutation probes.
- **Temporary files outside the repo:**
  - `/tmp/p35_probe_r2/` (copy of `workspace.css`, `page.html`, `probe.swift`).
  - `/tmp/p35_base/` (`git archive` of `c8d3715^ apps/desktop`, plus a symlink to the repo's `node_modules`; nothing was
    written into the repo's `node_modules` beyond what any vitest run writes to its gitignored cache).
  - `/tmp/p35_r2_run_*.log` and `/tmp/p35_base_run_*.log`.
- No database or app-state side effects.
- **Commits:** none (the runner commits).
