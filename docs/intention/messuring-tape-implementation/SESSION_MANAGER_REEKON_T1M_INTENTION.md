# Session Manager — REEKON T1M Measurement Integration

**Status:** Initial implementation intention / planning input  
**Target:** macOS Session Manager (Tauri, Rust backend, React/TypeScript UI)  
**Hardware:** REEKON T1M Utility Digital Tape Measure — **Metric**  
**Implementation approach:** Build software before device arrival; validate against real hardware afterward.

## 1. Purpose

Enable the Beyo Vintage photographer to measure furniture with a physical tape, press the T1M capture button, and transfer the recorded dimension into the currently open furniture item's measurement workflow. The application—not the tape—determines whether a captured value means height, width, depth, seat height, or another configurable dimension.

**No live measurement streaming is required.** Only discrete, deliberately captured measurements are relevant.

## 2. Fundamental product behavior

- The T1M can remain connected independently of whether the photographer is measuring an item.
- The item form includes a **tape-measure button** that opens an explicit measurement interface and starts an armed measurement session.
- Outside an armed session, incoming device measurements **must not change any item fields**.
- The measurement interface shows the active step prominently (e.g., **1/4 — Height**) and the latest captured value.
- On capture, show the value for a configurable short review period before advancing to the next dimension. Do not assume a fixed delay; choose an accessible default and allow adjustment.
- Support previous/next navigation, retake, explicit overwrite, skip optional fields, cancel, and confirm.
- An accidental capture must never silently overwrite an existing dimension. Previously captured values are changed only by an explicit retake/replace action.
- Keep all captured values as **drafts** until the user confirms. Commit to the existing item form/domain through its normal save path; do not create a competing persistence model.
- Do not save incomplete measurement drafts automatically to Shopify or other external systems.
- Disconnecting the tape must not discard the in-progress draft; manual entry remains available.

## 3. Measurement templates belong to the application

Use the item's existing type/classification and form schema where available. Do **not** duplicate or invent a second furniture taxonomy or Shopify metafield schema.

Example templates (illustrative, not hard-coded):

| Furniture type | Suggested ordered steps |
|---|---|
| Generic item | Height → Width → Depth |
| Dining chair | Height → Width → Depth → Seat height |
| Sofa | Height → Width → Depth → Seat height → Seat depth |
| Extendable table | Height → Width → Depth → Extended length (if applicable) |

Templates should allow required/optional steps, display labels, order, and mappings to existing domain fields. If a product type has no template, provide a safe generic template and/or manual configuration. Values should be normalized internally to millimeters, with unit conversion and display handled at the UI boundary.

## 4. Architectural boundaries

### A. Persistent hardware transport and reliability (Rust)

A device adapter/service owns discovery, connection, BLE service discovery, subscription/read as specified by the vendor SDK, disconnection detection, reconnection policy, device identity, and status reporting. It must not know what furniture item is open or which dimension is active.

Responsibilities:
- Discover and connect to a user-selected T1M; remember its identity safely.
- Expose explicit states such as `disconnected`, `connecting`, `connected`, `reconnecting`, `error`.
- Recover gracefully from sleep, wake, out-of-range, app restart, and device power cycling, subject to actual protocol behavior.
- Parse vendor measurement packets into a device-neutral captured-measurement event.
- Normalize units, validate payloads, and suppress duplicates **only when reliably identifiable**; never drop legitimate repeated measurements merely because their values match.
- Do not claim that a measurement was received or persisted without evidence.
- Surface actionable connection errors and avoid endless aggressive reconnect loops.

### B. Measurement capture service (domain-neutral boundary)

Convert verified hardware events into a stable internal contract, e.g.:

```ts
type CapturedMeasurement = {
  deviceId: string;
  valueMm: number;
  receivedAt: string;
  eventId?: string; // only if provided or reliably generated
};
```

The service may emit events even when no measurement UI is active, but those events are **not authorized to mutate item data**. Define event lifetime, queueing policy, and stale-event rejection so measurements captured before arming cannot later populate fields.

### C. Item measurement session (application workflow)

The measurement controller owns: active item ID, ordered template, current step, draft values, armed/disarmed state, review timer, explicit retake/overwrite intent, cancellation, and confirmation. Only this layer may map a captured measurement to a field. An incoming capture is accepted only when the correct item and measurement session are active and the controller is ready for that capture. During the review/advance interval, reject or visibly handle extra captures rather than silently assigning them to the wrong step.

### D. React/Tauri presentation

A tape-measure action in the item form opens a focused measurement window/panel. Display device status, step progress, dimension label, large captured value, and clear controls for back, retake, skip, cancel, and confirm. Show what will be saved before committing. Preserve keyboard/mouse operation and manual input.

## 5. Official documentation and integration prerequisites

**Vendor-confirmed:** The T1M supports recorded-measurement sharing via BOLDR, Bluetooth keyboard emulation, and direct connection using the REEKON SDK. The official BLE Connection Guide and SDK is **v1.5 (October 2026)**, accessible by request.

