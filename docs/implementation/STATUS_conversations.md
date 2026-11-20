# STATUS, conversations with Pricing — read first (updated by the orchestrator at every stop)

Branch `feature/agentic`, worktree `~/Desktop/Developer/Photo-Session-Manager-agentic`. Never work in the sibling
checkout `~/Desktop/Developer/Photo-Session-Manager`; never edit `STATUS.md`.
Authority: book `09B_pricing_conversations.md` (RATIFIED 2026-11-20, changelog A27). External contract (read-only,
Pricing repo): `~/Desktop/Developer/pricing-app/docs/integrations/session-manager/`. Process: `docs/process/flow.md`
§10, with the deltas of `master_plan_conversations.md` once written.

- **2026-11-20:** the owner asked for conversations with Beyo Pricing (header chat icon, per-session panel, jump
  and back, sound). Plan approved ("perfect, let's implement it") with answers:
  - the Pricing account is already tied to the **real shop** (the package README's "Dev store" is outdated);
  - opening a session from a conversation only **visits** it (the capture folder stays);
  - Mac banners on by default.
- **2026-11-20:** A26 (description switch retired, a plain text box) committed `8fe9dfe`. Book 09B committed
  `78869e5`. Planner session started (master plan, `phases_conversations.txt`, plans C1–C4).
  - Live check after C4 is the orchestrator's, with a test question the owner creates in Pricing (writes to Pricing
    are outward-facing; ask first). The owner pastes the Pricing API key in Settings → Pricing themselves.
