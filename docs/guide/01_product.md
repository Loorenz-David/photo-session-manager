# 01 — The product: idea, intention, vocabulary, decisions

Authority: the intention book (`docs/intention/intention.md` maps the chapters). This file is the
short form with chapter pointers; when in doubt the chapter wins.

## Who uses it, and for what
- One photographer (the owner) photographs second-hand items (furniture, lamps, …) and lists them on
  a Shopify store. Book `02_product_direction.md`, `03_grounding.md`.
- Two Macs: the **dev Mac** (studio, where the pipeline runs) and the **work Mac** (where real
  sessions happen; receives an ad-hoc-signed `.app` as a zip). Book `03`.
- Outcome (book `01`): "position the piece, press the shutter, sort" → a completed session: adjusted
  photos in order, split into main and condition images, item information filled, exported locally
  or published to the connected platform.
- Three integration families, one adapter each in the MVP: **camera** (Sony, directly through the
  sidecar, or through Capture One's tether), **editing** (Capture One Pro 16.8.6), **commerce**
  (Shopify). The core workflow must work with no commerce integration configured (H6).

## The end-to-end workflow (book `04`; details in the chapters named)
| Step | What happens | Chapter |
|---|---|---|
| Create item/session | New session → search the shop by SKU/title or create a local item (SKU proposed from a prefix); a session belongs to one item; `Unassigned` is the built-in session with no item | `04` S-B/S-D, `05`, `05B` (MC-22 SKU) |
| Shoot | **Direct camera**: the sidecar connects over USB-C or Wi-Fi; files arrive over the app's own FTP receiver (Wi-Fi) or Save-to-PC (cable); the capture ledger files them into the active item's folder under the Capture One session's `Capture/` root. **Or** Capture One's tether: Capture One writes the RAW; its capture-done hook tells the app. Capture One is made to show the active folder (R46 "always jump") | `10E` MC-31…MC-34, `10A`, `10C` |
| Arrival | One photo row per (session, RAW); since P66 the photo **arrives approved** (Main, end of list, arrival order); original preview from the camera's embedded JPEG within ~1 s; Capture One renders the adjusted preview (`PSM Preview` recipe); the configured Style is applied once per RAW; the colour tag mirrors the classification (green main, blue condition, red deleted) | `06` §Arrival, `10A` MC-15…MC-20, changelog 46 |
| Shooting template (optional) | Per item type, a **shooting template** (Settings → Shooting templates): containers in final order, the order they are shot in, per container an example image, framing presets (split, shape, zoom; the editor previews the crop and the result) and an optional name and description (shown in the session and the Template window). Containers are dragged into final order and, in the Shooting order view, into shooting order. In a session of that type arriving photos fill the containers **by shot order** (shutter time, the camera's file counter, held places for reported shutters), take the container's presets, and become Condition after the last container. The session shows each empty container with its example and its shot number, outlines the one the next shot goes to, and the Template window (two containers per row, like a web shop) can list them in shooting order. Anything the person changes wins: the photo is "placed by hand", a quiet warning counts photos that differ, **Back to template** restores them (one undo step). A photo without a container dropped on a filled one is a **retake** (the old one is denied); Deny leaves its container empty and the **next shot refills it** (several empty containers fill in shooting order); **Shift the rest back** moves the later photos back instead (one undo step). No model, no network | `10G`; `docs/implementation/master_plan_templates.md` §5 (the placement contract P1–P10) |
| Review | The exceptions are marked: **C** → Condition, **⌫** → Deny (hidden, never published; its RAW is deleted when the undo window closes: next completed publish or 24 h); drag to reorder; move/copy between sessions; ⌘Z/⌘⇧Z up to 25 steps per session | `06`, `05A` MC-30, changelog 46 |
| Compose/reframe | Compose mode: shape (original, 1:1, 4:5, 3:2, 2:3, 16:9, free), quarter turns, flip, straighten, crop rect; Split makes left/right outputs from one photo; default shape per settings; a composition that no longer fits a changed frame is `needs_review` | `10D` MC-27…MC-29 |
| Product form | Details / Condition / Product steps; fields come from the shop's metafield definitions (plus built-ins); values saved locally as typed with provenance (human, AI, default); defaults and saved values per item type; linked items show the shop's current values ("from Shopify"); item type comes from Shopify's product Type; AI suggestions are written by the research assistant (§Research assistant below) | `08`, `08A`, `05A` MC-9, changelog 42, 46 |
| Export | Writes the ordered outputs (the app's own render of each composition from Capture One's master) to `<export folder>/<folder key>/…` with `item.json` and `manifest.json`; also step one of every publish | `07` §Completion, `05A` MC-8, `10D` MC-28 |
| Publish | Confirmation popover (H1) → steps `prepare` (create product if local item, export) → `upload` (staged uploads, media attach, poll) → `update` (gallery = Main list in order, attributes incl. damage report, title/type/tags/description, inventory for new products) → `done`. Resumable, idempotent (H4). Unreviewed photos publish after Main (changelog 44). Waits up to 10 min for Capture One instead of failing (changelog 45). Cancel stops after the request in flight | `07`, `07A` MC-10…MC-14, `09`, `09A` MC-23/24, changelog 44, 45, 47 |
| Retouch after publish | A Capture One edit to a published photo dirties the session; republish replaces the image in place | `07B` MC-21 |
| Retention/cleanup | Duplicates deleted 24 h after verification; Capture One preview exports deleted once copied; a storage budget (min free space, default 20 GB) with a removal order; **published, clean items are cleared N days after publish (default 4) and shown from Shopify afterwards** (amends H2); a per-item "keep files" pin | `10F`, M41 |

## Research assistant (book `08A`, `08B`; the code is `crates/research`)
What the person sees; the mechanics are in `02_architecture.md` §Research.
- **What it does.** For one item session it looks at the photos, finds visually matching listings
  (Google Lens through SerpApi), reads the pages, compares them with the piece, reads labels and
  condition photos, and writes **suggested** values into the item's form. Each suggestion carries a
  confidence (`low`, `medium`, `high`), a reason and the links it relied on. Models are OpenAI's, one
  model and effort per stage (Settings → Research → Stages).
- **When it runs.** Only on an explicit request: the header button **"Arranged — next"** (marks the
  open session arranged, queues its run, opens the next session not yet marked; shown as
  "Arranged ✓ — next" once marked, and an "Arranged" pill in the sessions list), opening the publish
  popover for a session not yet marked (it is marked first), or **Research / Run again** in the
  research panel. Photos arriving start nothing (08B changelog 13). The Unassigned session is never
  researched.
- **Only chosen fields.** Settings → Research → **"Fields research fills"** (`research.fill_keys`)
  lists the shop fields research may suggest. It is **empty on a fresh install: research does
  nothing until the person ticks fields**, and a new shop field stays empty until it is added (08B
  changelog 17). "Designer and maker" (`attribution_keys`) and the detailed-condition field only say
  *how* a chosen field is decided.
- **Detailed condition** is never read from a photo: it is worked out from the condition levels
  that have a value, through the Settings → Research → "Detailed condition" table (worst filled
  level → grade; shipped default None/Minor → Very Good, Moderate → Good, Major → Worn; "Levels with
  no value do not hold the grade back").
- **In the form** (`features/form`): the AI pill's popover shows confidence, value, reason and
  sources with **Looks right / Clear** (or just edit the field; a human edit makes it human-owned);
  a fourth tab, **AI review**, after Details / Condition / Product, gathers every field holding an
  unchecked suggestion in one place, with the count on the tab (A11, owner 2026-11-18). Each row shows the
  field and its step, what the field held before ("The field was empty" or "Replaces *x* · from Shopify /
  preset default"), the suggestion, confidence, reason and sources, and the same actions as the pill
  (`SuggestionBody`, shared with the popover, so the same rules and undo apply): Looks right / Clear, or a
  conflict's Keep previous / Use suggestion / Edit, where Edit goes to the field on its own step. **Show field**
  goes to the field and opens its popover. **Accept all** is at the top of the tab. At zero the tab stays
  visible and says why: research is running, hasn't run (and why it can't), every suggestion was
  checked, or the run suggested nothing. The form header's **"n to review"** chip and the session
  header's "AI · n to review" pill open the form on this tab. The header's Research button opens the **research
  panel** (every step the run took, each kind with its own icon and look: conclusions stand out, a page
  that could not be read is a muted "Could not read" (A7), with the reason underneath: a site blocking automated
  reading is asked once per run and its other pages say "Skipped: <site> is blocking automated reading"; the shop's
  own product pages are read through Shopify's API, not its web pages; one read per product, no collection or
  category pages (A9); "Finished · n fields filled · n to review", why research cannot
  run, Research / Run again, **Accept all**, which skips `low` suggestions and conflicts). Source
  links open in the default browser. A checked value is never overwritten and a cleared value is
  never suggested again (HA1).
- **One at a time.** Research runs one session at a time app-wide (MC-A6). A queued session's panel
  says "Waiting — researching *name · SKU* first (*n*th in line)"; "Research this next" moves it to
  the front. The sessions list (beside "Arranged") and the session header show each session's research
  state: AI researching…, AI waiting · *n*th in line, AI · *n* to review, AI done, AI stopped, AI
  can't run · *reason* (A6, owner 2026-11-17).
- **At publish.** An unchecked suggestion is sent only at `medium` or `high` confidence, inside a
  confirmed publish; a `low` one never is (the popover counts them). A **conflict** (a suggestion that
  replaced a preset default or a Shopify value and is still unchecked) is shown with Keep previous /
  Use suggestion / Edit; a confirmed item with conflicts waits at the publish step `held` ("Waiting
  for you · n conflicts") and sends nothing until each is resolved; other items run normally
  (`08A` §7, 08B MC-A10).
- **Settings → Research** also holds: the two keys (OpenAI model key, SerpApi visual-search key;
  kept in the macOS Keychain, never shown again), trusted sites (reading order), extra shop
  addresses, cost limits per run and per day, prices, language, "Export research" (the record of
  chosen runs as a zip) and "Reset research settings to defaults" (keeps the person's lists).
- **Where the book lags.** 08A and 08B still describe the first design (runs starting on photo
  arrival after a quiet period, an `auto` switch, "fields research leaves alone", detailed condition
  read from photos, own-shop listings as attribution evidence). The rule today is: chapter text,
  **as amended by 08B changelog 12–18 and by the sections "What this plan supersedes in the book" of
  `docs/implementation/plans/R11_research_on_arranged.md`, `R12_research_cost.md` and
  `R13_research_chosen_fields.md`**; the later entry wins (rows in the table at the end of this file).

## Core vocabulary (book `05`; code in `crates/core/src/domain/`)
| Term | Meaning |
|---|---|
| **Item** | The thing being sold: `sku` (unique, editable), `folder_key` (derived from the SKU, frozen once photos exist), `barcode`, `item_type`, `title`, `origin` (`linked` to the integration or `local`). External ids live only in `item_links` (H7). |
| **Session** | One photo session of one item (`sessions.item_id`); `Unassigned` is row 1 with no item. Carries `published_at`, `change_seq`/`published_seq` (dirty = change after publish), `created_at`. List order: creation, newest first (changelog 46). A dirty published session shows **"Changed after publishing"** with the reasons (new photos, edited in Capture One, edited in the app; `session_changes`) and **"Keep as published"** (late photos denied, published again without publishing, one undo step; A10, book `07` "Dirty"). |
| **Photo** | One row per RAW per session (`raw_path`, `raw_hash`, `uid`); copy to another session = second row on the same RAW. `classification` ∈ `unreviewed, main, condition, deleted`; `position` 1-based within Main or Condition; previews and thumbnail paths; `integrity` (`ok`/`raw_changed`/`raw_missing`). A **composition** row per photo (10D). |
| **Attribute** | Definition (`attribute_definitions`: key, label, core type, choices, validations, group, writable) imported from the integration; value (`attribute_values`: canonical JSON, provenance, AI fields); presets per item type; platform type map in `attribute_definition_links` (adapter-owned). |
| **Session status** (derived, never stored; `crates/core/src/mechanisms/status.rs`) | first match: `publishing` (a publish item in prepare/upload/update/describe) → `failed` (latest item failed and never published) → `published` (`published_at` set and not dirty) → `ready` (≥ 1 main or unreviewed photo **and** every non-deleted photo has its adjusted preview — P64 rule) → `in_progress`. Row labels: Publishing n% · Publish failed · Description failed · Published · Ready · Publish · In progress. "Archived in Shopify" (cleared item) and "Waiting for Capture One…" / "Publish cancelled" are labels on top of these, not statuses. |
| **Shooting template / container** (book `10G`; `domain/template.rs`) | One template per item type (`type_key` = trimmed, lower-cased item type): containers with `final_no` (Main order) and `shoot_no` (shooting order), optional example image, `Presets {split, shape, zoom}`, optional `name` (≤ 60 chars) and `description` (≤ 1000; blank clears). A session is *templated* when its item's type has a template with ≥ 1 container (read at each use; no per-session link). |
| **Placement row** (`photo_places`, `PlaceRow`) | Per photo of a templated session: `slot` (its container's `final_no`), `place_by_hand`, `frame_by_hand`, `released` (left the shot count: a retaken photo, or a denied one after Shift the rest back), `key_at`/`key_name` (shot-order key a retake took over), `pending_zoom`. No row = placed by hand. A photo *differs* when placed by hand away from its expected place, or framed by hand in a container with presets (P9). |
| **Publish run / item** | `publish_runs` (confirmation, H1) and `publish_items` (per session: frozen main/condition lists, frozen attributes, step, failed step, progress). |
| **Jobs** | `jobs` table: `kind` (`original_preview`, `hash`, `render_preview`, `render_master`, `render_output`, `mirror_tag`, `publish_step`, `verify_hash`, …), status, attempts, backoff; three runners (general, publish, render). |
| **Health item** | `{code, ok, detail}` shown on the Health page and as the header dots; codes registered in master plan §6.6. |
| **Camera status** | `connected{name, link} · connecting{link} · busy{reason} · needs_action{reason, holder, message} · disconnected`; precedence in `10E` §Port changes. |

## The owner's standing product decisions (book `15`, `16_changelog.md`, `CLAUDE.md`)
- Rust/Tauri, no FastAPI, no Docker, no Python in the product (Python only as a sidecar for a
  demonstrated need; none exists). React + TypeScript + Vite + shadcn/ui; typed bindings generated
  from Rust. Design handoff rules in `docs/intention/evidence/design_handoff.md`.
- Capture One is driven only through its AppleScript dictionary (H5). The app owns a Capture One
  session document (`<home>/Pictures/Session Manager` by default) and creates/repairs its recipes
  `PSM Preview` and `PSM Master`; the photographer may point it at another session (`10C` MC-26).
- Framing (crop/turn/straighten) is the app's job, from Capture One's full-size master; colour and
  development stay Capture One's (H3 as narrowed by `10D`). Master = 8-bit uncompressed TIFF,
  prepared in the background once a session is Ready (changelog 47, P69).
- The camera is Session Manager's: on cable plug-in the app takes the camera back from Capture One
  automatically (setting, default on): save document, ask Capture One to quit, claim USB, relaunch
  (`10E` MC-34, changelog 38). Capture One is never killed; no other process is ever killed. Live
  view is out of scope. The camera's own settings are never changed by the app.
- Shopify: custom app with client-credentials token (`09A` MC-23); condition photos go to the
  metafield `custom.damage_report` (MC-24); item type = product Type (changelog 42); new products
  get inventory tracking at a remembered location/quantity (changelog 45; app needs the scopes
  `read_locations`, `write_inventory`).
- Photos arrive approved; Deny deletes the RAW after the undo window; sessions list in creation
  order (changelog 46). Unreviewed photos publish after Main (changelog 44).
- Storage is a budget with Shopify as the archive (`10F`).
- Process: unattended phases, one automatic fix then triage, behaviour tests only, narratives for
  the owner (`docs/process/flow.md`).

## Book pending updates (the code is ahead; describe the code's behaviour)
| Topic | Code today | Book still says | Owner of the fix |
|---|---|---|---|
| Approve on arrival; Deny deletes the RAW after the undo window (P66, changelog 46) | `crates/core/src/services/review.rs`, `photo_denials` table, `StorageService` rule `denied` | `06` §Arrival "joins as unreviewed", §Classification "the RAW stays"; `01` H2 "bytes stay on disk" | orchestrator (STATUS 2026-11-11 19:15) |
| Unreviewed photos publish after Main; `ready` needs a main or unreviewed photo plus every preview (P64, changelog 44) | `mechanisms/status.rs`, `mechanisms/publish.rs::frozen_main` | `04` S-G and `07` §Publish N2 "unreviewed photos never publish"; `07` §Status rule 4 | changelog 44 recorded; chapters 04/07 not reworded |
| FTP password: 4 random digits (owner 2026-11-16; was 16 chars), shown again from the Keychain; "The Shopify client ID or secret is wrong" (P68) | `mechanisms/secrets.rs::ftp_password`, command `camera_ftp_password_get`, `core/error.rs` | `10E` R-I21 "32 characters … shown once"; `09A` "Shopify credentials rejected" | orchestrator (STATUS 2026-11-11 23:17) |
| 8-bit master, prepared in the background when Ready (P69, changelog 47) | `editing/capture_one/recipes.rs` (`bits: 8`), `services/prepare_masters.rs` | `10D` MC-28 "TIFF, 16 bits … rendered only when an output is needed" | orchestrator (STATUS 2026-11-12 05:15) |
| Research starts on "Arranged — next" or Run again, never on photo arrival; no `auto`, no quiet period (R11, 08B changelog 13) | `crates/research/src/runs.rs` (`arranged`, `run_again`), `crates/integrations/src/arranged.rs` | `08A` §5 "Start", "Quiet period"; `08B` MC-A6 event table and eighth prerequisite | orchestrator (08B changelog 14: the R11 plan's "supersedes" section is the rule) |
| Research fills only chosen fields (`fill_keys`); `excluded_keys` retired; prerequisite "no field chosen"; detailed condition derived from the filled levels (R11, R13; 08B changelog 12, 17) | `crates/research/src/rules/{confidence.rs (field_class), derived_condition.rs, run_rules.rs}`, `config.rs` | `08B` MC-A4 "Field class" (five classes), MC-A6 prerequisites; `08A` §6 "Other (visible) fields" | orchestrator (08B changelog 18: the R13 plan's "supersedes" section) |
| One stage-1 call over at most `observe_max_photos` photos; page text is a product extract; no Compare for a page listing another kind of piece; for designer and maker no page of the owner's shop is evidence (R11, R12) | `crates/research/src/{stages.rs, rules/photo_plan.rs, adapters/page_reader/extract.rs, rules/confidence.rs}` | `08A` §5, §6 row `high`; `08B` MC-A4, MC-A7 | orchestrator (08B changelog 14: the R11 and R12 plans' "supersedes" sections) |
| Shopify credentials location | still `settings.json` (mode 0600) per master plan §3 | `12` says the same ("Keychain after the MVP") — consistent; noted so nobody assumes they are in the Keychain | — |
