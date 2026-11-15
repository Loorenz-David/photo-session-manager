# Shooting templates — master plan (v1, 2026-11-16)

Short on purpose. It adds to `docs/implementation/master_plan.md` (roles, standing rules §3, environment §7, registry §6) and wins over it on a name listed here. Process: `docs/process/flow.md`, with the deltas of §3 below.

## 1. Goal
Build book `docs/intention/book/10G_shooting_templates.md` (RATIFIED 2026-11-16): per item type a shooting template; photos fill its containers by shot order on arrival; Condition after the last container; manual changes win, with a warning and a way back; Settings as a drawer of sections. Mechanical: no model, no network. Semantics live in 10G; §5 below is the part 10G §4.1 hands to planning (the exact order key and placement rule).

## 2. Phases (one runner phase each; order in `docs/implementation/phases_templates.txt`)
| Id | Builds | Units | Criteria | Rows | Useful on its own |
|---|---|---|---|---|---|
| `T1_templates_and_settings` | tables, service and commands for templates; Settings as a drawer; the Shooting templates editor | 3 | 6 | 26 | templates can be made and kept; nothing else changes |
| `T2_template_placement` | shot order, placement on arrival, presets, the session view with faded numbered containers and the info button | 3 | 8 | 38 | photos file themselves; a hand change is kept and never undone (no warning yet) |
| `T3_template_hand_changes` | placed-by-hand marking, drop onto a container (fill, swap), the warning, Back to template, undo | 4 | 4 | 24 | the owner's story up to A6 |
| `T4_template_retake_and_shift` | retake onto a filled container, Deny leaves the container empty, Shift the rest back, the story test, the guide | 3 | 4 | 14 | A7, A8; the whole story proven in one test |
Each depends on the one before (`depends_on`), gate = the **Tracker** line of that plan file. `master_plan.md` §4 and `STATUS.md` are not used or edited by these phases. Start each with `tools/pipeline/run_phase.sh phase <id>` (the runner resolves any id as `docs/implementation/plans/<id>.md`; read 2026-11-16, no change needed; `run_phase.sh all` reads `phases.txt`, not this list).

