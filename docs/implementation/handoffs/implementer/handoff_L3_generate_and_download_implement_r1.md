---
plan: L3_generate_and_download
role: implement
round: 1
state: DONE
date: 2026-11-23
actor: claude-opus (pipeline-implementer)
---
# L3 — Generate and download price labels · implement r1

## What was built
- **"Generate label" works.** In every session's ⋯ menu (Sessions list rows, the session page, Labels rows) the entry is
  now on when the product is in the shop and Pricing labels are allowed, or not yet known to be refused. It reads
  **Reprint label** when the label is already printed. A Labels row with no session on this Mac gets a ⋯ that holds only
  this entry. In the Labels box, **Generate labels for N** is now on and opens the same dialog for the ticked products,
  in the order they were ticked. Unpublished sessions and a missing or refused key still show the entry greyed out with
  the reason, as before.
- **The check.** The dialog opens at once with "Checking…". Pricing is asked what generating would do (up to 300
  products per request, a bigger selection in several requests shown as one; nothing changes yet). The dialog then
  shows:
  - one section per template, headed "Label 70×100 · Tables, Chairs · 70 × 100 mm" ("No template" last);
  - one line per product: SKU, name, price, and "Will be marked Printed", "Reprint · stays Printed", "No price · stays
    Not printed" or "Can't generate", with Pricing's reason under it;
  - products Pricing does not know, as "<SKU or id> · Not found in Pricing";
  - Pricing's own sentences ("1 products will be marked Printed", …; for one product "The product will be marked
    Printed.", "Printed 4 Nov. Reprinting keeps it marked Printed.", …);
  - on the right, a picture of the real label. The app turns Pricing's PDF into a PNG with macOS's `sips`, shows it on a
    white tile, and uses ‹ › to step through a batch. While it loads, a blank tile of the label's proportions shows;
    if the picture can't be made, a sentence shows in its place and Generate still works.
- **Generate.** **Generate PDF** (or **Reprint PDF** for one printed product) is off when every product fails.
  - One press makes one run key per chunk of up to 300 ready products. Each key is made once and kept in memory with
    its chunk.
  - A dropped connection or a busy renderer is retried twice at once with the same key and the same products. After
    that the dialog says why and offers **Try again**, which continues at the chunk that failed with the same key.
    Chunks already generated are never sent again.
  - While a run is in flight the dialog can't be closed, and a second press, or a new check, does nothing.
  - **Cancel** after a failed Generate shows what was generated so far.
  - After each finished chunk the markers on screen follow at once.
- **Done.** The title reads "Label generated" / "Label not generated" for one product, and "3 labels generated" or
  "2 of 3 labels generated" for a batch. Products the check refused count as failed. The view lists ✓ / ↻ / ○ / ✕
  lines, each failure as "SKU · name" with its message, and "✕ 47 not generated" when a chunk never ran.
  - One file: a single **Download PDF**, which saves it, opens it in Preview and closes the dialog.
  - Several sizes: one row per file (name, label count, W × H mm), each with its own **Download PDF**, plus **Done**.
  - No file: **Close**.
  - The view adds "Print at 100 % (not Fit to page)." and, for a batch with failures, "Failed products stay selected
    so you can fix and retry."
  - After a batch, the Labels box keeps only the failed products that were ticked, and asks Pricing for the page again.
- **Saving.** Files land in `~/Documents/Session Manager Labels/`. The folder is made when missing. Pricing's file name
  is cleaned so the file can only land in that folder. A taken name gets " (2)", " (3)" …. The file is written as
  `.part` and then renamed. If Preview can't open the file, the dialog says where it was saved and stays open. A failed
  save says "Couldn't save the PDF: <reason>." with **Try again**. An expired file says "This file has expired.
  Generate the labels again."
- **Nothing is stored.** The check, the run keys and the result live only in memory and are gone at quit. No
  migration and no column were added.

