# Photo Session Manager — Intention

**Status: COLLABORATING**
Owner: David Loorenz. Shaper: Claude (intention-shaper doctrine). Created 2026-10-23.
Raw source: `docs/intention/raw_intention.md` (identical to the project brief in `CLAUDE.md`).
Evidence beside this document: `docs/intention/evidence/`.

Naming note: the brief numbers milestones M0–M8. This document renames them **stages S0–S8** so that
`M1…` is free for the measurement ledger, as the pipeline charter requires.

---

## 1. Objective and hard constraints

**Outcome.** A photographer shooting vintage furniture positions the piece, presses the shutter, and
approves. Everything else — styling, review bookkeeping, grouping, ordering, export, upload to Shopify,
condition-report attachment — happens without them touching a file, a folder or the Shopify admin.
The finished product page on Shopify shows the gallery images in the order the photographer chose, and
the condition-report images in the condition metafield.

**Hard constraints (never traded for scope).**

| # | Constraint | Why |
|---|---|---|
| H1 | Nothing reaches Shopify without an explicit, recorded confirmation by the photographer. | Human gate; a wrong image on a live listing is a customer-facing defect. |
| H2 | RAW files are never modified, moved by us, or deleted by us. | Non-destructive; Capture One owns the files on disk. |
| H3 | All imaging (style, crop, masks, export) is done by Capture One. We never decode or process pixels. | Reuse; no image-processing code in this repo. |
| H4 | Every step is idempotent. After a crash or restart at any point, resuming produces the same end state with no lost product association and no duplicate upload. | Single-operator tool with no admin on site. |
| H5 | No fragile UI automation (no simulated clicks or keystrokes into Capture One). Only its AppleScript dictionary. | Breaks on every Capture One update. |
| H6 | Runs on a Mac with Capture One Pro, as one local process, started by a double-click launcher. | Work-Mac deployment with no engineer present. |

---

## 2. Grounding

### 2.1 Repository state
Empty at shaping time: only `CLAUDE.md`, the raw intention and the Capture One dictionary dump exist.
There is no code, no contract system and no architecture graph. The charter's standing rules are the
baseline.

### 2.2 Stack (owner decisions, 2026-10-23)
- Python managed by **uv** (pinned interpreter + lockfile), **FastAPI**, **SQLite**. One process.
- **React + TypeScript + Vite + shadcn/ui** for the UI, served as built static files by FastAPI.
  Node is installed on the work Mac for builds.
- Capture One control through **osascript (JXA)** from a single serial queue. The community
  `capture-one-mcp` is not used (resolved in §12, entry 3).
- Folder watching through **watchdog** (FSEvents) as fallback only.
- Launcher: macOS `.app` compiled from AppleScript source in `launcher/`.
- Shopify: **Admin GraphQL API 2026-10**, custom dev-app token.

### 2.3 Capture One 16.8.6.25 — what the dictionary gives us
Full findings in `docs/intention/evidence/capture_one_16.8.6.md`. Load-bearing facts:
- `variant.color tag` and `variant.rating` are read/write integers.
- `application.capture done script`, `processing done script` and `batch done script` are official
  properties: Capture One calls a script we name after each capture, after each processed file, and
  when the process queue drains.
- `document.kind` is `session` or `catalog`; a session exposes `captures` (Capture folder, read/write),
  `output`, `selects`, `trash`, `last captured file`.
- `process` takes a variant or a RAW path plus a recipe name. `apply style` takes a layer and a style name.
- Barcode scanning in Capture One is Enterprise-only; scripted autocrop is Cultural-Heritage-only.
  Neither is used.

### 2.4 Two Macs
Development and testing happen on the dev Mac, which has the camera, the cable and Capture One Pro.
The work Mac clones the repo and runs the launcher. The two are the same macOS version (26.5).

---

## 3. Core workflow (one shooting day)

1. **Start.** Photographer double-clicks the launcher. The app starts, opens the UI in the browser,
   checks Capture One is running with a *session* document open, and installs its capture-done and
   processing-done scripts into Capture One. If any check fails the UI says exactly what to do.
2. **Select product.** Photographer scans a barcode or types a SKU into the search bar. The app looks
   the product up in Shopify (S6+). Found: the card shows title, SKU, status and existing Shopify
   images. Not found: the app creates a *local* product with that SKU, flagged `create_on_publish`.
   The product becomes the **current product** and the app sets Capture One's capture folder to
   `Capture/<SKU>/`.
