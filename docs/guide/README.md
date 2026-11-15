# Session Manager — agent guide (START HERE)

Written 2026-11-15 at P70; the research assistant was added the same day, after R13 (research joined
`main` as PR #1). This folder re-orients a new agent without a repo-wide search. It
summarises and points; it never replaces the authorities below. Keep it current (`05_working.md`
§Keeping this guide current).

## What the app is, in six lines
1. A macOS desktop app (Tauri shell, **Rust backend**, React UI) for one photographer who lists
   second-hand items on Shopify: shoot an item, sort and frame the photos, fill the product form,
   publish. Two Macs: a studio/dev Mac and a work Mac; no server, no accounts.
2. A **Sony camera** delivers RAWs over Wi-Fi FTP or USB-C through a C++ helper process (the
   *camera sidecar*, Sony SDK never in the repo) or through Capture One's tether.
3. **Capture One** does all image development; the app drives it only through its AppleScript
   dictionary (`osascript`, JXA). The app's own pixel work is limited to framing (crop, turn,
   straighten, resize, JPEG encode) from Capture One's master.
4. **Shopify** (Admin GraphQL, custom app) receives product, gallery, metafields, damage report,
   inventory. Nothing is written without a recorded confirmation (H1).
5. Everything platform-specific lives behind ports in adapters; core knows Item, Session, Photo,
   Attribute only (H7, H8).
6. A **research assistant** (its own crate, `crates/research`) looks at an arranged session's photos,
   finds and reads matching listings on the web, and writes **suggested** form values with a
   confidence and sources, only into the fields the person chose; the person checks them (HA1–HA6).

## Authority order
| Question | Authority |
|---|---|
| What must the app do (semantics, contracts MC-n, constraints H1–H8, measurements M-n) | `docs/intention/intention.md` → `docs/intention/book/*.md` |
| Names, tables, commands, settings keys, environment, exact commands | `docs/implementation/master_plan.md` (§6 registry, §7 environment) |
| Process (roles, tests, verdicts, commits) | `docs/process/flow.md` (research phases: §10 first) |
| Research: what it must do (HA1–HA6, stages, contracts MC-A1…MC-A12, measurements MA-n) | `docs/intention/book/08A_research_agent.md`, `08B_research_contracts.md`. **Later wins:** 08B's changelog entries 12–18 and the sections "What this plan supersedes in the book" of the plans `R11`, `R12`, `R13` override the older chapter text until the chapters are rewritten (08B changelog 14, 18) |
| Research: names, tables, settings keys, seams, commands, test commands | `docs/implementation/master_plan_research.md` (§6 registry, §7 environment); it wins over `master_plan.md` on a research name |
| Research: what was built, measured and decided, newest state | `docs/implementation/STATUS_research.md`; `narratives/R*.md`; `backlog_research/R*.md`; measurements in `docs/intention/evidence/research_providers.md` |
| What exists and how it behaves today | **the code**; then `docs/implementation/narratives/P*.md` (plain-language result per phase), `STATUS.md`, `backlog.md` |
| Orientation | this guide |

Book > plan > guide for semantics. Code is the truth for what exists. If this guide disagrees with
either, **the guide is wrong: fix it in the same change.** The book is itself behind the code in a
few places (listed in `01_product.md` §Book pending updates).

## Reading order by task
| Task | Read, in order |
|---|---|
| Any task | this file; `CLAUDE.md`; `03_rules_and_principles.md`; `04_change_impact.md` rows for what you touch |
| Debug a publish failure | `05_working.md` §Debugging recipes; `02_architecture.md` §Publish; `crates/core/src/services/publish.rs`; `crates/integrations/src/commerce/shopify/adapter.rs`; book `07`, `07A`, `09A`; narratives P64, P65, P69 |
| Change the UI | `02_architecture.md` §Frontend; `docs/intention/evidence/design_handoff.md`; the feature folder under `apps/desktop/src/features/`; `04_change_impact.md` rows "Tauri command/DTO", "AppEvent", "settings field" |
| Touch the camera (sidecar, FTP, ledger, take-back) | `02_architecture.md` §Camera runtime and §Take-back; book `10E` (MC-31…MC-34); `native/camera-sidecar/README.md`; `crates/integrations/src/camera/`; narratives P53–P57, P60, P63, P68; `03_rules_and_principles.md` §Camera hard rules |
| Touch Capture One | `02_architecture.md` §Arrival, §Compose/render; book `10A`, `10C`, `10D`; `docs/intention/evidence/capture_one_16.8.6.md`; `crates/integrations/src/editing/capture_one/`; `docs/reference/capture_one_16.8.6.sdef` for property names |
| Add a DB field or table | `02_architecture.md` §Persistence; `04_change_impact.md` row "DB change"; master plan §6.5; `crates/infra/src/sqlite/` |
| Run a pipeline phase | `docs/process/flow.md`; `docs/process/orchestrator.md`; `05_working.md` §Pipeline in ten lines; `docs/implementation/STATUS.md` (top entries) |
| Ship a build to the work Mac | `05_working.md` §Release build with the SDK sidecar |
| Touch research (engine, rules, adapters, its settings or UI) | `01_product.md` §Research assistant; `02_architecture.md` §Research; `03_rules_and_principles.md` §Research hard rules; `04_change_impact.md` §Research; book `08A`, then the `08B` contract you touch **and** its changelog 12–18; the "supersedes" sections of plans `R11`–`R13`; `master_plan_research.md` §5 Seams, §6; `crates/research/src/`; narratives R11–R13 |
| Run a research measurement or replay a stage | `05_working.md` §Research tool; `crates/research-tool/src/lib.rs` (`USAGE`); `master_plan_research.md` §3 (Rule 9, Keys, Spending ceilings); `docs/intention/evidence/research_providers.md` (how the earlier measurements were run and what they found) |
| Run a research phase | `docs/process/flow.md` §10; `master_plan_research.md` §0, §5 Driver loop, §7.2; `docs/implementation/STATUS_research.md`; `docs/implementation/phases_research.txt`; the plan `docs/implementation/plans/R<n>_<slug>.md` (its Tracker line and Closing gate); `05_working.md` §Research phases |

