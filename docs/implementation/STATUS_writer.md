# STATUS, publication writer — read first (updated by the orchestrator at every stop)

Branch `feature/agentic`, worktree `~/Desktop/Developer/Photo-Session-Manager-agentic`. Never work in the sibling
checkout `~/Desktop/Developer/Photo-Session-Manager`; never edit `STATUS.md`.
Authority: book `08C_publication_writer.md` (RATIFIED 2026-11-19, changelog A22). Evidence:
`docs/intention/evidence/publication_writer_research.md`. Process: `docs/process/flow.md` §10 (milestones of units),
with the deltas of `master_plan_writer.md` once written. Implementer Claude Opus (`PSM_IMPLEMENTER=claude`), reviewer Opus.

- **2026-11-18:** the owner asked for a publication agent. The draft 08C was shaped with the owner.
  - Research subagent: `evidence/publication_writer_research.md`.
  - Owner answers on 2026-11-19 closed all §13 decisions:
    - "attributed to" is allowed
    - the era field is used as it stands
    - restoration becomes fixed choices
    - the shop name and "Ships from Sweden" go in when they fit
    - the research examples are the start
    - the same provider as research
    - the grade word comes from Detailed condition
    - at most one use sentence
- **2026-11-19:** RATIFIED (`84fae97`). The planner session started: master plan, `phases_writer.txt`, plans `W1…`,
  handoff `handoffs/reviewer/handoff_writer_plan_r1.md`.
- **2026-11-19: plans written** (`1d9b985`; handoff `handoffs/reviewer/handoff_writer_plan_r1.md`).
  - 6 milestones: W1 product note, W2 engine, W3 review, W4 settings, W5 SEO publish (projection), W6
    restoration choices. 17 units, 73 criteria, 231 rows. Contracts WC1–WC8 in `master_plan_writer.md` §5
    stand in for the inventory.
  - **Owner card 1** (Restoration choices only in the app (A, recommended) or as a new Shopify field (B))
    gates W6 only. Relayed.
  - W5 U2 task 1 writes a test SEO title on the probe product `PSM-PROBE`, using `~/.psm_shopify.env` (the owner's
    test store per the 2026-10-26 answer), then restores it. Confirm with the owner that this file still points to
    the test store before W5.
  - **Chain W1→W4 started** in the background: `scratchpad/writer_chain.sh`, log `scratchpad/writer_chain.log`.
- **2026-11-19: owner answers.**
  - Card 1 = **A**: restoration choices stay in Session Manager; the customer reads them in the condition report
    description. W6 unblocked; master plan rule 11 now checks the condition report only.
  - The store in `~/.psm_shopify.env` is now the **real shop**. W5 U2 task 1 amended: the probe creates its own DRAFT
    test product, probes it, deletes it, and touches no existing product.
  - Chain: W1 done, W2 under review (10:36).
- **2026-11-19 12:02: W1–W4 APPROVED** (all APPROVED_WITH_NOTES; narratives in `narratives/W1…W4`). Release build of
  `143369f` installed on the dev Mac (previous build kept as "2026-11-18 A21"); fresh zip on the Desktop.
  - W5 gains **U4, carried fixes** from the W1–W4 reviewer notes: "A 1 cm" read as a model number, findings about
    texts never written, a silent answer, "$0.00" costs, the stuck condition banner, the guide rows and the
    "shopify" constant. Master plan WC1.7, WC5.10(iii) and WC6 rule 2 amended.
  - Chain W5→W6 started: `scratchpad/writer_chain2.sh`, log `scratchpad/writer_chain2.log`.
- **2026-11-19 12:18: W5 projection r0 = AMENDMENTS_REQUIRED** (no owner decisions). The orchestrator applied L1–L22 to
  the W5 plan (Review log entry), updated the master plan counts (18 units / 78 criteria / 238 rows), and set W5's
  `projection:` to `done` so the runner goes straight to the implementer. Chain W5→W6 restarted (`writer_chain2.log`).
