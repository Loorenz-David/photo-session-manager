---
plan: T1_templates_and_settings
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-16
actor: opus
---
# T1 review r1 — CHANGES_REQUESTED (1 blocking)

Gate: tracker `IMPLEMENTED`; checkpoints `c6a63a5`, `45d4867`, `5dda588`, `23a9114` exist. One production
defect blocks: in the desktop app, Delete template deletes at once, with no confirmation. Everything else holds.
Seven notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Perimeter.** `git diff --stat c6a63a5^ 23a9114` lists 31 paths. They match the handoff's declared list,
  including the disclosed `commands/items.rs` (error codes), `p01_storage_identity.rs` (table list for
  migration 014) and the two guide files (flow §7A). No extra or missing paths.
- **Checks** (reviewer run, clean tree `4128f5e`, one cargo command at a time):
  - `cargo test -p psm-core mechanisms::template`: 1/1.
  - `cargo test -p psm-testkit --test suite t1_templates::`: 12/12.
  - `cargo test -p psm-desktop`: 27/27, and `git status` was clean afterwards (`bindings.ts` did not change).
  - `npx vitest run src/features/settings`: 18 files, 80 tests.
  - `npm run typecheck`, `cargo clippy -p psm-core -p psm-infra -p psm-desktop --all-targets -- -D warnings`
    and `cargo fmt --all -- --check`: green.
  - The implementer's full-suite stamp stands (900/901, with the stale p01 table list fixed and its module
    20/20).
- **U1 backend:**
  - Migration 014 is registry §4 verbatim, with `IF NOT EXISTS` (the p27 hazard).
  - Every store call runs in one transaction and returns the template as it is afterwards.
  - Containers are deleted before their template (foreign-key hazard).
  - The permutation check (length + set) runs before any write.
  - Remove renumbers both orders 1…n−1 and keeps relative order.
  - Zoom > 80 and shape `free` are refused before the store is touched.
  - An unreadable source is refused before any row is written and leaves no stray file: `thumbnail` returns
    `false` before `create_dir_all`.
  - `type_key` is trim + lowercase. The stored item type is the trimmed text as typed.
  - `templates/` is added to `AppPaths::create` and to the asset-protocol scope.
  - The ten commands are thin (parse → service → DTO), registered in `specta_builder()`, and present in the
    generated bindings.
  - Components use `services/templates.ts`, never `bindings.ts`.
- **U2 drawer:**
  - Sections appear in the plan's order, and Research shows only when `saved.research` exists, as before.
  - All panes stay mounted. Inactive panes are hidden by `.settings-panes [data-active="false"]`, and nothing
    else in the app uses `data-active="false"`.
  - The four form panes stay inside the one `<form>`, and the footer hides outside them.
  - No control was renamed. No existing settings test was changed.
- **U3 editor:**
  - Every action calls its command at once and shows the template the command returns.
  - On a failure, the editor shows an alert and reads the template again (`getTemplate`).
  - Tap order: the tapped containers come first, then the rest in their earlier shooting order.
  - Examples are always shown grayscale (`TemplateExample`, `data-bw`, `filter: grayscale(1)`).
  - The picker accepts JPEG/PNG/TIFF/WebP; `image` 0.25 default features include webp and tiff.
- **Criterion rows.** C1(a)–(c), C2(a)–(h), C3(a)–(b), C4(a), C5(a)–(d) and C6(a)–(g) each have a test at the
  row's boundary that would fail on a wrong outcome. Examples:
  - C2(a) fails without the resize (3000 px source).
  - C2(d) compares the whole template after the refusals.
  - C6(g) fails without the reload, because the controlled checkbox stays checked.
  - C3(b) follows the `a27_c7a` wired pattern, plus the ten names in `bindings.ts`.
