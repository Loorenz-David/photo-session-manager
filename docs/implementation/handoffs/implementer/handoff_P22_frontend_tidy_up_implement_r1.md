---
plan: P22_frontend_tidy_up
role: implement
round: 1
state: DONE
date: 2026-10-29
actor: codex
---
# P22 implementation handoff

## What was built
Published notifications show a local date and time, including older entries whose text contained a millisecond timestamp. The Settings secret mask clears when focused, returns on an empty blur, and keyboard Back leaves text editing alone. A drop keeps the new order through events and shows its destination number and Cover badge while held. The form queues rapid AI actions, keeps invalid text available for correction, flushes Title edits on every close path, and clears a barcode when emptied. Page headers derive Back from the current route. The two item search dialogs tie “shop unavailable” to the failed query.

## Tests → criterion rows
- C1(a): `p22_shell_tidy.test.tsx` — two published rows, readable local date/time and ISO `time`.
- C2(a–b): `p22_shell_tidy.test.tsx` — secret focus/blur/typed save, and Back inside versus outside an input.
- C3(a–b): `p22_workspace_tidy.test.tsx` — event during pending reorder, and held overlay projection across Main, Condition, and own slot.
- C4(a–d): `p22_form_tidy.test.tsx` — queued AI accept with failure reload, invalid decimal correction, immediate Escape after Title edit, barcode clear.
- C5(a): `p22_shell_tidy.test.tsx` — Back label and destination on four routes.
- Updated `p20_drop_continuity.test.tsx` to expect the projected overlay number `03` during a drag to slot 3. The complete frontend suite passes.

## Assumptions
- The session route's label is `Workspace`; its parent is Sessions.
- Publish history uses the plan's full medium local date and short local time.
- An AI popover closes as soon as its queued action is accepted; the action and reload continue in order.
- Repeat dismissal during a pending form close is ignored.
- No live Capture One or visual drag try-out was available in this session.

## Deviations
- Added `apps/desktop/src/lib/routes.ts` under the plan's delegated choice, avoiding a PageHeader/App import cycle.
- Changed the existing P20 overlay test expectation from `01` to `03`, since the old expectation contradicts P22's projected overlay behavior.
- Corrected C1(a)'s ISO expectation in the plan from 2026 to 2025. The specified `1761350400000` is `2025-10-25T00:00:00.000Z`; the production expression and test use that actual instant.
- No backend or generated bindings were changed. P23 planning edits visible in the shared working tree were not made by this session and are excluded from this checkpoint.

## Files changed
- UI: `apps/desktop/src/App.tsx`, `src/lib/{routes,format,keyboard}.ts`, `src/features/{publish/NotificationsDrawer,settings/SettingsPage,shell/{Header,PageHeader},health/HealthPage,sessions/NewSessionDialog,workspace/{PickerDialogs,WorkspacePage,useWorkspaceState},form/{AiPopover,FormDialog,IdentityEditor,useFormState}}.tsx` (with the named `.ts` hook), `src/styles/globals.css`.
- Tests: `src/features/{shell,workspace,form}/__tests__/p22_*`, `src/features/workspace/__tests__/{p09_export_action,p20_drop_continuity}.test.tsx`.
- Process: `docs/implementation/master_plan.md` (P22 tracker row only), `docs/implementation/plans/P22_frontend_tidy_up.md`, this handoff.

## Checkpoint SHA
This handoff is included in the checkpoint commit. Resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P22_frontend_tidy_up_implement_r1.md` (a literal self-SHA cannot be written into a file in the same commit).

## Stamp
- `cargo test --workspace`: exit 0; all Rust tests passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all -- --check`: exit 0.
- `cd apps/desktop && npm run typecheck`: exit 0.
- `cd apps/desktop && npm test`: exit 0, 15 files / 124 tests after the P20 expectation update. An earlier full run had 2 P20 failures from its obsolete `01` expectation and an unclosed drag after that assertion failed; the subsequent full run passed.
- `git diff --check`: exit 0.

## Manual try-out
1. Open the bell. Published entries should show a local date and time, including older entries, with no long number.
2. Open Settings. Focus Client secret to clear the dots; blur without typing to restore them. Type a replacement and save. Press ⌘[ inside an input, then outside it; only the latter returns to Sessions.
3. In a session with three Main photos and one Condition photo, hold photo 01 over slot 3: the copy should read 03 without Cover. Hold it over Condition: it should read the next Condition number. Hold photo 03 over Main slot 1: the copy should read 01 with Cover. Drop during an incoming photo event and check that the order does not jump back.
4. In the form, accept AI suggestions on two fields in quick succession; both should save. Correct an invalid decimal such as `12,5` to `12.5`. Edit Title and press Escape immediately; the workspace card should show the new title. Clear the barcode using the header pencil, save, and reopen to check it remains absent.
5. Check Back on Settings, Health, and a session workspace.

## ⚠ OWNER DECISIONS REQUIRED (0)
none
