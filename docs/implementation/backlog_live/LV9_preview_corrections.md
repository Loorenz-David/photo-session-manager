# Backlog — LV9_preview_corrections (reviewer notes; notes never block, flow.md §5)

- **LV9 reviewer r1 (note, UI): a pending chain hides what the Mac reports meanwhile, including the new loading cue.**
  While a drop or swipe is unanswered, both strips show the pending arrangement (`PhoneApp.tsx` `arranged` =
  `pending`), which was computed from the Mac's view at the moment of the gesture. A waiting tile, a shot turning
  `incoming`, or a photo arriving during that time is not drawn until the chain ends (≤ 8 s plus a read). Why it
  matters: sorting while shooting (LV-D21 (1) "from the shutter") can show the cue up to a few seconds late. Fix
  idea for a later phase: re-apply the outstanding changes on top of each new Mac view instead of freezing the old
  one.
- **LV9 reviewer r1 (note, test): no test shows a waiting template shot is not a swap target** (assumption 3, the
  homes filter in `Dock.liftAt`). LV9·C5(c) passes on PH9.3's span alone, as the implementer disclosed. Why it
  matters: if the filter is lost, dropping a photo on a shot whose own photo is on its way would ask the Mac for a
  swap the owner did not intend (the Mac would then place both). Not a B2 because the plan's row is observed as
  written.
- **LV9 reviewer r1 (note, UI): Next still treats a shot whose photo is on its way as empty** (`state.ts::nextStep`;
  implementer's note). Choosing it is refused by the Mac (`shot_filled`) as before LV9. Why it matters: a toast after
  a fast Capture-then-Next; the phone now knows `incoming` and could skip such a shot.
- **LV9 reviewer r1 (note, look): every photo placeholder is now one step darker** (`--ph-chip-border` instead of
  `--ph-fill`; assumption 2), so the light band can peak about 10 % above it. The plan's delegated choice named the
  peak, not the fill. Why it matters: it changes the look of every waiting picture, including the Preview's
  "Preparing preview…"; only the owner's eye on a phone can confirm it (narrative "Choices made for you").
- **LV9 reviewer r1 (note, process): backlog location.** The review prompt template says
  `docs/implementation/backlog.md`; the live-track plans (LV8, LV9 Review mode) say `backlog_live/<phase>.md`. These
  notes follow the plan. The template for live-track reviews could name `backlog_live/` so the two agree.
