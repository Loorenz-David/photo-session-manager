# Session Manager — master plan (v1, 2026-10-25)

Planner: Claude Fable 5.1 (`pipeline-planner`, implementation-planner doctrine under `docs/process/flow.md`).
Thin by design: shared truths are stated once here; everything phase-local is in `plans/<phase>.md`.

## 1. Goal
Build the Session Manager MVP described by the intention book (`docs/intention/intention.md`, RATIFIED
2026-10-24): a Tauri + Rust macOS app in which a photographer shoots through Capture One, sorts and orders
photos, sees live adjusted previews, fills item data, exports locally, and (optionally) publishes to Shopify
with resumable, confirmed runs. This plan never restates product semantics: where a sentence here and the book
differ, the book wins and this file is amended.

## 2. Sources of truth
| Content | Artifact |
|---|---|
| Product semantics, hard constraints H1–H8, domain, scenarios | `docs/intention/book/01…13` |
| Mechanism contracts MC-1…MC-26 (exact rules) | `book/05A_mechanism_contracts.md`, `book/05B_sku_generation_and_change.md`, `book/09A_shopify_credentials_and_condition_field.md`, `book/07A_publish_contracts.md` (+ `07C` grounding fold-back), `book/07B_retouch_after_publish.md`, `book/10A_capture_one_contracts.md`, `book/10B_shape_on_arrival.md` (MC-25, owner 2026-10-29; **retired 2026-11-04 by 10D**), `book/10C_capture_one_setup_by_the_app.md` (MC-26, owner 2026-11-03), `book/10D_composition_and_framing.md` (MC-27…MC-29, owner 2026-11-04; amended 069eca4 after VERIFY-15) |
| Measurement ledger M1–M35 (root of every criterion's trace; M31 retired) | `book/14_measurement_ledger.md` |
| Capture One facts and VERIFY probes | `docs/intention/evidence/capture_one_16.8.6.md`; dictionary `docs/reference/capture_one_16.8.6.sdef` |
| Design rules (field kinds, timings, keyboard, components) | `docs/intention/evidence/design_handoff.md`; pictures/prototype `claude_design/` |
| Shopify facts | `docs/intention/evidence/shopify_admin.md` — **does not exist yet**; written in phase P12 (SHOPIFY-VERIFY-1…7) |
| Process (roles, tests, verdicts, fix budget, narratives) | `docs/process/flow.md` |
| Skeleton: naming registry, environment, standing rules, tracker | this file |
| Phase goal / tasks / criteria / Review log | `docs/implementation/plans/<phase>.md` |
| Execution order consumed by the runner | `docs/implementation/phases.txt` |

**Fold-back rule.** A semantic change amends the book (lettered chapter + changelog entry); a skeleton or
environment change amends this file; a phase-local change amends its plan file. Nothing is patched into a
downstream artifact to paper over an upstream one.

## 3. Roles, workflow and standing rules
`docs/process/flow.md` is imported **by reference** and wins over `~/agent-skills/` doctrine. Project specifics,
restated so no session has to infer them:
- **No tester role.** The implementer writes the behaviour tests; the reviewer audits them (flow §2, §4).
- **No named mutations, no mutation ledgers, no `executed == declared` gates.** Reviewers never edit production
  code; in doubt they re-run a test with a changed input (flow §4).
- **One behavioural test per criterion row at the row's boundary** (service call, command handler, produced
  file, stored state, calls a fake adapter received). Unit tests only for functions listed under a plan's
  **Core mechanisms**. Never internals, call counts of internal collaborators, generated SQL, snapshots.
- **Blocking only B1–B4** (flow §5): B1 behaviour contradicts book/plan at an observable boundary; B2 a row has
  no test at its boundary or its test cannot fail; B3 H1–H8 / secrets / live-services violated; B4 build, suite,
  clippy or typecheck red. Everything else is a note in `docs/implementation/backlog.md`.
- **Fix budget: one automatic fix, then triage** (MUST/DROP); a second fix only with `FIX2_AUTHORISED:` in the
  plan's Review log (flow §5).
- **No questions during a phase** (flow §6): open choices → smallest option consistent with the cited chapter,
  recorded under **Assumptions**; `BLOCKED` only for a plan/book contradiction or an impossible task.
- **Narratives** (flow §7): the approving reviewer writes `docs/implementation/narratives/<phase>.md` from the
  plan's **Manual try-out** section.
- **Commits** (flow §8): implementer/fix sessions make `CHECKPOINT (not approved):` commits of their perimeter
  only; the runner commits prompts/handoffs/state and the `APPROVED <phase>` commit. Nobody pushes but the
  orchestrator.
- **Hardware** (flow §6A): sessions may fire the shutter through Capture One for manual checks; **the automated
  suite never needs the camera, Capture One, Shopify or the network** (charter rule 9). Fakes are in §7.5.
- **Perimeter**: a phase touches only the files its plan lists (plus the test file(s) it names). A new file not
  listed is declared in the handoff's **Deviations**.
- **Trace conventions** (charter trace chain, flow §9): every criterion row's trace cell names an `M` id
  (M1–M35; M31 retired), an `MC` id (MC-1…MC-29; MC-25 retired) or a hard constraint (H1–H8). A row may instead cite a chapter rule
  (`ch07 §Open filter`, `evidence/design_handoff §Field kind map`) **only** for UI-rendering rows, operations
  rows the inventory listed as "not contracted", and packaging rows (ch11); such a cell is prefixed `[rule]` so
  the reviewer knows the row has no ledger measurement behind it. Every plan file derives its own counts of
  criteria, rows and `[rule]` rows.
- **Migrations are never rewritten once a checkpoint commit contains them** (charter rule 7); add the next
  numbered file.
- **Secrets** (09A MC-23): the Shopify client ID and client secret live only in `settings.json` (mode 0600), or in
  the env vars `SHOPIFY_SHOP`, `SHOPIFY_CLIENT_ID`, `SHOPIFY_CLIENT_SECRET` during the P12 probe. The access token
  (client credentials grant) lives in memory only. None of them ever appears in logs, fixtures, commits or handoffs.
  The owner keeps the probe variables in `~/.psm_shopify.env` (outside the repo, never copied into it); the
  session that runs the probe loads it with `set -a; . ~/.psm_shopify.env; set +a` before task 0.

