# Backlog: LV8_phone_sort_and_arrange (reviewer notes; none of them blocks)

- **LV8 reviewer r1 (note): the 8 s wait is not stopped when the Mac answers a drop.**
  - `PhoneApp.drop` starts the `DRAG.pendingMs` timer at the drop and leaves it running after a successful answer
    (assumption 14). If the Mac answers at, say, 7.9 s and the read that follows is still out at 8 s, the strip shows
    the last view from the Mac (the photo back in its old place) until that read returns with the drop.
  - Why it matters: a one-read flash back on a slow Mac. PH9.5 (c) reads "8 s without an answer". The Mac's view
    ends up correct either way.
  - Destination: if the owner sees a drop jump back and forth on a slow network, clear the timer when the answer
    arrives and let (a) end the wait, with a separate bound for a read that never returns (one row).
- **LV8 reviewer r1 (note): a photo with no preview yet does not follow a swipe.**
  - Assumption 6: the swipe moves the picture element, and `PhotoImage` does not pass its style to the grey
    "Preparing preview…" placeholder. The hint shows and the swipe still counts. Only the grey box stays still.
  - Why it matters: the swipe looks dead on a just-taken photo, although it works. Fix: `PhotoImage` passes `style` to
    its placeholder (outside LV8's files).
- **LV8 reviewer r1 (note): nothing of LV8 has been seen on a real phone.**
  - Every row is proven in jsdom with mouse events of the pointer types and stubbed rectangles. These are untested:
    the lifted look and the scroller's 48 px reach, the hint over a bright photo, the sheet's motion, the iOS long-press
    callout and selection on the strip, and a real second finger. jsdom has no `isPrimary`, so assumption 15's
    "a new first finger forgets a lost one" path never runs in the suite.
  - Why it matters: the owner's first requirement is reliability ("buggy drag and drop behaviours"). The manual
    try-out is the first real check. Record what it shows in STATUS_live.
- **LV8 reviewer r1 (note): `p33_split_grid` C4(d) flaked again in the implementer's `npm test`** (1049 ms against
  its 1 s `waitFor`; green alone). This is a further recurrence of the `backlog.md` item (P35 → P33). LV8 changed no
  `src/features` file.
- **LV8 reviewer r1 (note, process): the implementer ran `cargo test --workspace` detached** (deviation 2) against
  flow §4A's "never in the background". No other cargo command overlapped and it took 5 min 4 s. If foreground
  10-minute limits keep pushing sessions to do this, the runner's wrapper should state how to run the full suite.
