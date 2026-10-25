# STATUS — read first (updated by the orchestrator at every stop)

Updated: 2026-10-25 by the orchestrator.

## Where things stand
- Intention: **RATIFIED** 2026-10-24 (`docs/intention/intention.md`).
- Mechanism inventory: **done** 2026-10-25 (exit 10, state OWNER_DECISIONS_PENDING, one card I1).
  20 contracts MC-1…MC-20 in 05A/07A/10A; 17 resolutions R-I1…R-I17 listed for ratification in
  `handoffs/reviewer/handoff_intention_inventory_r1.md`. Scripted capture now WORKS: autofocus first,
  then `capture` (evidence VERIFY-12, 2026-10-25); the inventory's capture-dependent probes are
  still to be run by the first phase that needs them. Colour-tag visual check
  (evidence/capture_one_16.8.6.md VERIFY near line 100) still owed.
- Master plan and 17 phase plans: written 2026-10-25 (exit 10, OWNER_DECISIONS_PENDING, 2 cards).
  Order in `phases.txt` (P00…P16); Shopify phase P12 needs the token.
- Phases: none approved yet.
- Run in flight: **all**, `tools/pipeline/run_phase.sh all`, relaunched 2026-10-25 15:16 after P00 attempt 2 BLOCKED on rusqlite pin; all Rust pins re-resolved together and npm pins verified (master plan §6.2).

## Needs the owner
Nothing. Card 1: owner said "install it"; rustup installed cargo/rustc 1.99.1 (2026-10-25).
Card 2: generate with editable proposal, and SKU editable after photos → 05B MC-22, R31, M29;
plans P01/P07/P09/P12/P14 + master plan amended. Optional: 10-second colour-tag visual check.

## Next command
On exit: per orchestrator.md table. Before P12 is next: ask the owner for the Shopify token.

## Facts the next orchestrator needs
- Shopify dev-app token is not available yet; the owner provides it before the publishing phase.
  Tell the owner when that phase is next in `phases.txt`.
- Codex trust for this repo is set in `~/.codex/config.toml`; `claude -p --agent pipeline-reviewer --model opus` verified working.
