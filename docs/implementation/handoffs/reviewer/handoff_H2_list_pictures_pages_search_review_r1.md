---
plan: H2_list_pictures_pages_search
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-23
actor: opus (pipeline-reviewer)
---
# H2 — A picture on every row, pages, and pictures in the search: review r1

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. Five notes are in the backlog.

## Gate
- Tracker line of the plan: `IMPLEMENTED` (2026-11-23).
- Checkpoints U1 `5e1a0d6`, U2 `a54ad20`, U3 `b9cb01f`, U4 `ba13aed`, close `a76020f`: all in `git log`.

## Verified correct
**Perimeter.** `git diff --stat 5e1a0d6^ a76020f` has 51 paths, and they equal the handoff's declared files. That includes the declared deviations: `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`, `__tests__/pageFromList.ts`, the mock conversions of `p18_pending_state`, `p15_publish_ui` and `c4_visit`, and the `cover_pending` fixture lines in `p08_workspace_ui` and `p66`. No path is extra or missing. `services/items.ts` is unchanged, as declared.

**Checks** (the plan's Review mode, run once by the reviewer on tree `a76020f` plus the runner's prompt commit; one cargo command at a time):

| Command | Result |
|---|---|
| `cargo test -p psm-core --lib mechanisms::remote` | 20 passed |
| `cargo test -p psm-testkit --test suite h2_row_pictures::` | 24 passed |
| `cargo test -p psm-testkit --test suite h2_pages_and_search::` | 10 passed |
| `cargo test -p psm-desktop` | 88 passed |
| `npx vitest run src/features/sessions src/features/workspace` | 29 files, 157 passed |
| `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-desktop -p psm-testkit --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | clean |

The implementer's full-suite stamp stands (`cargo test --workspace` green, `npm test` green after the `c4_visit` fix). Nothing contradicts it.

**Production code, read against MC-H1, MC-H2 and MC-H5:**
- **MC-H1.1, M-H4.** The rule lives in one place, `RowPictures::row_picture`. Without `RemoteItems` it falls back to `remote::local_picture`, which covers rules (i) and (iii).
  - Every path that `list`, `page`, `items_search` (`item_pictures`) and `items_search_shop` (`search_picture`) hand out has passed an `is_file` check.
  - A gone cover (empty URL) is neither shown, nor pending, nor fetched.
  - `Store::remote_covers` is one query, and it picks the lowest-position gallery picture, so C1(g) gets no cover.
  - Both production `list` callers attach `with_remote`.
- **MC-H1.2.** Downloads write `<name>.part`, then rename. After a cover is written, the item's other `cover-*.jpg` files are deleted. `refresh` keeps `cover_name(new cover)` (L8).
- **MC-H1.3.**
  - `request_covers` and `search_picture` only take the queue lock. They never download, and no lock is held while a download runs.
  - A new request from a source replaces only that source's queued covers.
  - `run_pending` runs at most `PARALLEL_DOWNLOADS` scoped threads and emits `PictureReady` with the file's path. The `TauriEvents` arm sends `picture_ready` with a flattened `{kind, session_id | external_id, path}`.
- **MC-H1.4, MC-H5.3, MC-H5.5.**
  - Only `ImageGone` on a cover makes it stale. `Fail` is logged and dropped, and a search picture that answers `Gone` is dropped.
  - The refresh checks the cool-down (`clock.mono()`, 10 minutes) and then `must_wait`. It asks `media_urls` in chunks of 25 and stores the URL or the empty "gone" URL, then retries the download once.
  - `media_urls` is the only GraphQL call the queue adds. The first-open reads from H1 stay behind `must_wait`.
- **MC-H1.5.** The in-memory gone map produces the exact Health words (`1 picture is no longer in the shop: <SKU>`) and emits `HealthChanged`. An item leaves the map in `read_record` (first-open read, Refresh) and in `forget_item`.
- **M-H5, 10H §9.** `session_delete::delete` removes `previews/remote/<item>/` and `previews/archived/<item>/` only when the item row is gone. These are cache files, not RAWs (H2), and no adapter call is made.
- **MC-H2.1.**
  - `page` filters `list`'s rows by `created_from` and `only_ids`. `Some([])` keeps none.
  - It cuts in `list`'s order through `page_of`, which clamps and always gives at least one page.
  - Groups keep their pre-cut `total`, and empty groups are left out. `counts` and `unassigned_count` come from `list`.
  - Pending covers on the page are requested with source `Page`.
- **MC-H2.2.** `sessions_list` keeps its shape and requests no cover. The six other `listSessions` callers are untouched (`git diff --stat` on `Header.tsx`, `usePublish.ts`, `reviewQueue.ts`, `VisitBar.tsx` is empty, and `SessionsPanel`/`PickerDialogs` still call `listSessions`).
- **MC-H2.3.** A new filter, query, time range or "To review" set means a new question, which asks for page 1. `sessions_changed` reloads the page shown, and a clamped answer moves the page state. `rangeStart` is the exact bound that `withinRange` now uses.
- **MC-H2.4–6.**
  - `search_shop_pictures` keeps the filter that drops shop hits already local or linked.
  - `search_name` drops the query string from the URL.
  - After each write, `evict` runs over the folder's `.jpg` files under a lock. The written file's mtime is set to the app clock, so it is the newest file.
- **H1–H8.**
  - H1: no mutating adapter call anywhere in the queue (C5(a)).
  - H2: only cache folders are removed.
  - H3: no pixel work, the bytes are stored as fetched.
  - H4: `.part` + rename.
  - H6: without commerce every row is local and nothing is queued.
  - H7: core names no platform. The only Shopify strings in core are the test literals rows C5(d) and C13(e) dictate.
  - H8: commands delegate. See the note on the search orchestration placed in `commands/items.rs` by the plan.
- **Agent guide.** `02_architecture.md` and `04_change_impact.md` describe the row-picture rule, the queue, the stale rule, pages, the search cache and `picture_ready`. They are current.

**Tests, row by row** (does it observe the row's outcome at the row's boundary, and would it fail on a wrong one?). Yes for all 67 rows:
- **C1(a)–(g)** assert the exact `(cover_preview_path, cover_pending)` pair from `list` with `RemoteItems` attached.
- **C2**
  - (a) checks the bytes, the `(url, 320)` fetch and that the folder holds exactly that one file (so no `.part` is left).
  - (b) checks the folder holds exactly the `m2` cover.
  - (c) checks the fetch count is unchanged.
  - (d) checks the exact event list.
- **C3**
  - (a) checks that no fetch happens before `run_pending`.
  - (b) asserts `(10 written, 4 in flight)` exactly, so a serial or unbounded run fails.
  - (c) and (d) check the exact fetched URL sets.
- **C4**
  - (a) checks one `url_reads` call with the 3 ids, the refreshed stored URLs, 3 files and exactly 6 fetches.
  - (b) checks the empty URL, no file, the exact Health detail, and no fetch and no read on a second run.
  - (c) checks reads go 1, then 1 at 9 minutes, then 2 at 11 minutes.
  - (d) checks no read.
  - (e) checks `Worked`, the fine cover and no read, then one read and the other cover.
  - (f) checks Health is cleared.
- **C5**
  - (a) compares mutation ops after the setup publish.
  - (b) checks both folders are gone and the read and mutation counts are unchanged.
  - (c) checks Health is cleared.
  - (d) checks the digest is computed with `sha2`.
- **C6** asserts exact tuples from `page_of`. **C13(a)–(c), (e)** assert exact outputs of `evict` and `search_name`.
- **C7**
  - (a)–(h) assert exact id lists, totals, pages, group `(label, rows, total)` and `counts`.
  - (c) and (h) use the 40th newest SKU, and (f) feeds the ids out of order and asserts newest-first.
  - **C13(d)** checks the folder holds exactly the two newest names.
- **C8**
  - (a) checks the IPC response `page 2 / pages 2 / total 30 / 5 rows`, each with a boolean `cover_pending`.
  - (b) checks 30 rows.
- **C9**
  - (a)–(g) assert the exact `sessionsPage` argument lists and the page control's `aria-current`.
  - (c) waits past the debounce and asserts exactly one call.
  - (d) computes the local start of day 6 days back.
- **C10**
  - (a)–(c) check the `<img>` src or the placeholder.
  - (c) also asserts that `sessionsPage` was not called again.
  - (d) checks the hover card's img for both rows, and (e) does the same for the panel.
- **C11, C12(c)–(e)** go through Tauri IPC on a `TestApp`-backed state and check `picture_path` / `session_id`, the file, the `(U, 160)` fetch, the event, the cached second answer, and no `media_urls` call.
- **C12(a)** checks `thumbnail_url` from the schema-grounded fixture and `None` for `sv6`.
- **C12(b)** checks the exact `(external_id, thumbnail_url)` list.
- **C14(a)–(c)** check the img src or the first letter in both dialogs, and the change after `picture_ready` of kind search and of kind cover.

The existing Sessions-page tests changed only their mock (`sessionsPage` answered from the existing `listSessions` mock through `pageFromList`) and their row fixtures (`cover_pending: false`). No assertion changed.

**Assumptions 1–14** (handoff). None contradicts 10H or MC-H1/MC-H2.
- 4: search pictures are queued in their own de-duplicated list. MC-H2.5 does not ask for replacement.
- 8: the empty `picture_path`/`session_id` fields are left out of the JSON. The UI reads an absent field like null.
- 10: the "To review" count is taken from the research overview, as plan U3 task 2 says.
- 12: the gone map is not left on a 10F clearing, because an already-remote item is not cleared again in H2's flows.

The owner-relevant ones are in the narrative's "Choices made for you".

## Findings
None blocking (B1–B4).

## Notes written to backlog
These are in `docs/implementation/backlog_archive/H2_list_pictures_pages_search.md`, the destination the plan's Review mode names.
1. **A held stale cover can be queued twice while the budget is low.** The retries are then duplicated and write the same `.part` file at once. Harmless today. Before H4 adds view pictures to the batch, de-duplicate `stale` by media id. → H4.
2. **The gone-pictures Health item is in memory only, and Refresh from Shopify has no button until H4.** After a restart the Health line disappears while the row stays a placeholder. → H4 decides whether to rebuild it at start.
3. **"To review · N" counts every session, not the tab and search shown.** This is as the plan says, and it may read oddly to the owner. → fold into 10H §5 or change the count.
4. **In "To review" mode, a research update sends the list back to page 1.** → UI.
5. **The search-picture orchestration sits in `commands/items.rs`,** where the plan's U4 task 4 put it. It should move into a core service before the phone remote serves the search. → phone remote (H8).

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1 stale de-duplication | H4 (planner reads the H2 backlog) |
| 2 Health in memory / Refresh button | H4 |
| 3 "To review" count | orchestrator → owner if they ask, else 10H §5 fold |
| 4 page reset in "To review" | backlog (UI polish) |
| 5 search orchestration placement | phone-remote phase |

## Lessons for the plans
- The plan listed the existing Sessions-page tests to convert, but `p18_pending_state`, `p15_publish_ui` and `c4_visit` also render the Sessions page. A grep for `SessionsPage` renders would have listed them.
- The manual try-out of the implementer handoff had a step 6, "Refresh from Shopify clears the Health line". That step cannot be done until H4 adds the command and button. The narrative moves it to "Not yet".

## Probe and side-effect declaration
- No production or test file was edited, not even temporarily (flow.md §4).
- No database or app state was touched outside the test runs' temporary folders.
- Files written by this review: this handoff, the narrative `docs/implementation/narratives/H2_list_pictures_pages_search.md`, the backlog file `docs/implementation/backlog_archive/H2_list_pictures_pages_search.md` (new), and the plan's Tracker line and Review log entry.
- No commit was made; the runner commits these files.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
