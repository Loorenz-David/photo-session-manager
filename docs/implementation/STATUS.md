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
- Phases: none started.
- Run in flight: **none**.

## Needs the owner
Cards verbatim in `handoffs/reviewer/handoff_master_plan_r1.md`:
- **Card 1 — Rust not installed** (verified: no cargo/rustc/rustup). Gate holds: P00 not launched
  until `cargo --version` works. Owner to install, or say so and the orchestrator runs rustup.
- **Card 2 — SKU for a new local item** (typed vs generated with prefix). Recommendation: generate
  with editable proposal. On silence P07 builds the typed box only.
- Optional: 10-second colour-tag visual check.

## Next command
Once `cargo --version` works: `tools/pipeline/run_phase.sh all` (background). Fold card 2's answer
into P07 (plan row) if it arrives before P07.

## Facts the next orchestrator needs
- Shopify dev-app token is not available yet; the owner provides it before the publishing phase.
  Tell the owner when that phase is next in `phases.txt`.
- Codex trust for this repo is set in `~/.codex/config.toml`; `claude -p --agent pipeline-reviewer --model opus` verified working.
