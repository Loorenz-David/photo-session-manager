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
> **VERIFY-1 result (2026-10-25, PARTIAL).** The dictionary's full `processing done script`
> description (truncated above) reads: "passed the identifier string of the finished batch job in
> argv 1, the original RAW file path in argv 2, and a list of the output file paths in argv 3".
> Observed with a compiled AppleScript `on run argv` recorder, 13 calls: argv = `[text job id,
> text RAW path, list of text output paths]`. `batch done script`: argv = one text output path
> per call. `primary variant adjusted script`: **argv is empty** (count 0), on every call.
> `capture done script`: **UNPROBED**: no capture happened (VERIFY-12). Run later, with Capture
> One frontmost on a session and the camera able to release:
> `osacompile -o /tmp/psm_probe/cds.scpt -e 'on run argv' -e 'do shell script "echo " & quoted form of ((count of argv) as text & " " & (argv as text)) & " >> /tmp/psm_probe/hooks.log"' -e 'end run'`
> then `osascript -l JavaScript -e 'Application("Capture One").captureDoneScript = "/tmp/psm_probe/cds.scpt"'`,
> `osascript -l JavaScript -e 'Application("Capture One").capture()'`, read `/tmp/psm_probe/hooks.log`,
> and clear with `… .captureDoneScript = ""`. A JXA hook (`osacompile -l JavaScript`) was
> installed for the capture probe too and is equally unprobed; the contracts use AppleScript hooks.

**VERIFY-2:** whether the scripts fire when set at runtime via AppleScript, or only from the
Scripts menu/preferences.

> **VERIFY-2 result (2026-10-25, PASS for processing, batch and adjusted hooks).** Setting the
> application properties at runtime from `osascript -l JavaScript` to a POSIX path (text) of a
> compiled `.scpt` makes them fire at once; no restart or menu action. Read-back returns a file
> reference through `/private` (`/tmp/x.scpt` → `Path("/private/tmp/x.scpt")`). Setting `""`
> clears a property (reads back `null`). **Hooks run synchronously on Capture One's main thread:**
> with a processing hook that slept 6 s, an unrelated `name of current document` query issued
> 1.5 s after processing took 5.1 s instead of ~0.11 s. The properties are application-wide
> (not per document).

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

