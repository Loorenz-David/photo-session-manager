---
plan: P13_publish_runs
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-26
actor: opus
---
# Handoff: P13_publish_runs, review round 1

**Summary.** All 35 criterion rows hold at their boundaries. Their tests observe the row's outcome and can fail on a
wrong one (C5(f) passes only because of the adapter contract; see notes). The perimeter equals the declared files,
and suite, lint and typecheck are green. There are **two blocking findings (B1, route production)**:
- **F1:** an upload stalls forever on a product that holds a ready non-image media (video, 3D model).
- **F2:** publish steps run on the single background loop that also handles capture arrival and previews, so every
  publish freezes both.

11 notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The book answers both findings (ch07 Failure and retry and "Publishing never blocks the workspace"; 07A MC-12),
so they go to the fix session.

## Gate
- Tracker row P13_publish_runs was `IMPLEMENTED`. Checkpoint `3b034dd7a56ce154ec4f49aa5caeae73b2835dfe` is in
  `git log`. Passed.
- `git show --stat 3b034dd` lists 37 paths. They equal the handoff's "Files changed" list exactly: commands
  `items/mod/publish.rs`, `lib.rs`, `bindings.ts`, `items.ts`, `NewSessionDialog.tsx`, the 4 UI test files, the 17
  core files, `store.rs`, the Shopify `adapter.rs`/`graphql.rs`, `integrations/lib.rs`, the 4 testkit files,
  `master_plan.md` (P13 row only), the plan file (Review log) and `shopify_admin.md`. Nothing extra, nothing
  missing. The plan's `sessions.rs` and `infra/jobs.rs` are unchanged, as deviation 2 discloses. Commits after the
  checkpoint touch the handoff and prompt only.

## Evidence (one run, master plan §7.2 commands)
Tree: HEAD `cdf3bc0`, code identical to `3b034dd`, `git status` clean before and after the review.

| Command | Result |
|---|---|
| `cargo test --workspace` | 343 passed, 0 failed (34 in `p13_publish_runs`) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 5 files, 79/79 passed |

PATH used `~/.cargo/bin` and Node 22 (`~/.nvm/versions/node/v22.22.3/bin`), per §7.1.

**Reviewer probes (flow §4: changed inputs, no production edit).** I ran one scratch file,
`crates/testkit/tests/zz_review_p13_probe.rs`, created and deleted in this session and never committed. It touched
no production file and no state outside its tempdirs. Results:
1. Publish [a,b,c], reorder c to 1, publish again: the gallery becomes [c,a,b] and both items are `done` (M3; this
   is the case C5(c)'s fixture cannot tell apart from "no reorder").
2. Confirm, prepare, classify the photo, drain: the item reaches `done`, `published_seq = confirmed_change_seq = 2`,
   `change_seq = 3`, dirty (C7(b) reaches `done`, not vacuous).
3. A linked product with a READY media with no filename, announced for removal: after 40 ticks of 61 s the item is
   still `upload`, with no `failed_step` and no error, 39 `list_media` reads, and the gallery unchanged (F1).
4. `commerce_from_settings` with `{"kind":"shopify","shop_domain":…,"client_id":"c"}` (no secret) returns `Err`.
   `App::open` propagates it and `run()` `.expect`s it (backlog note).

## Verified correct
- **C1(a)–(d)** `PublishService::confirm`: one transaction inserts the run and items. The frozen lists come from
  position order. Condition = condition minus the MC-14 `{"excluded":[…]}` ids. Unreviewed and deleted photos are
  excluded. `frozen_attributes` = valid, live, canonical values only. `announced_removals` = `item_get` media not
  in own gallery links or superseded ids. `confirmed_summary` = the passed text. `confirmed_change_seq` =
  `session.change_seq` and `confirmed_at` = clock. The confirm also re-checks that the lists did not change since
  the preview. The tests assert exact lists, maps, ids, text and time.
- **C2(a)** Every mutating port method takes `&ConfirmedRun`, whose only constructor is P12's `from_persisted`
  (checked by reading the port and the service). The test checks that every recorded mutation carries the run with
  `confirmed_at ≤ at`.
- **C3(a)–(e)** `prepare`:
  - Without an item link it searches the SKU: 0 hits → `item_create`, then `insert_item_link` before any other
    remote call; 1 hit → adopt; more → `several_products_carry_sku`.
  - `IntegrityCheck` runs per selected photo (`raw_changed` / `raw_missing`).
  - The export is staged from the frozen lists via `run_staged`, and each link gets the SHA-256 of its staged file
    with state `exported`.
  - Each test pins its outcome: mutation count 1, no `item_create`, the error code, no `staged_uploads_create`, and
    a hash equal to the file's.
- **C4(a)–(e)** `upload`:
  - Ready links are skipped.
  - Before creating, it lists the product's media (or the shop's files for condition photos) and adopts a media by
    `stem + ("_" + uuid)? + ".jpg"`, per SV-2. `name_matches` is not a prefix match.
  - Each transition is committed after its remote effect, and the poll leads to `ready`.
  - `fail_after_effect` panics after the fake's effect. `kill_and_restart` re-opens on the same dir. The tests
    count media and `product_create_media` (2, so no duplicate).
  - C4(e): the stem includes the new item's `folder_key`. The new link differs from the old one, and the old link is
    byte-identical.
