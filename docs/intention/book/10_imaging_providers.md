# 10 — Camera and editing providers

Two of the three integration families (commerce is chapter 09). Same rule for each:
`core → service → provider interface → adapter`; adapters declare capabilities; core never names
a manufacturer or an application (H7). One adapter per family in the MVP; no plugin loader.

## Camera provider
Capabilities an adapter may declare: `camera.discover`, `camera.connect`, `camera.status`,
`camera.capture`, `camera.transfer`, `camera.settings.read`, `camera.settings.write`. The UI
hides what is absent: the header dot needs `camera.status`, the capture button `camera.capture`.

**MVP adapter (card A2): camera through Capture One.** Capture One holds the USB tether to the
Sony ZV-E10, and one host owns that connection. The adapter reports status from the document's
`camera` and `available camera identifiers`, fires the shutter with the `capture` command, and
"transfer" is Capture One writing the RAW into the capture folder. It declares `camera.status`,
`camera.capture`, `camera.transfer`. A direct Sony adapter (Camera Remote SDK or PTP) is a later
provider that would take over the tether; its feasibility is a research item, never assumed
(owner memo, 2026-10-24).

## Editing provider
Capabilities: `document.inspect` (open document, kind, folders), `capture_folder.set`,
`events.capture_done`, `events.processing_done`, `events.variant_adjusted`,
`render.preview(recipe)`, `render.export(recipe)`, `review.mirror` (color-tag write; read-back
optional), `import`, `style.apply`. Capture One is the only editing adapter in the MVP (C5).

### Capture One adapter
- Transport: `osascript -l JavaScript` spawned from the Rust process, script files in the repo,
  one serial queue, timeout and duration logged per call. Dictionary facts and VERIFY items in
  `evidence/capture_one_16.8.6.md`.
- Startup: find the open document; require kind `session`; set `capture done script`,
  `processing done script` and `primary variant adjusted script` to our scripts; read the two
  recipes and the style by name; report what is missing on the health screen.
- Per session: set the document's capture folder to `Capture/<SKU>/` (D1, resolved yes).
- Per capture: receive the hook (VERIFY-8), copy the embedded JPEG out of the RAW (byte copy,
  H3), request `process` with the preview recipe, receive processing-done, record the adjusted
  path.
- **Live adjusted preview.** When `primary variant adjusted script` fires for a photo we know,
  or as fallback its settings sidecar changes under the watcher (VERIFY-9), debounce 1 s and
  re-run the preview render; the UI swaps the file with the usual crossfade. Capture One's own
  render cache is never read: its format is undocumented, and reproducing its rendering would be
  pixel processing (H3).
- Per publish/export: `process` each main and condition photo with the web recipe into the
  export folder under our filename, wait for processing-done, verify the file exists (timing: D4).
- Tags: write `color tag` on classification; batched read only if read-back is enabled (D2).
- Basic layer: the style, masks and crop on arrival are Capture One's Next Capture Adjustments,
  configured in S1 and named in settings. Scripted autocrop is CH-only.

## Not scriptable (manual in Capture One)
Brush strokes, healing, AI Eraser. The app never needs to know a retouch happened beyond the
adjusted hook: the preview refreshes, and the export at publish time carries the latest edits.