## 3. Process deltas (10G §10, owner: "open for you to modify the flow")
- No mechanism-inventory gate and no projection (`projection: no` in every plan). §5 below stands in for the mechanism contract of the one silent-failure mechanism (ordering).
- A phase is a milestone made of units, as `flow.md` §10 (the recorded reason for more than 8 criteria per phase: four cycles overnight, each closing gate costs 10–17 minutes). Units in order, a `CHECKPOINT (not approved):` commit after each, one review, one fix round, one narrative.
- Codex implements (`difficulty: high`), Opus reviews. No plan sets `implementer:`.
- The full suite runs once per phase, in its Closing gate (`flow.md` §4A). While working: the phase's stem only.
- Row ids `T<n>·C<k>(<row>)`; tests are named after their row (`t2_c3a_<slug>`). Trace cells name 10G sections and its acceptance checks A1–A9 (10G §8 is this feature's measurement ledger).
- Reviewer notes go to `docs/implementation/backlog.md` under the phase id.
- **Join:** when T1–T4 are approved and the full checks pass, the orchestrator opens a PR from `feature/agentic` into `main` on GitHub (owner answer 4), as research did. The release build for the work Mac is made from a separate worktree with the SDK sidecar (`docs/guide/05_working.md` §Release build).

## 4. Names (registry; exact)
**Tables** (numbered migrations, `CREATE TABLE IF NOT EXISTS`, because `p27` rewinds `user_version` and re-runs later files):
- `014_shooting_templates.sql` (T1): `shooting_templates(id INTEGER PRIMARY KEY, item_type TEXT NOT NULL, type_key TEXT NOT NULL UNIQUE, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL)`; `template_containers(id INTEGER PRIMARY KEY, template_id INTEGER NOT NULL REFERENCES shooting_templates(id), final_no INTEGER NOT NULL, shoot_no INTEGER NOT NULL, example_file TEXT, split INTEGER NOT NULL DEFAULT 0, shape TEXT, zoom INTEGER NOT NULL DEFAULT 0)`.
- `015_photo_places.sql` (T2): `photo_places(photo_id INTEGER PRIMARY KEY REFERENCES photos(id), slot INTEGER, place_by_hand INTEGER NOT NULL DEFAULT 0, frame_by_hand INTEGER NOT NULL DEFAULT 0, released INTEGER NOT NULL DEFAULT 0, key_at INTEGER, key_name TEXT, pending_zoom INTEGER)`.
Research's schema steps (`ensure_research_columns`, `psm_research::schema::ensure`) are not in the numbered list and are not touched.

**Why SQLite and not `settings.json`:** placement reads the template inside its own transaction; every editor change is its own small write with no Save button, no leave guard and no App rebuild (a `settings.json` field outside the `Simple` list rebuilds the App on save); the database is per Mac and is in the backup. No per-session template link: the template is looked up by the session's item type at each use.

**Files:** example images at `<data>/templates/<template id>/<uuid>.jpg` (`AppPaths::create` adds `templates`; the asset-protocol scope in `apps/desktop/src-tauri/src/lib.rs` allows that folder beside `previews`). Never in the repo.

**Core (`crates/core/src`):** `domain/template.rs` (`Template`, `Container`, `Presets { split, shape: Option<Shape>, zoom: u8 }`, `PlaceRow`, `TemplateView`); `mechanisms/template.rs` (`type_key`, `expected`, `zoomed`, `preset_values`); `mechanisms/shot_order.rs` (`order`); `services/templates.rs` (`TemplatesService`, the editor); `services/template_placement.rs` (T2 `place_arrival`, `session_view`, the hand-change hooks; T3 `drop_on_container`, `back_to_template`; T4 the retake, `shift_back`). Store methods are added to the `Store` / `StoreTx` ports with the prefixes `template_` and `photo_place`; their exact names are the implementer's.

**Commands** (`apps/desktop/src-tauri/src/commands/templates.rs`, registered in `specta_builder()`; wrapper `apps/desktop/src/services/templates.ts`): T1 `templates_list`, `template_get`, `template_create`, `template_delete`, `template_container_add`, `template_container_remove`, `template_container_example_set`, `template_containers_reorder`, `template_shooting_order_set`, `template_container_presets_set` · T2 `template_session_view` · T3 `template_drop`, `template_back` · T4 `template_shift_back`.

**Error codes** (`psm_core::Error`, snake_case): `template_exists`, `template_image_unreadable`, `no_template`, `container_filled` (T3 only; retired by T4), `nothing_to_shift`; a bad list, number or preset is the existing `invalid_value`.

**History:** `StepKind::Template` (T3), labels "Move to container <n>", "Back to template", "Retake container <n>", "Shift the rest back".

**No new** settings key, AppEvent, health code or job kind. The UI re-reads `template_session_view` on the existing `photo_added`, `photo_changed`, `session_changed`.

**Frontend:** `features/settings/` (`SettingsNav.tsx`, `TemplatesSection.tsx`, `TemplateExample.tsx`); `features/workspace/` (`ContainerGrid.tsx`, `TemplateInfoDialog.tsx`, `TemplateWarning.tsx`, `useTemplateView.ts`). Test stems: `crates/testkit/tests/suite/t1_templates.rs`, `t2_template_placement.rs`, `t3_template_hand.rs`, `t4_template_retake_shift.rs` (each needs its `mod` line in `suite/main.rs`); UI tests `features/<f>/__tests__/t<n>_<slug>.test.tsx`.

## 5. The placement contract (10G §4 made exact; T2–T4 implement it verbatim)
**P1 Templated session.** The session has an item, a template exists whose `type_key` equals `type_key(item.item_type)` (`type_key` = trimmed, lower-cased), and that template has at least one container. Read at each use. Unassigned is never templated. A session that is not templated behaves exactly as before this feature (P66) and gets no `photo_places` row.

**P2 Shots.** The shots of a session are all its `photos` rows of any classification (a denied photo keeps its place in the count) whose `photo_places.released` is not 1. A shot's key: `at` = `key_at`, else `photos.captured_at`, else `created_at`; `name` = `key_name`, else the `file_key` of the newest `camera_deliveries` row with `final_path = photos.raw_path`, `state = 'done'`, `outcome != 'duplicate'`, else none; counter = `capture_ledger::seq_of(stem_key(name))` (prefix, number), else none. What exists in code (verified): `captured_at` is the matched exposure's `exposed_at` for a direct-camera file (`delivery_for_path`), the file's mtime for Capture One's tether and for imports; the ledger matches a file to the **oldest** expecting exposure (FIFO), so two files that arrive swapped carry each other's times, and only the camera's file counter tells them apart.

**P3 Holds.** One hold per `camera_exposures` row with `session_id` = this session, `status = 'expecting'`, no `camera_deliveries` row of its own with `outcome != 'duplicate'` (an exposure that waits only for its second file, RAW + JPEG, is not a hold), and `now − exposed_at ≤ FIFO_WINDOW_MS` (5 minutes: after that the ledger can no longer match a file to it), at its `exposed_at`.

**P4 Shot order** (`mechanisms::shot_order::order(shots, holds)`, pure):
1. *Slots.* One entry per shot (time `at`) and per hold. Sort by (time, shots before holds, photo id). The sorted places are the slots.
2. *Runs.* Walk the entries that are holds or shots with a counter, in slot order. A run is a maximal sequence where each entry's time is at most `FIFO_WINDOW_MS` after the previous entry's and all its shots have one counter prefix (a shot with another prefix starts a new run). Shots without a counter are in no run and keep their slot.
3. *Inside a run* with m slots and k ≥ 1 shots: sort the shots by counter number (restart rule: when the largest and smallest number differ by more than 5000, add 10000 to every number more than 5000 below the largest; ties by time, then id). Shot 1 takes the run's slot 1; shot j takes slot `min(previous + max(1, c_j − c_(j−1)), m − (k − j))`. The run's remaining slots are holds.
4. The shot index of a photo is its 1-based place in the result, holds counted.
In words: times say how many shots there were and when; among camera files shot within five minutes of each other the counter says which file is which; a missing file keeps the place the counter leaves open for it, else the last place.

**P5 Expected place** (`mechanisms::template::expected`). Index k ≤ n (n containers): Main, in the container whose `shoot_no` is k; its `final_no` is the photo's `slot`. Index k > n: Condition.

**P6 Placing** (`place_arrival`, one transaction, called by `ArrivalService::accept` after a photo was inserted, for captures and imports; an error is logged and the photo stays where P66 put it). The new photo gets a `photo_places` row. Every photo of the session that has a row with `place_by_hand = 0` and is not Deleted is set to its expected place. A photo with no row (it was there before the template, or was moved in) counts as a shot and is treated as placed by hand. Photos placed by hand are never changed. The existing events are emitted after the commit (`photo_changed` per changed photo, `session_changed`); no undo step.

**P7 Orders.** Main positions stay dense (1…m), so export and publish are unchanged: Main photos sorted by (has no slot, slot, placed by hand, previous position). Empty containers are not rows. Condition: positions are kept; the template-placed Condition photos take, in shot order, the positions that group holds (a new one first joins at the end). Two photos may share a slot only when a hand-placed photo sits in a container another shot expects; both are shown under that number.

**P8 Presets.** `Presets` empty (no split, no shape, zoom 0) prescribes nothing. When the template puts a photo with `frame_by_hand = 0` into a place, its composition becomes that place's: the container's presets, or the default composition when the place has none and the photo's composition was set by the template before (origin `user` with `frame_by_hand = 0`; a photo whose composition the template never set is left alone). Values: kind Split when `split`, else Normal; shape = the preset's, else the photo's current shape; orientation Named; turns, flip, straighten kept; rect = `zoomed(default_rect(kind, shape, …), zoom)` (`zoomed`: both sides × (1 − zoom/100), same centre; zoom 0–80); origin `user`. The frame is unknown when a photo has just arrived: then kind, shape and origin are written and `pending_zoom` is stored; `adopt_frame_tx` applies `zoomed` to the first rect it writes and clears it. 10D decides everything after that (re-fit, `needs_review`).

**P9 By hand, differs.** `place_by_hand` is set by classify, reorder and deny (T2) and by a drop (T3); a moved photo loses its row and a copy has none, and no row means placed by hand (T2); `frame_by_hand` is set by a Compose edit (T2). Differs is computed from T3 on. A photo *differs* when it is not Deleted and (it is placed by hand or has no row, and its place is not its expected place) or (`frame_by_hand = 1` and it sits in its expected container and that container has presets). The app never changes a photo placed by hand except through Back to template.

**P10 Retake and release** (T4). A retake takes the replaced photo's key (`key_at`, `key_name`) and the replaced photo is Deleted and `released`. Shift the rest back sets `released` on the session's Deleted photos and re-places the template-placed ones.

**Edges.** Template edited mid-session: nothing moves at once; the next arrival places template-placed photos by the current template; Back to template uses the current template. Item type changed mid-session: the next arrival uses the new type's template, or none. Template deleted: rows stay and are ignored. Photo moved out: it leaves the count; the others move only at the next arrival. Photo moved or copied in: counts by its own key, placed by hand. First shot of a session arriving after the second: the second sits in the first's container until the first arrives (seconds), then both are right. No exposures reported (upload only), or RAW + JPEG with the JPEG first: no hold, order by counter; a swapped pair is corrected when the second file arrives.

## 6. Environment pointers
Commands and caveats: `master_plan.md` §7 and `docs/guide/05_working.md`. While working: `cargo test -p psm-core <module>::` · `cargo test -p psm-testkit --test suite <stem>::` · `cargo test -p psm-desktop` (regenerates `apps/desktop/src/services/bindings.ts` through `export_bindings`; never hand-edit) · `cd apps/desktop && npx vitest run <files>` · `npm run typecheck`. Closing gate (once): `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time, never in the background, never piped. Known intermittent tests: `docs/guide/04_change_impact.md` §Fragile spots (rerun alone before blaming a change). Tests use `TestApp` (`crates/testkit/src/test_app.rs`): `arrival_photo(session, "<camera file name>")` delivers a camera file through the ledger and accepts it; `ledger().exposure_observed(..)` reports a shutter; `store().adopt_frame(..)` makes a frame known; `kill_and_restart()` reopens the data folder.

## 7. Log
- **v1, 2026-11-16 (planner).** Written from 10G. One owner card (drop of a placed photo onto a filled container: swap or delete), built as "swap" on silence; see `handoffs/reviewer/handoff_templates_plan_r1.md`.
