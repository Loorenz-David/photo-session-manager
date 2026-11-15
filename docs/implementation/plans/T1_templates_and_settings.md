---
phase: T1_templates_and_settings
title: Shooting templates can be made and kept — Settings becomes a drawer of sections with the existing settings moved unchanged, and a new section, Shooting templates, where you make a template per item type from example photos (shown in black and white), put the containers in final order, set split, shape and zoom per container, and tap in the order you shoot; every change is saved at once
difficulty: high
projection: no
depends_on: []
blocked_on: none
---
# T1 — Shooting templates can be made and kept

**Tracker:** NOT_STARTED · 2026-11-16 · planner · plan written from 10G (ratified 2026-11-16)

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. `master_plan.md` §4 and `STATUS.md` are not used for these phases.)

## Goal
Book `10G` §2 and §3. (U1) Two tables, a service and ten commands hold shooting templates: at most one per item type, an ordered list of containers, a shooting order, per container an optional example image and framing presets. (U2) The Settings page gets a side list of sections; every existing control keeps its content and behaviour. (U3) The section **Shooting templates** edits a template and saves every change at once. **Not here:** anything that happens to a photo (T2–T4); `photo_places` (T2); the faded containers in a session (T2). After this phase a session behaves exactly as before.

## How to work this plan (flow.md §10, master_plan_templates.md §3)
One runner phase made of 3 units. Work them in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids are `T1·C<n>(<row>)`; name each test after its row (`t1_c2a_<slug>`). One behaviour test per row at the boundary the row names; unit tests only for the Core mechanisms. Decide open details as the smallest option consistent with 10G and record them under **Assumptions**; never stop to ask.

| Unit | What it builds | Criteria | Rows | Changes |
|---|---|---|---|---|
| U1 | Tables, service, commands, bindings | 4 | 14 | Rust, generated bindings |
| U2 | Settings as a drawer of sections | 1 | 4 | TypeScript |
| U3 | The Shooting templates editor | 1 | 8 | TypeScript |
| **Total** | | **6** | **26** | |

## Read first (whole phase)
`docs/process/flow.md` (§4, §4A, §6, §10) · `docs/implementation/master_plan_templates.md` (all; §4 holds every name) · `docs/implementation/master_plan.md` §3, §7 · book `docs/intention/book/10G_shooting_templates.md` (§2, §3, §7, §8 A1) · `docs/guide/03_rules_and_principles.md` · `docs/guide/04_change_impact.md` rows "Tauri command/DTO", "The schema", "`AppPaths` layout", "Add a behaviour-test file" · then each unit's own Read first.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Templates live in the app's database on this Mac**, not in the settings file, so each change is saved by itself and the Save button of the other sections is not involved.
- **A type matches without regard to capitals or spaces at the ends** ("Dining chair" = "dining chair ").
- **Example photos are copied into the app's data folder as JPEG (long edge at most 1600 px)**; the original file is not touched. JPEG, PNG, TIFF and WebP work; a file the app cannot read (a RAW, a HEIC) is refused with a sentence and nothing is lost.
- **Black and white is how the examples are shown**, everywhere; the stored copy keeps its colour.
- **A container may be added without a photo** (it shows its number only).
- **A new container is shot last** until you set the shooting order; removing a container closes the gap in both orders.
- **Tapping the shooting order saves after every tap:** the containers tapped so far come first, in tap order, the others follow in their earlier order.
- **Zoom is a percentage, 0 to 80**: 20 means the crop keeps 80 % of the width and height, centred.
- **Shape "free" is not offered as a preset** (it has no crop of its own).
- **The sections:** Capture One · Camera · Export and previews · Shopify (with the stock setting) · Storage · Research · Shooting templates. Capture One, Camera, Export and previews and Shopify still share the one **Save settings** button.

## Unit 1 — Tables, service, commands

**Unit id:** `T1·U1` · **Rows:** T1·C1–C4

### Goal
The stored shape and the editor's backend, reachable through the wired desktop app. **Not in this unit:** any screen.

### Read first
`master_plan_templates.md` §4 · `crates/infra/src/sqlite/store.rs` (`SqliteStore::migrations()`, how a store method is written) · `crates/infra/src/sqlite/migrations/012_photo_denials.sql` (the `IF NOT EXISTS` form) · `crates/infra/src/paths.rs` · `crates/core/src/ports/{store.rs, images.rs}` (`Imaging::thumbnail`) · `crates/infra/src/imaging/mod.rs` (`ImageOpsImpl`) · `crates/core/src/{error.rs, domain/composition.rs (Shape)}` · `apps/desktop/src-tauri/src/commands/photos.rs` (the command pattern: `state.handles()`, `spawn_blocking`, `CommandId`) · `apps/desktop/src-tauri/src/lib.rs` (`specta_builder()`, the asset-protocol scope, the inline wired test `a27_c7a_…` built on `mock_builder`) · `crates/testkit/src/test_app.rs`.

