# 16 — Shaping changelog

1. **2026-10-23 — Shaper.** First draft (single file) grounded on the Capture One 16.8.6 dictionary;
   official hook properties replace the undocumented `CO_CaptureDone`; milestones renamed S0–S8.
2. **2026-10-23 — Shaper.** `capture-one-mcp` not adopted; `approved` removed from image state;
   per-product capture folder proposed (D1); default group gallery.
3. **2026-10-23 — Owner.** uv runtime, FastAPI standard layout without the contract set, React +
   shadcn/ui, Node allowed, custom dev-app token, public repo.
4. **2026-10-24 — Owner (chat).** Review model: approve / condition / delete with undo; fast original
   preview then adjusted version with toggle; phone remote wanted.
5. **2026-10-24 — Owner (architecture memo).** Commercial direction; Tauri + Python sidecar;
   Items/Attributes/Photos/Sessions vocabulary; integration layer with capabilities; generic
   attribute schema; AI on the generic model; session completion independent of publishing;
   MVP scope, production architecture.
6. **2026-10-24 — Owner.** C1 public, C2 session per item, C3 schema now / agent later, C4 one
   transport, C5 Capture One only provider.
7. **2026-10-24 — Shaper.** Draft restructured into this book (map + chapters) at the owner's
   request so downstream sessions load only relevant chapters. Design handoff folded in:
   ready rule, status order, dirty state, undo timing, toggle, keyboard, field kinds, customize
   fields, batch publish, notifications, condition attribute = `custom.damage_report`.
   "Replace" flag retired in favour of the dirty-state re-publish (N1 opened). N2 and N3 opened.
   Measurement M7 (integration-free completion) added.
8. **2026-10-24 — Owner.** D1 yes; D2 mirror as an editing-provider capability, coloring likely
   not needed; N1 mirror; N2 exclude (yes to recommendation); N3 after the desktop MVP. D4 was
   misread as "Capture One retouch export" and is re-asked in plain words; the owner's real
   question (how the adjusted preview can be live, and reflect later edits) is answered in
   chapter 10 (adjusted hook + sidecar watcher, re-render through the preview recipe).
9. **2026-10-24 — Owner (second architecture memo).** Rust/Tauri replaces Python/FastAPI as the
   backend; Python optional; camera connectivity and editing software are integrations behind
   provider interfaces with capabilities; Shopify stays an optional commerce integration; layers
   frontend / core / services / integrations / infrastructure; Tauri commands thin; MVP
   integrations exactly Sony ZV-E10, Capture One, Shopify; validate packaging early.
10. **2026-10-24 — Shaper.** Memo folded in: chapters 01 (H3, H5, H7, H8), 02, 03, 04, 05, 06, 09,
    10 (renamed "Camera and editing providers"), 11, 12, 13, 14 (M2 → write-direction mirror),
    15 (R16–R25; R1, R2, R8, R10 superseded). Conflicts found and opened as cards: A1 (R8 "one
    transport" vs Tauri commands), A2 (camera provider vs Capture One owning the tether).
    `primary variant adjusted script` added to the hooks in use; VERIFY-8/9/10 added to the
    Capture One evidence. `.gitignore` and `CLAUDE.md` updated to the Rust stack.
11. **2026-10-24 — Owner.** A1 Tauri commands now, server later (R26). A2 camera through Capture
    One for the MVP; the physical camera link (USB or wireless) remains open and must not disturb
    the implementation (R27). D4 to be explained again.
12. **2026-10-24 — Owner.** D4 at publish (R28). Preview refresh rule: 15–20 s after the last edit
    in Capture One, overridden by the app window returning to the front (R29). Open-decision ledger
    empty; status set to READY_FOR_RATIFICATION and the ratification surface presented.
13. **2026-10-24 — Owner: RATIFIED.** David Loorenz approved the ratification surface presented
    2026-10-24 (outcome; measurements M1–M7 verbatim; scope ladder; D2 interpretation confirmed by
    approval; no open cards). Post-ratification changes follow the amendment path: contract-grade
    additions by the mechanism inventory, material semantic changes reopen the gate.
14. **2026-10-25 — Mechanism inventory (round 1).** Contract chapters added: 05A (MC-1…MC-9),
    07A (MC-10…MC-14), 10A (MC-15…MC-20); invariants M8–M27 registered in chapter 14; map rows
    added. Runtime probes against Capture One 16.8.6.25 recorded under VERIFY-1…10 in the
    evidence (VERIFY-11/12 added). Capture-dependent probes are UNPROBED: the camera did not
    release the shutter on any scripted capture. Contradictions resolved by contract, pending
    owner ratification (no product sentence changed): R-I1 session attribution by capture folder
    plus `_Unassigned` folder; R-I2 unreviewed position null, displayed after Main; R-I3 rows may
    move out of a published session; R-I4 photo links scoped by item; R-I5 media nobody confirmed
    removing makes `update` fail rather than be deleted; R-I6 photos deleted or moved after
    confirmation are skipped; R-I7 "Publish failed" label whenever the latest item failed;
    R-I8 attribute writes are a diff and never clear; R-I9 M3's condition clause is over the
    damage-report selection; R-I10 remote dedupe by upload filename only; R-I11 SKUs whose folder names collide ignoring
    case cannot coexist; R-I12 move/copy into a
    session already holding the RAW is refused; R-I13 dirty survives changes made during a
    publish; R-I14 a run sends the attribute values frozen at confirmation; R-I15 sidecar fallback only at startup and focus; R-I16 startup scan covers every
    per-SKU folder; R-I17 `date_time` metafields read-only. Open card I1 (retouch after
    publishing) in chapter 15. Status header unchanged: no material semantic change.
15. **2026-10-25 — Owner: I1 answered Yes** (relayed by the orchestrator). A Capture One retouch
    of a published photo makes the session dirty, and the next publish uploads a new revision and
    replaces the old media in the same gallery place (R30). Contract MC-21 and invariant M28 in
    new chapter 07B; pointers in 05A MC-6, 07 Dirty, 07A MC-11/MC-12, 10A MC-17; map row added.
    Owner decision, so the header stays RATIFIED. R-I1…R-I17 stand (owner informed, no objection).
16. **2026-10-25 — Owner: planner card 2 answered** (relayed by the orchestrator). SKU proposed
    from a prefix and editable before confirming; the owner added that a SKU must stay editable
    after the first picture lands, replacing chapter 05's "immutable once a photo exists" (R31).
    Contract MC-22 and invariant M29 in new chapter 05B: stored folder key frozen at the first
    photo, export under the current SKU, SKU/barcode diff-written to the shop at publish, shop
    clash check at create and edit. Pointers in 05, 05A MC-2/MC-8, 08; map row. Owner decision,
    header stays RATIFIED. Plans P01, P07, P09, P12, P14 and the master plan amended by the
    orchestrator.
17. **2026-10-26 — Grounding fold-back from the Shopify adapter projection** (orchestrator).
    07A: SHOPIFY-VERIFY-6 extended with a search-lag check, SHOPIFY-VERIFY-7 added (atomic create
    with SKU/barcode, identity write). MC-11 prepare notes that a non-atomic create or search lag
    requires a recovery lookup, to be contracted if the probe shows it. MC-13 writability also
    requires the definition's access to permit writes; SV-1 and SV-4 record access and media
    delete. 09: the type map pointer now names 07A MC-13. No product sentence changed; header
    stays RATIFIED.
18. **2026-10-26 — Owner: Shopify access, probe store, condition field** (relayed by the
    orchestrator). Owner's words: "im using a dev app … in the app i have the client key and the
    secrete"; "this is a test store, later upon completion we will use the real store, it should
    be more than enought to change the credentials and shop to make that change"; "the condition
    report images are uploaded to a specific metafield, that is set up on app settings, for now no
    interface, so we will set it up manually ( metafield key space )". New chapter 09A: MC-23
    (client credentials grant, token in memory only, refresh and one 401 retry, error messages,
    store switch by settings) and MC-24 (`integration.condition_photos_key`, initial
    `custom.damage_report`, the only `media_list` definition; misconfiguration shown in health and
    the damage report skipped), invariant M30, decision R32. Pointers in 07A MC-13 table and MC-14
    (and SHOPIFY-VERIFY-0 in its Grounding list), 09, 12, intention.md map. 05B MC-22 unchanged: when the shop
    holds the proposal, the proposal advances to the next free number (offered to the owner as
    "keep as written"; no change requested). Owner decisions, header stays RATIFIED.
19. **2026-10-26 — Tag merge base corrected** (orchestrator, from P14 review). 07A MC-13 Product
    tags: `base` is the app's own tag list at the last successful `update` (refreshed even when
    nothing is written), not the remote tags at the last read, which removed admin-added tags on
    the second publish and never propagated app removals. Realises the clause's stated purpose
    ("tags added in the Shopify admin are not lost"); option A of the P14 review card. R33.
20. **2026-10-26 — Ch04 S-G aligned with N2** (orchestrator, from P15 implement r1). S-G said the
    popover notes that unreviewed photos "will publish in Main"; the owner's decision N2 (R19) and
    ch07 §Publish exclude them ("n unreviewed photos will not be published"), and 05A MC-7 requires
    no unreviewed photo for `ready`. S-G's sentence is corrected; no behaviour changes.
