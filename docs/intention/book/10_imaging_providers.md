# 10 — Imaging providers

## Shape
`core → imaging provider interface → Capture One provider`. The interface has two halves the brief
already named: the **Capture Adapter** (events in: capture done, processing done; commands:
set capture folder, select, capture, import) and the **Edit Engine** (`apply(photo, command)`:
`apply_style`, `set_tag`, `process(recipe)`), both logged to `edits` with before-values for undo
and later learning. Capture One is the only provider in the MVP (decision C5).

## Capture One provider
- Transport: `osascript -l JavaScript` with script files in the repo, one serial queue, timeout
  and duration logged per call. Dictionary facts and VERIFY items in
  `evidence/capture_one_16.8.6.md`.
- Startup: find the open Capture One document; require kind `session`; set `capture done script`
  and `processing done script` to our scripts; read the two recipes and the style by name and
  report missing ones on the health screen.
- Per session: set the document's capture folder (decision D1).
- Per capture: receive the hook, copy the embedded JPEG preview out of the RAW (byte copy, H3),
  request `process` with the preview recipe, receive processing-done, record the adjusted path.
- Per publish/export: `process` each main and condition photo with the web recipe into the
  export folder with our filename, wait for processing-done, verify the file exists.
- Tags: set and batch-read `color tag` per variant (chapter 06).
- Camera: `document.camera` and the available-camera list feed the header dot; "retry" calls
  `select camera`.
- Basic layer: the style, masks and crop applied on arrival are Capture One's Next Capture
  Adjustments, configured by the photographer in S1 and named in settings. Scripted autocrop is
  not available in Pro; AI Crop stays a Next Capture setting.

## Not scriptable (manual in Capture One)
Brush strokes, healing, AI Eraser. The app never needs to know a retouch happened: export at
publish time always carries the latest adjustments (decision D4).
