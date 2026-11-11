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
