---
plan: L3_generate_and_download
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-23
actor: opus (pipeline-reviewer)
---
# L3 — Generate and download price labels · review r1

**Verdict: APPROVED_WITH_NOTES.**
- Production matches 09C §6 and §10, MC-L2.1–.14, and every projection r0 decision (D1–D17).
- All 107 criterion rows have a test at their boundary that fails on a wrong outcome.
- The Review-mode checks are green, and every Review-mode reading check holds.
- H1–H8 hold, and no assumption contradicts the intention.
- No blocking finding. Two notes and one carried-forward note go to the milestone backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- The plan's Tracker line read `IMPLEMENTED` (2026-11-23).
- Checkpoints `00dd4e2f` (U1), `51815be6` (U2), `9acd03f1` (U3), `4ec02089` (U4) and `f7880ee7` (close) are all in `git log`.
- The working tree is clean. `f7880ee7..HEAD` adds only the runner's implementer handoff and review prompt.

## Verified correct
**Perimeter (step 1).**
- `git diff --stat 00dd4e2f^ f7880ee7`: 42 paths. They equal the handoff's Files changed.
- Deviations, both disclosed:
  - `Header.tsx` and `SessionRow.tsx` were listed under U4 but not needed, because `SessionActionsMenu` hosts the dialog. L3·C17(b) passes on the unchanged `Header`.
  - `docs/guide/02_architecture.md` was edited in U4 (flow §7A).
- `crates/infra/src/sqlite/` is untouched: no migration, no column (L3·C8(c)).

