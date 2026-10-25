# 10A — Mechanism contracts: Capture One adapter (hooks, renders, live preview, folders, tags)

Written by the mechanism inventory (2026-10-25, round 1) from runtime probes against Capture One
Pro 16.8.6.25 on the dev Mac. Results and measurements are in
`../evidence/capture_one_16.8.6.md` under each VERIFY item. Conventions as in
`05A_mechanism_contracts.md`. Chapter 10 is the authority for what the adapter does; this chapter
is the authority for how.

**Storage key for a RAW in app storage:** `path_key` = the first 16 lowercase hex characters of
SHA-256 over the UTF-8 bytes of `canon(raw_path)` (05A MC-1). Previews live under
`<app data>/previews/<path_key[0..2]>/<path_key>/`. Copy rows on the same RAW share them.

---

## MC-15 — Hook installation and delivery

**Hook scripts** are AppleScript (not JXA), generated at startup from templates in the repo
with the absolute spool path and the loopback port written in, compiled with `osacompile` into
`<app data>/hooks/`, and installed by setting the application properties
`capture done script`, `processing done script` and `primary variant adjusted script` to their
POSIX paths (text). After setting, each property is read back (it returns a file, through
`/private` where applicable) and compared by `canon`; a mismatch is a health error "hooks not
installed". Clearing is setting `""` (reads back null). The properties are **application-wide**:
events from other documents arrive too and are ignored when their RAW is not under the
configured document's capture root (MC-2).

**Hooks never wait for the app.** Capture One runs a hook synchronously on its main thread
(evidence: a 6 s hook delayed an unrelated query by 5.1 s). Each hook body, entirely inside
`try`, does only this: build one JSON object (strings escaped for `\`, `"` and control
characters), write it to `<spool>/<epoch µs>-<pid>-<hook>.json.tmp`, rename it to `.json`,
start `curl -s -m 1 http://127.0.0.1:<port>/nudge` **in the background**, and return. No
response is awaited and no error escapes.

**Payloads** (argument shapes per evidence):
| Hook | argv | JSON written |
|---|---|---|
| capture done | not yet observed (VERIFY-1, camera) | `{"hook":"capture_done","argv":[every argument as text]}` |
| processing done | `[job id, RAW path, [output paths]]` | `{"hook":"processing_done","job","raw","outputs":[…]}` |
| primary variant adjusted | none | the hook asks Capture One for `primary variant` and writes `{"hook":"variant_adjusted","variant_id","raw"}` (measured 30–55 ms in-hook) |

If the capture-done argv carries no path, the hook writes `last captured file` of the current
document instead (dictionary property).

**Consumption.** The app reads the spool on a nudge, on an FSEvents change of the spool folder,
at startup, and every 5 s; files in name order; each file becomes one `capture_events` row plus
its dispatch, and is deleted only after that transaction commits. A malformed file moves to
`<spool>/bad/`. The nudge carries no data, so the loopback listener (bound to 127.0.0.1 only)
cannot be used to inject events. Events written while the app is closed are processed at the
next start.

**Invariant (M22).** Every hook call results in exactly one effective dispatch whether the app was
running, busy or closed when it fired; no hook makes Capture One wait on the app.

---

## MC-16 — Render jobs (preview and export)

**Recipes.** Two recipe names in settings (preview, web). At startup the adapter checks both
exist, then sets on each: `existing files = overwrite` (with `add suffix`, the processing-done
hook reports the unsuffixed path while Capture One writes `name 1.jpg`, `name 2.jpg`: the app
would silently record the old file), and `output sub folder = _session_manager` (sub folders
are created by Capture One). `root folder type` cannot be set by script (writes are silently
ignored); the root stays as the user configured it, and the adapter never computes output paths.

