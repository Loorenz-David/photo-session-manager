---
plan: live-tools
role: plan
round: 1
state: DONE
date: 2026-11-27
actor: opus (pipeline-planner, Claude Opus 5.5)
---
# Handoff — live tools plan r1 (book 10J LV-D21 → LV9, LV10)

**Summary.** Gate passed: book `10J` RATIFIED (D18), LV-D21 recorded by the owner (changelog D21); master plan §5 stands in
for the mechanism inventory on this track (§3, owner 2026-11-26). Two milestone plans written in LV8's format, both
`implementer: fable`, projection off: **`LV9_preview_corrections`** (4 units, 13 criteria, 44 rows; `difficulty: high`)
and **`LV10_zoom_reframe_guides`** (4 units, 16 criteria, 53 rows; `difficulty: high`). Master plan amended to v1.3
(dated, LV-D21); `phases_live.txt` appended. No code, no book, no STATUS file touched; nothing committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner: LV-D21 and 10D settle every point; the smallest consistent choices are recorded as planning
points P1–P7 (LV9) and P1–P10 (LV10) for the narratives' "Choices made for you".

## Write perimeter (every file this session wrote)
| File | Change |
|---|---|
| `docs/implementation/plans/LV9_preview_corrections.md` | new |
| `docs/implementation/plans/LV10_zoom_reframe_guides.md` | new |
| `docs/implementation/master_plan_live.md` | amended v1.3: header + amendment note; §2 (two rows, totals); §3 (implementers, projection); §4.2, §4.3, §4.5, §4.7 (names); §5 new PH4b, PH6b, PH6c, PH9.5a, PH10, PH11, PH12, PH7 amended in place (preview URL), Invariant PH9 updated; §6 texts v1.3; §9 sequencing |
| `docs/implementation/phases_live.txt` | appended a comment line, `LV9_preview_corrections`, `LV10_zoom_reframe_guides` |
| `docs/implementation/handoffs/reviewer/handoff_live_tools_plan_r1.md` | this file |
No archgraph in this repo (no `.archgraph/`); no tool-recorded state. Not touched: the book, `STATUS.md`, `STATUS_live.md`,
other tracks' files, other roles' prompts/handoffs, the LV4 plan, the `main` checkout, any code or binary.

## Counts (derived by `grep -cE "^\| LV<n>·C[0-9]+\([a-z]\)"` and distinct `LV<n>·C<k>` on each plan file)
| Milestone | Units (criteria / rows) | Criteria | Rows |
|---|---|---|---|
| LV9_preview_corrections | U1 3/8 (Rust) · U2 3/10 · U3 4/17 · U4 3/9 | 13 | 44 |
| LV10_zoom_reframe_guides | U1 4/9 (Rust) · U2 4/13 · U3 5/18 · U4 3/13 | 16 | 53 |
Track totals after v1.3: 10 milestones, 160 criteria, 499 rows (master plan §2). Every unit ≤ 8 criteria.

## Plain-words map (what each milestone gives the owner)
- **LV9 U1** — the Mac tells the phone which shot (or which group) has a photo on its way, from the shutter until the
  photo exists (CS4a's existing set, read in `phone_view`; two fields).
- **LV9 U2** — those tiles shimmer in both strips (the live screen's and the Preview's), Condition/extra captures get a
  waiting tile at their group's end; a picture that gave up stops shimmering; the shimmer is a little stronger.
- **LV9 U3** — moves settle in one step: tiles follow their photos (no old picture), pictures are not refetched when only
  the composition revision changed, swipes show at once like drops, quick swipes/drops stack without blinking back; a
  lifted template photo leaves its grey template picture in place.
- **LV9 U4** — the Preview is the photo, the strip, ⋯ (details + Retake) and ×; Move image leaves.
- **LV10 U1** — the phone view carries each photo's composition (when Capture One's full-frame preview exists) and the
  default shape; two routes save or reset a composition through the Mac's `CompositionService`, refusing a stale revision.
