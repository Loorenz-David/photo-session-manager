---
plan: live-sort
role: plan
round: 1
date: 2026-11-27
---
# Plan "sort and arrange on the phone" (book 10J LV-D20)

Repository root: `/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager-live` (git worktree, branch
`feature/live-preview`; never touch the `main` checkout, `docs/implementation/STATUS.md` or other tracks' files).

## Context
The live preview track is built and hardware-checked: LV1–LV3, fixes LV5, LV6, LV7 APPROVED; LV4 DONE. Read
`docs/implementation/STATUS_live.md`, `master_plan_live.md` (§4 names, §5 contracts incl. PH4 routes, PH6 view, PH8
announcements, CS current shot), the plans and narratives of LV3, LV6, LV7, and the phone code under
`apps/desktop/src/phone/` (Dock, Preview, Move drawer, gestures, PhotoImage, api, state) and the phone routes
`crates/integrations/src/phone/routes.rs`, `crates/core/src/services/phone_view.rs`, the review/order services the
Mac uses (`crates/core/src/services/review.rs`, template hand changes). Book: `docs/intention/book/10J_live_preview.md`
**LV-D20** (owner 2026-11-27, changelog D20) — the authority for this work — with LV-D19 and card LV8; ordering rules in
book chapter 06 and 10G §6. The design README (`claude_design/live_preview/design_handoff_shoot_companion/README.md`,
local) stays the authority for look, tokens and haptics; there is **no new design prototype** (owner: build from the
description, judge in use).

## Write
- One milestone plan, `docs/implementation/plans/LV8_phone_sort_and_arrange.md`, same format as LV6 (frontmatter
  `phase`, `title`, `difficulty`, `projection`, `depends_on: [LV7_tile_loading_cue]`, **`implementer: fable`**; units ≤
  8 criteria; criteria tables row/boundary/input/exact outcome/trace; core mechanisms; delegated choices; manual try-out;
  guide files; empty Review log). If server routes or core services must change (e.g. a phone route to classify Main /
  Condition, to reorder with insert-and-shift, to move a template-shot photo to Main), put those in a **first unit** of
  the same milestone (Fable may do small server work; keep H8: logic in core/services, routes thin; PH8 announcements
  must fire) — or, if the server part is large, a separate milestone `LV8a_*` with the default implementer before it;
  your call, say why.
- Amend `master_plan_live.md` (new/changed PH routes and rules, dated, LV-D20) and `phases_live.txt` (append).
- Handoff `docs/implementation/handoffs/reviewer/handoff_live_sort_plan_r1.md` (frontmatter `plan: live-sort`,
  `role: plan`, `round: 1`, `state`, `date`, `actor`): files, counts, plain-words map (what the owner can try after),
  `⚠ OWNER DECISIONS REQUIRED (n)` with cards verbatim — only for what LV-D20 truly leaves open.
Commit nothing. Change no code. Do not edit the book.

## What LV-D20 asks (make each exact in rows)
1. **Swipe to classify in the full-screen Preview:** left = Deny (undoable; a template shot it filled becomes empty and
   current — the Retake semantics of MC-LV3), right = Main, up = Condition; after a swipe the next photo shows. Right on a
   Condition photo → Main at the end; right on a Main photo outside the template → nothing; **right on a photo in a
   template shot → a confirmation "Move this photo to Main, outside its template order?" Move / Cancel**; up on a
   template-shot photo → Condition (shot empty and current). Distinguish swipe from the Preview's existing hold (template
   overlay) and from the row's gestures; thresholds named, with visual feedback while swiping (the photo follows the
   finger, a hint of the outcome), and a cancel when released short.
2. **The Preview shows the live screen's photo row** (the same strip), to move between photos and to arrange.
3. **Drag and drop** in the live screen's strip and in the Preview's row: **press and hold to lift** (the tile grows, a
   lifted look, haptic where available), drag, release to drop; **swap between template shots**; **insert and shift among
   Main photos outside the template and among Condition photos** (others slide aside while dragging); what happens when
   dragging across groups (template ↔ extra Main ↔ Condition) must be decided by the book's rules or carded; auto-scroll
   at the strip's edges; a still tap still opens/selects (LV-D19 (1)); horizontal scroll still works without a hold.
   **Reliability is the owner's first requirement**: name the state machine (idle → pressing → lifted → dragging →
   dropping/cancelled), what cancels a drag (pointer cancel, visibility change, a session message changing the list
   mid-drag, a second finger), what the list shows while the server confirms (optimistic or not, and the rollback), and
   rows for each. Pointer events with capture; no HTML5 drag-and-drop API (unreliable on iOS).
4. Remove = left swipe (Deny); Retake stays.
Tests at the component boundary as LV3/LV6 (jsdom has no `PointerEvent`: mouse events of pointer types with
`pointerId`, fake timers); server rows at the route boundary.

## Owner's standing constraints
Hard rules as in `master_plan_live.md` and `docs/guide/03_rules_and_principles.md`; the suite never needs the camera or
a phone. Few units, one review per milestone, gate once at the end (`npm run build` first). Guide updates in the same
change.

Final message, first line exactly:
`HANDOFF: docs/implementation/handoffs/reviewer/handoff_live_sort_plan_r1.md | STATE: <DONE|OWNER_DECISIONS_PENDING|BLOCKED> | OWNER_CARDS: <n>`
