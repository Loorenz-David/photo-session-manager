---
plan: R13_research_chosen_fields
role: implement
round: 1
state: DONE
date: 2026-11-15
actor: codex
---

# What was built

Research now suggests values only for fields ticked under **Settings → Research → Fields research fills**. The list starts empty, so research waits until a field is chosen. An unticked field is left out of model requests and writes; unticking all condition fields also removes the Condition model call. Designer and maker and detailed-condition settings control how chosen fields are decided, and the screen warns when either modifier points outside the list. A detailed-condition grade follows the worst condition level that has a value, with its reason counting empty levels. The measurement tool's owner settings now carry the owner's 22 chosen keys. The four-piece live check is assigned to the orchestrator by the plan and was not run in this session.

# Tests → criterion rows

| Rows | Boundary tests |
|---|---|
| A27·C1(a–c) | `rules::confidence::tests::a27_c1a`, `a27_c1b`, `a27_c1c` |
| A27·C2(a–d) | `chosen_fields::a27_c2a`, `a27_c2b`, `a27_c2c`, `a27_c2d` |
| A27·C3(a–c) | `rules::run_rules::tests::a27_c3a`, `a27_c3b`, `a27_c3c` |
| A27·C3(d–e) | `chosen_fields::a27_c3d`, `a27_c3e` |
| A27·C4(a–c) | `chosen_fields::a27_c4a`; `defaults::a27_c4b`, `a27_c4c` |
| A27·C5(a–b) | `chosen_fields::a27_c5a`, `a27_c5b` |
| A27·C6(a) | `u4_accept::a27_c6a` |
| A27·C7(a–b) | desktop `a27_c7a`; settings UI `a27_c7b` |
| A27·C8(a–f) | `rules::derived_condition::tests::a27_c8a` through `a27_c8f` |
| A27·C9(a–d) | `derived::a27_c9a` through `a27_c9d` |
| A27·C10(a–b), C11(a), C12(a–c) | `r13_research_fills.test.tsx`: `a27_c10a`, `a27_c10b`, `a27_c11a`, `a27_c12a`, `a27_c12b`, `a27_c12c` |
| A27·C13(a) | `r11_research_pickers.test.tsx`: `a27_c13a` |
| A27·C14(a–b) | `u4_accept::a27_c14a`, `a27_c14b` |
| A27·C15(a) | Manual check assigned to the orchestrator; open. |

Existing tests that pinned the retired exclusion list or incomplete-level rule were replaced or amended as the plan's pin table directs. Total new/amended coverage for 39 automated rows; the one manual row is open.

# Assumptions

- Stage 6 is omitted entirely when no condition field is chosen; no empty stage output or looked step is recorded.
- A saved field of a type research cannot fill remains visible in the picker with an explanation so it can be unticked.
- A settings save serializes the complete research slot after merging, which removes retired keys while preserving supported settings.
- The test harness chooses the known keys of its fixture layouts; list-specific tests set their own list.

# Deviations

- `crates/integrations/src/lib.rs` is outside U1's named perimeter. The app's deep settings merge otherwise preserved `excluded_keys` in the saved file, contrary to U1 task 1. The normalization is scoped to saves containing a research patch, and the desktop command test verifies the saved key is gone.
- The desktop `settings_set` command now performs its local save directly inside its async handler. Its previous concrete `AppHandle` argument cannot be invoked by Tauri's mock runtime, so this keeps A27·C7(a) at the real command boundary. It may occupy the command handler during a settings save.
- U1's new UI file was created in U3, where its seven rendering rows were implemented. The U1 picker row is in the existing picker test file.
- U4 task 3 and A27·C15(a) are explicitly the orchestrator's work under the plan, with a USD 2 ceiling. No network or live provider call was made here.

# Files changed

- Research engine: `crates/research/src/{api_impl.rs,config.rs,decide.rs,domain.rs,rules/confidence.rs,rules/derived_condition.rs,rules/run_rules.rs,runs.rs,stages.rs}`; `crates/research/tests/engine/{chosen_fields.rs,defaults.rs,derived.rs,harness.rs,main.rs,settings_r11.rs,stages.rs}`.
- Tool and fixtures: `crates/research-tool/src/accept.rs`; `crates/research-tool/tests/tool/{u3_report_and_replay.rs,u4_accept.rs}`; `crates/testkit/src/test_app.rs`; `docs/test-items/accept_settings_owner.json`.
- App seam and UI: `crates/integrations/src/lib.rs`; `apps/desktop/src-tauri/src/{commands/settings.rs,lib.rs}`; `apps/desktop/src/services/bindings.ts`; `apps/desktop/src/features/settings/{ResearchSection.tsx,ShopFieldPicker.tsx}`; `apps/desktop/src/features/settings/__tests__/{r6_research_settings.test.tsx,r7_research_export.test.tsx,r11_research_pickers.test.tsx,r13_research_fills.test.tsx}`.
- Coordination: `docs/implementation/master_plan_research.md`; `docs/implementation/plans/R13_research_chosen_fields.md`; this handoff.

# Checkpoint SHA

1. U1 `e80ce98` — chosen list, engine and app seam.
2. U2 `f877640` — detailed condition from filled levels.
3. U3 `4d398b0` — Settings explanation and warnings.
4. U4 `f65c0bc` — owner settings and tool checks.

The closing corrections and coordination files are in the checkpoint commit carrying this handoff.

# Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS; app suite 889, research engine 233, tool 30; other crate and doc tests green. Run once at the closing gate. |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm run typecheck` | PASS |
| `cd apps/desktop && npm test` | PASS; 81 files, 515 tests. |

Per-unit commands were run before each unit checkpoint. During development the targeted research, tool, app-command and settings UI tests also passed after each correction. No pre-existing failing test remains.

# Manual try-out

1. On scratch data, start the app with `cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`, with a shop and research keys configured.
2. Open an item's research panel before choosing fields. It asks for fields under Settings → Research; pressing **Arranged — next** starts no run.
3. In **Settings → Research → Fields research fills**, tick Designer, Manufacturer, wood and other facts you want, the condition levels, and Detailed condition. Save. Untick Designer briefly to see its warning, then tick it again.
4. Shoot and sort a piece with main and condition photos, press **Arranged — next**, and inspect the form. Only ticked fields receive suggestions; the detailed-condition reason names the worst filled level and counts empty levels.
5. Add a field to the list later. It does not start a run by itself; press **Run research** on that item.

# ⚠ OWNER DECISIONS REQUIRED (0)

None. The orchestrator still owns the four-piece check in U4 task 3 and its evidence entry.
