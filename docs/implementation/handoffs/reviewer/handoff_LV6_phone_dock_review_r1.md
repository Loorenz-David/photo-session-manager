---
plan: LV6_phone_dock
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-26
actor: opus
---
# Handoff: LV6 phone dock, review r1

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. Six notes went to the backlog. Nothing needs the owner.

Gate: the plan's tracker read `IMPLEMENTED`, and checkpoints `a4df2ebd` (U1), `b0728c90` (U2) and `da576109` (row C3(b))
are in `git log`.

## Verified correct
- **Perimeter.** `git show --stat` of the three checkpoints lists exactly the handoff's "Files changed". That is the
  plan's lists plus the declared test-only deviations (`lv3_preview`, `lv3_drawers`, `lv3_main`, `lv3_geometry`, all
  forced by the new layout, the required `preview` field or the 55 px lock). No production file is outside the plan.
  `996ee63d` and `b76f3286` touch only the implementer handoff and the plan. `67b7dcff` is the orchestrator's.
- **Checks (Review mode, once, on `b815e3b2`, clean tree).**
  - `cd apps/desktop && npx vitest run src/phone`: 9 files, 103 passed.
  - `npm run typecheck`: exit 0.
  - `npm run build`: built (only the usual chunk-size warning).
  - The implementer's full-gate stamp on `da576109` stands: Rust 2760 passed, clippy, fmt, sidecar, UI 1058 passed.
    Later commits change documents only.
- **Rows (code read, then test; each would fail on a wrong outcome):**
  - C1(a): loads the real `phone.css` into jsdom. It asserts the computed 56×56 size and 12 px radius of all 6 tiles,
    the row gap of 8 px, the strip at 84 px, and that no dock element is taller than 84 px. The old 260 px scroller
    fails this.
  - C2(a)/(b): taps through `PhoneApp`'s dock. It asserts the exact kicker, vibrate calls equal to `[[15]]`, and no POST.
  - C2(c): a 600 ms hold changes no tile and shows no label. Release opens B's preview, and vibrate calls equal exactly
    `[[15]]`. The old hold haptics would fail this.
  - C2(d): a 60 px upward move gives no preview, no POST and no vibration.
  - C3(a): the layout stub subtracts `scrollLeft`. The test asserts +128, nothing selected, then `/current` with
    `{session_id: 7, container_id: C}`, the tile two pitches right of "02".
  - C3(b): 50 px gives "TEMPLATE" after release and `live: false`. 55 px gives "LIVE · LOCKED" 1 s after release and
    `live: true`. `lockProgress(-55) = 1`.
  - C4(a)–(c), C5(a)/(b): assert whole URLs (`&r=3&v=k1`, `&v=k2`), and that no `<img>` exists while waiting or failed.
    Retries are checked at the 1999/2000 ms edges, `&try=1..3` then none for 10 s, and a restart without `try` on `k2`.
    "Preparing preview…" in the Preview, and the placeholder in the Move drawer with no request for photo 201.
  - C6(a)/(b): the exact amended §6 text, and the token removed, for the 401 at start and the socket's 4401.
- **Review-mode reading checks.**
  - No hold, magnify, label, edge-scroll or drag-up code is left in `Dock.tsx` or `gestures.ts` (one comment names the
    old design).
  - Exactly the six listed LV3 tests are removed. The nine kept ones exist and pass. `lv3_c1c` is re-texted and the C8
    lock tests are moved to 55 px, as the plan's Notes ask.
  - `previewUrl` is called only by `PhotoImage`, which draws every preview `<img>` (dock, Preview, Move drawer). A
    failed image is unmounted, so the broken-image glyph cannot show.
  - No secure-context API or absolute URL under `src/phone/`. `previewUrl` is relative.
  - `TEXT.removed` matches `master_plan_live.md` §6 v1.1 verbatim, and serves both 401 and 4401.
  - The Mac side already sends `preview: Option<String>` (`crates/core/src/services/phone_view.rs`, LV5·U2).
- **Assumptions 1–8.** None contradicts book 10J LV-D19, PH6a or the plan's points P1–P4. Each is reported in plain
  words in the narrative.
- **H1–H8.**
  - Phone UI and CSS only.
  - No external write (H1) and no file touched (H2).
  - No pixel processing (H3): the grey look is the existing CSS display filter.
  - No platform names in core (H7), and no business logic in the UI (H8).
  - No secrets, and no live service is used by any test.
- **Guide (flow §7A).** `docs/guide/01_product.md` covers the dock, removal and the 55 px lock.
  `docs/guide/02_architecture.md` covers the image rule and `DOCK`. Both are current.

## Findings
None blocking.

## Notes written to backlog
To `docs/implementation/backlog_live/LV6_phone_dock.md` (the live track's backlog, as the plan's Review mode names):
1. A picture that failed four times stays grey until its preview or revision changes.
2. The Preview's opaque placeholder hides the template on hold while the photo is not ready.
3. The dock layout stub in `helpers.ts` encodes the 64 px pitch by hand, so tap coordinates in four test files follow
   any size change.
4. Two pairs of tests assert the same outcome: `lv3_c1c`/`lv6_c6a` and `lv3_c8b`/`lv6_c3b`.
5. Plan fold: the plan header and `master_plan_live.md` §2 say 13 rows (14 now), and the design README still says
   110 px for the lock.
6. C3(a)'s scroll is exact only because jsdom does not clamp `scrollLeft`. The real end stop is for LV4.

## Write perimeter (this session)
- `docs/implementation/plans/LV6_phone_dock.md`: the Tracker line, and the Review log entry "review r1".
- `docs/implementation/backlog_live/LV6_phone_dock.md`: new.
- `docs/implementation/narratives/LV6_phone_dock.md`: new.
- This handoff.
- `apps/desktop/dist/` was rebuilt by `npm run build`. It is git-ignored.
- No production or test file was edited, and no probe was applied. Nothing was committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
