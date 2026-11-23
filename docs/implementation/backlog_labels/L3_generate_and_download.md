# Backlog: L3_generate_and_download (reviewer notes; notes never block, flow.md §5)

- **L3 reviewer r1 (note, D10 edge; handoff assumption 2): a refused check leaves the earlier check kept in some cases.**
  - **What happens.** `check` empties the kept check only when Pricing answers an error. These leave the earlier check (and its generation) in place: an empty list, `Off`, `Denied`, `Busy`, and a key change (`key_changed` does not touch `kept`). A later `generate` of the earlier id would still answer (a finished result) or run against the gate.
  - **Why it does not block.** No screen can reach this today: one modal dialog holds the only check id, and it closes before another check starts.
  - **Why it matters.** D10's literal rule is "a check that fails while an earlier one is kept empties the slot". A future entry point that keeps an id (a second dialog, a phone remote) would generate the products of a check the photographer no longer sees. The fix is one `drop_check()` before the early returns in `check`, plus one in `key_changed`.
- **L3 reviewer r1 (note, UX): after "Saved, but Preview couldn't open it", the single-file done view keeps its Download PDF button.**
  - **What happens.** Pressing it downloads and saves again, as `<name> (2).pdf`. The press does not retry the open.
  - **Why it matters.** A photographer who presses it to "try again" collects duplicate PDFs in Documents › Session Manager Labels. An "Open again" or "Show in Finder" action on the saved path would serve that moment better.
- **Carried forward from L2 reviewer r1 (routed "→ orchestrator / L3", still open): removing or replacing the Pricing key leaves the Labels box rows on screen.**
  - **What happens.** `LabelsService::key_changed()` still emits no `LabelsChanged`. L3's plan did not take this note, so it stays with the orchestrator for the next labels phase or a small fix.