- **C5(a)–(e)** `reconcile::plan` implements MC-12 steps 2–4 exactly (foreign = current − D − own − announced;
  deletes = (current − D)). `update` does the following in order:
  1. It fails `upload_incomplete` before any read or delete when a D element is not `ready`.
  2. `own` = gallery links + superseded ids + media matching any of this item's gallery stems.
  3. It fails `foreign_media_added` before deleting, then deletes, reorders all of D and polls the job.
  4. It requires the read-back to equal D, else `gallery_verification`.
- **C6(a)/(b)** `selected()` drops frozen photos that are now deleted or in another session. It runs again in
  `prepare`, `upload` (re-checked per photo just before the remote write) and `update`. The tests check that no link
  was made, the gallery and one notification.
- **C7(a)–(c)** `complete_publish_item` does all of this in one transaction: `done`,
  `published_at = last_activity_at = at`, `published_seq = confirmed_change_seq` (SQL subselect), a `published`
  notification, and removal of the export-id row. A failure stores `failed`, `failed_step` and the error, and bumps
  `last_activity_at`. The status derivation reads `failed` vs `PublishFailed`, and the tests assert both.
- **C8(a)/(b)** `retry` refuses non-failed items, sets `step = failed_step` and re-enqueues. Startup
  `requeue_running_jobs` re-queues running jobs, and `active_publish_items` re-enqueues every active item. The
  tests check that no `item_search` happens after the retry and that the job goes from `running` to `queued` to
  done with one media.
- **C9(a)–(g)**:
  - The adapter is built from settings with the default `custom.damage_report` and nine capabilities; there is
    none without a block.
  - The schema refresh reads first, then replaces definitions and links in one store transaction (an error leaves
    both untouched).
  - Search merges local results with unlinked shop hits whose SKU is not already local. An unreachable shop gives
    local results and `Ok`.
  - `adopt` inserts the linked item and its `item_links` row in one transaction.
  - `check_shop_sku` excludes the item's own link and is skipped on outage.
  - `next_sku` walks at most 50 candidates and falls back to the local proposal on error.
  - The UI test checks the "in the shop" label, then that `adoptItem` is called before `createSession(42)`.
- **Hard constraints:**
  - H1: see C2.
  - H2: RAWs are only hashed, and the export goes through the provider.
  - H3: no pixel code.
  - H4: the crash points walked are create→link, stage, PUT, create→record, attach→ready, reorder→read-back and
    swap→complete; each resumes without a duplicate.
  - H5: n/a.
  - H6: with no adapter, `publish` is `None` and the commands return `no_integration`.
  - H7: link tables only, remote ids opaque; but see the `"shopify"` note.
  - H8: the commands are thin wrappers.
  - No live service in the suite: C9(a) builds a real adapter but calls only `capabilities()`.
  - No credential appears in the diff or the evidence addendum.
- **Assumptions** (implementer handoff): none contradicts the intention. Each is reported in plain words below and
  will go into the narrative at approval:
  1. One queued job advances one item by one step.
  2. Export renders use P09's staging folder and are swapped into the export folder at `done` (the plan's
     delegated choice).
  3. The notification time is raw UTC milliseconds (delegated; backlog note for P15).
  4. Progress events are emitted at every state change, unthrottled.
  5. Title search uses Shopify prefix terms plus a local substring filter. File recovery pages through all files
     because the filename index lagged (SV-3 evidence).
  6. An unreachable shop leaves local search and the local SKU proposal working (as the plan says).

## Findings