3. **Shoot.** Each shutter press lands a RAW in `Capture/<SKU>/`. Capture One applies the house style,
   AI masks and AI Crop (Next Capture Adjustments, S1). Capture One calls our capture-done script,
   which posts the RAW path to the app. The app records the image under the current product and asks
   Capture One to process it with the **preview recipe**; the preview JPEG appears on the product
   card within seconds.
4. **Review.** Photographer accepts or rejects in either place: color tags in Capture One, or buttons
   or keys in our UI (which set the same color tags through AppleScript). Capture One's color tag is
   the single fact; the app polls it and mirrors it.
5. **Classify and order.** On the product card, accepted images are dragged between **Gallery**,
   **Condition** and **Discard**, and ordered within Gallery and Condition. Quick keys do the same.
6. **Retouch (optional, any time before publish).** Photographer edits in Capture One by hand. The
   app does not need to know; export happens later.
7. **Publish.** Photographer selects one or several products and presses Publish. The app shows the
   confirmation screen: per product, "add X gallery, add Y condition, replace: no/yes, create as
   draft: no/yes". Confirming records a **publish run** (H1). The app then: exports accepted Gallery
   and Condition images with the **web recipe** into `Output/<SKU>/` with ordered filenames; uploads;
   attaches gallery media and reorders; uploads condition files and sets the metafield; verifies;
   marks the product published. Discards never leave the Mac.
8. **Resume.** If the app or the Mac dies at any step, the next start resumes every incomplete publish
   run from its recorded state (H4).

The **unassigned** case: a capture that arrives with no current product selected is recorded under a
built-in "Unassigned" bucket and shown prominently; the photographer assigns it to a product later.
No capture is ever dropped (M1).

---

## 4. Domain model

Field ownership column: **U** = photographer via UI, **S** = system (our code), **CO** = Capture One
(read by us, written by us only through AppleScript), **SH** = Shopify (read by us; written by us
only inside a confirmed publish run).

### 4.1 `products`
| Field | Owner | Notes |
|---|---|---|
| id | S | local integer |
| sku | U | required, unique, immutable after first capture (it is in folder and file names) |
| barcode | U/SH | optional; from scan or from Shopify |
| title | SH/U | from Shopify when found; typed by the photographer when `create_on_publish` |
| shopify_product_id | SH/S | null until found or created |
| create_on_publish | S | true when lookup found nothing; cleared when the product is created |
| replace_flag | U | default false; shown on the confirmation screen |
| status | S | see §4.5 |
| created_at, updated_at | S | |

### 4.2 `images` (one row per RAW)
| Field | Owner | Notes |
|---|---|---|
| id | S | |
| product_id | S/U | set at capture from the current product; "Unassigned" bucket if none; reassignable by the photographer until published |
| raw_path | CO | absolute path, unique; never changed by us |
| raw_hash | S | SHA-256 of the RAW bytes, computed once; used for dedupe and for M6 |
| co_variant_id | CO | Capture One variant id, resolved after capture |
| preview_path | S | output of the preview recipe, null until it exists |
| review_status | CO (fact) / S (mirror) | `pending` / `accepted` / `rejected`, derived from the color tag (§5) |
| group | U | `gallery` (default) / `condition` / `discard` |
| position | U/S | 1-based, contiguous within (product, group) for gallery and condition; null for discard |
| captured_at | CO | from the file's mtime at capture |
| shopify_media_id | SH | gallery only |
| shopify_file_id | SH | condition only |
| upload_status | S | `none` → `exported` → `staged` → `uploaded` → `attached` → `ready` / `failed` |

### 4.3 `publish_runs` and `publish_items`
A publish run is the recorded confirmation (H1): id, created_at, confirmed_at, confirmed_summary
(the exact text shown), status (`confirmed` → `exporting` → `uploading` → `verifying` → `done` /
`failed`), error. One `publish_items` row per product in the run with its own status and the
per-product options frozen at confirmation (replace, create_as_draft, title).

