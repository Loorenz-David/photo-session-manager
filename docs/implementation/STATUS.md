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
- Phases: P00–P11 APPROVED 2026-10-25/26 (most with notes; narratives relayed). P04 projection r0 →
  amendments applied by the orchestrator (plan Review log). P00 Automation prompt check PASSED with
  the owner 19:25 (evidence packaging_spike.md).
- P12 (Shopify): projection r0 (2026-10-26 00:18) → AMENDMENTS_REQUIRED, ledger L1–L25 in
  `handoffs/reviewer/handoff_P12_shopify_adapter_projection_r0.md`; applied 2026-10-26 (commit a1cf9b4: P12 59 rows,
  P13 task 0 wiring + C9, master plan, 07A/09 grounding, changelog 17). Only the probe-store line
  (owner card 1) is left in P12's Precondition. No run in flight.
- Implementer and fix sessions run on Claude (owner 2026-10-25: Opus for Codex 6 Sol, Sonnet for
  Codex 6 Luna); flow.md §2 updated; `PSM_IMPLEMENTER=codex` switches back.

## Needs the owner
1. Shopify token (`SHOPIFY_TOKEN`, `SHOPIFY_SHOP`): P12 precondition. Asked 2026-10-26 00:20.
2. P12 projection card 1: is `68c5b4-6a.myshopify.com` a test store or the live shop (probe writes)?
   Branches: test store / live shop writes OK / live shop read-only + separate dev store for writes.
   Recommendation: if live, read-only there plus a dev store. Asked 2026-10-26 00:20.
3. Small: when the shop already has the proposed SKU, should New session suggest the next free number?
   Book 05B MC-22 says no (Create stays off until the SKU changes); P07's narrative implied yes; P13 notes it.
Earlier cards (I1, plan cards 1–2) answered; see changelog and Facts.

## Next command
On exit: per orchestrator.md table. Before P12 is next: ask the owner for the Shopify token.

## Facts the next orchestrator needs
- 2026-10-25 ~15:55: owner took the camera away (not tethered). Capture One stays open on
  `session test 1` on this Mac; the owner will not use Capture One here. Capture-dependent probes →
  UNPROBED; rerun them (autofocus + capture, evidence VERIFY-12) when the camera is back.
- Shopify dev-app token is not available yet; the owner provides it before the publishing phase.
  Tell the owner when that phase is next in `phases.txt`.
- Codex trust for this repo is set in `~/.codex/config.toml`; `claude -p --agent pipeline-reviewer --model opus` verified working.
