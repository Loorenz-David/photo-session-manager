# STATUS — read first (updated by the orchestrator at every stop)

Updated: 2026-10-24 by the shaper session.

## Where things stand
- Intention: **RATIFIED** 2026-10-24 (`docs/intention/intention.md`).
- Mechanism inventory: **not run**. Needs Capture One open with a session document holding at
  least one RAW; camera tethered and awake for the capture probes (owner connects it tomorrow).
- Master plan and phase plans: not written.
- Phases: none.
- Run in flight: **none**.

## Needs the owner
Nothing.

## Next command
`tools/pipeline/run_phase.sh inventory` (background), then `plan`, then `all`.

## Facts the next orchestrator needs
- Shopify dev-app token is not available yet; the owner provides it before the publishing phase.
  Tell the owner when that phase is next in `phases.txt`.
- Codex trust for this repo is set in `~/.codex/config.toml`; `claude -p --agent pipeline-reviewer --model opus` verified working.
