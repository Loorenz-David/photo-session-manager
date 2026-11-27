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
| Photo files follow their session (A21) | Moving photos to another session (and its undo/redo, and Assign product) moves their RAWs with Capture One's files of them (settings and masks, so the edits travel; proxies, thumbnails) into that session's folder; Capture One is told to look at both folders. Never over an existing file; refused while an export or publish of a session involved runs. The session's ⋯ menu shows **Fix photo files location (N)** when photos of it still have their files elsewhere (moved before A21) | changelog A21 |
| Delete session (A17) | The session's ⋯ menu (list row, session header): **Delete session…** for test items and mistakes. Session Manager only: its files (RAWs no other session uses, Capture One's files of them, the app's caches, exported JPGs, then the emptied capture folder) go to the macOS Trash, its records are removed in one step, Shopify is not changed. Not undoable in the app; refused for Unassigned and while an export, a publish step or research of it runs | changelog A17 |
| Archived in Shopify (H4) | An archived or shop-only product opens in the **usual workspace**: its Shopify pictures (1600 px, downloaded in the background, 4 at a time, each as it arrives) fill the stage and the Main / Condition groups, and a line under the header says "Photos are on Shopify. Editing a photo brings them back to this Mac first." with **Bring back now**. Opening it **never moves the camera's capture folder** (new photos keep going where they went). Selecting and looking, the Details form, the writer and conversations need nothing back; the item's ⋯ menu offers **Refresh from Shopify**; **Publish sends its details only** ("Details only: the photos stay as they are on Shopify." / **Publish details**; gallery and damage report on Shopify untouched). An adopted product is read the first time it is opened ("Checking Shopify…"; on failure "Could not check Shopify for this product's photos." with Try again, and it behaves like any session). While visiting such a product from a conversation, **Shoot here instead** is not offered | `10H` §6; master plan archive MC-H3 |
| Bring back (H3, H4) | The first photo edit of a remote product (classify, deny, drag, compose, split, move, copy, assign) asks "Bring back N photos from Shopify to edit them? About X MB." (**Bring back** / **Cancel**), then shows "Bringing back k of N photos…"; afterwards the product is an ordinary session (the capture folder follows it). An archived product (its files cleared after publishing) or a shop-only product (linked, never photographed on this Mac) gets its **published pictures** back from Shopify at full size into its session folder (`<SKU>-main-01.jpg` …, `-b2` on a second time), **all of them or none** (three tries per picture; a failure removes what it downloaded and changes nothing). They become the session's photos in the published order, exactly as published (no crop, no turn, no house style), each linked to the Shopify media it came from, so the next Publish uploads only what was changed since. The earlier photo entries stay, hidden; the undo list starts empty; a published product stays Published and is not cleared again before its next publish. Brought-back photos are JPEGs and work like RAWs everywhere | `10H` §7; master plan archive MC-H4 |
| Retention/cleanup | Duplicates deleted 24 h after verification; Capture One preview exports deleted once copied; a storage budget (min free space, default 20 GB) with a removal order; **published, clean items are cleared N days after publish (default 4) and shown from Shopify afterwards** (amends H2); a per-item "keep files" pin | `10F`, M41 |

## Research assistant (book `08A`, `08B`; the code is `crates/research`)
What the person sees; the mechanics are in `02_architecture.md` §Research.
- **What it does.** For one item session it looks at the photos, finds visually matching listings
  (Google Lens through SerpApi), reads the pages, compares them with the piece, reads labels and
  condition photos, and writes **suggested** values into the item's form. Each suggestion carries a
  confidence (`low`, `medium`, `high`), a reason and the links it relied on. Models are OpenAI's, one
  model and effort per stage (Settings → Research → Stages).
