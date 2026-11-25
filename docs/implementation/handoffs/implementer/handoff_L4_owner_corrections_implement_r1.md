---
plan: L4_owner_corrections
role: implement
round: 1
state: DONE
date: 2026-11-25
actor: claude-opus-5.5 (pipeline-implementer)
---
# Handoff — L4_owner_corrections, implement r1

Gate: intention RATIFIED; `L3_generate_and_download` tracker APPROVED; L4 was NOT_STARTED. Passed.

## What was built
- **Products from the other Mac count as Published (U1, 09C §14.3).** A product that was only adopted from Shopify
  and read once now sits under **Published** with "Archived in Shopify" in the sessions list. It leaves the Open
  count and the Open filter. It moves back to Open only while a publish of it is running, held, failed or cancelled.
  The paged list, the unpaged list and both commands agree. Code: the private `row_status` in
  `crates/core/src/services/sessions.rs`. The label is now computed before the status, from the same `input`.
  `derive_status`, `row_label` and `ready_session_ids` are untouched.
- **The check dialog's cards and layout (U2, 09C §14.1).**
  - The dialog has a fixed height. Only the list of products scrolls. The footer stays at the bottom with the
    sentences, any Generate error and Cancel / Generate PDF. The preview sits beside the list.
  - Each template group has a two-line header: the name, then product types · size. Each line has an ellipsis and
    a tooltip.
  - Each product is a card (`LabelCard`):
    - a 1:1 picture with the SKU under it;
    - a one-line title with a tooltip;
    - the price and the outcome;
    - a warnings row, only when needed: `No price` (amber), `Can't generate: …` (red), `Not published yet` (amber)
      plus a `QR link` pill button that shows or hides the address (clicking the address opens the browser), and
      grey pills for other notes.
  - Clicking a card selects it and shows its preview.
  - Backend:
    - Notes keep their kind and address: `PlanNote {kind, message, link}`. The adapter's `plan_note` reads the
      address out of Pricing's sentence.
    - The check answers a picture per product, using only what is already on this Mac: the session row picture,
      else the cached search picture of the `thumb_url` that the Labels list or Select all remembered. No new call,
      nothing downloaded in `check`.
    - The DTO carries `notes` and `picture_path`.
- **Generate goes straight to Preview (U3, 09C §14.2).**
  - After a run (or after `labelsPartial`), the dialog saves and opens every file itself, in order, each at most
    once. While it does this it cannot be closed and shows "Opening in Preview…".
  - If everything was generated, it closes and shows one toast ("Label generated · opened in Preview" /
    "N labels generated · opened in Preview · k stay Not printed (no price)").
  - Otherwise it stays on the result:
    - the title ("2 of 3 labels generated", which counts not-found products);
    - the result lines;
    - a card per failed product;
    - the not-found lines;
    - "Failed products stay selected…";
    - one line per file: saved and opened / saved but Preview couldn't open it (with its path, no retry) / the
      failure with Try again when it can help;
    - the print hint and **Close**.
  - The Download PDF and Done buttons are gone.

## Tests → criterion rows
| Rows | Test |
|---|---|
| L4·C1(a)–(k) | `crates/testkit/tests/suite/h4_details_only_publish.rs` `l4_c1a_…` … `l4_c1k_…` (h4_c9a replaced) |
| L4·C2(a), (b) | `apps/desktop/src-tauri/src/lib.rs` `l4_tests::l4_c2a_…`, `l4_c2b_…` |
| L4·C3(a)–(d) | `crates/integrations/src/conversations/beyo_pricing/labels.rs` inline `l4_c3a_…` … `l4_c3d_…` (+ `l1_c7c` note assertion now a `PlanNote`) |
| L4·C4(a)–(g) | `crates/testkit/tests/suite/l4_labels_check_cards.rs` `l4_c4a_…` … `l4_c4g_…` |
| L4·C5(a), (b) | `lib.rs` `l4_tests::l4_c5a_…`, `l4_c5b_…` |
| L4·C6(a)–(e), L4·C7(a)–(h) | `apps/desktop/src/features/labels/__tests__/l4_label_check_cards.test.tsx` |
| L4·C8(a)–(d), L4·C9(a)–(h) | `apps/desktop/src/features/labels/__tests__/l4_label_result.test.tsx` |

