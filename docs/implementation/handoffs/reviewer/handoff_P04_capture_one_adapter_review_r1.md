---
plan: P04_capture_one_adapter
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-25
actor: opus
---
# P04 Capture One adapter — review round 1

**Verdict: APPROVED_WITH_NOTES.** 0 blocking findings (B1–B4). 21/21 criterion rows hold at their boundaries,
and each test can fail on a wrong outcome. Suite, lint and typecheck are green. 11 notes went to the backlog.
Narrative: `docs/implementation/narratives/P04_capture_one_adapter.md`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker row `IMPLEMENTED` (codex, 2026-10-25). Checkpoint `f21601175dd9b8ef372ba60a9b5801af3b0ff218`
  (`CHECKPOINT (not approved): implement P04 Capture One adapter`) exists. It was resolved through the
  handoff's `git log -1 -- <handoff>` instruction.
- `git show --name-only f216011` gives 55 paths, and the handoff's "Files changed" list gives 55 paths.
  `diff` of the sorted lists is empty, so there are no extra or missing paths. The two out-of-plan files
  (`commands/items.rs` error mapping, `ports/clock.rs` `Arc` forwarding) are declared under Deviations and are
  minimal.

## Evidence (one stamp, flow §4 / master plan §7.2)
Tree: HEAD `938ebb5` = `f216011` plus the review prompt only (`git diff --stat f216011 HEAD` is one docs file).
`git status --porcelain` was clean before and after.
| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0. P04 21/21, P00 16, P01 20, P02 25, P03 28. Unit tests: psm-integrations 4 (incl. `mc19_tag_mapping`, `mc16_process_result`), psm-infra 2 (incl. `mc19_backoff`), psm-desktop 2 (incl. `export_bindings`) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` (Node 22) | exit 0 |
| `npm test` | exit 0 (no frontend test files yet) |
| bindings | regenerated `bindings.ts` equals the committed file (tree clean after the run) |

## Verified correct
Each row: I read the production code first, then the test. Both questions were answered yes: the test observes
the row's outcome at its boundary, and it would fail on a wrong outcome.
- **C1(a)** `CaptureOneAdapter::apply_folder` runs `save`, then `create_dir_all`, then `set_captures`, then
  `get_captures`, then compares by `canon`. Under `private_prefix` a string compare would fail, so the test
  catches it. The fake rejects a set to a missing folder, which enforces mkdir-before-set. The test asserts that
  `save` comes before `set_captures`. `active_session_id` is written only after success.
- **C1(b)/(c)**: on failure the revert re-applies the prior `captures` through the same four steps.
  `capture_folder_not_set` goes into health. `CameraService::status` returns `Disconnected` while the fake has a
  camera, and `capture` is refused with no `capture` op. In (c) the sticky garbage read-back fails the revert's
  read-back too, but the revert's `set` restores the prior folder. A missing revert would leave `K1` or
  `<cwd>/Capture`, and the test would fail.
- **C1(d)** `deactivate` points at `<root>/_Unassigned`, creates the folder and clears `active_session_id`.
- **C2(a)/(b)**: hooks are set as text (VERIFY-2), read back and compared by `canon`. A `null` read-back gives
  `hooks_not_installed`.
- **C3(a)/(b)**: each recipe is handled on its own. An existing recipe gets `overwrite`/`_session_manager`. Each
  missing recipe adds its own `recipe_missing` item (`HealthService::set` keeps several of that code). `render`
  refuses only the purpose whose recipe is missing, and Preview returns the fake's job id.
- **C4(a)/(b)**: the position-1 pick happens in `find_image.js`/`find_image_in_folder.js`
  (`variants.whose({position:1})`). The adapter falls back to the folder collection at `Path::parent(raw)`. The
  tests assert the exact `process:<id>:PSM Preview` op with ids that are not positions (`7`/`3`, `9`/`3`).
- **C4(c), C5(b)**: in `JobRunner::tick`, a retryable error requeues the job as `queued`, `attempts+1`,
  `error=<code>`, `next = now + backoff(attempts)`. The tests assert the full tuple (`queued`, 1, code, 1000).
  `variant_not_found` reaches health.
- **C5(a)/(d)/(e)**: `enqueue_mirror_tag` runs inside the same `transaction` closure in `classify`, `delete` and
  `undo` only; `reorder`/`move`/`copy` are untouched. It coalesces only into a `queued` row for the same target,
  overwrites the payload and resets `next_attempt_at`. The handler reads the row's **current** classification
  (`store.photo(photo_id)`). Read-first skips redundant writes. Writing the classification as it stood at enqueue
  time would produce extra `set_color_tag` ops or a wrong final tag, so the tests catch it.
- **C5(c)/C6(a)**: `mark_own_write` (an upsert into `raw_state`) runs before `set_color_tag`, whatever the
  outcome. A timed-out write returns `Timeout`, the job requeues, and the retry reads first and finds the tag
  applied, so there is exactly one write. A missing stamp fails `query_row`.
- **C7(a)–(c)**: `CameraService` is in core over `CameraProvider` and `HealthService` (H8). Status comes from
  `document_info.camera`. `camera_disconnected` is set on `None`. Capture reaches the provider when healthy.
- **C8(a)**: when `inspect_document` gives `DocumentNotSession`, `EditingStartup::run` returns before
  `install_hooks`.
- **Scripts**: every JXA script checks `running()` before anything else, so the app never launches Capture One.
  `capture.js` implements the VERIFY-12 recipe (autofocus, 2 s, `focusIndicator == "focused"`). `process` output
  goes through `parse_process_result`. Hook read-backs use `.toString()`. `OsaRunner` uses a 30 s timeout with
  cwd set to the data dir, and one `Mutex` serialises calls.
- **Startup** runs on a worker thread. The provider commands (`session_activate`/`deactivate`,
  `camera_status`/`retry`/`capture`) are `async` and release the `App` lock before `spawn_blocking`. This
  resolves the P00 backlog note on `health_get` holding the lock across osascript. `requeue_running_jobs` runs at
  open (07A MC-11).
- **Hard constraints H1–H8**: no violation.
  - H2: no RAW is written; only folders are created under the capture root.
  - H5: only dictionary commands and properties are used.
  - H7: core holds only neutral types (`DocumentState`, `CameraStatus`, `Job`). Variant ids, document kind and
    recipe properties stay in `integrations`. See the note on the `capture_one` health code.
  - H8: commands are thin.
  - Secrets and live services: the suite reaches no Capture One. P00's production `App::open` tests have an
    empty `document_id`, so no startup and no Apple events.
- **Core mechanisms**: the table tests are on the real functions (`tag_for`, `backoff` 1…7 → 1…60,
  `parse_process_result`). No orphan tests.
- **Assumptions**: none contradicts the book. All are reported in plain words in the narrative. The handoff's
  "successful mirror clears `variant_not_found`" is broader than the delegated wording ("when the RAW is later
  found"); see note 1.

## Findings
None blocking.

Reviewer probes: one temporary test file, `crates/testkit/tests/zz_review_probe_p04.rs`. It was created, run
and deleted, and `git status --porcelain` is clean. No production file was touched.
1. RAW A has no variant and RAW B mirrors fine in the same tick. Result: health has no `variant_not_found`.
   This is a note, not B1: the warning returns at A's next retry (≤ 60 s), and no row covers it.
2. Activate while the document is closed. Result: `Err(DocumentNotOpen)`, no health item, the previous session
   stays active, and a capture after reopening lands in the previous (still active) session's folder. This is
   consistent with M25, so it is a note.

## Notes written to backlog
11 rows, phase `P04 → …`, in `docs/implementation/backlog.md`:
1. `variant_not_found` is cleared by any successful mirror (confirmed by probe).
2. `activate` returns `document_not_open` when inspection fails, instead of the plan wording's
   `capture_folder_not_set`.
3. The MC-18 revert re-applies Capture One's reported folder without checking it is under the capture root.
4. `JobRunner` leaves a job `running` when its kind has no handler.
5. Spool drain and provider jobs share one thread, so a hung Capture One delays hook consumption.
6. `render` calls `recipe_exists` on every render (one extra round trip).
7. A core service writes a health code named `capture_one` (H7 hygiene).
8. The JXA scripts have not run against the real Capture One. Still to verify: `documents.byId().camera()`,
   `collections.whose({file: Path()})` (VERIFY-11), and recipes as document elements.
9. Startup's folder switch can interleave with a user's `session_activate`.
10. The fake's `find_image_in_folder` ignores its folder argument (test).
11. Health shows startup state only. "Test Capture One" no longer probes, and duplicate React keys appear for two
    `recipe_missing` rows.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1, 3, 5, 8, 9 | P05 (capture arrival; the first live use of the adapter) |
| 4 | P05/P06 (new job kinds) |
| 6 | P06 (render latency) |
| 2, 11 | P07 (shell/UI wording, health re-check) |
| 7, 11 | P16 (health codes) |
| 10 | next phase touching the fake's folder collection (P05) |

## Lessons for the plans
- A delegated choice phrased as "smallest option consistent with MC-19" (clearing `variant_not_found`) still left
  room for a broader implementation. If it matters, state the scope ("for that RAW").
- The plan's Task 5 names the error for a failed `set_capture_folder` but not for an earlier failure (document
  not inspectable). The next plan that touches activation should state the code for each failure point.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P04_capture_one_adapter_review_r1.md` (new)
- `docs/implementation/narratives/P04_capture_one_adapter.md` (new)
- `docs/implementation/backlog.md` (11 rows appended)
- `docs/implementation/master_plan.md` (P04 tracker row only → `APPROVED`)
- `docs/implementation/plans/P04_capture_one_adapter.md` (Review log entry appended)
- Probe file `crates/testkit/tests/zz_review_probe_p04.rs` was created and deleted (never committed). Its build
  artefacts are only under the ignored `target/`.
- No production code, no database outside temp dirs, no commits.
