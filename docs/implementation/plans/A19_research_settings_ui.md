---
phase: A19_research_settings_ui
title: Settings → Research — enter the keys, list the websites you trust and order them, choose the model and effort for each stage, set the cost limits, the language, the automatic switch and the quiet period
difficulty: normal
projection: no
depends_on: [A13_research_run_lifecycle]
---
# A19 — Settings → Research (08A §5, §6 Trusted sites, §9; 08B MC-A3 entries, MC-A6, MC-A8; MA7, MA13, MA16)

## Goal
A Research section in Settings backed by thin commands: provider keys into the Keychain (never read back), the
trusted-sites list with add, remove and reorder under MC-A3's rules (the shop's own domain shown first and fixed),
model, effort and image size per stage, `max_cost_per_run` and `max_cost_per_day`, the price table, language,
`auto`, `quiet_seconds`, `record_days`, attribution keys. **Not in this phase:** Export research (A21), default
values for models and prices (A24).

## Read first
`master_plan.md` §6.7, §6.8 · `master_plan_research.md` §3 (keys), §6.6 (accounts, settings keys, commands) · book
`08A` §5, §6 (Trusted sites), §9, `08B` MC-A3 (Trusted entry, Shop hosts), MC-A6 (eligibility re-evaluated on every
save of a research setting or key; `auto` off), MC-A8 (decimal strings, at most 6 fractional digits), MC-A11
(`record_days` minimum 2) · code `apps/desktop/src/features/settings/{SettingsPage.tsx, CameraSection.tsx,
StorageSection.tsx}` (section pattern, secrets entry for the camera login), `apps/desktop/src-tauri/src/commands/
{settings.rs, camera.rs (camera_secrets_set)}`, `crates/infra/src/{settings_file.rs, keychain.rs}`,
`crates/core/src/mechanisms/research/sites.rs`, `crates/core/src/services/research/runs.rs` (`settings_changed`).

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `apps/desktop/src/features/settings/ResearchSection.tsx`,
`apps/desktop/src/features/settings/__tests__/a19_research_settings.test.tsx`,
`crates/testkit/tests/suite/research/a19_research_settings_ui.rs` (+ `mod` line).
Edited: `apps/desktop/src-tauri/src/{commands/research.rs, lib.rs}`, `apps/desktop/src/services/{research.ts,
settings.ts, bindings.ts}`, `apps/desktop/src/features/settings/SettingsPage.tsx`,
`crates/core/src/services/research/mod.rs` (validation at the service), `crates/infra/src/settings_file.rs` (save
validation), own tracker row.

## Core mechanisms
None new.

## Ordered tasks
1. `research_secrets_set({model_key?, search_key?, reader_key?})` writes the given keys through the secrets port;
   `research_secrets_state()` returns only which are set. No command ever returns a key.
2. `research_site_add(text)`, `research_site_remove(entry)`, `research_sites_reorder(entries)`: `TrustedSites` (A04)
   with the shop hosts from `public_listing`; a refusal's text is shown as is.
3. The other fields save through `settings_set` with a `research` patch; the service refuses: a cap or price that is
   not a decimal string with at most 6 fractional digits, `record_days` below 2, a negative `quiet_seconds`, a long
   edge that is not a positive integer.
4. Every successful save or key change calls `settings_changed()` (A13).
5. The section: keys (masked, "set" / "not set"); trusted sites (the shop's domain first, labelled always trusted,
   not removable; the list with drag or up/down reorder and remove; an add field); one row per stage (model,
   effort, image long edge); caps; price rows; language; Automatic research switch; quiet period; record days;
   attribution keys with a warning line under a key that is not one of the shop's field keys.

## Criteria (5 criteria, 11 rows, 3 `[rule]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | secrets commands (suite, `FakeSecrets`) | set a model key | the secrets port holds it under `research-model-key`; `research_secrets_state` says model set, search not set; `settings.json` on disk does not contain the value | MA16 |
| C1(b) | prerequisites | the keys set, a session with a Main photo | the status no longer names `NoModelKey` (eligibility re-evaluated by the save) | MC-A6 |
| C2(a) | `research_site_add` | `https://www.pamono.com/x` | the list ends with `pamono.com`; saved in `research.trusted_sites` | MC-A3 |
| C2(b) | same | `shop.etsy.com` while `etsy.com` is listed; the shop's own domain | refused with `already covered by etsy.com`; refused with `already covered by <shop host>` | MC-A3 |
| C2(c) | reorder, remove | three entries, the third moved first; one removed | the saved order is the new order; a run started afterwards reads pages in it, a run started before keeps its snapshot | MC-A3 |
| C3(a) | save validation | `max_cost_per_run` = `0.50`; `0,5`; `0.0000001`; `record_days` = 1 | saved; refused; refused; refused | MA15, MC-A11 |
| C4(a) | `auto` | switch Automatic research off while a run is queued (`auto`) | that run is `cancelled` | MC-A6 (R-IA25) |
| C5(a) | section (UI test) | load with two entries and the shop domain | the shop domain is first, has no remove control; the two entries follow in order with remove controls | `[rule]` 08A §6 |
| C5(b) | section | add refused by the service | the refusal text is shown; the list is unchanged | `[rule]` 08A §6 |
| C5(c) | section | the key fields | never show a stored key; show "set" after saving; the save call carries the typed value once | MA16 |
| C5(d) | section | attribution key `custom.designer` exists in the definitions, `custom.maker` does not | a warning under `custom.maker` only | `[rule]` master_plan_research §8 |

## Delegated choices
Layout within the settings page's existing pattern · reorder control (drag with dnd-kit, or up/down buttons) · how
price rows are edited (a small table). Refine at prompt time against `claude_design/`.

## Manual try-out
1. `PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev` → Settings → Research.
2. Paste the model key and the visual-search key; they show as "set".
3. Add your nine sites one by one, in your order; try adding `shop.etsy.com` after `etsy.com` and see it refused.
4. Drag one site to the top; reopen Settings and see the order kept.
5. Set the cost per run and per day, the language, the quiet period.

## Notes
- No `implementer: fable`. Keys are typed into the app, never into a file in the repo.
- Review mode: this phase's suite module, `npx vitest run src/features/settings`, typecheck, clippy, fmt.

## Review log