## Assumptions
- **Status rule placement:** a small private function `row_status` beside `status_input` (a delegated choice).
- **Shop-only fixture in the desktop tests (C2):** an adopted product with three gallery pictures and no condition
  pictures. Its first-open read gives `Remote(ShopOnly)`, which the test asserts.
- **The link rule is implemented verbatim (MC-L2.15).** The only extra is that a note whose code is not
  `url_not_published` never gets a link.
- **The check's pictures read one `SessionsService::list(All, None)`, and only when at least one product matched**
  (MC-L2.16). A failed read is logged at warn and leaves `pictures` empty.
- **Select all now remembers each row's SKU and `thumb_url`.** It did not remember SKUs before. So after Select all,
  opening a row also matches by SKU, as after a list.
- **Cards are not buttons.** A card is a focusable `<li>` with click and Enter/Space handlers. It is not a `button`
  role, because the L3 tests that count the dialog's buttons (`['Cancel', 'Try again']`, `['Close']`) had to keep
  their expectations.
- **Dialog height:** `min(720px, calc(100vh - 4rem))`, applied with `.label-dialog:has(.label-check)` /
  `:has(.label-done)`. WebKit supports `:has`.
- **Tones:** amber uses the existing `--warn` token; red uses `--danger`; grey uses `--hair-strong` / `--muted`. The
  selected card gets a text-colour outline.
- **While files are saved the check view stays visible** with its buttons disabled and the busy line "Opening in
  Preview…". The title stays "Generate label(s)…" until the result.
- **Problem cards in the result view (U3):** with a check item, the card shows that item's picture, SKU, title and
  price. The only pill is the red `Can't generate: <failure message>`, and the outcome reads `Can't generate`.
  Without a check item, the card uses the failure's SKU and title, and the price reads "No price".
- **Try again on a file:** Close stays enabled while it runs (only the automatic saves are "not closable").
- **Not-found lines** also appear in the result view (09C §14.2 "each problem").

