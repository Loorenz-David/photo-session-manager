# 10I — Measuring tape (dimensions from a Bluetooth tape)

**Status: COLLABORATING, parked (shaper draft 2026-11-21, not ratified, not planned).** Owner 2026-11-23: implementation
held until later; cards MT1–MT5 to be answered then. Resume here, with the Claude Design handoff for the panel. Source: the owner's draft
`docs/intention/messuring-tape-implementation/SESSION_MANAGER_REEKON_T1M_INTENTION.md` (kept unchanged as the
raw input). Evidence: `evidence/reekon_t1m.md`. The UI look and interaction come from the owner's Claude Design
work, still in progress; this chapter fixes the behaviour the design must satisfy, not its layout.
Open owner cards MT1–MT5 at the end. Changelog entries D1–D2 (first written as 10G on a stale checkout, renamed 2026-11-26).

## Why
Measuring furniture is part of the photographer's job at the shoot. Today each dimension is read off a tape and
typed into the product form. A Bluetooth tape (first device: REEKON T1M Utility, metric, ordered; vendor SDK
v1.5 requested) lets the photographer press the tape's button and have the reading land in the right field of
the item being shot, without typing, and without a stray press ever changing an item.

## Vocabulary (shaper decision: "measurement" is taken)
In this book **M-n** and "measurement" mean the measurement ledger (chapter 14). This chapter therefore uses:

| Term | Meaning |
|---|---|
| **Tape** | A measuring device integration. Core knows only "tape"; the brand lives in its adapter (H7). |
| **Reading** | One deliberate capture from the tape (one button press): a length, a time, a device. MC-MT1. |
| **Measuring order** | The ordered list of dimension steps for an item type (the draft's "template"). |
| **Step** | One entry of a measuring order: which attribute it fills, its label, required or optional. |
| **Measuring run** | One opening of the measuring panel for one item: armed state, current step, drafts. MC-MT2. |
| **Draft** | A value a run holds for a step and has not yet written. |

## The integration family
A tape is a fourth integration family beside camera, editing and commerce (R22/R23 pattern): **tape service →
tape port (core trait) → device adapter with declared capabilities**. One adapter in this chapter: REEKON T1M
over Bluetooth Low Energy. Nothing REEKON-, BLE- or UUID-specific enters `psm-core` or core tables (H7). The
tape is optional: with no tape configured, or none connected, everything else works and the dimension fields
are typed by hand as today (same spirit as H6).

Layers, mapped to this repo (`docs/guide/02_architecture.md`):

| Layer | Owns | Must not know |
|---|---|---|
| Adapter (`psm-integrations`, e.g. `measuring/reekon/`) | discovery, connect, BLE service discovery and subscription as the vendor documents them, packet decoding, disconnect detection, reconnect with backoff, device identity, status | which item is open, which step is active |
| Tape port + service (`psm-core`) | turning adapter output into MC-MT1 readings; status; the event to the UI | REEKON, BLE, packet layouts |
| Measuring service (`psm-core`, application service) | MC-MT2: the run, its item, measuring order, current step, drafts, review pause, retake intent, cancel, confirm; the **only** place a reading becomes a value | Tauri, React, the device |
| Tauri commands/events (`psm-desktop`) | thin calls into the measuring service; events for status, readings accepted/ignored, run state | any rule |
| UI (`apps/desktop/src/features/…`) | the tape button in the product form, the measuring panel, status display, manual entry | any rule (the run lives in Rust, so the phone remote later drives the same run, H8) |

The draft placed the run controller in the UI layer; it moves to a Rust service (H8, R24) — the panel only shows
and forwards. The draft's module list is a responsibility map, not a layout; names go to the master plan
registry at planning.

## What the photographer gets
1. The tape can stay connected all day, whatever the app is doing. Its state is always visible (same family of
   states as the camera: `disconnected`, `connecting`, `connected`, `reconnecting`, `needs_action{reason}`).
2. The product form shows a **tape button** on the steps that hold dimension fields. Pressing it opens the
   measuring panel for **this item** and arms a run.
3. The panel shows the current step large ("**1/4 — Height**"), the latest reading, and what is already stored
   for each step. The run starts at the **first step with no value**; steps that already have a value show it
   and are skipped by auto-advance.
4. A press on the tape fills the current step's draft; the value stays on screen for the **review pause**
   (setting, default 1.5 s, range 0.5–5 s), then the panel advances to the next empty step.
5. Back / Next move between steps without changing anything. **Retake** on a step makes that step ready for one
   reading, replacing its draft or its stored value. Optional steps can be skipped. A value can always be typed
   into the panel instead of measured.
6. **Confirm** shows what will be saved (old → new per field) and writes it. **Cancel**, closing the panel,
   switching to another session, or quitting writes nothing; the item keeps its values.
7. Disconnecting the tape (sleep, out of range, battery) keeps the run, its drafts and its step; the panel says
   the tape is away and manual entry stays available; when the tape returns the run continues.
8. Nothing reaches Shopify until the photographer publishes as today (H1). The tape writes into the item exactly
   as typing would; publishing is unchanged.

## Measuring orders (no second taxonomy)
- A measuring order belongs to an **item type** (`items.item_type`, the Shopify product Type, changelog 42), the
  same key Customize-fields presets use (chapter 08). No furniture taxonomy of its own, no metafield schema of
  its own.
- A step targets **one attribute definition** that exists in the imported schema (chapter 08) and that the tape
  can fill (which kinds: card MT1). The step's label defaults to the definition's label.
- An item type without its own order uses the **default order** (initially Height → Width → Depth, mapped to
  the fields card MT1 picks). The owner's examples (dining chair + Seat height; sofa + Seat height, Seat depth;
  extendable table + Extended length) are configured, never hard-coded.
