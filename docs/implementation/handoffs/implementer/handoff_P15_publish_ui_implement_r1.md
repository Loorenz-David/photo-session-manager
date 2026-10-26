---
plan: P15_publish_ui
role: implement
round: 1
state: DONE
date: 2026-10-26
actor: claude
---
# P15 implement r1 (FIX2 relaunch): F4 only, batch publish review inside the form dialog

## What was built
- **Batch publish works again.** In the batch queue ("Review & publish N"), pressing "Publish N confirmed" now opens
  the "Review publish" panel as a dialog layered **on top of** the product details form. Before, it opened behind the
  form and could not be clicked, and any click on it closed the whole queue without publishing.
- The panel shows the same contents as before: thumbnails, counts, unreviewed and gallery-removal notes, and the
  summary. "Publish N images" sends exactly the summary on screen for the confirmed sessions only, then closes the queue.
- Cancel (or Escape) closes only the review. The queue and its ✓ confirmations stay.
- Mechanism:
  - `PublishPopover` gained a `modal` prop that renders the same body inside its own Radix `Dialog`.
  - `BatchQueue` mounts it inside the form dialog's `publishSlot`, so in the React tree it is nested in the form
    dialog and Radix stacks it as the top layer.
  - The single-session popover (workspace / header) is unchanged.
- **Scope:** this round fixed F4 only (FIX2_AUTHORISED, plan Review log 08:20). The F1–F3 fixes from checkpoint
  `7ace3d5` were not touched.

## Tests → criterion rows
| Row | Test | Change |
|---|---|---|
| C5(a) | `p15_publish_ui.test.tsx` › `C5(a) publishes only the first two confirmed sessions…` | `FormDialog` is no longer mocked, so the real modal dialog renders. The test presses (pointerdown + click) "Confirm & next" ×2 and "Publish 2 confirmed", then "Publish 4 images" found **by role** inside the `dialog` named "Review publish". It asserts the summary T_AB and the removal note are shown, `publish_confirm([1, 2], 'T_AB')` is called, and the queue closes. **On the pre-fix tree it failed**: the review dialog cannot be reached by role. It passes after the fix. |
| C5(b) | `C5(b) marks a confirmed queue item…` | Uses the real dialog now. It awaits the ✓, because the real "Confirm & next" saves the form first (async). The asserted outcome is unchanged. |
| All other rows | unchanged | The other 9 UI tests and 3 Rust tests are untouched and green. |

## Assumptions
- While "Publish N images" is in flight, the review dialog ignores close requests (Escape / outside press). This
  prevents a half-closed state mid-confirm. It is the smallest choice. The book does not address it.

## Deviations
None.

## Files changed
In checkpoint `c4b056d`:
- `apps/desktop/src/features/publish/BatchQueue.tsx`
- `apps/desktop/src/features/publish/PublishPopover.tsx`
- `apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx`
- `docs/implementation/master_plan.md` (P15 tracker row only)
- `docs/implementation/plans/P15_publish_ui.md` (Review log entry only)

Also this handoff, which is not in the checkpoint (the runner commits it). `FormDialog.tsx` was not touched. Mutation
probes: none (flow.md §4). The pre-fix red came from running the new test before the production edit.

## Checkpoint SHA
`c4b056d`

## Stamp
Tree: `c4b056d` (clean apart from this handoff).

| Command | Result |
|---|---|
| `cargo test --workspace` | 374 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` (apps/desktop, Node 22) | exit 0 |
| `npm test` (apps/desktop) | 6 files, 90 passed |

Environment note: `cargo` is in `~/.cargo/bin`, which is not on the runner shell's PATH. I prefixed
`PATH=$HOME/.cargo/bin:$PATH`.

## Manual try-out
1. Run the app (`npm run tauri -- dev` in `apps/desktop`) with Shopify configured and at least three Ready sessions.
2. In Sessions, tick three rows and click "Review & publish 3".
3. In the form, click "Confirm & next" on the first two items. The strip shows ✓ on both.
4. Click "Publish 2 confirmed". A "Review publish" dialog appears on top of the form, with the summary and notes.
   Click "Cancel": you are back in the queue and both ✓ are still there.
5. Click "Publish 2 confirmed" again, then "Publish N images". The queue closes and the two rows start "Publishing n%".
   The third session is not published.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
