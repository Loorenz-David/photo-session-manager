# 10D — Composition and framing in Session Manager (owner decision 2026-11-04)

Owner decision 2026-11-04 (chapter 15 R37, chapter 16 entry 27), from the brief "Native image composition &
framing" and the reviewed proposal (Claude Doc "Session Manager — Composition & Framing proposal"):
**Capture One develops; Session Manager frames and makes the final files.** Capture One's job ends at a
lossless, full-size, uncropped master with the photographer's adjustments. Session Manager owns the framing
(shape, position, size, rotation, straightening, flip, Normal or Split) and renders every published and
exported file from that master, compressing it once. Owner's words: "the app can't compress the images it must be one to one the image
coming from the adjustments"; option B was chosen "because of the future scalable implementation of letting
session manager to handle image adjustments".

This chapter is the authority on composition, outputs and the editor. It **replaces** 10B (shape on arrival:
the app no longer writes Capture One's crop) and narrows H3 (chapter 01). 05, 05A MC-8, 07A MC-11/12/14,
07B MC-21, 10A MC-16/17 and 10C step 4 point here for what changed.

## What the photographer sees

- A new photo arrives already framed with the default shape (`default_shape`, Square unless changed in
  Settings), centred and as large as fits. Often nothing more is needed.
- **Compose** (R, double-click, Compose… in the ⋯ and right-click menus, or the **Compose button** on the stage,
  owner 2026-11-05) turns the stage into the editor. The frame stays centred and fills the stage; the photo moves
  under it. While composing, the photo browser on the right shows as its **single strip** (the folded view of F)
  and stays put: no hiding, no edge reveal (owner 2026-11-05: "the hidden panel is uncomfortable … i prefer the
  single strip mode"). Clicking a photo in the strip moves the editor to it and keeps the changes; F still unfolds
  and folds the panel.
- A **Tools panel** on the left is made of sections that fold open and closed (Capture One tool tabs,
  Photoshop panels), remembered across launches and hidden with ⌘T. **Composition** is its first section:
  Normal | Split, the Shape menu, Turn (portrait/landscape), rotate 90° left/right, flip, a straighten
  control, the output previews at their real proportions, and Centre, Reset and Done. The straighten control
  takes a typed number of degrees and the up/down arrows (0.1° a step, Shift 1°), besides its slider (owner
  2026-11-05). An **Overlay** control picks the alignment guides drawn inside the frame (Off, Thirds, Grid,
  Golden ratio, Diagonals; O cycles them) and is remembered; while straightening, a fine square grid covers
  the image so lines can be aligned (as Capture One does), with its own on/off switch. Later colour and light
  tools arrive as more sections of the same panel.
- **Every shortcut has a control** (owner 2026-11-05: "all this keyboard commands … should have a clicking
  interface, hover over shows the shortcut"): each editor action reachable by a key also has a button or menu
  entry in the Tools panel or above the stage, and hovering any of them shows its name and key ("Split (S)").
- **Split**: one 2:1 composition whose exact middle makes two square files, Left and Right. Allowed for **any
  photo**, unreviewed included (owner 2026-11-05: "split shouldn't be limited"); places and outputs are counted
  only once it is Main or Condition. In the grid a split photo shows a middle hairline and the two places it fills
  ("03–04").
- Gestures and keys (stage only, while composing): drag or two-finger swipe moves the photo; pinch, or ⌘ +
  mouse wheel, zooms the composition around the pointer; corner and edge handles trim the frame (a locked
  shape keeps its ratio) and on release the frame glides back to the centre; arrows nudge one screen pixel
  (Shift ×10); `.` centres; 1–6 and 0 pick Original, 1:1, 4:5, 3:2, 2:3, 16:9, Free; X turns the frame; S
  toggles Split; [ ] previous/next photo keeping changes; Return finishes; Esc reverts this photo to how it
  was when the editor came to it; ⌘⌥R resets to the default; hold Z for a 100 % loupe that changes nothing.
  Snapping to the centre and the image edges; ⌥ suspends it. All workspace keys (A, C, Delete, Space, F…),
  stage swipes, the webview's page zoom and the two-finger "back" swipe are off while composing.
- Changes save automatically when a gesture ends and when leaving the photo; a dot marks unsaved changes in
  between. The composition can never extend past the photo's edge.
- The first time the editor opens, one line says "Crops made in Capture One are not used."
- **The result is shown** (owner 2026-11-05): outside the editor the stage shows the photo **as composed**
  (turned, flipped, straightened, cut to the frame; a split shows both halves with the middle hairline), not the
  whole frame. Tiles show the composed picture too, with a small **crop mark** when the composition leaves part
  of the frame out, so the photographer knows there is more to re-frame. Compose is unavailable only for a photo
  without a full-frame preview yet; its button and menu entry say why ("Waiting for Capture One's full-frame
  preview").
- **Workspace chrome** (owner 2026-11-05): the header's **Details** button opens the product details form (D);
  the duplicate Details button on the stage is removed. The **Sessions** button moves from the stage to the photo
  panel's header (beside fold and export) as an icon with a tooltip. The stage's corner stays free for later tools.

## MC-27 — The composition

**Source frame.** The developed image as Capture One exports it with **its crop ignored** and its own
orientation applied: the preview and the master (MC-28) show the same frame at different sizes. Capture
One's rotation, straighten and keystone are expected to stay at their defaults; keystone remains the
photographer's tool in Capture One (assumed, owner 2026-11-04: "keystone stays in Capture One").

**Values** (one composition per `photos` row; a copy of a photo to another session copies its composition):
- `kind`: `normal` | `split`.
- `shape`: one of `original`, `1:1`, `4:5`, `3:2`, `2:3`, `16:9`, `free` (and their turned forms through
  `orientation`); `split` is always 2:1.
- `quarter_turns` (0–3, clockwise) and `flip` (none, horizontal): applied first, lossless.
- `straighten`: degrees, −45…45, rotation about the image centre after the quarter turns and flip.
- `rect`: `x, y, w, h` as fractions (0–1) of the **turned, flipped, unstraightened** frame, origin top-left,
  y down. With `straighten ≠ 0` the rect must lie inside the rotated image (no empty corners).
- `frame_w`, `frame_h`: the source frame size in pixels when the composition was saved (the master's size
  once one exists, else the preview's), plus the Capture One geometry values read with that render: together
  the fingerprint (Revisions, below).
- `origin`: `default` | `user`. `revision`: integer, +1 on every saved change. `state`: `ok` |
  `needs_review`. `updated_at`.

**Default.** On a new photo row (capture, import), the app saves `kind normal`, `shape = default_shape`,
no turn, flip or straighten, and the largest rect of that ratio centred in the frame, `origin default`.
Setting `default_shape` replaces `crop_on_arrival` (missing → `1:1`; `original` keeps the whole frame).
No Capture One call is made. Photos that exist when this chapter ships get the default composition with
`revision 0`; that never makes a session dirty.

**Validation** (core, on every save): fractions inside 0–1, rect inside the image after straightening,
the shape's ratio honoured to within one pixel of `frame_w × frame_h`, a split exactly 2:1. Invalid → the
save is refused and the previous composition stays.

**Pixel rule** (one function, used by every renderer and by the tests): with the turned frame `W × H`,
`left = round(x·W)`, `top = round(y·H)`, `width = round(w·W)`, `height = round(h·H)`; for a split, `width`
is made even (rounded down to even) and the two halves are `[left, left + width/2)` and
`[left + width/2, left + width)`, same `top` and `height`, and `height = width/2`. Rounding is half away from zero.

**Revisions from Capture One.** With every preview and master render the adapter also reads the variant's
Capture One geometry (rotation, flip, keystone vertical, horizontal and amount). The fingerprint is those
values, the orientation, and (once a master exists) the master's pixel size. A later render is
**compatible** when the geometry values and orientation are unchanged and the master size, when both exist,
is equal: colour and light edits stay compatible and the composition stands. The aspect ratio alone is not
enough: VERIFY-15 showed that a Capture One straighten or keystone keeps 3:2 and only shrinks the export
(5981 × 3987 → 5553 × 3702 at 3°). Anything else → `state needs_review`, with the notice "Changed in Capture
One: straighten here instead" when a geometry value moved (straightening is Session Manager's, R37). In
`needs_review` the tile shows an amber corner and "Re-frame", publish confirmation and export refuse while
any of the session's main or condition photos needs it (naming them), and the editor shows the old rect mapped proportionally for confirmation with Return. Nothing
is cropped silently.

**Invariant (M33).** A saved composition reads back identically after a relaunch; the core refuses any rect
outside the image or off its shape; a photo whose source frame changed (Capture One geometry or master
size) is `needs_review` until confirmed, and is never rendered for publish or export meanwhile.

## MC-28 — Master and output render

**Recipes** (10C step 4, provisioned the same way): `PSM Preview` — JPEG, sRGB, long edge 2560, quality 85,
`export crop method = ignore`; `PSM Master` — TIFF, 16 bits, sRGB, full size, **uncompressed** (VERIFY-15: a 61 MP
master in 2.1–2.4 s and 361 MB; ZIP took 8.0–8.5 s for 307–313 MB), `export crop method = ignore`. Both disabled, as today.
`PSM Web` is no longer created or repaired.

**Master.** Rendered by Capture One through MC-16 (purpose `master`), only when an output is needed (publish
or export run) and the RAW has no master for its current `adjusted_gen` (10A MC-17). Stored in app storage
`masters/<path_key>/master-<adjusted_gen>.tif` (temp + rename); the previous one is deleted when a new one
is committed. Masters are a cache: those of a session are deleted once its publish (or, with no integration,
its export) completes and the session is not dirty, and are rendered again if needed later (orchestrator
reading of the owner's "kept while the session is in use, deleted when archived": the app has no archive
state).

**Output render** (Rust, infrastructure, job kind `render_output`, one at a time): decode the master
(16-bit), convert to linear light, apply quarter turns and flip, apply `straighten` (rotation about the
centre, high-quality interpolation), cut the pixel rect (MC-27 pixel rule; a split cuts its two halves),
resize to the output size (Lanczos3) when larger, convert back to sRGB, and encode **once**. Output setting
`output_format` (replaces the PSM Web recipe values): default JPEG, long edge 2048, quality 90, 4:4:4 chroma,
sRGB profile embedded; never upscaled. Metadata: capture date, camera, copyright copied; GPS and Capture
One private tags dropped; orientation written as upright. No other pixel operation is allowed (H3).

**Thumbnail.** From each adjusted preview, a 640 px long-edge JPEG of the whole frame for grid tiles, which
show the composition's result from it with CSS (owner 2026-11-05: tiles show the composed picture, with a crop
mark when part of the frame is left out).

**Cache key.** An output is valid for `(master adjusted_gen, composition revision, output_format)`; a change
to any of them makes it stale. Stale outputs are never exported or uploaded.

**Invariant (M34).** Every exported or uploaded image equals, within resize tolerance, the composition's
pixel rect of the master for the RAW's current `adjusted_gen` and the composition's current revision, in
sRGB; a split's two files are adjacent equal squares with no shared or missing column; the master and RAW
are never modified by composition.

## MC-29 — Outputs in export and publishing

**Outputs.** A `normal` photo has one output (slot `single`); a `split` photo has two (slots `left`, then
`right`). Positions stay per photo (chapter 06); outputs expand in photo order when a run stages its files.

**Export (05A MC-8).** Files are numbered per output within the group: a split at main place 3 writes
`<key>_03_main.jpg` (left) and `<key>_04_main.jpg` (right) and later photos move down one. `item.json`
lists the output file names in order. M15 holds per output.

**Upload filename (07A MC-11, 07B MC-21).** `<folder_key>-<uid12>.jpg` for `single`;
`<folder_key>-<uid12>-L.jpg` / `-R.jpg` for split halves; a revision adds `-r<n>` after that
(`…-L-r2.jpg`). Existing products keep their names.

**Links.** `photo_links` gains `slot`; its key becomes `(photo_id, integration, item_id, role, slot)`, and
it records the `composition_rev` its output was rendered from (beside `retouch_gen`). The replacement rule
of 07B MC-21 also fires when the photo's composition revision is greater than the link's. A link whose slot
the photo no longer has (Split → Normal, Normal → Split) is retired: its media or file id joins the item's
`own` set and is removed by gallery reconciliation (07A MC-12 step 3).

**Gallery and cover.** `D` (07A MC-12) is the ordered list of main **outputs**; the cover is main 01's first
output (the left half of a split).

**Damage report (07A MC-14).** The selected condition photos' outputs in order; a split condition photo
contributes Left then Right.

**Dirty (05A MC-6).** A saved composition change (revision + 1) on a photo of a published session is a photo
change: the session becomes dirty, like a retouch (07B MC-21). Opening and closing the editor without a change
is not.

**Invariant (M35).** After a publish completes, the gallery is exactly the main outputs in order (a split's
halves adjacent, Left first), the damage report exactly the selected condition outputs in order, and every
uploaded file was rendered from the photo's current composition revision.

## Grounding (VERIFY-15, evidence file, verified 2026-11-04)

Live on Capture One 16.8.6 (dev Mac) with ZV-E10 and a7R (9504 × 6336) RAWs: a recipe with `export crop
method = ignore` exports the full developed frame of a Square-cropped variant (5981 × 3987) in 16-bit sRGB
TIFF and in JPEG; every recipe property the app sets reads back; Capture One's straighten and keystone keep
3:2 and shrink the export (fingerprint rule above); the master and preview times are above. Owed on the work
Mac: the same times on its hardware (operational, not blocking).

## Performance targets (measured in the build, not tests)

Dragging and pinching keep the display's refresh rate on the 2560 px preview (the editor never loads the
master; gestures update a CSS transform once per frame and never wait for Rust). Output time per photo and
master render time are recorded for an a7R session of 20 photos and reported to the owner.
