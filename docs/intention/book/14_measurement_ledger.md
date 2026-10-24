# 14 — Measurement ledger (root of the trace chain)

| ID | Observable outcome, measured true ⇒ shipped | Defect family it guards |
|---|---|---|
| **M1** | Every RAW that appears in the provider's capture folder while the app runs has exactly one `photos` row in the session that was active at its arrival (or Unassigned). Holds when only the hook fires, only the watcher fires, or both. | lost captures, duplicate rows, wrong session |
| **M2** | For any photo, the classification the API reports equals the mapping of the provider's current color tag within one poll interval; a classification set through our UI is read back from the provider identically. | stale or inverted review state |
| **M3** | After a publish item reaches `done`, the integration's gallery order equals the session's main order at confirmation; every condition photo is in the condition attribute and none is in the gallery; no `deleted` photo has left the Mac in any form. | misorder, wrong destination, leak |
| **M4** | Resuming a publish item from any interrupted step — including a process kill between upload and attach — ends with the same integration media set, file set and attribute values as an uninterrupted run. | duplicate uploads, half-published items |
| **M5** | Every integration mutation the app has ever issued is traceable to a `publish_runs` row whose `confirmed_at` precedes it and whose summary names that session. Zero mutations exist without one. | unintended writes |
| **M6** | SHA-256 of every RAW equals its `raw_hash` recorded at first sight, at every later check; RAW paths are never changed by the app. | RAW corruption or relocation |
| **M7** | With no integration configured, a session with reviewed photos completes and its export folder contains exactly the main and condition photos, in order, under the naming rule, plus `item.json` with the attribute values. | integration coupling of the core workflow |

Operational measurements (recorded in S0 and S8, not tests): shot-to-original-preview and
shot-to-adjusted-preview times; cold start of the packaged app on a fresh Mac.
