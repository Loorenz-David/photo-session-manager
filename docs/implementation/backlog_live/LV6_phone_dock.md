# Backlog: LV6_phone_dock (reviewer notes; none of them blocks)

- **LV6 reviewer r1 (note): a picture that failed four times stays grey until its preview changes.**
  - `PhotoImage` stops after the third retry and asks again only when `preview` or `revision` changes (plan P3 allows
    exactly this).
  - Why it matters: a short Mac-side hiccup longer than ~6 s leaves that dock tile grey for the rest of the session,
    unless the photo changes. Opening the Preview tries again, because it is a separate image.
  - Destination: the LV4 hardware check shows whether this happens; if it does, retry once more when the socket
    reconnects or the session is re-read.
- **LV6 reviewer r1 (note): the Preview's placeholder hides the template while the photo is not ready.**
  - Assumption 5: the placeholder is opaque, so hold-for-template shows nothing visible until the photo loads.
  - Why it matters: for the ~3 s after a shot, holding the Preview does nothing visible. The owner may notice this at
    the hardware check.
  - Destination: none unless the owner asks.
- **LV6 reviewer r1 (note): the dock's test layout stub encodes the 64 px pitch and the divider by hand.**
  - `helpers.ts` places tiles at 16 + 64·n (+19 after the divider). It does not read `phone.css`, so a later size
    change has to edit both, and the tap coordinates in `lv3_preview`, `lv3_drawers` and `lv6_*`.
  - Why it matters: the next dock-size change breaks unrelated tap tests in four files.
  - Destination: a later phone phase may derive the stub from the CSS values. Optional.
- **LV6 reviewer r1 (note): two pairs of tests assert the same outcome.**
  - `lv3_c1c` and `lv6_c6a` (removed text at start); `lv3_c8b` and `lv6_c3b` (55 px lock). The plan asked for both.
  - Why it matters: a future text or distance change edits two tests each.
  - Destination: fold when the LV3 test files are next touched.
- **LV6 reviewer r1 (note, plan fold): counts and the design reference are stale.**
  - The plan's header table and frontmatter title, and `master_plan_live.md` §2's LV6 row, say 13 rows. With C3(b)
    there are 14.
  - The design README (git-ignored) still says 110 px for the lock. LV-D19 (3) supersedes it (55 px).
  - Why it matters: the LV4 resume and later readers count rows from these tables.
  - Destination: the orchestrator's next plan fold.
- **LV6 reviewer r1 (note): C3(a) scrolls past the strip's end in jsdom only.**
  - The fixture's content is 427 px wide on a 390 px stage. jsdom does not clamp `scrollLeft`, so the 128 px swipe is
    exact there. A real browser stops at the end of the strip (the implementer reported this).
  - Why it matters: the test shows the tap rule, not the real scroll range. The real end stop is for the hardware check.
  - Destination: LV4 (dock look and scroll re-run).
