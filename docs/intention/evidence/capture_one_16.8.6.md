# Source evidence — Capture One Pro 16.8.6.25 AppleScript dictionary

Source: `/Applications/Capture One.app/Contents/Resources/CaptureOne.sdef`, copied verbatim to
`docs/reference/capture_one_16.8.6.sdef` on 2026-10-23 (dev Mac, macOS 26.5). Values below are
read from that file. Items marked **VERIFY** are not answered by the dictionary and are owed to
mechanism-inventory as runtime probes against the real application.

## Event hooks (application properties, PRO only)

| Property | Dictionary description | Our use |
|---|---|---|
| `capture done script` | "The AppleScript that gets called when an image has completed being captured." | primary capture event |
| `processing done script` | "…called when a file is finished processing. The script is passed the ident…" (truncated in sdef) | preview/export file ready |
| `batch done script` | "…called when the batch process queue completes." | export batch complete |
| `primary variant adjusted script` | exists (untyped) | live adjusted-preview refresh (chapter 10) |
| `selection changed script` | exists | not used in v1 |
| `barcode scanned script` | Enterprise only | not available |

**VERIFY-1:** exact handler signature and arguments each script receives (RAW path? variant id?
output path?). The brief reports `CO_CaptureDone(rawFilePath)` works. The property type is
untyped in the sdef (`None`), so test whether it takes a path or a script object.
**VERIFY-2:** whether the scripts fire when set at runtime via AppleScript, or only from the
Scripts menu/preferences.

## Document (session) properties

| Property | Access | Note |
|---|---|---|
| `kind` | r/o | enum `session` / `catalog` |
| `captures` | r/w | the Capture folder — basis for per-product folders (D1) |
| `output` | r/w | Output folder |
| `selects`, `trash` | r/w | session only |
| `last captured file` | r/o | reconciliation aid |
| `camera`, `available camera identifiers` | r/o | health screen |
| `next capture settings` | r/w | `apply styles`, `capture profile`, orientation — S1 could be scripted, kept manual |
| `processing queue enabled` | r/w | check before export |
| `capture name format`, `capture counter` | r/w | not used; naming is ours at export |

**VERIFY-3:** setting `captures` to a non-existent folder — does Capture One create it, or must we
create it first?

## Variant properties and commands

| Item | Type | Note |
|---|---|---|
| `color tag` | integer r/w | **VERIFY-4:** integer → color mapping (expected 0 none, 1 red, 2 orange, 3 yellow, 4 green, 5 blue, 6 pink, 7 purple) |
| `rating` | integer r/w | 0–5 |
| `file` | file r/o | RAW path — the join key to `images.raw_path` |
| `id` | text r/o | stored as `co_variant_id` |
| `styles` | list of text r/w | applied styles |
| `queued` | boolean r/o | in process queue |
| `process` command | direct: variant or RAW path; param `recipe` (text) | preview and web export |
| `apply style` command | direct: layer; param `named` | Edit Engine `apply_style` |
| `select` command | document; param `variant` / `variants` | focus the viewer on a product's image |
| `apply adjustments`, `reset adjustments`, `copy adjustments` | exist | future |

**VERIFY-5:** does `process` with a RAW path and recipe name honour the recipe's own output folder
(`root folder type` / `root folder location`) and `output name format`? We need the output path
of the processed file: either from `processing done script`'s argument, or by computing it from the
recipe settings, or from `output event` history.
**VERIFY-6:** cost of reading `color tag` for N variants in one osascript call (we need a batched
read every 3 s).

## Recipe properties
`name`, `enabled`, `output format`, `JPEG quality`, `color profile`, `scaling method`,
`primary scaling value`, `root folder type` (`output location` / `image folder` / `custom location`),
`root folder location`, `output name format`, `output sub folder`, `existing files`
(`add suffix` / `overwrite` / `skip`), `thumbnails`, `include ratings`.
We read these to compute expected output paths and to validate the two recipes at startup.

## Not available
- `autocrop` — CH only. AI Crop stays a Next Capture Adjustments UI setting (S1).
- Brush strokes, healing, AI Eraser — no commands exist (brief confirmed).
- Barcode — Enterprise only. Scanning goes into our UI.

**VERIFY-8:** how a hook script reaches the app. Candidates: `do shell script "curl …"` to a
loopback port the Rust process opens; writing a small file into a spool folder the app watches;
`open "sessionmanager://…"` (Tauri deep link). Measure latency and behaviour when the app is not
running.
**VERIFY-9:** does `primary variant adjusted script` fire per slider change, per mouse-up, or per
variant switch, and at what rate while dragging? Fallback signal: the per-image settings sidecar
(`CaptureOne/Settings*/<name>.cos` beside the capture folder) — confirm the path and that its
mtime changes on adjustment.
**VERIFY-10:** latency of `process` with the 1600 px preview recipe, command to processing-done,
one image, Capture One idle.

## Transport
`osascript -l JavaScript` spawned from the Rust process with a script file and arguments; one
serial queue; timeout per call.
Capture One must be running with the session frontmost or referenced by name; **VERIFY-7:**
addressing a session document by path when more than one document is open.
