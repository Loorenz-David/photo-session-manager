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
| Mechanism contracts MC-1…MC-34 (exact rules) | `book/05A_mechanism_contracts.md`, `book/05B_sku_generation_and_change.md`, `book/09A_shopify_credentials_and_condition_field.md`, `book/07A_publish_contracts.md` (+ `07C` grounding fold-back), `book/07B_retouch_after_publish.md`, `book/10A_capture_one_contracts.md`, `book/10B_shape_on_arrival.md` (MC-25, owner 2026-10-29; **retired 2026-11-04 by 10D**), `book/10C_capture_one_setup_by_the_app.md` (MC-26, owner 2026-11-03), `book/10D_composition_and_framing.md` (MC-27…MC-29, owner 2026-11-04; amended 069eca4 after VERIFY-15), `book/10E_direct_camera_adapter.md` (MC-31…MC-34, owner 2026-11-06 at `53d6ea4`; inventory additions R-I18…R-I38 at `41bef14`; card CI1 answered "always jump", R46 at `e0ce354`) |
| Measurement ledger M1–M40 (root of every criterion's trace; M31 retired) | `book/14_measurement_ledger.md` |
| Capture One facts and VERIFY probes | `docs/intention/evidence/capture_one_16.8.6.md`; dictionary `docs/reference/capture_one_16.8.6.sdef` |
| Sony camera facts (lab findings H-n/F-n, constraints C1–C10, the app's own checks V1–V6, the spike and acceptance results written by P47/P52) | `docs/intention/evidence/sony_camera_lab.md`; the lab repo `/Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests` at `221aa49` (**read-only**; `sdk/` is copied by P45, `camlab/ftp` and `camlab/ledger.py` are ported by P43/P44, `docs/session-manager-handoff.md` §4 is the protocol) |
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
  (M1–M40; M31 retired), an `MC` id (MC-1…MC-34; MC-25 retired) or a hard constraint (H1–H8). A row may instead cite a chapter rule
  (`ch07 §Open filter`, `evidence/design_handoff §Field kind map`) **only** for UI-rendering rows, operations
  rows the inventory listed as "not contracted", and packaging rows (ch11); such a cell is prefixed `[rule]` so
  the reviewer knows the row has no ledger measurement behind it. Every plan file derives its own counts of
  criteria, rows and `[rule]` rows. **Owner-attended phases** (P47 spike, P52 hardware acceptance; planner 2026-11-06) carry
  `[manual]` rows: flow §4 rule 1's exemption for environment-lifecycle checks, each recorded in `evidence/sony_camera_lab.md` and the
  plan's Review log by the orchestrator who runs the phase with the owner; `run_phase.sh` never dispatches them (the plan's task 0 makes
  an implementer report `BLOCKED`).
- **Camera hard rules (owner, 2026-11-06; restated in every P42–P52 plan):** never commit a Sony SDK file, the lab's hand-written
  `sdk_shim` headers, a `.env` file, a credential or an SSH fingerprint (the repo is public); no Python; never change the camera's network
  or FTP settings, delete a camera file, or kill `ptpcamerad` or any other process; live view is out of scope; the automated suite never
  needs the camera (flow §6A). **No `implementer: fable`** on any camera phase (owner).
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
| P36_editor_controls_and_overlays | APPROVED | 2026-11-05 | opus (pipeline-reviewer) | re-review r2 APPROVED_WITH_NOTES: fix perimeter = the allowed files (`9cbfd24`: `p36_straighten.test.tsx`, `p36_strip_overlays.test.tsx` + tracker/Review log/handoff); F1 resolved as corrected (C7(a)/(b) focus the control, `x` dispatched at `document.activeElement`; changed-input probe: `x` at a still-focused select/slider does nothing, so both tests now need the blur); F2 resolved (C1(b) panel tile 2 moves the editor 1 → 2); Rust 532, clippy, fmt, typecheck, UI 339/339 green (reviewer run, code tree = `9cbfd24`); triage 0 MUST; backlog P32/P34 focus, guide and ⌥-drag rows marked resolved |
| P37_editor_chrome | APPROVED | 2026-11-05 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 17 declared paths (`4b4fbba`); Rust 532, clippy, fmt, typecheck, UI 348/348 green (reviewer run, clean `4b4fbba`); all 12 rows have a boundary test that fails on a wrong outcome (C2(a) all 32 T2 cards with exact text; C8(a) changed-input probe: `]` really switches photo with both drawers open); assumptions within Delegated choices; H1–H8 hold; 4 backlog notes (Item details/⋯ hover outside T2, WebKit layout unchecked, dead CSS resolved, P33 flake recurred) |
| P38_editor_polish | APPROVED | 2026-11-05 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 13 declared paths (`1b7a328`); UI 355/355, Rust suite, clippy, fmt, typecheck green (reviewer run, code tree `1b7a328`); all 12 rows have a boundary test that fails on a wrong outcome (C4(d) discriminates via the revert save; changed-input probes: arrows nudge with the pad open, keys off with the menu open, second Esc reverts); assumptions within 10D; H1–H8 hold; 3 backlog notes (pad id breaks `aria-controls`, duplicate outside-dismiss handler, WebKit look unchecked) |
| P39_undo_history_core | APPROVED | 2026-11-05 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 22 declared paths (`bbfede7`) + the orchestrator's C4(h) test edit (`a22dc41`); Rust 561, clippy, fmt, typecheck green (reviewer run, code tree `a22dc41`); all 27 rows have a boundary test that fails on a wrong outcome (changed-input probes: the C4(c) message text, C4(a) session ids, `Unreview 02`); assumptions within MC-30 and Delegated choices; H1–H8 hold; 5 backlog notes (copy redo with its source row gone is stuck in the redo list → P40; move redo inserts at the old place instead of appending); the tracker read NOT_STARTED at review start (the implementer was BLOCKED, the orchestrator's ruling resumed it) |
| P40_undo_history_compositions_and_attributes | APPROVED | 2026-11-05 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 13 committed paths (`69d3842`: 10 declared + tracker + plan); Rust suite, clippy, fmt, typecheck green (reviewer run, clean tree `57495ec`); all 20 rows have a boundary test that fails on a wrong outcome; the `photo_id` assumption matches the generated binding; H1–H8 hold; 6 backlog notes (probe-confirmed: undoing a re-frame of a `needs_review` photo restores the old rect as `ok`, not `needs_review` → P41/owner) |
| P41_undo_history_ui | APPROVED | 2026-11-05 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 28 committed paths (`84fb443`, = handoff's declared list; `p19_navigation` mock line disclosed); Rust suite, clippy, fmt, typecheck green; UI 369/369 (reviewer run, clean `7e72698`; known P33 C4(d) flake 1 of 10 runs); all 16 rows have a boundary test that fails on a wrong outcome (changed-input probe: ⌘Z from the dialog outside a field reaches the history, so C1(b) discriminates); assumptions within MC-30 and Delegated choices; H1–H8 hold; 4 backlog notes (custom Edit-menu ⌘Z inside a text field unverified live → owner try-out; multi-photo Split silent on refusals) |
| P42_camera_runtime | APPROVED | 2026-11-06 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 47 committed paths (`cd76ef3`, = handoff list: plan's 45 + `Cargo.lock` + plan Review log); Rust 606/0, clippy, fmt, typecheck, UI 369/369 green (reviewer run, clean `8f1facb`); all 26 rows have a boundary test that fails on a wrong outcome (C3(b) at the orchestrator-amended 8 s); `bind` last in `build`, `RunEvent::Exit` hook read; assumptions within Delegated choices; H1–H8 hold; 6 backlog notes (a failed sidecar spawn never retries and never raises `camera_sidecar_unavailable` → P46) |
| P43_ftp_receiver | APPROVED | 2026-11-06 | opus | re-review r3 APPROVED_WITH_NOTES: perimeter = the 1 allowed test file + tracker + log (`7e8178d`); F3 resolved (`c7b` pid wait 15 s, other settles and assertions unchanged); Rust suite 634/0, clippy, fmt, typecheck, UI 369/369 green (reviewer run, clean `1818599`); `p43` binary 19/20 green: run 8 red on a ~70 s OS stall of fresh fake-sidecar execs (3 apps held, receiver uninvolved) → triaged DROP, backlog → P44/testkit (copy the sidecar once per test process) |
| P44_capture_ledger | APPROVED | 2026-11-06 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 26 committed paths (`046d3c5`, = handoff list: plan's 24 + tracker + plan log); Rust 671/0 (p44 25/25), clippy, fmt, typecheck, UI 369/369 green (reviewer run, code tree `046d3c5`); all 36 rows have a boundary test that fails on a wrong outcome; association order, `RENAME_EXCL` with no fallback, pending-before-rename, take-over, recovery, freeze and rule 0 read against MC-33 R-I25…R-I29; no assumption contradicts 10E; H1–H8 hold; 5 backlog notes (recovery's EEXIST→duplicate does not re-evaluate the exposure; `first_raise` re-fires after each transfer → P46) |
| P45_sidecar_and_protocol | APPROVED | 2026-11-06 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 44 declared paths (`db3cdae`; `git ls-files native/` = 24 sources, no Sony file, no `sdk_shim`); suite 693 Rust, clippy/fmt/typecheck green, UI 369/369 on re-run (one unrelated P33 flake, backlog); all 21 rows have a boundary test that fails on a wrong outcome; C++ edits 1–7 read against the lab at `221aa49` and 10E R-I19–R-I23; fixture = handoff §4.2 verbatim; H1–H8 and the secrets rule hold; 7 new backlog notes |
| P46_direct_camera_adapter | APPROVED | 2026-11-07 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 38 committed paths (`64aa4f8`, = handoff list: plan's 34 + 2 disclosed test lines + tracker + plan log); Rust 723/0 (p46 25/25, desktop `p46_tests` 2/2, C++ 21/0 via P45 `c2a`), clippy, fmt, typecheck, UI 369/369 green (reviewer run, code tree `64aa4f8`); all 27 rows have a boundary test that fails on a wrong outcome (C8(c) on the disclosed reopen input, which is the one that can fail); event table, owned codes, clear rules, `Exited` reset, staging guard, watchdog glue and Wi-Fi-only reconnect read against the plan contract and 10E R-I19…R-I29/R-I37; no assumption contradicts 10E; H1–H8 hold; 6 backlog notes (`via` on a Wi-Fi Save-to-PC download, `ftp-status` re-fires after transfer gaps → P48, two `camera_sidecar_unavailable` texts) |
| P47_support_package_spike | APPROVED | 2026-11-08 | orchestrator + owner | manual spike, evidence `sony_camera_lab.md` §Spike S1–S7: cable and Wi-Fi connect from the packaged ad-hoc app; S2 needs `disable-library-validation`; S5 cannot see Capture One; S6 Keychain re-prompts after updates; S7 PASV/STOR, no 502; **Keychain reads block the camera start → P50 Notes (d)**; S1(c)/S3 second account → P52 |
| P48_camera_ownership | APPROVED | 2026-11-07 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 41 committed paths (`4da7fe9`, = handoff list: the plan's 43 minus the 2 disclosed unchanged; tracker + plan log committed by the runner); Rust 769/0 (p48 32/32, desktop `p48_tests` 2/2, unit `ownership`/`take_back` tests), clippy, fmt, typecheck, UI 369/369 green (reviewer run, code tree `4da7fe9`); all 38 rows have a boundary test that fails on a wrong outcome; gate on both lanes, deferral before each step's health handler, startup stops after a deferred ensure, process-wide gate/start order, take-back steps 1–6 + `running`-only preconditions, R-I33 store-first, sleep hold/release read against 10E MC-34 R-I30…R-I33, R-I23, M39, M40; no assumption contradicts 10E; H1–H8 hold; 6 backlog notes (gate check before the lane mutex → P51/P52, step 5 folder failure reported as take-back failure → P51) |
| P49_capture_one_alongside | APPROVED | 2026-11-07 | opus | re-review r3 APPROVED_WITH_NOTES: fix 2 perimeter (`3963477`) = `sqlite/store.rs` + `p49_capture_one_alongside.rs` + tracker + Review log, exactly as authorised; F2 resolved (`style_applied_at IS ?2 AND rendered_gen=0`, `rendered_gen` NOT NULL → never SQL NULL; missing row → false); tethered watcher capture with a style renders, no `apply_style`; F1 retouch test still green. Reviewer run on `e564532` (production = `3963477`): Rust workspace green (P49 21/21), clippy `-D warnings`, fmt, typecheck; UI 369/369 cited from fix 2 (no frontend change). 1 backlog note (new test reads the raw DB column) |
| P50_camera_settings_and_support | APPROVED | 2026-11-08 | opus | re-review r3 APPROVED_WITH_NOTES after authorised fix 2 (`00d112e`): perimeter = `runtime.rs` + `fake_secrets.rs` + p50 test + C5(e) row + tracker + Review log, as ruled; F3 resolved (`bind` reads `ftp-password` only when `support::installed` is `Some`, = the old `installed_sidecar` guard; read still on a thread; install → `support_changed` + `reopen` re-binds with the read). Rust 816/0 unskipped (the 5 real-Keychain `open_with` tests green), clippy, fmt, typecheck, UI 369/369 (reviewer run, code tree `00d112e`). Triage: C5(e) cannot fail on F3 (cache-enabled fake) and missing C5(d) row → DROP, backlog |
| P51_camera_setup_ui | APPROVED | 2026-11-08 | opus | re-review r2 APPROVED_WITH_NOTES: fix `6145db7` stayed in perimeter; F1 resolved (no Confirm or Not mine without the real fingerprint; the sidecar re-reports on its ≤30 s backoff, verified in code); C6(c) fails on the r1 code; UI 401/401 (intermittent p33 C4(d) red once, green on rerun), typecheck, cargo test, clippy, fmt green; backlog N5 (mismatch title while Settings is open), N6 (Not mine does not stop retries) |
| P51b_fingerprint_exact_confirm | APPROVED | 2026-11-08 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 25 committed paths (`6a06110`, = handoff list; `test_transport_manager.cpp` + `src-tauri/src/lib.rs` disclosed additions, both necessary); Rust 820/0 (p51b 4/4), UI 404/404, sidecar stub 21/0, clippy, fmt, typecheck green (reviewer run, code tree `6a06110`); all 13 rows have a boundary test that fails on a wrong outcome; `decide` read against the contract's case table (Match before consume, Replace/Pin only on display equality, drop otherwise, CLI flag only when nothing is pinned), single percent-decode at `main.cpp` like the credentials; MC-31 "pinned only on their confirmation" now holds end to end; no assumption contradicts 10E; H1–H8 hold; 2 backlog notes (policy checks inside the P45 percent-decoding case because of P45's fixed count; weak/strong `accept_fingerprint_on_next_connect` has no shared declaration) |
| P52_hardware_acceptance | IN_PROGRESS | 2026-11-09 | orchestrator + owner | **owner-attended, camera needed**: run 1 (dev Mac, 2026-11-08) partial, stopped by the owner after A15 to fix first: pass A4 A5 A7 A14 A18(USB, Wi-Fi), A9 so far, A12 with a note; fail A0 install (F1), A1 (F3), A3 (F4), A6 fallback (F6), A10 (F8), A15 (F9), A17 (F7); evidence §Acceptance (P52, 2026-11-08) → P53, P54, P55; run 2 (2026-11-09) after P53–P56: most rows pass, evidence §Acceptance run 2; F13 not winnable by timing, F6 hardware; F15–F21 + automatic take-back → P57; then the work Mac |
| P53_camera_upload_and_log | APPROVED | 2026-11-08 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 15 committed paths (`75b249b`, = handoff list); Rust suite (p53 7/7), UI 404/404, clippy, fmt, typecheck green (reviewer run, code tree `75b249b`); all 8 rows have a boundary test that fails on a wrong outcome (C3(a) also green alone, so its upload lines are its own); `226` after `.part`→`.done` rename only, filing spawned after the reply, `active` released before hashing, startup recovery of `.done`, listings include waiting files, same-name STOR while waiting → 450, gap age from the later file's arrival, reset rule, once-per-change gap log, top-level redaction read against MC-32/MC-33 and the plan contract; no assumption contradicts 10E; H1–H8 hold; 7 backlog notes (DEBUG provider calls vs 12 §Logs "every provider call", Sony remedy text in core → plan/book) |
| P54_camera_cable_and_take_back | APPROVED | 2026-11-08 | opus | re-review r2 APPROVED_WITH_NOTES: F1 (relaunch wait sends one `usb-retry`) and F2 (C2 driven by helper events) resolved; Rust 834/0, UI 407/407, helper 22/0, clippy, fmt, typecheck green on `01a0dba`; 2 new backlog notes; hardware check is P52 run 2 |
| P55_camera_setup_polish | APPROVED | 2026-11-08 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 16 committed paths (`12ffac6`, = handoff list); Rust 836/0, clippy, fmt, typecheck, helper stub green, UI 410/411 with only the known `p33_split_grid` intermittent (passes alone; P55 does not touch it) (reviewer run, tree `c5b9c76`); all 7 rows have a boundary test that fails on a wrong outcome; read-only probe: `manifest::verify` accepts Sony's real `CrSDK_v2.02.00_20260610a_Mac.zip` (7/7 files, 0.27 s); H1–H8 hold; immediate first restart deviates from MC-31's literal backoff list by plan (book acceptance "~1 s" supports it → book wording); 5 backlog notes |
| P56_camera_claim_and_render_loop | APPROVED | 2026-11-08 | opus (pipeline-reviewer) | review r1 APPROVED_WITH_NOTES: perimeter = the 24 paths in `ab6ee3d`, equal to the handoff's Files changed (out-of-plan infra/ports/migration/wiring/Health paths declared); reviewer run on clean `319afef` (code = `ab6ee3d`): 841 Rust / 411 UI, helper C++ green, clippy/fmt/typecheck green; all 8 rows have a boundary test that fails on a wrong outcome; H1–H8 hold; no assumption contradicts the book or the rulings; 7 backlog notes (`session test 1NNNN` naming unexplained, fingerprint covers `<DL>` only, holder flapping and slow-unheld interface → P52 run 2) |
| P57_camera_auto_take_back_and_run2_fixes | APPROVED | 2026-11-09 | opus | re-review r2 APPROVED_WITH_NOTES: F1/F2 resolved (fix `dc5769c`, test files only; both strengthened tests reviewer-verified red on pre-P57 `edb6f83`, green 3/3 on the fix). Rust 849/0, UI 413/413, clippy/fmt/typecheck green; 1 new backlog note (C4 count window); narrative written. Hardware try-out on the next camera run |
| P58_fast_compose_on_camera_preview | APPROVED | 2026-11-09 | opus | re-review r2 APPROVED_WITH_NOTES: F1/F2 resolved (fix `7549fc8`, perimeter within r1's allowed files). Rust 858/0, UI 418/419 with the plan's known-intermittent `p33_split_grid` C4(d) (7/7 alone), clippy/fmt/typecheck green; 1 new item triaged DROP → backlog (camera → master frame turns Compose off until the preview lands); narrative written. Hardware try-out on the next camera run |
| P59_item_type_from_shopify_product_type | APPROVED | 2026-11-09 | opus | re-review r2 APPROVED_WITH_NOTES: F1/F2 resolved (fix `a5ce4ab`, 7 paths, all within r1's allowed files). Rust 863/0, UI 422/422, clippy/fmt/typecheck green; no new blocking item, nothing to triage; r1's 5 notes stay in backlog; narrative written. Live-store try-out on the next hands-on run |
| P60_take_back_and_self_healing | APPROVED | 2026-11-09 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 23 declared paths (`6ed79bf`); Rust 874/0, UI 424/424, clippy/fmt/typecheck green (reviewer run, code tree `6ed79bf`); all 9 rows have a boundary test that fails on a wrong outcome; 12 assumptions, none contradicts the book (missing photos still stop export/publish per MC-3: owner choice in the narrative); H1–H8 hold; 9 backlog notes (hidden missing photo blocks its item's publish, refusal path untested, focus scan cost, per-arrival recipe read-back, dialog retry loop, P62 clean-up must not read as deletion) |
| P61_storage_cleanup_and_budget | APPROVED | 2026-11-10 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 30 declared paths (`214d730`); 8/8 rows hold at their boundaries (7 behaviour tests + 4 cleanup mechanism unit tests + 3 UI); Rust 885 passed / 0 failed, clippy/fmt/typecheck green (reviewer run); `npm test` 427/427 with `--maxWorkers=4`, plain `npm test` hit 5 s load timeouts in untouched compose tests (backlog); H1–H8 hold; 8 backlog notes |
| P62_retention_after_publish | APPROVED | 2026-11-10 | opus | re-review r3 APPROVED_WITH_NOTES: perimeter = storage.rs, arrival.rs, photos.rs, the P62 test file (`6f0a234`, all on FIX2_AUTHORISED's list); F2 resolved (retention only touches photos created at or before `cleared_at`; arrivals/imports into a cleared item go to Unassigned; `c1d`, `c1e` bite); Rust 900/0, clippy/fmt/typecheck clean (reviewer run, code tree `6f0a234`); 4 backlog notes (archived-item capture filed in its folder, `c1e` file location, re-deletion of restored RAWs, undo of an earlier move out of a cleared item) |
| P63_final_check_fixes | APPROVED | 2026-11-10 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 35 committed paths (`0247c78`, = handoff list; 23 outside the plan's named files, all disclosed and needed); reviewer ran p63 4/4, p48 32/32 in 10.3 s, p54/p57/p60/p62, core/integrations/desktop unit tests, clippy, fmt, typecheck, UI 442/442, sidecar stub 31/31 (full workspace trusted from the implementer's stamp, which stopped at a p62 flake that passes alone); all 8 rows have a boundary test that fails on a wrong outcome; one gate for every script, process-list `is_running`, missing-session rule, staging restart, leave guard, helper recovery policy read against MC-31/32/34 and 10C; no assumption contradicts the book; H1–H8 hold (H7 placement slip → note); 8 backlog notes |
| P64_publish_speed_and_visibility | APPROVED | 2026-11-10 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 29 declared paths (`d11a2d4`); 9/9 rows hold at their boundaries; P64 8/8, core mechanisms 64, integrations 36, UI publish/sessions/shell 73/73, clippy/fmt/typecheck green; implementer's partial workspace stamp consumed (noted); 8 backlog notes; handoff `handoffs/reviewer/handoff_P64_publish_speed_and_visibility_review_r1.md` |
| P65_capture_sync_waiting_jobs_and_inventory | APPROVED | 2026-11-11 | opus | review r1 APPROVED_WITH_NOTES: perimeter = declared paths across `b468cad`+`8b26018`+`b0edd78` (production swept into orchestrator commits, disclosed); P65 8/8 + touched testkit files (p01/p09/p29/p44/p62/p63/p64) + core/infra/integrations lib, clippy, fmt, typecheck, 125 touched UI tests green (reviewer run, clean `f46f1ba`); all 9 rows have a boundary test that fails on a wrong outcome; assumptions consistent with changelog 45; H1–H8 hold; 5 backlog notes (core error-text classifier, untracked fallback on any GraphQL error, Shopify fields unverified, stock section hidden on read failure, sweep commits do not build) |
| P66_approve_on_arrival_and_session_order | APPROVED | 2026-11-11 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 33 declared paths (`bc9ba7b`); p66 6/6 + touched testkit files (p01/p05/p17/p27/p39/p40/p44/p49/p61/p62/p64) + 87 touched UI tests, fmt, typecheck green (reviewer run, code tree = `bc9ba7b`); clippy stalled in the reviewer environment, implementer stamp consumed on the identical tree; all 7 rows have a boundary test that fails on a wrong outcome; H2 amended by changelog 46 only; 3 backlog notes (retire drops multi-photo undo steps, book/registry wording, tag after first preview) |
| P67_form_values_defaults_and_batch_step | APPROVED | 2026-11-11 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 27 declared paths (`a4ee3ae`); p67 7/7 + touched testkit files (p01/p11/p59/p64/p65) 42/42, touched UI tests (form, publish, history) 77/77, fmt, clippy -D warnings, typecheck green (reviewer run, code tree = `a4ee3ae`); implementer full-suite stamp consumed (p61 c1a load flake, green alone); all 7 automated rows have a boundary test that fails on the wrong outcome; assumptions A1–A8 consistent with the book; H1–H8 hold; 5 backlog notes |
| P68_setup_polish_address_password_shop | APPROVED | 2026-11-11 | opus | review r1 APPROVED_WITH_NOTES: perimeter = the 30 declared paths (`e39c105`); p68 3/3, p12 69/69, p50 22/22, core `secrets` unit, desktop `p50_tests` 2/2, touched UI tests (settings, shell, health, p07, p18) 102/102, fmt, clippy -D warnings, typecheck green (reviewer run, code tree = `e39c105`); implementer full-suite stamp consumed; all 5 automated rows have a boundary test that fails on the wrong outcome; assumptions A1–A8 consistent with the book (R-I21/09A wording superseded by the owner's rule: book fold); H1–H8 hold; 7 backlog notes |
| P69_faster_export_and_publish | IMPLEMENTED | 2026-11-12 | claude | 8-bit `PSM Master`; `PrepareMastersService` prepares Ready sessions' masters one at a time while Capture One is idle (`export.prepare_masters`, Settings → Storage); batch items go at most one ahead and the next item's export starts when one leaves prepare; one `publish.timing` line per item + popover `Masters ready in advance: n of m`; p69 7/7, Rust 945/0 (gate stopped at p56 c3c, fixed, p56–p69 re-run), UI 466/466, clippy/fmt/typecheck green |
| P70_fast_test_suite | NOT_STARTED | 2026-11-11 | orchestrator | test suite in minutes: one integration test program, injectable timings in tests (production values unchanged), optimised deps in test builds; owner 2026-11-11 (flow v1.4) |

Phase state machine (charter): `NOT_STARTED → [PROJECTED] → IMPLEMENTING → IMPLEMENTED → REVIEWING →
CHANGES_REQUESTED (→ IMPLEMENTING) → APPROVED`. The runner (`tools/pipeline/run_phase.sh`) drives it.

## 5. Sequencing and gates
`phases.txt` lists the 52 phases in execution order (17 MVP phases, the two real-run fix phases added
2026-10-26, the second real run's fix phases P19/P20 added 2026-10-29, the two tidy-up phases P21/P22 planned
2026-10-29 from `backlog.md`, P23_shape_on_arrival planned 2026-10-29 from book 10B, P24_capture_one_provisioning planned
2026-11-03 from book 10C, the second tidy-up pair P25/P26 planned 2026-11-04 from `backlog.md` and the P24 narrative, and the eight
composition phases P27–P34 planned 2026-11-04 from book 10D, the two composition-correction phases P35/P36 planned 2026-11-05 from the 10D amendment R38, P37_editor_chrome planned 2026-11-05 from the 10D amendment R39, the four phases P38–P41 planned 2026-11-05 from the 10D amendment R40 and the new 05A MC-30 (R41), and the eleven direct-camera phases P42–P52 planned 2026-11-06 from book 10E (R42–R45, inventory R-I18…R-I38)); the runner is sequential and a
phase starts only when every phase in its `depends_on` is APPROVED (checked by the implementer's gate). Dependency graph:

```
P00 → P01 → P02 → P03 ─────────→ P07 → P08 → P11 → P15 → P16 → P17 → P18 → P19 → P20 → P21 → P22 → P23 → P24 → P25 → P26 → P27 → P28 → P29 → P30 → P33 → P34 → P35 → P36 → P37 → P38 → P41
   ↘        ↘ P04 → P05 → P06 → P09 → P10 → P12 → P13 → P14 ↗                                                                          ↘ P31 → P32 ↗           ↘ P39 → P40 ↗
    ─────────↗            (P06 + P07 → P08; P09 + P12 → P13; P10 + P08 → P11; P14 + P11 → P15; P28 → P31; P30 + P32 → P33; P40 + P38 → P41)
P41 → P42 → P43 → P44 → P45 → P46 → P47 (spike, owner) → P50 → P51 → P52 (acceptance, owner)
                                   ↘ P48 → P49 ↗                 (P46 → P47; P46 → P48; P48 → P49; P47 + P49 → P50; P50 → P51; P51 → P52)
```
Exact `depends_on` per plan: P01[P00] · P02[P01] · P03[P02] · P04[P00,P01] · P05[P04] · P06[P05] ·
P07[P03,P04] · P08[P06,P07] · P09[P06,P02] · P10[P09] · P11[P10,P08] · P12[P10] · P13[P09,P12] · P14[P13] ·
P15[P14,P11] · P16[P15] · P17[P16] · P18[P17] · P19[P18] · P20[P19] · P21[P20] · P22[P21] · P23[P22] · P24[P23] · P25[P24] · P26[P25] ·
P27[P26] · P28[P27] · P29[P28] · P30[P29] · P31[P28] · P32[P31] · P33[P32,P30] · P34[P33] · P35[P34] · P36[P35] · P37[P36] · P38[P37] ·
P39[P37] · P40[P39] · P41[P40,P38] · P42[P41] · P43[P42] · P44[P43] · P45[P44] · P46[P45] · P47[P46] · P48[P46] · P49[P48] · P50[P47,P49] ·
P51[P50] · P52[P51] · P51b[P51] · P53[P51b] · P54[P53] · P55[P54] · P56[P55] · P57[P56] · P58[P57] · P59[P58] · P60[P59] · P61[P60] · P62[P61] · P63[P62] · P64[P63] · P65[P63] · P66[P65] · P67[P66] · P68[P67] · P69[P68] · P70[P69] (P53–P70: fixes from P52 run 1, orchestrator 2026-11-08; P52 run 2 after P55).
- **Direct camera adapter (planner 2026-11-06; book 10E).** Order: the risky, hardware-free core first. P42 (the one-camera-layer-per-process
  rule, R-I18, which every later camera phase builds on; the sidecar supervisor over a fake sidecar binary; the new `CameraStatus`), P43 (the
  FTP receiver, MC-32), P44 (the capture ledger, MC-33, the single worker and the no-replace rename), P45 (the lab sidecar imported without any
  Sony file, the stub build, the protocol client), P46 (the direct adapter: exposures, Save-to-PC, store destination, first copy wins). Then
  **P47, the support-package spike, owner-attended with the camera**, placed before P50 because P50's install/sign rules depend on its answers
  S1–S3 (library validation, Gatekeeper); the spike needs only P46's runtime and a hand-installed support folder (its plan says how). P48
  (camera ownership guard, MC-34) and P49 (Capture One alongside, CI1 as a one-line policy) do not depend on the spike, so the orchestrator may
  run them before P47 by reordering `phases.txt` when the owner cannot spare the camera (their `depends_on` allow it). P50 (settings, support
  package, secrets, health items, commands) and P51 (the setup UI) close the hardware-free work; **P52, hardware acceptance, owner-attended with
  the camera**, is last. Only P47 and P52 need the camera; every other phase's suite runs on fakes (fake sidecar binary, loopback FTP client).
  `projection: yes` on P42, P44, P45, P46 and P48 (silent-failure mechanisms: runtime lifetime, ledger, external sidecar fixtures, ownership).
  **No `implementer: fable`** anywhere (owner). The Capture One camera adapter (A2) stays the fallback whenever no camera support package is
  installed and is retired by a later tidy-up after P52 passes on the work Mac (10E §Port changes).
- **Editor polish and undo history (planner 2026-11-05).** P38 (frontend: the R40 polish of what P37 built) runs first so the owner sees the
  cheap corrections at once; P39 (Rust: the `History`, review steps, undo/redo, commands) and P40 (Rust: composition and attribute steps)
  carry `projection: yes` — MC-30 is a new load-bearing mechanism (charter rule 6) and both phases write reversal transactions whose silent
  failure is a wrong undo; P41 (frontend + the Tauri Edit menu) wires the keys, notices and icons and needs P40 (every step kind recorded) and
  P38 (the Composition section it adjusts). The frontend keeps working through P39/P40: `photos_undo` stays as a compatibility wrapper until
  P41 removes it. **No `implementer: fable`** (owner 2026-11-05): all four run on the default implementer.
- **Editor chrome (planner 2026-11-05).** P37 follows P36: it re-skins the controls P36 introduced (toolbar, hints, nudge pad, straightening
  grid switch) and moves the Tools panel's hide button, so it needs P36's `EditorToolbar`, `StraightenControl`, `Hint` usage and tests in place.
  Frontend only; **no `implementer: fable`** (the owner's rule of 2026-11-05 stands): default implementer.
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
  events are allowed, as before. **P45** — the implementer must be able to **read** the lab repo
  `/Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests` (Codex: reads outside the workspace are allowed by the sandbox; if refused,
  the orchestrator copies `sdk/` minus `sdk/tests/sdk_shim/` into `native/camera-sidecar/` before dispatch and says so in the prompt) and
  `cmake` + `clang` must be installed (§7.1: cmake 4.4.5, Apple clang 17). The SDK-dependent sources are compiled only by `tools/sidecar/build.sh
  --sdk <root>` on a machine holding the SDK (the dev Mac: the lab's `vendor/sony-sdk`); no session needs that build before P47. **P47 and P52** —
  owner present with the A7R V, the cable, the Wi-Fi network and the lab's camera configuration; the orchestrator runs them (`--pause` or outside
  the runner) and records results in the evidence; the runner never dispatches them.

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
| **10E (P42–P50)** `crates/core/src/domain/camera.rs`, `crates/core/src/mechanisms/{camera_status.rs, capture_ledger.rs}`, `crates/core/src/services/camera_ledger.rs`, `crates/core/src/ports/{camera.rs (grown), secrets.rs, system.rs}` | neutral camera domain (status, link, reasons, exposure, delivery), the status precedence and the ledger mechanisms, the `LedgerService` (single worker), the ports `SecretStore`, `ProcessList`, `SleepInhibitor`, `DeliveryFs` | — |
| `crates/infra/src/{ftp/ (mod.rs, server.rs, session.rs, peers.rs), camera_files.rs, keychain.rs, processes.rs, sleep.rs}` | the FTP receiver (hand-written over tokio, passive only), `rename_exclusive` (`libc::renamex_np` + `RENAME_EXCL`), `KeychainSecrets` (security-framework), `PsProcessList` (`ps -axo comm=`), `IokitSleepInhibitor` (`IOPMAssertionCreateWithName`) | psm-core, tokio, libc, security-framework, ipnet |
| `crates/integrations/src/camera/{runtime.rs, sidecar/ (supervisor.rs, protocol.rs, client.rs, fixtures/*.jsonl), direct/ (adapter.rs, filenames.rs, ownership.rs, take_back.rs), support/ (mod.rs, sony.rs, manifest.rs)}`; `crates/integrations/src/editing/capture_one/{launch_gate.rs, follow.rs}` | the process-wide `CameraRuntime`, the sidecar process supervisor and protocol client, the direct (Sony SDK sidecar) adapter, the camera ownership guard and "Take the camera back", camera support packages, the editor-launch gate, the follow service (current collection on every arrival and activation, R46; variant wait; style once) | psm-core, psm-infra, zip, getrandom |
| `native/camera-sidecar/` (CMake project; **no Sony file, no `sdk_shim`**), `tools/sidecar/{build.sh, manifest.sh}`, `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` (**gitignored**, built by `build.rs` as the stub when absent) | the C++ camera sidecar copied from the lab's `sdk/` (P45), its mock-camera tests, the stub backend for machines without the SDK | CMake ≥ 3.20, Apple clang; the Sony SDK only on the build machine |
| `crates/testkit/src/{bin/fake_sidecar.rs, ftp_client.rs, fake_secrets.rs, fake_processes.rs, fake_sleep.rs, fixtures/sidecar_scripts/*.txt}` | the scripted fake sidecar binary (`CARGO_BIN_EXE_fake_sidecar`), the minimal FTP test client (STOR/REST/APPE/ABOR, RST by `SO_LINGER 0`), fakes for the new ports | psm-core, psm-infra |

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
**Camera crates (verified on crates.io 2026-11-06 by the planner):** `libc = "0.2"` (0.2.191 is already in the local registry; it exports
`renamex_np` and `RENAME_EXCL` for Apple targets — `libc-0.2.191/src/unix/bsd/apple/mod.rs:3557,5010`; **not** the 1.0.0-alpha line),
`security-framework = "3"` (3.7.0; `passwords::{set_generic_password, get_generic_password, delete_generic_password}`), `ipnet = "2"` (2.12.2;
peer ranges), `zip = "9"` (9.4.1; the support archive is extracted by the app so no quarantine attribute reaches the copies), `getrandom = "0.4"`
(0.4.3; the FTP password). No FTP server crate: the receiver is hand-written over the `tokio` already pinned (P43 Notes say why). No `cmake`
crate: `build.rs` invokes `/usr/bin/env cmake` directly.
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
- **10E (P42–P50):** under the data dir, `camera/sony/<sdk version>/` (the installed support package: the Sony runtime dylibs, `CrAdapter/`,
  **and a copy of the bundled sidecar**, `camera-sidecar`, signed ad hoc after copying; P50) and `camera/fingerprints/` (pinned SSH host-key
  fingerprints, `CRLAB_FINGERPRINT_DIR`; never committed). Under the **capture root** (`canon(<document folder>/Capture)`, 05A MC-2):
  `.incoming/.<name>.part` (the receiver's staging files, kept on failure for a resume), `.incoming/sdk-<sidecar run id>/` (Save-to-PC
  downloads of one sidecar process), `.duplicates/` (identical later copies, never a photo, never deleted). `is_candidate` already ignores every
  dot-prefixed component (`mechanisms/arrival.rs:23–28`). `AppPaths::create` adds `camera` and `camera/fingerprints`.
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
**P39** removes `StoreTx::set_undo_slot` and adds `StoreTx::delete_photo_row(id)` (the `photos` row and its `compositions` row) and
`StoreTx::publish_confirmed_since(session_id, seq) -> bool` (any `publish_items` row of the session with `confirmed_change_seq >= seq`; the
copy-undo guard, projection D5); **P40** adds
`StoreTx::composition(photo_id)`, `StoreTx::save_composition(photo_id, &CompositionValues, Origin, now_ms) -> (Composition, bool)` (the same body
as `Store::save_composition`, shared through `save_composition_in(conn, …)`) and `StoreTx::delete_attribute_value(item_id, key)`. `EventSink`
gains nothing; `AppEvent::HistoryChanged` (§6.7) is emitted by `History`.
**10E (P42–P50; the exact signatures are in §6.11):** `CameraProvider` becomes `{ raw_extensions, status() -> CameraStatus, capture() ->
Result<CaptureId>, capabilities() -> CameraCapabilities, reconnect() -> Result<()> (P46), request_ftp_status() (P46) }`; `CameraStatus` moves to
`domain/camera.rs` and grows to five variants (R-I37). New ports: `CaptureSink` (the ledger's inbound face: `exposure_observed`, `delivery`,
`session_uncertain`, `transfer_in_progress`), `ArrivalSource` (what the receiver and the Save-to-PC path are: `via`, `transfer_in_progress`),
`SecretStore` (Keychain), `DeliveryFs` (`rename_exclusive`, `create_dir_all`, `file_size`), `ProcessList`, `SleepInhibitor`, and from P44
`CompletedNames { fn completed_names_since(&self, since_ms: i64) -> Vec<(String, u64, i64)> }` in `psm_core::ports::camera` (defined by P43 in
`psm_infra::ftp`, moved to core by P44 because core's `LedgerService` implements it; `psm_infra::ftp` re-exports it). `Store`/`StoreTx` gain the
ledger methods (P44; the list is in §6.11 Services) and `StoreTx::item_folder_key_frozen(item_id) -> bool` (05B MC-22 at a committed delivery:
**derived, no write** — `item_has_photos OR` a `photo`/`renamed_photo` delivery in state `pending` or `done` for one of the item's sessions; a `Store`
variant with the same body for tests; replaces the earlier `freeze_folder_key`, struck after P44 projection r0 L9/L23). `CaptureOneScripting` gains `quit()`
(P48), `set_current_collection(id, folder)`, `apply_style(id, variant, name)` (P49) and loses `import` (P49);
`EditingProvider` gains `set_current_collection(folder)`, `apply_style(raw, name) -> StyleOutcome`, `find_variant(raw) -> bool` and loses `import`.

### 6.5 Tables (migration `001_init.sql` in P00: `settings`, `capture_events`; `002_domain.sql` in P01: the rest)
All ids `INTEGER PRIMARY KEY`; all `*_at` are UTC ms integers (MC-4); enums stored as snake_case TEXT; JSON as TEXT.
| Table | Columns |
|---|---|
| `settings` | `key` PK, `value` — **app state**, not user config: `mechanisms_version`, `active_session_id` |
| `capture_events` | `id`, `source` (`hook`/`watcher`/`scan`/`import`), `hook` (nullable), `payload`, `received_at`, `error` |
| `items` | `id`, `sku` UNIQUE, `folder_key`, `folder_key_ci` UNIQUE (lower-cased, MC-8), `barcode`, `item_type`, `title`, `origin`, `created_at`, `updated_at` |
| `item_links` | `item_id`, `integration`, `external_id`, `external_url`, `last_synced_at`, `create_pending`, `tags_base` (JSON), PK(`item_id`,`integration`) |
| `sessions` | `id`, `item_id` (NULL only for Unassigned, which is row id 1 created by the migration), `published_at`, `published_seq` DEFAULT 0, `change_seq` DEFAULT 0, `last_activity_at`, `created_at`, `undo_slot` (JSON, MC-5 single slot — **unused from P39**: MC-30's history is in memory; the column stays, never written again) |
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
| **006** (P44) `006_camera_ledger.sql` | `camera_exposures(id, capture_id TEXT NULL, exposed_at INTEGER NOT NULL, initiated TEXT NOT NULL (`app`/`camera`), session_id INTEGER NULL, status TEXT NOT NULL (`expecting`/`received`/`missing_at_deadline`/`uncertain`/`dismissed`), expected_files INTEGER NULL, created_at INTEGER NOT NULL)`; `camera_deliveries(id, exposure_id INTEGER NULL, camera_filename TEXT NOT NULL, file_key TEXT NOT NULL, sha256 TEXT NOT NULL, size INTEGER NOT NULL, via TEXT NOT NULL (`cable`/`wireless`), completed_at INTEGER NOT NULL, association TEXT NOT NULL (`filename`/`stem`/`fifo`/`none`), outcome TEXT NOT NULL (`photo`/`duplicate`/`renamed_photo`), session_id INTEGER NULL, written_path TEXT NOT NULL (the staging path, needed by R-I25's recovery sentence and the re-send take-over; added after P44 projection r0 L7 — the R-I25 column list is short of it), final_path TEXT NOT NULL, state TEXT NOT NULL (`pending`/`done`), created_at INTEGER NOT NULL)`; indexes on `camera_deliveries(file_key)`, `(final_path)`, `(written_path)`, `camera_exposures(status, exposed_at)`. `missing_at_deadline` is reserved: nothing writes it in P44. Rows are kept (10E R-I25) |
| **007** (P49) `007_style_applied.sql` | `ALTER TABLE raw_state ADD COLUMN style_applied_at INTEGER` (10E R-I35: the style is applied at most once per RAW) |

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
  `sku_collision{existing_sku}`, `sku_in_shop{sku}`, `target_has_raw`, `target_published`, `nothing_to_undo`, P39: `nothing_to_redo`,
  `step_cannot_apply{label, reason}` (message `Can't undo|redo <label>: <reason>`; reasons `that session is published`, `that session already
  has this photo`, `that photo is gone`, `that photo already changed`, `the copy is gone`; P40 adds `that photo changed in Capture One — re-frame it first`, `that photo has no frame`,
  `the framing no longer fits this photo`, `that product is gone`),
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
- **10E (P42–P51).** `psm_core::domain::camera::{CameraStatus {Connected {name, link}, Connecting {link}, Busy {reason}, NeedsAction {reason,
  holder: Option<String>, message: Option<String> /* P48: the MC-34 sentence for `HeldByOtherProgram`, `None` for every other reason */}, Disconnected}, Link {Cable, Wireless}, BusyReason {Exposure, Transfer, Switching}, NeedsActionReason
  {AuthenticationRefused, FingerprintMismatch, FingerprintUnconfirmed, PhoneAppConnected, HeldByOtherProgram, StoreDestinationUnverified,
  FilesNotReceived {count}, SupportNotInstalled} (this is the precedence order), CaptureId(String), ExposureInitiator {App, Camera},
  ExposureObserved, Delivery, CameraCapabilities {status, capture, transfer, exposure_events, settings_read, settings_write_store_destination,
  depends_on_editing_provider}}` (serialized snake_case; `CameraStatus` moves out of `domain/editing.rs`). Ledger enums: `ExposureStatus
  {Expecting, Received, MissingAtDeadline, Uncertain, Dismissed}`, `Association {Filename, Stem, Fifo, None}`, `DeliveryOutcome {Photo,
  Duplicate, RenamedPhoto}`, `DeliveryState {Pending, Done}`.
  Error codes: `camera_not_connected`, `editor_launch_deferred` (P48; a script that would launch Capture One was refused while "Take the camera
  back" holds the gate), `take_back_blocked{reason}` (P48; reasons in check order `a file is still arriving`, `a photo is still expected`, `an export is
  running`, `a publish is running`, `a preview is rendering`, `Capture One is not holding the camera`, and `already running` for a second `run` while one
  is in progress; "running" jobs = status `running` only, P48 projection r0 L25), `take_back_failed{step, cause}` (P48; the step number 2–5 and the cause;
  `CommandError.data` is a `BTreeMap<String, String>`, so on the wire `step` is the string `"2"`…`"5"`, never a number — L18), `reconnect_on_cable_refused` (P46), `sidecar_unavailable` (P46), `support_archive_invalid{file}`,
  `support_version_unsupported{version}`, `support_not_installed` (P50), `fingerprint_not_pending` (P50).
  Health codes (`HealthItem{code, ok, detail}`; the camera dot shows the first that holds in the precedence order, the Health page lists them
  all — R-I37): ok item `camera` (detail `<camera name> · cable` / `… · wireless`; replaces P04's `camera` detail), `camera_disconnected`
  (kept), `camera_connecting` (detail `Connecting over the cable…` / `… Wi-Fi…`), `camera_busy` (detail `Taking a photo` / `Receiving a
  photo` / `Switching connection`), `camera_auth_refused` (`The camera refused the password`), `camera_fingerprint_mismatch` (`The camera's
  fingerprint changed: confirm it in Settings → Camera before shooting over Wi-Fi`), `camera_fingerprint_unconfirmed` (`Confirm the camera's
  fingerprint`), `camera_phone_connected` (`A phone app is connected to the camera: disconnect it`), `camera_held` (P48; detail = the MC-34
  sentence from `name_programs` (R-I30): `Capture One is holding the camera's USB connection — Take the camera back` whenever Capture One runs;
  otherwise `programs that can take the camera: Imaging Edge Desktop, Adobe Lightroom — quit it and stop it opening at login` (the running known
  programs, in list order, with the first one's guide); `The system camera service is holding the camera: close apps that use cameras` when none
  runs; `<owner> is using the camera: close it or stop it from using cameras` for a holder other than `ptpcamerad`; the same sentence is
  `CameraStatus::NeedsAction.message`), `camera_store_destination_unverified` (`The camera did not confirm where photos are saved`),
  `camera_files_not_received` (`<n> photos taken but not received for 60 s`, plus ` · <g> gaps in the camera's numbering` when any, plus ` · camera FTP:
  <key>=<value>, …` once a `camera.ftp_state` answer arrived (P46); **its single writer is the ledger's `watchdog`/`dismiss`** — the direct adapter
  publishes every other camera code it owns (`camera`, `camera_connecting`, `camera_busy`, `camera_disconnected` and the other `NeedsActionReason::code()`s)
  and never this one, P46 projection r0 L11/L13),
  `camera_photo_session_uncertain` (P44; `Photo <camera filename> may belong to <other session title>`), `camera_support_not_installed`
  (P50; `Add camera support in Settings → Camera`), `editor_not_following` (P48; `Capture One will not show this session's new photos until
  it is reachable`), `camera_sidecar_unavailable` (P42; `The camera helper could not start`).

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
reason}]}` · **P39** `history_get(session_id) -> {undo: Option<String>, redo: Option<String>}`, `history_undo(session_id) -> {label, direction:
"undone" | "redone", sessions: [SessionResponse], active_photo_id}`, `history_redo(session_id)` (same DTO; errors `nothing_to_undo`,
`nothing_to_redo`, `step_cannot_apply`); `photos_undo` answers in its old `ReviewResponse` shape through the history until **P41** removes it ·
**P40** `composition_apply` calls `CompositionService::apply` once (one step) · **P41** `edit_menu_set(undo: Option<String>, redo: Option<String>)`
(sets the Edit menu items `edit.undo` / `edit.redo`: text `Undo <label>` / `Undo`, enabled when set; accelerators `CmdOrCtrl+Z`, `CmdOrCtrl+Shift+Z`) ·
**P42** `camera_status` / `camera_retry` answer the new `CameraStatusResponse` (tag `kind`: `connected {name, link}`, `connecting {link}`, `busy
{reason}`, `needs_action {reason, holder, message}`, `disconnected`; `link` = `cable` | `wireless`, `reason`s snake_case, `message` = the
health detail sentence) · **P46** `camera_capture -> { capture_id: string }`, `camera_reconnect` (Wi-Fi only; `reconnect_on_cable_refused`),
`camera_files_not_received_dismiss` · **P48** `camera_take_back -> { completed_steps: number, camera_held: boolean }` (errors `take_back_blocked{reason}`, `take_back_failed{step, cause}`;
every `CommandError.data` value is a string, so `step` arrives as `"3"`, not `3` — P48 projection r0 L18),
`camera_test_connection -> { holder: string | null, programs: string[], sentence: string | null }` · **P50** `camera_support_get -> { make:
"sony", version, sidecar_path } | null`, `camera_support_install(archive_path) -> CameraSupportDto`, `camera_secrets_set({ access_auth_user,
access_auth_password })` (writes the Keychain; never echoed back), `camera_ftp_password_generate -> { password }` (the one time it is shown),
`camera_fingerprint_confirm`, `camera_fingerprint_reject`; `settings_get` / `settings_set` carry the `camera` block (§6.8).
From P17 every command is `pub async fn` over `AppState(RwLock<App>)` through `AppState::handles()` / `mutate()`
(`state.rs`), work inside `tauri::async_runtime::spawn_blocking`; no command holds the app lock across provider work.
Events (`AppEvent` → Tauri event name): `session_changed{session_id}`, `photo_added{photo_id, session_id}`,
`photo_changed{photo_id, session_id}`, `publish_progress{session_id, item_id, step, uploaded, total}`,
`export_progress{session_id, done, total, state}`, `health_changed{}`, `camera_status_changed{status}` (P42: the payload is the new
`CameraStatusResponse`; in P42 only `camera_retry` emits it, through the App's `EventHub` — `TauriCameraEvents` is deleted; from P46 the runtime emits it
on every status change), P39 `history_changed{session_id, undo,
redo}` (the labels of the steps ⌘Z / ⌘⇧Z would act on, after every record, undo, redo and dropped step), P41 `edit_menu{action: "undo" | "redo"}`
(emitted by `on_menu_event` for the Edit menu's two items; not an `AppEvent`), P50 `camera_fingerprint_pending{fingerprint}` (the sidecar
reported an unpinned Wi-Fi fingerprint; Settings → Camera shows it for confirmation).
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
**10E (P43, P50):** `camera` (object, defaults filled on read and shown by Settings → Camera): `{"receiver_port": 2121,
"passive_port_start": 50000, "passive_port_end": 50009}`; validation in `SettingsService::set`: each port 1024–65535, `passive_port_start ≤
passive_port_end`, at least 2 passive ports (MC-32 guarantee 5: two concurrent uploads), the control port outside the passive range; a save
that changes any of the three closes and reopens the receiver's listener **in place** (R-I18: never two listeners; the sidecar is untouched).
Nothing secret is ever in `settings.json`: the camera's Access Authentication user and password and the FTP password live in the Keychain
(service `com.loorenz.sessionmanager.camera`, accounts `access-auth-user`, `access-auth-password`, `ftp-password`; R-I21). The receiver's FTP
**username** is the constant `sessionmanager` (not a setting; shown in camera setup).
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
- **10E (P42–P50):** the sidecar is never real in the suite: `crates/testkit/src/bin/fake_sidecar.rs` (`[[bin]] fake_sidecar` of psm-testkit,
  reached from `crates/testkit/tests/*` as `env!("CARGO_BIN_EXE_fake_sidecar")`) speaks the stdio protocol from a script file (P45 §The fake
  sidecar), records every stdin line and its own argv/env into files under the test's temp dir, and can exit on command. The FTP receiver is
  tested at its socket boundary on `127.0.0.1:0` through `psm_testkit::ftp_client` (a minimal hand-written client, so a reset mid-transfer is
  a real `RST`). The C++ transport-manager tests run through `ctest` from `tools/sidecar/build.sh --stub` (one Rust test in
  `p45_sidecar_and_protocol.rs` drives it; build dir under `target/camera-sidecar/`). Sidecar event fixtures (`crates/integrations/src/camera/
  sidecar/fixtures/*.jsonl`) are the lab's real event lines (handoff §4.2) plus lines whose shape is cited to a `Fields` emit site in the
  lab's C++ (rule 17; P45 names each).

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
- **Editor chrome (P37, book 10D amended `3c7623e`, R39, changelog 29; planned 2026-11-05).** `components/ui/tooltip.tsx` → `type HintCard = {
  name, key?, description }`, `Hint({ text: string | HintCard, children })`; a card's tooltip text is exactly `name · key — description` (U+00B7,
  U+2014) or `name — description` without a key, rendered as `.hint-card` > `.hint-name`, `kbd.hint-key`, `.hint-description`. Accessible names
  `Hide Tools panel` (in `.tools-panel-head`, with `.tools-panel-title` "Tools"), `Show Tools panel` (the only button of `aside.tools-rail[aria-
  label="Tools"]`, rendered **instead of** `.tools-panel` while hidden — the `hidden` attribute is no longer used), `Rotate and flip` and `Nudge`
  (drawer toggles, `aria-expanded`, `aria-controls` → `#compose-rotate-flip.compose-drawer-body`, `#compose-nudge-pad[role="group"]`; open state in
  `ComposeMode`, closed at every mount, not remembered); icon-only buttons keep their P36 names (`Revert`, `Loupe`, `Snapping`, `Copy framing`,
  `Paste framing`, `Turn frame`, `Rotate left`, `Rotate right`, `Flip`, `Straightening grid` (switch), `Nudge up|left|down|right` in that DOM
  order). `ToolsPanel` prop `onToggleHidden`. CSS `.tools-panel-head`, `.tools-panel-title`, `.tools-rail`, `.compose-drawer`,
  `.compose-drawer-toggle`, `.compose-drawer-body`, `.compose-nudge-pad`, `.hint-card`, `.hint-name`, `.hint-key`, `.hint-description`.
  **Retired:** the stage-bar buttons `Tools panel` and `Frame meter` (⌘T and ⌥⌘P stay keys), `EditorToolbar` props `tools`/`onTools`/`meter`/
  `onMeter`, `.compose-buttons`, the dead `.compose-readout` and `.compose-straighten-grid + path`. P36's table T is superseded by P37's T2.
  `localStorage` keys unchanged. No command, table, setting or Rust change.
- **Editor polish (P38, book 10D amended `3e4b760`, R40, changelog 30; planned 2026-11-05).** Split line: in the editor two SVG lines at the
  frame's middle, `line.compose-middle-edge` (`strokeWidth 4`, dark) under `line.compose-middle` (`strokeWidth 2`, light); on the stage and tiles
  `.composed-split` becomes 2 px white with a 1 px dark `box-shadow`. On/off icons (`Loupe`, `Snapping`, `Straightening grid`): the lucide icon
  gets `fill={on ? 'currentColor' : 'none'}` and the button `data-on`; CSS `.compose-tool[data-on]` = accent. **Rotate and flip** = Radix
  `DropdownMenu` (trigger `button.compose-tool[aria-label="Rotate and flip"]`, `aria-haspopup="menu"`; content `.compose-menu` with `menuitem`s
  `Rotate 90° left`, `Rotate 90° right`, `Flip horizontal` as `.compose-menu-item`; closes after a pick). **Nudge** = Radix `Popover`
  (`modal={false}`; trigger `button.compose-tool[aria-label="Nudge"]`; content **is** `#compose-nudge-pad[role="group"][aria-label="Nudge"]`, never
  `role="dialog"` — `LAYER_OPEN` would switch the editor keys off; the four `Nudge up|left|down|right` buttons as P37; closes on an outside
  `pointerdown` and Esc). The three icons sit in `.compose-degrees` after the `°` span (grid, rotate, nudge); `StraightenControl` takes
  `children`. Hover table **T3** = T2 minus the three rotate/flip entries (29). **Retired:** `.compose-drawer`, `.compose-drawer-toggle`,
  `.compose-drawer-body`, `#compose-rotate-flip`, buttons `Rotate left` / `Rotate right` / `Flip`, props `rotateFlipOpen` / `onRotateFlipOpen` /
  `nudgeOpen` / `onNudgeOpen`. No command, table, setting or Rust change.
- **Undo history (P39–P41, book 05A MC-30, M36, R41, changelog 30; planned 2026-11-05).** `crates/core/src/services/history.rs`:
  `HISTORY_LIMIT = 25`; `HistoryStore = Arc<Mutex<HashMap<session_id, SessionHistory { undo: VecDeque<Step>, redo: Vec<Step> }>>>`; `History {
  store, events: Arc<dyn EventSink> }` with `new`, `with_store`, `store()`, `record(session_id, Step)` (drops the oldest beyond 25, clears redo,
  coalesces consecutive single-field `Attribute` steps of the same `(item, key)` while redo is empty, emits `HistoryChanged`), `state(session_id)
  -> HistoryState { undo, redo }`; `Step { label, kind: StepKind }`; `StepKind::{ Classify { session_id, photos: Vec<PhotoPlace> }, Reorder {
  session_id, photo_id, from, to }, Move { source, target, photos }, Copy { source, target, seq_after, copies: Vec<(source_id, copy_id)> },
  Composition { session_id, photos: Vec<CompositionChange { photo_id, before: (CompositionValues, Origin), after }> }, Attribute { session_id,
  item_id, fields: Vec<FieldChange { key, before: FieldState, after }> } }`; `Place { classification, position }`, `PhotoPlace { photo_id, before,
  after }`, `FieldState::{ Row(Option<Value>), Title(String) }`; `HistoryService { store, clock, history }::{ undo, redo } -> HistoryOutcome
  { label, direction: Direction::{Undone, Redone}, sessions, active_photo_id }`. Recording happens **after** the transaction committed, in
  `ReviewService` (P39; `ReviewService::new(store, clock, &History)`; `PhotosService.history`), `CompositionService` (P40; field `history:
  Arc<History>`) and `AttributesService` (P40; `new(store, clock, integration)` unchanged + `with_history(&History)` — the two command sites attach it, schema refresh does not). Every reversal is
  one `Store::transaction` (rollback on `Err`) run under **one operation lock per `History`** (take → transaction → place/drop; lock order History
  → store; `HistoryChanged` emitted after release — projection D10); **only `Error::StepCannotApply` drops the step**, any other error leaves the
  lists as they were (D11); a reversal whose computed state equals the current one writes nothing and touches nothing (D12); photos re-inserted
  into a group go in **ascending `before.position`** (positions read before the whole operation, D4); copy undo is refused once any publish was
  confirmed after the copy (`publish_confirmed_since`), repoints the RAW's mirror job to the surviving row, and redo rewrites `seq_after` (D5, D8,
  D9). **Labels** (P39 §Labels): `Approve 03`, `Condition 02`, `Delete 03` / `Delete photo`, `Unreview 03` / `Unreview photo`, `Approve 3 photos`,
  `Reorder 03 → 01`, `Move to <title|SKU|Unassigned>`,
  `Move 2 photos to …`, `Copy to …`, `Compose 03` / `Compose photo`, `Apply framing to 4 photos`, `Edit <label>`, `Accept all suggestions`, `Clear
  <label>`. `App.history: Arc<History>` and `AppHandles.history`; `SettingsSetContext.history_store`; `App::build(…, history: Option<HistoryStore>)`
  (carried across a runtime settings rebuild, built with the **new** `events` hub; empty after a relaunch). `TestApp::{history(), attributes(…)}`.
  **Retired:** `UndoSlot`, `StoreTx::set_undo_slot`, `ReviewService::undo` (P39); `photos_undo`, `services/photos.ts undoDelete`,
  `WorkspaceKeys.undo` (P41). **Frontend (P41):** `services/history.ts` (`historyGet`, `historyUndo`, `historyRedo`, `editMenuSet`,
  `onHistoryChanged`, `onEditMenu` — the listeners live here, not in `events.ts`, so the eleven tests that enumerate `events.ts`'s exports keep
  passing); `features/history/{useHistory.ts, HistoryButtons.tsx, useHistoryKeys.ts}`; header buttons `Undo`, `Redo` (`.history-button`, icon-only,
  `aria-disabled` when none; cards `Undo · ⌘Z — <label>` / `… — Nothing to undo`, `Redo · ⌘⇧Z — …`); toast id `history`, texts `Undone: <label> ·
  Redo ⌘⇧Z`, `Redone: <label> · Undo ⌘Z`, `Nothing to undo.`, `Nothing to redo.`; ⌘Z / ⌘⇧Z handled at window level while a session is open,
  never when `isTyping(target)`; the editor dispatches `Saved` when the stage photo's stored revision changes without a pending save; a grid
  Split/Normal on ≥ 2 photos goes through `composition_apply`. **Tauri Edit menu (P41):** `Menu::default` with Edit rebuilt as `edit.undo`,
  `edit.redo` (custom `MenuItem`s), separator, cut, copy, paste, select all; `EditMenu { undo, redo }` in managed state; `on_menu_event` →
  `edit_menu{action}`.

### 6.11 Direct camera skeleton (book 10E, planned 2026-11-06; the names P42–P52 share)
- **Constants** (`psm_core::domain::camera` unless noted): `FIFO_WINDOW_MS = 300_000`, `WATCHDOG_MIN_AGE_MS = 60_000`, `WATCHDOG_MAX_AGE_MS =
  1_800_000`, `TRANSFER_EXPECTED_MS = 10_000` (busy `Transfer` from `capture.exposed` on a USB session to the matching completion),
  `COMPLETED_NAMES_WINDOW_MS = 86_400_000` (LIST/NLST/SIZE/MDTM answer for names completed in the last 24 h); `psm_integrations::camera::sidecar::
  {RESTART_BACKOFF_S: [u64; 5] = [1, 2, 4, 8, 16], RESTART_STEADY_S = 30, QUIT_GRACE_MS = 3_000}`; `psm_integrations::camera::direct::
  {START_ORDER_WAIT_MS = 15_000, TAKE_BACK_QUIT_WAIT_MS = 20_000, TAKE_BACK_FREE_WAIT_MS = 5_000, TAKE_BACK_READY_WAIT_MS = 15_000,
  TAKE_BACK_EXPOSURE_GRACE_MS = 10_000, USB_FAILURES_BEFORE_WIFI = 3 (mirrors the sidecar's `usb_failures_before_wifi_fallback`)}`;
  `psm_integrations::editing::capture_one::follow::{VARIANT_WAIT_POLL_MS = 1_000, VARIANT_WAIT_MAX_MS = 15_000}` (the current-collection rule is fixed by
  R46 — switch on every arrival while Capture One shows another collection and on every activation, frontmost or not — so there is no policy constant); `psm_infra::ftp::{FTP_USERNAME = "sessionmanager", DEFAULT_CONTROL_PORT = 2121,
  DEFAULT_PASSIVE = 50000..=50009}`; `psm_core::domain::camera::secrets::{SERVICE = "com.loorenz.sessionmanager.camera", ACCESS_AUTH_USER =
  "access-auth-user", ACCESS_AUTH_PASSWORD = "access-auth-password", FTP_PASSWORD = "ftp-password"}`; `FTP_PASSWORD_LEN = 32` over `[A-Za-z0-9]`.
- **Domain** (`crates/core/src/domain/camera.rs`): the enums of §6.6; `ExposureObserved { capture_id: Option<CaptureId>, at: i64 /* Mac-clock UTC
  ms = the sidecar envelope's timestamp */, initiated: ExposureInitiator, provisional: bool }`; `Delivery { camera_filename: String /* as the
  camera named it, after the adapter's normalisation */, written_path: PathBuf, sha256: String, size: u64, via: Link, completed_at: i64 }`;
  `CameraName(String)`.
- **Ports** (`crates/core/src/ports/`): `camera.rs` — `CameraProvider { fn raw_extensions(&self) -> &'static [&'static str]; fn status(&self) ->
  Result<CameraStatus>; fn capture(&self) -> Result<CaptureId>; fn capabilities(&self) -> CameraCapabilities; fn reconnect(&self) -> Result<()>;
  fn request_ftp_status(&self) -> Result<()> }` (the last two added in P46 as **default methods** answering `Err(Error::Provider("not supported"))`, so the
  Capture One adapter and `FakeDirectProvider` inherit the refusal untouched and only the direct adapter overrides them; P46 projection r0 L14),
  `CaptureSink { fn exposure_observed(&self, e: ExposureObserved) -> Result<i64>; fn delivery(&self, d: Delivery) -> Result<DeliveryOutcome>; fn
  session_uncertain(&self) -> Result<()>; fn transfer_in_progress(&self, source: Link, active: bool) }`, `ArrivalSource { fn via(&self) -> Link;
  fn transfer_in_progress(&self) -> bool }`, `DeliveryFs { fn rename_exclusive(&self, from: &Path, to: &Path) -> Result<RenameOutcome {Moved,
  Exists}>; fn create_dir_all(&self, dir: &Path) -> Result<()>; fn file_size(&self, path: &Path) -> Result<u64> }`; `secrets.rs` — `SecretStore
  { fn get(&self, account: &str) -> Result<Option<String>>; fn set(&self, account: &str, value: &str) -> Result<()>; fn delete(&self, account:
  &str) -> Result<()> }`; `system.rs` — `ProcessList { fn running(&self) -> Result<Vec<String>> /* process names */ }`, `SleepInhibitor { fn hold(&self,
  reason: &str) -> Result<()>; fn release(&self) }`.
- **Mechanisms** (`crates/core/src/mechanisms/`): `camera_status.rs` — `StatusInputs { needs_action: Vec<(NeedsActionReason, Option<String> /* holder */, Option<String> /* message; P48: the MC-34 sentence for `HeldByOtherProgram` */)>,
  busy: Option<BusyReason>, session: Option<(CameraName, Link)>, connecting: Option<Link>, helper_running: bool }`, `derive_status(&StatusInputs)
  -> CameraStatus` (R-I37 order; `NeedsAction` chosen by `NeedsActionReason::rank()`), `health_items(&StatusInputs) -> Vec<HealthItem>` (every
  reason that holds, codes of §6.6), `status_message(&NeedsActionReason, holder) -> String` (the §6.6 details; from P48, `health_items` and the desktop's `CameraStatusResponse` use the
  entry's `message` when present and fall back to this — P48 projection r0 L9). `capture_ledger.rs` (names aligned to
  the P44 plan after projection r0 L5/L6/L14/L23) — `file_key(name) -> String` (strips only `__dup<n>` before the extension, upper-cases),
  `stem_key(file_key)`, `FifoCandidate { exposure_id, exposed_at, session_id: Option<i64>, expected_files: Option<u32> }`, `PriorDelivery { id, file_key,
  stem_key, sha256, exposure_id: Option<i64>, session_id: Option<i64>, outcome, completed_at, final_name }`, `associate(file_key, stem_key, sha256,
  completed_at, prior: &[PriorDelivery], candidates: &[FifoCandidate]) -> AssociationDecision { exposure_id: Option<i64>, association: Association,
  session_id: Option<i64>, ambiguous_with: Vec<Option<i64>> /* other sessions in the candidate set; `None` = Unassigned */ }` (applies every filter
  itself: rule (1) same key **and** sha, rule (2) same stem within `FIFO_WINDOW_MS`, rule (3) `exposed_at ≤ completed_at`, the window and "still
  expecting" = non-duplicate priors `< expected_files.unwrap_or(1)`; the service passes every `expecting` exposure and every `done` prior with the same
  key or stem), `is_complete(expected_files: Option<u32>, non_duplicate_count: u32) -> bool`, `duplicate_decision(file_key, sha256, prior:
  &[PriorDelivery]) -> DuplicateDecision { Photo, Duplicate { of: i64 }, RenamedPhoto { n: u32 } }`, `dup_name(camera_filename, n) -> String`
  (`DSC01234__dup1.ARW`), `target_folder(capture_root, folder_key: Option<&str>) -> PathBuf` (`_Unassigned` when `None`), `pending_count(exposures:
  &[(ExposureStatus, i64)], now_ms) -> usize` (both age bounds exclusive), `sequence_gaps(stems: &[&str]) -> Vec<Gap { prefix, after, before, missing }>`
  (the lab's `seq_of` rule: `_<1–2 digits>` stripped **only here**, for gap reporting).
- **Services** (`crates/core/src/services/camera_ledger.rs`): `LedgerShared { worker: Mutex<()>, raised: Mutex<bool>, ftp_note: Mutex<Option<String>> /* P46:
  the short form of the last `camera.ftp_state`, set through `LedgerService::set_ftp_note`, appended to the `camera_files_not_received` detail while the
  watchdog raises, cleared by `dismiss()` and by a non-raising `watchdog` */ }` — **one per app process, owned
  by the `CameraRuntime`** (R-I18; P44 projection r0 L3) and shared by every `LedgerService` the runtime builds; `LedgerService { store, clock, fs:
  Arc<dyn DeliveryFs>, events, health, capture_root: Arc<dyn Fn() -> Option<PathBuf> + Send + Sync>, shared: Arc<LedgerShared> }` implementing
  `CaptureSink` and `CompletedNames` — one decision at a time under `shared.worker`, against `done` rows only: decide → `insert_delivery(pending,
  written_path, final_path)` + the exposure's `received` transition in the same transaction (when `expected_files` is `None` or met) → `rename_exclusive`
  (on `Exists`: compare SHA-256 → `.duplicates/` or the next `__dupN`; any non-`EEXIST` error → `Err`, no fallback) → `set_delivery_done`; a delivery whose
  `written_path` matches a `pending` row **takes that row over** (no second row); `exposure_observed` reads `active_session_id` at that moment;
  `transfer_in_progress` is a no-op in P44 (P46 computes `transfer_active`); `recover_pending()` at every bind under `shared.worker` (pending + staging
  file present → rename again to the stored `final_path`; final present → done; neither → left pending, logged); `watchdog(now_ms, transfer_active: bool)
  -> WatchdogState { pending_count, gaps, raised, first_raise }` sets/clears `camera_files_not_received` (gaps over deliveries completed within
  `WATCHDOG_MAX_AGE_MS`; `gaps` = the number of `Gap` entries; the raised flag lives in `shared` so it survives a settings save); `dismiss()`;
  `complete_for(exposure_id) -> bool` (over `is_complete`); `completed_names_since(ms) -> Vec<(String, u64, i64)>` (`done` deliveries, deduplicated by
  name keeping the latest, for the receiver's listings). MC-2 rule 0 is served by `Store::delivery_for_path(canonical) -> Option<(Option<i64>, i64)>`
  (the delivery's session and `captured_at`; reads `pending` rows too), called by `ArrivalService::accept` — there is no `session_for_path` on the
  service. `Store`/`StoreTx` ledger methods (P44 Task 0): `insert_exposure`, `set_exposure_status`, `set_exposures_status_where(from, to)`,
  `exposures(Option<ExposureStatus>)`, `insert_delivery`, `update_pending_delivery(..)`, `set_delivery_done`, `deliveries_by_key`, `deliveries_by_stem`,
  `pending_deliveries`, `pending_delivery_for_written_path`, `delivery_for_path`, `completed_names_since`, `item_folder_key_frozen` (§6.4).
- **Runtime** (`crates/integrations/src/camera/runtime.rs`): `CameraRuntime` (one `Arc` per process, created in `App::open_with_adapter` and carried
  through `SettingsSetContext`, `App::build(…, camera: Arc<CameraRuntime>)` and `App::reopen`): fields `supervisor: Mutex<Option<SidecarClient>>`,
  `receiver: Mutex<Option<ReceiverHandle>>`, `binding: RwLock<Option<AppBinding>>`, `sink: RoutingSink`, `secrets: Arc<dyn SecretStore>`; **no provider
  slot** (P46 projection r0 L18): `provider() -> Arc<dyn CameraProvider>` returns the current `binding.provider`, and `direct_adapter() -> Arc<DirectCameraAdapter>`
  creates the direct adapter **once per runtime** (lazily) and returns the same `Arc` to every later `App::build`, so its `StatusInputs` survive a settings
  save (the sidecar does not re-send `connection.ready` for a session that stays open). `App::build` selects **up front** with `support::installed(&paths)`:
  `Some` → `camera = camera_runtime.direct_adapter()`, `None` → `CameraViaCaptureOne`; that `Arc` goes to `ArrivalService`, `AppBinding.provider` and
  `App.camera`/`AppHandles.camera`; `credentials: Mutex<Option<(String, String)>>` (read from the `SecretStore` once at
  the first spawn and cached; restarts reuse it; P50's `camera_secrets_set` refreshes it — a Keychain prompt never stalls the background loop), `clock`,
  `run_id: String /* uuid; the staging folder suffix */`; from P45 the sidecar client's seam: `send_command(Command) -> Result<CmdId>` (the returned id is
  the one on the wire), `command_outcome(&str) -> Option<CommandOutcome { Done, Rejected { reason }, Error { error } }>`, `sidecar_events() -> Vec<SidecarEvent>`
  (the last 1 000 delivered; `TestApp::camera_runtime()` reaches them; P45 projection r0 L12), and the staging folder `<binding document>/Capture/.incoming/
  sdk-<run_id>` computed at the first spawn and never moved (L14); `bind(AppBinding { store, events, health, previous_health, provider, paths,
  camera_settings: CameraSettings, capture_root })` (the fields at HEAD `e907dfb`; no `support`/`editing` field — the selection is `App::build`'s) is **the
  last statement of every `App::build`** (after every `?` and
  thread spawn, right before `Ok(Self {..})`, so a failed build never re-targets or spawns) and (a) re-targets events and health (P42–P45: copying the
  camera-family health items from `previous_health`; from P46: re-publishing the direct adapter's `StatusInputs` through `health_items` for the codes the
  adapter owns — never `camera_files_not_received`, §6.6), (b) rebuilds the
  `LedgerService` over the new store **sharing the runtime's one `ledger_shared: Arc<LedgerShared>`** (created with the runtime, so the worker lock and
  the watchdog's raised flag are process-wide and survive a settings save; P44 projection r0 L3) and runs `recover_pending` under that lock; the
  `DeliveryFs` the service uses comes from the runtime's `delivery_fs: RwLock<Arc<dyn DeliveryFs>>` slot (default `CameraFiles`; `set_delivery_fs(fs)` is
  the test seam, P44 L11), (c) starts the supervisor and the receiver on the **first** bind that finds support
  installed, (d) applies changed camera settings in place (receiver port/range → close + reopen; credentials → the cache, used at the next spawn, restart
  only while no USB session is open), never spawning a second sidecar or listener; `tick()` from the App's background thread (supervisor restarts,
  watchdog), **serialised under the supervisor `Mutex`** (two background loops run during a rebuild) — from P46 (projection r0 L10/L13/L19): **one**
  `client.poll` per tick feeding the ring buffer and the direct adapter's step (the adapter's effects — status event, owned health codes, deliveries, helper-log
  lines, FTP note — are performed against the current binding; a download's hash runs outside the supervisor lock), then the watchdog glue once per mono
  second (`transfer_active = uploads_in_progress() > 0 || expected_transfer.is_some()`; `ledger.watchdog`; `first_raise` → `provider().request_ftp_status()`)
  while the adapter's `FilesNotReceived` input mirrors `LedgerShared.raised` on every tick; `shutdown()` is public, idempotent and terminal
  (`quit`, `QUIT_GRACE_MS`, then kill + wait; the receiver stopped; no restart afterwards), called by `impl Drop` (tests) and by the desktop `run()`'s
  `RunEvent::Exit` hook (production: tao ends the process with `process::exit`, the managed state is never dropped). `App::shutdown` **does not** stop
  the runtime.
- **Sidecar** (`crates/integrations/src/camera/sidecar/`): `SidecarSpec { program: PathBuf, args: Vec<String> /* ["auto", "--save-dir", <staging>,
  "--usb-product-id", "0x0E0C"] */, env: Vec<(String, String)> /* CRLAB_FINGERPRINT_DIR only */, first_line: String /* `credentials user=<pct>
  password=<pct>` */ }`; `Supervisor { spec, child: Option<Child>, attempts, next_spawn_at, events_tx, stop }` with `spawn()`, `send_line(&str) ->
  io::Result<()>` (P45; the client writes `Command::encode()` through it), `tick(now) -> Vec<SupervisorEvent>` (`Started { pid }`, `Exited { code }`,
  `Line(String)`, `Stderr(String)`), `quit_and_wait()`; `client.rs` — `SidecarClient { supervisor, pending, events: Sender<SidecarEvent> }` with
  `send(Command) -> io::Result<CmdId>`, `poll() -> Vec<SidecarEvent>`, `outcome(&str) -> Option<CommandOutcome>` (a `CmdRejected`/`CmdError` attaches to
  the most recent `CmdReceived` whose id has no `CmdDone` yet; `CmdDone` yields `Done` only when nothing was attached; an absent or empty `cmd_id` is never
  tracked); `protocol.rs` — **field sets aligned to the lab's emit sites after P45 projection r0 (L8, L9); the P45 plan's `SidecarEvent` table is the
  authority, required = written unconditionally at the site, everything else `Option`:** `Transport { Usb, Wifi, None }`, `StoreDestination { Verified,
  Failed }`; `SidecarEvent::{TransportState {from, to, reason, transport}, ConnectionReady {transport, connect_ms: Option<i64>, model: Option<String>,
  store_destination: Option<StoreDestination>, store_value: Option<String>} /* no `camera_id` — the sidecar never sends one */, SessionClosed {transport,
  reason}, SessionLost {transport, reason, busy}, CmdReceived {cmd, cmd_id: Option<String>}, CmdDone {cmd, cmd_id: Option<String>}, CmdRejected {cmd,
  reason}, CmdError {cmd, error}, CaptureRequested {capture_id, transport, af}, CaptureAf {capture_id, af_result}, CaptureAcknowledged {capture_id,
  success, sdk_error}, CaptureExposed {capture_id: Option<String>, transport, association}, CaptureExposureMissing {capture_id}, CaptureUncertain
  {operation}, CameraBusy {reason, safe_point}, SwitchDeferred {to, busy}, UsbArrived {detail}, UsbRemoved {detail}, UsbInitial {present}, UsbClaimedBy
  {owners, hint: Option<String>}, UsbFallbackToWifi {usb_failures: i64, usb_retry}, UsbRetryRequested, SdkDownloadCompleted {filename, size_bytes: i64 /*
  −1 when the sidecar's file_size fails */, success, transport, r#type: Option<String>, provisional_capture_id: Option<String>, ms_since_capture_request:
  Option<i64>}, SshFingerprint {status, success: Option<bool>, fingerprint: Option<String>}, CameraFtpState {fields}, SdkUnavailable {error}, ConnectFailed
  {transport, sdk_error, error /* never `message` */, timeout: Option<bool>}, Unknown {event}}` with `timestamp_ms: i64`; `sdk.download.hashed` and
  `usb.detected_by_discovery` parse to `Unknown`; `parse_line(&str) -> Result<SidecarEvent, ProtocolError { Json, MissingTimestamp, BadTimestamp }>`
  (`iso_to_ms` accepts `YYYY-MM-DDTHH:MM:SS[.f{1,9}]Z`); `Command::{Credentials {user, password}, Capture {id, af: true, cmd_id}, Props {cmd_id}, FtpStatus
  {cmd_id}, Status {cmd_id}, UsbRetry {cmd_id}, Quit}::encode() -> String` (percent-encoding of `user`/`password` per RFC 3986 unreserved set; P46 adds
  `Reconnect {cmd_id}`).
- **Direct adapter** (`crates/integrations/src/camera/direct/`): `DirectCameraAdapter` (Sony SDK sidecar driver; implements `CameraProvider` and
  `ArrivalSource {via: Cable}`; created once per runtime by `CameraRuntime::direct_adapter()`; state `StatusInputs`, `expected_transfer: Option<Instant>`,
  `sdk_unavailable: bool`, `staging_dir` — no `usb_fallen_back`, no `pending_captures`; the P46 plan's §The adapter contract is the authority for its event
  table, the owned health codes, the refusal clear rules and the `Exited` reset), `needs_action_from_connect_failed(error) -> Option<NeedsActionReason>`,
  `holder_from_owners(owners) -> String` (first owner before the first `"; "`, then the text after its last comma, trimmed),
  `normalize_camera_filename(name) -> String` (Sony: `_<1–2 digits>` after the stem's trailing digits removed, e.g.
  `DSC01234_1.ARW → DSC01234.ARW`; `IMG_1234.CR3` would be untouched here too because the rule needs trailing digits before the suffix), `HolderReport
  { holder: Option<String>, programs: Vec<String>, sentence: Option<String> }` and `name_programs(owner_process: &str, running: &[String]) -> HolderReport`
  (`direct/ownership.rs`; R-I30 list `KNOWN_CAMERA_PROGRAMS`, Capture One first; `owner_process` = `holder_from_owners(owners)`), `TakeBackService`
  (`direct/take_back.rs`; R-I32 steps with the bounds above; `run() -> Result<TakeBackReport { completed_steps: u8, camera_held: bool }>`,
  `holder_report()` for `camera_test_connection` — `programs` = the known programs running in every case, `holder`/`sentence` only while the status is
  `HeldByOtherProgram`), `StartOrder` (`direct/start_order.rs`; R-I31 conditions plus `Satisfied(AlreadyRunning)` set by the first ensure whose
  `editor_info` reports running; the 15 s deadline counts from the first `wait()`) and `LaunchGate` (`editing/capture_one/launch_gate.rs`: `hold() ->
  GateGuard`, `GateGuard::permit(f)`, `is_held()`, `permitted_here()`). **Both are process-wide** (P48 projection r0 L14): created in
  `open_with_adapter_bind` beside the lanes, passed to `GatedScripting::pair`, carried through `SettingsSetContext`/`reopen`, never rebuilt by
  `App::build`. `GatedScripting` (both lanes) refuses exactly the **six** scripts that do not check `running()` first — `launch`, `front_document`,
  `make_document`, `open_document`, `ensure_recipe`, `ensure_capture_style` — with `Error::EditorLaunchDeferred` while held and not permitted on this
  thread; `quit` is **not** gated (`quit.js` guards itself with `running()`, L13) and the take-back sends `save`/`quit` through the ungated `App.scripting`.
  `GatedScripting::launch` waits on `StartOrder` only on a non-permitted thread, outside the lane mutex (L4/L5). `ensure_provision` turns a deferral into
  `Ok(ProvisionReport::deferred())` before the deferring step's health handler runs (L2); `EditingStartup::run` stops after a deferred ensure and
  `SessionsService::with_camera(..)` branches R-I33 on the provider's capability (L1/L10). `AppBinding` gains the take-back's inputs (editing provider,
  ungated scripting, provision inputs, document id, gate; names the P48 implementer's) and the runtime the seams `set_process_list`/`set_sleep_inhibitor`.
- **Receiver** (`crates/infra/src/ftp/`): `ReceiverConfig { bind: IpAddr /* 0.0.0.0 in production, 127.0.0.1 in tests */, port: u16 /* 0 = ephemeral
  */, passive: RangeInclusive<u16> /* 0..=0 = ephemeral */, allowed_peers: Vec<IpNet>, username, password: String }`, `FtpReceiver::start(config,
  sink: Arc<dyn CaptureSink>, names: Arc<dyn CompletedNames>, capture_root: Arc<dyn Fn() -> Option<PathBuf> + Send + Sync>) -> Result<ReceiverHandle
  { port(), passive(), stop(), uploads_in_progress() }>`; `CompletedNames { fn completed_names_since(&self, since_ms) -> Vec<(String, u64, i64)> }`
  (the ledger implements it; defined in `psm_core::ports::camera` from P44 and re-exported by `psm_infra::ftp`, §6.4). Reply codes fixed by MC-32 (R-I24): 421 peer refused, 530 bad login, 553 hidden/staging/`..` names and every `RNTO`,
  550 `DELE`/`RMD`/`RNFR`/unknown name in `SIZE`/`MDTM`, 450 a second `STOR` of a name in flight, 554 `REST` past the `.part` length, 451 capture root
  missing, 426 data connection failed, 226 completed, 502 `PORT`/`EPRT` and unknown commands.
- **Support packages** (`crates/integrations/src/camera/support/`): `InstalledSupport { make: &'static str /* "sony" */, version: String, sidecar_path:
  PathBuf, folder: PathBuf }`; `installed(paths) -> Option<InstalledSupport>` (a free function in `support/mod.rs`, P46: the newest `camera/sony/<version>/`
  folder holding `camera-sidecar`; P50 adds "every manifest file" — the P50 plan names it `installed(paths)` too), `install(paths, archive, bundled_sidecar) -> Result<InstalledSupport>` (extract with `zip`, verify
  against `manifest::SONY` (`native/camera-sidecar/manifest/sony-<version>.json`: `{version, files: [{path, sha256}]}` — hashes only, generated
  by `tools/sidecar/manifest.sh` on the build machine), copy the runtime + the sidecar, `codesign -s - --force` each Mach-O, run `camera-sidecar
  version` and require `sdk_version` on the supported list), `refresh_sidecar_copy(paths, bundled)` at each start (SHA-256 mismatch → copy and sign
  again). The Capture One camera adapter is used whenever `installed()` is `None`.
- **Frontend (P51):** `services/camera.ts` (`cameraStatus`, `cameraRetry`, `cameraCapture`, `cameraTakeBack`, `cameraTestConnection`, `cameraSupportGet`,
  `cameraSupportInstall`, `cameraSecretsSet`, `cameraFtpPasswordGenerate`, `cameraFingerprintConfirm/Reject`, `onCameraStatus`, `onFingerprintPending`),
  `features/shell/CameraDot.tsx` (five states by `data-state`: `connected` (green; text `<name> · Cable|Wi-Fi`), `connecting` (amber pulse), `busy`
  (amber), `needs_action` (red; the sentence as the tooltip and the `Take the camera back` button when `reason === 'held_by_other_program'` and the
  holder is Capture One), `disconnected` (red)), `features/settings/CameraSection.tsx` (Add camera support → Sony, installed version, FTP password
  (generate, shown once, copy), receiver port and passive range, Access Authentication user/password, fingerprint card, Test camera connection),
  Health names for the §6.6 codes.

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
| CMake / C++ (planner, 2026-11-06) | `cmake --version`, `c++ --version` | cmake 4.4.5; Apple clang 17.0.0 (Command Line Tools). Enough for the stub sidecar build (P45) and the C++ mock-camera tests. |
| Sony Camera Remote SDK (planner, 2026-11-06) | `ls ~/Library/Developer/SonyCrSDK` | **absent** there; the SDK the lab built against lives at `/Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk` (gitignored in the lab). `tools/sidecar/build.sh --sdk <that path>` is the only SDK build; nothing in this repo ever reads a Sony file at test time. |
| Camera | owner | Sony A7R V, configured once by the owner for Auto FTP to this Mac (lab `camera_steps.py`: `Directory Hierarchy: Standard`, `Same File Name: Does Not Overwrite`, port 2121, user `sony` in the lab — **P50's setup names the user `sessionmanager` and a generated password; the owner re-enters both on the camera once**) and Access Authentication over Wi-Fi. Used daily: only P47 and P52 may ask for it. |
| macOS firewall / Local Network | — | Unknown for a packaged app (spike S4). Tests bind the receiver to `127.0.0.1` to avoid the "accept incoming connections" prompt. |

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
| Camera sidecar, stub (P45+) | `tools/sidecar/build.sh --stub` → `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` (also run by `apps/desktop/src-tauri/build.rs` when the file is absent) · C++ tests: `tools/sidecar/build.sh --stub --test` (`ctest` in `target/camera-sidecar/`) |
| Camera sidecar, with the SDK (build machine only) | `tools/sidecar/build.sh --sdk /Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk` (never commits anything; the Sony runtime is **not** bundled — the user installs it as a support package, P50) · manifest: `tools/sidecar/manifest.sh <sdk root> <version>` → `native/camera-sidecar/manifest/sony-<version>.json` |
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
- **10E (P42–P50):** `TestApp::with_camera()` = `with_camera_behaviour(json!({}))` (P42; `with_fake_document` plus a "support folder" under the temp
  data dir holding the fake sidecar binary copied (`fs::copy`, 0o755) as `camera/sony/test/camera-sidecar`, the fake's recording dir `camera/sony/test/
  rec/` (derived by the fake from `current_exe()`; **no test-only environment variable**) with `behaviour.json` written **before** the first spawn,
  `FakeSecrets` pre-filled with `access-auth-user = "u name"`, `access-auth-password = "p/w"`, `ftp-password = "ftp-pw-unique-1"` (distinctive values that
  appear in no expected text), the receiver bound to `127.0.0.1:0` with loopback allowed (P43), `ManualClock` driving `CameraRuntime::tick`);
  `set_fake_sidecar_behaviour(json)` (read by the next spawn); `with_camera_script(script)` (P45: the fake replays a script; the script and `behaviour.json`
  are written before the spawn); `settle(a, pred)` (loop `a.tick(0)` / 10 ms sleep, 3 s bound — every positive read of the fake's recording goes through
  it, negative reads after a 500 ms settle); `TestApp::camera_runtime()`, `camera_status()`, `ledger()` (P44), `receiver_port()` (P43), `ftp()` →
  `FtpClient` connected and logged in (P43), `fake_sidecar_recording() -> Recording { pids: Vec<u32>, argv: Vec<Vec<String>> (from `argv.<pid>.json`),
  env: Vec<Map> (`env.<pid>.json`), stdin: Vec<(u32, String)>, exits: Vec<(u32, i32)> }`, `fake_processes` (`FakeProcessList`, P48), `fake_sleep`
  (`FakeSleepInhibitor`: `held() -> Option<String>`, `holds() -> usize`, P48), `take_back()`, `launch_gate()`, `user_lane()` (the user-lane
  `GatedScripting`, P48), `FakeDirectProvider` (`fake_direct_camera.rs`, P42). The fake Capture One gains
  `quit()` (`running` false after `quit_after_polls`, default 1; op `quit`), `restart_after_quit: bool` (the `editor_info` after the first one that reports
  not-running reports `running = true` with no `launch` op, P48 C5(e)), `fail_save: bool` (P48; every `running = false` in a test is paired with
  `running_after_polls = Some(1)`, or the ensure's 45 s real-time launch wait runs — P48 projection r0 L6), `set_current_collection` (op `set_current_collection:<folder>`, state
  `current_collection`, fault `fail_set_current_collection`; `find_image` answers only for images in the current collection, MC-19), `apply_style`
  (op `apply_style:<variant>:<name>`, per-variant `styles` list) and loses `import` (P49).

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
- 2026-11-05 editor chrome — planner (Fable) added P37_editor_chrome from the owner's corrections after the second editor try-out (book 10D
  amended `3c7623e`, R39, changelog 29; MC-27…29 unchanged). §4 one NOT_STARTED row, §5 count 37 / graph tail / `P37[P36]` / the editor-chrome
  note (no Fable), §6.10 the P37 block (`HintCard`, the card text rule, the rail, the drawers, retired toolbar buttons and props). Tests of P36
  (C5(a), C5(b) text, C6(f), C6(g), C6(h)) and P32 (C3(c)) change their expectation as listed in the plan's Read first. No intention-book change;
  no owner card.
- 2026-11-05 editor polish and undo history — planner (Fable) added P38_editor_polish (frontend; R40, changelog 30), P39_undo_history_core
  (Rust; 05A MC-30, M36, R41; projection on), P40_undo_history_compositions_and_attributes (Rust; projection on) and P41_undo_history_ui
  (frontend + the Tauri Edit menu). §4 four NOT_STARTED rows, §5 count 41 / graph tail / `P38[P37] · P39[P37] · P40[P39] · P41[P40,P38]` / the
  phasing note (no Fable), §6.4 (`delete_photo_row`, tx `composition`/`save_composition`, `delete_attribute_value`; `set_undo_slot` removed),
  §6.5 (`undo_slot` unused from P39), §6.6 (`nothing_to_redo`, `step_cannot_apply{label, reason}` and its reasons), §6.7 (`history_get/undo/redo`,
  `photos_undo` compatibility until P41, `edit_menu_set`, events `history_changed`, `edit_menu`), §6.10 the P38 block and the P39–P41 block (the
  step model, labels, App wiring, frontend and menu names). Tests of P37 (C2(a) table, C5(a), C7(a)/(b), C8(a)), P32 (C3(c)), P36 (C6(h)), P02
  (c3b–c3f, the MC-6 row), P10 (`attrs()`), P08 (C2(b)/(c)) change their expectation as listed in the plans' Read first. Mechanism check of MC-30
  against the code recorded in `handoffs/reviewer/handoff_polish_and_undo_plan_r1.md`. No intention-book change; no owner card.
- 2026-11-05 P39 projection r0 folded — planner (Fable) folded `handoff_P39_undo_history_core_projection_r0.md` (D1–D18, R1–R5) into
  `plans/P39_undo_history_core.md` (rows 22 → 27; `projection: done`), carried D7/D10–D12 into `plans/P40_…` tasks 2 and 5 (its projection stays
  on), and amended §6.4 (`StoreTx::publish_confirmed_since`), §6.6 (`that session already has this photo`), §6.10 (the lock, drop, no-op and
  ascending-order rules; the copy-undo refusal; `Unreview` labels). Nothing declined.
- 2026-11-05 P40 projection r0 folded — planner (Fable) folded `handoff_P40_undo_history_compositions_and_attributes_projection_r0.md` (D1–D16,
  reality checks) into `plans/P40_…` (criteria 7 → 8, rows 19 → 20; `projection: done`): fixtures classify through `ReviewService` (a step),
  `ai_suggest` never records, only `set_value` saves coalesce, `AttributesService::with_history(&History)` (`new` unchanged), the command rows in
  `lib.rs mod p40_tests`, `product.description` for the command row, the copy-redo `that photo is gone` fix folded from the backlog (task 8, C8(a)),
  D10–D16 delegations written down. §6.10 amended (`with_history`). P41 needs no change. Nothing declined.
- 2026-11-06 direct camera adapter — planner (Fable) added P42_camera_runtime, P43_ftp_receiver, P44_capture_ledger, P45_sidecar_and_protocol,
  P46_direct_camera_adapter, P47_support_package_spike (owner-attended), P48_camera_ownership, P49_capture_one_alongside,
  P50_camera_settings_and_support, P51_camera_setup_ui and P52_hardware_acceptance (owner-attended) from book 10E (MC-31…MC-34, M37–M40, R42–R45
  at `53d6ea4`; inventory R-I18…R-I38 at `41bef14`). §2 (10E, M1–M40, the Sony evidence and the read-only lab repo), §3 (trace ranges, `[manual]`
  rows, the camera hard rules), §4 eleven NOT_STARTED rows, §5 count 52 / graph / `depends_on` / the direct-camera phasing note / P45, P47, P52
  preconditions, §6.1 new modules (core camera domain and mechanisms, infra ftp/keychain/processes/sleep, integrations camera runtime/sidecar/direct/
  support, `native/camera-sidecar/`, testkit fakes), §6.2 camera crates (libc, security-framework, ipnet, zip, getrandom; no FTP crate), §6.3
  support folder, fingerprints, `.incoming`/`.duplicates`, §6.4 port changes, §6.5 migrations 006/007, §6.6 camera enums, error and health codes,
  §6.7 camera commands and events, §6.8 the `camera` settings block, §6.9 fake sidecar / FTP client / C++ tests, **new §6.11** (the camera
  skeleton), §7.1 cmake/clang/SDK/camera facts, §7.2 sidecar build commands, §7.5 harness. Owner card CI1 was answered "always jump" during
  planning (R46 at `e0ce354`, relayed by the orchestrator): P49 plans the fixed rule, no policy constant. No intention-book change by the planner;
  one planner reading on the Import feature recorded in P49 Notes.
- 2026-11-06 P42 projection r0 folded — planner (Fable) folded `handoff_P42_camera_runtime_projection_r0.md` (L1–L12, L15–L23 and the reality checks;
  L13/L14 folded upstream by the orchestrator at `789f035`) into `plans/P42_camera_runtime.md` (`projection: done`; rows 26 → 26, 14 reworded; L6 taken
  as the `RunEvent::Exit` hook) and amended §6.7 (`camera_status_changed`: only `camera_retry` emits in P42, through the hub), §6.11 Runtime (`bind` last,
  the health copy until P46, credentials cache, serialised `tick`, terminal `shutdown` + the exit hook), §7.5 (`with_camera_behaviour`, the fake's recording
  dir and files, distinctive secret fixtures, `settle`). Carried into P43, P44, P45, P46 and P50 Review logs. Nothing declined.
- 2026-11-07 P48 projection r0 folded — planner (Fable) folded `handoffs/reviewer/handoff_P48_camera_ownership_projection_r0.md` (L1–L25, 0 owner cards)
  into `plans/P48_camera_ownership.md` (`projection: done`; rows 38 → 38, paths 33 (typed) → 43 (derived)) under orchestrator rulings (L4 a/b/c, L7 knob,
  L18 wire shapes, L19 programs in every case, L20 deferred scan, L24 stands, L25 `running` only). Amended §6.6 (`NeedsAction.message`, `take_back_blocked`
  + `already running`, `take_back_failed{step, cause}` as strings, R-I30-consistent `camera_held` examples), §6.7 (`camera_take_back -> { completed_steps,
  camera_held }`, string `data`), §6.11 Mechanisms (`StatusInputs` message, `status_message` fallback) and Direct adapter (`name_programs(owner_process, …)`,
  the six-script gate list without `quit`, process-wide `LaunchGate`/`StartOrder`, `AlreadyRunning`, `holder_report`, `AppBinding` additions, runtime seams),
  §7.5 (`restart_after_quit`, `fail_save`, `take_back()`, `launch_gate()`, `user_lane()`, `FakeSleepInhibitor::holds`), §4 P48 row → PROJECTED. P51 C2(c)'s
  mock `step` corrected to a string (P51 Review log). Backlog: Imaging Edge Desktop executable name / `PhotosReliveWidget` (P52, spike S5). Nothing declined.
