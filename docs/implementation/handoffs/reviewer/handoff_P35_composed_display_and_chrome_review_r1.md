---
plan: P35_composed_display_and_chrome
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer)
---
# Review r1: P35 composed display and chrome

**Summary.** One blocking finding: **F1** (B1, route `production`). On the stage, the composed photo is drawn a quarter of
the frame height too high, because a legacy image rule still applies inside the new composed box. I measured it in
WebKit and verified a one-line CSS correction there. Everything else holds: perimeter, checks, the other 17 rows,
the assumptions and H1–H8.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker row `IMPLEMENTED` (2026-11-05, codex). The checkpoint `c8d3715` ("CHECKPOINT (not approved): implement P35 …") is in `git log`. The handoff
  gives the checkpoint as "HEAD after the checkpoint commit", so I resolved it from the log.

## Verified correct
- **Perimeter.** `git show --stat c8d3715` = exactly the 31 paths the handoff declares (18 source incl. the disclosed `BrowserOverlay.tsx`;
  10 test files incl. `p31_compose_mode.test.tsx`, whose menu expectation changed with the behaviour; 3 process docs). `compose.css`
  unchanged (the plan allowed "if they live there").
- **Checks** (code tree `c8d3715`, HEAD `45252df`, `git status --porcelain` empty). `cargo test --workspace` all `ok` (exit 0),
  `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --all -- --check` clean, `npm run typecheck` clean.
  `npm test`: the first run had 1 failure out of 320 (failing ID not captured, my lapse). Then **12 re-runs at 320/320**: 3 sequential runs
  plus 3 rounds of 3 parallel runs under load. I treat it as an unreproduced flake, not B4, and recorded it as a backlog note.
- **Core mechanism** `composed.ts`, re-derived by hand:
  - **C2(a).** Square `A` on 2560 × 1707: ratio `1707 / 1707`, `1/w = 149.9707 %`, `−x/w = −24.9854 %`.
  - **C2(b).** `S`: `2414.0626 / 1207.0313`, `106.0453 %`, `141.4214 %`, `−3.0227 %`, `−20.7107 %`.
  - **C2(c).** Turned frame 1707 × 2560, rect `(0, .1666, 1, .6668)`: `100 %`, `149.9707 %`, `0 %`, `−24.9854 %`.
  - **C3(c).** All seven cases, under the pixel rule on the turned frame (half away from zero) and with straighten ≠ 0 marking.
- **Transform order** matches MC-27 §Values: turns and flip on the image, straighten on `.composed-frame` about its centre, and the rect
  in unstraightened turned-frame fractions. The camera original gets `imageTransform` then `orientationTransform`, with the swap parity
  `turns + (orientation 5–8)` (MC-20; the C2(d) fixture gives `rotate(0deg) scaleX(1) rotate(90deg)`, swapped).
- **Compose availability.** `canCompose` is the only rule. Button, R (now enabled whenever a photo is on the stage) and double-click all go through
  `enterCompose`, which toasts `COMPOSE_WAITING` with id `compose-waiting`. The menu shows the reason in place of `R`. I ran a changed-input check
  with a temporary test file, run and then deleted: **R alone** and **double-click alone** each produce the toast and no editor.
- **Split for any photo.** The gate is gone in `CompositionSection`, `ComposeMode.setKind`, `WorkspacePage.setKind` and `PhotoMenu`. `outputs.ts` is
  untouched, so an unreviewed photo still fills one place (C4(a) pins `01–02, 03, 04–05, 06`). The Rust guard C4(c) exercises the real
  `CompositionService::set` on an unreviewed photo with a materialised frame.
- **Details.** `detailsOpen` is lifted to `App` exactly like `assignOpen` and reset on `sessionId` change. Header (with a `Hint`), D and the card's
  "Edit details →" open the same `FormDialog`. The stage duplicate is deleted.
- **Sessions.** The icon-only `panel-sessions` sits in `.panel-tools` and in the folded strip, with `aria-pressed`. `.sessions-toggle` and the
  "Show sessions panel" button are gone. Unassigned opens pressed.
- **Apply.** The patch sets `state 'ok'`, `review_reason null`, matching what `save_composition` stores.
- **Rows.** All 18 rows have a test at the boundary they name, and each test fails on a wrong outcome there. The C2 rows assert inline styles and
  attributes, which are correct; F1 sits in the cascade, which jsdom cannot see (plan Notes, "jsdom limits"). Adjusted P31–P34 expectations
  were re-derived (`left = −x/w`: `11.5467 % → −17.3166 %`, `6.4332 % → −9.6479 %`). The P33 stage-bar `waitFor` is a timing fix only.