## Tests → criterion rows
| Rows | Where | Tests |
|---|---|---|
| L3·C1(a)–(i) | `crates/testkit/tests/suite/l3_labels_check.rs` | `l3_c1a_…` … `l3_c1i_…` (C1(i) asserts `generate` → `NotFound` for the earlier id and any id) |
| L3·C2(a)–(e) | same | `l3_c2a_…` … `l3_c2e_…` |
| L3·C3(a)–(g) | same | `l3_c3a_…` … `l3_c3g_…` (C3(e) asserts `generate` → `Ok` after the failed raster) |
| L3·C4(a)–(f) | `crates/infra/src/label_files.rs` inline | `l3_c4a_…` … `l3_c4f_…` (C4(c) runs the real `sips` on a hand-written one-page PDF) |
| L3·C5(a)–C9(e) | `crates/testkit/tests/suite/l3_labels_generate.rs` | `l3_c5a_…` … `l3_c9e_…` (24) |
| L3·C10(a)–(g) | `crates/testkit/tests/suite/l3_labels_download.rs` | `l3_c10a_…` … `l3_c10g_…` |
| L3·C11(a)–(q) | `apps/desktop/src-tauri/src/lib.rs` `mod l3_tests` (command bodies over `TestApp.app.handles()`) | `l3_c11a_…` … `l3_c11q_…` |
| L3·C12(a)–C14(e) | `apps/desktop/src/features/labels/__tests__/l3_label_check_dialog.test.tsx` | 16 tests named by row |
| L3·C15(a)–C16(e) | `…/__tests__/l3_label_done_dialog.test.tsx` | 10 tests named by row |
| L3·C17(a)–(f) | `…/__tests__/l3_label_wiring.test.tsx` | 6 tests named by row |
| Core mechanisms | `crates/core/src/mechanisms/labels.rs` inline | `merge_two_plans_…`, `merge_with_a_chunk_found_nothing_…`, `merge_positions_ignore_…` (D4), `groups_by_template_…`, `label_file_names`, `numbered_names`, `merge_runs_totals_…`, `merge_runs_orders_failures_…`, `merge_runs_takes_sku_and_title_from_the_check_item` (D3) |

Replaced by design (the only changed earlier expectations): `l1_label_menu.test.tsx` `l1_c19c_…` now expects an
enabled `Generate label` for access `allowed` **and** `unknown` (D6); `l2_labels_selection.test.tsx` `l2_c13a_…` now
expects `Generate labels for 2` enabled.

## Assumptions
1. `merge_plans`'s `usize` is the chunk index (answers are ordered by it); an `Err` member lists the chunk's products and
   each becomes `Unresolved {message: "Not found"}`.
2. D10 is applied to provider errors and to "every chunk found nothing". An empty list (`InvalidValue`), `Off`,
   `Denied` and `Busy` leave the kept check alone.
3. `check` answers `Busy` before any request while a generation runs, and checks again just before replacing the kept
   check.
4. `generate` checks, in this order: the check id (`NotFound`), a finished result (answered again without a request,
   even when the connection is now off), `Busy`, nothing ready (`NothingToGenerate`), then the connection (`Off` /
   `Denied`).
5. A `Reprinted` item in the display copy keeps the `has_price` the copy had (unknown when none). MC-L2.8 names only
   status and time.
6. An unrendered run item without an error message gets an empty message.
7. A preview picture's name keeps only `[A-Za-z0-9-_]` of the product id, so it can't leave `<previews>/labels/`.
   Product ids are digits, so `<check id>-1.png` is unchanged.
8. `DiskLabelFiles::raster` removes an older `<name>.png` before running `sips`, so a failed render never leaves a stale
   picture behind.
9. `download` finds the artifact in any finished chunk of the kept generation, so the files of a partial result (Cancel
   after a failed chunk) can be downloaded.