- A step whose definition disappears from the schema (deleted in Shopify) is shown as unavailable and skipped,
  never written.
- Orders are local data, edited where the design places it (Customize mode or Settings; pending the handoff).
- Required/optional affects only the display and the confirm summary ("Seat height not measured"). **Confirm is
  never blocked** by a missing step, matching "the form never blocks publishing" (chapter 08). Shaper decision.

## Writing the values (fits MC-9, no competing persistence)
- On Confirm the measuring service writes each step whose draft differs from the stored value through the
  existing attribute write path (`AttributeService::set_value`, `crates/core/src/services/attributes.rs`) with
  provenance `human` (the photographer measured it deliberately). No new value table; drafts are never stored
  in `attribute_values`.
- All writes of one Confirm are **one undo step** (MC-30, ⌘Z restores every field the confirm changed).
- The stored form is the field's MC-9 canonical form, from the reading by MC-MT3 (unit and rounding: card MT2).
- **Drafts live in memory in the measuring service.** They survive tape disconnects, panel minimising and window
  focus changes; they do not survive quitting the app (nothing half-measured is ever on disk). Shaper decision;
  the owner may ask for persistence.
- Switching to another session while a run holds unsaved drafts disarms the run at once and asks
  "Save n dimensions to <SKU>? Save / Discard"; Save writes to the run's item, never to the newly shown one.

## Device behaviour the adapter must respect (from the draft; facts pending the SDK)
- Discover, let the photographer pick a tape, remember its identity (user configuration, not a secret).
- Reconnect after sleep/wake, out of range, power cycle and app restart **as far as the protocol allows**;
  bounded backoff, no aggressive loop; after the limit the status says what to do.
- Decode only what the vendor documents. No invented UUIDs, opcodes, packet layouts, pairing rules or
  button semantics; every protocol fact cites `evidence/reekon_t1m.md`.
- The T1M stores 1000+ readings on the device (vendor page). **A stored reading synced on connect is never a
  reading for a run** (MC-MT1 `origin`); if the protocol cannot tell live from stored, readings arriving in a
  connect-sync window are refused (contract finalised from the SDK).
- Duplicate suppression only where the protocol gives an identity (sequence number or id). Two presses with the
  same value are two readings.
- macOS: the app needs `NSBluetoothAlwaysUsageDescription` in `apps/desktop/src-tauri/Info.plist` and the
  Bluetooth privacy permission. Risk to verify on the work Mac: an ad-hoc re-signed build may ask again, as the
  Keychain does (`docs/guide/04_change_impact.md`, fragile spots).
- Rust BLE stack: evaluate `btleplug` (CoreBluetooth backend) against the SDK; no official Rust SDK is assumed.
  Where the adapter's source may live depends on REEKON's terms (card MT5).

## Mechanism contracts (to be deepened by the mechanism inventory)