- **When it runs.** Only on an explicit request: the header button **"Arranged — next"** (marks the
  open session arranged, queues its run, then goes to the Sessions page with the New session dialog
  open and its field focused (A19); shown as "Arranged ✓ — next" once marked, and an "Arranged" pill in
  the sessions list), opening the publish popover for a session not yet marked (it is marked first), or
  **Research / Run again** in the research panel. A19 (owner 2026-11-18): research starts by itself at
  most **once per session**: arranging queues a run only while the session has no run queued, running,
  finished or stopped at the cost limit (a failed or cancelled one may be retried); afterwards only
  **Run again** researches it again, and photo changes never queue one. Photos arriving start nothing
  (08B changelog 13). The Unassigned session is never researched.
- **Review queue (A19).** The Sessions page has **"To review · *n*"** (only the sessions whose research
  finished with suggestions waiting, in the list's order); opening one goes to its session page.
  **Details** (button or D) opens on the AI review tab while suggestions wait. When the last one is
  checked the tab shows **All checked ✓** with **Next to review →** (the next waiting session's page;
  none left: the Sessions page and "All reviewed"). In the header, an arranged session whose research
  has finished (suggestions waiting or all checked) shows **Next to review (*n* left)** in place of
  "Arranged — next" while other sessions wait.
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
- **Search this photo** (A12, owner 2026-11-18): tapping a photo in the form's media preview opens a separate
  window for that item: the photo pinned on the left (the item's other photos below it, tap one to pin it and
  search again), a browser on the right showing **Google Lens results for the pinned photo**, as if the person had
  dropped the picture into Google. Back / Forward / Reload, **Results** (back to Google's results page), the
  current site and title. **Save page** keeps the page as a small tab above the browser (site · title; tap to
  reopen, ✕ to remove); saved pages belong to the item and are still there in a later search. Selecting text on a
  page shows it in the bar under the browser with **Put in Manufacturer / Designer** (the fields whose name says
  so first; "Other field…" lists the rest), which writes the text as the person's own edit (one undo step; a
  choice or number field takes it only when the text maps to one of its values), and **Ask AI…** with the person's
  instruction ("find the production year"): the model reads only the selected text and its answers become
  **AI suggestions** (medium, the page as source) in the AI review tab, never applied. Google's first page in the
  EU asks about cookies: the person chooses, and the search starts right after. When Google cannot be used
  (no results within 30 s, a "confirm you are a person" page) the window shows its own list: the matches
  research already found for the item, or one new visual search of the pinned photo (a research call, counted
  in the day's spend). Every Ask AI and visual search is a `browse` run in the research record and export, but
  never shows as the session's research run.
  A14 (owner 2026-11-18): the selection bar is one floating bar under the page: the selected text as a quoted
  one-line snippet (the whole text on hover), **Manufacturer** / **Designer** buttons, **Other field…** (a
  searchable list inside the bar: type to filter, arrow keys, Return, Esc), **Ask AI** (a prompt inside the bar:
  Return sends, Esc cancels; "Asking AI…", then "*n* suggestions sent to AI review" with **Open AI review**,
  which brings the main window forward on that item's AI review tab) and ✕ to dismiss it until the next
  selection; with nothing selected a small hint pill shows. A18: every search is its own tab (photo thumbnail + the
  site it shows; switch and close; the others keep their page); pinning another photo only changes the reference
  picture, and "Search this photo" on it (hover) opens a new tab; page zoom (−/%/+, ⌘+ ⌘− ⌘0, kept per tab) and a
  zoomable pinned photo; Ask AI has "Answer me" (a short answer in words with one-tap "Put '<value>' in <Field>"
  chips, nothing written until tapped, kept on its tab until dismissed) and "Fill fields" (suggestions to AI
  review); selections up to 6,000 characters. A15: the selection taken stays in the bar until another
  selection, ✕ or another page (a #fragment change is the same page); leaving **Other field…** or **Ask AI**
  without acting, a "Put in" and an AI answer all keep it, because the page's own selection goes as soon as
  the owner clicks into the bar. "Put in" is confirmed by a toast with **Undo** (the
  field gets back what it held, or is emptied). Toasts show over the photo panel. The toolbar: Back / Forward /
  Reload grouped, Results, the address as a pill, Save page (Saved once the page is saved), search again; saved
  pages are pills in a "Saved" row, the one on show highlighted. Links and pop-ups a page opens in a new window
  (Google's AI Overview sources, `target=_blank`, `window.open`, an empty window pointed somewhere afterwards)
  open in the same browser area.
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

## Publication writer (book `08C`; the code is `crates/research/src/writer`, `docs/implementation/master_plan_writer.md`)
What the person sees (W1–W4; the search texts come in W5).
- **What it does.** A second kind of run in the research engine drafts the item's **title**, **description** and
  **condition report** (`custom.condition_report_description` by default) from the fields the writer may see (Details
  and Condition, no prices), research's photo note with the owner's decisions over it, and the titles of matching
  listings. One text call, no images (HW4); a deterministic post-check (banned words and patterns, lengths, damage
  named, grade word, stability, names and years only from fields) with one rewrite; a text that still fails arrives
  `low` with a "Check: …" line.
- **When it runs.** By itself when the owner presses **Next to review** (AI review tab or header) or closes Details
  / leaves the session while AI review shows **All checked ✓**: once per session (a failed or cancelled run may
  be retried), never beside research of the session, never with a toast. **Write texts** in the Product step writes
  again whenever the owner likes and shows why when it cannot ("Research is still working on this session; write
  texts when it finishes", "Add the model key in Settings → Research", …).
- **In AI review.** The texts arrive as suggestions like research's (a long text shows the previous text and the
  suggestion as paragraphs; Accept / Keep previous or Dismiss / Edit). The writer's **questions** sit above them with
  an answer box (answering writes the texts again, without photos; the answer is kept for the item) and its
  **notes** with Dismiss (a dismissed note never comes back; HW3: what the owner decided is never questioned again).
  While it writes the tab says "Writing texts…".
- **The title.** A title suggestion changes the item's title only when the owner accepts it (undo brings the old
  one back); until then the shop keeps the old title, even at publish. Typing a title removes the suggestion.
- **At publish.** A writer description that replaces the shop's description is a conflict: the item waits until the
  owner chooses (as research's conflicts do). Clean texts are `medium`, flagged ones `low` (withheld at publish).
- **Research panel.** Its own **Writer** block: what the latest write used (fields, observations, listing titles),
  each text's reason and facts, the post-check per attempt and "Rewritten once", findings and questions, cost.
- **Settings → Writer (W4).** Next to Settings → Research (shown when research is): model (those priced in Settings →
  Research; a saved model without a price stays, marked "no price") and effort, language (English, Swedish), automatic
  writing on "Next to review", the instruction of the chosen language (**Preview what is sent** shows the exact request
  for a built-in example item under the *saved* settings; **Earlier versions**, the last 20 per language, with **Use
  this**; **Reset to default**), the fields the writer may see (Details and Condition, prices unticked by default),
  style examples per language, extra banned words, target lengths, one off / note / ask mode per safeguard type with
  how often each was answered and dismissed, questions per write, the shop name for search titles. **Save** sends the
  whole `writer` slot (validated, never rebuilds the App); the writer reads it at its next run. A changed instruction
  keeps the text it replaced as an earlier version; an instruction or example list equal to the shipped one is stored
  as the shipped default, so later builds' improvements reach it.

## Conversations with Pricing (book `09B`; `docs/implementation/master_plan_conversations.md`; C1–C4 built)
Beyo Pricing is the owner's second app, where a **pricer** prices one product at a time from what Session Manager
published. The pricer asks the photographer questions about a product there; Session Manager keeps an exact **local
copy** of the questions addressed to the photographer and (from C2) shows them, notifies and lets the photographer answer.
C1 (done): **Settings → Pricing** takes the server address (https only, default
`https://pricing.beyoworkaroundtheclock.com`) and the API key (macOS Keychain only, never shown again), matches the
key's store to the connected Shopify shop (case-insensitive domain) and says in words whether it is connected; a
different server or shop asks for **Test connection** again before any call. Then the app pulls Pricing's event log
every 10 s (at once at launch, on waking and after the key changes; backing off 10 s → 5 min while Pricing cannot be
reached), re-reads each named conversation and replaces it in the copy; a refused key or a store that is gone stops
the pull until the key is replaced. The first connection is silent. Nothing is written to Pricing in C1. Read marks are
per Mac; an open conversation enters unread. Notices (`New question · <SKU> — <topic>`, `New reply · <SKU>`) are made
for questions and pricer messages, never for the app's own replies or the first connection.
C3 (answering): opening a conversation from the panel shows it as in Pricing (the pricer's dark bubbles on the left,
the photographer's light ones on the right, the quote block with its field chip, photos that open large, the reply
line, "· edited"); it counts as read while on screen with the window focused. The photographer answers with text
(Enter sends, Shift+Enter is a new line, at most 4000 characters), a reply to one message and up to ten **published**
photos of that product (gallery order, then condition; both halves of a split; "Publish first to attach photos"; none
for a conversation "Not on this Mac", where text still works). A reply is saved on this Mac at the press and shows at
once as "Sending…"; it leaves within a quarter of a second (also after a restart), never twice (one id per press,
reused by Retry), and a failure shows "Couldn't send" (or Pricing's words) with **Retry** and **Remove**; a failed
reply is never resent by itself. **Mark answered** (open conversations only) tells Pricing the fix was made in the
data; replying to a resolved conversation re-opens it as answered. The only writes to Pricing are these two.
C4 (the jump): **Open session** in a conversation of another session opens that session as a **visit**: the panel stays
open on the same conversation and the camera's new photos keep going where they went (the session being shot, or
Unassigned from the Sessions page or Settings). A slim bar under the header says `Visiting <SKU>: new photos still go to
<SKU or Unassigned>.` (the session the backend reports active) with **← Back to <origin>** (where the photographer was
before the first jump, also after several jumps) and **Shoot here instead** (activates the visited session, as a normal
opening does). Any ordinary navigation ends the visit and behaves as before; a visit does not survive a restart. From
Settings with unsaved changes, Open session asks "Save your changes?" first.

## Price labels (book `09C` incl. §14; `docs/implementation/master_plan_labels.md`; L1–L4 built)
Beyo Pricing also owns price-label templates, renders the labels and records which products have a printed label.
Session Manager only asks and generates; it never stores label state (M-L1), never edits templates and never marks a
label printed itself. Same key and store as conversations; labels need the scopes `labels:read` and `labels:write`
(Settings → Pricing → Test connection says "Labels: allowed").
- **Labels box:** the fourth box after Open / Published / All, its count = Pricing's missing price tags (has a price,
  not printed), shown as soon as the list opens. Inside it the filters (Missing price tag · Not printed · Printed ·
  All), counts and search go through Pricing; pages of 25 with Previous / Next. A row with a session on this Mac is
  that session; any other row shows Pricing's title and picture and opens as a shop-only item (adopt, 10H). Rows can
  be ticked, or **Select all N** across pages.
- **Generate label / Reprint label** in every published session's ⋯ (rows and session page; greyed "Publish first so
  Pricing can see it" before publishing), and **Generate labels for N** in the Labels box.
- **Check dialog** (nothing changes yet): products grouped by template; one card per product (1:1 picture with the
  SKU under it, one-line title, price, outcome), warnings as pills ("No price", "Can't generate: …", "Not published
  yet", a "QR link" toggle); a picture of the real label (PDF rasterised with macOS `sips`); a pinned footer.
- **Generate PDF:** runs in chunks of 300 with one idempotency key per chunk reused on every retry (M-L3), then saves
  every PDF in `~/Documents/Session Manager Labels/` and opens it in Preview at once. All well: the dialog closes
  with a toast; otherwise it stays on the problems ("N of M labels generated").
- **Markers** on every row and the session page ("Label printed · 4 Nov", "Missing price tag", "Price changed: new
  label needed", "No price yet") come from Pricing's status call and its `label.printed` / `label.reset` events in the
  10 s pull; no notifications.
- **Products from the other Mac** (shop-only items) count as Published in the sessions list until their photos are
  brought back for editing (09C §14.3).

## The phone and the current shot (book `10J`, `11` §Phone remote; `docs/implementation/master_plan_live.md`; LV1–LV3 built, LV4 the hardware check)
- **Current shot.** In a templated session one container is "current" (the Mac outlines it "Next"). Every shutter —
  the phone, the Mac or the camera's own button — goes to the shot that was current when the camera fired, even
  when its file arrives late over Wi-Fi; then the current shot moves to the next empty container in shooting order,
  else the first empty one, else none (shots go to Condition). After a Condition shot it goes back to the first empty
  container (P1). A photo with no shutter report (Capture One tether, or a file more than 5 minutes after its shutter)
  takes the shot that is current when it arrives (P2).
- **Retake** (phone) denies the photo in a shot at once (undoable) and makes that shot current; the next shot fills it
  with its presets. A Deny on the Mac no longer refills the container by itself; "Shift the rest back" stays (P3).
- **Pairing.** Settings → Phone → Pair a phone shows a QR code; the phone opens the app from it and keeps its token.
  The phone server is on by default on port 8686 and answers only paired phones (P4). Remove stops a phone at once;
  its installed app stays unpaired, so the phone's screen says to delete the app from the home screen and add it
  again from the new QR code (LV6, LV-D19).
- **Session guard.** Every phone action names its session; if the Mac switched session in between, the phone is told
  "Session changed on the Mac" and nothing happens (P5).
- **The phone app ("Shoot Companion", LV3; the owner's design, book 10J §The phone design).** Plain HTTP, no install
  needed (iPhone: Add to Home Screen opens it full screen; Android: a **Full screen** button). It follows the Mac's
  active session and shows:
  - **Main screen.** The current shot's template in black and white. **Hold** the picture for live, release for the
    template; **slide up** (55 px: half the design's distance, LV-D19) to lock live (double-tap unlocks); **slide sideways** to set how much of the template shows
    through live (0–100 %, starts at 35 %); while locked, **tap** to focus there ("AF LOCKED"; × sends the focus point
    back to the centre). Long shots turn the view 90° so the phone is held sideways. A split shot shows a centre line
    and Left/Right. The exposure (shutter · aperture · ISO) is read-only on live.
  - **Shutter.** Flash and haptic at once; the toast ("Shot 04 captured", "Condition image added") comes when the Mac
    shows where the photo landed. The phone never moves the current shot itself: it shows what the Mac says.
  - **Shot dock.** Template shots in shooting order with their final numbers, extra Main images, the **Free shot** tile
    (in a templated session its captures are Condition, card LV7), then Condition images (C1…). 56 px tiles that
    **scroll sideways**; a **tap** selects an empty shot or opens an image's preview (LV6, book 10J LV-D19: no
    magnifier, no drag-up). **Press and hold a photo (0.4 s) to lift it** (LV8, book 10J LV-D20 (3)): it grows and
    follows the finger; dropped **on a template shot** it swaps with that shot's photo (an empty shot is filled, card
    LV8); dragged **along its own group** (extra Main photos, or Condition images) the others slide aside and it is
    inserted there; dropped **into the other group** it goes to that group's end (a template shot's photo dropped
    among the extras asks first, as the right swipe does). Held at an end of the strip, the strip scrolls. A second
    finger, the phone taking the touch away, leaving the app, the Mac changing the photos meanwhile, or letting go
    more than 48 px above or below the strip puts the photo back and asks nothing. **The drop shows at once**; if
    the Mac refuses, the strip goes back and the phone says why; after 8 s without an answer it goes back and asks the
    Mac again. An empty shot and the Free shot tile never lift (their long press is a tap). **A photo on its way
    shows a quiet loading cue from the shutter until its picture** (LV9, book 10J LV-D21 (1)), in the live screen's
    strip and in the Preview's: a template shot the camera has fired for shows a slow light moving over grey in
    place of its template picture and keeps its number; a capture on its way to the Condition images (or to the end
    of Main in a session without a template) shows a waiting tile at the end of that group. "On its way" is the
    Mac's own rule: from the reported shutter until the photo exists on the Mac, at most 5 minutes. Waiting tiles
    take no tap, never lift, are no drop target, and one appearing never cancels a drag. Once the photo exists its
    own tile keeps the cue until its picture is there; the picture appears by itself, never a broken image. A
    picture that failed to load after three more tries is plain grey (it is not on its way); with Reduce Motion on
    nothing moves at all. **Next** jumps to the next empty shot in shooting
    order, then to Free shot. In Free shot live is always on (there is no template to show).
  - **Preview.** A captured image with Back, **Retake** (template shots: the photo is denied at once, undoable on the
    Mac, and the shot is current again) and **Move image**: any Main position, a new extra, or the Condition group; a
    position that has an image swaps (card LV8; the Mac's drop still replaces). **Swipe to sort** (LV8, book 10J
    LV-D20 (1)): **left = Deny** (a template shot's photo as Retake does; ⌘Z on the Mac brings it back), **right =
    Main** (a Condition image goes to the end of Main; a template shot's photo first asks "Move this photo to Main,
    outside its template order?" with Move / Cancel, and its shot is then empty and current), **up = Condition** (a
    template shot it filled is empty and current). A swipe counts from 96 px; the next photo shows at once (after the
    last one the Preview closes); a photo already in that group springs back ("ALREADY MAIN" / "ALREADY CONDITION");
    downwards does nothing. On a template shot's photo a still hold of 0.2 s shows its template instead (sideways
    then sets the overlay). There is **no Delete button** (the left swipe denies). The Preview shows **the same photo
    strip** as the live screen between the picture and the buttons, marking the photo shown: a tap shows another
    photo, a tap on an empty shot or the Free shot makes it current and closes the Preview, and photos are lifted and
    dragged there as on the live screen.
  - **Item drawer** (menu button): item name, SKU, article number (the barcode, only when the item has one), current
    shot, angle, image counts; **Fill item properties** opens an empty page for now (LV-D17).
  - Haptics on Android only (iPhone browsers have none). No wake lock: the screen follows the phone's Auto-Lock; when
    it comes back the app reconnects by itself.

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
| **Remote item** (book `10H` §3; `domain/remote.rs`) | An item whose pictures are only in the shop: **archived** (its files cleared on this Mac after publishing) or **shop-only** (adopted from the search, linked, its session has no photo of its own, and its first read found pictures); `unknown` = adopted and not read yet; everything else is `local`. Its pictures are the record in `archived_images`; it shows them in the normal workspace and its row reads "Archived in Shopify" (both kinds). Bring back turns it into an ordinary session. |
| **Publish run / item** | `publish_runs` (confirmation, H1) and `publish_items` (per session: frozen main/condition lists, frozen attributes, step, failed step, progress). |
| **Jobs** | `jobs` table: `kind` (`original_preview`, `hash`, `render_preview`, `render_master`, `render_output`, `mirror_tag`, `publish_step`, `verify_hash`, …), status, attempts, backoff; three runners (general, publish, render). |
| **Health item** | `{code, ok, detail}` shown on the Health page and as the header dots; codes registered in master plan §6.6. |
| **Conversation / message** (book `09B` §3; `domain/conversation.rs`) | One question thread about one product (Pricing calls it a *case*): `topic`, `blocking`, `status` open → answered → resolved (`resolved_by` asker or activation, known only when this Mac saw the event), `sku`, its messages (author `asker` = the pricer, `me` = this app's service account, `other`), optional quote (`field`, label, text), reply-to, attachments, `edited_at`. Core says *conversation, message, provider, store, asker, me*; no Pricing word in core (H7). Only conversations addressed to the photographer are kept. |
| **Read mark** | Per conversation, per Mac: a message is unread when the asker wrote it after the mark; a conversation *has news* when it has an unread message (`mechanisms/conversation_rules.rs`). |
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