## Repo map (one screen)
```
CLAUDE.md                         build decisions, auto-loaded by Claude Code
Cargo.toml, .cargo/config.toml    workspace (edition 2024); target dir = apps/desktop/src-tauri/target
crates/core/        psm-core      domain, mechanisms (pure rules), ports (traits), services, events, errors. No Tauri/SQL/HTTP/osascript.
crates/infra/       psm-infra     SQLite store + migrations/, settings.json, paths, spool, nudge, watcher, osascript runner,
                                  jobs runner, logging (redacting), backup, imaging (the only pixel code), media caches,
                                  FTP receiver, Keychain, process list, sleep assertion, disk/net helpers
crates/integrations/ psm-integrations  App wiring (lib.rs); editing/capture_one (adapter, JXA scripts, hooks, gates, follow),
                                  camera (runtime, sidecar supervisor+protocol, direct adapter, take-back, support packages),
                                  commerce/shopify (adapter, graphql documents, auth, type map, fixtures), render_output job
crates/research/    psm-research  the research engine, isolated: domain, config, ports, rules/ (pure), stages, decide, gateway, store,
                                  adapters/ (OpenAI, SerpApi Lens, page reader), fakes, fixtures/ (sanitised captures); own tests in
                                  tests/engine/. App code reaches it through api.rs (ResearchApi) and the wiring in crates/integrations/src/lib.rs
crates/research-tool/ psm-research-tool  dev-Mac CLI, bin `psm-research`: probes, benchmarks, accept, score-accept, report, replay;
                                  tests in tests/tool/. Nothing in the app depends on it
crates/testkit/     psm-testkit   TestApp harness, fakes (Capture One, commerce, transport, secrets, processes, sleep, direct camera),
                                  fake_sidecar binary, fixtures; tests/suite/ = ALL behaviour tests (one program, one module per phase)
apps/desktop/src-tauri/           Tauri app psm-desktop: commands/, state.rs, lib.rs (specta builder, event bridge, run()), build.rs
                                  (builds the stub sidecar if binaries/ is empty), tauri.conf.json, Info.plist, binaries/ (gitignored)
apps/desktop/src/                 React + TS: services/ (bindings.ts GENERATED + wrappers), features/<feature>/, components/ui (shadcn),
                                  lib/ (routes, keyboard, pending), styles/
native/camera-sidecar/            C++ sidecar (CMake): SDK build on the build machine, stub build everywhere; manifest/sony-<ver>.json
tools/sidecar/                    build.sh (--stub | --sdk <root> [--test]), manifest.sh
tools/pipeline/                   run_phase.sh (the runner), templates/, wrappers/, timeline.sh
docs/intention/                   intention.md (map) + book/ (chapters) + evidence/ (source facts)
docs/implementation/              master_plan.md, plans/, narratives/, backlog.md, STATUS.md, handoffs/, prompts/, state/, phases.txt
                                  research track: master_plan_research.md, phases_research.txt (R1–R13), plans/R*.md, narratives/R*.md,
                                  backlog_research/, STATUS_research.md, research_row_mapping.md (where each criterion row went)
docs/test-items/                  research test pieces + answer key, accept_settings_owner.json (the owner's research set-up for `accept`)
docs/process/                     flow.md (process authority), orchestrator.md, prompts/
docs/reference/                   capture_one_16.8.6.sdef
docs/guide/                       this guide
claude_design/                    design handoff (read-only reference)
```
Runtime data never lives in the repo: `~/Library/Application Support/com.loorenz.sessionmanager/`
(`PSM_DATA_DIR` overrides it). Layout in `02_architecture.md` §Persistence.

## The six files
| File | Read it for |
|---|---|
| `01_product.md` | who uses it, the end-to-end workflow, vocabulary and states, standing decisions, where the book lags the code |
| `02_architecture.md` | layers, crates and modules, the runtime flows step by step, persistence, events, bindings, frontend, logging |
| `03_rules_and_principles.md` | every hard rule with its source |
| `04_change_impact.md` | if you change X you must also change Y, otherwise Z breaks; fragile spots and known flaky tests |
| `05_working.md` | run, test, lint, logs, debugging recipes, release build, pipeline, keeping the guide current |