- **LV10 U2** — pinch to zoom (1–6×), pan, double-tap; photos shown as composed on the phone.
- **LV10 U3** — the reframe editor (Mac model: frame fixed, photo moves), one/two-finger gestures within MC-27 at every
  move, fine grid while straightening, guides, Normal/Split, shape, turn, rotate/flip, straighten, Reset, Cancel.
- **LV10 U4** — Done saves into the Mac (undo there), refusals handled; guides on live from the menu, remembered.

## Findings that shaped the plans (for the coordinator; evidence by reading)
1. **Why the owner saw no loading cue (LV-D21 (1)).** The LV7 cue is `PhotoImage`'s placeholder, which exists only after
   the photo row and before its preview. From shutter to row there is no phone field (a shot shows its template picture,
   a Condition capture has no tile), and right after the row `OriginalPreviewJob` (`crates/integrations/src/lib.rs:1658`)
   sets the camera's embedded preview and emits `PhotoChanged`, so the cue lasted a blink. Fix = PH6b + LV9·U2.
2. **Flicker causes (LV-D21 (2)).** F1 shot tiles keyed by container (`state.ts::dockTiles`); F2 `r=<revision>` in
   `api.ts::previewUrl` and `PhotoImage`'s key, while a move into a container raises the revision
   (`template_placement::apply_frame`); F3 swipes not optimistic (LV8 PH9.5 note); F4 the 8 s timer survives a successful
   answer (LV8 backlog note) and a second drop/swipe is computed from the Mac's view (`PhoneApp.drop`: `applyDrop(view, …)`).
   Fix = LV9·U3 with element-identity rows (DOM-level frames cannot see a source swapped inside one element) and
   `stripFrames` sequence rows.
3. **Composed display is a prerequisite of reframing on the phone.** Today the phone draws the full frame with
   `object-fit: cover`; a phone reframe would be invisible on the phone. LV10·U2 draws composed with the Mac's
   `composedLayout` (10D §The result is shown), only where PH6c gives a composition (adjusted preview present), so the
   camera's own (possibly un-oriented) preview is never used as a frame.
4. **Geometry is imported, not copied.** `features/compose/{geometry, refit, overlays, composed}.ts` are pure (type-only
   import of `@/services/compositions`); the vite alias `@` serves the phone entry. Review mode checks that nothing in
   `src/phone` pulls `@/services/bindings`/`@tauri-apps/*` and that the built phone chunk has no `__TAURI`.
5. **Superseded by v1.3:** LV8's PH9.5 single pending drop and "swipes not shown ahead" (→ PH9.5a); LV8·C15(d) and the
   8 s-timer backlog note (→ LV9·C9(b)); LV8·C8(e) (→ LV9·C11(a)); the LV7 gave-up note (→ LV9·C6). Test edits each
   milestone must list are named in its plan (preview URLs lose `&r=` in five files; Move-drawer tests removed).

## Points the coordinator may want to watch
- **P7 of LV9 (Move image leaves the Preview)** is a literal reading of LV-D21 (4) ("actions (Retake now; more later)");
  drag and swipes reach every Move target. It is reversible in one line; it is stated in the narrative's choices, not a
  card.
- **LV10's gesture convention** (photo follows the fingers: spread = closer; the Mac's model) is P4; if the owner meant
  "the frame grows when I spread", it is one sign in `reframe.ts`.
- LV9·U1 and LV10·U1 are Rust units run by Fable (allowed by the prompt; H8 kept: rules in core, routes thin).
- Fixture numbers in LV10 rows (rects, frame box 24/89/342, zoom translates) were computed from the named functions and
  the stub sizes; an implementer finding a different stub size recomputes from the same formulas (each row names it).
- The book (`10J`) is not edited: LV-D21 already covers everything; PH6b/PH6c/PH10–PH12 are planning contracts.