### Files
New: `crates/infra/src/sqlite/migrations/014_shooting_templates.sql` · `crates/core/src/domain/template.rs` · `crates/core/src/mechanisms/template.rs` · `crates/core/src/services/templates.rs` · `apps/desktop/src-tauri/src/commands/templates.rs` · `apps/desktop/src/services/templates.ts` · `crates/testkit/tests/suite/t1_templates.rs`.
Edited: `crates/core/src/{domain/mod.rs, mechanisms/mod.rs, services/mod.rs, error.rs, ports/store.rs}` · `crates/infra/src/{paths.rs, sqlite/store.rs}` · `apps/desktop/src-tauri/src/{commands/mod.rs, lib.rs}` · `apps/desktop/src/services/bindings.ts` (generated by `cargo test -p psm-desktop export_bindings`, never by hand) · `crates/testkit/tests/suite/main.rs` (`mod t1_templates;`) · `crates/testkit/src/test_app.rs` only if a helper is needed (each helper has a caller here).

### Core mechanisms (the only functions with unit tests)
`mechanisms::template::type_key`.

### Ordered tasks
1. Migration `014_shooting_templates.sql` with the two tables of `master_plan_templates.md` §4, `CREATE TABLE IF NOT EXISTS`, and its `M::up(include_str!(…))` line after 013.
2. `AppPaths::create` adds `templates`; the asset-protocol scope in `src-tauri/src/lib.rs` allows `<data>/templates` beside `previews`.
3. Domain: `Template { id, item_type, containers }`, `Container { id, final_no, shoot_no, example_file, example_path, presets }` (`example_path` is the absolute path for the screen, none when there is no file), `Presets { split, shape: Option<Shape>, zoom: u8 }`. `type_key(item_type)` = trimmed, lower-cased.
4. `TemplatesService` (store, clock, `&dyn Imaging`, the templates folder). Every call is one transaction and returns the template as it is afterwards:
   - `list()` (item type and number of containers per template, by item type); `get(item_type)` (by `type_key`; none when absent);
   - `create(item_type)`: the trimmed text is stored as typed; a second template for the same key → `template_exists`; an empty type → `invalid_value`;
   - `delete(template_id)`: rows and the template's folder;
   - `container_add(template_id, source: Option<path>)`: `final_no` and `shoot_no` = n + 1. With a source: `Imaging::thumbnail(source, <data>/templates/<template id>/<uuid>.jpg, 1600)`; `false` → `template_image_unreadable`, nothing written;
   - `container_example_set(container_id, source)`: the same conversion; the earlier file is deleted after the row is updated;
   - `container_remove(container_id)`: the row and its file; the remaining `final_no` and `shoot_no` are renumbered 1…n−1, each keeping its relative order;
   - `containers_reorder(template_id, container_ids)` sets `final_no` by list order; `shooting_order_set(template_id, container_ids)` sets `shoot_no` by list order. A list that is not exactly the template's containers, each once → `invalid_value`, nothing changed;
   - `container_presets_set(container_id, presets)`: zoom above 80 or shape `free` → `invalid_value`.
5. Error variants and codes `template_exists`, `template_image_unreadable` (message: "The app could not read that file as an image. Use a JPEG, PNG or TIFF."), `no_template` (used from T2).
6. The ten commands of `master_plan_templates.md` §4 in `commands/templates.rs` (thin: parse → service → DTO; `ImageOpsImpl` and `app.paths` passed to the service), registered in `specta_builder()`; regenerate the bindings; wrappers in `services/templates.ts` (components never import `bindings.ts`).
7. Tests for the rows below.

