# 15 — Decisions

## Resolved (owner, unless marked shaper)
| ID | Decision | Date |
|---|---|---|
| R1 | Native macOS app, uv-managed Python; no Docker. | 2026-10-23 |
| R2 | FastAPI, industry-standard layout; the owner's contract set is not applied. | 2026-10-23 |
| R3 | React + TypeScript + Vite + shadcn/ui; Node on the work Mac is acceptable. | 2026-10-23 |
| R4 | Shopify through a custom dev-app token; lookup fields barcode and SKU (design adds title). | 2026-10-23 |
| R5 | Public GitHub repo, commercially open source for now (C1). | 2026-10-24 |
| R6 | A Session is one item's photography job (C2). | 2026-10-24 |
| R7 | Attribute schema and provenance in the MVP; the AI agent is the first post-MVP phase (C3). | 2026-10-24 |
| R8 | One transport, HTTP + WebSocket, under Tauri and on the phone; Tauri is lifecycle only (C4). | 2026-10-24 |
| R9 | Capture One is the only imaging provider in the MVP; no pixel processing in the repo (C5). | 2026-10-24 |
| R10 | Tauri desktop shell with Python sidecar replaces the AppleScript launcher as the target. | 2026-10-24 |
| R11 (shaper) | `capture-one-mcp` not adopted: extra Node/MCP runtime designed for LLM clients; the dictionary covers every command. | 2026-10-23 |
| R12 (shaper) | Review and group collapse into one classification state (design: approve / condition / delete). | 2026-10-24 |
| R13 (shaper, from evidence) | Condition photos map to `custom.damage_report`; the brief's `custom.condition_images` does not exist in the shop. Theme rendering unverified. | 2026-10-24 |
| R14 (shaper) | "Replace" flag removed; re-publishing is the dirty-state flow from the design (see N1 for reconciliation). | 2026-10-24 |
| R15 (shaper) | Shopify API version pinned in settings, defaulting to 2026-10. | 2026-10-24 |

## ⚠ OWNER DECISIONS REQUIRED (6)

**D1 — Capture folder per session?**
Question: May the app set Capture One's capture folder to `Capture/<SKU>/` when a session becomes active?
Story: You open the sideboard session, shoot eight frames, open the lamp session, shoot six. With per-session folders the Finder and Capture One's browser both show one item at a time, and if the database were ever lost the folders still say which frame belongs where. With one folder, only the database knows.
Branches: **Yes** — one command per session switch; trivial recovery. **No** — single folder.
Recommendation: Yes.
On silence: gate holds.
Trace: chapter 06 arrival, chapter 10, M1.

**D2 — Color tags as the mirror?**
Question: Main = green, Condition = blue, Deleted = red, Unreviewed = no tag, ratings ignored?
Story: You sort in Capture One with the keyboard as you do today. The app reflects it; sorting in the app sets the same tags, so filtering in Capture One shows exactly the app's groups.
Branches: **Tags only** — one key per verdict. **Tags and ratings** — a rule for every combination.
Recommendation: Tags only; ratings shown read-only if cheap.
On silence: gate holds.
Trace: chapter 06, M2.

**D4 — Export at publish time?**
Question: Should the web-size export run when you confirm Publish (or Export), rather than when a photo is approved?
Story: Seven frames approved at 10:00, two retouched at 14:00, published at 16:00. Export-at-publish uploads the retouched versions and costs about a minute at 16:00. Export-at-approve would upload the 10:00 versions.
Branches: **At publish** — always the latest edit. **At approve** — faster publish, stale risk.
Recommendation: At publish; the design's "Preparing images" step is exactly this.
On silence: gate holds.
Trace: chapter 07 steps, chapter 10, M3.

**N1 — What does re-publishing do to photos already on the platform?**
Question: When a session is published again, should the platform gallery become exactly the session's current Main list, or should the session's photos be placed first and anything else on the listing left untouched after them?
Story: The sideboard listing had five old photos from before Session Manager. You shoot it fresh and publish. With "exactly the session" the five old photos are deleted and the listing is clean, and the design's promise "nothing was overwritten" has to be rephrased for this case. With "session first, rest untouched" the old five trail behind the new set until someone removes them in the admin.
Branches: **Mirror** — deterministic, deletes platform photos the session does not own; shown in red on the confirmation. **Session first, keep the rest** — never deletes; leftovers possible.
Recommendation: Mirror, with the deletion count stated on the confirmation popover.
On silence: gate holds; re-publish unavailable until decided.
Trace: chapter 07 update step, M3, M5.

**N2 — Unreviewed photos publish as Main?**
Question: Keep the design's rule that unreviewed photos publish in Main, with the note on the confirmation popover?
Story: You shot three extra frames after sorting and press Publish without looking. The popover says "3 new photos haven't been reviewed and will publish in Main". If you confirm, a test frame may go live. The alternative excludes unreviewed photos and says "3 unreviewed photos will not be published".
Branches: **Publish them** — faster, risk of a bad frame live. **Exclude them** — safer, one more sort step.
Recommendation: Exclude them; the human gate is cheaper than a live mistake.
On silence: gate holds.
Trace: chapter 04 S-G, chapter 07, M3.

**N3 — Where does the phone remote sit?**
Question: Is the phone remote part of the MVP (needs a phone layout designed now), or the first UI phase after the desktop MVP?
Story: The handoff says "desktop only, no phone layout". Building the MVP on one transport means the phone costs a layout and a pairing screen later, nothing in the backend. Putting it in the MVP means designing it now and adding a phase before the first publish.
Branches: **After desktop MVP** — no design work now. **In MVP** — design the phone screens next.
Recommendation: After the desktop MVP, as its first follow-up phase.
On silence: gate holds for the phone only.
Trace: chapter 04 S-M, chapter 11, chapter 13.
