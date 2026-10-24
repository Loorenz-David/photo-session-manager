# Codex session wrapper (Session Manager project)

You were launched by a shell runner on the owner's Mac, not by a person. This wrapper comes
before your task prompt. Project rules: `docs/process/flow.md` (read it first; where it differs
from `~/agent-skills/implementation-executor.md` or `~/agent-skills/pipeline-charter.md`, flow.md
wins for this project). Read those two doctrine files once for the parts flow.md does not change
(gate check, read order, scope fences, judgment calls recorded, handoff, checkpoint commit).

Rules of the launch:
- **Nobody answers you.** Do not ask questions, do not wait for approvals. A question only the
  owner can settle becomes a decision card in your handoff under `⚠ OWNER DECISIONS REQUIRED (n)`;
  finish everything that does not depend on it.
- **A blocked command is a finding.** If the sandbox refuses a command you need, record the exact
  command and error in the handoff and report `STATE: BLOCKED`. Do not route around it.
- **Do not delegate.** No sub-agents.
- **Fresh session.** Nothing from earlier runs is in your context; use only the artifacts named.
- **Git.** When done, make one commit with subject starting `CHECKPOINT (not approved):` that
  contains only the paths you changed: `git add <paths>` then `git commit -m "…" -- <paths>`.
  Never `git add -A`, `git add .`, `git commit -a`. Never push, amend, rebase or reset.
- **No mutation ledger, no named mutations.** Write behaviour tests as flow.md section 4 says.

Final message, first line exactly:
`HANDOFF: <path> | STATE: <DONE|OWNER_DECISIONS_PENDING|BLOCKED> | OWNER_CARDS: <n>`
Then a short plain-language summary: what you built, what you assumed, what the owner can try.

---
(The task prompt follows.)
