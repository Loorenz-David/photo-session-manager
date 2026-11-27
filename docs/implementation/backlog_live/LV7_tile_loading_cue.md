# Backlog: LV7_tile_loading_cue (reviewer notes; none of them blocks)

- **LV7 reviewer r1 (note): a picture that gave up loading shimmers forever, as if still arriving.**
  - Assumption 4: the sweep rides on `.ph-photo-wait`, which `PhotoImage` also shows after the third failed retry
    (LV6 backlog note: it then waits for a new `preview` or `revision`).
  - Why it matters: before LV7 a stuck tile was plain grey; now it promises "on its way" for the rest of the session.
    The owner may read it as the Mac being slow when the picture will not come by itself.
  - Destination: if the hardware check shows stuck tiles, `PhotoImage` adds a class when retries are used up and the
    sweep is off for it (one row), together with the LV6 retry-on-reconnect note.
- **LV7 reviewer r1 (note): `lv7_loading_cue.test.tsx` reads `src/phone/phone.css` relative to the working directory.**
  - It passes from `apps/desktop` (how `npm test` and the plan's commands run) and fails to find the file from the repo
    root. `lv6_dock.test.tsx` does the same, so it is the existing convention.
  - Why it matters: only if someone runs Vitest from another directory. Optional: resolve from `import.meta.url`.