### Criteria (4 criteria, 14 rows)
Boundary: `TemplatesService` on `TestApp` (real SQLite, temp data folder, the real `ImageOpsImpl`; example sources are small images written by the test); the wired desktop command test.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T1·C1(a) | `create`, `get`, `list` | `create("Dining chair")` on an empty database; `get("dining chair ")` | a template with no container; the get returns that template (same id); `list` has one entry, item type "Dining chair", 0 containers | 10G §2 (one per type), §3 |
| T1·C1(b) | `create` | then `create("DINING CHAIR")` | refused, code `template_exists`; `list` still has one entry | 10G §2 |
| T1·C1(c) | `delete` | a template with two containers that have example files; `delete` | `get` returns none; `list` is empty; the folder `<data>/templates/<id>` no longer exists | 10G §3 (a template can be deleted) |
| T1·C2(a) | `container_add` | three adds: a JPEG 3000×2000, a PNG, no source | containers with `final_no` 1, 2, 3 and `shoot_no` 1, 2, 3; the first two have an example file under `<data>/templates/<id>/` that decodes as JPEG with long edge ≤ 1600; the third has none; the two source files still exist unchanged | 10G §3 (upload examples), A1 |
| T1·C2(b) | `container_add` | then a source file with text bytes named `x.jpg` | refused, code `template_image_unreadable`; the template still has its three containers and two files | 10G §3 (a failure is shown, nothing saved is lost) |
| T1·C2(c) | `containers_reorder` | containers c1, c2, c3 as in (a); the list [c3, c1, c2] | `final_no`: c3 = 1, c1 = 2, c2 = 3; `shoot_no` unchanged (c1 = 1, c2 = 2, c3 = 3) | 10G §3 (final order) |
| T1·C2(d) | `containers_reorder`, `shooting_order_set` | the lists [c1, c2] and [c1, c1, c2], on each call | each refused, code `invalid_value`; both orders as before | 10G §2 (a permutation) |
| T1·C2(e) | `shooting_order_set` | the state after (c); the list [c2, c3, c1] | `shoot_no`: c2 = 1, c3 = 2, c1 = 3; `final_no` as after (c) | 10G §3 (shooting order) |
| T1·C2(f) | `container_remove` | the state after (e); remove c1 | two containers; `final_no`: c3 = 1, c2 = 2; `shoot_no`: c2 = 1, c3 = 2; c1's example file is gone | 10G §3 |
| T1·C2(g) | `container_presets_set` | `{split: true, shape: "4:5", zoom: 20}` on c3; then zoom 81; then shape `free` | after the first, `get` shows exactly those presets on c3 and none on c2; the second and third are refused, code `invalid_value`, and the presets stay | 10G §2 (presets) |
| T1·C2(h) | `container_example_set` | a container with an example; a second image | the container names a new file that exists; the earlier file does not | 10G §3 |
| T1·C3(a) | `TestApp::kill_and_restart`, then `get` | the state after C2(g) | the same template: ids, both orders, presets, and every example file exists | A1 (quit and reopen: all kept) |
| T1·C3(b) | the wired app (as `a27_c7a_…`): commands `template_create`, `template_container_add`, `template_get`; the generated bindings | create "Chair", add one container without a source, get "chair" | the get answers one container with `final_no` 1 and `shoot_no` 1; `bindings.ts` contains the ten command names of this phase | 10G §3 |
| T1·C4(a) | `mechanisms::template::type_key` | "Dining chair", "  STOL ", "Ö-stol" | "dining chair", "stol", "ö-stol" | 10G §2 |