- **2026-11-19 13:08: W5 APPROVED** (APPROVED_WITH_NOTES, `2c60497`). Shopify probe on the real shop: unset SEO
  reads blank (no fallback), read-back byte-identical, a 75-character title stored whole, and the draft test product
  deleted (ledger closed).
  - **W6 implementer exited 1 mid-run** (most likely the usage limit). The state is NEEDS_ORCHESTRATOR, "implementer
    state MISSING". Partial, **uncommitted** edits are in `attributes.rs`, `write_table.rs`, `writer/defaults.rs`
    and `writer/postcheck.rs`.
  - **Next:** discard the partial edits (`git checkout -- <those 4 files>`) or let a fresh implementer continue from
    them, then re-run `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase W6_restoration_choices`.
  - After W6: release build plus install, then the PR into main.
- **2026-11-19 13:52: W6 APPROVED** (restarted clean after the stop; the partial edits were saved as
  `scratchpad/w6_partial.patch` and discarded). **All six milestones are approved.**
  - Release build of `19fdc10` installed on the dev Mac (previous build kept as "2026-11-19 W4"); fresh zip on the
    Desktop.
  - PR #21 merged into main (`cf9ded6`).
  - **Follow-ups offered to the owner, not built yet:**
    - The default instruction should say "name each restoration step exactly" (today it says one sentence, so a
      step may be missed and need a rewrite).
    - Restoration words should match as whole words ("soiled" counts as "oiled" today).
    - Open notes in `backlog_writer/W5`, `backlog_writer/W6`.
- **2026-11-19: A23 owner correction (company listing rules).** Grade word off and three more switches (stability
  sentence, title formula, dimensions line), all off by default; `item {type, sku}` in the writer input; Settings →
  Writer **Export… / Import…**. The company's instruction, example, banned words and lengths are in an importable
  file on the owner's Desktop (`Beyovintage writer settings.json`), deliberately **not** in the public repo.
  Damage details + Restoration are the normal restoration source; the Restoration done ticks stay optional.

- **2026-11-19 16:55: A23 installed** (`dc0e19e`; previous build kept as "2026-11-19 W6"); PR #22 merged into main.
  - The Beyovintage writer settings live **outside the repo**, in `~/Desktop/Beyovintage writer settings.json`
    (generator `scratchpad/beyo/make_beyo.py`). The owner imports it with Settings → Writer → Import… on each Mac.
  - The orchestrator removed "This piece" from that file's banned list: the company rule is only "don't start with
    it", and the instruction already says so.

- **2026-11-19: A24 + A25 owner corrections, installed** (`6332967`; previous build kept as "2026-11-19 A23");
  PR #23 merged into main (`3bf5553`).
  - Work-Mac log (Sdt1-191126, run 32): the writer drafted every text, but description ("the photographer cleared
    this field") and condition report ("…accepted this value") were refused by MC-A1, silently. The run did follow
    the imported Beyovintage settings.
  - A24: Write texts / answers are deliberate requests and offer new suggestions over accepted, typed or cleared
    texts (prior kept whole; Keep previous restores). Automatic requests unchanged. The writer's state shows: session
    marker, end toast, failed-run reason with Try again, texts not offered with reasons, "Last written".
  - A25: Customize fields: the Add-a-field box filled the step and hid the field list; it now takes its own height
    and lists matches only while typing.
  - Not yet seen on screen by the orchestrator: the new writer display and the Customize fix in the installed app.

- **2026-11-20: A29 owner correction** (owner's correction brief from the test run of 10 products, `173ab12`).
  - Cause: the texts followed the shipped default rules (pasted restoration words, photo numbers and sizes, the
    " | shop name" suffix, four paragraphs), so the Mac that wrote them was almost certainly not using the
    imported Beyovintage settings.
  - Code: the description is one `text`; the defaults follow the brief's generic rules; a SKU title is a
    placeholder (Ch1 kept its SKU because `adopt` set no title base); the publish title update also moves a
    SKU-derived handle to one made from the title, with a 301 redirect.
  - Company file (outside the repo, `~/Desktop/Beyovintage writer settings.json`, generator
    `scratchpad/beyo/make_beyo_v2.py`; the 2026-11-19 file is kept in "Session Manager old (2026-11-12)"):
    - The brief's rules are added.
    - Five examples: four approved texts from the brief, plus the chest of drawers.
    - Safeguard `field_vs_field` is `ask`; `seo_title_max` is 75.
    - More banned phrases.
  - Next: the owner re-imports the file on the work Mac. The 10 test products in the real shop keep the old
    texts until rewritten, and Ch3's manufacturer is still "Soro".