### MC-MT1 — Tape reading
A reading the tape service hands to the measuring service:
`{ device_id: string (adapter-assigned, stable per physical tape), length_dmm: integer (length in tenths of a
millimetre, ≥ 0), received_at: monotonic instant + wall time, origin: live | stored, reading_id: optional string
(only when the protocol supplies an id or sequence) }`.
- Integer tenths of a millimetre, never a float (MC-9 spirit: decimals are never floats). The T1M's stated
  accuracy is 0.5 mm; one decimal of a millimetre is enough for every unit the shop accepts.
- The adapter converts the device's unit (metric or imperial blade) to `length_dmm`; an undecodable or
  out-of-range packet produces no reading, a log line and a status note, never a guess.
- `received_at` is stamped by the app when the packet arrives, not by the device.

### MC-MT2 — Measuring run acceptance
One run at most per app process. State: `item_id` (fixed at arming), `order` (snapshot of the measuring order at
arming), `step` (index), per step `{draft?, stored?, ready: bool}`, `armed_at`, `phase ∈ {ready, reviewing(until),
summary, closed}`.
A reading is **accepted** only when all hold: a run exists and is not closed; `origin = live`;
`received_at ≥ armed_at` (and ≥ the moment the current step became ready); `phase = ready`; the current step is
ready. Otherwise it is **ignored**, the UI is told why (no run · stored reading · before arming · reviewing ·
step already filled · summary), and nothing changes. Accepting sets the step's draft, clears its `ready`, enters
`reviewing` for the review pause, then moves to the next step with neither draft nor stored value (or to
`summary` when none is left). Retake sets `ready` on the chosen step and phase `ready`. Back/Next change only
`step`. Readings are never queued for a later step. What a reading during `reviewing` does: card MT3.

### MC-MT3 — Reading to field value
Converts `length_dmm` to the target field's canonical form (MC-9). Unit codes are the adapter's (MC-9), so core
cannot know what `cm` means: the commerce adapter declares, in the definition's `validations`, the length of each
of its unit codes in tenths of a millimetre (`mm` 10, `cm` 100, `m` 10000, `in` 254, `ft` 3048, `yd` 9144 for
Shopify, `evidence/shopify_admin.md` SV-1). Rounding is half away from zero to the step's decimals (card MT2).
A text target (if card MT1 allows it) gets `"<number> <unit label>"`. Exact; no floats anywhere.

## Development before the tape and the SDK arrive (draft §7, kept)
- A **fake tape** in `psm-testkit` implements the port: inject readings (live and stored), duplicates, malformed
  packets, disconnect, reconnect, sleep. The automated suite never needs a tape or Bluetooth (flow §6A).
- A development-only way to inject readings into the running app (debug builds only), so the panel can be tried
  on screen before hardware.
- Order of work: port + MC-MT1 + fake → measuring orders + MC-MT2 run + MC-MT3 writes → panel and tape button
  (after the design handoff) → BLE adapter strictly from the SDK → hardware acceptance on the work Mac.
- If the SDK is late, keyboard mode may bridge the gap only as card MT4 says; it never becomes the architecture.

## Measurement ledger entries (proposed, enter chapter 14 on ratification)
| ID | Contract | Observable outcome | Defect family guarded |
|---|---|---|---|
| **M-MT1** | MC-MT2 | With no armed run, or for a run on another item, a reading changes no attribute value of any item; a stored (synced) reading or one received before arming never fills a step. | a stray press or a memory sync silently editing an item |
| **M-MT2** | MC-MT2 | A step that holds a draft or a stored value changes only after Retake on that step; a reading during the review pause or the summary never fills any step. | a quick second press overwriting the next dimension |
| **M-MT3** | MC-MT2, MC-MT3, MC-30 | Confirm writes exactly the changed steps to their mapped fields in the field's canonical form (unit, rounding), provenance human, as one undo step; Cancel, close, session switch or quit write nothing; nothing reaches the shop outside a confirmed publish. | wrong field, wrong unit, partial or ghost writes |
| **M-MT4** | MC-MT1, MC-MT2 | A disconnect, sleep, reconnect or adapter restart during a run leaves its item, step and drafts unchanged; reconnect attempts are bounded; manual entry works with no tape. | losing work when the tape sleeps; reconnect storms |
| **M-MT5** | MC-MT1 (hardware acceptance, operational) | On the work Mac with the real T1M: 100 presses give exactly 100 readings, each equal to the tape's display; sleep/wake and power-cycle reconnect; results recorded, not assumed. | a protocol guess that passes every fake-based test |