**Issuing.** One `process` call per render, direct parameter = the variant at position 1 of the
image found by MC-19, `recipe` = the name. The call returns a batch job id (≈100 ms). The job map
stores `job id → (purpose preview|export, raw, publish item if export, generation if preview,
issued_at)`. At most one render in flight per output base name, compared case-insensitively
(two RAWs called `X.ARW` in different folders would otherwise overwrite each other's output);
a second waits.

**Completion.** The processing-done payload is matched by job id (its argv 1 equalled the
returned id in all 10 probe runs where both were recorded). The output path is `outputs[0]`, accepted only if the file
exists and its mtime is no earlier than `issued_at − 2 s`. The file is copied (temp + rename)
into app storage: preview → `previews/…/<path_key>/adjusted-<generation>.jpg`; export → the
run's staging folder under its position name (05A MC-8). The path recorded for the photo
therefore changes with every render, so the webview never shows a cached older image under an
unchanged URL. The previous generation is deleted after the new path is committed. An unknown
job id is logged and ignored. No completion within 60 s (preview) or 120 s (export): re-issue
once, then fail with the reason. Measured: command → processing-done 0.52–0.72 s for one RAW,
1600 px long edge, Capture One idle.

**Invariant (M23).** Every adjusted preview and export file the app records was produced by the
job issued for that RAW and purpose, after the request; a newer render always changes the
recorded path.

---

## MC-17 — Live adjusted preview: staleness and the render trigger

Per RAW: `adjusted_gen` (integer, starts 0), `rendered_gen`, `last_adjusted` (monotonic
instant), `render_in_flight` (job id or null).
- An adjusted event (MC-15 `variant_adjusted`, matched by `canon(raw)` to a known RAW; unknown
  RAWs are ignored) does `adjusted_gen += 1`, `last_adjusted = now`.
- **Stale** ⇔ `adjusted_gen > rendered_gen`.
- A stale RAW with no render in flight is rendered when `now − last_adjusted ≥ delay`, or at
  once when the Session Manager window becomes visible or focused (all stale RAWs, active
  session first in display order). `delay` = the setting clamped to 10–30 s; missing or not a
  number → 15 s.
- A render request records `g = adjusted_gen`; its completion sets
  `rendered_gen = max(rendered_gen, g)`. If more events arrived meanwhile, the RAW is still stale
  and is rendered again by the same rule. A completion never clears staleness it did not render.
- **Fallback (ratify R-I15).** The settings sidecar is
  `<RAW folder>/CaptureOne/Settings<engine>/<RAW file name>.cos` (glob `Settings*`, newest).
  Capture One writes it only at its autosave, 9–15 s after an edit, and also when we write a
  colour tag. So the sidecar is checked only at startup and on window focus: a RAW whose sidecar
  mtime is later than both its last render's completion and `last_own_write + 30 s` (MC-19) gets
  one adjusted event. During normal running the hook is the trigger.
- A colour-tag write by the app fires the adjusted hook when that variant is Capture One's
  primary; the resulting extra render is accepted (identical output, invisible crossfade).

An adjusted event outside the own-write window is also a retouch (07B MC-21).

**Invariant (M24).** After the last adjusted event for a RAW, the recorded preview comes from a
render requested after that event, within `delay` plus render time, or right after the window is
focused; an event arriving during a render is never lost.

---

## MC-18 — Capture-folder switch (D1)

When a session becomes active (or the last one closes, target `_Unassigned`, MC-2):
1. `save` the document (the dictionary asks for this before changing the capture folder);
2. create the target folder (`mkdir -p`); **Capture One does not create it**;
3. set the document's `captures` to the target path;
4. read `captures` back and compare by `canon` with the target;
5. on error or mismatch: health error "capture folder not set", camera status red,
   `camera.capture` disabled, and the previous folder re-applied by steps 1–4. Step 3 is never
   attempted without step 2.

Evidence for step 4: setting a missing folder fails with "Cannot set because there is no file at
this location", and afterwards `captures` reads back as `<working directory of the calling
osascript>/Capture`, a folder that does not exist. `osascript` is therefore always spawned with
the app data directory as its working directory, and step 4 must pass before the app reports
the session as ready to shoot. A frame that lands in the previous folder during a switch is
attributed by its folder (MC-2), so it joins the session that was active when it was shot.

**Invariant (M25).** After a session is activated, Capture One's capture folder reads back as
that session's folder, or the app shows the error and blocks capture; captures never silently go
to another session's folder.

---

## MC-19 — Document and variant addressing; the colour-tag mirror

- **Document:** `id` is the Capture One document folder path (evidence). The configured id is
  stored in settings, and every call addresses `documents.byId(<id>)`. `current document` is
  used only for `capture`, which works only on the foreground document. Startup requires kind
  `session`.
- **Variant ids** are small per-document integers (`"1"`), not UUIDs. `provider_ref` stores
  `<document id>#<variant id>` as an advisory cache. The join to our photos is the RAW path.
- **Resolving a RAW:** `images whose path = <raw>` in the configured document (20 ms). The
  document's direct image elements are "the current collection" per the dictionary, so if none
  is found, search the folder collection whose `file` is the RAW's folder. Still none →
  `variant_not_found` (retried with backoff; health shows it). The variant used is position 1
  (the pick). Behaviour while Capture One browses another folder is VERIFY-11.
- **Mirror (M2).** Jobs are coalesced per RAW. A job writes the mapping of the row's
  **current** classification at execution time, not the classification at enqueue time:
  `main` 4 (green), `condition` 5 (blue), `deleted` 1 (red), `unreviewed` 0 (none). It then
  reads `color tag` back and requires equality. It records `last_own_write = now` for the RAW.
  Read-back of tags from Capture One stays off (chapter 06). Copy rows on one RAW share one
  variant: the variant carries the tag of the most recently changed row.
- **Capture One not running or not responding:** jobs stay queued and retry with backoff; every
  osascript call has a 30 s timeout; a timed-out write is followed by a read before any retry.

**Invariant (M26).** Once the provider queue is idle, each RAW's variant colour tag equals the
mapping of its most recently changed row's classification, however many classify actions were
queued; a write is confirmed by reading it back.

---

## MC-20 — Original preview extraction

Read the RAW read-only. Walk the TIFF structure (IFD0 chain and SubIFDs). Candidates are the JPEG
streams named by tag pairs `0x0201` (offset) / `0x0202` (length). Keep a candidate only if it
starts with `FF D8` and its last non-zero bytes are `FF D9`. Choose the largest pixel area, read
from its SOF header (header parsing only; no pixel decoding, H3). Byte-copy it to
`previews/…/<path_key>/original.jpg` (temp + rename). Store IFD0 tag `0x0112` (orientation 1–8)
on the photo. The UI applies it to the original preview, because the embedded JPEG carries no
orientation of its own; the adjusted preview is already oriented by Capture One. No candidate
→ no original preview, and the photo shows "Processing" until the adjusted one arrives.

Evidence (ZV-E10, dev Mac): IFD0 holds a 1088 × 1080 JPEG of 180 KB and IFD1 a 160 × 120
thumbnail. The embedded preview is framed tighter than the RAW render (as if the camera's aspect
setting were close to 1:1), so the Original/Adjusted toggle shows different framing with this
camera configuration.

**Invariant (M27).** The original preview is a byte-exact copy of the largest valid embedded JPEG
stream of the RAW, and extraction leaves the RAW's bytes unchanged.