### F1 — B1 · route: production · a ready media with no filename stalls `upload` forever
**What is wrong.** `PublishService::upload` (`crates/core/src/services/publish.rs` ~570) returns `Ok(false)`
("not yet", which becomes `Timeout` and a job retry with backoff) whenever `existing` holds any media whose
`filename` is `None` and whose status is not `Failed`. The Shopify adapter derives filenames only from MediaImage
`image`/`originalSource` URLs (`types.rs::MediaNode::filename`), so a READY video, external video or 3D model in the
product's gallery has no filename, permanently. Publishing to such a product therefore never leaves `upload`. It
never fails, shows no reason, polls the shop every minute forever, and the session row stays "Publishing". This
happens even when the photographer confirmed removing that media. Probe 3 above shows it.
**Authority.** ch07 Failure and retry ("A failed step marks the item failed with the reason"; publishing ends in
`done` or `failed`) and N1 (announced gallery removals are carried out); 07A MC-12 (unowned, unannounced media fail
`update` with the "gained n photos" message, they do not stall `upload`). The SV-2 evidence asks the matcher to wait
only while media are PROCESSING.
**Correction.** Treat a nameless media as "wait" only while its status is `Processing` or `Uploaded`. A READY or
FAILED nameless media is never an adoption candidate and is left to `update` (MC-12 deletes it if announced, or fails
with `foreign_media_added` if not). Add one boundary test: a linked product holding a READY nameless media announced
for removal publishes to `done`, and the gallery equals D.
**Files the fix may touch.** `crates/core/src/services/publish.rs`, `crates/testkit/tests/p13_publish_runs.rs`,
`crates/testkit/src/fake_commerce.rs` (only if the fixture needs it).

### F2 — B1 · route: production · publish steps freeze capture arrival and previews
**What is wrong.** `publish_step` is registered on the shared `JobRunner` (`crates/integrations/src/lib.rs` ~217). It
is ticked by the one background loop that also drains the hook spool, handles watcher events and ticks previews
(`lib.rs` ~285–300, sequential, 50 ms cadence). `PublishService::run_item` does a whole step synchronously. For
`upload` that means, per photo: list media, staged-upload create, PUT of the JPEG, media create, and
`poll_media_ready` (≥ 1 s interval, up to 120 attempts). Real network time per item is roughly tens of seconds to
minutes (more in a batch), and for all of it no capture is recorded, no original or adjusted preview is produced, and
no colour tag is mirrored. The photographer shooting the next item while the last one publishes sees nothing arrive
until the upload ends. The backlog's P04 → P05 note flagged this shared loop; P13 makes the stall routine.
**Authority.** ch07 Publish: "Publishing never blocks the workspace; rows show live progress." (The workspace's
core is live arrival and previews: ch06, M1/M24.)
**Correction.** Run `publish_step` jobs on their own worker thread: a runner that picks only that kind, with the
shared runner skipping it (for example `job_due` filtered by kind). The arrival/preview loop then keeps its cadence.
Add one boundary test: with the fake shop's media poll held, a capture delivered during the upload step gets its
photo row before the poll is released.
**Files the fix may touch.** `crates/integrations/src/lib.rs`, `crates/infra/src/jobs.rs`,
`crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`, `crates/testkit/src/test_app.rs`,
`crates/testkit/src/fake_commerce.rs`, `crates/testkit/tests/p13_publish_runs.rs`.

## Notes written to backlog
11 rows, all prefixed `P13`:
1. An incomplete `integration` block makes the app panic at launch (`commerce_from_settings` → `Err` →
   `.expect`). The plan's task 0 says "otherwise `None`". It is consistent with the P00 settings note, so it is
   routed to P16 and not blocking, but it is the first thing the owner may hit while hand-editing credentials.
2. 13 `"shopify"` literals in core services, plus SQL in the store (H7 hygiene). The plan's task 0 prescribed one.
3. The C5(f) fake flag is decoration; the not-found rule is owned and tested at the adapter (P12).
4. C7(a) `published_at.is_some()` and C9(b) count-only assertions.
5. The notification text shows raw epoch milliseconds.
6. The export job id is kept in the `settings` table (deviation 1).
7. Shopify calls on the IPC thread under `Mutex<App>` (search, SKU checks, up to 50 calls for `next_sku`, preview,
   confirm).
8. The preview's removal count ignores stem-matched own media after a crash.
9. Before the first schema refresh there is no condition key, so condition photos are silently left out (for P14,
   MC-24).
10. The NULL-hash `raw_changed` carry-forward (P05/P09 → P13, still open).
11. The P12/P09/P03 → P13 carry-forwards: the media-id filename check, the stale partial sweep, and the
    out-of-order confirmation test (still open).

## Lessons for the plans
- A core-service task must not spell the platform name. Task 0's `import_definitions("shopify", …)` invited 13
  literals. The integration name should flow from the adapter or settings.
- C5(f) ("fake returns not-found") is not observable at the core boundary when the port contract has the adapter
  absorb not-found. Rows should sit where the rule lives.
- A phase that adds long synchronous network work should name the thread it runs on. "Publishing never blocks the
  workspace" had no row, and the shared loop was a known risk (P04 → P05 note).

## Carry-forward dispositions
The notes above go to the backlog with destinations P14, P15, P16, the next migration and the plan, as written in
each row. F1 and F2 go to fix 1.