## Deviations
- **`LabelCard.tsx` is changed in U3** (a `failure` prop for the result view's problem cards). The plan lists
  LabelCard as a U2 file and names reusing it for U3 as a delegated choice.
- **`l3_label_check_dialog.test.tsx`:** I removed the constants `PRICE` and `T50`, the helper `result` and the
  `labelsPartial` import. Only the removed tests (`l3_c12b`, `l3_c14d`) used them, and `tsc` (noUnusedLocals) fails on
  unused locals. No remaining expectation changed. In `l3_label_done_dialog.test.tsx` (later deleted, as planned) and
  `l3_label_wiring.test.tsx`, only the fixtures gained `picture_path: null`. `l3_c17e`'s mocks needed no change
  (its result has no files).
- **The task prompt asked for actor `codex`.** This session ran on Claude Opus 5.5, so the tracker and this handoff
  name `opus` / `claude-opus-5.5`.
- **The old label-line styles were removed**, because no component uses them now. These are the
  `.label-line-main/-sku/-price/-outcome/-fail/-issue/-note` and `.label-check-list` rules, the old `.label-file`
  grid, and the `@media` line rule.

## Files changed
Write perimeter (all in the three checkpoint commits plus the closing commit):
- U1: `crates/core/src/services/sessions.rs`, `crates/testkit/tests/suite/h4_details_only_publish.rs`,
  `apps/desktop/src-tauri/src/lib.rs`, `docs/guide/02_architecture.md`.
- U2: `crates/core/src/domain/label.rs`, `crates/core/src/services/{labels.rs, label_runs.rs}`,
  `crates/integrations/src/conversations/beyo_pricing/labels.rs`, `crates/testkit/src/fake_pricing.rs`,
  `crates/testkit/tests/suite/{main.rs, l4_labels_check_cards.rs (new)}`,
  `apps/desktop/src-tauri/src/{lib.rs, commands/labels.rs}`, `apps/desktop/src/services/{bindings.ts (generated),
  labels.ts}`, `apps/desktop/src/features/labels/{LabelCard.tsx (new), LabelCheckView.tsx, LabelDialog.tsx,
  labelWords.ts, useLabelDialog.ts}`, `apps/desktop/src/styles/globals.css`,
  `apps/desktop/src/features/labels/__tests__/{l4_label_check_cards.test.tsx (new), l3_label_check_dialog.test.tsx,
  l3_label_done_dialog.test.tsx, l3_label_wiring.test.tsx}`, `docs/guide/{02_architecture.md, 04_change_impact.md}`.
- U3: `apps/desktop/src/features/labels/{LabelCard.tsx, LabelCheckView.tsx, LabelDialog.tsx, LabelDoneView.tsx,
  labelWords.ts, useLabelDialog.ts}`, `apps/desktop/src/styles/globals.css`,
  `apps/desktop/src/features/labels/__tests__/{l4_label_result.test.tsx (new), l3_label_check_dialog.test.tsx,
  l3_label_done_dialog.test.tsx (deleted)}`, `docs/guide/02_architecture.md`.
- Close: `docs/implementation/plans/L4_owner_corrections.md` (Tracker line and Review log only), this handoff.
- Not touched: `crates/core/src/mechanisms/labels.rs` (its `vec![]` fixture still compiles), `derive_status`,
  `row_label`, `ready_session_ids`, `SqliteStore::migrations()`, `LabelsService::download`, `provider.run` call
  sites, `STATUS.md`. No Rust file changed in U3. No mutation-probe files (no mutation regime, flow.md §4).

## Checkpoint SHA
1. U1 `9cda2589`: CHECKPOINT (not approved): L4 U1 shop-only items count as Published
2. U2 `7600a70b`: CHECKPOINT (not approved): L4 U2 check dialog cards, pinned footer, notes with address, a picture per product
3. U3 `5d25b5f3`: CHECKPOINT (not approved): L4 U3 Generate saves and opens every PDF at once; toast or result view
4. Closing commit (tracker, review log, this handoff): the next `CHECKPOINT (not approved): L4 close` after `5d25b5f3`.

## Stamp
Closing gate, run once, one command at a time:
- `cargo test --workspace`: green (suite 1464 passed; desktop 124; core 340 / 270 / 174; integrations 143; others
  green; 0 failed).
- `cargo clippy --workspace --all-targets -- -D warnings`: green.
- `cargo fmt --all -- --check`: green.
- `cd apps/desktop && npm run typecheck`: green.
- `npm test`: 138 files, 948 tests passed.

No pre-existing failures.

## Manual try-out
You need the Pricing key with `labels:read` and `labels:write`. **Generating marks products Printed in Pricing.**
1. Sessions list: open a product from the other Mac once. It now sits under **Published** with "Archived in
   Shopify", and the Open number no longer counts it.
2. Labels box: tick three or more products, then press **Generate labels for N**. The dialog keeps its size, the
   product list scrolls, and the summary with **Cancel / Generate PDF** stays at the bottom.
3. Each product is a card: picture with the SKU under it, a one-line title (hover to see all of it), the price and
   what will happen, and pills only when needed.
4. On a product that is not published yet, press **QR link**. The address appears under the pills. Press it again to
   hide the address. Click the address to open it in the browser.
5. Click another card: its preview shows on the right.
6. Press **Generate PDF**. The PDF opens in Preview, the dialog closes, and a note says "N labels generated · opened
   in Preview". Print with ⌘P at 100 %.
7. Try a batch that includes a product that can't be generated. The files still open. The dialog stays with "2 of 3
   labels generated" and shows the problem as a card. After **Close**, that product is still ticked.
8. From a session's ⋯ menu choose Generate label, then Generate PDF. Preview opens and the note says "Label generated
   · opened in Preview".

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