21. **2026-10-26 — Capture root corrected** (orchestrator, from the owner's first real run, R4). 05A MC-2: the
    capture root is `canon(<document id>/Capture)`; the `capture` favourite is the current capture folder,
    not the root (evidence Correction 2026-10-26). R34. Plans P17/P18 added for the real-run findings.
22. **2026-10-29 — Owner: shape on arrival** (relayed by the orchestrator). Owner's words: "on the camera I'm
    choosing cropping 1:1 … the imported images because they are raw are brought without that 1:1 cropping
    … my end intention is 1:1"; choice "App sets Square". New chapter 10B: MC-25 (setting `crop_on_arrival`,
    default `Square`; once per new RAW whose crop is untouched, through the editing-provider lane; health on
    failure or unknown ratio; tiles in true proportions), invariant M31, decision R35. Chapter 10 "Basic
    layer" no longer names crop as a Next Capture Adjustment. Evidence: "Crop by script" (2026-10-29).
    Owner decision, header stays RATIFIED.
23. **2026-10-29 — 10B give-up limit** (orchestrator, from the P23 planner's card 1). MC-25 §Failures said "give up
    after the same limit" as MC-19, which has none. Fixed at 8 attempts (≈3 min of back-off), realising the
    clause's stated purpose (a visible health warning instead of silent retries); option A of the card.
24. **2026-11-03 — Owner: Capture One set up by the app** (relayed by the orchestrator). Owner's words: "what I'm
    looking for is Session Manager managing all that infrastructure and configuration so that the photographer
    only focuses on take picture, retouch if needed, upload"; choices "its own file" and "open ours alongside".
    New chapter 10C: MC-26 (default own session document; ensure sequence launch → create → open → recipes →
    style → hooks; front document on activation; other documents never touched; VERIFY-13/14 with a stop if two
    documents cannot be open), invariant M32, decision R36. Pointers in 10 (startup), 12 (health), map.
    Owner decision, header stays RATIFIED.
25. **2026-11-03 — 10C grounded** (orchestrator, P24 task 0 live probe). Recipes are application-wide (step 4 no
    longer says "in that document"); Capture One's `open` replaces the front window, so step 3 opens the app's
    document via a throwaway new window, keeping the "alongside" decision. No product sentence changed.
26. **2026-11-04 — Owner: no house style by default** (relayed by the orchestrator). The default `style_name` was
    `PSM House`, a style Capture One does not have, so a fresh setup started with a red Health item. Owner accepted
    the orchestrator's recommendation (an empty default): 10C step 5 now states `style_name` defaults to empty, so
    the next-capture settings are left as the photographer set them. Owner decision, header stays RATIFIED.
27. **2026-11-04 — Owner: composition and framing in Session Manager** (brief "Native image composition &
    framing", proposal reviewed in a shared doc). New chapter 10D (MC-27…MC-29, M33–M35, VERIFY-15): Capture
    One exports an uncropped 16-bit master, the app frames and renders the final files once. H3 narrowed (01),
    R9 narrowed and R37 added (15), 10B retired (no more Capture One crop writes; `crop_on_arrival` becomes
    `default_shape`), 10C recipes become PSM Preview 2560 px and PSM Master, M15 counts outputs, M31 retired;
    05, 05A MC-6/MC-8, 07A MC-11/12/14, 07B MC-21 and 10A MC-16/17 point to 10D for what changed. Owner
    decision, header stays RATIFIED.
28. **2026-11-05 — Owner: composition corrections after the first try-out** (relayed by the orchestrator). Owner's words include "the hidden panel is uncomfortable … i prefer the single strip mode", "split shouldn't be limited", "all this keyboard commands … should have a clicking interface, hover over shows the shortcut". 10D §What the photographer sees: single strip while composing (edge reveal dropped), Compose button, overlays and straightening grid, typed straighten, a control and tooltip for every shortcut, Split for any photo, composed result on the stage and tiles with a crop mark, header Details wired and the stage duplicate removed, Sessions to the photo panel header; MC-28 §Thumbnail follows. Decision R38. Orchestrator readings (reversible): the overlay set and O to cycle them (Photoshop's crop-overlay key), 0.1°/1° straighten steps, the crop mark shown whenever the rect is smaller than the frame or the photo is straightened. Owner decision, header stays RATIFIED.


29. **2026-11-05 — Owner: editor chrome after the second try-out** (relayed by the orchestrator). Owner's words include "i will prefer icons rather than names, and the pop over shows the name of the tool the shortcut and a brief small description", "the nudge buttons … should only show if i expand them", "tools panel hides and shows the tool panel that should be on the tool panel and when hidden remain on the left side", "the turn right flip panel button can be done better … perhaps on the drawer". 10D §What the photographer sees: icon toolbar with name/key/description hover cards, folded arrow pad, hide button in the Tools panel header and a left rail when hidden, Rotate and flip drawer, Turn icon beside Shape, grid icon for the straightening grid, frame meter without a button. Decision R39. Orchestrator readings (reversible): the frame meter is a measuring aid and loses its button (the owner asked what it is); the arrow pad and the drawer start closed each time the editor opens. Owner decision, header stays RATIFIED.


30. **2026-11-05 — Owner: editor polish and undo/redo after the third try-out** (relayed by the orchestrator). Owner's words include "the line in the middle is hard to see", "i can't tell if it is active or not", "it looks like to many buttons now, i imagine something more simple in terms of drop down", and "the ability to do comand z and move back also to do comand shift z and go forward. this has to be reliably and save up to 25 actions", "not only on the editing panel but over all". 10D §What the photographer sees (split line, toggle states, dropdowns, undo in the editor); new 05A MC-30 with invariant M36; MC-5's single delete slot replaced; chapters 04 and 06 aligned. Decisions R40, R41. Orchestrator readings (reversible): one history per session kept while the app runs (cleared on quit); the list of actions that are and are not steps; a step that cannot apply is dropped with a notice; Undo/Redo icons in the header and the Edit menu. Owner decision, header stays RATIFIED; MECHANISMS_VERSION unchanged (no stored canonical form changes).


31. **2026-11-06 — Owner: direct camera adapter** (the owner's brief and the Sony Camera Lab handoff, Camera-Connection-Tests `221aa49`; relayed by the orchestrator). New chapter 10E (MC-31 camera sidecar, MC-32 FTP receiver, MC-33 capture ledger and target folder, MC-34 camera ownership guard), evidence `evidence/sony_camera_lab.md` with Session Manager's own Capture One checks V1–V6 (Capture One sees external files only in its current collection; Next Capture Adjustments do not apply, a script-applied style does; Capture One triggers `ptpcamerad`'s USB claim even with its Sony tethering off; an SDK session opened first keeps the camera; a release or cable re-plug while Capture One runs loses it within 28 ms; store-destination writes verified over USB). Chapters 10, 03, 05A MC-2, 10A MC-18, 10C MC-26, 11, 12, 13 and 14 (M37–M39) amended. Decisions R42–R45. Owner answers D1–D7 recorded in 10E; D8 the orchestrator's call; "Take the camera back" on click first (owner: "right when i see it working … we will move to automatic"). Owner decision, header stays RATIFIED.

32. **2026-11-06 — Mechanism inventory of 10E** (pipeline-inventory, Opus; grounded in the lab code at Camera-Connection-Tests
    `221aa49` and this repo at `53d6ea4`). Inventory additions written into 10E under MC-31…MC-34, §Port changes, §Capture One
    alongside, §Camera support packages and §Hardware acceptance; pointers in 05A MC-2 (rule 0) and MC-4, 05B MC-22, 10A MC-18,
    10C MC-26, 11, 12; M37–M39 reworded and M40 added in chapter 14. Gaps and contradictions resolved by contract, pending owner
    ratification as a list (no change to what the photographer gets): R-I18 one camera runtime per process, outside the settings
    rebuild; R-I19 the sidecar's connect sequence (Save-to-PC folder, persistent store-destination write, never *Host PC* alone);
    R-I20 Save-to-PC staging per sidecar run and completion; R-I21 Keychain item names, credentials over stdin, redaction; R-I22 busy
    reasons (Auto FTP never defers a switch); R-I23 the closed list of reasons a USB session may close, idle-sleep assertion,
    `usb.initial` and `usb-retry`; R-I24 the receiver's virtual namespace, upload lifecycle, peers and capture root without Capture
    One; R-I25 ledger tables; R-I26 file key (camera suffixes stripped by the adapter only) and association order with a 300 s FIFO
    window; R-I27 global duplicate identity, one ledger worker, no-replace rename; R-I28 target folder at rename time, folder-key
    freeze at a delivery, MC-2 rule 0 and `captured_at`; R-I29 watchdog counts, reconnect on Wi-Fi only; R-I30 naming the holder;
    R-I31 start order; R-I32 the editor-launch gate and "Take the camera back" step by step; R-I33 camera and session activation
    independent of Capture One's health; R-I35 waiting for the variant before render and style; R-I36 support-package manifest,
    location, signing, spike S1–S6; R-I37 camera status values and precedence; R-I38 M37–M40 wording. (R-I34 is not used: the
    current-collection rule went to card CI1.) Open owner card CI1 (Capture One's browser on arrival), chapter 15. Header stays
    RATIFIED: no material semantic change.

33. **2026-11-06 — Owner: card CI1 answered "always jump"** (relayed by the orchestrator). Capture One's browser switches to the session's folder on every arrival, even while the photographer works in Capture One. 10E §Capture One alongside and 10A MC-18 step 6 updated; R46. Header stays RATIFIED.

34. **2026-11-06 — Orchestrator wording folds from the P42 projection** (no semantic change): MC-31 §Secrets now defers to R-I21 (credentials on the sidecar's first stdin line, not its environment); R-I37 states that `Connecting` needs the sidecar to report a connecting attempt, otherwise `Disconnected`. Header stays RATIFIED.

35. **2026-11-06 — Owner: inventory ratify items R-I18…R-I38 approved as a list** ("perfect, all approved"), after the orchestrator's plain-language summary of all twenty (R-I34 unused: it became card CI1, answered as R46). Entry 32's items are now ratified. Header stays RATIFIED.

36. **2026-11-06 — Orchestrator clarification of R-I26 from the P44 projection (card 1 answered from the book, flow §6).** Association rule (1) now needs the same `file_key` **and** the same SHA-256 (an identical copy). Rule (2) (same stem, RAW + JPEG) applies only within 300 s. Reason: the camera's counter wraps after DSC09999, and the rule as written would file a new photo into an old session. That contradicts §What the photographer gets ("every photo lands in the session that was open when the shutter fired") and R-I27 (same key, other bytes → a new photo). Header stays RATIFIED.

37. **2026-11-08 — Orchestrator rulings from the P56 implement block (flow §6; answered from the owner's instruction "the whole point is that connection and disconnection happen fast and the flow is not disturbed", after P52 run 1).** MC-31 §Transport policy amended: (1) on a USB arrival with the camera not busy, the helper closes the Wi-Fi session at once, attempts USB within 300 ms and repeats at ≤ 500 ms through the 2.5 s settle window; if no USB session results, Wi-Fi reconnects at once; these attempts do not count toward the three-failure fallback. (2) After the three-failure fallback to Wi-Fi, USB is retried only when the cable's holder releases it (IOKit `UsbExclusiveOwner` cleared or ours), detected within 1 s, never by blind probing that would drop a working Wi-Fi session. The P52 A10 expectation ("the camera re-sends the pending shot") is superseded by the measurement in evidence §Acceptance (P52): the A7R V stops FTP after a failed upload until FTP Function is toggled. Header stays RATIFIED; the owner may override either ruling.

38. **2026-11-09 — Owner decisions after P52 run 2.** (1) MC-34 §Take it back: when the camera is plugged in and its cable is held while Capture One runs, the app takes the cable back **automatically, once per plug-in** (same preconditions; it waits and says why when one blocks), behind the setting `camera.auto_take_back` (default on). This replaces "Capture One is quit only on the photographer's click" for this case. Reason (evidence §Acceptance run 2): `ptpcamerad`, started with Capture One, claims the cable at enumeration; the SDK cannot win that race; no Capture One setting stops it. (2) Grabbing the cable directly at plug-in or an own PTP connection is deferred. Header stays RATIFIED.

39. **2026-11-09 — Owner: faster previews (P52 F11).** 10D §What the photographer gets: Compose is also available on the camera's original preview (10A MC-20) when it matches the camera frame (same aspect within 1 %, ≥ 1600 px long edge; the A7R V embeds a full-resolution 6240 × 4160 JPEG), with the line "Camera preview — Capture One's version is on its way"; when Capture One's adjusted preview arrives the image swaps and the composition (fractions) stays. Exports and masters still come only from Capture One (MC-28). The preview render queue serves the focused photo first, then the newest arrivals, then the rest. Header stays RATIFIED.

40. **2026-11-09 — Owner: storage and retention (chapter 10F, DRAFT, not planned).** Duplicates deleted 24 h after verification (amends MC-33); Capture One preview exports deleted once copied; **N days (setting, default 4) after an item is published and clean, its local files including RAWs are deleted and Shopify serves as the archive (amends H2 for that case)**; a minimum-free-space setting with a removal order; recipes checked and repaired before each render batch (the 2026-11-08 TIFF previews came from a hand-changed recipe). Q1–Q5 open. H2 and MC-33 keep their current text until 10F is ratified.

41. **2026-11-09 — Owner ratifies chapter 10F** ("for the 5 questions the suggestions you make are correct"): Q1 all photos of a published item are cleared; Q2 a per-item "keep files" pin; Q3 nothing image-dependent is editable after clearing; Q4 cleared files stay as missing entries in Capture One; Q5 default minimum free space 20 GB. H2 and MC-33 are amended as 10F states; M41 added. Not planned yet (owner: work-Mac installation first).

42. **2026-11-09 — Owner: item type from Shopify's product Type (temporary).** 05 `items.item_type` (U/I) is sourced from the Shopify product's **Type** (`productType`) for found items; a new item picks its type from the shop's existing product Types (free text without an integration, H6); publish writes it back to `productType` (H1). Customize-fields presets are per item type; the preset saved under the empty type becomes the default for types without their own. Later the source moves to the `beyo.item_type` metaobject (owner). Needed before the work-Mac installation.

43. **2026-11-09 — Owner: files deleted outside the app never break the flow** ("if I manually delete a picture this shouldn't break the flow of the app"). A photo whose file is gone is marked missing: it leaves the session views, no render is requested for it, it never blocks a take-back, Health counts it; it returns if the file returns. Also from the evening check: Capture One is never launched while a take-back is in progress; a dialog blocking the quit and a camera service holding the cable with no app are named in the header.

44. **2026-11-10 — Owner, after the first real publish:** (1) photos not yet reviewed are published as main photos, after the approved main photos, in position order (07A `frozen_main`); `ready` (07) no longer needs every photo reviewed; (2) a publish can be cancelled (stops at the next safe point; status `Publish cancelled`; nothing on Shopify deleted by the cancel; the next publish reuses what was uploaded); (3) the publish progress explains its step and errors; uploads are batched and parallel, and files are looked up by name, not by listing the shop; (4) Settings → Style is a drop-down of Capture One's styles.

45. **2026-11-11 — Owner, after the first work-site publish** ("why it takes so long to apply the style", "jobs should queue instead of failing", "the shopify product that is created has inventory track off but I want to be able to select an inventory location and the quantity, I should be able to save the preset I selected"). (1) 10A: right after the app files a photo into an item's folder, it asks Capture One to synchronise that folder's collection (`synchronizing = true`, sdef §collection) instead of waiting for Capture One's own folder check (measured 17–80 s on 2026-11-11). (2) 07/07A: a publish or export whose render meets "Capture One has not seen this photo yet", Capture One busy, or Capture One closed **waits** (state `Waiting for Capture One…`, retried with back-off) instead of failing; it fails only after 10 minutes, with the reason in plain words. Publish may be pressed before previews are ready; it queues. (3) 07A/09: a product **created** by the app has inventory tracking on and the chosen quantity at the chosen Shopify location; the form shows Location (the shop's locations) and Quantity; the last choice is remembered as the default for the next new item (editable in Settings → Shopify). Products found in the shop and later publishes never change tracking or quantities (Shopify owns stock after creation: sales change it). Header stays RATIFIED.

46. **2026-11-11 — Owner, after the first day at work** (answers to the orchestrator's four questions in brackets). (1) 04/sessions list: sessions stay in **creation order, newest created on top**; opening, editing or publishing a session never moves it (time groups and the time filter use the creation time too). (2) 06 Classification: an arriving photo is **approved on arrival** (`main`, at the end of Main, arrival order), so it can be reordered at once and a well-shot session can be published without reviewing. The photographer marks **Condition** or **Deny** instead. `unreviewed` is no longer given to new arrivals [photos already `unreviewed` stay as they are]. **Deny** hides the photo and **deletes its RAW** [owner chose deletion]: amends 01 H2 for denied photos. The RAW is deleted only after the undo window closes (when the session is next published, or 24 h after the deny, whichever comes first); undo restores it until then; a denied photo is never published. Mirror tags unchanged (`main` green on arrival, `deleted` red). (3) 08 Values: the form of an item linked to a Shopify product **shows the product's current values** (title, description, type, tags, every metafield the form shows) instead of blank fields; local edits win over what was read. (4) 08 Groups and presets: every customizable field has a **default value** and a list of **saved values** to pick from, **per item type** [owner choice]; defaults fill empty fields of new items and of linked items whose Shopify value is empty. (5) The form never chooses a product type the photographer did not choose. (6) Batch publish forms open on the step the photographer was on in the previous form of the batch. Header stays RATIFIED.

47. **2026-11-11 — Owner: faster export and publish** (after a 9-item batch where two items failed `export failed: timeout` on the P64 build; retry worked). Answers to the orchestrator's two questions: (1) 10D MC-28: the full-size master is an **8-bit** TIFF (was 16-bit; halves what Capture One writes: ≈ 180 MB instead of ≈ 360 MB per a7R V photo); the published JPEG is still one-to-one from the master and compressed once. (2) Masters are **prepared in the background when a session is Ready** and Capture One is idle (low priority, paused during captures, take-backs, preview renders and publishes of other sessions); they are deleted after publish by the 10F rules; a later adjustment or composition change invalidates them as today. Also: a batch publish overlaps steps (Capture One renders the next item's masters while the current item uploads), and every publish logs per-step times. Header stays RATIFIED.

A1. **2026-11-12 — Owner: research agent ratified (branch `feature/agentic`).** New chapter 08A (the first of three assistants: product facts), evidence `research_providers.md`, measurements MA1–MA9 in chapter 14, chapter 08 §AI agent points to 08A. Engine in-app and service-ready, no Temporal/Python/PostgreSQL; OpenAI as first model provider with a model and effort per stage; unchecked suggestions publish at medium and high only; trusted sites are a user setting with priorities; research record, export and replay tool. The shaping history is 08A's own changelog. (Lettered entry: the numbered entries continue on `main`.)

A2. **2026-11-16 — Owner: shooting templates ratified (branch `feature/agentic`).** New chapter 10G: per item type a template of containers (final order), a shooting order and per-container framing presets; photos fill containers by shutter order on arrival; Condition after the last container; manual changes win with a warning and "Back to template"; retake deletes the old photo; Deny leaves the container empty with "Shift the rest back"; Settings becomes a sectioned drawer. Mechanical, no model. (Lettered entry: the numbered entries continue on `main`.)

A3. **2026-11-16 — Owner: shooting guidance in the session (10G §5).** Empty containers show their example and "Shot n"; the container of the next shot is outlined; the template window switches between final order and shooting order. Display only; placement unchanged. (Owner: "i like the improvement you suggest, let's implement it".)

A4. **2026-11-16 — Owner corrections after the first look at templates.** Settings: the unselected sections are readable; every drop-down in the app is the app's own styled drop-down (shadcn Select) instead of the browser's. Template editor (10G §3): a dashed add tile, an × to remove in the card's corner, a name and description per container (shown in the session and the Template window), the crop and result previews of the presets, no move buttons, and the shooting order set by dragging. Camera (10E): the FTP password is **four digits** (was 16 characters; owner: "too complex to write on the camera"); the receiver stays on the local network only. Joined into `main` directly (owner: "these corrections go directly merged to main").

A5. **2026-11-16 (evening) — Owner corrections after testing templates.** A deleted photo's empty container is **refilled by the next shot** (10G §6 rule 4; several empty containers fill in shooting order; the "Next" outline points there). The Template window shows the containers two per row like a web shop (10G §5). Template editor: every card the same height (the example fits a fixed area), nothing spills out of a card. The item type field is the app's own drop-down list (the browser's suggestion list was white on white and gave no sign it opened); in Settings it marks the types that already have a template ("6 containers"), replacing the unexplained "Armchairs · 6 containers" button.

A6. **2026-11-17 — Owner corrections after the first research runs.** Source links open in the default browser (the app's window ignored them). "n to review" opens the list of fields to review; choosing one goes to the field and opens its suggestion (08A §7, amended). The sessions list and the session header show each session's research state (researching, waiting with its place in line, n to review, done, stopped, or why it cannot run). A queued run says it is waiting and behind which session; research still runs one session at a time (08B MC-A6 unchanged). Deferred by the owner: researching several sessions at once and OpenAI's cheaper "Flex" tier.

A7. **2026-11-17 — Owner corrections: research export and the steps' looks.** Export research stopped with "…images/<sha>.jpg contains a key": the check read the compressed pixels of a photo as text and found a short stored value in them by chance. A JPEG is now checked up to its pixel data (its header, where any text lives), other files whole; the camera's access user name, a name and not a secret, is no longer among the values looked for (08B MC-A9, MC-A11, amended). In the research panel each kind of step has its own icon, label and rail: looking and searching quiet, reading and comparing in the AI colour, a conclusion standing out most, a page that could not be read a muted warning ("Could not read"; 08A §7, amended).

A8. **2026-11-18 — Owner correction: research export, again.** With A7 installed, Export research still stopped with "…images/<sha>.jpg contains a key": listing images are kept as the shop sent them (PNG, WebP, GIF, AVIF under a `.jpg` name), and only JPEGs were trimmed. Every recognised image format is now checked only where text can live; anything else is still checked whole. A choice that matches no run says "No research runs match this choice." instead of writing an empty zip (08B MC-A11, amended).

A10. **2026-11-18 — Late photos, sessions that reopen, and what the logs say.** (1) Photos the camera's FTP receiver had filed were given up when the background loop was held up by Capture One work for more than 120 s, then added at the next launch to sessions already published, which reopened them. Arrival now runs on its own thread, Capture One's follow-up for a new photo is queued to the background loop, the stability limit counts looks at the file instead of wall time, and dropped watcher events start a scan (05A MC-2). (2) A published session that changed says "Changed after publishing" with the reasons and offers "Keep as published" (late photos denied, one undo step; 07 "Dirty"). (3) Opening sessions one after another could leave Capture One, and the capture target, on a session no longer shown, because overlapping switch requests applied in the order they finished; they now apply in the order asked and a request a newer one overtook is skipped. (4) New log lines: per photo the arrival timeline (filed → photo → preview), every Capture One switch step with its result and time, and every photo moved or copied (which also says that moving does not move the RAW or tell Capture One: the known gap "moving photos to a product does not create a Capture One collection for them" is not fixed yet).

A9. **2026-11-18 — Owner corrections from the work Mac's research records.** (1) Export research still stopped on the same image after A8: it was a real JPEG with a 42 KB header, and the cause was the camera's short FTP code on research's secret list; research now looks only for the secrets it can hold (the research keys, the shop's client secret), which also ends the mangling of records (one search result had been redacted) (08B MC-A9, MC-A11). (2) The connected shop's own product pages are read through Shopify's API instead of its web pages, which answered 21 of 23 reads with "too many requests" and cost a minute each. (3) A site that blocks automated reading (403, or 429 without a short wait) is asked once per run; its other pages are skipped at once, saying so. (4) The same product in several languages or collections is read once; collection and category pages are not read (08B MC-A3, 08A §6).

A11. **2026-11-18 — Owner correction: one place to check AI suggestions.** Reaching each suggestion (the "n to review" list, the field, its AI pill) took too long. The form now has a fourth tab, AI review, listing every field with an unchecked suggestion: its step, the previous value, the suggestion with confidence, reason and sources, and the popover's own choices (Looks right / Clear, or a conflict's Keep previous / Use suggestion / Edit), so the rules and the undo are unchanged. The tab shows the count, stays visible at zero and says why it is empty; the form's "n to review" chip and the session header's "AI · n to review" open the form on it (08A §7, amended).

A12. **2026-11-18 — Owner correction: search this photo.** When a prediction was off, the owner closed the form, took a screenshot, dropped it into Google, compared, and typed facts back. A photo of the form's media preview now opens a **Search this photo** window: the photo pinned on the left (another photo of the item can be pinned, which searches again), a browser on the right with Google Lens results for it (Back, Forward, Reload, Results, the site and title), **Save page** keeping pages as tabs per item, and a bar under the browser for selected text: **Put in** a field (the person's own edit, one undo step) or **Ask AI…** with an instruction, whose answers are AI suggestions (medium, the page as source) in the AI review tab and never applied. When Google cannot be used the window lists the matches research already found, or runs one visual search of the photo. Each AI or search call is a `browse` run: recorded, costed, inside the day limit and in the export, but never the session's research run. The web pages are untrusted: they cannot call the app, load only public web pages, and only the person's clicks in the app's own page write a field or call the model (08A §7, amended).

A13. **2026-11-18 — Owner corrections: search window from the first live test.** The window opened, but Google looked old, after the cookie choice it showed an empty Google page instead of results, and the toolbar (Back, Forward, Results, Save page, saved tabs) was hidden. Three causes, each confirmed:

1. **The browser identity.** The page browser now presents itself as desktop Safari. WebKit's default identity made Google serve its basic pages, so the photo's results came back as an empty search page.
2. **The photo was lost after the cookie choice.** On a fresh profile, Google's cookie page swallowed the first post, and A12 never posted again. The photo is now posted once more when the route lands on a Google page that is not the results, at most twice per search, then the window shows its own list.
3. **The toolbar was hidden.** On macOS the window's content runs under the 32 pt title bar, so both views sat 32 pt too high. The app's page is now laid out below the title bar on open and on every resize, and the browser is placed inside it, also when a bar above it appears or goes.

Logs now name every page loaded (host and kind only) and every repeated post (docs/guide/02, 05).

A14. **2026-11-18 — Owner corrections: the search window's selection bar, and links that open new windows.** With the window working, two corrections:

1. **The selection bar is restyled** ("doesn't look that nice or modern"). It is now one floating, rounded bar under the page: the selection as a quoted one-line snippet (whole text on hover), Manufacturer / Designer buttons, "Other field…" as a searchable list inside the bar (arrow keys, Return, Esc), "Ask AI" as a prompt inside the bar (Return sends, Esc cancels) that reports "*n* suggestions sent to AI review" with "Open AI review" (the main window comes forward on that item's AI review tab), and ✕ to dismiss. With nothing selected a hint pill shows. "Put in" is confirmed by a toast with Undo (the field gets back what it held). The toolbar and the saved pages got the same style. Nothing is drawn over the page itself, because the page is a native view on top of the app's page: lists and prompts stay inside the bar, and toasts sit over the photo panel.
2. **Links in Google's AI Overview did nothing.** Probed in WebKit (wry 0.57.1): a page that opens an empty window and then sets its address (`window.open('')`, then `w.location = …`) reaches the app as an empty address, which Tauri drops before the app sees it. An `about:blank` window would have loaded a blank page over the results. That this is what Google's AI Overview does is likely but not confirmed live (Google showed its "unusual traffic" page to the dev Mac). Every frame of the page browser now runs a small script that opens any link, form or `window.open` aimed at a new window in the same browser area, http/https only. It reads nothing, calls nothing in the app, and the navigation check still decides. Refused navigations and new-window requests are logged by host with a reason (docs/guide/02, 03, 04, 05).

A15. **2026-11-18 — Owner correction: the selection is kept when leaving Other field or Ask AI.** Going back from "Other field…" (arrow or Esc) or from the Ask AI prompt without acting lost the selection and its actions. Cause: the window asks the page every 0.7 s what is selected, and replaced the selection taken with "nothing" as soon as the page reported none. The page reports none once the owner clicks into the app's own bar, because its selection goes when focus leaves it. While the field list or prompt was open the bar worked on its own copy, so the loss only showed on the way back. Now an empty reading never drops the selection taken. It goes only when the owner selects something else, selects too much (the reason shows), presses ✕, or the page changes to another address (a #fragment change is the same page). "Put in" and an AI answer keep it for a second action. After ✕, the same text shows again only once it has been deselected and selected anew (docs/guide/01, 04).

A16. **2026-11-18 — Owner correction: session switches no longer wait on the whole library.** On the work Mac (1,593 photos) photos arrived late, some Capture One styles were not applied, and the photos taken for a newly created session (T1-181126) were filed into the previous one (T4-181126) while Capture One was left "loading". Causes, each confirmed:

1. **A switch looked at every photo of the library.** Its last step checked every photo's file and retried every photo's preview, one by one, inside the switch: 4–8 minutes on the work Mac (`collection_switched ms=475730`, `ms=267849`; the 15:49 UTC switch never ended). The step now only notes the switch; the follow-up runs on the background loop and looks only at the opened session's photos. Per-switch cost: the Capture One calls plus one query and one preview pass over that session's photos, never the library (a test with 1,600 photos elsewhere: 81 s before, under 3 s after, no call about another session's photo).
2. **The next session waited behind it.** A10 runs switches one at a time, so T1's switch, and with it where the camera's photos are filed, waited behind T4's unfinished one. Opening a session now sets where photos are filed at once, before Capture One is asked for anything (a short claim in the same order; a request already overtaken is skipped), and a late older switch cannot take it back.
3. **Styles were skipped by a race A10 introduced.** A10 queued the arrival's Capture One work for the background loop, including marking the photo for its style and waiting for Capture One to list it. The photo's preview job could run first and render it without the style; a rendered photo never gets its style later (rule unchanged). Those two marks are now made at once on the arrival thread; only Capture One's work waits for the loop.

Owner rule (2026-11-18): recurring work leaves out deleted photos and sessions that are published (unchanged since) or cleared; the library-wide missing-file check runs only at startup, window focus and Capture One reopening. Every preview-state read is one query (it was one per photo). A photo Capture One does not list is asked again after up to 10 minutes instead of every minute, and logged on the 1st, 2nd, 4th, 8th… attempt. New log lines: `arrival.target`, `capture_one.switch_follow_up`, `capture_one.style applied|skipped|failed` (docs/guide/02, 04, 05).

A17. **2026-11-18 — Owner correction: "Delete session" (Session Manager only).** The app had no way to delete a session. The owner needs one for test items (test-4-4: its Shopify product was deleted, so retention can never verify and clear it) and for mistakes. Scope, in the owner's words: only Session Manager; nothing is asked of or changed in Shopify. The session's ⋯ menu (sessions list row, session header) has "Delete session…"; a dialog names the SKU, the photo count and whether it was published, and says that the files go to the macOS Trash (recoverable there), that the app cannot undo it, and that Shopify is not changed. Amends H2 a third time: the owner's delete moves the session's RAWs to the **Trash** (never a permanent delete).
- **Files to the Trash:** every RAW only this session uses and inside the capture root, Capture One's files named after it, its verified duplicates, the app's preview, master and output folders of it, the item's export folder, and the session's capture folder once only Capture One's data is left in it. A RAW another session uses (a copy) stays.
- **Rows:** in one transaction, every row naming the session, its item, its photos, its RAWs (when no other session uses them), its publish items and its research runs (with their steps, calls, outbound and stage-output rows), and their jobs. The capture ledger keeps its rows without a session (a late file goes to Unassigned). The session's undo history, and any move or copy step naming it, goes too.
- **Guards:** the Unassigned session cannot be deleted; refused while an export, a publish step or a research run of the session is running. When photos are filed into it, the filing target goes to Unassigned before the first file moves, and Capture One is asked to leave it (the ordered switch of A10/A16); Capture One not running never stops the delete. A file that cannot be moved keeps the session and says so.
- **Capture One's collection** for the folder is not removed: the dictionary's only way is the generic `delete` on a session-folder collection, not documented for folders on disk, so the app does not risk it.
- Log line `session.deleted session= sku= photos= files_trashed= files_missing= folder_trashed= research_runs= capture_one_result=` (docs/guide/01, 02, 03, 04, 05).

A18. **2026-11-18 — Owner corrections: search tabs, answers in words, zoom, and a clearer AI review.** After using the research tools for some time:

1. **Changing the pinned photo no longer searches.** Pinning another photo only changes the reference picture. "Search this photo" shows on the pinned photo on hover (as in the form's media preview) and opens a **new search tab**; the tabs already open keep their page, history and scroll. Tabs show the photo's thumbnail and the site they show, switch with a click and close with ✕ (the last one stays; at most 8). The main window's "Search this photo" on an open window pins that photo and opens a new tab. Saved pages open in the current tab.
2. **Ask AI answers in words.** "Answer me" (the default) gives a short plain answer about the selected text (e.g. which name in a table is the designer and which the manufacturer) and offers each value it found as a "Put '<value>' in <Field>" chip; nothing is written until a chip is tapped, and then exactly like "Put in" (with Undo). The answer stays on its tab until dismissed. "Fill fields" keeps A12's suggestions to AI review. Both are recorded browse calls (day limit, export). Selections up to 6,000 characters (was 2,000) so a whole table fits.
3. **Zoom.** The page zooms per tab (−/%/+ in the toolbar, ⌘+ ⌘− ⌘0 from the View menu, 50–300 %). The pinned photo zooms with the wheel or a pinch, a click zooms in on the spot, drag pans, double-click fits it again.
4. **AI review.** Every suggestion over a value the field held offers **Keep previous** (not only conflicts; the backend's Keep previous now takes any suggestion with a prior, same feedback row and undo); one over an empty field offers **Dismiss**. Rows were redesigned: field and step quietly, "before → suggested" in one line, confidence as a soft word badge, the reason as one readable sentence with More, sources as small site chips, and three clear buttons: Accept, Keep previous / Dismiss, Edit (docs/guide/01, 02, 03, 04).

A19. **2026-11-18 — Owner decisions: research runs by itself once per session, "Arranged — next" goes home, and a review queue.** The owner's day: create a session, shoot, let the template arrange, correct a few classifications, press "Arranged — next" and type the next session; later sit down and review what research found, session by session, retouching where needed.

1. **Research starts by itself at most once per session.** "Arranged — next" queues research only when the session has no run that is queued, running, finished or stopped at the cost limit. A failed or cancelled run produced nothing, so arranging again retries it. After that the only way to research again is **Run again** in Details. Photo changes after a run (a new main or condition photo, another cover, a new framing), also during a run, no longer queue a follow-up (supersedes MC-A7's follow-up trigger set; a queued automatic run left by an older build still runs and still reuses unchanged units).
2. **"Arranged — next" goes home.** It marks the session arranged and opens the Sessions page with the New session dialog open and its field focused, ready to type the next SKU. It no longer jumps to the next unarranged session; the "Every session is arranged" toast is gone (a short "Arranged" toast says whether research was queued).
3. **Review queue.**
   - **Sessions page:** "To review · *n*" at the top lists only the sessions whose research finished with AI suggestions waiting (the research pill's "*n* to review" state), in the list's order. Opening one goes to its session page (to retouch), not straight into Details.
   - **Details** (button or D) opens on the AI review tab when suggestions are waiting.
   - **AI review tab:** once the last suggestion is checked it shows "All checked ✓" with **Next to review →**, which closes Details and opens the next waiting session's page; with none left it goes to the Sessions page with "All reviewed".
   - **Session header:** an arranged session whose research has finished (suggestions waiting or all checked) shows **Next to review (*n* left)** in place of "Arranged — next" while other sessions wait; every other session keeps "Arranged — next".
   - **Order:** the sessions list's order (newest created first), starting after the current session and wrapping round.

A20. **2026-11-18 — Owner correction: opening a session no longer slows down with every session.** On the work Mac (A16 build) a switch still took 15–22 s, nearly all of it Capture One answering scripts. Causes, each confirmed in the logs and measured on the dev Mac's Capture One 16.8.6: the document description script listed every collection (three Apple events each, one folder collection per session: ~3.5 s a call with ~125 collections) and a switch asked it four times; finding a session's collection asked each collection in turn; and every switch re-checked both recipes (~13 property reads each) and all three hooks.
- The description reads a fixed handful of properties; the collection list it carried was unused.
- A session's collection is found with one bulk read of every collection's folder (dev Mac, 16 collections: 10 ms against 434 ms), also for the arrival's folder sync and the photo lookups.
- Recipes and hooks are verified once per Capture One process; again after Capture One restarts, after any failed ensure, after a failed render-time recipe repair, and on the Health page's check. A recipe changed by hand is still repaired before each render batch (10F §5). The capture style is still checked on every switch.
- Log lines `capture_one.switch_scripts reason= to= scripts=` and `capture_one.ensure checks=cached|full instance=` (docs/guide/02, 04, 05).

A21. **2026-11-18 — Owner correction: photo files follow their session.** On the work Mac eight photos taken for T1-181126 were filed into T4-181126's folder (the A16 switch bug) and then moved to T1 in the app. Moving only re-pointed the rows: the RAWs and Capture One's files of them stayed in T4's folder, T1's folder stayed empty, and T1's publish waited forever ("waiting for Capture One"). Why Capture One could not find them: it answers a lookup through the collection it shows (T1, empty) or through its own list of the photo's folder, which it refreshes only when it looks at that folder (measured read-only on the dev Mac: two folders listed 0 of 13 and 2 of 9 of their files). Amends H2 a fourth time.
- **Move:** moving photos to another session (and the undo or redo of a move, and Assign product) moves each RAW with Capture One's files of it (`CaptureOne/Settings*/<name>.*`: settings and masks, so the edits travel; `CaptureOne/Cache/**/<name>.*`: proxies and thumbnails) into the target session's capture folder in Capture One's layout, and the app's caches of it; the rows follow in one transaction. Capture One is asked to re-read both folders.
- **Never:** over an existing file (the photo stays and is named); a RAW another session also uses (a copy); a RAW outside the capture root; a RAW Capture One is rendering (later). Refused up front, in the owner's words, while an export or a publish step of a session involved runs. If a file cannot move, what moved of that photo is moved back.
- **Rescue:** the session's ⋯ menu shows **Fix photo files location (N)** when photos of the session have their files in another folder (moved before this change); it moves them the same way. Chosen over an automatic startup repair: the owner sees what moves and when.
- **Stale lists:** a photo on disk that Capture One does not list in its folder makes the app ask Capture One to re-read that folder, at most once a minute per folder.
- Log lines `photos.files_moved`, `photos.files_not_moved`, `photos.files_fixed`, `capture_one.folder_synced reason=photo_files_moved|photo_not_listed`; `photos.moved … files=follow|shared` (docs/guide/01, 02, 03, 04, 05).

A22. **2026-11-19 — Owner ratified chapter 08C: the publication writer.** The writer drafts the title, description, Detailed-condition-led condition report description and Shopify's search title and description. It works from the fields the owner lets it see, from research's new product note (photos are analysed once, in research stages 1 and 6, and the owner's review decisions override the note) and from the titles of matching listings. It runs when the owner moves on with "Next to review", or from "Write texts". It has a deterministic post-check (banned words and patterns, lengths, damage coverage, names only from fields), a typed safeguard (off / note / ask per type, never re-raising what the owner decided), and settings for model, effort, instruction, visible fields, examples and language (English default, Swedish). Restoration becomes fixed choices. Learning from edits is deferred. Evidence: `evidence/publication_writer_research.md`.

A23. **2026-11-19 — Owner correction: the writer follows the company's listing rules.** The owner compared the writer's texts with the instruction the company has used with its listing assistant and asked to bring the parts that matter into the writer. Decisions: (1) "yes we follow that for now": the condition grade words (Excellent / Very Good / Good / Worn) are never written; Detailed condition only shapes the tone (reverses 08C §7's grade word first). (2) "we can leave the tick boxes", then "for now we use the damage details": Damage details and the Restoration notes are the normal restoration source; the "Restoration done" ticks stay as an optional extra (post-check rule 11 unchanged).
- **Switches** in Settings → Writer, all off by default: grade word first (post-check rule 8 only when on; when off the instruction forbids the grade words), stability sentence (rule 9 only when on), the built-in title formula, the dimensions line (format sent only when on). The shipped English and Swedish instructions no longer ask for these or for short paragraphs, and the search description no longer names the grade.
- **The item:** the writer input carries `item {type, sku}` (the item type, which follows Shopify's product type, and the SKU), so an instruction can tell chairs from other furniture.
- **Export… / Import…** of the writer settings (only the `writer` slot; earlier instruction versions are left out of the file and kept on import). The company's own instruction, examples, banned words and lengths are delivered as an importable file outside the public repository.


A24. **2026-11-19 — Owner correction: Write texts always offers new texts, and the writer's state shows.** On the work Mac the owner pressed Write texts on an item reviewed earlier: the writer wrote every text, but the description ("the photographer cleared this field") and the condition report ("the photographer accepted this value") never reached AI review, and nothing said so. The owner also said "it is hard to tell if the publisher is working on it". Decision: "perfect, we can build that".
- **Deliberate requests** ("Write texts", an answer to a writer question) offer a new suggestion for each of the writer's texts even over a text the photographer accepted, typed or cleared. An accepted or typed text becomes the suggestion's previous value, kept whole (its accepted state, reason and sources), so **Keep previous** brings the field back exactly as it was; a cleared field has no value to keep, so the new text is a plain suggestion and **Clear** empties the field again. The same text as the current one is not offered. A deliberate request does not consult the texts dismissed before (the photographer is asking again). A deliberate request that finds an automatic run still queued makes that run deliberate.
- **Automatic requests** ("Next to review →", "All checked") keep MC-A1's refusals and never bring back a dismissed text.
- **Seeing the writer:** "Writing texts…" (or "Writing failed") beside the research pill on the session's row and in the session header; a toast when a run the app saw writing ends ("Texts ready for <SKU> — see AI review", or the failure); in AI review a failed run says why with **Try again**, and a finished run lists each text it did not offer and why ("Description was not rewritten: you cleared it. Press Write texts to get a new one."); the research panel's Writer block says "Last written: <time> · finished / failed / stopped".

A25. **2026-11-19 — Owner correction: Customize fields shows the fields again.** The owner: "there is no way for me to re-arange the ones that already exist, or to manage them, i think this is a ui bug". The "Add a field" box filled the whole step (the command primitive takes the full height), so the step's existing fields (reorder, default value, remove) sat below it out of sight. The box now takes its own height and lists the fields to add only while the owner types; picking one adds it at the end and empties the box.

A26. **2026-11-20 — Owner correction: the description is a plain text box.** The owner: "there is the button on the description field which says ai writes or write manually, i think this is outdated given the current publisher and agentic system flow". The switch predated the publication writer (08C): an empty description opened on "AI agent writes it", which hid the text box and said the agent was not available yet. The description is now a plain multi-line text box like the title, also when empty; the writer's texts arrive in AI review as for the other texts.

A27. **2026-11-20 — Owner: conversations with Pricing (new chapter 09B, RATIFIED).** The owner built Beyo Pricing, where a pricer prices products from what Session Manager published, with a chat per product; the owner asked for "a simple modern ui for this chat interface" in Session Manager: a header icon "like in social medias" for all unresolved conversations or new messages, the same icon inside a session private to that session, jumping to a conversation's session and coming back easily, and sound notifications the user can mute. Plan approved ("perfect, let's implement it") with three answers: the Pricing account is already tied to the real shop; opening a session from a conversation only visits it (the capture folder stays); Mac banners on by default. Chapter 09B: Settings → Pricing (key in the Keychain, store matched to the Shopify shop), event pull every 10 s (no webhooks: a desktop app has no public address), local copy and per-Mac read marks, header icon and panel, Pricing-style conversation view, answering with `clientMutationId` and photo attachments, Mark answered, visit-and-back jump, sound, toast and Mac banner. Milestones C1–C4.

A28. **2026-11-20 — Owner correction: test buttons answer, and the panel says why it cannot connect.** The owner, on the first live connection to Pricing: "on test connection nothing happens, also on the conversation drawer i only see the open settings connect pricing setting -> pricing"; then "with that fix we should also fix the test camera connection which also doesn't work properly". The Pricing key could see only the Dev store, so Session Manager rightly stayed off, but nothing on screen made that plain: pressing Test connection again showed the same words, and the panel always said "Connect Pricing in Settings → Pricing".
- **Settings → Pricing, Test connection** reads "Checking…" while it runs, then the connection in words (or why Pricing could not be reached) and "Checked at HH:MM:SS", on every press.
- **The conversations panel** says why it cannot show conversations, in the same words as Settings (09B §4), with **Open Settings → Pricing**, which opens Settings on the Pricing section; a refused key or a store that is gone shows its sentence with the same button. "Connect Pricing in Settings → Pricing" remains only when there is no key.
- **Settings → Camera, Test camera connection** said only what held the cable ("Nothing is holding the camera"), never whether a camera was there. It now says the camera's state first: "Connected: <name> on the cable / on Wi-Fi", "Connecting on the cable / on Wi-Fi…", what it is busy with, the action it needs, or "No camera found. Check that the camera is on and the cable is plugged in, or that Wi-Fi is set up."; then what holds or could take the camera, when that applies; then "Checked at HH:MM:SS". It reads "Checking…" while it runs. The check only reads: it never takes the cable from Capture One.

A29. **2026-11-20 — Owner correction: the publisher's titles, descriptions and condition reports.** The owner tested the writer on ten products (Ch1–Ch9-201126, Sto1-201126) and had the company's usual listing assistant review them: "it did not do a good job at all on titles and descriptions and the other fields also" (correction brief of 20 Nov, approved before/after texts per product). The texts followed the shipped default guide: plural titles with "Set of 4", country and years; names in capitals; three or four paragraphs; the Damage details pasted word for word ("Surface has been refinished, Completely restored, Reupholstered."); condition reports listing photo numbers and sizes; "| Beyo Vintage" after the search titles; one product (Ch1) kept its SKU as its title. Amends 08C §7 and §9.
- **Default guide** (English and Swedish) and its examples follow the owner's rules where they are not company-specific: singular titles without count, country, years or measurements; names in normal name case; one paragraph opening with provenance; veneer and solid parts; upholstery type and colour; restoration in natural words; "Sold as a set of N." for sets; no shapes; a one-or-two-sentence customer condition report without photo numbers or sizes; no shop name in the search title; ask when a field looks wrong. Default lengths: description 40–100 words, condition report at most 35 words. "Completely restored", "fully restored", "full restoration" and "the whole piece" (and Swedish "helt renoverad", "totalrenoverad") are banned words of the post-check.
- **One paragraph:** the writer's answer gives the description as one string, like the other texts.
- **SKU titles:** a title that is only the item's SKU counts as no title, so the writer offers one also from "Next to review"; adopting a shop product records its title as the shop's.
- **URL handle:** when the publisher changes a title and the handle is still the SKU's, the handle follows the new title with a 301 redirect from the old address (once more with `-<sku>` if taken); a customised handle is never changed.
- The company's own rules (category keywords, price tiers, "price per set", its approved examples) live in its importable writer settings file, outside the public repository.

A30. **2026-11-22 — Owner correction: the product is the conversation's container.** On a Mac without the session, three questions the pricer sent on Ch1-201126 ("title?", "why not ?", "yes") showed as three rows. The owner: "why is it that each new message from the pricing app on the same item on the session manager opens a new conversation?", then, after the explanation that each Pricing case is its own question: "the product is the container for the conversation, with in that container questions about the product and answers are made … on pricing app it behaves that way, so should session manager treat it like that". Amends 09B §6, §7 and §8.
- **List:** one row per product (its Pricing product id, else its SKU), with "N questions", the newest message over all its questions and an unread dot when any has news. Open = a product with any unresolved question; Resolved = all its questions resolved within 30 days.
- **Thread:** "Questions · SKU" with every question of the product oldest first, each with its own header line (topic, "· Blocking", status, Mark answered on an open one); resolved questions stay, quieter. Opening it marks every question read.
- **Composer:** one per thread. It answers the question the thread was opened on (the newest unresolved, or a notification's own), changed with **Answering** or by pressing Reply on any message.
- **Counts:** the header icon counts products with news; the sessions-list "N open" still counts unresolved questions.

A31. **2026-11-23 — Owner correction: the product thread is a plain chat.** Seeing A30's thread, the owner: "the chat interface seems to be adding things that i don't need or ask for … line divisions … status saying other answered … a button that says mark answered, and a drop down also for answering. conversations in chats are simple, i open i see who sent the time, and i answer, i want to reply to some specific text then i select that message and select reply (common industry practice in chat apps)." Amends 09B §7 and §8.
- **Thread:** one chat with every message of the product's questions in time order; no question dividers, topic or status lines, "· Blocking" or dimming. Each message shows who sent it and when.
- **Reply:** on the message itself, a small button on hover or keyboard focus, and a right-click menu; it shows the message above the box (✕ cancels) and sends to that message's question with the reply-to.
- **No Mark answered and no Answering choice** on screen. Without a reply-to, a reply goes to the newest unresolved question, unseen. The list rows stay as A30 made them.

A32. **2026-11-23 — Owner: archived and shop-only items seen, opened and edited like any session (new chapter 10H, RATIFIED).** The owner: archived items show a broken picture in the sessions list and a read-only page; "when i enter into the session i should have the same session interface as if the picture where still existing … the program should allow to download the picture from shopify to be edited and then i can publish those changes", with pages so a load does not spend Shopify points, and pictures on search results. Cards answered: products from the other Mac get the same flow; editing from the published JPEG is fine; 25 sessions per page.
- **Pictures:** every sessions-list row and search result shows the first main picture; remote ones come from Shopify's CDN at 320/160 px, fetched only for rows on screen, cached on the Mac; stale URLs refreshed in one batched query per page.
- **Pages:** the sessions list shows 25 per page with page numbers; search and filters apply before paging.
- **Workspace:** archived and shop-only items open in the normal workspace; details, the writer and details-only publish need nothing downloaded.
- **Bring back:** the first photo edit asks to download the item's published pictures at full size into its folder and Capture One; they become ordinary (JPEG) photos linked to their Shopify media, so a publish uploads only what changed and reorders the rest.
- **Points:** CDN downloads cost none; background GraphQL waits when the throttle budget is low.
- Amends 10F: the read-only rule and Q3 are reversed.

A33. **2026-11-23 — Owner: price labels from Pricing (new chapter 09C, RATIFIED).** The owner: Session Manager should "download the appropriate print templates for a single product / session or a selection of products", with a "labels" box in the sessions list whose search and filters go through the Pricing API (default: not printed), label generation from every session's ⋯ menu, and batch generation from the Labels box. Cards answered: products with no session here are shown as sessions (10H shop-only flow); default "has a price, not printed"; the PDF is saved in a Labels folder and opened in Preview; check list plus label preview before generating; the label marker everywhere; Select all N in the Labels box only; price-change resets update quietly; Pricing's template choice only.
- **Labels box:** a fourth box with the count of missing price tags; provider-backed filters, counts, search and Previous / Next pages of 25; rows matched to local sessions by shop product or SKU, else shown from Pricing and opened by adopt.
- **Generate:** check (grouped by template, outcomes, issues, preview image) → run with one idempotency key per chunk of 300 → done with one Download PDF per file → saved to `~/Documents/Session Manager Labels/` and opened in Preview.
- **Marker and events:** label state per product from Pricing's status call and its `label.printed` / `label.reset` events in the existing 10 s pull; kept only in memory, never as Session Manager's truth; no notifications.
- Extends 09B §4 (scopes `labels:read`, `labels:write`; Test connection reports labels) and §5 (label events no longer skipped).

A34. **2026-11-24 — Owner corrections after the first use of labels and of 10H (09C §14, amends 10H §6).** The owner, with screenshots: the check dialog's titles wrap word by word, warnings and the "not published / QR opens …" note crowd the rows, the footer is too low; "why doesn't it download and open the document ready for command + p?"; and a product from the other Mac, opened without changes, shows under Open.
- **Check dialog:** fixed height with a pinned footer and a scrolling list; one card per product (1:1 picture with the SKU under it, title on one line, price and outcome), warnings as pills under the card ("No price", "Can't generate: …", "Not published yet"), the QR address behind a "QR link" pill.
- **Generate:** saves and opens every PDF in Preview at once; the dialog closes with a toast when all went well and stays on the problems otherwise. No Download PDF step.
- **Shop-only items count as Published** in the sessions list (counts and filters), unless a publish of theirs is running, held, failed or cancelled.

**Devices track (D-series).** Entries for the measuring tape (10I) and live preview (10J), numbered D1… so they
cannot collide with the A-series written on `feature/agentic` in parallel.

D1. **2026-11-21 — Shaper: chapter 10I Measuring tape (COLLABORATING)** from the owner's draft
`messuring-tape-implementation/SESSION_MANAGER_REEKON_T1M_INTENTION.md` (kept as raw input). Conflicts resolved by the
shaper, each open to the owner: "measurement" is the ledger's word, so the chapter says tape, reading, measuring order,
step, run, draft; the run controller moves from the UI to a Rust service (H8, R24); readings are integer tenths of a
millimetre (MC-MT1) written in the field's MC-9 canonical form (MC-MT3), unit lengths declared by the commerce adapter
(H7); Confirm goes through `AttributeService::set_value`, provenance human, one undo step (MC-30); drafts in memory only;
Confirm never blocked; readings synced from the tape's memory never fill a step. Cards MT1 (the store has text and
number-plus-unit size fields), MT2 (unit and precision), MT3 (second press in the review pause), MT4 (keyboard mode
before the SDK), MT5 (adapter source vs REEKON's terms in a public repo).

D2. **2026-11-23 — Owner: 10I parked.** Implementation held until later; cards MT1–MT5 answered then.

D3. **2026-11-25 — Owner + shaper: chapter 10J Live preview (COLLABORATING, research phase).** Owner: live view is ruled
in now (reverses the 10E/13 deferral and the camera hard rule "live view is out of scope"); the app may change camera
settings live view requires (network/FTP untouched); one frame source in Rust with many viewers; USB and Wi-Fi; live
view sleeps while nobody looks; overlay style is a setting; the phone app is an installable PWA; the owner's working
loop is the acceptance story. Shaper: own frame channel from the sidecar, newest-frame hub, one WebSocket path for
desktop and phone, no re-encoding; spikes LV-S1…S4.

D4. **2026-11-26 — Owner answers and rebase.** Templates are 10G's shooting templates (published order + shooting
order; placement by shooting order keeps photos in the right container despite transfer delays; drag always allowed;
made in Settings → Shooting templates) — LV-D8; a capture moves to Next, retake and view what was taken — LV-D9; a
retake replaces, the old photo is denied — LV-D10; live-view settings not restored if they neither slow the workflow
nor change captures, live shooting becomes normal — LV-D11; any paired phone, several at once — LV-D12; image analysis
for machine guidance is a future H3 exception — LV-D13. Shaper: MC-LV3 retake aim (extends 10G: the next shutter after
Retake goes to the chosen container, bound at exposure) for owner confirmation; card LV3 (HTTPS for the PWA) stays open.
Both chapters were first committed as 10G/10H on a checkout 610 commits behind `origin/main` (where 10G/10H already
exist); renamed 10I/10J with per-chapter IDs (MC-MT*, M-MT*, MC-LV*, M-LV*). The 10E, 13 and guide texts are amended
at ratification, not before. Book header stays RATIFIED; 10I and 10J are outside it until ratified.

D5. **2026-11-25 — Lab: spikes LV-S1 and LV-S2 done** (camera lab Experiment 07, owner-attended; evidence
`sony_camera_lab.md` §Live view). a7R V live view works over USB (25 fps) and Wi-Fi (~20 fps), 1024×680 High /
640×424 Low, no dropped frames, glass → Mac ≈ 120 ms; no camera setting needed (LV-D2: list is empty); the SDK never
signals "no new frame", so the sidecar paces pulls and de-duplicates by frame number (MC-LV1); capture and Auto FTP
work during live view, FTP uploads dip Wi-Fi live view to ~13 fps, live view never slows uploads; a release without
half-press is silently not taken (Capture keeps 10E's half-press). **The camera keeps live view running whether or not
the app pulls**: wake is instant, pausing saves no camera battery (≈ 27–33 %/h over Wi-Fi) → LV-D5 annotated, MC-LV2
grace proposed 2 s, new card LV4 (what sleep means). Also recorded: one remote session per camera (another Mac's
Session Manager blocks with a generic 0x8202, C12) and an SSH fingerprint change after the owner changed the camera's
Access Authentication credentials (C13).

D6. **2026-11-25 — Owner: no sleep (card LV4 → c).** Since pausing does not rest the camera, live view sleep is not
built: frames are pulled whenever the camera session is up and sent only to viewers showing Live (LV-D5 revised,
MC-LV2 rewritten "Who receives frames", M-LV2 restated; "Waking camera…" and "Live paused" statuses dropped). Reopened
only if LV-S3 measures a real Mac or Wi-Fi cost. Spike LV-S4 prepared in the camera lab (Experiment 08: the same test
phone app over HTTP and HTTPS with a name-constrained lab test certificate, live camera frames over ws/wss).

D7. **2026-11-25 — Lab: spike LV-S4 done, LV-S3 phone half done** (Experiment 08, owner's iPhone, iOS 18.7; new
`evidence/live_preview_lab.md`). The installed phone app opens full screen on plain HTTP and HTTPS and shows the
camera's live view at 25 fps, ~15–30 ms behind the Mac (glass → phone ≈ 145 ms on USB); the screen wake lock and the
service worker exist only on HTTPS: on plain HTTP the phone locked at ~61 s and the stream closed; on HTTPS (one-time
trusted profile, a lab authority limited to local names and private addresses) it stayed on > 3 min. Mac cost ≈ 5–7 %
of a core each for pulling and serving. Card LV3 now carries the evidence; shaper recommends (a), own certificate with
a one-time profile at pairing.

D8. **2026-11-25 — Owner: card LV3 → plain HTTP.** No HTTPS and no certificate on phones: everything the phone app
needs works on HTTP; screen lock is the user's phone setting; offline start has no use without the Mac. Consequences
recorded in 10J: the phone app uses no secure-context API; Android full screen unverified (fallback: public
certificate on an owner domain, no phone install); studio Wi-Fi traffic unencrypted, accepted, per-phone revocable
tokens. The video keep-awake test (option A) was not run, not needed after this answer.

D9. **2026-11-25 — Lab + owner: Android test; card LV3 → HTTPS on an owner domain.** Android Chrome over plain HTTP
refuses to install the app ("This app cannot be installed"; shortcuts open in Chrome); a Full screen button works on
HTTP but needs a tap each open. The owner chose the full app experience: HTTPS with a publicly trusted certificate for
a subdomain of the owner's Route 53 domain pointing at the Mac's local address, renewed and re-pointed automatically
by Session Manager; nothing installed on phones (supersedes D8). Lab Experiment 09 to verify (router, iPhone, Android).

D10. **2026-11-25 — Owner: card LV3 final → plain HTTP now, HTTPS later** (supersedes D9's route as the build target;
D9's owner-domain design is kept as the later HTTPS plan, lab Experiment 09 deferred). iPhone: Add to Home Screen
once, full screen; Android: shortcut + Full screen button (added to the phone UI). The phone app uses no
secure-context API and the server keeps the same paths so HTTPS can be added without changing the phone app.

D11. **2026-11-25 — Owner: MC-LV3 retake aim confirmed** as written (any button; bound at exposure; Next returns; one
photo; cancel / session switch / relaunch clears). 10J has no open owner card; left before READY_FOR_RATIFICATION:
spike LV-S3's desktop-webview half. At ratification the 10E, 13 and guide texts are amended (live view in scope,
camera-settings rule narrowed, phone remote moves up).

D12. **2026-11-26 — Owner: focus from the live picture (LV-D14).** Clicking or tapping the live view (Mac or phone)
moves the camera's focus point there and focuses, as Imaging Edge Desktop does; needed because moving the furniture
moves the subject. Non-goals narrowed; new spike LV-S5 (lab Experiment 10). Grounding: SDK 2.02 has
`AF_Area_Position` (x<<16|y in 640×480), `RemoteTouchOperation` (Spot / Tracking AF) and `GetLiveViewProperties`
(focus frames to draw). Also left before ratification: LV-S3 desktop half; the owner's phone design folded in first.

D13. **2026-11-26 — Lab: LV-S3 desktop half and LV-S5 done** (Experiments 09 and 10, owner-attended). Desktop window:
a Tauri channel delivered every frame at ~16 ms (p95 22 ms) even with the window hidden, while a WebSocket in the same
window (allowed from `tauri://localhost`, loopback and LAN name) dropped frames with ~110 ms tails → 10J delivery is
now two outlets over one hub: Tauri channel for the desktop, WebSocket for phones. Focus from the live picture works
on the a7R V with the owner's settings unchanged (AF-S, Flexible Spot S): 9/9 focused, click → focused ≈ 0.4 s; the
focus frame (centre + size, 640×480 grid) is drawn over live view; focus and capture run on their own threads so live
view keeps moving. All spikes done; 10J waits only for the owner's phone design before ratification.

D14. **2026-11-26 — Owner: focus setting and three more tests.** If the camera's Focus Area is not movable, the app
reports it and does not change it (confirmed). Before implementation the owner wants: tap-to-focus from a phone over
Wi-Fi, a photo sharp where tapped, and several phones at once → new spike LV-S6 (lab Experiment 11).

D15. **2026-11-26 — Lab: spike LV-S6 done** (Experiment 11, Wi-Fi, owner-attended). From the Android phone over Wi-Fi
10/10 taps focused in 0.38–0.83 s; both photos taken after a tap are sharp exactly where tapped; the Mac and the Android
phone watching together each got every frame (19.4 fps) with none skipped. The iPhone could not reach the Mac that
session (worked the day before); the owner skipped it. All 10J spikes done; the chapter waits for the phone design.

D16. **2026-11-26 — Owner design: Shoot Companion (phone app).** Handoff in `claude_design/live_preview/
design_handoff_shoot_companion/` (high fidelity). Recorded: LV-D15 the design is the phone app (gestures; overlay
opacity gesture replaces LV-D6's style setting), LV-D16 split preview on live, LV-D17 item drawer with an empty "Fill
item properties" page for later inputs (e.g. 10I), LV-D18 the phone follows the Mac's active session (session
completion and opening from the phone later). Conflicts with the book raised as cards: LV5 current shot vs 10G shot
order, LV6 retake timing vs MC-LV3, LV7 after the last shot extra Main vs Condition, LV8 move = swap vs replace,
LV9 clearing focus, LV10 the Mac's live view.

D17. **2026-11-26 — Owner answers LV5–LV10; 10J READY_FOR_RATIFICATION.** LV5 (a): one current shot per templated
session, every exposure bound to it at the shutter, advancing in shooting order → MC-LV3 rewritten as "Current shot"
(absorbs 10G's shot-index placement, refill and the retake aim; amends 10G §4 rule 2 and §6 at ratification). LV6 (a):
Retake denies the old photo at once (undoable). LV7: shots after the last template shot stay Condition (10G §4 rule 4);
the design's Free shot adapted. LV8 (a): phone Move swaps, the Mac keeps replace-by-drop. LV9 (a): clearing focus
returns the camera's focus point to the centre. LV10: the Mac's live view screen waits for a later owner design; the
phone first.

D18. **2026-11-26 — Owner: chapter 10J RATIFIED** ("i ratify"). Amended with it: guide `03_rules_and_principles.md`
(live view in scope; only live-view settings and the focus point are written, never exposure, network or FTP;
focus/capture off the pull loop; clean camera stops), 10E (capabilities `camera.live_view`, `camera.focus.point`;
live view leaves §Out of scope), 10G (§4 note: placement by 10J's current shot), 11 (phone remote = Shoot
Companion, plain HTTP, follows the active session), 13 (phone remote and live view leave the deferred list),
04 S-M. Owner: design folders are not committed (`claude_design/` git-ignored from now on; files already in the
repo stay until the owner decides). Next: mechanism inventory, then planning (docs/process/flow.md).

D19. **2026-11-26 — Owner answers at the 10J hardware check (LV4, paused part-way).** 10J LV-D19: the phone's shot
dock drops hold-to-magnify and hold-and-drag-up (larger tiles, sideways scroll, single tap opens a filled shot's
preview or makes an empty shot current); a removed phone's installed app is reinstalled from the new QR code (no
re-pair into an installed app for now). Findings of the check so far (to fix, not semantic changes): Mac review
actions do not reach phones; the next shot can briefly aim at a shot whose file is still on the way (contrary to
MC-LV3); a Wi-Fi stall closes a phone's socket instead of skipping frames; Focus Area Wide not reported; the camera's
exposure readout not read; tile previews stuck as broken images. Evidence: `docs/implementation/STATUS_live.md` (LV4
running record), to be written into the evidence files when the check completes.
