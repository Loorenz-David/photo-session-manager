# 14 — Measurement ledger (root of the trace chain)

| ID | Observable outcome, measured true ⇒ shipped | Defect family it guards |
|---|---|---|
| **M1** | Every RAW that appears in the provider's capture folder while the app runs has exactly one `photos` row in the session that was active at its arrival (or Unassigned). Holds when only the hook fires, only the watcher fires, or both. | lost captures, duplicate rows, wrong session |
| **M2** | After a classification is set in the app, the editing provider's variant for that photo carries the mapped color tag (main green, condition blue, deleted red, unreviewed none) once the provider queue is idle; the app never changes a classification from provider state unless read-back is enabled. | stale or inverted review state in the editor |
| **M3** | After a publish item reaches `done`, the integration's gallery order equals the session's main order at confirmation; every condition photo is in the condition attribute and none is in the gallery; no `deleted` photo has left the Mac in any form. | misorder, wrong destination, leak |
| **M4** | Resuming a publish item from any interrupted step — including a process kill between upload and attach — ends with the same integration media set, file set and attribute values as an uninterrupted run. | duplicate uploads, half-published items |
| **M5** | Every integration mutation the app has ever issued is traceable to a `publish_runs` row whose `confirmed_at` precedes it and whose summary names that session. Zero mutations exist without one. | unintended writes |
| **M6** | SHA-256 of every RAW equals its `raw_hash` recorded at first sight, at every later check; RAW paths are never changed by the app. | RAW corruption or relocation |
| **M7** | With no integration configured, a session with reviewed photos completes and its export folder contains exactly the main and condition photos, in order, under the naming rule, plus `item.json` with the attribute values. | integration coupling of the core workflow |

## Mechanism-contract invariants (mechanism inventory, 2026-10-25)

Each row is the invariant of one contract; the full statement is in the cited chapter. Criteria
cite either the `M` id or the `MC` id.

