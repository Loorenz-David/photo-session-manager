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
| Mechanism contracts MC-1…MC-21 (exact rules) | `book/05A_mechanism_contracts.md`, `book/07A_publish_contracts.md`, `book/07B_retouch_after_publish.md`, `book/10A_capture_one_contracts.md` |
| Measurement ledger M1–M28 (root of every criterion's trace) | `book/14_measurement_ledger.md` |
| Capture One facts and VERIFY probes | `docs/intention/evidence/capture_one_16.8.6.md`; dictionary `docs/reference/capture_one_16.8.6.sdef` |
| Design rules (field kinds, timings, keyboard, components) | `docs/intention/evidence/design_handoff.md`; pictures/prototype `claude_design/` |
| Shopify facts | `docs/intention/evidence/shopify_admin.md` — **does not exist yet**; written in phase P12 (SHOPIFY-VERIFY-1…6) |
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
  (M1–M28), an `MC` id (MC-1…MC-21) or a hard constraint (H1–H8). A row may instead cite a chapter rule
  (`ch07 §Open filter`, `evidence/design_handoff §Field kind map`) **only** for UI-rendering rows, operations
  rows the inventory listed as "not contracted", and packaging rows (ch11); such a cell is prefixed `[rule]` so
  the reviewer knows the row has no ledger measurement behind it. Every plan file derives its own counts of
  criteria, rows and `[rule]` rows.
- **Migrations are never rewritten once a checkpoint commit contains them** (charter rule 7); add the next
  numbered file.
- **Secrets**: the Shopify token lives only in `settings.json` (mode 0600) or the env var `SHOPIFY_TOKEN`
  during a probe; never in logs, fixtures, commits or handoffs.

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
| P11_form_ui | IMPLEMENTED | 2026-10-26 | sonnet | fix 1: the Title now has one home, `items.title` (F1/B1): `form_get` presents it as `product.title`, human set/clear of it updates the item; +1 service test, +1 form test; suite 236 Rust / 78 UI, clippy/fmt/typecheck green |
| P12_shopify_adapter | NOT_STARTED | 2026-10-25 | planner | GraphQL adapter, grounding probes, fixtures |
| P13_publish_runs | NOT_STARTED | 2026-10-25 | planner | confirm/freeze, prepare, upload, gallery mirror, resume |
| P14_publish_update | NOT_STARTED | 2026-10-25 | planner | attribute writes, damage report, retouch replacement |
| P15_publish_ui | NOT_STARTED | 2026-10-25 | planner | popover, progress, failure, batch, notifications |
| P16_operations | NOT_STARTED | 2026-10-25 | planner | settings, health, backup, logs, final build |

Phase state machine (charter): `NOT_STARTED → [PROJECTED] → IMPLEMENTING → IMPLEMENTED → REVIEWING →
CHANGES_REQUESTED (→ IMPLEMENTING) → APPROVED`. The runner (`tools/pipeline/run_phase.sh`) drives it.

## 5. Sequencing and gates
`phases.txt` lists the 17 phases in execution order; the runner is sequential and a phase starts only when every
phase in its `depends_on` is APPROVED (checked by the implementer's gate). Dependency graph:

```
P00 → P01 → P02 → P03 ─────────→ P07 → P08 → P11 → P15 → P16
   ↘        ↘ P04 → P05 → P06 → P09 → P10 → P12 → P13 → P14 ↗
    ─────────↗            (P06 + P07 → P08; P09 + P12 → P13; P10 + P08 → P11; P14 + P11 → P15)
```
Exact `depends_on` per plan: P01[P00] · P02[P01] · P03[P02] · P04[P00,P01] · P05[P04] · P06[P05] ·
P07[P03,P04] · P08[P06,P07] · P09[P06,P02] · P10[P09] · P11[P10,P08] · P12[P10] · P13[P09,P12] · P14[P13] ·
P15[P14,P11] · P16[P15].

- **Local workflow usable after P09** (shoot → arrive → review → live previews → export). Attributes and the
  form (P10, P11) complete the local item data; Shopify starts at P12.
- **Parallelism**: `depends_on` allows P04 beside P02/P03, P07 beside P05/P06, and P12 beside P11. The runner
  commits to one working tree, so parallel runs need separate git worktrees; the orchestrator decides. Nothing in
  this plan requires it.
- **Projection** (flow §2): on for P04 and P12 (criteria rows whose fixtures are owned by Capture One /
  Shopify, charter rule 17). Off elsewhere; P05's one unprobed shape (capture-done argv) is handled by design
  (plan P05 Notes).
- **External preconditions the orchestrator must see met before launching**: P00 — Rust toolchain installed
  (§7.1, owner card 1). P12 — Shopify dev-app token present (`SHOPIFY_TOKEN` env for the probe, then
  `settings.json`), otherwise the grounding task cannot run and the session reports BLOCKED.

## 6. Shared skeleton and naming registry
Fixed up front so sessions cannot diverge. Anything **new** below does not exist yet; P00/P01 create it.

### 6.1 Repository layout and crates (Cargo workspace at the repo root)
| Path | Package / role | Depends on |
|---|---|---|
| `Cargo.toml`, `rust-toolchain.toml` (`channel = "stable"`), `.cargo/config.toml` | workspace, resolver 2, edition 2024 | — |
| `crates/core/` | `psm-core` (lib `psm_core`): domain, mechanisms, ports (traits), services, events, errors. **No Tauri, SQL, HTTP, osascript types.** | serde, uuid, sha2, unicode-normalization, chrono, tracing, thiserror |
| `crates/infra/` | `psm-infra` (`psm_infra`): `paths`, `settings_file`, `sqlite` (Store impl + `migrations/`), `spool`, `nudge` (loopback listener), `watcher` (FSEvents), `osascript` (runner), `rawpreview` (MC-20), `jobs` (runner), `logging`, `backup` | psm-core, rusqlite, rusqlite_migration, notify, axum, tokio, tracing-appender |
| `crates/integrations/` | `psm-integrations` (`psm_integrations`): `editing/capture_one/` (`adapter.rs`, `scripting.rs` trait, `jxa.rs` impl, `jxa/*.js`, `hooks/*.applescript.tmpl`), `camera/via_capture_one.rs`, `commerce/shopify/` (`adapter.rs`, `graphql.rs`, `transport.rs` trait, `type_map.rs`, `fixtures/*.json`) | psm-core, psm-infra (osascript runner), reqwest |
| `crates/testkit/` | `psm-testkit`: `src/` fakes + `TestApp`; `tests/` **all behaviour tests** (§6.9) | psm-core, psm-infra, psm-integrations, tempfile |
| `apps/desktop/src-tauri/` | `psm-desktop` (bin + lib `psm_desktop_lib`): `commands/`, `events.rs`, `state.rs`, `tauri.conf.json`, `Info.plist`, `capabilities/default.json`, `build.rs` | all of the above, tauri, tauri-specta |
| `apps/desktop/` | Vite + React + TS: `package.json`, `src/services/bindings.ts` (**generated, never hand-edited**), `src/services/*.ts`, `src/components/ui/` (shadcn), `src/features/{shell,sessions,workspace,form,publish,settings,health}/`, `src/lib/`, `src/styles/tokens.css` (copied from `claude_design/tokens/tokens.css`) | — |

Layer rule (ch11): core ← nothing above it; services (in core) depend on ports only; infra and integrations
implement ports; `psm-desktop` commands are thin (parse → service → serialize).

### 6.2 Pinned dependency versions (verified on crates.io / npm, 2026-10-25)
Rust: `tauri = "2"` (2.12.5, feature `specta`), `tauri-build = "2"` (2.7.4), `tauri-specta = "=2.0.0-rc.26"`
(features `derive`, `typescript`), `specta = "=2.0.0-rc.26"`, `specta-typescript = "0.0.13"`, `rusqlite = "0.40"`
(feature `bundled`), `rusqlite_migration = "2.6"`, `notify = "8.2"`, `tokio = "1"` (1.54; `rt-multi-thread`,
`macros`, `sync`, `time`, `process`), `axum = "0.8"`, `serde`/`serde_json = "1"`, `uuid = "1"` (`v4`),
`sha2 = "0.11"`, `unicode-normalization = "0.1"`, `chrono = "0.4"` (plus `chrono-tz = "0.10"` as a
dev-dependency for TZ-pinned tests), `tracing = "0.1"`, `tracing-subscriber = "0.3"`, `tracing-appender = "0.2"`,
`thiserror = "2"`, `reqwest = "0.13"` (`json`, `multipart`, `rustls`, no default features; 0.13 has no `rustls-tls` feature), `tempfile = "3"` (dev).
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
- `osascript` is always spawned with **cwd = the data dir** (MC-18 evidence).

### 6.4 Ports (traits in `psm_core::ports`)
`Store` (transactional repository; one SQLite impl; exact method set designed in P01 and extended per phase) ·
`Clock` (`now_ms() -> i64`, `mono() -> Instant`; `SystemClock`, testkit `ManualClock`) · `EditingProvider`
(`inspect_document`, `set_capture_folder`, `install_hooks`, `check_recipes`, `render(raw, RenderPurpose) -> JobId`,
`mirror_tag(raw, Classification)`, `import(paths)`, `capabilities()`) · `CameraProvider` (`status`, `capture`,
`capabilities`) · `CommerceAdapter` (ch09 capability methods; every mutating method takes `&ConfirmedRun`) ·
`RawPreviews` (`extract(raw, dest) -> Option<Extracted{orientation}>`) · `EventSink` (`emit(AppEvent)`).
Capture One internals: `psm_integrations::editing::capture_one::scripting::CaptureOneScripting` (typed
operations over the dictionary; impls `JxaScripting`, testkit `FakeCaptureOneScripting`). Shopify internals:
`commerce::shopify::transport::HttpTransport` (impls `ReqwestTransport`, testkit `FakeTransport`).

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

### 6.6 Enums, codes and kinds (Rust `enum` names in `psm_core::domain`, serialized snake_case)
- `Classification {Unreviewed, Main, Condition, Deleted}` · `PhotoSource {Capture, Import}` · `ItemOrigin {Linked, Local}` ·
  `Integrity {Ok, RawChanged, RawMissing}` · `UploadState {None, Exported, Staged, Uploaded, Attached, Ready, Failed}` ·
  link `role`: `gallery` or `attribute:<key>` · `PublishStep {Prepare, Upload, Update, Describe, Done, Failed}` ·
  `JobKind {OriginalPreview, Hash, RenderPreview, RenderExport, MirrorTag, PublishStep, VerifyHash}` ·
  `JobStatus {Queued, Running, Done, Failed}` · `Provenance {Human, Ai, Default}` · `AiConfidence {High, Medium, Low}` ·
  `AiStatus {Suggested, Accepted, Cleared}` · `AttributeGroup {Details, Condition, Product}` ·
  `CoreType` = the 18 types of ch08 · `SessionStatus {Publishing, Failed, Published, Ready, InProgress}` ·
  `RowLabel {Publishing(pct), PublishFailed, DescriptionFailed, Published, Ready, InProgress(n_to_review)}` ·
  `CameraStatus {Connected(name), Disconnected}` · `RenderPurpose {Preview, Export}` · `MechanismsVersion = 1`.
- Error codes (`psm_core::Error`, snake_case, carried to the UI as `{code, message, data}`): `relative_path`,
  `sku_collision{existing_sku}`, `sku_in_shop{sku}`, `target_has_raw`, `target_published`, `nothing_to_undo`,
  `variant_not_found`, `capture_folder_not_set`, `hooks_not_installed`, `recipe_missing{name}`, `tag_verification`,
  `raw_changed{photo_id}`, `raw_missing{photo_id}`, `several_products_carry_sku{sku}`, `foreign_media_added{n}`,
  `gallery_verification`, `upload_incomplete`, `attribute_verification{keys}`, `not_confirmed`, `timeout`,
  `no_integration`, `invalid_value`.
- Health codes (`HealthItem{code, ok, detail}`): `camera_disconnected`, `document_not_open`, `document_not_session`,
  `hooks_not_installed`, `recipe_missing`, `capture_folder_not_set`, `variant_not_found`, `integration_token_invalid`.
- Colour-tag mapping (MC-19): `main 4`, `condition 5`, `deleted 1`, `unreviewed 0`.

### 6.7 Tauri commands (`#[tauri::command] #[specta::specta]`, snake_case) and events
Commands by phase (payload types are the service DTOs; thin wrappers only):
P00 `health_get`, `app_paths_get` · P01 `item_create`, `item_update_identity`, `session_create`, `session_get` ·
P02 `photos_classify`, `photos_reorder`, `photos_delete`, `photos_undo`, `photos_move`, `photos_copy` ·
P03 `sessions_list` · P04 `session_activate`, `session_deactivate`, `camera_status`, `camera_retry`, `camera_capture` ·
P05 `photos_import`, `photos_assign_unassigned` · P07 `items_search` · P09 `export_run` ·
P10 `attributes_definitions_list`, `attribute_values_get`, `attribute_values_set`, `attribute_ai_accept_all`,
`attribute_ai_clear`, `presets_get`, `presets_set`, `form_get` · P13 `publish_preview`, `publish_confirm`, `publish_retry`,
`notifications_list` · P16 `settings_get`, `settings_set`.
Events (`AppEvent` → Tauri event name): `session_changed{session_id}`, `photo_added{photo_id, session_id}`,
`photo_changed{photo_id, session_id}`, `publish_progress{session_id, item_id, step, uploaded, total}`,
`export_progress{session_id, done, total, state}`, `health_changed{}`, `camera_status_changed{status}`.
Bindings: `tauri_specta::Builder::<tauri::Wry>::new().commands(collect_commands![…]).events(collect_events![…])`;
exported by the test `psm-desktop::export_bindings` to `apps/desktop/src/services/bindings.ts`
(`specta_typescript::Typescript::default()`); the committed file must equal the regenerated one.

### 6.8 Settings file keys (`settings.json`, user configuration; `psm_infra::settings_file::Settings`)
`document_id` (Capture One document folder path, MC-19) · `preview_recipe` (default `"PSM Preview"`) · `web_recipe`
(`"PSM Web"`) · `style_name` (`"PSM House"`) · `export_folder` (default `~/Pictures/Session Manager Export`) ·
`preview_delay_s` (15; read clamped 10–30, MC-17) · `theme` (`"light"`) · `sku_prefix` (owner card 2 answered
"generate", 05B MC-22; absent or empty = no proposal) · `integration` (`null` or `{"kind":"shopify","shop_domain","api_version":"2026-10","token"}`).
Naming rule: snake_case keys; booleans prefixed `is_`/`has_` if any are added; durations carry their unit suffix.

### 6.9 Test layout
- Behaviour tests: `crates/testkit/tests/<phase_lower>_<slug>.rs` (e.g. `p02_review_order.rs`); one `#[test]` per
  criterion row named `c<criterion><row>_<slug>` (e.g. `c3b_undo_restores_position`). They use `TestApp` (§7.5).
- Core-mechanism unit tests: inline `#[cfg(test)] mod tests` in `crates/core/src/mechanisms/<mechanism>.rs`,
  functions named `mc<n>_<slug>`; only for functions a plan lists under **Core mechanisms**.
- Frontend tests: `apps/desktop/src/features/<feature>/__tests__/<phase_lower>_<slug>.test.tsx` with vitest + jsdom
  + Testing Library; `src/services/*` mocked with `vi.mock`.
- Fixtures: `crates/testkit/src/fixtures/` (synthetic RAW builder, hook payloads, settings) and
  `crates/integrations/src/commerce/shopify/fixtures/*.json` (sanitized Shopify captures, P12).

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
| Shopify token | STATUS.md | not available yet (precondition for P12) |
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
  over this fake (so adapter logic is under test) — never a second implementation of `EditingProvider`.
- `FakeCommerce`: in-memory shop (products with variant SKU, media list in order with filenames incl. Shopify-style
  uniquifiers, files, metafields, tags); `fail_after_effect(step)` to simulate a kill between remote effect and local
  record; records every mutation with its `ConfirmedRun`. `FakeTransport` replays sanitized GraphQL fixtures for the
  real Shopify adapter's tests.
- `SyntheticRaw::build(streams, orientation)` writes a minimal TIFF with IFD0/IFD1 JPEG streams for MC-20 tests.

## 8. Tool protocols
No architecture graph, no contract system in this repo (ch03). Sessions use this file and the plan file only.

## 9. Amendment log
- 2026-10-25 v1 — created by the planner (round 1).
- 2026-10-25 P00 r1 continuation — corrected the stale environment note after resolving SQLite, running the workspace suite and building an ad-hoc bundle; recorded the LaunchServices refusal at the bundle-launch boundary.
- 2026-10-25 P00 r1 close — recorded the orchestrator's successful launch outside the Codex sandbox and the remaining interactive Automation check.
