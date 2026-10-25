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
- Master plan and phase plans: not written.
- Phases: none.
- Run in flight: **plan**, `tools/pipeline/run_phase.sh plan`, started 2026-10-25 13:19.

## Needs the owner
Nothing. I1 answered Yes 2026-10-25 → 07B MC-21, R30, M28. Still asked of the owner (not a
decision): 10-second colour-tag visual check. Camera focus setting no longer needed.

## Next command
When the plan returns 0: relay the phase map, then `tools/pipeline/run_phase.sh all` (background).

## Facts the next orchestrator needs
- Shopify dev-app token is not available yet; the owner provides it before the publishing phase.
  Tell the owner when that phase is next in `phases.txt`.
- Codex trust for this repo is set in `~/.codex/config.toml`; `claude -p --agent pipeline-reviewer --model opus` verified working.
