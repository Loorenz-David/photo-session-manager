# Backlog: LV10_zoom_reframe_guides (reviewer notes; none of them blocks)

- **LV10 reviewer r1 (note): a photo that needs review cannot be confirmed from the phone.**
  - PH12 says "values equal to the open values → close, no request". For a `needs_review` photo the editor opens on
    the refitted values, so Done without a change closes silently. The photo stays "needs review", and export and
    publish still refuse it. On the Mac, Return in the editor confirms it (10D MC-27 §Revisions: "needs_review until
    confirmed").
  - Why it matters: the photographer may think the framing is confirmed when it is not. Nothing wrong is saved, but
    a later export or publish says "Re-frame first" for a photo they believed they had handled.
  - Destination: one PH12 line, "…or the photo needs review → the save route", plus one row. The server already
    handles it: `set_at_revision` falls through to `set` when the state is not `ok`. The phone change is one
    condition in `ReframeEditor.tsx::done`.
- **LV10 reviewer r1 (note): `frame_unknown` and `invalid_value` on the two new routes have no row and no test.**
  - U1's Goal names 409 `frame_unknown` and 400 `invalid_value` (an unknown enum spelling). The mappings exist
    (`wire::error_code`, `routes::body`), but no criterion row covers them.
  - Why it matters: the phone shows "That did not work. Try again." for both, which is the PH12 fallback. A
    regression in either mapping would go unseen.
  - Destination: a later phone milestone may add two rows if these paths change.
- **LV10 reviewer r1 (note): nothing of LV10 has been seen on a real phone or in a browser.**
  - Everything is proven in jsdom (two fingers = two pointer ids, stubbed rectangles). Never drawn by a real browser:
    the composed picture's CSS (`cqw`/`cqh` container units), the zoom layer and its 200 ms settle, the editor's
    look, the grid and the guide lines over light and dark photos, and real iOS/Android multi-touch, including when
    the system takes a touch away.
  - Why it matters: the owner's try-out is the first time any of this is seen.
- **LV10 reviewer r1 (note): the bounded moves feel different from the Mac's editor (PH12 verbatim).**
  - Right after straightening, the frame touches the image's edge, so one finger barely moves the photo until it is
    zoomed in a little. Zooming out at an edge stops there, while the Mac's editor slides the frame inward instead.
    A diagonal move that reaches one edge stops both ways.
  - Why it matters: it may feel stuck in use. If the owner says so, `panWithin` could slide along the edge, and
    `zoomWithin` could recentre the frame instead of stopping (a PH12 change).
- **LV10 reviewer r1 (note): the editor rows read the rect at display precision.**
  - The editor's transform is written in 0.01 px steps, so rows C10(a), C10(b) and C11(a) check the rect to about
    1e-5, not the plan's 1e-6. The 1e-6 values are pinned in `lv10_core` (the bounded moves) and in C14(a)'s request
    body.
  - Why it matters: none for behaviour. A future plan can state "display precision" for rows read from a transform.
- **LV10 reviewer r1 (note, process): checkpoint `1764d2d6` was committed with typecheck red.**
  - An unused test helper. The implementer committed before reading the check's result, and `21ffb636` fixed it.
    The closing gate ran on `764ba540`.
  - Why it matters: a red checkpoint in history. The doctrine says to commit after the unit's test commands pass.
