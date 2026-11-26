# Backlog: LV5_live_fixes (reviewer notes; none of them blocks)

- **LV5 reviewer r1 (note): C3(a)/(b) do not use the row's clock offsets.**
  - The row says s3's file is filed at 5.6 s and s4's at 7.6 s, each taken in 1.0 s after filing.
  - The tests file s3 at 6.0 s and s4 at 8.0 s, and take files in 0–1.0 s after filing.
  - Every binding happens by 4.8 s, so neither bindings nor placements depend on these later times. The order the row
    cares about still holds: s3's file arrives before s2's, and s5 fires while earlier files are filed but not taken
    in.
  - Why it matters: the test reads as the hardware sequence but is not exactly that sequence.
  - Destination: the LV5 fix that touches the file (R1-B1) may align the times; otherwise none.
- **LV5 reviewer r1 (note): the Mac-action tests assert `rev > 0`, not "above every earlier `rev`" (C5–C7).**
  - The rows ask for a `session` message whose `rev` is above every earlier `rev` the client received. The tests
    check only `rev > 0`.
  - `rev` comes from the LV2 bridge, which LV5 did not change and LV2's own tests cover.
  - Why it matters: low. A broken counter would show in LV2's tests first.
  - Destination: none, unless the bridge changes.
- **LV5 reviewer r1 (note): Settings → Phone shows an error toast every 5 s while the list cannot be read.**
  - The new re-read interval in `PhoneSection.tsx` calls `toast.error` on every failed `phonePairingsList`. If the
    backend keeps failing while Settings is open, a toast appears every 5 s.
  - Why it matters: noise, but only in a state that is already broken.
  - Destination: the next touch of `PhoneSection.tsx`. Toast once per failure streak.
- **LV5 reviewer r1 (note): the implementer handoff's file list is off by one unit.**
  - `crates/testkit/tests/suite/lv2_phone_ws.rs` is listed under U4 but was committed in U3 (`78018009`), as the plan
    says. `main.rs` is not listed for U3.
  - The perimeter union is correct.
  - Why it matters: only for anyone who reconstructs a unit from the handoff.
  - Destination: none.
- **LV5 reviewer r1 (note): the guide 04 coupling for U2 is a paragraph, not a row.**
  - `docs/guide/04_change_impact.md` gained the coupling (a service that changes a session's photos ↔
    `SessionChanged` ↔ the phone `session` message; `PhoneImage.preview` ↔ `apps/desktop/src/phone/api.ts`) as prose
    above the table. The file's other couplings use the table's "If you change… / You must also change… /
    Otherwise… / Check" shape.
  - Why it matters: a reader scanning the table misses it.
  - Destination: LV6, which changes `phone/api.ts` for the token. It can move the paragraph into a row.