10. The dialog offers **Try again** after a failed Generate only for `labels_unreachable` and
    `labels_renderer_unavailable`. Every other refusal (denied, rejected, bad reply, store gone, busy) offers only
    **Close**, which behaves as Cancel (`labelsPartial`, closes on `not_found`). This extends D16.
11. A download offers **Try again** for `labels_save_failed` and `labels_unreachable`. `labels_expired` shows its
    message only.
12. The batch footer always shows "{mark_printed} products will be marked Printed", also when the number is 0. The
    failing and reprint sentences show only when their number is not 0.
13. One product whose outcome is `fail` gets no footer sentence; its line already says "Can't generate" with the
    reason.
14. One product to reprint with no `printed_at` shows "Label status stays Printed." and "Reprinting keeps it marked
    Printed." (no date).
15. The dialog title before the done view is "Generate label" (one product) or "Generate labels for N" (a batch).
16. Saved-but-not-opened words: "Saved, but Preview couldn't open it: <path>". A file row in a several-file result that
    did open shows "Saved and opened in Preview: <path>". For a single file that saved but did not open, a **Close**
    button appears beside Download PDF.
17. Item `notes` from Pricing are shown in small text under the product's line.
18. The preview placeholder of a product without a template is square.
19. In the Labels box, Escape does not clear the selection while the label dialog is open.
20. A dialog opened from a Labels row's ⋯ refreshes the box when it closes after a run (the batch does the same, per
    MC-L1.9).
21. `sizeText` shows `?` for a millimetre value the bindings type as nullable (specta writes `f64` as
    `number | null`). Pricing always sends one.
22. `FakePricing`: plan positions are the request index; a run marks a product Printed at the run's `createdAt` and logs
    one `label.printed`. The run's `filename` is `labels-<n>-<w>x<h>.pdf` and its artifacts are `art-<n>-<k>`.
23. `FakeLabelFiles` writes a fixed 1 × 1 PNG constant (never displayed in tests).
24. The tracker actor is written `opus`. The prompt template says `codex`, but this session ran on Claude Opus.

## Deviations
- `apps/desktop/src/features/shell/Header.tsx` and `features/sessions/SessionRow.tsx` (U4 Files) were **not changed**:
  `SessionActionsMenu` hosts the dialog, so both get the enabled entry through it (L3·C17(b) passes on `Header`
  unchanged).
