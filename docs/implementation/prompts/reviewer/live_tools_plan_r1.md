---
plan: live-tools
role: plan
round: 1
date: 2026-11-27
---
# Plan the LV8 try-out corrections and the next phone tools (book 10J LV-D21)

Repository root: `/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager-live` (git worktree, branch
`feature/live-preview`; never touch the `main` checkout, `docs/implementation/STATUS.md` or other tracks' files).

## Context
LV1–LV8 are APPROVED, LV4 (hardware check) DONE; the owner is trying LV8 on the phone and sent corrections and new
tools. Authority: book `docs/intention/book/10J_live_preview.md` **LV-D21** (changelog D21), with LV-D19/LV-D20; book
`10D_composition_and_framing.md` (MC-27 composition, MC-28 render, the editor's Overlay guides and straightening grid)
for everything about reframing. Read `docs/implementation/STATUS_live.md`, `master_plan_live.md` (§4 names, §5
contracts incl. PH4/PH6/PH8 and the LV8 amendments), the plans/narratives/handoffs of LV6, LV7, LV8, the phone code
under `apps/desktop/src/phone/`, the phone routes and `phone_view`, and the Mac's composition code (the core service
that stores a composition and triggers the re-render; the desktop editor's guides and grid, to reuse names and
geometry). Design README (`claude_design/live_preview/design_handoff_shoot_companion/README.md`, local) for look and
tokens; no new prototype (owner: build from the description, judge in use).

## Write
Two milestone plans, same format as LV6/LV8 (frontmatter incl. `depends_on`; units ≤ 8 criteria; row/boundary/input/
exact outcome/trace; core mechanisms; delegated choices; manual try-out; guide files; empty Review log), both
**`implementer: fable`** (phone UI; Fable may do small server work in a first unit, H8: logic in core/services, routes
thin, PH8 announcements fire):
- **`plans/LV9_preview_corrections.md`** (`depends_on: [LV8_phone_sort_and_arrange]`) — LV-D21 (1)–(4):
  1. The loading cue (LV7) on a tile whose photo is on its way, in **every** strip incl. the Preview's row, from the
     shutter (the exposure bound to that shot) until its picture — find why the owner saw none (is the tile known
     before the photo row exists? the bound-but-pending state from LV5 CS4a may need to reach the phone view).
  2. No flicker when tiles move or are removed in either strip (LV8 optimistic drop / swipe / server `session`
     message): find the cause (double render between optimistic state and the server's answer, keys, image reload on
     reorder) and make each move settle in one visual step; rows that would fail on a blink (render sequence).
  3. A lifted template-shot photo leaves its template example picture in its place (not an empty gap).
  4. The Preview without header and action-button row: photo, row, a ⋯ button bottom-left opening a bottom drawer
     (info that the header showed + Retake; room for more), a close icon bottom-right. Swipes, hold-for-template and
     the row keep working.
- **`plans/LV10_zoom_reframe_guides.md`** (`depends_on: [LV9_preview_corrections]`) — LV-D21 (5)–(7):
  5. Pinch to zoom in the Preview (two fingers; one-finger pan while zoomed; swipes/drag disabled while zoomed;
     double-tap or pinch-out to reset — decide and name), as one gesture module reused by (6).
  6. Reframe from the Preview's drawer: edits the photo's composition exactly as 10D MC-27 (Normal/Split, shape ratio,
     quarter turns, flip, straighten, rect), with phone gestures (two fingers scale/rotate, one or two fingers move),
     a fine straightening grid while rotating, composition guides (Thirds, Grid, Golden ratio, Diagonals) inside the
     frame; Reset, Done/Cancel; saved through a phone route into the same core service the Mac uses (validation as
     MC-27; the Mac re-renders as for its own edits; undo on the Mac). Rect constraints (no empty corners when
     straightened) enforced while dragging.
  7. Composition guides on the live view: on/off and which guide, from the live screen's ⋯ menu; remembered on the
     phone; drawn by the UI only (H3).
  Split the server part (composition route) into its own first unit; consider `difficulty: high` for LV10.
Amend `master_plan_live.md` (dated, LV-D21) and `phases_live.txt` (append LV9, LV10). Handoff
`docs/implementation/handoffs/reviewer/handoff_live_tools_plan_r1.md` (frontmatter `plan: live-tools`, `role: plan`,
`round: 1`, `state`, `date`, `actor`): files, counts, plain-words map, `⚠ OWNER DECISIONS REQUIRED (n)` with cards
verbatim, only for what LV-D21 and 10D truly leave open. Commit nothing. Change no code. Do not edit the book.

## Owner's standing constraints
Reliability of touch interactions first (pointer events with capture, explicit state machines, cancels named); jsdom
has no `PointerEvent` (mouse events of pointer types with `pointerId`; two-finger gestures need two `pointerId`s); hard
rules as in the master plan and guide; the suite never needs the camera or a phone. Few units, one review per milestone,
gate once at the end (`npm run build` first; rebuild nothing in `apps/desktop/src-tauri/binaries/` that is not the
stub the gate needs). Guide updates in the same change.

Final message, first line exactly:
`HANDOFF: docs/implementation/handoffs/reviewer/handoff_live_tools_plan_r1.md | STATE: <DONE|OWNER_DECISIONS_PENDING|BLOCKED> | OWNER_CARDS: <n>`
