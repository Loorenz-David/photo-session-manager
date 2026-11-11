---
plan: P65_capture_sync_waiting_jobs_and_inventory
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-11
actor: opus
---
# P65 review r1: handoff

APPROVED_WITH_NOTES. All 9 rows have a test at their boundary that fails on a wrong outcome. Production matches changelog 45 and the plan. The 11 assumptions are consistent with the book. H1–H8 hold. Targeted suites, clippy, fmt, typecheck and the touched UI tests are green. 5 notes went to the backlog. Nothing blocks.

⚠ OWNER DECISIONS REQUIRED (0): none.

## Gate and perimeter
- Tracker row was `IMPLEMENTED`; checkpoint `b0edd78` is in `git log`. Tree reviewed: `f46f1ba`, clean (only runner commits after the checkpoint).
- `git show --stat` of `b468cad` (34 paths), `8b26018` (5) and `b0edd78` (20): their union equals the handoff's declared files changed. The extra paths in the two sweep commits (`docs/implementation/backlog.md`, `docs/implementation/STATUS.md`) are orchestrator content. Nothing is undeclared and nothing is missing. The sweep is disclosed in the handoff and logged as a process note.

## Evidence (reviewer, plan "Review mode": full workspace not re-run)
| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | ok |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `npm run typecheck` | ok |
| `npx vitest run src/features/form src/features/settings src/features/sessions src/features/publish` | 19 files, 125/125 |
| `cargo test -p psm-testkit --test p65_… --test p01_… --test p09_… --test p29_… --test p44_… --test p62_… --test p63_… --test p64_…` | 8, 20, 8, 26, 25, 12, 4, 8: all pass |
| `cargo test -p psm-core -p psm-infra -p psm-integrations --lib` | 67, 10, 36: all pass |

The implementer's full-suite stamp is consumed. Its 3 reds were explained: the p29 helper (fixed after that run; 26/26 here) and a load-timing p43 case (26/26 on the final tree, per the handoff). One clean full run on a quiet Mac is still owed. The narrative says so.

## Verified correct
- **C1(a).** `photo_arrived` → `request_sync` → `run_syncs` → `GatedScripting::launching` → `jxa/sync_folder.js`. The script uses `collectionForFolder` from `lib.js`, sets `synchronizing = true` and throws when Capture One is not running, so it never launches it (H5: an sdef property only). Both log lines are emitted. The test asserts one `sync_folder:<folder>` op, a preview issued 3–4 s after filing against a fake that lists the photo 3 s after a sync (otherwise 60 s), style before process, and both log lines. Without a sync the test fails.
- **C1(b).** Syncs are capped at one per folder per 1 s, with one pending re-sync. `EditorLaunchDeferred` keeps the sync pending. The test observes 1–2 syncs for 3 arrivals, 0 during the hold and exactly 1 after release.
- **C2(a).** `editor_wait` matches the Display texts of `VariantNotFound`, `EditorLaunchDeferred` and `DocumentNotOpen` (checked in `error.rs`) and busy/timeout texts. A master render that is refused at request time or fails later becomes a `Wait`. `ExportService::tick` runs from `PreviewService::tick` and retries after 1/2/4/8/15 s. `refresh_waiting` marks the run row, and the session row turns `WaitingForEditor` only while publishing. `PublishJob` has no `max_attempts` and caps its back-off at 15 s, so the wait cannot run out of attempts. The test asserts Prepare for 20 s, the exact activity text, the label, Done, gallery equal to both photos once and in order, and Published.
- **C2(b).** The deadline is kept in `waited_since` across retries. The `editor_wait: ` prefix becomes `Error::EditorWaitExpired(sentence)`. Non-editor errors fail at once. The test asserts the exact sentence after 10 min, Prepare at 9:45, and `provider: process failed` at once.
- **C2(c).** `finish_cancel` in Prepare calls `ExportService::cancel_run`. The test asserts `(Cancelled, None)`, the text, the label, and that no export is running.
- **C3(a).** `product_set_input` adds `inventoryItem{tracked:true}` and `inventoryQuantities[{locationId,name:"available",quantity}]` only through `item_create_stocked`, followed by the `InventoryReadback` query. Stock is frozen at confirm into `publish_item_stock` (H1). `Tracked` remembers the default in settings.json. The tests assert the exact variables, one read-back, `Tracked{verified:true}`, one stocked request at the service, and settings.json.
- **C3(b).** Stock is used only in `create_item`, which runs only when the SKU search finds nothing. Found and republished products never reach it. The tests check the fake's stock requests and the adapter's plain-create variables.
- **C3(c).** Locations are cached per service. "No location" overrides the default. Negative quantities are refused. ACCESS_DENIED on locations returns the sentence. A refused stocked create is re-sent untracked and the publish ends Done. The UI tests check prefill, the change being saved, `Choose a location to track stock`, the permission sentence in the form and in Settings, and the Settings save.
- **C4(a).** See the evidence above.
- **Assumptions 1–11.** None contradicts changelog 45, 07A or 10A. They are relayed in the narrative.
- **H1–H8.** H1: stock frozen with the confirmation. H2: no file moved or deleted. H3: n/a. H4: the stocked create is not retried on transport errors (MC-11). H5: sdef only. H6: stock only under a commerce integration. H7: location ids sit in an integration-keyed table and a frozen publish table, like the existing link tables; the "Capture One" sentences in core follow the editing_startup/previews precedent (see note 1). H8: commands are thin over `PublishService`.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
1. The core `editor_wait` classifier matches AppleScript numbers (`-600`, `-1712`) and free text. This is adapter knowledge in core; the adapter should map to typed errors.
2. Any top-level GraphQL error on a stocked create leads to a silent untracked create. Retry only on ACCESS_DENIED, and show the reason to the photographer.
3. The stock GraphQL shapes are unverified against Shopify's schema; the Shopify docs tool was not permitted to the implementer or the reviewer. The custom app needs `read_locations` and `write_inventory`.
4. The form's Stock section disappears when locations cannot be read, while the remembered default still applies.
5. The sweep commits `b468cad` and `8b26018` do not build. The orchestrator should commit only its own paths while an implementer session is live.

## Lessons for the plans
- The plan asked for the Shopify fields to be "checked with the docs", but no session had the docs tool. Either grant the tool to implementer and reviewer, or plan a live probe row against the owner's dev store.

## Write perimeter (this session)
`docs/implementation/backlog.md` (5 notes appended), `docs/implementation/master_plan.md` (P65 tracker row only), `docs/implementation/plans/P65_capture_sync_waiting_jobs_and_inventory.md` (Review log entry), `docs/implementation/narratives/P65_capture_sync_waiting_jobs_and_inventory.md` (new), this handoff. No production or test code touched, no probes, no commits. Build artifacts under `target/` came from test runs.