- BLE SDK guide/request: https://support.reekon.tools/hc/en-us/articles/35510332936596-BLE-Connection-Guide-and-SDK-for-REEKON-Tools
- T1M Bluetooth support: https://support.reekon.tools/hc/en-us/articles/37713390218260--T1M-Is-the-T1M-Utility-Digital-Tape-Measure-Bluetooth-Connected
- Vendor developer contact: dev@reekon.tools

**First implementation task:** Request and obtain the official SDK/protocol guide. Store the received documentation in the project according to its licensing/distribution terms. Do not invent characteristic UUIDs, opcodes, packet layouts, pairing requirements, or button-event semantics. Verify that the exact **Metric T1M Utility** model is supported and that captured measurements can be delivered directly to a macOS BLE client without the vendor app/cloud.

Questions to resolve from documentation or vendor:
1. How does a T1M capture event arrive: notification, indication, read, or command/response?
2. What service/characteristic UUIDs and packet schemas are used? What are the units and endianness?
3. Is the event generated once per capture press? Is there a sequence number or unique ID?
4. What happens to BLE connections during sleep/wake and after power-off?
5. Are bonding, pairing, authentication, firmware minimums, or exclusive connections required?
6. Can the macOS app reconnect automatically? Are there vendor restrictions on SDK redistribution?

## 6. Rust implementation strategy

Prefer a small REEKON-specific adapter behind a generic `MeasurementDevice` interface. Evaluate `btleplug` (CoreBluetooth backend on macOS) against the SDK requirements; select another CoreBluetooth binding if necessary. Do not assume an official Rust SDK exists.

Suggested logical modules (adapt to the existing repository conventions):

```text
measurement_devices/
  mod.rs               # stable device abstraction
  connection_manager.rs # discovery, lifecycle, retry, status
  reekon_t1m/
    mod.rs
    protocol.rs         # SDK-defined encoding/decoding
    adapter.rs          # BLE implementation
measurement_workflow/
  templates.rs          # mappings to existing item fields
  session.rs            # arming, steps, drafts, review, retake
  commands.rs           # Tauri command/event boundary
```

These are **responsibility boundaries, not a mandated directory layout**. Follow Session Manager's established architecture and conventions.

## 7. Development before hardware arrives

Implement the application against a **simulated measurement source** using the same event interface as the future BLE adapter. Provide a development-only way to inject captures, connection loss, reconnect, duplicate packets, and malformed/stale events. Build the UI and template logic independently from the physical tape.

Suggested sequence:
1. Inspect existing Session Manager architecture, item form, field mappings, and persistence flow.
2. Obtain the REEKON SDK; record verified protocol facts and open questions.
3. Specify transport and domain event contracts; implement simulator and automated tests.
4. Implement configurable template resolution and guarded draft-based workflow.
5. Build tape-measure button, step-by-step panel, review delay, retake and confirmation.
6. Implement BLE adapter strictly from verified documentation.
7. Validate on macOS with real hardware; adjust only documented/observed assumptions.

If SDK access is delayed, continue with simulator/UI/domain work. Bluetooth keyboard mode may be used as a temporary manual proof of concept, but it must not be confused with a verified direct-BLE implementation or made the primary architecture.

## 8. Reliability and acceptance criteria

- No capture outside an explicitly armed session changes an item field.
- No measurement changes an already populated draft field unless the user explicitly selects retake/replace.
- Captures arriving during review/transition do not accidentally populate the next field.
- Closing/canceling the panel preserves the original item values.
- Confirming applies the correct dimension-to-field mappings through existing form save logic.
- Manual entry works when the device is unavailable.
- Connection state is visible and reconnection does not reset the measurement workflow.
- Duplicate/replayed/stale events cannot silently corrupt the sequence.
- Simulated integration tests cover normal capture, wrong item, extra capture, retake, cancellation, disconnect, reconnect, and malformed payload.
- On hardware arrival, run repeat-capture tests (e.g., 100 captures), sleep/wake and reconnect tests, and comparison against a conventional metric tape. Record observed success rates and failures rather than assuming vendor-level reliability.

## 9. Non-goals for this phase

- Live streaming of the moving tape position.
- Automatic inference of which physical furniture dimension was measured.
- Replacing the existing item form or redesigning the broader Session Manager.
- Cloud-dependent measurement processing.
- Support for multiple tape brands in the initial adapter (but preserve the generic boundary).
- Automatic Shopify updates before the existing item-save workflow confirms the data.

## 10. Implementation-agent directive

Treat this document as **product intention**, not an instruction to start coding blindly. First inspect the existing codebase and official REEKON SDK, produce an implementation plan with explicit protocol evidence, identify integration points, and call out unresolved assumptions. Keep transport reliability separate from measurement-session behavior. The safety invariant is simple: **a captured hardware value never edits an item unless the photographer has explicitly entered the measurement workflow and the workflow accepts that capture for a particular step.**