**Checks (step 2; the plan's Review mode, once each, one cargo command at a time, none piped or backgrounded).**

| Command | Result |
|---|---|
| `cargo test -p psm-core --lib mechanisms::labels` | 23 passed |
| `cargo test -p psm-infra --lib label_files` | 6 passed (C4(c) runs the real `sips`) |
| `cargo test -p psm-testkit --test suite l3_labels_check::` | 21 passed |
| `cargo test -p psm-testkit --test suite l3_labels_generate::` | 24 passed |
| `cargo test -p psm-testkit --test suite l3_labels_download::` | 7 passed |
| `cargo test -p psm-desktop` | 120 passed; regenerated `bindings.ts` identical (tree clean afterwards) |
| `npx vitest run src/features/labels` | 11 files, 82 passed |
| `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit -p psm-desktop --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | clean |

The implementer's closing-gate stamp stands (workspace suite 1447, npm 935). It is not contradicted, and it covers the shared `FakePricing::send`/`fetch` refactor.

**Rows (step 3).** I read the production code behind every row, then its test.

*U1, the check, preview and files (C1–C4).*
- `check` dedups the ids (first kept), refuses an empty list before any request, and sends one `plan` per `RUN_CHUNK`.
- Chunks are merged by `merge_plans`: positions renumbered over the merged items (D4), sizes merged in first-appearance order, a `NoProducts` chunk turned into `Not found`.
- `NoProducts` is answered only when every chunk returns it. Any other provider error drops the kept check (D10).
- One templates read, and a failure there leaves the product types empty.
- Tests assert the exact request bodies, counts, summaries, groups, sizes (`[{70,100,201},{50,30,100}]`) and errors.
- `preview`:
  - answers `NotFound` before any request;
  - caches the path while the file exists;
  - maps a `raster` error through `reason()` (D2: the inner `io::Error` text);
  - names the picture `<check>-<product id with only [A-Za-z0-9-_]>`.
- `DiskLabelFiles`:
  - `raster` writes `<name>.src.pdf` in `<previews>/labels/`, removes it on every way out, and removes a stale PNG (D1);
  - `save` writes a `.part`, renames it, numbers taken names (symlink-aware), and removes the `.part` on failure;
  - every name passes through `label_file_name`.
- The C4 tests check real files, including that only `x.png` is left.

*U2, generating (C5–C9).*
- The generation is created once, holding the ready ids in plan order. Each chunk's `sm-<uuid v4>` key (39 characters) is made at that moment and stored.
- The retry loop sends the stored ids and key again only for `Unreachable` / `RendererUnavailable`, at most `RUN_RETRIES` = 2 more times.
- `NoProducts` becomes the chunk's outcome. Every other error returns with the chunk still pending, so Try again resumes it with the same key.
- A finished chunk is skipped, and a finished result is answered without a request.
- The `Running` drop guard clears `running` on every way out, including `gate()?` and the `NotFound` returns. On every return path the `kept` guard drops before `_running`, so there is no self-deadlock.
- `follow_run` applies MC-L2.8 under the copy's sequence rule, using the check item's `printed_at` for a reprint (D5), then emits one `LabelsChanged`.
- `merge_runs` follows MC-L2.9 exactly, including the failure order and D3 (the check item's sku and title first).
- The tests compare:
  - bodies and keys (C7(a) compares the JSON bodies for equality);
  - the fake's stored runs and its `label.printed` counts;
  - the copy as seen through `session_labels` with no status request;
  - events;
  - whole `LabelFailure` values.
- C8(a) holds the fake to prove a second `generate` and a `check` answer `Busy` before any request.

*U3, download and commands (C10–C11).*
- `download` finds the artifact in a finished chunk of the kept generation, else `NotFound` with no request.
- It passes `download_url` through untouched to the adapter, which keeps L1's address guard (MC-L2.12).
- A save error becomes `SaveFailed(reason)`. A failed open is logged and answered as `opened: false`.
- Each command is one `*_body` call on `spawn_blocking`. `labels_generate` takes only `check_id`.
- Every `LabelsFailure` maps to its code, with the 09C §10 words.
- The result DTO omits `download_url`.
- The C11 tests compare whole JSON values, millimetres included (`70.0`).

*U4, the dialogs and wiring (C12–C17).*
- `useLabelDialog`:
  - one `labelsCheck` on open;
  - one `labelsPreview` per product shown, stepping in group display order (D15);
  - `inFlight` blocks a second Generate;
  - `leave()` does nothing while `generating`, calls `labelsPartial` after a failed Generate (D16), and closes on its rejection;
  - `labels_off` shows `CONNECT_PRICING` or `connectionLine` (D17).
- `LabelDialog` blocks Escape and outside clicks while generating, and decides one-product versus batch wording by how many ids it was opened with (D11).
- `labelWords` texts match 09C §6.1 / §6.3 and `LABELS.md` §3.3 / §3.5 word for word, including Pricing's "1 products". The date uses the existing `day()` (D7).
- The ⋯ entry:
  - is on for access `allowed` or `unknown` with a product id (D6), and reads Reprint when printed;
  - opens its dialog beside the menu, so it survives the menu closing;
  - on a Labels row without a session, sits in a ⋯ of its own;
  - Generate labels for N opens the dialog with the selection in selection order.
- After a run, `batchClosed` keeps only the selected ids in `failed_product_ids` and calls `refresh()` once.
- The tests assert exact texts, exact mock call lists, disabled states, and the selection plus one `labelsList` call (C17(e)).

**Review-mode reading checks.** All hold:
- `provider.run` has one call site (`label_runs.rs:315`), inside `generate`, with the stored key and ids. No command takes product ids for a run.
- The key is made once per chunk and never logged (`labels.run chunk= of= items= attempt= outcome=`).
- `running` is cleared on every way out.
- The labels folder comes from `paths.home` only in `psm_integrations::labels::files_for`. `TestApp` installs `FakeLabelFiles` in its one constructor path, and its `open` only records.
- No test calls `DiskLabelFiles::open`.
- The service never changes a `download_url`.
- Nothing of a check, run or key is written to the database, and `SqliteStore::migrations()` is unchanged.
- Core names no platform: `sips` and `open -a Preview` are in `psm-infra`, and the port is neutral.
- Only `l1_c19c` and `l2_c13a` changed among earlier tests: both replaced by design, and `l1_c19c` covers both `allowed` and `unknown`. Only additions or the fake refactor touch other shared test files.
- D1, D2 and D3 hold as written. MC-L2.12 is served by L1's rows, as the plan says.

**Assumptions (step 4).** None of the 24 contradicts 09C, MC-L2 or the package. Points I checked:
- 12, the batch footer at 0: this is `LABELS.md` §3.3's own "{summary.markPrinted} products will be marked Printed".
- 10 and 11, which failures offer Try again: they match the "with Try again" column of 09C §10, and D16 for the rest.
- 9, downloads from a partial result: consistent with MC-L2.10 and 09C §6.5.
- 2, D10 only on provider errors: not reachable at a user boundary today, so it is a note, below.

All 24 are told to the owner in the narrative's "Choices made for you".

**Hard constraints (step 5).**
- H1: the only Pricing write is the Generate press on a shown check (M-L2). No `CommerceAdapter` method is called; C10(g) asserts no shop call and no label call on Delete session (M-L5).
- H2: no RAW is touched.
- H3: `sips` rasterises Pricing's label PDF, not a photo, as master plan §3 decided.
- H4: one key per chunk, reused on every retry and Try again. Finished chunks are never resent.
- H5: no UI automation.
- H6: labels are optional.
- H7: core is platform-neutral.
- H8: commands are thin and use `spawn_blocking`.

**Guide (§7A).** `02_architecture.md` §Labels and Frontend, `03_rules_and_principles.md` M-L2/M-L3/M-L4 and `04_change_impact.md` (the L3 command/DTO row and the labels folder row) are current for the changed areas.

## Findings
None blocking (B1–B4): no CHANGES_REQUESTED.

## Notes written to backlog
Written to `docs/implementation/backlog_labels/L3_generate_and_download.md`, the labels track's backlog named by the plan's Review mode (the prompt says `backlog.md`; L1 and L2 used the per-phase file).
1. **D10 only on provider errors.** An empty list, `Off`, `Denied`, `Busy` or a key change leaves the earlier kept check in place. No screen can reach this today (one modal dialog), but a future entry point that keeps an id could generate a check the photographer no longer sees. The fix is one `drop_check()` in each place.
2. **UX.** After "Saved, but Preview couldn't open it", the single-file view's Download PDF saves a second `(2)` copy rather than retrying the open.
3. **Carried forward from L2 r1, still open.** `key_changed()` emits no `LabelsChanged`, so the Labels box keeps its rows after the key is removed. It was routed "→ orchestrator / L3" and L3's plan did not take it.

**Carry-forward dispositions**

| Note | Destination |
|---|---|
| 1 (D10 edge) | the next labels phase that adds an entry point, or a small fix at the orchestrator's choice |
| 2 (Download after a failed open) | the orchestrator; owner-visible polish |
| 3 (L2 key removal) | the orchestrator; next labels phase or a small fix |

## Lessons for the plans
- The plan named two backlog destinations: Review mode names the per-phase `backlog_labels` file, and the reviewer prompt template names `backlog.md`. The runner's prompt template could read the destination from the plan.
- Rows that need a later unit's API (L3·C1(i) and L3·C3(e) assert `generate`, which only exists in U2) make U1's checkpoint commit a partial row. Planning could place such a row in the unit that adds the second call, or split it into its two halves.

## Probe declaration
- No file was edited to probe. Flow §4 turns the mutation regime off, and no production code was touched.
- `cargo test -p psm-desktop` regenerated `apps/desktop/src/services/bindings.ts` byte-identical (`git status` clean afterwards).
- Files written by this review:
  - this handoff;
  - the plan's Tracker line and Review log entry;
  - `docs/implementation/backlog_labels/L3_generate_and_download.md` (new);
  - `docs/implementation/narratives/L3_generate_and_download.md` (new).
- Nothing was committed.
