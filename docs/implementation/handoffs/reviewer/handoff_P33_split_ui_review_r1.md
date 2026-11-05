---
plan: P33_split_ui
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer, Claude Opus 5.5)
---
# P33_split_ui — review r1: APPROVED_WITH_NOTES

Gate: tracker `P33_split_ui` = `IMPLEMENTED`; checkpoint `d47b6ce` exists in `git log`. Intention RATIFIED (2026-10-24). Process authority:
`docs/process/flow.md` (no mutation probes, blocking only for B1–B4). The implementation does what the plan asks at every one of its 15 rows.
There are no blocking findings, and four non-blocking notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
**Perimeter.** `git show --stat d47b6ce` lists 22 paths, which equal the handoff's declared perimeter exactly. Three of them are beyond the
plan's list and are disclosed: `compose/machine.ts` (the `'kind'` cause task 0 names), `compose/ComposeStage.tsx` (passes `kind` to the frame) and
`compose/ComposeMode.tsx` (the stage photo's kind change goes through the machine). `workspace/FoldedStrip.tsx` is listed in the plan and was not
changed, which is correct because it renders `PhotoGrid`. `git diff d47b6ce HEAD -- apps crates` is empty: no code changed after the checkpoint.

**Suite** (master plan commands, Node 22; code tree = `d47b6ce`, HEAD `80c4e00` differs only in docs):
| Command | Result |
|---|---|
| `cargo test --workspace` | 529 passed / 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 34 files / 291 passed |

**Rows.** For each row I read the production code first, then the test.
| Row | Production that makes it true | Test verdict |
|---|---|---|
| C1(a) | `refit` `case 'kind'` → `reshaped`. `shapeRatio` returns 2 for split, and the `original` short-circuit is skipped while split. Back to Normal uses the kept shape (`free` keeps the rect, `original` gives the whole frame). Shape and orientation changes while split return the field only. | Exact contract values plus plan literals ± 1e-3, plus `validate`. Re-derived: (1) `w = 1707√2/2560 = 0.943052`, `h = 0.707107`, `x = 0.028474`, `y = 0.146447`; (2) the default square at 1e-6; (3) `(0,0,1,1)`; (4) `h = 0.333398`, `y = 0.333301`. Fails on a wrong outcome. |
| C1(b) | `outputs.ts`: `outputCount` (2 only for a split that is not unreviewed), `outputNumbers`, `outputLabel` (zero-padded, en dash). | `toEqual` per photo, exact. |
| C2(a) | S → `ComposeMode.toggleKind` and the control → `CompositionSection.setKind`; both dispatch `SetValues { refit(kind), cause 'kind' }`, and `machine.ts:341–355` handles it as `'set'`. `ComposeFrame` draws `.compose-middle` at `left + width/2`. | One save with exact values. Box 952×476 at (24, 112) and hairline x = 500 re-derived for the 1000×700 stage. A repeated click saves nothing. |
| C2(b) | The same path back to `normal`. | Default square (1e-6), no hairline, square box. |
| C2(c) | `refit` shape/orientation while split → field only. | Rect `toEqual` the split rect after 4:5 and after X. Back-rect is 5:4 with `w_px = 1707·√1.25 = 1908.5`, box 815×652. |
| C2(d) | Trim ratio from `shapeRatio(kind,…)` (`machine.ts:221`) = 2 → locked, nw anchor. | Held box 852×426, save exactly 2:1 (1 px), `x`/`y` unchanged, `w·852/952`. |
| C3(a) | `SplitPreviews`: halves `[x, x+w/2]` and `[x+w/2, x+w]`, the same `previewUrl`, `aspect-ratio 1 / 1`, Left/Right labels. | `data-half`, `data-x0`, `data-x1` exact (1e-9), plus labels and src. |
| C4(a) | `PhotoGrid` → `outputNumbers(photos)` → `PhotoTileFace` `outputLabel`; `.tile-split` at `(x + w/2)·100 %`; Cover still at `number === 1`. | Exact label arrays, hairline position, aria-labels, Cover on tile 1 only. |
| C4(b) | The strip renders `PhotoGrid` (`variant strip`): hairline yes, numbers no. | Order, no numbers, hairline set exactly {1, 3, 5}. |
| C4(c) | `WorkspacePage`: `outputNumbers(placedAt(group, dragging, projectedSlot))`. Drops unchanged. | `03–04` and `05` re-derived; `reorderPhoto(3, 2)` and `(2, 3)`. See note 1 on the `05` substring match. |
| C4(d) | `Stage.placement` → `outputNumbers` / `outputTotal` of the group. | `Main 4–5 of 6` plus four more placements. |
| C5(a) | `entries` shows exactly one of Split/Normal (Split disabled without a frame or for an unreviewed photo). `WorkspacePage.setKind` refits, calls `setComposition` and patches in place. | One call with exact values, renumbering `01–02, 03–04, 05–06, 07`, tile 1 offers Normal and not Split. |
| C5(b) | `onMenu` acts on the selection, and `setKind` skips photos already of the kind. | Exactly one call (photo 2). |
| C5(c) | Condition photos are allowed. | `setComposition(6, split)`; Condition `01–02, 03–04`; Main unchanged. |
| C6(a) | The P31/P32 flows run on `values.kind` unchanged. | Navigation without a save, Esc no-op, drag+Esc reverts to the entry split values, Return exits. |

**Assumptions (flow §6).** Nine assumptions; none contradicts the intention. Assumption 1 (unreviewed photos cannot split, and count one place)
follows 10D's "Allowed for Main and Condition photos" and the plan's task 3. The rest are UI conventions, reported in plain words in the narrative.

**Hard constraints H1–H8.** The phase is frontend only. `setComposition` is the local P28 service: no external write (H1), no RAW touched (H2),
no pixel processing (H3), saves serialised per call with backend validation unchanged (H4), no app automation (H5), works with no commerce
configured (H6), no platform concept in core (H7), no logic moved into Tauri commands (H8). No secret or live service in the diff.

**Over-engineering test.** C3(a) also asserts the clip frame's `style.width/height/left`, which is beyond the plan's assertion surface. If the
implementer later switched to `clip-path`, those lines would need rewriting. I record this as a judgment, not a finding: they are the visible
placement of the half.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
1. **P33 (test):** C4(c)'s `toHaveTextContent('05')` is a substring match, so `04–05` and `05–06` would also pass. It is not B2: every wrong label
   the overlay's slot arithmetic can produce (`03`, `04`, `06`) still fails, and `03–04` pins the split range. The hardening is to assert
   `textContent` with `toBe` (confirmed in `@testing-library/jest-dom` `matches`: `textToMatch.includes(matcher)`).
2. **P33 → P34:** the grid menu reads the photo from before its save answers. A quick second Split duplicates the save, and Split then Normal
   drops the Normal.
3. **P33 → P34:** grid kind saves bypass `useCompositionSaves`' per-photo queue (the same pattern as P32's Reset note).
4. **P33 → P30 row:** the known publish race (Normal ↔ Split between `upload` and `update`) is now one right-click away while a publish runs.

## Lessons for the plans
- Rows asserting a short label (`05`) should say "exact text". jest-dom's `toHaveTextContent(string)` is a substring check, and the implementer used
  it as if it were exact.

## Write perimeter (this session)
`docs/implementation/master_plan.md` (P33 tracker row only), `docs/implementation/plans/P33_split_ui.md` (Review log entry appended),
`docs/implementation/backlog.md` (4 rows appended), `docs/implementation/narratives/P33_split_ui.md` (new), this handoff (new). No production or
test file was edited, and nothing was probed. Commands run: the suite above plus `npx vitest run p33 outputs.test` (18 passed, for the narrative's
step count). Nothing committed (the runner commits).