## Scope
**Ships with this chapter:** tape port and service, fake tape, measuring orders per item type with a default,
the measuring run and Confirm through the attribute write path, the tape button and measuring panel (design
handoff), tape status and Health entry, Settings to pick/forget the tape and the review pause, the REEKON T1M BLE
adapter, hardware acceptance (M-MT5).
**Only if cheap:** the keyboard-mode bridge (card MT4); device battery level in the status if the protocol gives it.
**Non-goals:** live streaming of the moving blade; inferring which dimension was measured; a second tape brand
(the boundary is kept for it); replacing or redesigning the product form; cloud processing or the vendor's apps
in the loop; writing to Shopify outside a confirmed publish; persisting drafts across quit (unless the owner asks).

## Open questions for the vendor (block the adapter, not the rest) — tracked in `evidence/reekon_t1m.md`
How a press arrives (notify / indicate / read / command); service and characteristic UUIDs and packet layout;
units, scale and endianness; one event per press and whether it carries an id; how stored readings are synced
and marked; behaviour on sleep, wake, power-off; bonding, pairing, firmware minimum, exclusive connection;
auto-reconnect from macOS without the vendor app; **licence and redistribution terms** (card MT5).

## ⚠ OWNER DECISIONS REQUIRED (5)

**Card MT1 — Which fields does the tape fill?**
- **Story:** Your store has two sets of size fields. "Height", "Width", "Depth", "Seatheight" are plain text;
  "Height Dimension", "Width Dimension", "Depth Dimension", "Extension Dimension" are number-plus-unit fields.
  There is no number field for seat height or seat depth. If the tape fills the set your theme does not show,
  measured chairs look unmeasured on the shop.
- **Branches:** (a) number fields only — clean values; seat height/depth need new "Dimension" definitions you
  create in Shopify. (b) text fields only — matches today's listings; values written as "72 cm". (c) a step may
  target either kind — most flexible, slightly more to configure.
- **Recommendation:** (c), so each step points at whatever your theme shows today and can move later.
- **On silence:** the gate holds.
- **Trace:** §Measuring orders, MC-MT3, M-MT3.

**Card MT2 — In what unit and precision are dimensions saved?**
- **Story:** The tape reads to half a millimetre. A sideboard measured at 1,203.5 mm can be saved as "120 cm",
  "120.4 cm" or "1204 mm". Listings that mix styles look careless, and the shop's filters compare numbers.
- **Branches:** (a) whole centimetres; (b) centimetres with one decimal; (c) whole millimetres; any of these
  per step if you want exceptions.
- **Recommendation:** (a) whole centimetres as the default, changeable per step, because furniture is listed
  that way.
- **On silence:** the gate holds.
- **Trace:** MC-MT3, M-MT3.

**Card MT3 — A second press during the short review pause.**
- **Story:** You press for the height, see 74 cm, realise the blade slipped and press again within a second.
  The panel was about to move to Width. If the press counts as Width, the width is silently wrong.
- **Branches:** (a) ignored, with a visible "ignored — use Retake" flash; (b) replaces the value just captured
  and restarts the pause; (c) ends the pause and fills the next step.
- **Recommendation:** (b), because a quick re-press almost always means "that was wrong", and it never touches
  another dimension.
- **On silence:** the gate holds.
- **Trace:** MC-MT2, M-MT2.

**Card MT4 — Use the tape's keyboard mode until the SDK arrives?**
- **Story:** The tape may arrive before REEKON answers. In keyboard mode it types the number like a keyboard.
  Typing straight into fields would bypass every safeguard, but the measuring panel could listen for it only
  while it is open and armed.
- **Branches:** (a) yes, as a temporary second adapter that works only inside the armed panel; (b) no, wait
  for the SDK and use the fake tape until then.
- **Recommendation:** (a) if the tape arrives first, because you can measure for real from day one with the
  same safeguards.
- **On silence:** the gate holds.
- **Trace:** §Development before the tape arrives, Scope.

**Card MT5 — If REEKON forbids publishing protocol details, where does the adapter code live?**
- **Story:** The repository is public. A BLE adapter must contain the tape's identifiers and message layouts.
  If REEKON's terms forbid sharing them, publishing the adapter would breach the terms you accept to get the
  SDK.
- **Branches:** (a) ask REEKON for permission to publish the adapter; if refused, keep it private, built in on
  your build Mac like the Sony camera files; (b) always keep it private.
- **Recommendation:** (a), because the public repo stays complete when allowed and safe when not.
- **On silence:** the gate holds.
- **Trace:** §Device behaviour, Scope, evidence open questions.
