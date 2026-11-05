---
plan: P37_editor_chrome
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer)
---
# P37 editor chrome — review r1

Verdict **APPROVED_WITH_NOTES**. Gate passed: tracker `IMPLEMENTED`, checkpoint `4b4fbba` in `git log`. No blocking finding (B1–B4).
Four non-blocking notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Verified correct

**Perimeter.** `git show --stat 4b4fbba` lists exactly the 17 paths the handoff declares. These are the 11 source paths of the plan's
list, the new `p37_editor_chrome.test.tsx`, the two adjusted tests (`p32_controls`, `p36_controls`), plus the tracker, plan Review log
and handoff. No extra paths and none missing.

**Suite (reviewer run, tree `4b4fbba`, clean `git status`).** `npm run typecheck` exit 0. `npm test`: 47 files, 348/348 green on the
first run. `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`: green (see
Stamp below). The phase makes no Rust change.

**Rows: production read, then test.** Each row has exactly one test and there are no orphan tests (12 tests ↔ 12 rows).
- **C1(a).** `EditorToolbar` renders five `compose-tool` buttons with `aria-label` = card name, one `aria-hidden` lucide icon and no
  text. `aria-pressed` is set from `loupe`/`snap`, and `aria-disabled` from `!canPaste`. The `tools`/`onTools`/`meter`/`onMeter` props and
  their two buttons are gone, and `ComposeMode` no longer passes them. The test asserts the name list with `toEqual`, icon-only on each,
  the pressed states `false`/`true`, `aria-disabled`, document-wide `queryByRole(...) === null` for both retired names, and `Done` in the
  stage bar.
- **C2(a).** `Hint` renders a card as `strong.hint-name` + ` · kbd.hint-key` (only with a key) + ` — span.hint-description`. Its
  normalised text is exactly `name · key — description` (or `name — description` without a key). All 32 T2 entries are passed as cards
  (`CompositionSection`, `StraightenControl`, `EditorToolbar`, `ToolsPanel`, `Stage`, `FoldedStrip`, `PhotoPanel`). Workspace strings
  `Compose (R)`, `Sessions`, `Import photos` and `Details (D)` stay strings. The test iterates all 32 entries (30 in the table + rail +
  Fold panel), each scoped `within` its place by role and name. It compares with `toBe`, and the T2 texts match the plan character for
  character (checked by reading). The two identical `Done` cards are separated by a different preceding card, so a stale tooltip cannot
  satisfy a row.
- **C3(a).** No meter button. ⌥⌘P toggles `.frame-meter` both ways, and `useComposeKeys` is untouched.
- **C4(a)/(b)/(c).** `ToolsPanel` returns `aside.tools-rail[aria-label=Tools]` holding one button `Show Tools panel` **instead of** the
  panel when hidden. The `hidden` attribute is gone. When shown, `.tools-panel-head` is the first child, holding the title `Tools` and
  `Hide Tools panel`. Both call `tools.toggleHidden` (`useToolsPanelState` unchanged, so `tools.hidden` is `'1'` or removed). The tests
  assert first-child structure, the rail's next sibling `.stage-column`, one button by name list, the storage value, ⌘T both ways, and the
  remembered rail plus the clean exit to `.stage-column`.
- **C5(a).** The drawer toggle has `aria-expanded`/`aria-controls`, and its body renders only while open, with three icon-only buttons in
  order. The `set` calls are unchanged. The test asserts both states, document-wide absence of the three names before opening, one
  `setComposition` call, `quarter_turns 1` and the `ROTATED` rect ± 1e-6.
- **C5(b).** The Turn button is a sibling of `#compose-shape` inside `div.compose-field`, which uses `label htmlFor`. The select keeps
  its `blur()` and P36 C7(a) stays green. The test asserts parent identity, icon-only, one call, `orientation 'turned'`, `shape '1:1'`
  and the unchanged rect.