### 4.4 `jobs`, `edits`, `capture_events`, `app_state`
- `jobs`: id, type (`preview` / `export` / `upload` / `attach` / `reorder` / `metafield` / `verify`),
  target id, status, attempts, next_attempt_at, error. Durable queue; the worker is in-process.
- `edits`: id, image_id, provider (`capture_one`), command (JSON), before_values (JSON), created_at.
  Written by the Edit Engine; v1 ships the interface and the log, and only the commands §9 lists.
- `capture_events`: every hook call and every watcher event, raw, with source and timestamp. The
  exactly-once guarantee (M1) is proven against this table.
- `app_state`: single row — current_product_id, capture_one_session_path, last_poll_at.

### 4.5 Product status (complete order)
`shooting` → `reviewing` → `ready` → `publishing` → `published`, plus `failed` (reachable only from
`publishing`, returns to `ready` on retry). Transitions: `shooting` while the product is current;
`reviewing` once it stops being current and has at least one image; `ready` when it has at least one
accepted gallery image and no pending image; `publishing` while its publish item is active;
`published` when the item is `done`. A published product that receives new captures goes back to
`shooting`. The order is used for sorting the product list: active work first.

### 4.6 Capture One objects we depend on
Capture One *session* document (not catalog) with its Capture and Output folders. Two process
recipes created by the photographer in S1: a **preview** recipe (JPEG, ~1600 px long edge, into
`Output/_preview/`) and a **web** recipe (JPEG sRGB, long edge set in S1, into `Output/`). One
house **style**. Exact names are configuration, recorded in `.env` and defaulting to
`PSM Preview`, `PSM Web`, `PSM House`.

---

## 5. Facts versus derived values

- **Facts** we store: RAW path and hash, Capture One variant id, color tag integer as last read,
  group, position, Shopify IDs, publish-run confirmation text, job states.
- **Derived, never stored as a fact:** `review_status` is a pure function of the color tag
  (green → accepted, red → rejected, anything else → pending; mapping integers verified in
  mechanism inventory). Product status is derived from its images and publish items and
  recomputed, never hand-set. "Ready to publish" counts on the confirmation screen are computed
  at display time and frozen only into the publish run's summary text.
- **Missing data is never inferred:** an image with no preview shows a placeholder, a product with
  no Shopify match is shown as "not on Shopify", never guessed from a similar SKU.

---

## 6. External-source strategy

### 6.1 Capture One (local, AppleScript)
- One serial queue; every osascript call has a timeout and is logged with its duration.
- Inbound events: capture-done and processing-done scripts post to `http://127.0.0.1:8765/events/...`.
  Folder watcher on Capture and Output folders is the fallback and reconciler; both paths write
  `capture_events`, and the image row is created once by RAW path.
- Outbound commands (v1): set color tag, read color tag/rating for a set of variants, set capture
  folder, process variant with recipe, apply style, select variant. Each is one JXA script file in
  the repo, tested against the real app on the dev Mac (rule 9 exemption: the dependency is local).
- Polling: color tags for the current product's images every 3 s while the UI is open, and for a
  product when its card opens.

### 6.2 Shopify (remote, Admin GraphQL 2026-10)
- Read (S6): product lookup by SKU and by barcode via product variant search; existing media list
  with positions; metafield `custom.condition_images` current value.
- Write (S7, only inside a confirmed publish run): `productCreate` (status DRAFT, one variant with
  SKU and barcode), `stagedUploadsCreate` + HTTP upload, `productUpdate` with media, poll media
  status to `READY`, `productReorderMedia` + job poll, `fileCreate` for condition files + poll,
  `metafieldsSet` on `custom.condition_images` (`list.file_reference`).
- Replace semantics: see decision card D3.
- Every write is preceded by a read that checks whether it already happened (by stored ID, then by
  filename and hash in `alt`/filename) — H4.
- Source-evidence doc `docs/intention/evidence/shopify_admin_2026-10.md` is written when the owner
  provides the dev store (pre-implementation protocol, §10).

---

## 7. Operations

- **Start/stop:** launcher app syncs the uv environment, builds the UI if its sources changed,
  starts the server, waits for `/health`, opens the browser. A Stop app ends the process.
- **Logs:** one rotating file per day under `data/logs/`, plus the capture-event table.
- **State backup:** SQLite file copied to `data/backup/` at every start and after every completed
  publish run; last 30 kept.
