# Backlog — W5_seo_publish (reviewer notes, never blocking)

Written by the review r1 (2026-11-19, Opus). Each line: what, why it matters, where it should land.

- **A search title over Shopify's suggested length is published as written.** The probe showed Shopify stores a
  75-character search title whole, with no `userErrors` (evidence `shopify_admin.md` §SEO search texts). The writer's
  own limit (`seo_title_max`, 60) is the only guard, and it only flags the writer's text as `low`; a title the owner
  types or keeps has no limit at publish, and search engines cut it. → owner (the narrative states it); a later
  form hint if the owner wants one.
- **`LINK_INTEGRATION = "shopify"` is now `pub` in core and read by the research crate.** The literal sat in core
  before W5 (a pre-existing H7 slip: a platform name in a core module); U4 made one constant of the three copies, as
  the plan asked, but it now has a second crate depending on it. → a tidy phase that moves the link-integration name
  into configuration or the adapter (H7), together with the other `"shopify"` literals in `services/publish.rs`.
- **A stale doc comment stacks over `DIMENSION_LETTERS`** (`crates/research/src/writer/postcheck.rs` ~l.187): the old
  line "Single letters that are dimensions, never model numbers" stays above the new doc that adds A and I. Harmless;
  misleading to a reader. → next edit of `postcheck.rs`.
- **The Answer toast shows only when the reply carries a reason.** `WriterBlock` toasts on
  `!queued && reason`; a reply `{queued: false, reason: null}` stays silent. The command always sends a reason today,
  so this is a guard against a future change, not a defect. → none (recorded).
- **Resolved here from W3/W4 notes:** an answer that queues nothing now says why (toast); sub-cent writer costs show
  "< $0.01"; guide row "The research runs list…" names `browse`, `look` and `write`; guide row "A writer command or
  `WriterStatus` field" lists `WriterSection.tsx` / `WriterInstruction.tsx` and `src/features/settings`. The entries in
  `W3_writer_review.md` / `W4_writer_settings.md` can be marked resolved by the orchestrator.
