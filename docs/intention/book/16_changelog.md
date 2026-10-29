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
