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
  P13 task 0 wiring + C9, master plan, 07A/09 grounding, changelog 17). Owner answers folded in
  (book 09A, R32, changelog 18; P12 68 rows / 9 criteria, P13, P14 21 rows, P16, master plan). P12 APPROVED (with notes) 2026-10-26 06:02 on Claude; narrative relayed.
  P13 APPROVED (with notes) 06:52 on Codex. P14 fix r2 → OWNER_DECISIONS_PENDING (tag merge base);
  answered from 07A MC-13's intent (option A, R33, changelog 19), C2(c) amended + C2(d) (22 rows), attempt archived.
  P14 APPROVED (with notes) 07:29 on Codex. P15 implement r1 (07:37) BLOCKED: failure/upload-state/notification
  data not exposed by commands → plan amended (task 0 backend reads, C7(a)–(c), 14 rows; ch04 S-G aligned with N2,
  changelog 20); attempt to be archived by the orchestrator, next implementer continues from `e3848bd`.
  P15 attempt archived 07:41; relaunch → re-review r2 one MUST (F4 batch panel inert) → FIX2_AUTHORISED in the
  P15 Review log 08:20. Run in flight: **all** (from P15, Codex), started 08:20.
- Implementer and fix sessions: Codex again from P13 (owner 2026-10-26); P07–P12 were built on Claude
  (Opus/Sonnet) while Codex was rate-limited; `PSM_IMPLEMENTER=claude` switches.

## Needs the owner
Nothing. Files scope granted 2026-10-26 (re-check: scope `write_products,write_metaobject_definitions,
write_metaobjects,write_files`; `files` + `products` queries readable; token never printed).
Earlier cards (I1, plan cards 1–2, P12 card 1, SKU suggestion) answered; see Facts and changelog 18.

## Next command
When the owner says the credentials file is filled: check the token request (never print it), then
`tools/pipeline/run_phase.sh all` in the background (resumes at P12; projection is marked done). On exit: per orchestrator.md table.

## Facts the next orchestrator needs
- 2026-10-25 ~15:55: owner took the camera away (not tethered). Capture One stays open on
  `session test 1` on this Mac; the owner will not use Capture One here. Capture-dependent probes →
  UNPROBED; rerun 2026-10-26 08:10–08:20 with the camera: VERIFY-11 (fallback script broken → P16 task 0) and VERIFY-8 capture latency recorded; VERIFY-6 at N≥50 still unprobed.
- Codex trust for this repo is set in `~/.codex/config.toml`; `claude -p --agent pipeline-reviewer --model opus` verified working.
- 2026-10-26 owner answers: (B) the P12 probe store is the owner's **test store**; the real store comes later by
  changing shop + credentials in settings only, so nothing store-specific is hard-coded. (A) Shopify access is a Dev
  Dashboard app: client ID + secret, client credentials grant, token in memory only (09A MC-23); the owner keeps them
  in `~/.psm_shopify.env` (outside the repo). (C) Condition photos go to one metafield named by the setting
  `integration.condition_photos_key`, initial `custom.damage_report`, set by hand (09A MC-24). (D) No answer on the
  next-free-SKU suggestion → kept as written in 05B MC-22: the proposal advances to the next free number
  (P13 row C9(g)). The orchestrator first misdescribed this to the owner; corrected 2026-10-26.
