# Backlog: L4_owner_corrections (reviewer notes; notes never block, flow.md §5)

- **L4 reviewer r1 (note, UX): a check card for a product without a title shows an empty first line.**
  - **What happens.** Pricing's plan item `title` is nullable (L1·C7(c)). `LabelCard` renders `item.title ?? ''`, so the card's first line is blank and its tooltip is empty. Only the SKU under the picture tells which product it is.
  - **Why it matters.** 09C §14.1 wants "a simple well made card, easy to read". A blank title line looks like a rendering fault. Showing the SKU or "Untitled product" in muted text would read better. This is a one-line change in `LabelCard.tsx`, with a UI test at the card.
- **L4 reviewer r1 (note, UX): the result view's problem cards look clickable but do nothing.**
  - **What happens.** `.label-card` always has `cursor: pointer` and a hover border. In the result view (`LabelDoneView`), a card has no `onSelect` and no tab stop, so the click does nothing.
  - **Why it matters.** This is a small affordance lie on the screen the photographer reads when something went wrong. The fix is to apply the pointer and hover styles only to a selectable card (for example `.label-card[tabindex]`). It is CSS only.
- **Closed by L4: L3 reviewer r1's note "after 'Saved, but Preview couldn't open it', the done view keeps its Download PDF button".** L4 removed Download PDF. A file that was saved but not opened is never asked again (L4·C9(g)), so no duplicate PDF can be made from the dialog.
- **Carried forward again from L2 reviewer r1 (routed "→ orchestrator", still open): removing or replacing the Pricing key leaves the Labels box rows on screen.** L4 did not touch `LabelsService::key_changed()`, which still emits no `LabelsChanged`. It does call `forget()`, so since L4 a key change also empties the remembered picture addresses, together with the SKUs.