- `docs/guide/02_architecture.md` was also edited in U4 (not in U4's Files), because its Frontend line said the ⋯ entry
  had only disabled states (flow §7A).
- L3·C1(i) and L3·C3(e) were committed in U1 without their `generate` half, because U1 has no `generate`. U2 completed
  them as the rows read.
- L3·C8(c) re-runs the L1·C13(f) table and column scan inside the L3 test. `SqliteStore::migrations()` and
  `crates/infra/src/sqlite/` are untouched by L3 (`git diff fcb75cb5 -- crates/infra/src/sqlite` is empty).

## Files changed
Write perimeter (all in the five checkpoint commits):
- **Core:** `crates/core/src/domain/label.rs`, `crates/core/src/mechanisms/labels.rs`, `crates/core/src/ports/labels.rs`,
  `crates/core/src/services/{mod.rs, labels.rs, label_runs.rs (new)}`.
- **Infra:** `crates/infra/src/{lib.rs, label_files.rs (new)}`.
- **Integrations:** `crates/integrations/src/{labels.rs, lib.rs}`.
- **Testkit:** `crates/testkit/src/{lib.rs, fake_pricing.rs, test_app.rs, fake_label_files.rs (new)}`,
  `crates/testkit/tests/suite/{main.rs, l3_labels_check.rs, l3_labels_generate.rs, l3_labels_download.rs}` (new
  stems).
- **Desktop backend:** `apps/desktop/src-tauri/src/{lib.rs, commands/labels.rs}`.
- **Desktop frontend:** `apps/desktop/src/services/{bindings.ts (generated), labels.ts}`;
  `apps/desktop/src/features/labels/`:
  - new: `LabelDialog.tsx`, `LabelCheckView.tsx`, `LabelDoneView.tsx`, `useLabelDialog.ts`;
  - edited: `labelWords.ts`, `LabelMenuItem.tsx`, `LabelRow.tsx`, `LabelsBatchBar.tsx`, `LabelsBox.tsx`;
  - new tests: `__tests__/{l3_label_check_dialog, l3_label_done_dialog, l3_label_wiring}.test.tsx`;
  - replaced expectations: `__tests__/l1_label_menu.test.tsx`, `__tests__/l2_labels_selection.test.tsx`;
  - also `apps/desktop/src/features/sessions/DeleteSession.tsx` and `apps/desktop/src/styles/globals.css`.
- **Docs:** `docs/guide/{02_architecture.md, 03_rules_and_principles.md, 04_change_impact.md}`; this plan's Tracker
  line and Review log; this handoff.
- No archgraph deltas, no mutation probe files (flow.md §4: the regime is off). No migration.

## Checkpoint SHA
1. U1 `00dd4e2f` — the check, the preview picture, the label files.
2. U2 `51815be6` — generating: keys, retries, the result.
3. U3 `9acd03f1` — download and the commands.
4. U4 `4ec02089` — the dialogs, the ⋯ entry and the batch bar wired.
5. Close `f7880ee7` — Tracker IMPLEMENTED and Review log (the handoff is committed by the runner).

## Stamp
Unit commands, all green:
- U1: `cargo test -p psm-core --lib mechanisms::labels` (20, later 23); `cargo test -p psm-infra --lib label_files`
  (6); `l3_labels_check::` (21); `cargo check --workspace`; clippy on core, infra, integrations and testkit.
- U2: `l3_labels_generate::` (24); `l3_labels_check::` (21); clippy on core and testkit.
- U3: `l3_labels_download::` (7); `cargo test -p psm-desktop` (120, bindings regenerated); `npm run typecheck`;
  clippy on core, testkit and desktop.
- U4: `npx vitest run src/features/labels src/features/sessions src/features/shell` (33 files, 197 tests);
  `npm run typecheck`.

Closing gate (once, one command at a time, at `4ec02089`):

| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0, 5 min 45 s; suite 1447 passed, psm-desktop 120, others green; 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npm test` | 137 files, 935 tests passed |

No pre-existing failures were seen.

## Manual try-out
The owner does this on the installed app. It needs a Pricing key with `labels:read` and `labels:write`. **Generating
marks products Printed in Pricing:** for the first try, pick products you mean to label.
1. Sessions → Published → ⋯ of a product with a price → **Generate label**. The dialog shows the template and its size,
   "The product will be marked Printed." and a picture of the label on white. Press **Cancel**: nothing changed.
2. Open it again and press **Generate PDF**. You see "Label generated". Press **Download PDF**: the file appears in
   Documents › Session Manager Labels and opens in Preview (macOS asks once to allow the Documents folder). Print at
   100 %.
3. Back in the list, the row reads "Label printed · today" and its ⋯ now says **Reprint label**. The Labels box's
   number drops by one at the next refresh.
4. In the Labels box, tick three products, one of them without a price, and press **Generate labels for 3**. The check
   lists them by template and says how many will be marked Printed. Generate: the result says what was marked and what
   stays Not printed.
5. A product that cannot be generated (for example no barcode on a barcode template) shows as "Can't generate" with the
   reason. After Generate the title reads "N of M labels generated" and that product is still ticked.
6. A batch over two label sizes ends with two Download PDF rows and **Done**.
7. Turn Wi-Fi off and press Generate PDF. Wait for "Pricing can't be reached right now.", turn Wi-Fi on and press
   **Try again**: the labels are generated once.
8. A session that was never published: its ⋯ shows Generate label greyed out with "Publish first so Pricing can see
   it".

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
