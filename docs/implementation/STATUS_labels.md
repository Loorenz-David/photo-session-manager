# STATUS — price labels from Pricing (book 09C)

Running log of the 09C track on branch `feature/agentic`. The main tracker `STATUS.md` is not edited by this track.

- **2026-11-23:** owner request: generate price labels in Session Manager through Pricing's label API: a Labels box in
  the sessions list (Pricing-backed search and filters, not printed by default), Generate label from every session's
  ⋯, batch generation in the Labels box.
  - Read the Pricing package (`pricing-app/docs/integrations/session-manager/LABELS.md`, `API.md` › Price labels,
    `README.md`; read-only) and the sessions list, 09B and 10H code paths.
  - Plan presented to the owner. Eight cards, all answered as recommended: unmatched products shown as sessions
    (10H shop-only flow); default "has a price, not printed"; PDF saved to a Labels folder and opened in Preview;
    check list plus preview; marker everywhere; Select all N in the Labels box only; quiet updates on price-change
    resets; Pricing's template choice only.
  - Chapter `09C_pricing_labels.md` RATIFIED (changelog A33). Planner next.
  - **Owner action before the first real use:** a Pricing key with `labels:read` and `labels:write` (Replace in
    Settings → Pricing), and the service account bound to the live Beyo store.
  - **Plan written** (`e3c9f70`): L1 labels provider and marker (projection), L2 Labels box, L3 generate and download (projection). 11 units, 50 criteria, 294 rows. Preview picture by macOS `sips` (probed on this Mac). 1 owner card: should the Labels box count show as soon as the list opens (recommended yes); L2 carries `blocked_on: owner card 1`.
  - **L1 started** in the background (`scratchpad/labels_chain.sh`, log `scratchpad/labels_chain.log`).