### Delegated choices
Names and signatures of the store methods · the DTO field names (snake_case, ids as `CommandId` / `Number` like the other commands) · whether `list` is one query or two · the JPEG quality of the copy (the port's).

### Notes
- Hazard: `p27_retire_shape_on_arrival` rewinds `user_version` and re-runs the later migrations over existing tables; a `CREATE TABLE` without `IF NOT EXISTS` fails there, in the closing gate only.
- Hazard: foreign keys are enforced; delete containers before their template.
- `item_types_list` needs a shop connection; templates do not. Nothing here calls the commerce adapter.
- The service does not look at sessions or photos.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-core mechanisms::template` · `cargo test -p psm-testkit --test suite t1_templates::` · `cargo test -p psm-desktop` · `cd apps/desktop && npm run typecheck` · `cargo clippy -p psm-core -p psm-infra -p psm-desktop --all-targets -- -D warnings` · `cargo fmt --all -- --check`

## Unit 2 — Settings as a drawer of sections

**Unit id:** `T1·U2` · **Rows:** T1·C5

### Goal
`#/settings` shows a side list of sections and one section at a time. Every existing control, text, save path and the unsaved-changes prompt work as before. **Not in this unit:** any redesign of a section; the content of Shooting templates (U3; here the section exists and shows a placeholder line).

### Read first
book `10G` §3 first bullet · `apps/desktop/src/features/settings/SettingsPage.tsx` (one `<form>` with the Save footer; `StockSection`, `StorageSection`, `ResearchSection` after it) · the tests in `apps/desktop/src/features/settings/__tests__/` (they render the page and reach controls by label and role) · `apps/desktop/src/styles/globals.css`.

### Files
New: `apps/desktop/src/features/settings/SettingsNav.tsx` · `apps/desktop/src/features/settings/__tests__/t1_settings_drawer.test.tsx`. Edited: `SettingsPage.tsx` · `apps/desktop/src/styles/globals.css` · an existing settings test only if it fails, and then only by choosing the section first (list each in the handoff).

### Core mechanisms
None.

### Ordered tasks
1. A navigation list labelled "Settings sections" at the side of the page with the sections, in this order: Capture One · Camera · Export and previews · Shopify · Storage · Research (only when the page shows `ResearchSection` today) · Shooting templates. The first is active when the page opens. The active item has `aria-current="page"`.
2. Each section is a pane (`data-section="<name>"`, `data-active="true|false"`). **All panes stay mounted**; an inactive pane is hidden by the stylesheet (`[data-active="false"] { display: none }`), not unmounted, so the form's state, the one Save and the leave guard are untouched. The existing blocks move as they are: Capture One (session, recipes, style, default shape) · Camera (`CameraSection`) · Export and previews · Shopify (the four fields, then `StockSection`) · Storage (`StorageSection`) · Research (`ResearchSection`). The four panes that belong to the form stay inside the one `<form>`; the **Save settings** footer carries `data-active="true"` while one of those four is active and `"false"` (hidden the same way) on Storage, Research and Shooting templates.
3. The Shooting templates pane renders `TemplatesSection` (a stub line "Shooting templates" in this unit).
4. No control is renamed, re-labelled or removed.

### Criteria (1 criterion, 4 rows, all `[rule]`)
Boundary: `SettingsPage` rendered in a UI test (`getSettings`, `saveSettings` and the other services mocked as the existing settings tests do).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T1·C5(a) | `SettingsPage` | settings with an integration and research settings | a navigation "Settings sections" with exactly Capture One, Camera, Export and previews, Shopify, Storage, Research, Shooting templates, in that order; Capture One has `aria-current="page"`; its pane has `data-active="true"`, the six others `"false"` | `[rule]` 10G §3 (sectioned) |
| T1·C5(b) | same | click "Shopify" | the pane that holds the "Shop domain" field has `data-active="true"`; the pane that holds "Preview recipe" has `"false"` | `[rule]` 10G §3 |
| T1·C5(c) | same | change "Preview recipe"; click "Shopify"; change "Shop domain"; press Save settings | `saveSettings` is called once and its argument carries both new values | `[rule]` 10G §3 (content moved unchanged) |
| T1·C5(d) | same | click "Shooting templates"; then "Camera" | the Save settings footer (`data-testid="settings-footer"`) has `data-active="false"` on Shooting templates and `"true"` on Camera | `[rule]` 10G §3 |

### Delegated choices
The look of the list (use the design tokens; a plain vertical list is enough) · whether the list collapses on a narrow window.

### Notes
- Hazard: unmounting a pane loses typed values and breaks `p63`/`p68` (unsaved-changes prompt). Hidden, never unmounted.
- vitest does not load stylesheets, so the existing tests still reach every control. If one fails all the same, select its section in the test; change no assertion.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/settings src/features/shell src/features/pending` · `npm run typecheck`

## Unit 3 — The Shooting templates editor

**Unit id:** `T1·U3` · **Rows:** T1·C6

### Goal
The section where a template is made and changed; no Save button; a failed change is shown and the screen returns to what is stored.

### Read first
book `10G` §3 second bullet, §8 A1 · `apps/desktop/src/services/templates.ts` (U1) · `apps/desktop/src/features/form/ItemTypePicker.tsx` (`useItemTypes`, `ItemTypePicker`: a text box with the shop's types as suggestions; works without a shop) · `apps/desktop/src/services/photos.ts` (`pickImportFiles`: the file picker) · `apps/desktop/src/services/sessions.ts` (`previewUrl`) · `apps/desktop/src/features/workspace/PhotoGrid.tsx` (dnd-kit sortable) · `docs/intention/evidence/design_handoff.md`.

### Files
New: `apps/desktop/src/features/settings/TemplatesSection.tsx` · `apps/desktop/src/features/settings/TemplateExample.tsx` · `apps/desktop/src/features/settings/__tests__/t1_templates_editor.test.tsx`. Edited: `apps/desktop/src/services/templates.ts` (a file picker for images, if not already there) · `apps/desktop/src/styles/globals.css`.

### Core mechanisms
None.

### Ordered tasks
1. `TemplateExample`: the one component that shows an example image, always black and white (`data-bw`, `filter: grayscale(1)`), through `previewUrl(example_path)`; with no image, an empty frame.
2. `TemplatesSection`: an item type box (`ItemTypePicker`; suggestions = the shop's types when available plus the types that have a template) and a list of the existing templates to pick from. A type with no template: **Create template**. A type with one: its containers as cards in final order, each with its number, the example, a small badge with its shooting number, and its presets: **Split** (on/off), **Shape** (None, Original, 1:1, 4:5, 3:2, 2:3, 16:9), **Zoom %** (0–80), **Replace photo**, **Remove**.
3. **Add photos** opens the picker (several files) and adds one container per file, in the order chosen; **Add empty container** adds one without a photo.
4. Final order: drag a card (dnd-kit), and **Move earlier** / **Move later** buttons on each card (the same handler; they make the order reachable by keyboard and by a test).
5. **Set shooting order**: a mode; each tap on a card gives it the next number and saves at once (`shooting_order_set` with the tapped containers in tap order, then the others in their earlier shooting order); **Done** leaves the mode; entering it again starts from 1.
6. **Delete template** with a confirmation.
7. Every control calls its command at once and shows the template the command returned. On a failure: the message in an alert, and the template is read again (`template_get`).

### Criteria (1 criterion, 8 rows, all `[rule]`)
Boundary: `TemplatesSection` rendered in a UI test, `services/templates` and the picker mocked.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| T1·C6(a) | `TemplatesSection` | type "Dining chair" (no template); press Create template | `createTemplate("Dining chair")` is called once; the editor for that template is shown with no card | `[rule]` 10G §3, A1 |
| T1·C6(b) | same | a template with three containers with examples, shooting order c2, c3, c1 | three cards in final order numbered 1, 2, 3; each example has `data-bw`; the shooting badges read 3, 1, 2 | `[rule]` 10G §3 (shown in black and white; second number) |
| T1·C6(c) | same | press Add photos; the picker returns two paths | `addContainer` is called twice, with the first path then the second; the cards shown are those of the last returned template | `[rule]` 10G §3 (one per container) |
| T1·C6(d) | same | on card 3 switch Split on; then set Zoom to 20 | `setPresets(c3, {split: true, shape: null, zoom: 0})`, then `setPresets(c3, {split: true, shape: null, zoom: 20})`; the page has no Save button | `[rule]` 10G §3 (saved immediately) |
| T1·C6(e) | same | containers c1, c2, c3 with shooting order c1, c2, c3; press Set shooting order; tap card 2, then card 3 | after the first tap `setShootingOrder(t, [c2, c1, c3])`; after the second `setShootingOrder(t, [c2, c3, c1])` | `[rule]` 10G §3 (tap in order) |
| T1·C6(f) | same | press Move earlier on card 3 | `reorderContainers(t, [c1, c3, c2])` | `[rule]` 10G §3 (final order) |
| T1·C6(g) | same | `setPresets` rejects with message "disk full" | an alert shows "disk full"; `getTemplate` is called again and the card shows the presets it returned | `[rule]` 10G §3 (a failure is shown, nothing lost) |
| T1·C6(h) | same | press Delete template, confirm | `deleteTemplate(t)` is called once; the Create template button is shown for that type | `[rule]` 10G §3 |

### Delegated choices
Layout and wording beyond the names in the tasks · how the list of existing templates looks · whether Zoom is a number box or a slider (the saved value is the row's) · debounce of the Zoom box (at most one call per value; the row needs the final value saved).

### Notes
- The example images are loaded through the asset protocol; U1 allowed the folder.
- No call to `saveSettings` from this section.

### Unit test commands
`cd apps/desktop && npx vitest run src/features/settings` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time. A failure is fixed and only the failing files re-run.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core mechanisms::template` · `cargo test -p psm-testkit --test suite t1_templates::` · `cargo test -p psm-desktop` (and `git diff --stat apps/desktop/src/services/bindings.ts` is empty afterwards) · `cd apps/desktop && npx vitest run src/features/settings` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check: no existing settings test lost an assertion; every pane stays mounted. Notes go to `docs/implementation/backlog.md` under T1.

## Manual try-out (the narrative is built from this)
On scratch data (`PSM_DATA_DIR=$HOME/psm-templates-dev npm run tauri -- dev`):
1. Open Settings: a list of sections at the side. Click through them: every setting you know is there; Save settings still saves the first four.
2. Shooting templates → type "Dining chair" → Create template.
3. Add photos: choose four or five pictures of a chair. They appear in black and white, numbered.
4. Drag them into the order of the listing. On the third, switch Split on and set Zoom to 20.
5. Set shooting order: tap the containers in the order you shoot them; each shows a second, small number. Done.
6. Quit the app and open it again: everything is as you left it.
7. Try a file that is not a picture: a sentence says so, and the template is unchanged.

## Review log
