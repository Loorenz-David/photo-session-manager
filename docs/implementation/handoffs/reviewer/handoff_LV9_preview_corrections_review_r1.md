---
plan: LV9_preview_corrections
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-27
actor: opus
---
# Review — LV9_preview_corrections, round 1

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. Five notes, written to the backlog.

## Verified correct
- **Gate.** The tracker read `IMPLEMENTED`. Checkpoints `75c1b755` (U1), `59cefe03` (U2), `7ad59d7f` (U3), `7b9d7df1` (U4)
  and the close commit `1e4a1fbf` are all in `git log` on `feature/live-preview`.
- **Perimeter.** `git diff --stat 75c1b755^ 7b9d7df1` lists exactly the handoff's 32 code, test and guide paths:
  `phone_view.rs`, `current_shot.rs`, `lv9_phone_incoming.rs`, suite `main.rs`; the phone's `api.ts`, `arrange.ts`,
  `state.ts`, `PhoneApp.tsx`, `phone.css`, `Dock.tsx`, `PhotoImage.tsx`, `Preview.tsx`, `PhotoDrawer.tsx` (new),
  `MoveDrawer.tsx` (removed); 15 test files including `helpers.ts`; guide 01, 02, 04. `1e4a1fbf` changes only the plan
  file. Nothing extra and nothing missing.
- **Checks (plan's Review mode, run once each, in the foreground, on code tree `7b9d7df1`).**
  `cargo test -p psm-testkit --test suite lv9_phone_incoming::`: 8 passed. `npx vitest run src/phone`: 18 files, 217
  passed. `npm run typecheck`: exit 0. `npm run build`: exit 0 (chunk-size warning only). The implementer's
  full-suite stamp stands (`cargo test --workspace` 2782/0, clippy, fmt, sidecar stub, `npm test` 1172/0); nothing
  contradicts it.
- **Review-mode reads.**
  - `incoming` comes from CS4a's set, computed in core (`shot_plan`'s `held` and new `held_free`;
    `hold_times` for an untemplated session). The route is unchanged (H8).
  - Waiting tiles have no home in `Dock.liftAt`, so they are never a target, never count in an insert index and never
    lift. A tap on one selects nothing. `arrangementKey` reads only photo ids.
  - Every tile showing a photo is keyed `p<photo_id>`, in one flat keyed list.
  - No preview URL carries the revision, and neither does `PhotoImage`'s key.
  - Drops and swipes share one `Chain` (`ask` / `answered` / `refusedAhead`) with PH9.5a's four ends:
    - (a) a read numbered after the last answer;
    - (b) a refusal, then `refused`;
    - (c) each request's own 8 s bound, stopped by its answer;
    - (d) the follow-up read's bound.

    Each new change is applied to `shownNow`. The next photo is `nextPhoto` of what the strips show.
  - The lift home is drawn only while `drag.home` is set. Only a shot tile sets it, at lift. It has no `data-dock-k`.
  - The Preview has no header, resting pill, hint or button block. Its root `aria-label` is the kicker. The ⋯ and ×
    bar is there.
  - `PhotoDrawer` closes on ×, on its backdrop and with its Preview. The photo and the strip are disabled while the
    drawer is open.
  - The Move drawer and everything only it used are gone.
- **Criterion rows (44).** For each row I read the production code that makes it true and then its test. Every test
  observes the row's outcome at the row's boundary and would fail on a wrong outcome:
  - U1: real ledger, filing, arrival, the 301 s window, the delete, and the paired HTTP route.
  - C7: `<img>` element identity plus `src`.
  - C8(a) and C9: exact `stripFrames` lists.
  - Everywhere: exact POST lists, vibrations, toasts and `aria-hidden`.
  - C9(b) answers at 7.9 s and holds the read past 8 s, so LV8's never-stopped timer would have flashed back.
  - C9(c) taps 205 where the pending arrangement put it.
  - C8(b) as written closes the Preview, because 207 is the last photo. The added test from 203 covers the Preview's
    strip.
  - C5(c) holds on PH9.3's span, as the implementer disclosed. The row's outcome is still observed (note 2).
- **Changed and removed earlier tests.** They are the plan's list plus four disclosed deviations. None weakens its row:
  - `lv7_c2a`'s filter now skips only `animation: none` and still catches any moving rule.
  - `lv6_c5b` went with the Move drawer.
  - The four `kicker` helpers now read the `aria-label`.
  - The `lv3_main` fixture gained a field.
- **Assumptions 1–8.** Read against 10J LV-D21, PH6b, PH9.5a and PH10. None contradicts the intention. Each is in the
  narrative's "Choices made for you".
- **H1–H8.**
  - H8: the rule is in `crates/core` and the route stays thin.
  - H7: no platform concept in core.
  - H2: Deny still hides and never deletes.
  - No external write (H1), no pixel work (H3), no UI automation (H5).
  - No secrets, and no live service in the tests.

## Findings
None blocking (B1–B4).

## Notes written to backlog
Written to `docs/implementation/backlog_live/LV9_preview_corrections.md`, the plan's Review mode location (see note 5):
1. A pending chain freezes the strips at the Mac's view of the moment, so a waiting tile or arriving photo shows only
   when the chain ends (up to about 8 s).
2. No test shows that a waiting template shot is not a swap target (assumption 3, the homes filter).
3. Next still offers a shot whose photo is on its way. The Mac refuses it, as before.
4. Every placeholder's grey is one step darker (`--ph-chip-border`), not only the cue's peak (assumption 2). The owner
   should judge it.
5. Process: the review prompt template names `backlog.md`, while live-track plans name `backlog_live/`.

## Write perimeter of this session
- `docs/implementation/plans/LV9_preview_corrections.md`: the Tracker line, and the Review log entry appended.
- `docs/implementation/backlog_live/LV9_preview_corrections.md` (new).
- `docs/implementation/narratives/LV9_preview_corrections.md` (new).
- This handoff.

No production code or test touched. No probes. Nothing committed (the runner commits).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