- **Assumptions** (crop mark at upper right below the unreviewed dot; Sonner default toast; Details reset by an App effect; previews keep their own
  renderer). None contradicts 10D, so none is B1.
- **H1–H8 hold.** The change is frontend only, plus one Rust guard test. No pixel processing (CSS only, H3) and no Tauri dependency in logic (H8).

## Findings

### F1 — blocking (B1) — route: production
- **What is wrong.** Outside the editor, the stage draws the composed photo shifted up by 25 % of the frame height, so the top of the composition is
  cut off and a band of empty stage shows at the bottom.
  - **Cause.** `StageImage` places its layers inside `.composed-frame` but keeps `class="stage-layer …"`. The rule
    `.stage-layer { inset: 0; margin: auto }` (`workspace.css:35`) still applies: `.composed-frame img` (`:88`) overrides only
    `left/top/width/height/max-*`, so `right: 0`, `bottom: 0` and `margin: auto` survive. With `top: 50%`, `height: 100%` and `bottom: 0`
    on an absolutely positioned replaced element, the two auto block margins resolve equal and negative (−¼ H). Then `translate: −50%`
    lifts the image a further half.
  - **Measured in WebKit** (WKWebView probe loading the real `workspace.css`, the C2(a)/C2(b) DOM on a 1000 × 700 stage):
    - **C2(a).** Adjusted img `margin-top: −175px`; img rect top `−175` vs `.composed-frame` top `0` (the box is 700 × 700).
    - **C2(b).** `−176.8px`.
    - **Tile.** `.thumb-image` has no inset or margin, so tiles are exact.
    - **Cascade.** Tailwind's preflight lives in `@layer base` and cannot override the unlayered rule, so the app behaves as the probe does.
  - The camera original (`stage-original`, also `stage-layer`) is affected the same way.
- **Authority.** 10D §What the photographer sees, "The result is shown" ("outside the editor the stage shows the photo as composed … cut to the
  frame"); plan task 2 (images inside `.composed-frame` at `left/top 50%`, `translate −50% −50%`, `width/height 100%`); C2(a)–(d) rows (the
  boundary is the stage; the rows assert the styles that are meant to produce this geometry).
- **Correction.** Add `right: auto; bottom: auto; margin: 0;` to `.composed-frame img`. I verified it in the same probe: the img rect equals the
  frame rect on both fixtures, and tiles are unchanged. Alternatively, do not give the layers the `stage-layer` class inside the composed box.
  No new test is required (jsdom has no layout).
- **Files the fix may touch.** `apps/desktop/src/features/workspace/workspace.css`; optionally `apps/desktop/src/features/workspace/StageImage.tsx`.

## Notes written to backlog
1. **C1(b) row.** A deduplicated toast means "exactly one toast after each input" cannot tell a no-op R or double-click apart. My isolated probe shows
   both work. Plan lesson.
2. **Unreproduced UI flake.** 1 of 320 on the first run; ID not captured; 12 green re-runs.
3. **WebKit probe for layout.** jsdom cannot see F1-class layout defects. A `swift` + `WKWebView` probe measures them on this Mac, which matters for P36.
4. **Stale comment.** The comment in `p33_split_grid.test.tsx` C5(a) still says unreviewed photos cannot be split.
5. **Details before load.** Header Details clicked before the session view loads mutes the workspace keys until the form appears.

## Lessons for the plans
- A phase that moves existing elements into a new layout container should name the **classes those elements keep** and their layout rules. The
  plan's C2 rows asserted the inline styles, which were all correct, while the defect came from an inherited class's `inset`/`margin`. The
  "jsdom limits" note was right, but a WebKit measurement is cheap here and could be a manual-probe line in P36's plan.
- Rows whose inputs share one deduplicated side effect (toast id) need one fresh fixture per input.

## Write perimeter (this session)
- **Documents:** `docs/implementation/master_plan.md` (P35 tracker row only), `docs/implementation/plans/P35_composed_display_and_chrome.md` (Review log
  entry appended), `docs/implementation/backlog.md` (5 rows appended), this handoff.
- **Code:** none. **Probes:** `/tmp/p35_probe/{workspace.css copy, page.html, probe.swift}` (outside the repo). A temporary
  `apps/desktop/src/features/workspace/__tests__/zz_review_probe.test.tsx` was created, run and deleted (`git status --porcelain` was empty
  afterwards, before these document edits). No production file was opened for writing. No database or state side effects.
- **Commits:** none (the runner commits).
