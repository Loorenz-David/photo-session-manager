# Backlog — LV3_phone_app (reviewer notes; none blocks)

- **LV3 reviewer r1 (note, plan lesson): row LV3·C24(a) contradicts its own fixture.** The row says Images "1 main ·
  1 condition", but U3's session has two Main photos (B's and the extra at 04), and the phone shows the Mac's
  `counts.main` (PH6) unchanged. The test asserts the true "2 main · 1 condition". The phone does what PH6 says.
  Why: the next planner should not copy the slip. Destination: the coordinator (fold into the plan lessons; no code
  change).
- **LV3 reviewer r1 (note): the capture toast needs the camera's `exposed` report to arrive before the session re-read
  that first lists the photo.** `PhoneApp.reconcile` credits only photos that are new in a re-read made after
  `exposed`. Three cases fall outside: (a) a capture path that never reports `exposed` (any camera provider other than
  the direct sidecar, HB5), which never shows a toast; (b) a file that lands before its exposure report (CS6), which
  shows no toast; (c) a photo from another source (the camera's own button, the Mac, a second phone) that lands while
  a phone exposure is still waiting, which gets that capture's toast. Why: the toast is missing or names the other
  photo. Nothing is placed wrongly (the Mac decides placement). Destination: LV4 hardware check (watch the toast on
  real captures). If it misleads in practice, the capture id could be matched to the photo on the server.
- **LV3 reviewer r1 (note): after a rebuild the old App's event hub keeps the carried phone observer.**
  `EventHub::transfer_target_to` copies `carried` to the new hub and leaves the old hub's list as it is. If the old
  App emits anything while it shuts down, phones get extra `session` messages. Why: harmless, because a phone only
  re-reads `GET /session`. It is noted so nobody later relies on the old hub going silent. Destination: next touch of
  `crates/integrations/src/lib.rs` (drain `carried` in the transfer).
- **LV3 reviewer r1 (note, plan lesson): jsdom in this repo has no `PointerEvent`.** `master_plan_live.md` §3 calls the
  Pointer API "standard" in jsdom. `helpers.ts` sends mouse events of the pointer types with a `pointerId`. Why: the
  next phone or UI plan should not promise it. Destination: the coordinator (amend `master_plan_live.md` §3).
- **LV3 reviewer r1 (note): the overlay tick compares against the overlay of the last render.** `Stage.move` and
  `Preview.move` tick when `floor(value/10)` differs from `floor(state.overlay/10)`, read from the render. Two pointer
  moves batched into one render can tick twice for one 10 % step. Why: an extra 5 ms buzz on Android at most.
  Destination: next touch of `Stage.tsx`/`Preview.tsx` (compare with the last value sent, in a ref).
- **LV3 reviewer r1 (note): the side-by-side look comparison with the prototype was not done by the reviewer.** This
  session ran unattended with no browser. The look was seen only in the implementer's headless-Chrome run against a
  mock. Every threshold and timing was checked against the design README by reading. Why: differences in look are
  notes, never blocking, and need a real screen. Destination: LV4 / the owner's try-out on a real phone.
- **LV3 reviewer r1 (resolved from LV2):** the LV2 notes routed to LV3 are done in U0: phone Delete events (LV3·C26),
  session delete announced (C27), sockets fed across a rebuild (C28), generic 500/404 bodies (C29). The LV2 notes
  left for LV4 (revoke wiring, the LV1 carry-overs) are untouched.
