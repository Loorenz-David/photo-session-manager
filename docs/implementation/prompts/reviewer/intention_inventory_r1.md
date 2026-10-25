---
plan: intention
role: inventory
round: 1
date: 2026-10-25
---
# Mechanism inventory of the ratified intention

Repository root: `/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager`. Read `docs/process/flow.md` first (project rules), then your
doctrine's gate: `docs/intention/intention.md` must read RATIFIED. Read the whole book
(`docs/intention/book/*.md`) and both evidence docs in `docs/intention/evidence/`.

**Inventory** every load-bearing mechanism (hashes, identity keys, status derivation, order and
position math, reconciliation of the gallery, idempotency keys and resume rules, time handling,
file naming, attribute type mapping, hook delivery). Rank by silent-failure risk. Write a
contract-grade definition for each risky one **into the book as lettered chapters**
(`book/05A_mechanism_contracts.md` and further lettered files as needed; never renumber), add a
row to the map table in `intention.md`, register each contract's invariant in chapter 14 as
`M8…` entries, and add a changelog entry in chapter 16.

**Runtime probes.** VERIFY-1 … VERIFY-10 in `docs/intention/evidence/capture_one_16.8.6.md` are
yours to run against Capture One Pro on this Mac with `osascript -l JavaScript`. Preconditions:
Capture One open with a *session* document and, for capture probes, the camera tethered. You may fire the shutter yourself with the dictionary's `capture` command to produce
captures for the hook probes (flow.md §6A). If a
precondition is missing, record the probe as `UNPROBED` with the exact command to run later and
continue; never fabricate a result. Write results into the evidence doc under each VERIFY item.
Probes may create files only under the Capture One document's folders or `/tmp`; never modify or
delete a RAW.

**Contradictions** you resolve on your own are listed for owner ratification in the handoff.
Anything only the owner can decide is a decision card. Do not reopen product semantics.

Handoff at `/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager/docs/implementation/handoffs/reviewer/handoff_intention_inventory_r1.md` (frontmatter `plan: intention`, `role: inventory`, `round: 1`, `state`,
`date`, `actor`): inventory table (mechanism / risk / contract status), probes table (VERIFY id /
result / evidence), what you added to the book, resolved contradictions, `⚠ OWNER DECISIONS
REQUIRED (n)`. Commit nothing; the runner commits.

Final message, first line exactly:
`HANDOFF: /Users/davidloorenz/Desktop/Developer/Photo-Session-Manager/docs/implementation/handoffs/reviewer/handoff_intention_inventory_r1.md | STATE: <DONE|OWNER_DECISIONS_PENDING|BLOCKED> | OWNER_CARDS: <n>`