## 4. Progress tracker (one row per phase; agents update only their own row)
| Phase | State | Date | Actor | Note |
|---|---|---|---|---|
| P00_spike | APPROVED | 2026-10-25 | opus | approved with notes: 16/16 rows hold, suite/lint/typecheck green; bundle Automation prompt still to be confirmed by the owner |
| P01_storage_identity | APPROVED | 2026-10-25 | opus | approved with notes: 20/20 rows hold at their boundaries, suite/lint/typecheck green; 5 notes in backlog |
| P02_review_order | APPROVED | 2026-10-25 | opus | approved with notes: 25/25 rows hold at their boundaries, suite/lint/typecheck green; 3 notes in backlog |
| P03_session_state_list | APPROVED | 2026-10-25 | opus | approved with notes after fix 1: 29/29 rows hold at their boundaries, C1(l) now discriminates the latest publish item; suite/lint/typecheck green; 5 notes in backlog |
| P04_capture_one_adapter | APPROVED | 2026-10-25 | opus | approved with notes: 21/21 rows hold at their boundaries, suite/lint/typecheck green; 11 notes in backlog; live Capture One smoke still unrun |
| P05_capture_arrival | APPROVED | 2026-10-25 | opus | approved with notes after fix 1: 33/33 rows hold, capture arrivals now first-sight per RAW across sessions (C7(b) discriminates); suite/lint/typecheck green; 9 notes in backlog |
| P06_live_previews | APPROVED | 2026-10-25 | opus | approved after fix 1: F1 (later row of a rendered RAW gets the path) and F2 (sidecar layout moved to the Capture One adapter) resolved; 18 rows + F1 case hold; suite/lint/typecheck green; 9 notes in backlog from r1 |
| P07_shell_sessions_ui | APPROVED | 2026-10-25 | opus | approved with notes after fix 1: F1 resolved (launch opens the session the backend still captures into, so screen and capture folder agree); 19 rows + F1 cases hold; suite/lint/typecheck green; 6 notes from r1 + 1 in backlog |
| P08_workspace_ui | APPROVED | 2026-10-25 | opus | approved with notes after fix 1: F1 resolved (a capture arriving in the open session becomes the stage photo, and A/C/⌫ act on it; candidate C9 discriminates); 22 rows + C9 hold; suite/lint/typecheck green; 4 notes from r1 + 1 in backlog |
| P09_local_export | APPROVED | 2026-10-25 | opus | approved with notes, review r1: 10/10 rows hold at their boundaries; perimeter = declared files; suite/lint/typecheck green; 5 notes in backlog |
| P10_attributes | APPROVED | 2026-10-25 | opus | approved with notes, review r1: 31/31 rows hold at their boundaries; perimeter = declared files; suite/lint/typecheck green; 7 notes in backlog (title binding for P11/P14 first) |
| P11_form_ui | APPROVED | 2026-10-26 | opus | approved with notes after fix 1: F1 resolved (the form's Title is `items.title`, one home; list, header, export and form agree); perimeter = allowed files; 30 rows + F1 test hold; suite 236 Rust / 78 UI, clippy/fmt/typecheck green; 5 notes from r1 + 3 in backlog |
| P12_shopify_adapter | APPROVED | 2026-10-26 | opus | approved with notes, review r1: 68/68 rows hold at their boundaries; perimeter = declared files (+ disclosed `items.rs` match arms); suite 309 Rust / 78 UI, clippy/fmt/typecheck green; no credential in HEAD; 8 notes in backlog (condition-attribute read-back for P14 first) |
| P13_publish_runs | APPROVED | 2026-10-26 | opus | approved with notes after fix 1: F1 resolved (a ready nameless media no longer stalls `upload`; reconciliation removes it); F2 resolved (`publish_step` runs on its own worker; the shared loop excludes it); perimeter = allowed files; suite 345 Rust / 79 UI, clippy/fmt/typecheck green; 11 notes from r1 + 1 in backlog |
| P14_publish_update | APPROVED | 2026-10-26 | opus | approved with notes after fix 1 (re-review r2): F1 resolved (clearing the last tag in the form removes the app's tags from the shop, admin tags kept, base → `[]`); perimeter = allowed files; suite 371 Rust / 79 UI, clippy/fmt/typecheck green; 22/22 rows hold; notes from earlier rounds in backlog, none added |
| P15_publish_ui | APPROVED | 2026-10-26 | opus | review r1 of fix 2 (`c4b056d`): APPROVED_WITH_NOTES; F4 resolved (review dialog nested in the form dialog; C5(a) red on pre-fix code, green now); F1–F3 settled earlier; suite 374 Rust / 90 UI, clippy/fmt/typecheck green; 1 backlog note |
| P16_operations | APPROVED | 2026-10-26 | opus | review r1 APPROVED_WITH_NOTES: 12/12 rows hold at their boundaries; suite 387 Rust / 92 UI, clippy/fmt/typecheck green (re-run by reviewer); C6(a) re-run live by reviewer → "130", still browsing Capture; no secret in the checkpoint diff; 10 backlog notes, none blocking |
| P17_real_run_fixes | APPROVED | 2026-10-26 | opus | approved with notes after fix 1 (re-review r2): F1 resolved (a settings rebuild no longer requeues or re-enqueues jobs the live workers own; real starts keep crash recovery); perimeter = allowed files (`43adbee`: 2 paths); 16/16 rows + F1 test hold; suite 406 Rust / 92 UI, clippy/fmt/typecheck green; 4 notes from r1 + 1 in backlog |
| P18_pending_state_ui | APPROVED | 2026-10-26 | opus | approved with notes after fix 1 (re-review r2): F1 resolved (shop hits keyed to the query that produced them in both dialogs; new C3(c) cases red on pre-fix code by reading); perimeter = allowed files (`bcd4f51`: 3 paths); suite 407 Rust / 102 UI, clippy/fmt/typecheck green; 5 notes from r1 + 1 in backlog |
| P19_navigation_and_drop | APPROVED | 2026-10-29 | opus | review r1 APPROVED_WITH_NOTES: 9/9 rows hold at their boundaries (8 tests red on the pre-phase tree, C1(d) a regression guard that already held); perimeter = declared files (`3058be8`: 13 paths); suite 407 Rust / 111 UI, clippy/fmt/typecheck green (re-run by reviewer); 4 backlog notes; drop animation not verified live (owner try-out step 3) |
| P20_drop_continuity | APPROVED | 2026-10-29 | opus | review r1 APPROVED_WITH_NOTES: 3/3 rows hold at their boundaries; perimeter = declared files (`63e5899`: 7 paths); suite 407 Rust / 114 UI, clippy/fmt/typecheck green (re-run by reviewer); 2 backlog notes (gliding copy keeps its pre-drop number/Cover badge, so the flushSync claim does not hold; the glide is not verified live) |
| P21_backend_robustness | APPROVED | 2026-10-29 | opus | review r1 APPROVED_WITH_NOTES: 14/14 rows hold at their boundaries; perimeter = declared files + disclosed `paths.rs` (`e33ddfb`: 17 paths); suite 421 Rust / 114 UI, clippy/fmt/typecheck green (re-run by reviewer); 2 reviewer probe tests (deleted) confirmed the post-publish backup half and an incomplete-block save; 4 backlog notes |
| P22_frontend_tidy_up | APPROVED | 2026-10-29 | opus | re-review r2 APPROVED_WITH_NOTES: F1 resolved (`2f5308b`: shared `projectedSlot` for overlay and `useDrop`; C3(b) extended with a hover over a to-review tile, `03` then `reorderPhoto(1, 3)`); perimeter = allowed files; suite 421 Rust / 124 UI, clippy/fmt/typecheck green (re-run by reviewer); r1's 5 backlog notes stand, none new; overlay and glide not verified live (owner try-out step 3) |
| P23_shape_on_arrival | APPROVED | 2026-10-29 | opus | re-review r2 APPROVED_WITH_NOTES: F1 and F2 resolved (`5d2210d`: untouched check accepts `null`; maximum size read as `x+width`/`y+height`; C1 harness uses the live values); perimeter = allowed files (2 code + tracker + Review log); suite 432 Rust / 127 UI, clippy/fmt/typecheck green (re-run by reviewer); live C6(a) re-run on Capture One 16.8.6 passes (applied Square 3987×3987, repeat skipped, restored `[null,5981,3987]`); r1's 6 backlog notes stand, none new |
| P24_capture_one_provisioning | APPROVED | 2026-11-03 | opus | re-review r2 APPROVED_WITH_NOTES: F1 and F2 resolved (`79e5c44`: p00 c1b/c4d over the fake with startup off, no test builds the real `App`; `open_document.js` makes the `PSM Open Window` throwaway then opens, C4(b) asserts the ordered calls); perimeter = the 4 allowed files; suite 445 Rust / 129 UI, clippy/fmt/typecheck green, Capture One state identical before and after the suite; live: new `open_document.js` reopened a probe session beside the owner's document; 1 new backlog note (silent `open` failure leaves the throwaway) |
| P25_tidy_up_after_provisioning | APPROVED | 2026-11-04 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 30 declared paths (`b2f2ce8`); suite 457 Rust / 129 UI, clippy/fmt/typecheck green (reviewer run on clean `5c1f62e`); all 12 rows have a boundary test that fails on the wrong outcome; H1–H8 hold; no live Capture One/Shopify call; 4 new backlog notes, 10 backlog rows marked resolved |
| P26_frontend_leftovers | APPROVED | 2026-11-04 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 7 declared paths (`791446e`); suite 457 Rust / 131 UI, clippy/fmt/typecheck green (reviewer run on clean `033791e`); both rows have a boundary test that fails on the pre-change code (scratch worktree); H1–H8 hold; 4 new backlog notes, 2 backlog rows marked resolved |
| P27_retire_shape_on_arrival | APPROVED | 2026-11-04 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 48 declared paths (`463cb2e`); suite 456 Rust / 131 UI, clippy/fmt/typecheck green (reviewer run on clean `7fbe1d3`); all 13 automated rows have a boundary test that fails on the wrong outcome; C6(a) run live by the reviewer on scratch recipe names (both shapes read back = spec, second ensure `changed []`, recipes list restored); H1–H8 hold; 4 new backlog notes |
| P28_composition_foundation | APPROVED | 2026-11-04 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 56 declared paths (`9b24cbe`); suite 481 Rust / 131 UI, clippy/fmt/typecheck green (reviewer run on clean `345ff13`); all 25 rows have a boundary test that fails on the wrong outcome; geometry property reads checked live read-only on Capture One 16.8.6; H1–H8 hold; 4 new backlog notes (invalid hand-edited `default_shape` breaks session views, second row on a previewed RAW stays frameless, → P29) |
| P29_master_and_output_render | APPROVED | 2026-11-04 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 55 declared paths (`1d43c22`); suite 508 Rust / 132 UI, clippy/fmt/typecheck green (reviewer run on clean `b78cbae`); all 27 rows have a boundary test that fails on the wrong outcome; renderer read against the render contract (turn/flip index maps, straighten sign, linear light, Lanczos3, 4:4:4, allowlist EXIF); H1–H8 hold; 4 backlog notes (queued output renders a photo flagged meanwhile, master_ready error leaves a run stuck → P30) |
| P30_outputs_in_export_and_publish | APPROVED | 2026-11-04 | opus (pipeline-reviewer) | re-review r2 APPROVED_WITH_NOTES: fix perimeter = the allowed files (`c96cbd9`: `services/publish.rs`, P30 test file + tracker/Review log); F1 resolved as corrected (staged/uploaded/attached link older than the staged file is replaced; crash-then-reframe test bites on the r1 code); C8(a) `-r1` name a sound declared divergence; suite 529 Rust / 132 UI, clippy/fmt/typecheck green (reviewer run on clean `c54508c`); triage: no blocking items; 2 backlog notes |
| P31_compose_mode | APPROVED | 2026-11-05 | opus (pipeline-reviewer) | re-review r2 APPROVED_WITH_NOTES: fix perimeter = the allowed files (`f937a9c`: `useCompositionSaves.ts`, `ComposeMode.tsx`, `useComposeGesture.ts`, `p31_gestures.test.tsx` + tracker/Review log); F1 and F2 resolved as corrected (stage no longer snaps back while a newer save waits; Esc closing a menu no longer reverts); suite 529 Rust / 223 UI, clippy/fmt/typecheck green (reviewer run on clean `24bd3d6`); triage: 1 new item DROP (the layer guard also mutes Esc/Return while the non-modal item card or a header popover is open → P32); 1 backlog note |
| P32_composition_tools | APPROVED | 2026-11-05 | opus (pipeline-reviewer) | review r1 APPROVED_WITH_NOTES: perimeter = the 25 declared paths (`c0cc169`); suite 529 Rust / 268 UI, clippy/fmt/typecheck green (reviewer run on clean `59cb210`); all 25 rows have a boundary test that fails on a wrong outcome (plan arithmetic slips in re-fit (4)/(8)/(10) and C7(d) asserted at the contract's exact values, re-derived by the reviewer); H1–H8 hold; no assumption contradicts the book; 5 new backlog notes (one probe-confirmed: a control used while a scroll is still settling re-fits from the pre-scroll rect) |
| P33_split_ui | APPROVED | 2026-11-05 | opus (pipeline-reviewer) | review r1 APPROVED_WITH_NOTES: perimeter = the 22 declared paths (`d47b6ce`; machine/ComposeStage/ComposeMode disclosed, FoldedStrip needed no change); suite 529 Rust / 291 UI, clippy/fmt/typecheck green (reviewer run, code tree = `d47b6ce`); all 15 rows have a boundary test that fails on a wrong outcome (plan arithmetic re-derived); H1–H8 hold; no assumption contradicts 10D; 4 new backlog notes (overlay `05` asserted as a substring; a quick second Split/Normal from the grid menu before the answer is dropped or duplicated) |
| P34_composition_polish | APPROVED | 2026-11-05 | opus (pipeline-reviewer) | re-review r2 APPROVED_WITH_NOTES: fix perimeter = the allowed files (`34299ba`: `Loupe.tsx`, `ComposeMode.tsx`, `compose.css`, `p34_loupe_snap.test.tsx`, `p34_clipboard.test.tsx` + tracker/Review log); F1–F4 resolved as corrected (loupe at source size with the stage's turn/flip/straighten, inverse-straighten mapping re-derived; pointer tracked while composing; C2(a) pins `−1421.8/−602.6`; C3(a) pins the re-fitted rect); suite 531 Rust / 301 UI, clippy/fmt/typecheck green (reviewer run, code tree = `34299ba`); triage: no blocking item remains, 0 MUST; 2 new backlog notes |
| P35_composed_display_and_chrome | APPROVED | 2026-11-05 | opus (pipeline-reviewer) | re-review r2 APPROVED_WITH_NOTES: fix perimeter = the allowed files (`bd90002`: `workspace.css` + tracker/Review log/handoff); F1 resolved as corrected (`.composed-frame img` resets right/bottom/margin; WebKit probe on the committed CSS: img rect = frame rect on C2(a), C2(b), a turned camera original and a tile); Rust tests, clippy, fmt, typecheck green; UI suite flaky on pre-existing `p33_split_grid` C4(d) (4/8 runs here, 1/6 on the pre-P35 tree), not P35's delta, backlog N1; triage 0 MUST |
| P36_editor_controls_and_overlays | IMPLEMENTED | 2026-11-05 | codex | single photo strip, overlays, typed straightening and shortcut controls; 22/22 rows covered, suite/lint/typecheck green |

Phase state machine (charter): `NOT_STARTED → [PROJECTED] → IMPLEMENTING → IMPLEMENTED → REVIEWING →
CHANGES_REQUESTED (→ IMPLEMENTING) → APPROVED`. The runner (`tools/pipeline/run_phase.sh`) drives it.

## 5. Sequencing and gates
`phases.txt` lists the 36 phases in execution order (17 MVP phases, the two real-run fix phases added
2026-10-26, the second real run's fix phases P19/P20 added 2026-10-29, the two tidy-up phases P21/P22 planned
2026-10-29 from `backlog.md`, P23_shape_on_arrival planned 2026-10-29 from book 10B, P24_capture_one_provisioning planned
2026-11-03 from book 10C, the second tidy-up pair P25/P26 planned 2026-11-04 from `backlog.md` and the P24 narrative, and the eight
composition phases P27–P34 planned 2026-11-04 from book 10D, and the two composition-correction phases P35/P36 planned 2026-11-05 from the 10D amendment R38); the runner is sequential and a
phase starts only when every phase in its `depends_on` is APPROVED (checked by the implementer's gate). Dependency graph:

```
P00 → P01 → P02 → P03 ─────────→ P07 → P08 → P11 → P15 → P16 → P17 → P18 → P19 → P20 → P21 → P22 → P23 → P24 → P25 → P26 → P27 → P28 → P29 → P30 → P33 → P34 → P35 → P36
   ↘        ↘ P04 → P05 → P06 → P09 → P10 → P12 → P13 → P14 ↗                                                                          ↘ P31 → P32 ↗
    ─────────↗            (P06 + P07 → P08; P09 + P12 → P13; P10 + P08 → P11; P14 + P11 → P15; P28 → P31; P30 + P32 → P33)
```
Exact `depends_on` per plan: P01[P00] · P02[P01] · P03[P02] · P04[P00,P01] · P05[P04] · P06[P05] ·
P07[P03,P04] · P08[P06,P07] · P09[P06,P02] · P10[P09] · P11[P10,P08] · P12[P10] · P13[P09,P12] · P14[P13] ·
P15[P14,P11] · P16[P15] · P17[P16] · P18[P17] · P19[P18] · P20[P19] · P21[P20] · P22[P21] · P23[P22] · P24[P23] · P25[P24] · P26[P25] ·
P27[P26] · P28[P27] · P29[P28] · P30[P29] · P31[P28] · P32[P31] · P33[P32,P30] · P34[P33] · P35[P34] · P36[P35].
- **Composition corrections (planner 2026-11-05).** P35 (workspace: composed display, Compose button, Split for any photo, header Details,
  Sessions icon) precedes P36 (editor: strip, overlays, straighten, controls and hints) because P36 builds on P35's shared `Hint` and its
  Sessions toggle on `PhotoPanel`/`FoldedStrip`. **No `implementer: fable`** (owner 2026-11-05: no Fable for implementation until the owner
  says so): both run on the default implementer (Codex; Claude Opus/Sonnet as the fallback, flow §2).
- **Composition order (planner 2026-11-04).** `phases.txt` runs the backend render and publish phases (P29, P30) **before** the editor
  (P31, P32): from P29 on every exported or published file is the app's own render of the (default, square) composition, so no build
  between phases saves compositions that the files silently ignore (M33/M34's "silent wrong crop"). P31 depends on P28 only, so the
  orchestrator may run P31/P32 before P29/P30 (edit `phases.txt`) if the owner prefers to try the editor first; the plans need no change.
- **Implementer per phase (owner 2026-11-04).** P31–P34 carry `implementer: fable` (frontend interaction and UI logic; P34's small
  `composition_apply` backend piece rides along). P27–P30 are backend (plus two Settings form fields) and run on Codex.

- **Local workflow usable after P09** (shoot → arrive → review → live previews → export). Attributes and the
  form (P10, P11) complete the local item data; Shopify starts at P12.
- **Parallelism**: `depends_on` allows P04 beside P02/P03, P07 beside P05/P06, and P12 beside P11. The runner
  commits to one working tree, so parallel runs need separate git worktrees; the orchestrator decides. Nothing in
  this plan requires it.
- **Projection** (flow §2): on for P04 and P12 (criteria rows whose fixtures are owned by Capture One /
  Shopify, charter rule 17). Off elsewhere; P05's one unprobed shape (capture-done argv) is handled by design
  (plan P05 Notes).
- **External preconditions the orchestrator must see met before launching**: P00 — Rust toolchain installed
  (§7.1, owner card 1). P12 — `~/.psm_shopify.env` holds `SHOPIFY_SHOP`, `SHOPIFY_CLIENT_ID` and `SHOPIFY_CLIENT_SECRET` of the owner's
  **test store** (owner 2026-10-26) and a client credentials token request succeeds (orchestrator checks it without
  printing the token); otherwise the grounding task cannot run and the session reports BLOCKED. P24 — task 0 (VERIFY-13/14,
  live against Capture One in a scratch folder, alongside whatever the owner has open) and the live row C6(a) need a session that
  can send Apple events; the Codex sandbox refused them in P16 (`-2700`), so when the implementer is Codex the orchestrator runs
  task 0 before dispatch and records the results in the evidence file; the implementer then reads them. If task 0 shows Capture
  One cannot keep two documents open, the phase stops with the owner card drafted in the plan's Notes (book 10C stop rule). P27 — VERIFY-15
  (10D §Grounding) **was run live by the orchestrator on 2026-11-04 and recorded** (`069eca4`); the implementer reads the evidence and never
  re-runs it; the only owed measurement is the work-Mac master timing (operational). The optional live rows (P27 C6(a)) run where Apple
  events are allowed, as before.

## 6. Shared skeleton and naming registry
Fixed up front so sessions cannot diverge. Anything **new** below does not exist yet; P00/P01 create it.

### 6.1 Repository layout and crates (Cargo workspace at the repo root)
| Path | Package / role | Depends on |
|---|---|---|
| `Cargo.toml`, `rust-toolchain.toml` (`channel = "stable"`), `.cargo/config.toml` | workspace, resolver 2, edition 2024 | — |
| `crates/core/` | `psm-core` (lib `psm_core`): domain, mechanisms, ports (traits), services, events, errors. **No Tauri, SQL, HTTP, osascript types.** | serde, uuid, sha2, unicode-normalization, chrono, tracing, thiserror |
| `crates/infra/` | `psm-infra` (`psm_infra`): `paths`, `settings_file`, `sqlite` (Store impl + `migrations/`), `spool`, `nudge` (loopback listener), `watcher` (FSEvents), `osascript` (runner), `rawpreview` (MC-20), `jobs` (runner), `logging`, `backup`; **P28** `imaging/` (`ImageOpsImpl`: header-only `dimensions`, 640 px thumbnails — the only crate that decodes pixels, H3 as narrowed); **P29** `imaging/{render.rs, linear.rs, metadata.rs}` (the `render_output` renderer), `media_store.rs` (`MasterFiles`, `OutputFiles`) | psm-core, rusqlite, rusqlite_migration, notify, axum, tokio, tracing-appender; P28 `image`, `fast_image_resize`; P29 `jpeg-encoder`, `kamadak-exif` (+ delegated `imageproc`/`little_exif`) |
| `crates/integrations/` | `psm-integrations` (`psm_integrations`): `editing/capture_one/` (`adapter.rs`, `scripting.rs` trait, `jxa.rs` impl, `jxa/*.js`, `hooks/*.applescript.tmpl`; P24 adds `provision.rs` (the ensure sequence, `classify_editor_error`) and `recipes.rs` (`RecipeSpec`, `SRGB_PROFILE`)), `camera/via_capture_one.rs`, `commerce/shopify/` (`adapter.rs`, `graphql.rs`, `transport.rs` trait, `type_map.rs`, `fixtures/*.json`). **Wiring owner:** P12 builds the adapter only; P13 task 0 adds `App.commerce` from `settings.integration`, the startup schema refresh (definitions + `attribute_definition_links`) and the search integration (amended 2026-10-26, P12 projection L10) | psm-core, psm-infra (osascript runner), reqwest |
| `crates/testkit/` | `psm-testkit`: `src/` fakes + `TestApp`; `tests/` **all behaviour tests** (§6.9) | psm-core, psm-infra, psm-integrations, tempfile |
| `apps/desktop/src-tauri/` | `psm-desktop` (bin + lib `psm_desktop_lib`): `commands/`, `events.rs`, `state.rs`, `tauri.conf.json`, `Info.plist`, `capabilities/default.json`, `build.rs` | all of the above, tauri, tauri-specta |
| `apps/desktop/` | Vite + React + TS: `package.json`, `src/services/bindings.ts` (**generated, never hand-edited**), `src/services/*.ts`, `src/components/ui/` (shadcn), `src/features/{shell,sessions,workspace,form,publish,settings,health}/`, **P31+ `src/features/compose/`** (`geometry.ts` — the TS mirror of `psm_core::mechanisms::composition`, `machine.ts` — the interaction state machine, `refit.ts`, `snapping.ts`, hooks and components; `src/services/compositions.ts`), `src/lib/`, `src/styles/tokens.css` (copied from `claude_design/tokens/tokens.css`) | — |
| `crates/integrations/src/render_output.rs` (P29) | `RenderOutputJob` (`JobHandler` for kind `render_output`, on its own runner `App.render_jobs`) | psm-core, psm-infra |

Layer rule (ch11): core ← nothing above it; services (in core) depend on ports only; infra and integrations
implement ports; `psm-desktop` commands are thin (parse → service → serialize).

### 6.2 Pinned dependency versions (verified on crates.io / npm, 2026-10-25)
Rust: `tauri = "2"` (2.12.5, feature `specta`), `tauri-build = "2"` (2.7.4), `tauri-specta = "=2.0.0-rc.26"`
(features `derive`, `typescript`), `specta = "=2.0.0-rc.26"`, `specta-typescript = "0.0.13"`, `rusqlite = "0.40"`
(feature `bundled`), `rusqlite_migration = "2.6"`, `notify = "8.2"`, `tokio = "1"` (1.54; `rt-multi-thread`,
`macros`, `sync`, `time`, `process`), `axum = "0.8"`, `serde`/`serde_json = "1"`, `uuid = "1"` (`v4`),
`sha2 = "0.11"`, `unicode-normalization = "0.1"`, `chrono = "0.4"` (plus `chrono-tz = "0.10"` as a
dev-dependency for TZ-pinned tests), `tracing = "0.1"`, `tracing-subscriber = "0.3"`, `tracing-appender = "0.2"`,
`thiserror = "2"`, `reqwest = "0.13"` (`json`, `multipart`, `rustls`, no default features; 0.13 has no `rustls-tls` feature;
P12 may add `blocking` or use a private runtime inside `ReqwestTransport`, delegated, under the constraint that
the transport never panics when created, used or dropped from a Tauri async command; the choice is recorded in
P12's handoff), `tempfile = "3"` (dev).
**Composition crates (verified on crates.io 2026-11-04 by the planner):** `image = "0.25"` (0.25.10; TIFF/JPEG decode, JPEG/TIFF
encode for thumbnails and fixtures), `fast_image_resize = "6"` (6.1.0; Lanczos3), `jpeg-encoder = "0.7"` (0.7.1; 4:4:4, ICC and APP
segments), `kamadak-exif = "0.6"` (0.6.1; reads TIFF and JPEG EXIF). Delegated to P29's implementer: `imageproc = "0.27"` (0.27.0; bicubic
rotation) and/or `little_exif = "0.6"` (0.6.23; EXIF writing) — or hand-written equivalents. libvips, ImageIO and Python are out (owner).
Note: `tauri-specta` **1.0.2 is Tauri-v1 only**; the 2.0.0-rc line is the Tauri-v2 one and must be pinned with `=`.
**Corrected by the orchestrator 2026-10-25:** rc.27 of `tauri-specta` was never published; rc.26 requires
`specta = "=2.0.0-rc.26"` and `specta-typescript = "^0.0.13"`. This trio with `tauri 2.12.5` (feature `specta`) was
resolved with `cargo generate-lockfile` on this Mac.
**Second correction, 2026-10-25 (after P00 r1 attempt 2):** `rusqlite_migration 2.6` requires `rusqlite ^0.40`
(0.41 made two `libsqlite3-sys` links conflict). The **whole Rust list above, as now written, was resolved together**
in one probe manifest (`cargo generate-lockfile`: tauri 2.12.5, rusqlite 0.40.2, rusqlite_migration 2.6.0, axum 0.8.9,
notify 8.2.1, reqwest 0.13.5, sha2 0.11.1, chrono-tz 0.10.4), and every npm pin below exists on the registry.
npm: `@tauri-apps/cli@2.12.4`, `@tauri-apps/api@2.12.4`, `react@19.3.1`, `vite@8.3.5`, `@vitejs/plugin-react@6.1.2`,
`vitest@5.0.5` (+ `jsdom`, `@testing-library/react@16.3.3`, `@testing-library/jest-dom`), `tailwindcss@4.3.3`,
shadcn/ui via `npx shadcn@latest init`, `@dnd-kit/core@6.3.1`, `@dnd-kit/sortable@10.0.0`, `motion@14.6.0`,
`sonner@2.0.8`. TypeScript: the latest 5.x line that `@vitejs/plugin-react` supports (P00 pins the exact
version; `package-lock.json` committed). Exact versions in `package.json`, no `^` drift at build time.

### 6.3 Application identity and data directory
- Bundle identifier `com.loorenz.sessionmanager`; `productName` "Session Manager"; `bundle.macOS.signingIdentity = "-"`
  (ad-hoc); `bundle.macOS.minimumSystemVersion = "13.0"`; `src-tauri/Info.plist` (merged automatically by Tauri) carries
  `NSAppleEventsUsageDescription`.
- App data dir = Tauri `app_data_dir()` = `~/Library/Application Support/com.loorenz.sessionmanager/`; env
  `PSM_DATA_DIR` overrides it (dev/tests). Layout (`psm_infra::paths::AppPaths`):
  `session_manager.db` · `settings.json` (0600) · `previews/<pk[0..2]>/<pk>/{original.jpg, adjusted-<g>.jpg}` ·
  `hooks/{capture_done,processing_done,variant_adjusted}.scpt` · `spool/`, `spool/bad/` · `logs/psm.<date>.log` ·
  `backup/session_manager-<yyyyMMdd-HHmmss>.db`. Nothing runtime lives in the repo.
  **10D (P28–P30):** `previews/<pk[0..2]>/<pk>/thumb-<g>.jpg` (640 px long edge, beside `adjusted-<g>.jpg`, same lifecycle) ·
  `masters/<pk[0..2]>/<pk>/master-<adjusted_gen>.tif` (16-bit uncompressed TIFF, a cache: the previous generation deleted on commit; all of a
  session's masters deleted when its local export, or its publish, completes and the session is not dirty) ·
  `outputs/<pk[0..2]>/<pk>/<uid12>-<slot>-g<adjusted_gen>-r<revision>-<format_key>.jpg` (the output cache; the name is the cache key
  `(master adjusted_gen, composition revision, output_format)`; other files of the same `uid12` pruned on commit) · an export staging folder
  also holds `manifest.json` (`{"format_key", "outputs": {"<file>": {photo_id, slot, adjusted_gen, composition_rev}}}`).
- `AppPaths` (P24) also carries `home` (`$HOME`; tests set it to their temp dir) — the base of the default Capture One document
  `<home>/Pictures/Session Manager` (10C MC-26, §6.8).
- `osascript` is always spawned with **cwd = the data dir** (MC-18 evidence).

### 6.4 Ports (traits in `psm_core::ports`)
`Store` (transactional repository; one SQLite impl; exact method set designed in P01 and extended per phase) ·
`Clock` (`now_ms() -> i64`, `mono() -> Instant`; `SystemClock`, testkit `ManualClock`) · `EditingProvider`
(`inspect_document`, `set_capture_folder`, `install_hooks`, `check_recipes`, `render(raw, RenderPurpose) -> JobId`,
`mirror_tag(raw, Classification)`, `import(paths)`, `capabilities()`; P23 adds `crop_aspect_ratios() -> Vec<String>` (the
provider's crop-shape names, opaque to core) and `shape_on_arrival(raw, ratio, now_ms) -> ShapeOutcome {Applied, Skipped,
UnknownRatio}` (10B MC-25: set the ratio only on an untouched full frame, one provider call); P24 adds `ensure(&ProvisionInputs
{style_name, wait}) -> ProvisionReport {document_created, recipes_created, style: StyleOutcome {Applied, NotListed, NotRequested}}`
(10C MC-26 steps 1–6 in the adapter, under one lock, user lane; health items per failing step) and `bring_document_to_front()`
(`document_not_front` on a read-back mismatch), and removes `check_recipes`) · `CameraProvider` (`status`,
`capture`, `capabilities`) · `CommerceAdapter` (ch09 capability methods; every mutating method takes `&ConfirmedRun`; P21 adds
`render_description(paragraphs) -> String`, the description text as the platform stores it, so core compares before
writing) ·
`RawPreviews` (`extract(raw, dest) -> Option<Extracted{orientation}>`) · `EventSink` (`emit(AppEvent)`).
`Store` (P23): `arrival_insert(photo, path_key, shape_on_arrival: bool)` enqueues the `crop_on_arrival` job in the arrival
transaction; `job_fail(id, code, now_ms)`, `count_jobs(kind, status, exclude_error)`, `requeue_failed_jobs(kind, exclude_error, now_ms)` (P25: the
`exclude_error` filter mirrors `count_jobs`; a health check passes `Some("crop_ratio_unknown")` while that item is set, a real start passes `None`).
Capture One internals: `psm_integrations::editing::capture_one::scripting::CaptureOneScripting` (typed
operations over the dictionary; impls `JxaScripting`, testkit `FakeCaptureOneScripting`; P23 adds `crop_aspect_ratios(id)`
and `apply_crop_if_untouched(id, variant, ratio) -> ShapeOutcome`, scripts `jxa/crop_aspect_ratios.js`, `jxa/apply_crop.js`;
P24 adds `editor_info() -> EditorInfo {running, version, pro}`, `launch()` (`open -g -a "Capture One"`, not a script),
`make_document(parent, name) -> id`, `open_document(file)`, `front_document(id) -> current id`, `ensure_recipe(id, name,
&RecipeProps) -> RecipeEnsure {created, changed, props}` (one run: create if missing, write only drifted dependent properties,
read back), `ensure_capture_style(id, name) -> StyleEnsure {listed, applied}` (one run), scripts `jxa/{editor_info,
make_document, open_document, front_document, ensure_recipe, ensure_capture_style}.js`, and removes `set_recipe_props` and its
script — `SCRIPTS` 16 → 21. `RecipeProps {enabled, output_format, color_profile, scaling_method, scaling_unit,
primary_scaling_value, jpeg_quality, existing_files, output_sub_folder, root_folder_type (read-only)}`; the dependent values
live in `recipes.rs`: preview `1600`/`80`, web `2048`/`90`, both `enabled false`, `JPEG`, `SRGB_PROFILE` (the string Capture One
reads back, recorded by P24 task 0), `Long_Edge`, `pixels`, `overwrite`, `_session_manager`).
Shopify internals: `commerce::shopify::transport::HttpTransport` (impls `ReqwestTransport`, testkit `FakeTransport`).
**10D changes (P27–P30; details in §6.10):** P27 removes `EditingProvider::{crop_aspect_ratios, shape_on_arrival}`, `CaptureOneScripting::
{crop_aspect_ratios, apply_crop_if_untouched}` and the two scripts (`SCRIPTS` 21 → 19); `arrival_insert(photo, path_key)` loses its flag;
`RecipeProps` gains `export_crop_method`, `bits`, `tiff_compression`. P28 adds the port `psm_core::ports::images::Imaging { dimensions(path)
-> Option<Frame>; thumbnail(source, dest, long_edge) -> bool }` (infra `ImageOpsImpl`), `CaptureOneScripting::process -> ProcessStarted
{ job_id, geometry: EditorGeometry }` (the variant's `adjustments.{rotation, flip, keystoneVertical, keystoneHorizontal, keystoneAmount}` read in
the same script run), `RenderCompletion.geometry`, and `Store::{composition, compositions_in_session, save_composition, backfill_compositions,
mark_previews_stale_once, complete_preview(raw, g, path, thumbnail, observed: Option<Fingerprint>)}`; `SqliteStore::open_with_default_shape`.
P29 renames `RenderPurpose::Export` → `Master`, adds `Imaging::render_output(master, values, slot, format, dest) -> OutputInfo`, `Store::
{adopt_frame(raw, Fingerprint), adjusted_gen(raw), enqueue_render_output}`, the `MasterStore`/`OutputStore` helpers, `JobRunner::excluding_kinds`.
P30 changes `CommerceAdapter::staged_uploads_create(run, stem, byte_size)` (core builds the stem: `mechanisms::upload_name::stem(folder_key, uid,
slot, revision)`), `Store::photo_link(photo, item, role, slot)`, `retire_photo_links`, and `replacement_candidates(links, retouch_gens,
composition_revs)`.

### 6.5 Tables (migration `001_init.sql` in P00: `settings`, `capture_events`; `002_domain.sql` in P01: the rest)
All ids `INTEGER PRIMARY KEY`; all `*_at` are UTC ms integers (MC-4); enums stored as snake_case TEXT; JSON as TEXT.
| Table | Columns |
|---|---|
| `settings` | `key` PK, `value` — **app state**, not user config: `mechanisms_version`, `active_session_id` |
| `capture_events` | `id`, `source` (`hook`/`watcher`/`scan`/`import`), `hook` (nullable), `payload`, `received_at`, `error` |
| `items` | `id`, `sku` UNIQUE, `folder_key`, `folder_key_ci` UNIQUE (lower-cased, MC-8), `barcode`, `item_type`, `title`, `origin`, `created_at`, `updated_at` |
| `item_links` | `item_id`, `integration`, `external_id`, `external_url`, `last_synced_at`, `create_pending`, `tags_base` (JSON), PK(`item_id`,`integration`) |
| `sessions` | `id`, `item_id` (NULL only for Unassigned, which is row id 1 created by the migration), `published_at`, `published_seq` DEFAULT 0, `change_seq` DEFAULT 0, `last_activity_at`, `created_at`, `undo_slot` (JSON, MC-5) |
| `photos` | `id`, `uid` UNIQUE (UUID v4), `session_id`, `raw_path` (canonical), `raw_hash`, `integrity` (`ok`/`raw_changed`/`raw_missing`), `provider_ref`, `original_preview_path`, `adjusted_preview_path`, `orientation`, `classification`, `position`, `captured_at`, `source`, `created_at`; UNIQUE(`session_id`,`raw_path`) |
| `raw_state` | `raw_path` PK, `path_key`, `adjusted_gen`, `rendered_gen`, `retouch_gen`, `last_own_write_at` (MC-17/19/21 per-RAW counters) |
| `photo_links` | `id`, `photo_id`, `integration`, `item_id`, `role`, `external_id`, `revision` DEFAULT 0, `retouch_gen` DEFAULT 0, `superseded_ids` (JSON), `upload_state`, `export_sha256`, `error`, `updated_at`; UNIQUE(`photo_id`,`integration`,`item_id`,`role`) |
| `attribute_definitions` | `integration`, `key`, `label`, `core_type`, `choices` (JSON), `validations` (JSON), `grp`, `description`, `writable`, PK(`integration`,`key`) |
| `attribute_definition_links` | `integration`, `key`, `platform_type`, `platform_meta` (JSON) — adapter-owned (H7) |
| `attribute_presets` | `item_type`, `grp`, `layout` (JSON `[{key, default}]`), PK(`item_type`,`grp`) |
| `attribute_values` | `item_id`, `key`, `value` (canonical JSON), `valid` DEFAULT 1, `provenance`, `ai_confidence`, `ai_status`, `ai_reason`, `ai_sources` (JSON), `updated_at`, PK(`item_id`,`key`) |
| `publish_runs` | `id`, `integration`, `created_at`, `confirmed_at`, `confirmed_summary` |
| `publish_items` | `id`, `run_id`, `session_id`, `create_item`, `title`, `frozen_main`, `frozen_condition`, `frozen_attributes`, `announced_removals`, `confirmed_change_seq`, `step`, `failed_step`, `uploaded`, `total`, `error`, `finished_at` |
| `jobs` | `id`, `kind`, `target`, `payload` (JSON), `status`, `attempts`, `next_attempt_at`, `error`, `created_at`, `updated_at` |
| `notifications` | `id`, `session_id`, `run_id`, `kind`, `text`, `created_at` |
| **003** (P27) `003_retire_shape_on_arrival.sql` | `DELETE FROM jobs WHERE kind='crop_on_arrival'` |
| **004** (P28) `compositions` | `photo_id` PK → photos, `kind` (`normal`/`split`), `shape` (`original`/`1:1`/`4:5`/`3:2`/`2:3`/`16:9`/`free`), `orientation` (`named`/`turned`), `quarter_turns` 0–3, `flip` (`none`/`horizontal`), `straighten` REAL, `x`,`y`,`w`,`h` REAL (NULL until the first frame), `frame_w`,`frame_h` INTEGER, `frame_source` (`preview`/`master`), `geometry` (JSON `EditorGeometry`) — the three NULL together until the first render, `origin` (`default`/`user`), `revision`, `state` (`ok`/`needs_review`), `review_reason` (`geometry`/`frame`/NULL), `updated_at`; plus `photos.thumbnail_path TEXT` |
| **005** (P30) `photo_links` recreated | as before plus `slot` (`single`/`left`/`right`, default `single`) and `composition_rev` DEFAULT 0; UNIQUE(`photo_id`,`integration`,`item_id`,`role`,`slot`); `upload_state` gains the value `retired` |

### 6.6 Enums, codes and kinds (Rust `enum` names in `psm_core::domain`, serialized snake_case)
- `Classification {Unreviewed, Main, Condition, Deleted}` · `PhotoSource {Capture, Import}` · `ItemOrigin {Linked, Local}` ·
  `Integrity {Ok, RawChanged, RawMissing}` (P25: `IntegrityCheck::verify` on a photo whose `raw_hash` is NULL hashes the file now
  through `set_raw_hash` and reports `Ok`; `RawChanged` only against a stored hash — 05A MC-3) · `UploadState {None, Exported, Staged, Uploaded, Attached, Ready, Failed}` ·
  link `role`: `gallery` or `attribute:<key>` · `PublishStep {Prepare, Upload, Update, Describe, Done, Failed}` ·
  `JobKind {OriginalPreview, Hash, RenderPreview, RenderExport, MirrorTag, PublishStep, VerifyHash, CropOnArrival}` (P23:
  `crop_on_arrival`, target = the RAW, once per RAW; terminal `failed` with `error = crop_ratio_unknown`, or with the last
  code after `CROP_GIVE_UP_ATTEMPTS` attempts = **8** (10B §Failures, changelog 23); `failed` rows of this
  kind are re-queued with `attempts = 0` at a real start and by the Health page's "Retry checks") ·
  `ShapeOutcome {Applied, Skipped, UnknownRatio}` (P23) ·
  `JobStatus {Queued, Running, Done, Failed}` · `Provenance {Human, Ai, Default}` · `AiConfidence {High, Medium, Low}` ·
  `AiStatus {Suggested, Accepted, Cleared}` · `AttributeGroup {Details, Condition, Product}` ·
  `CoreType` = the 18 types of ch08 · `SessionStatus {Publishing, Failed, Published, Ready, InProgress}` ·
  `RowLabel {Publishing(pct), PublishFailed, DescriptionFailed, Published, Ready, InProgress(n_to_review)}` ·
  `CameraStatus {Connected(name), Disconnected}` · `RenderPurpose {Preview, Export}` (**P29: `Export` → `Master`**) · `MechanismsVersion = 1`.
- **10D (P27–P30):** `JobKind::CropOnArrival`, `ShapeOutcome`, `Error::CropRatioUnknown`, health `crop_not_applied`/`crop_ratio_unknown` and
  `CROP_GIVE_UP_ATTEMPTS` are **removed in P27**. P28 adds `psm_core::domain::composition::{CompositionKind {Normal, Split}, Shape {Original,
  Square, FourFive, ThreeTwo, TwoThree, SixteenNine, Free}, Orientation {Named, Turned}, Flip {None, Horizontal}, Origin {Default, User},
  CompositionState {Ok, NeedsReview}, ReviewReason {Geometry, Frame}, FrameSource {Preview, Master}, OutputSlot {Single, Left, Right},
  EditorGeometry, Fingerprint, Rect, Frame, PixelRect, CompositionValues, Composition}` and the errors `composition_invalid{reason}` (reasons
  `turns_range`, `straighten_range`, `fraction_range`, `rect_outside`, `rect_too_small`, `off_shape`, `split_ratio`) and `frame_unknown`. P29 adds
  job kinds `render_master` (tracking rows, replaces per-photo `render_export`) and `render_output`, `psm_core::domain::output::OutputFormat`, and
  `needs_review{photos}` (data `photos` = labels `Main 02`, message `Re-frame first: Main 02, Condition 01`). P30 adds `UploadState::Retired`.
- Error codes (`psm_core::Error`, snake_case, carried to the UI as `{code, message, data}`): `relative_path`,
  `sku_collision{existing_sku}`, `sku_in_shop{sku}`, `target_has_raw`, `target_published`, `nothing_to_undo`,
  `variant_not_found`, `capture_folder_not_set`, `hooks_not_installed`, `recipe_missing{name}`, `tag_verification`,
  `raw_changed{photo_id}`, `raw_missing{photo_id}`, `several_products_carry_sku{sku}`, `foreign_media_added{n}`,
  `gallery_verification`, `upload_incomplete`, `attribute_verification{keys}`, `not_confirmed`, `timeout`,
  `no_integration`, `invalid_value`, `shop_not_permitted`, `app_not_installed`, `credentials_rejected` (09A MC-23),
  `crop_ratio_unknown{name}` (P23; terminal for the `crop_on_arrival` job), `document_not_front` (P24; activation refused because
  Capture One's front document could not be made the app's).
- Health codes (`HealthItem{code, ok, detail}`): `camera_disconnected`, `document_not_open` (from P24 also detail `Capture One could
  not open <file>[: <error>]`), `document_not_session` (from P24 also detail `<folder> exists but is not a Capture One session`),
  `hooks_not_installed`, `recipe_missing` (from P24 detail `<name> could not be created or repaired: <error>`; several rows allowed),
  `capture_folder_not_set`, `variant_not_found`,
  P24 (10C MC-26, one per failing ensure step, plain wording): `editor_not_installed` (`Capture One is not installed on this Mac`),
  `editor_not_pro` (`Capture One Pro is required; the installed edition cannot be scripted`), `editor_automation_denied` (`macOS has not
  allowed Session Manager to control Capture One. Allow it under System Settings › Privacy & Security › Automation`),
  `editor_not_running` (`Capture One did not start within <n> s`), `document_not_created` (`Capture One could not create the session
  <file>: <error>`), `document_not_front` (`Capture One is showing another session; captures would land there`; also gates camera
  status and capture like `capture_folder_not_set`; P25: cleared by a successful front switch, by any editing check while **no**
  session is active, and by `deactivate` — it describes an active session only, M32), `style_not_found` (`Style '<name>' not found in Capture One`, or `Style '<name>'
  could not be applied: <error>`); the ok items `document` (detail `<file name> open`), `recipes` (`Ready`), `hooks` (`Installed`) are set
  by the ensure and cleared when their step or an earlier one fails; `integration_token_invalid` (detail = the 09A MC-23
  message), `condition_field_missing` (detail = key, "not found" or "not a file list"; 09A MC-24; from P21 also
  `Condition photos field <key> unknown: shop fields not read yet` when the schema could not be read and nothing is
  stored), `settings_invalid` (P21; detail `settings.json: <parser message>` or `integration: <serde message>`),
  `backup_failed` (P21; detail = the error; cleared by the next successful backup),
  `crop_not_applied` (P23; detail `Shape not applied to <n> photo` / `… photos`, n = `failed` `crop_on_arrival` jobs whose
  error is not `crop_ratio_unknown`; derived at a real start and at every health check, counted before that check's
  re-queue), `crop_ratio_unknown` (P23; detail exactly `Shape on arrival: '<name>' is not a Capture One crop ratio`; set by
  the startup/health ratio check when the adapter's list is readable and lacks the name, and by the job; cleared when the
  list contains the name, when a job applies, or when the setting is `off`).
- Colour-tag mapping (MC-19): `main 4`, `condition 5`, `deleted 1`, `unreviewed 0`.

### 6.7 Tauri commands (`#[tauri::command] #[specta::specta]`, snake_case) and events
Commands by phase (payload types are the service DTOs; thin wrappers only):
P00 `health_get`, `app_paths_get` · P01 `item_create`, `item_update_identity`, `session_create`, `session_get` ·
P02 `photos_classify`, `photos_reorder`, `photos_delete`, `photos_undo`, `photos_move`, `photos_copy` ·
P03 `sessions_list` · P04 `session_activate`, `session_deactivate`, `camera_status`, `camera_retry`, `camera_capture` ·
P05 `photos_import`, `photos_assign_unassigned` · P07 `items_search` · P09 `export_run` ·
P10 `attributes_definitions_list`, `attribute_values_get`, `attribute_values_set`, `attribute_ai_accept_all`,
`attribute_ai_clear`, `presets_get`, `presets_set`, `form_get` · P13 `publish_preview`, `publish_confirm`, `publish_retry`,
`notifications_list` · P16 `settings_get`, `settings_set` · P18 `items_search_shop` (`items_search` becomes local-only) ·
P21: `item_update_identity` reads `barcode: ""` as "clear the barcode" (`null` = unchanged); `items_search_shop` rejects
with code `provider` when the shop cannot be reached (no silent `[]`) · P23 `crop_ratios_list` → `Vec<String>` (the editing
adapter's crop-shape names; rejects with the provider error when Capture One cannot answer); `settings_get`/`settings_set`
carry `crop_on_arrival` · P24: `settings_get` adds `effective_document_id` (the folder the app uses: the stored `document_id` or
the default); P25: `settings_get` also adds `default_document_id` (always `<home>/Pictures/Session Manager`, so the Settings screen can
show the app's own path before a save; `SettingsDto.default_document_id: String`); `settings_set` reads `document_id: ""` as "back to the app's own document" (stored `null`; a non-empty value must be
absolute, else `relative_path`); `session_activate` runs the editing ensure, brings the app's document to the front and only then
switches the capture folder (rejects with `document_not_front` when the front cannot be verified); no new command · **P27** removes
`crop_ratios_list`; `settings_get`/`settings_set` carry `default_shape` and `master_recipe` instead of `crop_on_arrival` · **P28**
`composition_set(photo_id, values: CompositionValuesDto) -> CompositionDto`, `composition_reset(photo_id) -> CompositionDto` (errors
`composition_invalid{reason}`, `frame_unknown`); `PhotoResponse` gains `thumbnail_path` and `composition: CompositionDto` (`rect`, `frame`,
`review_reason` nullable; the geometry is not sent) · **P29** `settings_get`/`settings_set` carry `output_format {long_edge, quality}` and no
`web_recipe`; `export_run` rejects with `needs_review` · **P34** `composition_apply(items: [{photo_id, values}]) -> {applied, refused[{photo_id,
reason}]}`.
From P17 every command is `pub async fn` over `AppState(RwLock<App>)` through `AppState::handles()` / `mutate()`
(`state.rs`), work inside `tauri::async_runtime::spawn_blocking`; no command holds the app lock across provider work.
Events (`AppEvent` → Tauri event name): `session_changed{session_id}`, `photo_added{photo_id, session_id}`,
`photo_changed{photo_id, session_id}`, `publish_progress{session_id, item_id, step, uploaded, total}`,
`export_progress{session_id, done, total, state}`, `health_changed{}`, `camera_status_changed{status}`.
Bindings: `tauri_specta::Builder::<tauri::Wry>::new().commands(collect_commands![…]).events(collect_events![…])`;
exported by the test `psm-desktop::export_bindings` to `apps/desktop/src/services/bindings.ts`
(`specta_typescript::Typescript::default()`); the committed file must equal the regenerated one.

### 6.8 Settings file keys (`settings.json`, user configuration; `psm_infra::settings_file::Settings`)
`document_id` (Capture One document **folder** path, MC-19; P24, 10C MC-26: missing, `null` or `""` → the app's own document
`<home>/Pictures/Session Manager` holding `Session Manager.cosessiondb`, created and opened by the app; an explicit value is honoured
and provisioned the same way; the Settings screen shows the default as "Capture One session: Session Manager (created by the app)"
with "Use another session…" (a `.cosessiondb` picker; the parent folder is stored) and "Use the app's own session") · `preview_recipe` (default `"PSM Preview"`) · `web_recipe`
(`"PSM Web"`) · `style_name` (default `""` — owner 2026-11-04, 10C step 5 / changelog 26, P25; empty = the next-capture settings are left as
the photographer set them and no Style health item; was `"PSM House"` until P24 — a file that explicitly holds a name keeps it) ·
`export_folder` (default `~/Pictures/Session Manager Export`) ·
`preview_delay_s` (15; read clamped 10–30, MC-17) · `theme` (`"light"`) · `sku_prefix` (owner card 2 answered
"generate", 05B MC-22; absent or empty = no proposal) · `integration` (`null` or `{"kind":"shopify","shop_domain","api_version":"2026-10","client_id","client_secret","condition_photos_key":"custom.damage_report"}`, 09A MC-23/MC-24; no stored access token) ·
`crop_on_arrival` (P23, 10B MC-25: `"off"` = leave as shot, or one of the editing adapter's crop-shape names, e.g. `"Square"`,
`"4x5"`; missing → `"Square"`; any non-empty string is accepted and stored as typed — core never knows the names; the
Settings screen shows it as "Shape on arrival" with "Leave as shot" + the adapter's list).
**10D (owner 2026-11-04):** `crop_on_arrival` is **retired in P27** — `default_shape` (one of `original`, `1:1`, `4:5`, `3:2`, `2:3`, `16:9`,
`free`; missing → `1:1`; a file that still carries `crop_on_arrival` is read once through `legacy_default_shape`: `Square → 1:1`, `off →
original`, `4x5 → 4:5`, `2x3 → 2:3`, `16x9 → 16:9`, anything else → `1:1`, and the next save drops the old key) and `master_recipe` (default
`"PSM Master"`) replace it; the Settings screen shows "Default shape" and "Master recipe". `web_recipe` is **retired in P29** (ignored on read,
dropped on save) and `output_format` (`{"long_edge": 2048, "quality": 90}`; `long_edge` 512–8192, `quality` 60–100; JPEG, 4:4:4, sRGB fixed;
`format_key = jpeg-<long_edge>-q<quality>`) replaces the PSM Web recipe values; the Settings screen shows "Output size (long edge, px)" and
"Output JPEG quality" instead of "Web recipe".
Naming rule: snake_case keys; booleans prefixed `is_`/`has_` if any are added; durations carry their unit suffix.
**Reading rule (P21):** a `settings.json` that does not parse, or an `integration` block that is not a complete
`IntegrationSettings`, never stops the app: the defaults are used in memory (Export mode for the block), the file is
left untouched, health shows `settings_invalid`; the next save writes a complete file. `settings_get` always shows
`api_version` and `condition_photos_key` (defaults filled).

### 6.9 Test layout
- Behaviour tests: `crates/testkit/tests/<phase_lower>_<slug>.rs` (e.g. `p02_review_order.rs`); one `#[test]` per
  criterion row named `c<criterion><row>_<slug>` (e.g. `c3b_undo_restores_position`). They use `TestApp` (§7.5).
- Core-mechanism unit tests: inline `#[cfg(test)] mod tests` in `crates/core/src/mechanisms/<mechanism>.rs`,
  functions named `mc<n>_<slug>`; only for functions a plan lists under **Core mechanisms**.
- Frontend tests: `apps/desktop/src/features/<feature>/__tests__/<phase_lower>_<slug>.test.tsx` with vitest + jsdom
  + Testing Library; `src/services/*` mocked with `vi.mock`.
- Fixtures: `crates/testkit/src/fixtures/` (synthetic RAW builder, hook payloads, settings; P28 `synthetic_jpeg.rs`, P29
  `synthetic_master.rs` — a hand-written 16-bit RGB TIFF with EXIF/GPS sub-IFDs and test patterns) and
  `crates/integrations/src/commerce/shopify/fixtures/*.json` (sanitized Shopify captures, P12).
- **Shared geometry case table (P28/P31):** `crates/core/src/mechanisms/composition_cases.json` is run by the Rust unit test
  (`composition::tests`) and by `apps/desktop/src/features/compose/__tests__/geometry.test.ts` (read with `fs`); a geometry change edits the
  table once. Compose tests live in `apps/desktop/src/features/compose/__tests__/`; pure-TS mechanisms (`geometry.ts`, `machine.ts`, `refit.ts`,
  `snapping.ts`, `workspace/outputs.ts`) are the only frontend units with unit tests (flow §4 Core mechanisms).

### 6.10 Composition skeleton (book 10D, planned 2026-11-04; the names P27–P34 share)
- **Constants:** `MIN_RECT_FRACTION = 0.05` (a rect's short side is at least 5 % of the turned frame's short side), `MASTER_TIFF_COMPRESSION =
  "Uncompressed"` (10D MC-28 after VERIFY-15), `FRAME_MARGIN_PX = 24`, `ZOOM_PER_PX = 0.005` (wheel zoom factor `exp(−deltaY · ZOOM_PER_PX)`),
  `WHEEL_SETTLE_MS = 120`, `EDGE_REVEAL_PX = 12` (retired by P36), `SNAP_PX = 6`, `COMPOSE_HINT_KEY = 'compose.hintShown'`, `localStorage` `tools.hidden`,
  `tools.sections`; the app-state `settings` key `rerender_after_composition` (`done`) marks the one-time re-render of pre-10D previews.
- **Geometry contract** (one text, in `P28_composition_foundation.md` §The geometry contract; `crates/core/src/mechanisms/composition.rs` and
  `apps/desktop/src/features/compose/geometry.ts` implement it; the case table proves both): turned frame; shape ratio (`split` → 2, `original`
  → `W/H`, `free` → none; `orientation turned` → `1/r`); default rect (largest centred); validation order and reasons; pixel rule (half away
  from zero, split width even, `height = width/2`); straighten sign (positive = clockwise on screen; source `s = c + R(−θ)(p − c)`); inside
  after straighten (`ε = 1e-6 · max(W, H)`); fingerprint (geometry equal with `1e-6` on reals, orientation equal, master size equal when both
  are masters; `review_reason` `geometry` or `frame`).
- **Composition lifecycle:** default row at arrival/insert/copy (rect NULL until the first render); backfill at every build; the first render
  materialises the rect and the fingerprint; a compatible render keeps the composition; an incompatible one sets `needs_review`, the reason,
  and replaces the fingerprint (rect fractions kept); `save_composition` validates, bumps `revision`, clears `needs_review`, bumps the
  session's `change_seq` when the photo is not deleted (MC-6); identical values are a no-op.
- **Render pipeline (P29):** master through `RenderPurpose::Master` (`master_recipe`) only when an output is missing; `render_output` on its own
  runner; the render contract in `P29_master_and_output_render.md`; outputs cached by name; `manifest.json` in staging; masters released after a
  completed local export / publish of a non-dirty session.
- **Outputs (P30):** `CompositionKind::slots()`; `mechanisms::naming::output_names`; upload stem `<folder_key>-<uid12>[-L|-R][-r<n>]`; links per
  slot; retirement (`upload_state retired`, external id kept → `own`); `D` and the damage report per output.
- **Editor (P31–P34):** `features/compose/machine.ts` — phases `idle | dragging | pinching | wheeling | trimming`, events and rules 1–16 in
  `P31_compose_mode.md` §The interaction contract (+ P32's `SetValues`/`Nudge`/`Trim*`); view maths (`frameBox`, `viewTransform`, `panBy`,
  `zoomAbout`, `clampRect`); saves latest-wins per photo, ticketed; `refit.ts` (centre- and area-preserving re-fit; `trim`); keys: R, Esc, Return,
  arrows (Shift ×10), `.`, 1–6/0, X, S, `[`/`]`, ⌘⌥R, ⌘T, Z, ⌥ (snap off), ⌥⌘C/V/P; CSS classes per plan; `PhotoMenuAction` gains `compose`,
  `split`, `normal`, `apply`.
- **Composition corrections (P35–P36, book 10D amended `b75ad61`/`163857a`, R38; planned 2026-11-05).**
  P35: `COMPOSE_WAITING = "Waiting for Capture One's full-frame preview"` (exported from `services/compositions.ts` beside `canCompose`, the
  only availability rule), toast id `compose-waiting`, `HINT_DELAY_MS = 400`; `components/ui/tooltip.tsx` → `Hint({ text, children })` (Radix
  Tooltip, own provider; a control that cannot act is `aria-disabled`, never `disabled`); `features/compose/composed.ts` → `composedLayout`,
  `leavesFrameOut` (crop mark: straighten ≠ 0 or the pixel rect smaller than the turned frame); `ComposedPicture` (callers `StageImage`,
  `PhotoTileFace`); CSS `.stage-composed`, `.tile-composed`, `.composed-frame`, `.composed-split`, `img[data-swapped]`, `--composed-ratio`,
  `.tile-crop-mark`, `.stage-compose`, `.panel-sessions`, `.photo-menu-why`; retired `.tile-composition`, `.tile-frame`, `.tile-split`,
  `.sessions-toggle`, `canSplit`; `App` holds `detailsOpen` (the `assignOpen` pattern).
  P36: keys while composing `o` (cycle overlay) and `f` (fold/unfold the browser; no longer swallowed); `localStorage` `compose.overlay`
  (`off | thirds | grid | golden | diagonals`, default `thirds`), `compose.straightenGrid` and `compose.snap` (`'0'` = off, default on);
  `OVERLAY_ORDER`, `STRAIGHTEN_GRID_PX = 24`, `STRAIGHTEN_STEP = 0.1`, `STRAIGHTEN_STEP_SHIFT = 1`; `features/compose/overlays.ts` →
  `overlayLines`; `snap` guides become `'centre' | 'start' | 'end' | null`; new `EditorToolbar.tsx`, `StraightenControl.tsx`,
  `useComposePrefs.ts`; ids `#compose-overlay`, `#compose-straighten-degrees`; CSS `.compose-overlay-guides[data-overlay]`,
  `.compose-straighten-grid`, `.editor-toolbar`, `.compose-nudge`; **retired** `EDGE_REVEAL_PX`, `BrowserOverlay`, `.compose-browser` (the
  edge-reveal browser of P31, dropped by R38). No command, table, setting or Rust change in either phase.

## 7. Environment topology (verified 2026-10-25 on the dev Mac; "if reality disagrees, update this section")
### 7.1 Verified facts
| Check | Command run | Result |
|---|---|---|
| macOS | `sw_vers` | 26.5.2 (25F84) |
| Rust | `cargo --version` (P00, 2026-10-25) | Installed: `cargo 1.99.1 (5f94df478 2026-08-27)`. The earlier preflight found it absent; it was installed before P00 started. |
| Cargo dependency resolution | `cargo test --workspace` (P00, 2026-10-25) | Resolved after the §6.2 correction to `rusqlite 0.40`; the workspace compiles and all 16 P00 boundary tests pass. |
| Xcode | `xcode-select -p`, `xcodebuild -version` | Command Line Tools at `/Library/Developer/CommandLineTools`; full Xcode absent — sufficient for desktop Tauri (Tauri prerequisites: "Only developing for desktop targets? … xcode-select --install"). |
| Node / npm | `node --version`, `npm --version` | v22.22.3 / 10.9.8 (pnpm absent; use npm) |
| Node workdir selection | `which node npm` in `apps/desktop/` (P00, 2026-10-25) | This directory resolves `/opt/homebrew/bin` Node v23.11.0 / npm 11.5.2; `npm install` fails with Arborist `Cannot read properties of null (reading 'edgesOut')`. Prefixing `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH` selects the planned Node 22 / npm 10 and install succeeds. |
| Tauri CLI | — | not installed; installed per project as devDependency `@tauri-apps/cli` and run as `npm run tauri -- <cmd>` (no `cargo install` needed) |
| osascript | `which osascript` | `/usr/bin/osascript` (JXA available) |
| sqlite3 CLI | `sqlite3 --version` | 3.51.0 (inspection only; the app bundles its own via `rusqlite/bundled`) |
| Python | `python3 --version` | 3.13.2 (runner scripts only; never a build dependency) |
| Git / Homebrew | `git --version`, `brew --version` | 2.48.1 / 7.0.10 |
| Capture One | `ls /Applications` | `Capture One.app` present (Pro 16.8.6.25 per evidence); test document `~/Pictures/session test 1/` with `Capture/`, `Output/`, one ARW |
| App data dir | `ls ~/Library/Application Support/ \| grep -i session` | nothing yet (created at first run of P00) |
| Export folder default | `ls ~/Pictures/Session Manager Export` | does not exist; created on first export |
| Shopify credentials | `~/.psm_shopify.env` (owner, 2026-10-26) | Dev Dashboard app client ID + secret for the owner's test store; client credentials grant (09A MC-23); real store later = settings change |
| Codex sandbox | runner `run_codex` | `workspace-write` plus `--add-dir` for the repo `.git`, `~/.cargo` and `~/.npm` (orchestrator fix 2026-10-25 after P00 r1: Cargo's registry cache and npm's cache were refused; `npm install` of `apps/desktop/package.json` succeeds outside the sandbox). Use the default Cargo home and npm cache; no local `CARGO_HOME`. Codex's command review rejects `rm -rf`: remove generated files with `git rm`/`git clean -n` + targeted deletes or avoid creating them. |
| P00 bundle launch | `open -n --env PSM_DATA_DIR=/tmp/psm-spike-bundle -a '/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager/apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app'` | Build and ad-hoc signing succeeded; `codesign --verify --deep --strict` succeeded. LaunchServices returned `NSOSStatusErrorDomain Code=-10827 kLSNoExecutableErr` inside the Codex sandbox. The orchestrator launched the bundle outside the sandbox and the owner saw the Health page (evidence `packaging_spike.md`). Automation attribution with the corrected real-event probe awaits an interactive check. |
| Claude sessions | `.claude/settings.json` | Bash allowlist includes `cargo`, `rustup`, `npm`, `npx`, `node`, `osascript`, git read/write (no force-push) |

### 7.2 Commands (standard invocations; Rust is installed and the corrected SQLite pair resolves;
P00 records command outcomes and timings)
| Purpose | Command (from repo root unless noted) |
|---|---|
| Rust suite (L4 stamp) | `cargo test --workspace` |
| One phase's behaviour tests (L1) | `cargo test -p psm-testkit --test <file_stem>` |
| One core-mechanism module | `cargo test -p psm-core <module>::tests` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check` |
| Bindings regeneration | `cargo test -p psm-desktop export_bindings` (writes `apps/desktop/src/services/bindings.ts`) |
| Frontend install / typecheck / test | `cd apps/desktop && npm ci` · `npm run typecheck` (`tsc --noEmit -p tsconfig.json`) · `npm test` (`vitest run`) |
| Dev app | `cd apps/desktop && npm run tauri -- dev` (Vite dev server inside the Tauri window) |
| Build `.app` (ad-hoc signed) | `cd apps/desktop && npm run tauri -- build --bundles app` → `apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app` |
| Run with a scratch data dir | `PSM_DATA_DIR=/tmp/psm-dev npm run tauri -- dev` |
"Suite, lint and typecheck green" (flow §4, B4) means: `cargo test --workspace`, `cargo clippy … -D warnings`,
`cargo fmt --check`, `npm run typecheck`, `npm test` — all exit 0.

### 7.3 Database safety
Tests open SQLite only inside a `tempfile::TempDir` (never the configured data dir). The configured database is
left at head. Migrations are applied by `rusqlite_migration::Migrations::to_latest` at `App::open`; WAL mode.

### 7.4 Known baseline caveats
- Capture-dependent VERIFY items are UNPROBED (camera did not release: evidence VERIFY-12); P00's spike checklist
  re-runs them when the owner has set the camera to release without focus. Nothing in the suite depends on them.
- Hooks block Capture One while they run (evidence VERIFY-2); hook bodies must stay fire-and-forget (MC-15).
- 2026-10-25 is a DST change day in Europe/Stockholm — a useful fixture date for MC-4 tests, pinned with `chrono-tz`.

### 7.5 Test harness (`psm-testkit`, created in P00 and extended per phase; every helper has a caller in the phase that adds it)
- `TestApp::new()` → a `TempDir` data dir, real SQLite, `ManualClock`, `FakeEditing`, `FakeCamera`, `FakeCommerce`
  (from P12), `CollectingEventSink`; `app.open()` runs the real startup path; `app.deliver_hook(json)` writes a spool
  file and drains it (the hook → app seam); `app.watcher_event(path)` injects a watcher event; `app.tick(ms)`
  advances the clock and runs due jobs; `app.focus()` fires the window-focus path.
- `FakeCaptureOneScripting`: an in-memory Capture One document (kind, `captures`, capture-root collection, hook
  properties, recipes, images by path with variants and colour tags, `camera`); configurable faults (`fail_next_set`,
  `timeout_next_write`, `readback_garbage`); records every operation. `FakeEditing` = the real Capture One adapter
  over this fake (so adapter logic is under test) — never a second implementation of `EditingProvider`. P23: each
  `FakeVariant` carries `crop_ratio` (`""` untouched), `crop` and `max_crop` (default `(5981, 3987)`, the evidence's full
  frame); the state carries the evidence's 11 ratio names and the faults `fail_apply_crop`, `timeout_next_apply_crop`; a
  real write is recorded as op `set_crop:<variant>:<ratio>`. P24: editor state (`running`, `pro`, `automation_denied`,
  `launch_fails`, `running_after_polls`), `open_documents` / `front_document` (the configured document initially; `make_document`
  creates the session layout on disk in the temp dir and opens it; `open_document` needs the `.cosessiondb` on disk; `front:<id>`),
  faults `fail_make_document`, `fail_open_document`, `fail_front`, `fail_make_recipe`, `styles` (default `["PSM House"]` — kept after P25
  made the settings default empty; a test that wants a style applied saves `style_name` explicitly) and
  `next_capture_styles`; `recipes` become `HashMap<String, RecipeProps>`; `check_id` accepts any open document. `TestApp` sets
  `paths.home` to its temp dir and writes an empty `Capture One Session.cosessiondb` marker beside the temp document's `Capture/`.
- `FakeCommerce`: in-memory shop (products with variant SKU, media list in order with filenames incl. Shopify-style
  uniquifiers, files, metafields, tags); `fail_after_effect(step)` to simulate a kill between remote effect and local
  record; records every mutation with its `ConfirmedRun`. `FakeTransport` replays sanitized GraphQL fixtures for the
  real Shopify adapter's tests.
- `SyntheticRaw::build(streams, orientation)` writes a minimal TIFF with IFD0/IFD1 JPEG streams for MC-20 tests.
- **10D (P27–P30):** P27 removes the fake's crop state (`FakeVariant.{crop_ratio, crop, max_crop}`, `crop_ratios`, `fail_apply_crop`,
  `timeout_next_apply_crop`, `set_variant_crop`) and builds the default recipes from the new `RecipeSpec`s (`PSM Preview`, `PSM Web`; P29: `PSM
  Preview`, `PSM Master`). P28: `FakeVariant.geometry: EditorGeometry` (default), `fake.set_variant_geometry(variant, geometry)`, `process`
  returns it; `SyntheticJpeg::rgb(w, h, colour)`; `TestApp::{complete_render_with(job, bytes), composition(photo_id)}`. P29:
  `SyntheticMaster { w, h, pattern, meta }` (16-bit RGB uncompressed TIFF by hand: `Solid`, `Marks`, `ColumnMarker`, `Checker1px`; EXIF
  `Make`/`Model`/`Copyright`/`Orientation`, Exif sub-IFD `DateTimeOriginal`, GPS sub-IFD); `TestApp::{complete_master_render(job, &master),
  decode(path), outputs_dir(raw), masters_dir(raw)}`; `complete_all_renders` writes a default `SyntheticMaster` for master jobs and the stub
  JPEG for previews; `tick` also ticks `render_jobs`. P30: `FakeCommerce` records `uploaded_sha[filename]`; `SqliteStore::migrations()`
  exposed for the migration-005 row.

## 8. Tool protocols
No architecture graph, no contract system in this repo (ch03). Sessions use this file and the plan file only.

## 9. Amendment log
- 2026-10-25 v1 — created by the planner (round 1).
- 2026-10-25 P00 r1 continuation — corrected the stale environment note after resolving SQLite, running the workspace suite and building an ad-hoc bundle; recorded the LaunchServices refusal at the bundle-launch boundary.
- 2026-10-25 P00 r1 close — recorded the orchestrator's successful launch outside the Codex sandbox and the remaining interactive Automation check.
- 2026-10-26 real-run fix phases — planner added P17_real_run_fixes and P18_pending_state_ui (§4 rows, §5 order, §6.7 command rule and `items_search_shop`) from `real_run_2026-10-26.md`. Fold-back owed by the orchestrator before P17 launches: 05A MC-2's capture-root sentence (the `capture` favourite is the current capture folder per the evidence Correction of 2026-10-26; the root is `<document id>/Capture`).
- 2026-10-29 second real run — planner added P19_navigation_and_drop (§4 row, §5 order and `depends_on`) from `real_run_2026-10-26.md` "Second real run" R7, R8. Frontend only; no registry, command or environment change.
- 2026-10-29 orchestrator added P20_drop_continuity (owner feedback on the P19 drop; frontend only, 3 rows).
- 2026-10-29 shape on arrival — planner added P23_shape_on_arrival from book 10B (MC-25, M31, R35; owner decision
  2026-10-29). §2 (10B, MC-25, M31), §3 trace ranges, §4 row, §5 count 23 / graph tail / `depends_on` P23[P22], §6.4
  (`EditingProvider::{crop_aspect_ratios, shape_on_arrival}`, `Store::{arrival_insert flag, job_fail, count_jobs,
  requeue_failed_jobs}`, `CaptureOneScripting` two ops + two scripts), §6.6 (`JobKind::CropOnArrival`, `ShapeOutcome`,
  `crop_ratio_unknown`, health `crop_not_applied` / `crop_ratio_unknown`, `CROP_GIVE_UP_ATTEMPTS` = 8),
  §6.7 (`crop_ratios_list`), §6.8 (`crop_on_arrival`), §7.5 (fake crop state). Book hole reported, not patched: 10B's
  "give up after the same limit" has no limit in MC-19.
- 2026-10-29 tidy-up — planner added P21_backend_robustness and P22_frontend_tidy_up from `backlog.md` (owner: fix the known small leftovers before the work Mac and the real store). §4 rows, §5 count/graph/`depends_on` (P20 recorded too), §6.4 `render_description`, §6.6 health codes `settings_invalid`/`backup_failed` and a third `condition_field_missing` detail, §6.7 barcode `""` and the `provider` rejection of `items_search_shop`, §6.8 reading rule. Backlog rows 124, 125 / P00 note 1 / 103 and 129 were verified already fixed by P17/P18 and marked so; no intention-book change needed.
- 2026-11-03 Capture One set up by the app — planner added P24_capture_one_provisioning from book 10C (MC-26, M32, R36; owner
  decision 2026-11-03). §2 (10C, MC-26, M32), §3 trace ranges, §4 row, §5 count 24 / graph tail / `P24[P23]` / the Apple-events
  precondition for task 0, §6.1 (`provision.rs`, `recipes.rs`), §6.3 (`AppPaths.home`), §6.4 (`EditingProvider::{ensure,
  bring_document_to_front}`, `check_recipes` removed; `CaptureOneScripting` seven ops, `set_recipe_props` removed, `SCRIPTS` 21,
  `RecipeProps` and the dependent recipe values), §6.6 (`document_not_front`; seven health codes with their plain details; new
  details of `document_not_open`, `document_not_session`, `recipe_missing`), §6.7 (`effective_document_id`, `document_id: ""`,
  `session_activate` order), §6.8 (`document_id` default rule and the Settings wording), §7.5 (fake editor/document/style state,
  `TestApp` home and marker). No intention-book change; two readings recorded in the plan's Notes (folder vs file in `document_id`;
  M32's "frontmost while active" enforced at activation and at every ensure, not polled).
- 2026-11-04 second tidy-up — planner added P25_tidy_up_after_provisioning (backend) and P26_frontend_leftovers (frontend) from
  `backlog.md` rows 173, 168, 106 (+15, 69), 133 (+11), 164, 107 (first clause), 167, 157, the P24 narrative's "Not yet" and the owner's
  empty-default-style decision (10C step 5, changelog 26, written by the orchestrator 2026-11-04). §4 two rows, §5 count 26 / graph tail /
  `P25[P24]` / `P26[P25]`, §6.4 `requeue_failed_jobs(kind, exclude_error, now_ms)`, §6.6 the `document_not_front` clearing rule and the
  `Integrity` NULL-hash rule, §6.7 `default_document_id`, §6.8 `style_name` default `""`, §7.5 the fake's style list kept. Every owner item
  was verified still present in the code at `c90b95d`; none dropped. Backlog 107's second clause (P09 staging sweep) stays open.
- 2026-11-04 composition — planner (Fable) added P27_retire_shape_on_arrival, P28_composition_foundation, P29_master_and_output_render,
  P30_outputs_in_export_and_publish (Codex) and P31_compose_mode, P32_composition_tools, P33_split_ui, P34_composition_polish (`implementer:
  fable`) from book 10D (MC-27…MC-29, M33–M35, R37, changelog 27; amended `069eca4` after the orchestrator's VERIFY-15: uncompressed master,
  fingerprint by Capture One geometry). §2 (10D, 10B retired, M1–M35), §3 trace ranges, §4 eight rows, §5 count 34 / graph / `depends_on` /
  the composition order and implementer notes / VERIFY-15 recorded, §6.1 (`imaging/`, `media_store.rs`, `render_output.rs`, `features/compose/`),
  §6.2 composition crates, §6.3 storage (`thumb-`, `masters/`, `outputs/`, `manifest.json`), §6.4 port changes, §6.5 migrations 003–005, §6.6
  removals and the 10D enums/errors/job kinds, §6.7 commands, §6.8 `default_shape`/`master_recipe`/`output_format` (legacy mapping), §6.9 the
  shared case table, **new §6.10** (the composition skeleton), §7.5 harness. Owner's phasing kept except: the foundation split into P27
  (retire) + P28 (compositions), outputs before the editor (reason in §5), outputs split into P29 (render) + P30 (publish), the editor split
  into P31 (compose mode) + P32 (tools). No intention-book change; no owner card.
- 2026-11-05 composition corrections — planner added P35_composed_display_and_chrome and P36_editor_controls_and_overlays from the owner's
  corrections after the first composition try-out (book 10D amended `b75ad61` + `163857a`, R38, changelog 28). §4 two rows, §5 count 36 / graph
  tail / `P35[P34]` / `P36[P35]` / the no-Fable implementer note (owner 2026-11-05), §6.10 the P35–P36 block (constants, `Hint`, `ComposedPicture`,
  `composed.ts`, `overlays.ts`, `localStorage` keys, keys `o`/`f`, CSS, retirements incl. `EDGE_REVEAL_PX`). Folded in from the P31–P34 narratives'
  "Not yet": Re-frame badge after Apply, the item card's "Edit details →", focus back to the editor after the Shape/Overlay menus and a slider
  drag, the snap guide at the line it snapped to. No intention-book change; no owner card.
