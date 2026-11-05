---
plan: polish_and_undo
role: plan
round: 1
state: DONE
date: 2026-11-05
actor: Claude Fable 5.1 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# Editor polish (R40) and undo/redo history (R41, MC-30, M36) — planning handoff, round 1

**Summary.** Gate passed: `docs/intention/intention.md` is **RATIFIED** (owner 2026-10-24; changelog 30 keeps the header RATIFIED and
`MECHANISMS_VERSION` unchanged). MC-30 is a new load-bearing mechanism written by the orchestrator, not by a mechanism-inventory round, so I ran
the short mechanism check the task asked for against the code at `3e4b760` (§Mechanism check below) before planning; its findings are folded
into the plans as tasks, rows and hazards, none needs the owner. I read `git show 3e4b760`, 10D and 05A in full (MC-5, MC-6, MC-30), chapters
04/06/08 where cited, R38–R41, changelog 28–30, flow.md, the master plan (§3–§7, §9), the P36/P37 plans and narratives, the two previous
planner handoffs, and the code: `crates/core` services (`review`, `composition`, `attributes`, `photos`), ports, `mechanisms/position`,
`crates/infra/src/sqlite/store.rs`, `crates/integrations/src/lib.rs` (`App`, `apply_settings`, `EventHub`), the Tauri commands and `lib.rs`,
`apps/desktop/src/services/*`, `features/workspace/{useWorkspaceState, useKeyboard, WorkspacePage}`, `features/compose/{ComposeMode,
useCompositionSaves, useComposeKeys, useComposeGesture, machine, ComposeFrame, ComposedPicture, EditorToolbar, CompositionSection,
StraightenControl, compose.css}`, `features/form/useFormState`, `features/shell/Header`, `App.tsx`, the testkit (`TestApp`, p02/p10/p34 fixtures),
the test mocks of every `@/services/*` module, and the installed Tauri 2.12.5 menu API. **Four plan files written** (P38, P39, P40, P41; 27
criteria / 69 rows / 24 `[rule]` rows — counts derived from the files), `phases.txt` appended (four ids), the master plan amended. Not
committed (as instructed). **No owner card.** No `implementer: fable` anywhere (owner 2026-11-05).

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. Every point MC-30 leaves open is answered by the book or recorded as a reversible reading in the plans' Notes
(surfaced through the narratives' "Choices made for you"); the readings that change what the photographer sees are listed under §Choices.

## Write perimeter
- `docs/implementation/plans/P38_editor_polish.md`, `P39_undo_history_core.md`, `P40_undo_history_compositions_and_attributes.md`,
  `P41_undo_history_ui.md` (new).
- `docs/implementation/phases.txt` (four lines appended with `printf >>`).
- `docs/implementation/master_plan.md` (§2 ledger range M1–M36; §3 trace ranges M36 / MC-30; §4 four NOT_STARTED rows; §5 count 41, graph tail,
  `depends_on`, the phasing note with the no-Fable rule; §6.4 store additions/removal; §6.5 `undo_slot` unused; §6.6 the two codes and the
  reasons; §6.7 commands and events; §6.10 the P38 block and the P39–P41 block; §9 log entry).
- `docs/implementation/handoffs/reviewer/handoff_polish_and_undo_plan_r1.md` (this file).
- **Not touched:** `docs/intention/**`, `STATUS.md`, `backlog.md`, narratives, other plans, prompts, any code, any tool-recorded state. No commit,
  nothing pushed, no pipeline run, no live Capture One or Shopify call, no secret read.

## The phase map
| Phase | Kind | Difficulty / projection | Criteria / rows / `[rule]` | What |
|---|---|---|---|---|
| **P38_editor_polish** `[P37]` | frontend | normal / no | 6 / 12 / 11 | Split line (editor: two SVG lines 4 dark + 2 light; stage/tiles: 2 px white + dark shadow); on/off icons `fill` + `data-on` (Loupe, Snapping, grid); Rotate and flip = Radix `DropdownMenu` (menu items `Rotate 90° left/right`, `Flip horizontal`, closes after a pick); Nudge = Radix `Popover` whose content is `#compose-nudge-pad[role="group"]` (stays open on inside clicks, closes on outside pointerdown and Esc); the three icons in `.compose-degrees`; hover table T3 (29). |
| **P39_undo_history_core** `[P37]` | Rust | high / **yes** | 7 / 22 / 1 | `services/history.rs` (`History`, `HistoryStore`, `Step`/`StepKind`, `HistoryService::{undo, redo}`, labels); `ReviewService` records classify/delete/reorder/move/copy; reversal transactions with MC-5 maths, `touch_session`, `enqueue_mirror_tag`; `StepCannotApply`, `NothingToRedo`; `HistoryChanged`; `history_get/undo/redo`; `photos_undo` kept as a compatibility wrapper; `UndoSlot`/`set_undo_slot`/`ReviewService::undo` retired; `App.history` carried across a settings rebuild. |
| **P40_undo_history_compositions_and_attributes** `[P39]` | Rust | high / **yes** | 7 / 19 / 0 | `CompositionService` records `set`/`reset` and `apply` (one step); reversal = `StoreTx::save_composition` (new revision) for every photo in one transaction; `needs_review`/frame/validate guards; `AttributesService` records `set_value`/`accept_all`/`clear`/title with the prior `Value` row; byte-identical restore, `delete_attribute_value` for "no row before"; coalescing of consecutive same-field saves; `composition_apply` calls `apply` once. |
| **P41_undo_history_ui** `[P40, P38]` | frontend + Tauri menu | high / no | 7 / 16 / 12 | `services/history.ts`, `features/history/*`; ⌘Z/⌘⇧Z at window level (never in a text field; workspace, editor, dialogs); notices; header `Undo`/`Redo` icons with cards; delete toast → history; editor `Saved` sync on an external revision change; grid Split on ≥ 2 photos through `composition_apply`; the Edit menu (`edit.undo`/`edit.redo`, `edit_menu_set`, `edit_menu` event); `photos_undo`/`undoDelete` retired. |

**Why four, not three.** The task suggested P38 then one or two undo phases. Splitting the backend at the step kind (P39 review steps + the
whole history/command/App skeleton; P40 composition and attribute steps) keeps each Rust phase at 7 criteria with one mechanism each, and the
app stays green between them because `photos_undo` keeps answering through the history until P41 removes it (otherwise `npm run typecheck`
would go red the moment `photos_undo` disappears). P38 runs first: it is cheap and the owner sees it at once; P41 depends on it because it
rewrites the same `CompositionSection` row.

## Mechanism check — MC-30 against the code (what the task asked; recorded here, acted on in the plans)
1. **Is the step list complete and implementable as written?** Yes, with four additions the plans make explicit. (a) *Where recording can
   live:* every write to `photos.classification`/`position` goes through `ReviewService` (grep `set_photo_review|set_positions` → `review.rs`
   only), every composition save through `CompositionService::save` → `Store::save_composition`, every value write through `AttributesService::
   write`/`write_title`/`accept_all`/`clear`; Assign product is `PhotosService::assign_unassigned` → `ReviewService::move_photos`. Recording
   inside those three services therefore covers keys, buttons, menus, drag, pickers and the phone remote later — **no action reaches the store by
   a path that bypasses the services.** (b) *The constructors:* `ReviewService::new(store, clock)` is built per call in `commands/photos.rs:58`,
   `photos.rs:42` and the testkit — the plan makes the `&History` a required third argument so no route can forget it; `AttributesService` has two
   schema-only callers (`health.rs:36`, `lib.rs:572`) that pass `None`. (c) *"A field committed":* the form writes every field **300 ms after the
   last keystroke** (`useFormState.ts:7, 83–96`; ch08 "Values save locally as typed"), so a typed word produces several `set_value` calls; without
   a rule each pause would be a step. P40 coalesces consecutive single-field steps of the same `(item, key)` while the redo list is empty (Cocoa's
   typing coalescing), recorded as a reading. (d) *Steps that cannot apply:* the contract's two examples (photo removed by a non-step, target
   published) are made precise: preconditions checked **inside** the reversal transaction (photo exists in the expected session with the
   expected classification; the session a transfer writes into is not published-and-clean, exactly `transfer`'s rule at `review.rs:161–165`; a
   copy is not removed once a publish completed after it, `published_seq >= seq_after`, because its rows may carry `photo_links`; a composition
   whose `state` is `needs_review` or whose frame is unknown or no longer validates is not written — M33's "nothing is cropped silently"). Each is
   a row.
2. **Which existing operations are not atomic today?** `CompositionService::apply` (`composition.rs:19–33`) is a loop of independent `set`
   calls, and `composition_apply` (`commands/compositions.rs:189–216`) even calls `apply` once **per item** — so Apply is N transactions today.
   MC-30 wants its *reversal* as one step/transaction (not Apply itself); `Store::save_composition` is its own method with its own transaction
   (`store.rs:975–1000`), **outside `StoreTx`**, so a multi-photo reversal cannot use it. P40 adds `StoreTx::save_composition` sharing the body
   (`save_composition_in(conn, …)`) and makes the command call `apply` once so one step is recorded. `AttributesService::write` is one transaction
   per key (fine: one field = one step); `accept_all` is already one transaction over every row. `ReviewService::transfer` is one transaction for
   the whole set (fine). Grid Split/Normal on a selection (`WorkspacePage.tsx:227–248`) is one `setComposition` per photo → N steps; P41 routes
   ≥ 2 photos through `composition_apply` (reading, P40 Notes (6)).
3. **How does MC-6 `change_seq` interact?** `touch_session` (`store.rs:539–547`) is the one `change_seq` writer for review steps and runs in
   the same transaction; `save_composition` bumps it itself for non-deleted photos (`:995`); `set_item_activity` (`:760`) touches only
   `last_activity_at`. So: undo/redo of classify/delete/reorder/move/copy → `touch_session` in the reversal transaction (+1, dirty as usual);
   of a composition → a new revision bumps it exactly like a save; of an attribute step → `set_item_activity` only, `change_seq` untouched — the
   contract's "photo changes where the step is one" falls out of reusing the same primitives. Rows C2(a)/C3(a)/C4(a) (P39) and C1(a)/C4(a)
   (P40) pin `seq` before/after.
4. **How are colour-tag mirroring and output re-rendering triggered on reversal?** Mirroring: `StoreTx::enqueue_mirror_tag` (`store.rs:467–476`)
   coalesces to one queued `mirror_tag` job per RAW; the old `undo` already called it (`review.rs:140`); the reversal of a classify/delete calls
   it per photo in the same transaction (row C2(a) counts the job). Moves/copies keep the classification, so no mirror. Re-rendering: **no
   save triggers a render** — outputs are cached by `(adjusted_gen, revision, output_format)` (MC-28) and re-rendered when an export/publish run
   finds them stale (`export.rs:466 enqueue_render_output`), and `photo_links.composition_rev` (MC-29, P30 F1) makes the next publish replace the
   file. A reversed composition is written through `save_composition` → `revision + 1` → stale by construction; nothing more is needed, and the
   plans say so rather than adding a render trigger.
5. **Does the history survive what "while the app runs" implies?** Not for free: a runtime settings change rebuilds the whole `App`
   (`AppState::settings_set` → `prepare` → `App::build`, `lib.rs:144–176, 268–286`) — a history stored in `App` would be lost on every SKU-prefix
   edit. P39 carries the `HistoryStore` through `SettingsSetContext` into `App::build` (as `events.transfer_target_to` carries the event target)
   and rebuilds the `History` with the **new** `EventHub` (the old hub has no target after the transfer — emitting through it would lose every
   `history_changed`). Row C5(b) exercises this; C5(c) the relaunch.
6. **Keys and the Edit menu.** The contract wants "Undo <step>" / "Redo <step>" in the Edit menu "with the same keys" **and** ⌘Z inside a text
   field to stay the field's own undo. Tauri's default macOS menu already has a predefined Edit ▸ Undo ⌘Z, and the P08 window-level ⌘Z handler has
   worked beside it since 2026-10-25 (the owner's try-outs): WKWebView gives the page the key first; a `preventDefault` stops the menu accelerator;
   WebKit's editor consumes ⌘Z in a focused field. P41 therefore gives the custom items accelerators (so the menu shows ⌘Z) and routes their
   `on_menu_event` to the same frontend function as the keys; the try-out has an explicit "exactly one step undone" check and the recorded fallback
   (drop the accelerators) if WKWebView ever behaves otherwise. Mechanically: `useComposeKeys`' ⌘ branch returns without swallowing ⌘Z (`:48–66`),
   so a window-level handler also works in the editor; `useWorkspaceKeyboard`'s ⌘Z branch (`useKeyboard.ts:27–31`) is disabled while composing or
   a dialog is open, which is why P41 moves the keys to an app-level hook with only the `isTyping` guard.
7. **Frontend surfaces the contract names that do not exist yet.** The header has no undo icons; `Header.tsx` is rendered by `App` above
   `WorkspacePage`, which owns the active photo — P41 uses a module-level store (the `usePendingCompositions` pattern) so the header reads labels
   and the workspace registers "make this photo active". The editor does not re-sync to an externally changed composition today
   (`useComposeGesture.ts:159–180` reacts to `photo.id` and the stage size only); machine rule 16 (`Saved`) exists for refused saves and is the
   right hook — P41 dispatches it when the stage photo's stored `revision` changes with no save pending (row C5(a), proven through the Esc rule).
8. **Test-surface hazard (suite-wide).** Eleven tests mock `@/services/events` with enumerating factories; a new export there breaks all of
   them at `App` mount. P41 puts the two history listeners in `services/history.ts` instead and makes the hook swallow the "no Tauri" rejections,
   so the old tests pass untouched; the P08 C2(b)/(c) rows are re-pointed to `historyUndo` (listed with line numbers).
9. **Residual ambiguity in MC-30 I resolved as readings, not cards:** step *labels* (the contract gives one example, "Approve 03" — I number
   by position in Main/Condition, name sessions by product title; P39 §Labels is the contract the rows assert); which photo becomes active after
   an undo (classify/delete only, per MC-5); whether undoing a copy may delete rows (yes, until a publish completed after the copy); what
   "removed by a later action that is not a step" means in code (no such path exists today except direct store writes — the precondition
   check is still written and tested with one, C2(e)); the P37 drawer "open state kept while switching photos" is superseded by R40's
   dropdowns, which close like any menu. None changes the product beyond what R40/R41 say.

## Grounding worth knowing (orchestrator / reviewer)
- **Tests that change their expectation** are named with line numbers in each plan's Read first: P38 — `p37_editor_chrome` C2(a) table (`:34–36`
  deleted, `:65` deleted, `:67` opens the dropdown), C5(a), C7(a)/(b), C8(a) deleted and replaced by P38 rows; `p32_controls` C3(c) `:87, :110`
  (menu items instead of buttons); `p36_controls` C6(h). P39 — `p02_review_order` c3b–c3f and `:341` (`a.review().undo` → `a.history().undo`,
  assertions unchanged). P40 — `p10_attributes` `attrs()` (one argument). P41 — `p08_workspace_ui` C2(b)/(c) (mock `historyUndo`, resolved
  shape); six `undoDelete: vi.fn()` mock lines.
- **Every expected number is carried from P31/P32/P37** (`ROTATED` `(0, 0.166602, 1, 0.666797)`; nudge `0.167624 / 0.177851 / 0.176828`; the
  default square `(0.166601562, 0, 0.666796875, 1)`) or from p34's fixture (`V1 = {0.25, (1 − 1280/1707)/2, 0.5, 1280/1707}`); positions in the
  P39/P40 rows are hand-derived from MC-5's table.
- **Projection on for P39 and P40** (flow §2: `projection: yes`): the reversal transactions and the byte-identical attribute restore are where
  a plausible implementation diverges silently (a `revision − 1`, a restore through `set_value` that re-canonicalises and drops `ai_*`, a
  `History` rebuilt with the old `EventHub`, recording inside a transaction that rolls back).
- **Installed-API facts** the P41 Rust task relies on, read from `~/.cargo/registry/src/*/tauri-2.12.5/src/menu/`: `Menu::default(&handle)`
  (`menu.rs:142`), `MenuItem::with_id(manager, id, text, enabled, Option<accelerator>)` (`normal.rs:48`), `MenuItem::{set_text, set_enabled}`
  (`:94, :105`), `PredefinedMenuItem::{cut, copy, paste, select_all, separator}`, `Builder::menu` (`app.rs:2089`), `Builder::on_menu_event`
  (`:2112`). `radix-ui` 1.8.1 (one package) provides `DropdownMenu` and `Popover` for P38.
- **jsdom limits** per plan: the line's look, the filled icon's colour, the menu/pad placement (P38) and the Edit menu itself (P41) are manual
  try-out items; the rows assert DOM shape, attributes, saved values and the service calls.

## Choices the owner may want to know (recorded in the plans' Notes; for the narratives)
1. Step names: "Approve 03", "Condition 02", "Delete 03", "Reorder 03 → 01", "Move to Lamp 2" (the product title), "Compose 03", "Apply framing
   to 4 photos", "Edit Color", "Accept all suggestions", "Clear Color". The number is the photo's place in Main or Condition.
2. Typing into one field with pauses is one undo step; editing a field, another, then the first again is three steps.
3. Undoing a framing on a photo Capture One has since changed (amber "Re-frame") is refused with a notice; the step is dropped and the next
   ⌘Z continues. Undoing a copy or a move into a session that was published since is refused the same way.
4. Undo of an approval or a delete makes that photo active; other undos leave the active photo alone.
5. The history survives a settings change and is empty after a relaunch (as the book says).
6. ⌘Z/⌘⇧Z work with the details form or a picker open, except inside a text field; the notice is a bottom-right toast; the Undo/Redo icons sit
   in the session header beside the camera dot and are absent on the Sessions page.
7. The Edit menu entries show ⌘Z / ⇧⌘Z; the key is handled by the page first (observed since P08), the menu entry acts when picked.
8. The rotate menu entries show words (a menu is text); the nudge pad closes on an outside click or Esc and does not take the keyboard focus.
9. The split line's dark edge is a translucent black halo; tiles get it too.

## Criteria counts
| Phase | Criteria | Rows | `[rule]` rows |
|---|---|---|---|
| P38_editor_polish | 6 | 12 | 11 |
| P39_undo_history_core | 7 | 22 | 1 |
| P40_undo_history_compositions_and_attributes | 7 | 19 | 0 |
| P41_undo_history_ui | 7 | 16 | 12 |
| **Total** | **27** | **69** | **24** |

## Folded in / not folded
Folded: the grid Split on a selection as one step (P41, a direct consequence of MC-30). Not folded: the single save queue for Apply / Split /
Reset (P34/P36 narratives "Not yet" — still open; the history records whatever each path saves), the P33 stage-bar flake (backlog N1), the Item
details / ⋯ hover cards (P37 backlog).

## Lessons for the planner
- When a contract names a UI surface the frontend has no home for yet (the header icons needing the workspace's active photo), decide the
  ownership in the plan (a module store here); left open, two implementers will lift state in opposite directions.
- Before adding an export to a service module, grep the test mocks that enumerate that module's exports; a new export in the wrong module
  breaks every one of them at mount, far from the change.
- A contract that says "kept while the app runs" must be checked against every path that rebuilds the process's objects (here the settings
  rebuild), not only against quit.