- **Config:** `.env` with Capture One recipe/style names, session path, Shopify store and token,
  port. Secrets never committed (rule 8).
- **Health screen** in the UI: Capture One reachable, session open, hooks installed, camera
  connected, Shopify token valid.

---

## 8. Scope ladder

**Must ship (v1).**
Stages S2–S8 of the brief: capture hook + watcher; orchestrator with product selection and
restart safety; review mirroring; groups, ordering, product cards; Shopify lookup; Shopify publish
with create-as-draft, reorder, condition metafield, verify, retries; hardening.

**Only if cheap.**
Edit Engine v1 commands beyond logging: `apply_style(name)` and `set_color_tag` through the
interface. Rating display (read-only). Pywebview window instead of the browser.

**Explicitly deferred (non-goals for v1).**
AI or text-command editing (E3–E5). Voice. Multi-user or remote access. Catalog documents.
Video. Any Capture One operation outside the dictionary. Sony SDK tethering. Shopify theme work
(the metafield must already render; see D5).

---

## 9. Edit Engine (built now, barely used now)

`EditProvider.apply(image, command) -> EditResult`. Commands are JSON with a `type` and
parameters; v1 ships `apply_style`, `set_color_tag`, and `process` (recipe). Every call writes an
`edits` row with the values read before the change, so undo and "what do we always do to this kind
of shot" are possible later. No AI provider in v1.

---

## 10. Pre-implementation protocol

1. **S0 tether PoC** (photographer, no code): `docs/checklists/S0_tether.md`. 20 shots, 0 lost,
   shot-to-preview time measured.
2. **S1 auto-processing** (photographer, no code): `docs/checklists/S1_auto_processing.md`.
   Style, AI masks, AI Crop in Next Capture Adjustments; the two recipes created and named.
3. **Mechanism inventory** on this document once RATIFIED.
4. **Shopify evidence** when the owner provides the dev store, before S6 is planned.
5. Code before S0/S1 are confirmed is limited to what does not depend on them: project skeleton,
   data model, job queue, launcher, UI shell, Shopify read-only (owner decision 2026-10-23).

---

## 11. Measurement ledger (root of the trace chain)

| ID | Observable outcome, measured true ⇒ shipped | Defect family it guards |
|---|---|---|
| **M1** | Every RAW that appears in the Capture folder while the app runs has exactly one `images` row, associated with the product that was current at its capture time (or Unassigned if none). Holds when the hook fires, when only the watcher fires, and when both fire. | lost captures, duplicate rows, wrong product |
| **M2** | For any image, the `review_status` the API reports equals the mapping of Capture One's current color tag within one poll interval; a tag set through our UI is read back from Capture One identically. | stale or inverted accept/reject |
| **M3** | After a publish run completes, the product's Shopify media order equals the gallery order held locally at confirmation; every condition image is in the metafield and none is in product media; no discard image has left the Mac in any form. | misorder, wrong destination, leak |
| **M4** | Re-running a publish run from any interrupted job state — including process kill between upload and attach — ends with the same Shopify media set, file set and metafield as an uninterrupted run. | duplicate uploads, half-published products |
| **M5** | Every Shopify mutation the app has ever issued is traceable to a `publish_runs` row whose `confirmed_at` precedes it and whose summary names that product. Zero mutations exist without one. | unintended writes, silent publishing |
| **M6** | SHA-256 of every RAW equals its `raw_hash` recorded at first sight, at every later check; RAW paths are never changed by the app. | RAW corruption or relocation |

Operational measurements (recorded in S0 and S8, not tests): shot-to-preview time; cold start on
a fresh Mac by double-click.

---

## 12. Shaping changelog

1. **2026-10-23 — Shaper.** Grounded on the Capture One 16.8.6 dictionary (evidence doc). Replaced
   "undocumented `CO_CaptureDone`" with the official `capture done script` / `processing done
   script` properties.
2. **2026-10-23 — Shaper.** Milestones renamed S0–S8 so the measurement ledger owns `M` IDs.
3. **2026-10-23 — Shaper (resolution, owner may override).** `capture-one-mcp` not adopted: it adds
   a Node/MCP runtime designed for LLM clients, while the dictionary covers every command we need;
   our adapter is a handful of JXA files behind one queue.
