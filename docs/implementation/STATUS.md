# STATUS — read first (updated by the orchestrator at every stop)

Updated: 2026-10-25 by the orchestrator.

## Where things stand
- Intention: **RATIFIED** 2026-10-24 (`docs/intention/intention.md`).
- Mechanism inventory: **done** 2026-10-25 (exit 10, state OWNER_DECISIONS_PENDING, one card I1).
  20 contracts MC-1…MC-20 in 05A/07A/10A; 17 resolutions R-I1…R-I17 listed for ratification in
  `handoffs/reviewer/handoff_intention_inventory_r1.md`. Capture probes UNPROBED: camera never
  released the shutter on 9 scripted captures (likely focus priority). Colour-tag visual check
  (evidence/capture_one_16.8.6.md VERIFY near line 100) still owed.
- Master plan and phase plans: not written.
- Phases: none.
- Run in flight: **none**.

## Needs the owner
**I1 — Retouch after publishing.** (verbatim card in the inventory handoff.) Recommendation: Yes.
On silence the ratified rule (no replacement) is built. Owner asked to stop on non-success, so
planning waits for the owner's word.
Also asked of the owner (not decisions): set the camera to fire without focus (manual focus or
release priority); 10-second colour-tag visual check.

## Next command
After the owner answers I1 (fold a Yes into a lettered chapter + changelog, or keep the ratified rule): `tools/pipeline/run_phase.sh plan` (background), then `all`.

## Facts the next orchestrator needs
- Shopify dev-app token is not available yet; the owner provides it before the publishing phase.
  Tell the owner when that phase is next in `phases.txt`.
- Codex trust for this repo is set in `~/.codex/config.toml`; `claude -p --agent pipeline-reviewer --model opus` verified working.