> **VERIFY-3 result (2026-10-25, PASS: we must create it).** Setting `captures` (as `Path` or as
> text) to a missing folder raises "Cannot set because there is no file at this location" and
> creates nothing. **After the failed set, `captures` reads back as `<working directory of the
> calling osascript>/Capture`** (seen as `…/docs/reference/Capture` and `/private/tmp/Capture`
> depending on the caller's cwd), a folder that does not exist. With the folder created first,
> the set succeeds as `Path` or text and reads back exactly; `save` before the set took 7–25 ms.
> The original folder was restored and verified. The document's `capture` favorite collection
> (`collections.byId("capture")`) keeps the root `…/Capture` as its `file` regardless of the
> current `captures` value. Contract: 10A MC-18.

**Document identity (VERIFY-7 support).** Document `id` = the document folder path
(`/Users/davidloorenz/Pictures/session test 1`); `path` returns its parent folder. Variant `id`
is a small per-document integer (`"1"`), not a UUID; the `.cos` sidecar's UUIDs differ from it.
The document's direct `images`/`variants` elements are "for the current collection"
(dictionary). `images whose path = <RAW>` found the image in 20 ms; application
`primary variant` works independent of collection.

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

> **VERIFY-4 result (2026-10-25, PARTIAL).** `color tag` reads and writes as an integer
> (0 → 4 → 0 round trip); after `save` the sidecar holds `<E K="Color_Tag_Index" V="4" />`.
> **Writing a tag on Capture One's primary variant fires `primary variant adjusted script`.**
> The integer → colour mapping (expected 0 none, 1 red, 4 green, 5 blue) is **not** visually
> confirmed: this session had no screen access. Owner check, 10 seconds: in Capture One select a
> photo and run `osascript -l JavaScript -e 'var d=Application("Capture One").currentDocument(); d.variants[0].colorTag=4'`;
> the tag must show green; repeat with 5 (blue) and 1 (red), then 0.
>
> **Confirmed by the owner 2026-10-25:** on `session test 11952` the orchestrator wrote 4, 5, 1 five
> seconds apart; the owner saw green, blue, red ("correct"). Restored to 0, read back 0.

**VERIFY-5:** does `process` with a RAW path and recipe name honour the recipe's own output folder
(`root folder type` / `root folder location`) and `output name format`? We need the output path
of the processed file: either from `processing done script`'s argument, or by computing it from the
recipe settings, or from `output event` history.

> **VERIFY-5 result (2026-10-25, PASS).** A temporary recipe ("PSM probe preview", deleted after)
> was created with `make`. `process` accepts a RAW path (text) or a variant and returns the batch
> job id (UUID text) in 93–112 ms; a **disabled** recipe processes normally. Output goes to the
> recipe's root: **`root folder type` cannot be set by script**: the write raises nothing and
> is ignored (dictionary: no `rw` access); `root folder location` then stays the Output folder.
> `output name format`, `output sub folder` (created automatically), `existing files`,
> `scaling method` (`Long_Edge`), `scaling unit`, `primary scaling value`, `JPEG quality` are
> settable. The processing-done hook's argv 3 gives the output path **only under `existing files
> = overwrite`**: with `add suffix`, two runs wrote `psmprobe_… 1.jpg` and `… 2.jpg` while the
> hook reported `psmprobe_….jpg` both times. The variant's `output events` record the true
> paths (overwrites collapse into one event). Shipped recipes default to `add suffix`.
> Contract: 10A MC-16.
**VERIFY-6:** cost of reading `color tag` for N variants in one osascript call (we need a batched
read every 3 s).

> **VERIFY-6 result (2026-10-25, PARTIAL).** N = 1 (the only image): `variants.colorTag()` plus
> `variants.id()` batched took 13 ms inside the script; a whole `osascript` call costs
> 110–130 ms wall-clock. N ≥ 50 is **UNPROBED** (needs a document with many images): run
> `osascript -l JavaScript -e 'var t=Date.now(); var v=Application("Capture One").currentDocument().variants; var a=v.colorTag(); [a.length, Date.now()-t]'`.
> Read-back is optional and off by default (chapter 06), so this does not gate the MVP.

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

> **VERIFY-8 result (2026-10-25, PASS for processing events; capture latency UNPROBED).**
> Because hooks block Capture One (VERIFY-2), delivery must not wait on the app. Measured:
> (a) `do shell script "curl -m 2 …"` to a loopback listener: hook start → request received
> 30–35 ms; (b) **spool file + backgrounded curl nudge**: hook writes JSON with temp + rename
> into a spool folder and starts `curl … &`; processing command → spool file 0.56–0.67 s, nudge
> received +17 ms after the file, and Capture One answered a query during the hook in 110–130 ms
> (no blocking). (c) App not running: `curl` to a closed port fails at once (non-zero exit,
> caught in the hook's `try`); the spool file persists for the next start. Deep link
> (`sessionmanager://`) not probed (no app registered yet); not needed. Capture → hook latency
> is UNPROBED (VERIFY-12). Contract: 10A MC-15.
>
> **VERIFY-9 result (2026-10-25, PARTIAL).** Each scripted adjustment (`adjustments.exposure`
> set 8 times) fired `primary variant adjusted script` once per change, 130–165 ms later, with no
> arguments. A colour-tag write fires it too. Inside the hook, `tell application "Capture One"
> to get primary variant` (id and file) works and takes 30–55 ms. The settings sidecar is
> `<capture folder>/CaptureOne/Settings1680/<RAW name>.cos` (**inside** the capture folder, not
> beside it); its mtime does **not** change on an adjustment, only at Capture One's autosave,
> 9–15 s later (autosaves are at least ~15 s apart, per its log). **UNPROBED:** the firing rate
> while dragging a slider in the UI (needs a person; UI automation is excluded, H5). Owner
> check: install a recorder as in VERIFY-1 on `primaryVariantAdjustedScript`, drag the Exposure
> slider for 3 s, count lines. Exposure was restored to 0. Contract: 10A MC-17.
>
> **VERIFY-10 result (2026-10-25, PASS).** Recipe JPEG, long edge 1600 px, quality 85, one ARW
> (24.6 MB, ZV-E10), Capture One idle: `process` returns in ~0.1 s; processing-done fires
> 0.52–0.72 s after the command (10 runs; output 1600 × 1067, ~220 KB).

**VERIFY-11 (added 2026-10-25):** when Capture One browses a folder other than the one holding a
RAW (for example `Capture/<SKU-A>/` while the RAW is in `Capture/<SKU-B>/`), does
`images whose path = <RAW>` on the document still find it, and does the folder collection whose
`file` is the RAW's folder? UNPROBED: needs captures in two subfolders. Contract fallback: 10A
MC-19.

**VERIFY-12 (added 2026-10-25):** scripted capture. UNPROBED, blocked. The camera was listed
(`available camera identifiers`: one ZV-E10) and Capture One logged
`shutterReleaseForCamera … ID 401` for each of 9 scripted `capture` calls (background,
after `select camera`, and with Capture One brought to the front with `activate`), but no image
was written. The owner's own session at 12:33 shows six releases before the one frame that was
written. Most likely the camera refuses to release without focus (AF with focus priority, lens
cap, or nothing focusable). To run: set the camera to MF or release priority, then repeat
VERIFY-1 and time `capture` → RAW mtime → hook.

**VERIFY-12 resolved (orchestrator, 2026-10-25 13:23): PASS with autofocus first.** Camera in its
owner settings (AF, focus priority). A bare `capture` wrote nothing (1 more attempt, same as
above). Setting `camera.autofocusing = true` on the current document's camera, waiting 2 s
(`focusIndicator` read back `"focused"`), then `capture` wrote a RAW within 8 s, twice in a row
(`session test 11954.ARW`, `session test 11955.ARW`, ~24.6 MB each, in the capture folder root).
Recipe for scripted captures (sessions, probes, and the camera adapter's trigger):
`var c=Application("Capture One").currentDocument().camera(); c.autofocusing=true; delay(2);`
then require `c.focusIndicator()=="focused"` and call `Application("Capture One").capture()`.
Capture One did not need to be brought to the front. VERIFY-1 timing (capture → RAW → hook) is
still to be measured by the first phase that installs the hook.

**VERIFY-1 result (orchestrator, 2026-10-25 15:30, PASS).** With the P00 app running and
`captureDoneScript` set to its compiled `capture_done.scpt`, an autofocus + `capture()` produced
one spool event whose `argv` is exactly `["/Users/davidloorenz/Pictures/session test 1/Capture/session test 11957.ARW"]`
(the full POSIX path of the new RAW, nothing else), received 1.74 s after the capture call.
**Clearing a hook:** assign `""` (JXA or AppleScript); `null` / `missing value` raise -1700.
Read-back after clearing is `null` / `missing value`. (Details: `packaging_spike.md`, Live checks.)

## Other runtime observations (2026-10-25)
- Capture One writes a capture via a hidden `Capture/.capture` location
  (`writeImage … toURL: …/Capture/.capture` in its log) before it appears in the capture
  folder; the watcher must ignore dot paths (05A MC-2).
- RAW embedded previews (ZV-E10): IFD0 JPEG 1088 × 1080 (180 KB), IFD1 thumbnail 160 × 120;
  the IFD0 preview is framed tighter than Capture One's render. Orientation tag present (1).
  Contract: 10A MC-20.
- EXIF capture time in the sidecar (`Exp_Date`) is 7230 s ahead of the RAW's mtime: the camera
  clock is local wall time with no zone. Never use it as an instant (05A MC-4).
- 2026-10-25 is the CET/CEST change day; Capture One reported dates as GMT+0100.
- Probe perimeter: everything ran from `/tmp/psm_probe*`. Files created in the Capture One
  document: `Output/psmprobe_session test 11952.jpg`, `… 1.jpg`, `… 2.jpg`,
  `Output/psm_probe_sub/…`, their `CaptureOne/Settings1680/*.cos`, and the empty folder
  `Capture/PSMPROBE-0001/`. Restored: hooks (null), capture folder, colour tag 0, exposure 0,
  probe recipe deleted. The RAW's SHA-256 was identical before and after
  (`fe193d00…6041a9`).

## Transport
`osascript -l JavaScript` spawned from the Rust process with a script file and arguments; one
serial queue; timeout per call.
Capture One must be running with the session frontmost or referenced by name; **VERIFY-7:**
addressing a session document by path when more than one document is open.

> **VERIFY-7 result (2026-10-25, PARTIAL).** With one document open, `documents.byId("<document
> folder path>")` and `documents.byName("<name>.cosessiondb")` both address it, and all render
> and capture-folder probes ran through `byId` with Capture One in the background. Two open
> documents not probed (only one existed). `capture` acts on the foreground document only
> (dictionary). Contract: 10A MC-19.

## Read-only smoke for P04 (orchestrator, 2026-10-25 ~16:45; camera NOT tethered)
JXA against the open `session test 1` document:
- `Application("Capture One").running()` → `true` (checking it does not launch the app).
- `currentDocument().kind()` → `"session"`; `id()` → `"/Users/davidloorenz/Pictures/session test 1"`;
  `name()` → `"session test 1.cosessiondb"`; `captures()` (String) → `".../session test 1/Capture"`.
- **No camera:** `currentDocument().camera()` returns **`null`** (not an error); `availableCameraIdentifiers()` →
  `[]`. With the camera tethered (13:20 today) `camera().name()` was `"Sony ZV-E10"` and the identifier
  `"Sony__ZV-E10__00000000000000003285273206729157"`.
- Capture root: `collections.whose({id: "capture"})()[0].file()` → `".../session test 1/Capture"`.
- Recipe props read in JXA: `existingFiles()` → `"add suffix"`, `outputSubFolder()` → `""` (a built-in recipe;
  not written).
- `images.whose({path: <RAW>})()` → 1 image; its variants: ids `["1"]`, positions `[1]`.
- **Hook read-back shapes:** a hook property reads back as a JXA `Path` object (`typeof "object"`); use
  `.toString()`. Set as a **string** `"/private/tmp/…/capture_done.scpt"` → reads back `"/tmp/…"` (prefix
  stripped); set as `Path("/private/tmp/…")` → reads back `"/private/tmp/…"`. Compare by `canon` (MC-1), never
  by string. Set to a **non-existent** file → reads back a different path,
  `/var/folders/…/T/TempCOScript_<UUID>.scpt` (Capture One substitutes a temp copy), so install must run
  after the scripts are compiled and a read-back mismatch is the signal. Cleared with `""` → reads `null`.

**VERIFY-11 result (orchestrator, 2026-10-26 08:16, camera tethered).** Frames 12199, 12200 into
`Capture/PSM_PROBE_A/` and 12201, 12202 into `Capture/PSM_PROBE_B/` (capture folder set before each pair;
autofocus recipe). While Capture One browsed `PSM_PROBE_B` (it followed the capture folder):
- `doc.images.whose({path: <RAW in B>})` → 1 image, variant `"132"`, 46 ms; `<RAW in A>` → **0 images**.
  `images whose path` on the document searches **only the current collection**.
- `doc.collections.whose({file: Path(<folder>)})` → **error -1725 "Illegal logical operator called"** for every
  folder tried. The adapter's `find_image_in_folder.js` uses exactly this, so the MC-19 fallback always errors.
- Working form: iterate `doc.collections()` (top level lists every folder favourite, nested ones flattened too; 84 ms
  for 11 collections) and compare `c.file().toString()` with the folder; then `c.images.whose({path: RAW})` → 1 image,
  variant `"130"`, colour tag 0, 33 ms. `currentCollection` is unchanged. `collections.whose({name})` and
  `collections.byId(17)` also work. Folder collections exist for folders that were set as capture folder.
- Fix routed to P16 task 0 (row C6(a)). Frames and the two scratch folders stay in the test session.

**VERIFY-8 capture latency (2026-10-26, 4 frames).** Temporary compiled hook writing a timestamp + argv;
`captureDoneScript` cleared with `""` afterwards. `capture` call (incl. 2 s autofocus wait) returns in 2.21–2.34 s;
the RAW is visible in the capture folder 1.37–1.48 s after the call returns; the capture-done hook fires 1.71–2.54 s
after the call returns (0.3–1.1 s after the file is visible), argv = the full RAW path (VERIFY-1 confirmed again).