4. **2026-10-23 — Shaper (resolution).** `review_status` no longer carries `approved`; final
   approval is a property of the publish run (H1), not of an image.
5. **2026-10-23 — Shaper (resolution).** Product association is carried by the file system too:
   the app sets Capture One's capture folder to `Capture/<SKU>/` when a product becomes current.
   SQLite remains the authority; the folder makes recovery and manual inspection trivial. See D1.
6. **2026-10-23 — Shaper (resolution).** Default group for an accepted image is `gallery`.
   Discard images have no position and are never exported.
7. **2026-10-23 — Owner decisions folded from chat:** uv runtime, launcher app, FastAPI
   industry-standard layout without the owner's contract set, React + shadcn/ui, Node allowed on
   the work Mac, custom dev-app token for Shopify, search fields barcode and SKU, public repo.

---

## 13. ⚠ OWNER DECISIONS REQUIRED (5)

**D1 — Capture folder per product?**
Question: May the app set Capture One's capture folder to `Capture/<SKU>/` each time you select a
product?
Story: You scan BEYO-1042, shoot eight frames, scan BEYO-1043, shoot six. Without per-product
folders all fourteen RAWs sit in one Capture folder and only the database knows which is which. If
the database were ever lost, you would sort fourteen files by eye. With per-product folders the
Finder shows the answer, and Capture One's browser shows one product at a time.
Branches: **Yes** — folder set on select; recovery and inspection trivial. **No** — single folder,
database only; Capture One shows all day's frames together.
Recommendation: **Yes**; the dictionary exposes the folder as writable and it costs one command.
On silence: gate holds.
Trace: §3 step 2, §4.2 raw_path, M1.

**D2 — Review signal: color tags only?**
Question: Is green = accepted, red = rejected, no tag = pending, with star ratings ignored?
Story: You review forty frames in Capture One with the keyboard: G for green, R for red, as you do
today. The app mirrors that. If ratings also counted, a 3-star frame with no tag would be ambiguous
and two frames could disagree with each other.
Branches: **Tags only** — one key per verdict, no ambiguity. **Tags and ratings** — a rule is
needed for every combination.
Recommendation: **Tags only**; ratings shown read-only if cheap.
On silence: gate holds.
Trace: §5, M2.

**D3 — What does Replace do on Shopify?**
Question: When Replace is on for a product, should the app delete all existing product media and
replace the whole condition list, or only replace images whose filename matches?
Story: BEYO-1042 already has five old photos on Shopify. You reshoot it and tick Replace. With full
replace, the listing ends with exactly your new set; the old five are deleted permanently. With
match-only, old photos with different names survive next to the new ones and you clean up by hand.
Branches: **Full replace** — deterministic, destructive, shown in red on confirmation. **Match
only** — safer, leaves leftovers.
Recommendation: **Full replace**, because the confirmation screen already names it and leftovers
are the defect the flag exists to prevent.
On silence: gate holds; Replace unavailable until decided.
Trace: §6.2, M3, M5.

**D4 — Export timing?**
Question: Should the web-size export run when you confirm Publish, or earlier when an image turns
green?
Story: You accept twelve frames at 10:00, retouch three of them in Capture One at 14:00, and
publish at 16:00. Export-at-publish uploads the retouched versions and makes you wait about a
minute at 16:00. Export-at-accept uploads the 10:00 versions unless the app re-exports on every
adjustment, which it cannot reliably detect.
Branches: **At publish** — always the latest edit; publish takes longer. **At accept** — fast
publish; risk of stale exports.
Recommendation: **At publish**.
On silence: gate holds.
Trace: §3 step 7, M3.

**D5 — Does `custom.condition_images` exist and render?**
Question: Does the Shopify store already have the metafield definition `custom.condition_images`
(type list of files) and a theme section that displays it?
Story: The app uploads six condition photos and sets the metafield. If the definition is missing the
write fails; if the theme does not render it, the photos are stored but invisible to customers and
you discover it weeks later.
Branches: **Exists and renders** — nothing to do. **Exists, no theme support** — app ships, theme
work is a separate task. **Does not exist** — we create the definition via API in S7; theme work
separate.
Recommendation: check the store's Settings → Custom data → Products before S6.
On silence: gate holds for S7 only; S2–S6 unaffected.
Trace: §6.2, §8 non-goals, M3.
