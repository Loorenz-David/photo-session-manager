---
plan: P16_operations
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-26
actor: opus
---
# P16_operations: review r1 handoff

First review of the phase (no earlier rounds). Verdict **APPROVED_WITH_NOTES**: all 12 rows hold at their boundaries, the build is green, and there are no B1–B4 findings. Ten non-blocking notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker row P16_operations was `IMPLEMENTED`.
- Checkpoint `f18db80` is in `git log`.
- Passed.

## Verified correct
- **Perimeter.** `git show --stat f18db80` lists 33 paths, the same 33 the handoff declares under "Files changed". Nothing is extra or missing.
- **B4 (re-run on the same tree, clean working tree):**
  - `cargo test --workspace`: 387 passed, 0 failed.
  - `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
  - `cargo fmt --all -- --check`: exit 0.
  - `npm run typecheck`: exit 0.
  - `npm test`: 7 files, 92 passed.
- **C1(a).**
  - `Settings::open` no longer clamps, so the stored value stays 40.
  - `SettingsService::get` adds the clamped `effective_preview_delay_s`.
  - Runtime MC-17 still holds: `App::build` passes the raw value into `PreviewService`, which clamps it with `clamp_delay`.
  - `Settings::set` writes a `create_new` 0600 temp file, syncs it, renames it and sets the mode to 0600.
  - The test asserts the file, the DTO and the mode. It would fail on a clamp-on-write, a missing effective value or another mode.
- **C1(b).**
  - The mask is `••••` + the last 4 characters.
  - On save, an integration whose `client_secret` starts with `••••` gets the stored secret back.
  - The Rust test round-trips the shown DTO. The UI test checks the field's value and that the patch sends the masked block back unchanged.
- **C2(a), (b) and (c).**
  - `backup::create` runs the rusqlite online backup at `App::build` when `start_backup` is set. That is the case for `open`, not for `reopen`.
  - `PublishJob` backs up on the transition to `Done` only (`was_done` guard).
  - `prune` keeps the newest 30 by name. The `~NNNN` collision suffix sorts after its base name, so the order is preserved.
  - The tests check that the backup opens as SQLite with the Unassigned row, the count before and after a real publish to `Done`, and the exact surviving name list after 31 old files plus a restart.
- **C3(a).**
  - `App::health_check` re-runs `EditingStartup` and the camera status when a document is configured, then `HealthService::check_integration`.
  - `check_integration` sets the token item to `check_token()` and its MC-23 message. It looks the condition key up in the stored definitions and reports "not found in the shop" or "is not a file list", following the MC-24 wording.
  - The test asserts the exact list of failing codes and both details.
  - `health_get` is now a thin wrapper.
- **C3(b).** The UI test checks `data-ok` on the rows that carry the detail text, and that Retry calls `getHealth` a second time.
- **C4(a).** `OsaRunner::run` logs `op=<script stem>` and `duration_ms`. The test runs real `osascript` and parses the duration as `u64`.
- **C4(b).** The real `ShopifyAdapter` runs over `FakeTransport`. The test asserts that mutation lines exist, then checks that every one of them carries `run_id=<id>`.
- **C5(a).**
  - The tests capture logs through the production `redacting` writer.
  - The secret is registered in `App::build` and in `settings_set` before the write. Each fetched token is registered in `auth.rs`.
  - `Debug` was removed from `Settings` and stays redacted on `IntegrationSettings`.
  - The test also asserts that `capture_events` and `notifications` are non-empty and free of both strings.
- **C5(b).**
  - The test walks the temp data dir and asserts the secret is found only in `settings.json`.
  - Reviewer grep of the full checkpoint diff: the owner's real `SHOPIFY_SHOP`, `SHOPIFY_CLIENT_ID` and `SHOPIFY_CLIENT_SECRET` from `~/.psm_shopify.env` (counted, never printed) gave 0 hits each, and `shpss_|shpat_|shpca_` gave 0 hits.
- **C6(a), re-run live by the reviewer.**
  - Capture One was running with `session test 1.cosessiondb`, browsing `Capture`.
  - The exact row command printed `"130"` in 0.24 s.
  - It was still browsing `Capture` afterwards.
  - `grep -c "whose({file"` returned 0.
  - The script iterates `collections()`, canonicalises `/private` and the trailing `/`, and never assigns `currentCollection`.
  - The suite stand-in simulates the document under node.
- **Assumptions 1–7.** None contradicts ch12, 09A or 10A. Assumption 6 (a backup per item) follows the plan's wording, while ch12 says per run. It is a superset, because a backup always follows run completion. Noted for the planner.
- **Deviation.** Removing the core `capture_one` health item is sound: no consumer remains (grep), and it closes the P04 → P16 H7 backlog note.
- **H1–H8.**
  - H1: `check_token` is read-only (token grant plus `shop { name }`).
  - H2–H4: untouched.
  - H5: the script uses the dictionary only. The cold-start measurement's System Events count was a probe, not product code.
  - H6: the integration items are cleared when no adapter is configured.
  - H7: the new core health code takes the key from the adapter, the same pattern as `publish.rs`. The `"shopify"` integration literal belongs to the existing backlog family (P13/P14 notes).
  - H8: the settings and health commands only convert DTOs and lock the app.
- **Secrets and no-live-services rules.** The suite uses fakes, and `node` is the only external process.

## Findings
None blocking. No B1–B4 item.

## Notes written to backlog
Ten rows were appended to `docs/implementation/backlog.md`:
1. `health_get` now does Shopify network calls while holding the app lock. This extends the P00 note.
2. `IntegrationSettingsDto` lacks the `api_version` and `condition_photos_key` defaults, so the settings screen fails on a hand-written block without them.
3. The condition-field health item reads red until the background schema refresh finishes, or when that refresh failed.
4. Backup per item versus per run (plan versus ch12). A big batch rotates the start backups out. Planner lesson.
5. `settings_set`'s reopen shuts the workers down before the new build. A failed build leaves the workers stopped.
6. A start-backup failure aborts `App::open`, so the app does not start.
7. `health_changed` is emitted but nothing listens for it.
8. Any value prefixed `••••` keeps the stored secret, so an edit appended to the mask is silently ignored.
9. The P00 note persists: logging is initialised after `App::open`, so startup provider calls are not logged.
10. Resolved by P16: the `capture_one` health item, health assembly in the command, and a non-number preview delay (now 15).

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1, 5, 6, 9 | post-MVP hardening (backlog; orchestrator assigns) |
| 2, 3, 7, 8 | settings/health polish (backlog) |
| 4 | planner lesson, folded upstream by the orchestrator; owner may choose per-run (narrative "Choices made for you") |
| 10 | closed |

## Lessons for the plans
- Copy the intention's unit into rows. ch12 says "after every completed publish run", while task 2 and C2(b) say "publish item".
- The goal listed "theme" as a settings field, but ch12 does not list it. The implementer kept it on the header, which is fine. Planners should cite ch12's list verbatim.

## Write perimeter (this session)
- `docs/implementation/master_plan.md` (P16 tracker row only → `APPROVED`)
- `docs/implementation/plans/P16_operations.md` (Review log entry appended)
- `docs/implementation/backlog.md` (10 rows appended)
- `docs/implementation/narratives/P16_operations.md` (new)
- this handoff (new)

No production or test file was touched, and no mutation probes were run (flow §4). The live C6(a) run was a read-only lookup: Capture One's browsed collection was the same before and after. The suite runs used temp dirs. Scratch files outside the repo: `/tmp/p16_review_suite.txt`, `/tmp/p16_rt.txt`, `/tmp/p16_cl.txt`, `/tmp/p16_diff.txt`. There is no architecture-graph state. Nothing committed: the runner commits.