- **C6(a).** The switch moved into `.compose-degrees` after the `°` span. It is icon-only with `role=switch`. The test asserts parent
  identity with the degrees field, `aria-checked` true → false, and `compose.straightenGrid === '0'`.
- **C7(a)/(b).** The pad renders only while open, `role=group`, with DOM order up, left, down, right. The click handler is unchanged
  (`dx = −x·step`). The CSS has `grid-template-areas: ". up ." "left down right"`. The tests assert both states, the order, icon-only,
  and the three saved `x` values with `y = 0` and **exactly three** calls (the clamped Nudge up adds none).
- **C8(a).** `rotateFlipOpen`/`nudgeOpen` live in `ComposeMode` `useState(false)`. `ComposeMode` is not keyed (`WorkspacePage.tsx:382`),
  so it survives `]`, ⌘T and section folding, and it remounts on Return + R. **Changed-input probe** (temporary test file, deleted,
  production untouched): with both drawers open, `]` really changes `.stage-place` (photo 1 → 2) and both toggles still read
  `aria-expanded="true"`. So the row's "kept while switching photos" clause is exercised by a real switch.

**Adjusted tests.** Only the edits Read first names: P36 C5(a), C6(f), C6(g) were deleted, C5(b) has the new card text, and C6(h) opens
the pad first. P32 C3(c) opens the drawer before Rotate right and before Flip. No other test changed.

**Assumptions** (rail 36 px; visible short labels on the drawer toggles; card typography). All fall inside the plan's Delegated choices,
none contradicts 10D, and all are reported in the narrative.

**Hard constraints H1–H8.** The diff is frontend chrome only: no external writes (H1), no file operations (H2), no pixel work (H3), no
state or idempotency change (H4), no other app driven (H5), nothing commerce-related (H6), no platform concept in core (H7), no Tauri or
business logic in the UI (H8). No secrets, no live services. Hold.

**Stamp (reviewer, clean `4b4fbba`).** `cargo test --workspace` exit 0, 532 passed / 0 failed; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean; `npm run typecheck` exit 0; `npm test` 348/348.

## Findings

None blocking. (No B1: the behaviour matches 10D §What the photographer sees and the plan rows. No B2: every row's test observes its
outcome at its boundary and fails on a wrong one. No B3. No B4.)

## Notes written to backlog
1. **P37 (chrome).** The stage bar's Item details button keeps a native `title` and the ⋯ button has no hover. Both sit outside T2 but
   in the editor's stage bar.
2. **P37 (layout).** `.hint-card` is `nowrap` and long cards run about 85 characters. The rail width, the pad layout and the card
   typography were checked in jsdom only.
3. **P37 (resolves).** The dead `.compose-straighten-grid + path` and `.compose-readout` rules from the P36 backlog row were dropped.
4. **P37 → P33.** The `p33_split_grid` C4(d) flake recurred once in the implementer's run and is still unfixed.

## Carry-forward
| Note | Destination |
|---|---|
| 1 Item details / ⋯ hover | next phase touching `Stage.tsx` (or a book line adding them to T2) |
| 2 layout in WebKit | owner try-out step 8; WKWebView probe if anything looks off |
| 4 P33 flake | next phase touching the compose/workspace test harness |

## Lessons for the plans
- None material. The plan's Read-first list of tests that change their expectation, with line numbers, made the perimeter check
  mechanical. Keep that pattern.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P37_editor_chrome_review_r1.md` (this file)
- `docs/implementation/narratives/P37_editor_chrome.md`
- `docs/implementation/backlog.md` (4 rows appended)
- `docs/implementation/master_plan.md` (P37 tracker row only)
- `docs/implementation/plans/P37_editor_chrome.md` (Review log entry)
- Probe: `apps/desktop/src/features/compose/__tests__/zz_reviewer_probe.test.tsx` was created, run and deleted. `git status` was clean
  afterwards. No production file was touched.
