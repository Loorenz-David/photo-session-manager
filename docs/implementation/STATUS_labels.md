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
  - **L1 projection r0:** AMENDMENTS_REQUIRED, 0 owner cards (13 plan gaps: a test row that could never pass, two existing tests to adjust, Pricing's `lifecycleResult` example vs schema, the ⋯ hint source). Applied (`7ea89a7`). Owner card 1 answered: Labels count shown as soon as the list opens (`49ac9ff`, L2 unblocked).
  - **Chain L1→L3 restarted** in the background.
  - **L1 APPROVED** (`07b96c64`, re-review r2 after one fix: the session page's marker now refreshes on every open). **L2 APPROVED** (`7dff8dda`, first review). The Mac hibernated 16:02–18:21 mid-L1; the run resumed on wake, nothing lost; later runs are wrapped in `caffeinate -i`.
  - **L3 projection r0:** AMENDMENTS_REQUIRED, 0 owner cards (17 ledger rows); applied (`533a2252`). L3 implementation started.
  - **L3 APPROVED** (review r1, APPROVED_WITH_NOTES). **All three milestones approved.** Release build `19380f3d` installed (previous kept as "2026-11-23 10H"); zip for the work Mac on the Desktop. No live Pricing call made by any check; the first real Generate is the owner's.
  - **PR #30 merged** into main (`b131802b`).
- **2026-11-24/25: owner corrections (09C §14, A34)** after the first real use: check dialog cards and pinned footer, Generate saves and opens PDFs in Preview at once (no Download step), shop-only items count as Published. Planned as `L4_owner_corrections` (`530baeb9`; the planner was paused by the usage limit and resumed). Card 2 answered: a brought-back shop-only product moves to Open. **L4 APPROVED** (review r1, APPROVED_WITH_NOTES).
  - **2026-11-25: installed** (build `01753d8f`; previous kept as "2026-11-23 labels"); zip on the Desktop; PR #31 merged into main (`3da976f3`).
