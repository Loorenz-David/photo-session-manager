# STATUS — read first (updated by the orchestrator at every stop)

Updated: 2026-10-25 by the orchestrator.

## Where things stand
- Intention: **RATIFIED** 2026-10-24 (`docs/intention/intention.md`).
- Mechanism inventory: **done** 2026-10-25 (exit 10, state OWNER_DECISIONS_PENDING, one card I1).
  20 contracts MC-1…MC-20 in 05A/07A/10A; 17 resolutions R-I1…R-I17 listed for ratification in
  `handoffs/reviewer/handoff_intention_inventory_r1.md`. Scripted capture now WORKS: autofocus first,
  then `capture` (evidence VERIFY-12, 2026-10-25); the inventory's capture-dependent probes are
  still to be run by the first phase that needs them. Colour-tag mapping confirmed by
  the owner 2026-10-25.
- Master plan and 17 phase plans: written 2026-10-25 (exit 10, OWNER_DECISIONS_PENDING, 2 cards).
  Order in `phases.txt` (P00…P16); Shopify phase P12 needs the token.
- Phases: P00–P03 APPROVED (with notes) 2026-10-25. P04 projection r0 → amendments applied by the
  orchestrator (plan Review log); projection marked done so the runner skips a second one.
- Run in flight: **all**, delayed start 20:00 (P04 implement r1 hit the Codex usage limit at 19:30, resets 19:58; partial work uncommitted in the tree, noted in the P04 Review log). P00 Automation prompt check PASSED with the owner 19:25 (evidence packaging_spike.md).

## Needs the owner
Nothing. Card 1: owner said "install it"; rustup installed cargo/rustc 1.99.1 (2026-10-25).
Card 2: generate with editable proposal, and SKU editable after photos → 05B MC-22, R31, M29;
plans P01/P07/P09/P12/P14 + master plan amended. Colour-tag mapping confirmed by the owner (evidence).

## Next command
On exit: per orchestrator.md table. Before P12 is next: ask the owner for the Shopify token.

## Facts the next orchestrator needs
- 2026-10-25 ~15:55: owner took the camera away (not tethered). Capture One stays open on
  `session test 1` on this Mac; the owner will not use Capture One here. Capture-dependent probes →
  UNPROBED; rerun them (autofocus + capture, evidence VERIFY-12) when the camera is back.
- Shopify dev-app token is not available yet; the owner provides it before the publishing phase.
  Tell the owner when that phase is next in `phases.txt`.
- Codex trust for this repo is set in `~/.codex/config.toml`; `claude -p --agent pipeline-reviewer --model opus` verified working.
