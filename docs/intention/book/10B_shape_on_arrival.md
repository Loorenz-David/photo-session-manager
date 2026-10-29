# 10B — Shape on arrival (owner decision 2026-10-29)

Owner decision 2026-10-29 (chapter 15 R35, chapter 16 entry 22), after the second real run: **every
new photo gets the configured crop shape in Capture One when it arrives, Square by default, so the
product photos end up square without a manual select-all → Square step.** Capture One stays the only
editor: the app only asks Capture One to set its own crop ratio, and never touches a crop the
photographer has set. This chapter is the authority on that rule; chapter 10 ("Basic layer") and
chapter 12 (settings, health) point here.

Owner's words: "on the camera I'm choosing cropping 1:1, the imported images because they are raw
are brought without that 1:1 cropping, Capture One reads them like that also, but the preview on the
session manager is shown as 1:1. My end intention is 1:1; before, in Capture One I used to select
all imported images then select square on shape." Choice: "App sets Square".

Why the app does it: the camera's aspect setting reaches only the JPEG embedded in the RAW; Capture
One renders and exports the full 3:2 frame (evidence, "Crop by script", 2026-10-29: published files
were 2048×1365). Capture One's Next Capture Adjustments could copy a crop from the last capture, but
one uncropped capture would pass unnoticed; the app sees every arrival and can apply the shape once.

## MC-25 — Shape on arrival

**Setting.** `settings.json` key `crop_on_arrival`: either the name of one of Capture One's crop
aspect ratios as the editing adapter lists them (`available crop aspect ratios`, e.g. `Square`,
`4x5`, `Original`) or `off`. Missing → `Square`. The settings screen offers it as "Shape on
arrival" with the adapter's list plus "Leave as shot" (`off`). Core stores and passes the name
opaquely; only the adapter knows Capture One's names (H7).

**When it applies.** Once per RAW, when a new photo row is created for a RAW that has no earlier
photo row: capture arrival (hook, watcher or startup scan, MC-2) and import (`photos_import`).
Never for a copy or move of an existing photo, never for a RAW that has a publish link, never
again after it was attempted to completion (applied or skipped) — a relaunch does not re-apply.

**What it does.** Through the serialised editing-provider lane (the same queue as the colour-tag
mirror, MC-19), the adapter reads the variant's crop; **only if the crop is still the untouched full
frame** (Capture One's crop aspect ratio reads empty or `Unconstrained` **and** the crop size equals
the image's maximum crop), it sets the variant's crop aspect ratio to the configured name. Capture
One then centres the largest crop of that ratio in the frame (for Square on a 3:2 frame: full
height, centred — the camera's own 1:1 framing). A crop the photographer already changed is left
exactly as it is and the attempt counts as skipped.

**Consequences, no extra work.** The crop write makes Capture One fire its adjusted hook; the
adjusted preview re-renders through MC-17 and shows the new shape (evidence: 1600×1600 render
without app help). The web recipe exports the crop, so publish and export are square. The write
happens before any publish of the RAW, so it never makes a session dirty (MC-21 applies to
published photos only). The original preview (MC-20) is unchanged: it is the camera's embedded JPEG.

**Failures.** Capture One not answering or the variant not found yet: retry with the provider
back-off (MC-19 rules), then give up after 8 attempts (about three minutes; the colour-tag mirror
itself never gives up) and record a health item "Shape not
applied to n photos" until a later attempt succeeds. A configured name the adapter does not list:
no crop is attempted and health shows "Shape on arrival: '<name>' is not a Capture One crop
ratio". Neither blocks capture, review or publish.

**What the photographer sees.** Workspace tiles and the stage show each photo's whole adjusted
preview in its true proportions (fitted inside the tile, not trimmed to fill it), so the shape seen
is the shape that will be published. Before the adjusted preview exists the original preview is
shown the same way.

## Invariant
**M31** (chapter 14): after the provider queue is idle, every RAW that arrived uncropped with
`crop_on_arrival ≠ off` reads back from Capture One with that crop aspect ratio; no RAW whose crop
the photographer changed, and no published RAW, had its crop written by the app.