- **C6(h)** observes its row in jsdom but cannot see the in-app failure: see F1.
- **Assumptions.**
  - Colour copies shown grayscale, loading data when the pane is selected, local types joined with shop types,
    and file cleanup after the commit are all consistent with 10G §2/§3. They are reported to the owner in
    plain words.
  - "Cancel leaves it unchanged" is false as built (F1).
- **H1–H8 hold:**
  - No external write.
  - Source files are only read.
  - Copies are thumbnails of image files; a RAW is never decoded (H3).
  - No UI automation.
  - Templates work without a shop.
  - Core is free of platform terms.
  - Logic lives in `TemplatesService`; commands stay thin.
  - No secrets and no live services.

## Findings

### F1 — B1 · route: production · Delete template has no working confirmation in the desktop app
- **What is wrong.** `TemplatesSection.deleteCurrent` (`apps/desktop/src/features/settings/TemplatesSection.tsx:135`)
  guards with `if (!template || !window.confirm(...)) return;`. The app registers `tauri_plugin_dialog::init()`
  (`src-tauri/src/lib.rs:1177`). At the locked version 2.8.3, that plugin injects `src/init-iife.js`, which
  replaces `window.confirm` with an **async** function. The guard therefore reads a Promise, which is always
  truthy, and `deleteTemplate` runs immediately.
- **No dialog appears at all.** The capability grants only `dialog:allow-open`, so the plugin's
  `plugin:dialog|message` call is refused (an unhandled rejection).
- **Result.** One press of Delete template removes the template's rows and its example folder, whatever the
  photographer intended. The C6(h) test passes only because jsdom's `confirm` is synchronous and spied.
- **Authority.** Plan T1·U3 task 6 ("Delete template with a confirmation") and row C6(h) ("press Delete
  template, confirm"). The handoff assumption "A cancelled dialog leaves it unchanged" does not hold. 10G §3
  says nothing already saved is lost.
- **Correction.** Make the confirmation awaited and real in the app: either the in-app `Dialog` already used by
  `SettingsPage`, or `ask` from `@tauri-apps/plugin-dialog` wrapped in `services/templates.ts` together with
  `dialog:allow-ask` in the capability. C6(h) then drives that confirmation (confirm → `deleteTemplate` called
  once → Create template shown).
- **Files the fix may touch.**
  - `apps/desktop/src/features/settings/TemplatesSection.tsx`
  - `apps/desktop/src/features/settings/__tests__/t1_templates_editor.test.tsx`
  - optionally `apps/desktop/src/services/templates.ts`, `apps/desktop/src-tauri/capabilities/default.json` and
    `apps/desktop/src/styles/globals.css`
- **Fix scope.** Run the settings vitest and typecheck. No Rust change is needed unless the capability changes,
  and then only `cargo check -p psm-desktop`.

## Notes written to backlog
All seven are under T1 in `docs/implementation/backlog.md`:
1. Guide rule: never use `window.confirm`/`alert` in the desktop UI.
2. C2(c)/(e) check final order by sequence, not exact `final_no` values.
3. C2(d) runs 2 of 4 list/call pairs.
4. The Zoom box saves per keystroke through a controlled input (snap-back/stale-value race).
5. `StockSection` inside the form: Enter in the quantity box now submits Save settings.
6. `template_exists`/`no_template` display as raw codes.
7. File cleanup after commit: orphan copies, and an error reported after a saved change.

## Lessons for the plans
- A confirmation row tested in jsdom cannot see what the real webview does to `window.confirm`. Plans that ask
  for "a confirmation" should name the in-app dialog component, as T3/T4 warnings will need one anyway.

## Write perimeter
- `docs/implementation/plans/T1_templates_and_settings.md` (Tracker line and one Review log entry)
- `docs/implementation/backlog.md` (7 lines appended)
- this handoff
- No production or test file was edited, and no probe touched any file.
- Commands run: the checks above. `export_bindings` rewrote `bindings.ts` byte-identically (`git status` clean).
- No database or state outside temporary test folders.