| ID | Contract | Invariant, measured true ⇒ the mechanism holds | Defect family it guards |
|---|---|---|---|
| **M8** | MC-1 (05A) | Inputs naming the same file (NFC/NFD, `/tmp` vs `/private/tmp`, case on a case-insensitive volume) canonicalize to one path. | duplicate rows from path spelling |
| **M9** | MC-2 (05A) | Each accepted RAW gets exactly one row by arrival, in the session given by its capture folder, for any mix and order of hook, watcher and scan; rejected files get none. | lost captures, duplicates, wrong session |
| **M10** | MC-3 (05A) | `raw_hash` is SHA-256 of the stable file, never recomputed in place; a mismatch blocks that photo's export and publish. | false or missed corruption |
| **M11** | MC-4 (05A) | List grouping is stable within a local calendar day, DST days included; no stored instant depends on the camera clock. | wrong day groups, camera-clock drift |
| **M12** | MC-5 (05A) | Positions stay `1..n` per group after any operation sequence; undo restores classification and index. | gaps, duplicates, misorder |
| **M13** | MC-6 (05A) | A photo change after the latest completed publish's confirmation leaves the session dirty. | lost dirty flag, stale platform |
| **M14** | MC-7 (05A) | Status and row label equal the first matching rule for every enumerated input combination. | wrong status, hidden failure |
| **M15** | MC-8 (05A), MC-29 (10D) | Export folder holds exactly the named files (one per output: a split photo has two) plus a matching `item.json`; no two items share a folder. | stale export files, folder merge |
| **M16** | MC-9 (05A) | Read-then-write of an unchanged value produces no write; numeric-looking choices stay strings. | silent value drift |
| **M17** | MC-10 (07A) | Every integration mutation carries a run committed as confirmed before it; runs send only frozen content minus photos deleted or moved since. | unconfirmed writes, leaks |
| **M18** | MC-11 (07A) | Kill-and-resume at any instant ends equal to an uninterrupted run, with no duplicate product, media or file. | duplicates, half-published items |
| **M19** | MC-12 (07A) | After `update`, the gallery is exactly the frozen main list in order; nothing outside own ∪ announced was deleted. | misorder, unannounced deletion |
| **M20** | MC-13 (07A) | Only changed confirmed values are written; nothing is cleared; type round trips are the identity. | clobbered platform data |
| **M21** | MC-14 (07A) | The condition attribute lists exactly the selected condition photos, in order. | missing condition photos |
| **M22** | MC-15 (10A) | Each hook call yields one effective dispatch whether the app was running, busy or closed; hooks never block Capture One. | lost events, frozen editor |
| **M23** | MC-16 (10A) | Recorded preview and export files come from the job issued for that RAW and purpose; a newer render changes the recorded path. | wrong or stale file shown or sent |
| **M24** | MC-17 (10A) | After the last adjusted event, the preview comes from a later render within the delay or on focus; in-flight renders never swallow events. | preview stuck on an old edit |
| **M25** | MC-18 (10A) | After activation, the capture folder reads back as the session's folder, or capture is blocked with an error. | captures in the wrong folder |
| **M26** | MC-19 (10A) | With the queue idle, each variant's tag equals the mapping of its latest row classification, confirmed by read-back. | stale tags in the editor |
| **M27** | MC-20 (10A) | The original preview is a byte-exact copy of the largest embedded JPEG; the RAW is unchanged. | corrupted preview, RAW writes |
| **M28** | MC-21 (07B) | A retouch of a published photo makes the session dirty; the next publish leaves exactly one media for it, same position, from a post-retouch render; the app's own tag writes never dirty a session. | dust spot stays on Shopify |
| **M29** | MC-22 (05B) | After SKU edits, captures land in and are attributed from the item's one stored folder; export and `item.json` use the current SKU; after the next publish the shop variant SKU equals it; no proposal is confirmed while the shop holds it elsewhere. | captures misfiled after a SKU change, wrong SKU on Shopify |
| **M30** | MC-23, MC-24 (09A) | At most one integration definition is `media_list`, keyed by `condition_photos_key`; condition photos are written only there; the access token and client secret never leave memory and the settings file. | damage report written to the wrong field, leaked credentials |
| ~~**M31**~~ | MC-25 (10B), **retired 2026-11-04** by 10D (the app no longer writes Capture One's crop) | After the provider queue is idle, every RAW that arrived uncropped with `crop_on_arrival ≠ off` reads back with that crop ratio; no photographer-set crop and no published RAW was written. | rectangular photos on the shop, a hand-made crop overwritten |
| **M32** | MC-26 (10C) | After a successful ensure, the configured session document exists, is open (frontmost while a session is active) and holds both recipes with their dependent properties; no other Capture One document or recipe was closed, switched or edited. | manual Capture One setup, the photographer's other work disturbed |
| **M33** | MC-27 (10D) | A saved composition reads back identically after a relaunch; no rect outside the image or off its shape is stored; a photo whose source frame changed (Capture One geometry or master size) is `needs_review` and never rendered until confirmed. | a crop drifting, a silent wrong crop after a Capture One change |
| **M34** | MC-28 (10D) | Every exported or uploaded image equals the composition's pixel rect of the current master (within resize tolerance), sRGB, compressed once; split halves are adjacent equal squares; the RAW and master are never written. | off-by-one splits, colour shift, double compression |
| **M35** | MC-29 (10D) | After a publish, the gallery is exactly the main outputs in order (split Left then Right), the damage report the selected condition outputs in order, all from the current composition revision; a composition change dirties a published session. | old framing left on the shop, halves swapped |
| **M36** | MC-30 (05A) | Undoing k steps restores the session's photos (classification, positions, composition values, attribute values and provenance) exactly as before them, redoing restores exactly as after; at most 25 steps; a step that cannot apply changes nothing. | partial undo, lost or extra steps, order drift, redo kept after a new step |
| **M37** | MC-31, MC-32 (10E; reworded by the inventory, ratify R-I38) | Every completed camera transfer (completion as MC-31 and MC-32 define it) yields exactly one file, byte-identical (SHA-256) to what was sent: under a candidate name if it is the first copy of its (file key, SHA-256), otherwise in `.duplicates/`; an aborted transfer yields none under a candidate name; no file under a candidate name or in `.duplicates/` is ever overwritten or deleted. | truncated files reported complete, partial files visible, overwrite, deletion |
| **M38** | MC-33, MC-2 (10E, 05A; reworded, ratify R-I38) | Every distinct camera file (file key + SHA-256) becomes exactly one `photos` row, in the session MC-33's association rule gives (the session active at the matched exposure, else at completion), whichever path delivered it first; an identical later delivery becomes none; a same-name file with other bytes becomes its own row. | duplicate photos, lost photos, wrong session |
| **M39** | MC-31, MC-34 (10E; reworded, ratify R-I38) | At most one SDK session, one sidecar process and one FTP listener per app process at any time, settings saves included; no transport switch starts while the camera is busy (MC-31's busy reasons); a USB session closes only for MC-31's listed reasons; every store-destination write is verified and none is *Host PC* alone; a secret never appears in a log, argv, a settings file or the repo; no other process is ever killed; Capture One is quit only on the photographer's click. | two sessions, two listeners, switch mid-exposure, camera lost to another program, unverified setting, leaked secret, killed process |
| **M40** | MC-34, MC-26 (10E, 10C; inventory 2026-11-06) | While "Take the camera back" runs, nothing but its own step 5 launches Capture One; at app start with the camera on the cable, Capture One is launched only after one of MC-34's start-order conditions holds. | Capture One relaunched by the app before the camera is held, camera lost to the system camera service |

Operational measurements (recorded in S0 and S8, not tests): shot-to-original-preview and
shot-to-adjusted-preview times; cold start of the packaged app on a fresh Mac.

## Research agent (chapter 08A, ratified 2026-11-12)

Ids stay `MA…` (the `M` numbers continue on `main`). Criteria of research phases cite these or a
mechanism contract of 08A.

| ID | Observable outcome, measured true ⇒ shipped | Defect family it guards |
|---|---|---|
| **MA1** | After any run, every field that was `human` or `ai`+`accepted` before it holds the same value and provenance; no cleared value came back. | checked values overwritten |
| **MA2** | Every attribution value written by a run has the confidence the §6 table (08B MC-A4) gives for the evidence recorded in its steps, except a value a later run lowered to `low` because it lost its support (the reason says so); none is above `low` without a read label or the required matching listings. | false designer or maker claims |
| **MA3** | Every image sent to a provider has a `research_outbound` row; none belongs to a `deleted` photo; no RAW or master bytes are sent. | photos leaking |
| **MA4** | Kill-and-resume at any point of a run ends with the same values, confidences and steps as an uninterrupted run with the same provider answers; nothing is duplicated. | half-finished or doubled research |
| **MA5** | No run's recorded cost exceeds its cap by more than the one call in flight; no run starts after the day cap is reached. | runaway spend |
| **MA6** | Every value a run writes is in the item's layout and valid for its definition (a listed choice for choice fields); every `ai` value has at least one step naming its key. | invalid values, unexplained suggestions |
| **MA7** | With no key, no network or no integration, sessions, review, export and publish behave as on `main`; the panel states why research is not running. | core workflow coupled to AI |
| **MA8** | After a publish, the shop holds no value that was `ai`+`suggested` at `low` confidence (with no replaced value) at confirmation, and no unchecked suggestion that replaced a preset default or a value from Shopify; an item confirmed with conflicts issued no shop change before its last conflict was resolved, and sent each conflicted field as resolved; other items of that publish were not delayed (owner card 1, 2026-11-12). | unchecked weak guesses or replacements on a live listing |
| **MA9** | An export contains, for every run in its range, every step, every provider call with its exact request and response, every image sent, the values written and the photographer's action on each; replaying a stage from it sends the same inputs the original call sent, for every call whose images are in the export and whose record has not expired (any other call is reported not replayable). It contains no secret, no RAW and no `deleted` photo. | a wrong suggestion that cannot be explained or reproduced; secrets or deleted photos leaving in an export |

### Research mechanism-contract invariants (mechanism inventory, 2026-11-12; chapter 08B)

Each row is the invariant of one contract in `08B_research_contracts.md`. All contracts are ratified (owner 2026-11-12).

| ID | Contract | Invariant, measured true ⇒ the mechanism holds | Defect family it guards |
|---|---|---|---|
| **MA10** | MC-A1 (08B) | For every stored field state of the write table and every research write, the row changes only where the table allows; no value in the field's cleared set (current clear plus earlier clears) is ever written by research; never-written fields and types are never written. | checked values overwritten, cleared values returning |
| **MA11** | MC-A2, MC-A3, MC-A4 (08B) | For every enumerated evidence row, the written value and confidence equal that row's single outcome, computed by code, never by a model; two pages count as two sites only with different site identities; the item's own listing never counts; every stored research value is valid for its definition. | false attribution at medium or high, one dealer counted twice |
| **MA12** | MC-A5 (08B) | The photos in every recorded provider request are exactly its outbound rows, each eligible (not `deleted`, still in the run's session) in the transaction committed before the send; nothing but a research image can reach a provider port. | deleted photos or RAW/master bytes leaving the Mac |
| **MA13** | MC-A6 (08B) | At most one research run `running` app-wide and one `queued` per session; no automatic run starts inside its session's quiet period; no run starts while today's spend is at or above the day cap. | runs on test frames, parallel spend past the cap |
| **MA14** | MC-A7 (08B) | Kill-and-resume at any instant equals an uninterrupted run given the same provider answers; no committed unit is called again and no step appears twice; a follow-up calls providers only for units whose input fingerprint has no committed equal. | doubled cost and steps, follow-ups that redo or miss work |
| **MA15** | MC-A8 (08B) | Every call's cost equals the integer micro-USD formula over its reported usage and the run's price snapshot, or the fallback when none was reported; no run exceeds its cap by more than one call; today's spend sums calls by local calendar date. | silent overspend, cap on the wrong day, float money |
| **MA16** | MC-A9, MC-A11 (08B) | No call record, research log line or export file contains a value held by the secrets port (an export with one is aborted); no export holds a RAW, a master or an image of a photo `deleted` at export time; a replayed request differs from the recorded one only in the substituted fields; call records older than `record_days` are gone. | leaked keys, leaked deleted photos, replays that silently differ |
| **MA17** | MC-A10 (08B) | For every confirmed item: no integration mutation precedes the resolution of its last pending conflict; it sends no `ai`+`suggested`+`low` value without a prior and no unresolved conflict; each pending key is sent with the value recorded at its resolution and every other key with the value frozen at confirmation; the withheld count and conflict set equal those shown; other items of the same confirmation never wait for it. | unchecked replacements on the live listing, a held item that mutates early or holds up others, a popover that lied |
| **MA18** | MC-A12 (08B) | An undo or redo never overwrites a research write made after its step; feedback rows exist exactly for the photographer actions on `ai` rows that are currently applied. | undo erasing newer suggestions, false evaluation data |
