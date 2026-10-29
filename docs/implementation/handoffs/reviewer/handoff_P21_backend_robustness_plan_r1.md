---
plan: P21_backend_robustness (and P22_frontend_tidy_up, planned in the same session)
role: plan
round: 1
state: DONE
date: 2026-10-29
actor: Claude Fable 5.1 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# P21_backend_robustness + P22_frontend_tidy_up — planning handoff, round 1

**Summary.** Gate passed: `docs/intention/intention.md:3` reads **RATIFIED** (owner, 2026-10-24) and the book carries the
inventory's contract chapters (05A/07A/09A/10A; P00–P20 APPROVED on them). I read flow.md, the master plan, STATUS.md,
backlog.md (all 150 rows), the P17/P18/P19/P20 plans as format and precedent, book 12/14/07 and 09A MC-24, and the code
behind every backlog row in scope (settings file, job runner, backup, `App::build`/`prepare`/`AppHandles`, the
settings/health/items/publish services, `AppState`, the Tauri commands, and the frontend drawer, settings page,
keyboard hook, workspace state/page, form dialog/state/inputs, identity editor, page header, routes). Every row in
scope was verified against the current tree before it was planned; four were found already fixed and dropped. Two
plan files written (P21 backend: 7 criteria / 14 rows; P22 frontend: 5 criteria / 10 rows), `phases.txt` appended,
master plan amended (§4, §5, §6.4, §6.6, §6.7, §6.8, §9), backlog rows marked without deleting history. One commit,
not pushed. Nothing needs the owner.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. Two readings are recorded in the plans' Notes rather than carded, because the book decides them: (1) ch07 step 5
"notification row 'Published · time'" is read as what the photographer sees — P21 stores `Published` and the time as
`created_at`, P22 renders `Published · <local date, time>` (no epoch anywhere); (2) a damaged `settings.json` is kept
untouched on disk and the defaults are used in memory until the next save (ch12 §Settings, MC-17's fallback spirit) —
the owner's hand edits are never overwritten by the app on its own.

## Write perimeter
- `docs/implementation/plans/P21_backend_robustness.md` (new) — committed.
- `docs/implementation/plans/P22_frontend_tidy_up.md` (new) — committed.
- `docs/implementation/phases.txt` (two lines appended: `P21_backend_robustness`, `P22_frontend_tidy_up`) — committed.
- `docs/implementation/master_plan.md` (§4 two NOT_STARTED rows; §5 phase count 22, graph tail `→ P20 → P21 → P22`,
  `depends_on` P20[P19] · P21[P20] · P22[P21]; §6.4 `CommerceAdapter::render_description`; §6.6 health codes
  `settings_invalid`, `backup_failed`, third `condition_field_missing` detail; §6.7 `item_update_identity` barcode `""`
  and `items_search_shop` `provider` rejection; §6.8 reading rule; §9 log entry) — committed.
- `docs/implementation/backlog.md` (first-cell prefixes only: `→ P21 ·`, `→ P22 ·`, `→ P21 + P22 ·`, `→ P22 (task only) ·`
  or `✓ resolved by P17/P18 (verified 2026-10-29 …) ·` on rows 5, 7, 82, 84, 86, 97, 101, 103, 109, 118, 124, 125,
  126, 127, 129, 130, 132, 134, 138, 139, 140, 141, 145, 146, 147, 148, 150; no row deleted or reworded) — committed.
- `docs/implementation/handoffs/reviewer/handoff_P21_backend_robustness_plan_r1.md` (this file) — committed.
- No code, no tool-recorded state, no intention-book change. Nothing pushed; the pipeline was not run; STATUS.md
  untouched (orchestrator's file).

## Phases
| Phase | Scope (backlog rows) | Rows | depends_on |
|---|---|---|---|
| `P21_backend_robustness` (difficulty high) | 7/97/130/134 start fallbacks (damaged settings, incomplete integration → Export mode, failing backup) with health `settings_invalid`/`backup_failed`; 138 one save at a time (`AppState`); 132 exact-mask secret; 126 integration defaults on read; 139 unknown job kinds never left `running`; 140 `search_shop` propagates the provider error; 127 health refreshes the schema before judging the condition field, "not read yet" detail; 101/118 text `Published` (+ condition note), 109 description compared as rendered (`render_description` port method); 84 barcode `""` clears | 7 criteria / 14 rows (7 `[rule]`) | [P20_drop_continuity] |
| `P22_frontend_tidy_up` (difficulty high) | 101/118 drawer shows local date/time (legacy suffix stripped); 132 secret field clears on focus, restores the mask on blur; 148 ⌘[ ignored while typing; 145 unavailable heading keyed to its query (task only); 147 event reloads deferred while a drag command is in flight; 150 overlay shows the projected slot's number/Cover, inert `flushSync` removed; 141 AI actions queued, failures reload; 82 invalid value seeded into text-box inputs; 86 one close sequence (Escape/X/outside/Save & close) that flushes and refreshes the card; 84 barcode cleared from the header; 146 `PageHeader` derives the parent from `parentRoute`, dead `goHome`/`.health-subtitle` removed | 5 criteria / 10 rows (8 `[rule]`) | [P21_backend_robustness] |

Order: backend first so the frontend builds on the final command semantics (`Published` text + `created_at`,
`barcode: ""`, exact mask). Between P21 and P22 the drawer shows the bare word `Published` (P21 Notes).

## Dropped from scope (verified against the tree, 2026-10-29)
- **124** (batch review Cancel enabled while in flight): `apps/desktop/src/features/publish/PublishPopover.tsx:34` already
  renders Cancel with `disabled={confirm.busy}` (P18 task 3). Marked `✓ resolved by P18`.
- **125 / P00 note 1 (row 5) / 103** (health_get and shop calls under the app lock / on the IPC thread): since P17 every
  command is `pub async fn`, takes `AppState::handles()` (read lock held only to clone) and works inside
  `spawn_blocking`; `health_check` lives on `AppHandles` (`crates/integrations/src/lib.rs`, after `:157`); P17 C4(a)/(b)
  test it (`apps/desktop/src-tauri/src/lib.rs:222–300`). Marked `✓ resolved by P17`.
- **129** (a failed settings rebuild leaves workers stopped): `SettingsSetContext::prepare` (`lib.rs:120–157`) builds the
  new runtime before the old one is replaced and restores the previous file on failure; the old workers keep running
  (P17 task 3, handoff assumption). Marked `✓ resolved by P17`.
- **145** is fixed inside P22 task 2 but carries no row: the one-frame heading is not observable in jsdom after React's
  effects flush, so a row could not fail. Recorded in the plan's Notes and in the backlog mark `(task only)`.

## Grounding worth knowing (for the coordinator and the reviewer)
- **Description re-sent (109):** `publish.rs:865–873` compares the shop's `descriptionHtml` with the plain local text;
  the renderer is `commerce/shopify/adapter.rs:125 description_html`, which core cannot import (layer rule). P21 adds
  `render_description` to the `CommerceAdapter` port; `FakeCommerce::update_description` (`fake_commerce.rs:485`)
  already stores the rendered HTML, so the fake's impl is the same call.
- **False red condition field (127):** `health.rs check_integration` evaluates stored definitions only; the startup
  refresh runs on its own thread (`lib.rs:444–458`) and is absent when it failed. P21 reads the schema inside the check
  (`AttributesService::refresh_schema`, needs a `Clock` handed to the health service) — one GraphQL call on an on-demand
  page that already fetches a token. `p16 c3a` keeps its "not found" because the fake's schema is empty there.
- **Stranded publish job (139):** `jobs.rs:43` returns `Err` after `job_due` has set the row `running`
  (`sqlite/store.rs:904`). P21 returns such a job to `queued` through `job_retry` with `unknown_job_kind` and a 60 s
  retry, and a handler-less runner claims nothing. No kind without a handler is enqueued today (`verify_hash` appears
  nowhere), so C3(b) inserts its unknown kind through `rusqlite`.
- **Lost save (138):** `state.rs:7–18` prepares under the read lock and applies under the write lock, so two saves
  prepare from the same snapshot. C2(a) is a race row: deterministic with the lock, almost always red without it; the
  reviewer judges by reading.
- **Overlay flip (150):** confirmed by the P20 reviewer's probe; `draggedNumber` in `WorkspacePage.tsx` reads the photo's
  current position. P22 tracks `over` through `onDragOver` and projects the slot with the same rule `useDrop` applies
  at release; jsdom renders the overlay (P20 C1(a)), so C3(b) reads its badges mid-drag.
- **Form second click dropped (141):** one shared `useAction` in `FormDialog.tsx` (`run` ignores calls while locked)
  and `AiPopover` closes on click. P22 queues; per-field locks are allowed as long as C4(a) holds (two calls, in order).
- **Escape leaves the card stale (86):** Radix `Dialog` calls `onOpenChange(false)`, which unmounts `FormBody`; its
  unmount effect writes pending edits but cannot call `onItemChanged`. P22 routes every close through one sequence.
- Line numbers in both plans are from the tree at `55616ec` (HEAD at planning time).

## Lessons for the planner (none new, two re-confirmed)
- A plan row that names a time or a text should name the *unit* (ch07 says "time", P13's implementer chose epoch ms
  because the delegated choice left the format open). P21 C5(a) now pins the text and the time's home (`created_at`).
- A frontend row whose pre- and post-fix states coincide after an `act` flush (backlog 145) is not a row; say so in the
  plan rather than ship a green decoration.
