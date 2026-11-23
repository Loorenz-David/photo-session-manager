# 03 — Rules and principles (never break these)

Each rule names its source. When a rule here and the source disagree, the source wins; fix this file.

## Hard constraints H1–H8 (`docs/intention/book/01_objective_and_constraints.md`; a violation is a blocking B3 finding, `docs/process/flow.md` §5)
| # | Rule | Where it bites in code |
|---|---|---|
| H1 | Nothing is written to an external platform without an explicit, recorded confirmation | every mutating `CommerceAdapter` method takes a `ConfirmedRun`; `publish_confirm` writes `publish_runs`/`publish_items` first |
| H2 | RAW files are never modified, moved or deleted. **Exceptions the owner made:** published, clean items are cleared after retention (`10F`, M41: only after Shopify is verified to hold every published image); a **denied** photo's RAW is deleted when its undo window closes (changelog 46, P66); the owner's **"Delete session"** moves the session's RAWs (only those no other session uses, only inside the capture root) to the macOS **Trash**, never a permanent delete (changelog A17); a photo's RAW (with Capture One's files of it) is **moved into its own session's capture folder** when it is elsewhere (after a move, its undo/redo, Assign product, or "Fix photo files location"), never over an existing file and never when another session uses it (changelog A21); a **failed bring back** deletes only the files that attempt itself downloaded into the session folder (10H §7, MC-H4.3; a brought-back JPEG is a published picture, not a RAW). "Delete" before P66 only hid the photo | `services/storage.rs`, `services/storage/session_delete.rs`, `services/storage/photo_files.rs`, `ports/trash.rs`, `mechanisms/retention.rs`, `photo_denials`; the capture ledger never replaces a file (`camera_files::rename_exclusive`) |
| H3 | No pixel processing except the composition renderer (decode master, turn, flip, straighten, crop, resize, encode once, thumbnails); copying an embedded JPEG out of a RAW is allowed, decoding a RAW is not | `crates/infra/src/imaging/` is the only crate that decodes pixels |
| H4 | Every step is idempotent; resuming after a crash yields the same end state, no duplicate uploads | publish steps, `photo_links` upload states, export staging + swap, ledger `pending`/`done` rows |
| H5 | No UI automation of other apps; Capture One only through its AppleScript dictionary (`docs/reference/capture_one_16.8.6.sdef`) | all Capture One access goes through `CaptureOneScripting` → `jxa/*.js` → `osascript`; `launch()` uses `open -g -a`, `quit` is the dictionary's quit |
| H6 | The core workflow runs with no commerce integration configured; publishing is optional and last | `App.commerce: Option<…>`; header action "Export" without an integration |
| H7 | Device-, app- and platform-specific concepts never appear in core modules or core tables; they live in adapters and link tables | `item_links`, `photo_links`, `attribute_definition_links`; known debts listed in `backlog.md` ("H7 family": `"shopify"` literals, Capture One error-number heuristics, Sony sentences in core) — do not add more |
| H8 | Business logic is independent of Tauri, of any HTTP server and of the UI; commands and events are thin | `psm-core` has no Tauri dependency; commands call services inside `spawn_blocking`; the phone remote (post-MVP) reuses the services |

## 10H invariants M-H1–M-H5 (book `10H` §9; `docs/implementation/master_plan_archive.md` §5)
| # | Rule | Where it bites in code |
|---|---|---|
| M-H1 | No shop write outside a confirmed Publish: bring back, refresh, covers and search pictures only read | `services/remote.rs`, `services/bring_back.rs` call only read methods of `CommerceAdapter` (none takes a `ConfirmedRun`) |
| M-H2 | Bring back imports all of an item's pictures or none, and never deletes anything in the shop or in Capture One's catalogue | `services/bring_back.rs`: `Store::bring_back` (one transaction) runs only after every file is on disk; a failure removes only that attempt's files; the editor is only asked `inspect_document` and `sync_folder` |
| M-H3 | A publish after bring back uploads only photos whose composition, rotation or Capture One edit changed since, plus new ones | the links `Store::bring_back` writes (`ready`, revision 0, `composition_rev` 0, the stored `retouch_gen`) feed `mechanisms/publish.rs::replacement_candidates` and Upload's `ready` skip; no publish code knows about bring back |
| M-H4 | The screen is never given a local image path whose file is missing | `RowPictures::row_picture`; `Store::bring_back` nulls the retired rows' preview paths |
| M-H1 (H4) | A details-only run never touches media | `publish_items.details_only` (decided at confirmation, read on every resume): `prepare`/`upload` do nothing, `update` skips `update_gallery` and every media-list write (`services/publish.rs`) |
| M-H5 | "Delete session" never touches the shop or Pricing | `services/storage/session_delete.rs` (no adapter call); `RemoteItems::forget_item` removes only cache folders |

## Conversations rules CR1–CR6 (book `09B` §10 rules 1–6; `docs/implementation/master_plan_conversations.md` §3)
| # | Rule | Where it bites in code |
|---|---|---|
| CR1 | Session Manager writes to Pricing only the photographer's replies and "Mark answered" | the port has two writing methods (`post_message`, `mark_answered`); the adapter issues only `POST …/messages` and `PATCH … {"status":"answered"}`; `post_message` is called only by the flush inside `tick`, `mark_answered` only by `ConversationsService::mark_answered`; `reply`, `retry`, `discard`, `view` and `attachable` never call the provider |
| CR2 | The API key lives only in the Keychain; never printed, logged, exported, or sent anywhere but the configured server | `KEY_ACCOUNT` through `SecretStore`; `RedactingSecrets` registers it with `add_secret`; DTOs carry `key_saved` only; `PricingRequest`'s `Debug` hides the bearer; `FakePricing` records `authorized`, never the value |
| CR3 | Deleting a session never touches Pricing nor the local copy (conversations are keyed by SKU) | migration 019 has no session/item/photo column; `session_delete` makes no provider call |
| CR4 | Opening a conversation never moves the capture folder (C4) | `features/conversations/visit.ts`: route and visit are one state changed only by `visitStep`; `App.tsx` hands `useCaptureTarget` (the only caller of `activateSession` / `deactivateSession`) `captureTarget(state)` = the visit's capture while visiting; no backend read of a visit (`session_get`, `conversation_get`, `conversation_attachable`) touches the active session (`c4_conversations_visit::c4_c8*`) |
| CR5 | Pricing down or the key missing never blocks or slows another screen | the pull runs on `conversations_background`; `connection()` never takes the lane; commands run in `spawn_blocking`; no request without `ready` (MC-C1.1) |
| CR6 | No duplicate replies: every send carries its `clientMutationId`, every retry reuses it (C3) | the composer makes the id once per press (`crypto.randomUUID()`); it is the `conversation_outbox` primary key and nothing in the backend makes another; `checked_reply` (1–80 characters) and `reply` (same id → the stored row, nothing written); the flush sends exactly the stored id; a `failed` row is sent again only through `retry`, with the same id, never by a `tick` |
| — | No test, session or tool ever calls Pricing or Shopify; sessions never launch the built app in C1–C4 | the adapter is tested over a recording `PricingHttp` and `FakePricing` |

## Price labels invariants M-L1–M-L6 (book `09C` §9; `docs/implementation/master_plan_labels.md` §5; as far as L1 builds them)
| # | Invariant | Where it bites in code |
|---|---|---|
| M-L1 | Label state is never stored: markers come from the provider's latest answer or event | `LabelCopy` (memory) in `LabelsService`; the only stored fact is `conversation_sync.labels_access` (whether the key may read labels); `l1_labels_markers::l1_c13f` lists every `label` column |
| M-L2 | Only a run changes anything at the provider | `LabelsProvider::run` is the one writing method and the adapter's only `POST …/labels/runs`; in L1 nothing outside the adapter's tests calls `run`, `plan`, `preview` or `download` |
| M-L3 | One key per chunk, reused on every retry | (L3) `run(store, ids, key)` sends the key it is given and nothing else |
| M-L4 | A run with a failure is never reported fully generated | (L3) `RunItem.rendered` / `result` are read per item; `lifecycleResult` is read tolerantly so a run that marked products is never a `BadReply` |
| M-L5 | Labels never write to Shopify; the key is never logged | label calls use the conversations key closure and bearer header (CR2); log lines `labels.access`, `labels.status asked= answered=`, `labels.events changed=` carry no key, URL query, SKU or title |
| M-L6 | Without a key, a matched store and `labels:read`, no label call is made | `LabelsService::allowed_store` (connection `Ready` with a store, access not `denied`); `connected()` is the only call that ignores a remembered `denied` (one templates read per Test connection / Save key) |

## Public repository: what must never be committed (`CLAUDE.md`; master plan §3 "Camera hard rules" and "Secrets"; `native/camera-sidecar/README.md` §Never commit; `.gitignore`)
- Any Sony SDK file: headers, `*.dylib`, `CrAdapter/`, the `CrSDK_*` archive or folder, the lab's
  hand-written `sdk_shim` headers. The SDK lives only on the build machine, outside the repo.
- `.env` files, credentials, tokens, Keychain contents, the Shopify client id/secret, the shop
  domain, IP addresses, camera serials/ids, SSH/camera fingerprints (`*.fingerprint`).
- Runtime data: databases, logs, previews, photos (`data/`, `*.db`, `*.log`).
- Generated artefacts: `apps/desktop/src-tauri/binaries/` (the sidecar), `target/`, `node_modules`,
  `.pipeline/`, `vendor/`, `CrSDK*`.
- Secrets never appear in logs, fixtures, handoffs or narratives either (master plan §3; `12`
  rule 8). Register every new secret with `psm_infra::logging::add_secret` so the log redacts it.
  Use placeholders such as `<shop>.myshopify.com` in docs.

## Camera hard rules (owner 2026-11-06; master plan §3; book `10E`; narratives P52–P57)
- Never change the camera's network or FTP settings; the app only reads/verifies the store
  destination the sidecar sets for a session (`10E` MC-31).
- Never delete a file on the camera.
- Never kill `ptpcamerad` or any other process. `infra/processes.rs` is read-only by design (M39).
- Capture One is quit only by the take-back sequence (`direct/take_back.rs`: ask it to quit through
  the dictionary, wait, never kill), and only when the preconditions hold. Nothing else quits it.
- Live view is out of scope.
- The automated suite never needs the camera, Capture One, Shopify or the network (flow §6A;
  master plan §3 "Hardware"); the sidecar in tests is `testkit/src/bin/fake_sidecar.rs`.
- No `implementer: fable` on camera phases (owner).

## Research hard rules (book `08A` §2 HA1–HA6, same standing as H1–H8; `08B` MC-A1…MC-A12 with changelog 12–18; `docs/implementation/master_plan_research.md` §3; `docs/process/flow.md` §10)
| Rule | Source | Where it bites in code |
|---|---|---|
| Research never writes a field holding a `human` value or an `ai` + `accepted` value, and never suggests again a value the person cleared | HA1, MC-A1 | `crates/research/src/writes.rs` (the one door), `rules/write_table.rs::decide_write` |
| Never-written fields are never asked for, suggested or paid for: the Product step/group, media-list, reference, rich-text, money and rating types, non-writable fields, the product title; **and every key not in `research.fill_keys`** (empty by default) | MC-A1; 08B changelog 17, 18; R13 plan "supersedes" | `rules/write_table.rs::never_written`, `rules/confidence.rs::field_class` |
| A designer, maker or other attribution value is never above `low` on visual similarity alone; for an attribution field **no page of the owner's own shop is evidence** at any level | HA2, MC-A4; R11 plan "supersedes" (08B changelog 12, 14) | `rules/confidence.rs::decide_attribution`; `attribution_keys`, `extra_shop_hosts` |
| No RAW and no `deleted` photo ever leaves the Mac; every image sent is recorded against a run **before** the request leaves | HA3, MC-A5 | `images.rs` (`ResearchImage` has no public constructor), `gateway.rs` (`research_outbound` rows) |
| Without a key, a network or an integration every other part of the app works as before; research says why it does not run | HA4, MC-A6 | `App.research: Option<…>`, commands answer `research_unavailable`, `rules/run_rules.rs::first_failing` |
| No model name, vendor name or reseller domain as a literal in `crates/core` (or a core table); literals live in `crates/research/src/{adapters, config.rs}` and the tool | HA5 (H7); research master plan §3 | every provider/model/effort string is opaque in `ports.rs` |
| A run never spends more than its cap; a day limit stops new calls. Money is integer micro-USD, never floating point | HA6, MC-A8 | `rules/cost.rs` (`next_call`, `day_spend`), `gateway.rs`; settings `max_cost_per_run`, `max_cost_per_day` |
| An unchecked suggestion reaches the shop only at `medium` or `high` confidence and only inside a confirmed publish (H1); a `low` one never; a conflict holds its item at `held` until resolved, without blocking Confirm or other items | `08A` §7, MC-A10 | `crates/core/src/mechanisms/publish_partition.rs`, `services/publish.rs` |
| Research writes are not undo steps; the person's actions on them are, and each writes a feedback row | MC-A12 (05A MC-30) | `services/attributes.rs`, `services/history.rs`, `research_feedback` |
| Runs are created only by "Arranged — next" (and the publish preview's marking) and by Research / Run again; never by a photo change, a settings save or entering keys. A19: arranging starts research at most once per session (only while no run is queued, running, finished or stopped at the cap; a failed or cancelled one may be retried) | 08B changelog 7, 13; R11 plan "supersedes"; changelog A19 | `runs.rs` (`arranged`, `run_again`; `session_changed` creates nothing) |
| App keys live only in the Keychain (through `ports::secrets`); the tool's keys only in the owner's env file outside the repo, read from the environment. No key, and no URL carrying one, in a log, fixture, evidence doc, handoff, recording, export or commit; recordings and exports are scanned before they are written (fail closed) | research master plan §3 "Keys"; MC-A9, MC-A11 | `rules/calls.rs::redact`, `gateway.rs`, `export.rs`; tool `recording.rs` |
| The search window's page webview (A12) is untrusted: it may load only public http/https pages in any frame (never the app's origins, localhost, the local network, `file:`/`asset:`/`ipc:`), no capability names a remote URL, so a page can call no command; the app's page only *asks* it what is selected and checks the answer. A field write or an AI call happens only from a click in the app's own page; Ask AI only suggests (A12) or answers in words (A18, nothing written: a value it found is written only when the owner taps it, as a person's edit). Menu zoom keys reach the page only through the app (A18 `search_zoom`). The script every page frame runs (A14 `page_popups.js`) may only turn "open in a new window" into a navigation of the page itself: it reads nothing and calls nothing in the app | owner corrections A12, A14 (changelog A12, A14) | `src-tauri/src/search_window.rs` (`allowed_navigation`, `parse_selection_report`, `new_window`, `popup_script`), `src-tauri/src/page_popups.js`, `capabilities/search.json`, `commands/search.rs` |
| No suite calls OpenAI, a search vendor, a listing site or the shop. Real calls happen only from the tool, run by the orchestrator or owner, under a `--ceiling-usd` the plan names | research master plan §3 "Rule 9", "Spending ceilings" | adapters are tested over `fakes::FakeResearchHttp`; the tool tests over their `Stub` |
| A fixture whose shape a provider owns is a sanitised capture written by the tool (`crates/research/fixtures/<provider>/`), never typed by hand | research master plan §3 "Rule 17" | `crates/research/fixtures/README.md` |
| Isolation: the research crate never depends on `psm-integrations` or the desktop crate; app code uses it through `ResearchApi` and the wiring in `crates/integrations/src/lib.rs` | flow §10; research master plan §3 "Isolation" | `crates/research/Cargo.toml` |
| The research schema is never a numbered migration: `SqliteStore::migrations()` is not edited for research. A later numbered migration that recreates `attribute_values`, `publish_items` or `sessions` must carry the research columns | research master plan §0, §6.4 | `crates/infra/src/sqlite/ensure.rs`, `crates/research/src/schema.rs` |
| Research tests: rules as inline unit tests in `crates/research/src/rules/`; engine behaviour in `crates/research/tests/engine/<stem>.rs` and tool behaviour in `crates/research-tool/tests/tool/<stem>.rs` (**both `autotests = false`: a file needs its `mod` line in that folder's `main.rs`**); app-seam behaviour in `crates/testkit/tests/suite/research/<stem>.rs` (listed in `research/mod.rs`). Tests are named after their criterion row (`a07_c1a_<slug>`) | research master plan §0, §6.1; flow §10 | — |
| Research track documents stay apart: `STATUS_research.md`, `backlog_research/<milestone>.md`, `phases_research.txt`, the Tracker line in each `plans/R*.md`. No `implementer: fable`; projection only where a plan says so | research master plan §0, §3; flow §10 | — |

## Writer hard rules (book `08C` §2 HW1–HW7, same standing as H1–H8; `docs/implementation/master_plan_writer.md` §5 WC1–WC8)
| Rule | Source | Where it bites in code |
|---|---|---|
| The writer never overwrites a `human` or accepted `ai` text and never re-suggests a text the owner cleared or kept the previous over; it writes only its requestable text keys (research still refuses all of them) | HW1, WC4 | `crates/research/src/writer/writes.rs::write_text` over `rules/write_table.rs::decide_write` |
| `items.title` changes only by the owner's action: a title suggestion is an attribute row until accepted; the publish partition never freezes a `product.title` row (the title sent is `items.title` frozen at confirmation) | WC4.7, WC4.8 | `crates/core/src/services/attributes.rs` (`accept`, `accept_all`, `keep_previous`, `clear_title`, `write_title`), `mechanisms/publish_partition.rs::partition` |
| Every claim comes from a `fact` field or the note; the post-check flags names, years and models absent from them | HW2, WC2, WC5 | `writer/input.rs::build`, `writer/postcheck.rs::check` |
| What the owner decided is never questioned again; a dismissed or answered finding never comes back | HW3, WC6 | `writer/safeguard.rs::filter`, `writer/note.rs::reviewed` |
| A writer request carries no image | HW4, WC7 | `writer/run.rs::call` (`images` empty) |
| Texts never mention AI, the writing process or the shop's values | HW5 | the shipped instruction (`writer/defaults.rs`) |
| Writer calls are recorded, costed and day-capped like research calls, and exported | HW6, WC7 | `gateway.rs` with `Stage::Write`, `export.rs` |
| Without a key, network or integration the writer says why and everything else works; an automatic request ("Next to review", "All checked") never shows a toast and never holds up the owner | HW7, WC3, A19 | `writer/run.rs::writer_block`, commands answer `research_unavailable`, `requestTexts` in `features/sessions/reviewQueue.ts` |
| At most one `write` run queued per session; none while research or a look runs on the session; automatic origins create at most one per session | WC3 | `writer/run.rs::request_decision` |

## Stack and boundaries (`CLAUDE.md`; book `11`; master plan §6.1)
- Rust is the backend. **No Python in the product** (`11` §Processes: a Python sidecar only for a
  demonstrated need; none exists). `python3` is used only by the pipeline runner's template rendering
  (`tools/pipeline/run_phase.sh`), never by a build or by the app.
- `psm-core` depends on nothing above it; services depend on ports only; `infra` and `integrations`
  implement ports; `psm-desktop` commands are thin.
- No Tauri `invoke` from components: `services/*.ts` over the generated `bindings.ts`, which is
  never hand-edited (master plan §6.1, §6.7).
- Capture One Pro 16.8.6.25 is the only tested version; property names come from the sdef and the
  VERIFY items in `docs/intention/evidence/capture_one_16.8.6.md`.

## Persistence rules (master plan §3, §6.5, §7.3)
- Migrations are never rewritten once a checkpoint commit contains them: add the next numbered file
  and its `M::up` line (`crates/infra/src/sqlite/store.rs`).
- Tests open SQLite only inside a `TempDir`; the configured database is never touched by tests.
- `settings.json` stays user configuration; app state goes to the `settings` table. Secrets other
  than the Shopify client pair are Keychain only (`12`; `10E` R-I21).
- Derived values (session status, counts, fill counts, the publish summary) are never stored
  except as frozen into a confirmed run (`05` §Facts vs derived).

## The naming registry is binding (master plan §6)
Crate, module, table, column, enum, error code, health code, command, event, settings key and test
names are those of master plan §6.4–§6.11. A new name is added to the registry in the same change
(fold-back rule, master plan §2: semantics → book + changelog; skeleton/environment → master plan;
phase-local → plan file). Known registry gaps are recorded in `backlog.md` ("book and registry drift").

## Tests (`docs/process/flow.md` §4, §4A; master plan §3, §6.9)
- One behaviour test per criterion row, at the boundary the row names (service call, command
  handler, produced file, stored state, calls a fake adapter received), asserting the exact outcome.
- Unit tests only for core mechanisms a plan lists by name, inline in
  `crates/core/src/mechanisms/<m>.rs`, named `mc<n>_<slug>`.
- Never: internal call counts, private helpers, generated SQL, mocks of internal collaborators,
  snapshots of internals, "tests for confidence". The over-engineering test: a test that must change
  for a reason other than the behaviour changing is the defect.
- No mutation regime; reviewers never edit production code to see a test fail.
- Behaviour tests live in `crates/testkit/tests/suite/<phase>_<slug>.rs` **and must be listed as
  `mod <stem>;` in `crates/testkit/tests/suite/main.rs`** (`autotests = false`): an unlisted file never
  runs and nothing warns (P70). Tests share one process: no environment variables, current
  directory, global subscriber, fixed ports or fixed temp paths; a test that needs process-wide state
  wraps its body in `psm_testkit::isolation::in_own_process`.
- Test runs: while working, only the phase module (`cargo test -p psm-testkit --test suite <stem>::`),
  the changed module's unit tests, the touched UI test files; the full gate **once** at the end.
  **One cargo command at a time, never two in parallel, never piped into `head`** (a cut pipe leaves
  cargo holding the build lock). The reviewer by default runs no full suite.
- `FakeEditing` is the real Capture One adapter over `FakeCaptureOneScripting`; never a second
  `EditingProvider` implementation (master plan §7.5).

## Commits and pushes (`docs/process/flow.md` §8; STATUS 2026-11-11 11:00 and 11:40)
- Implementer and fix sessions commit `CHECKPOINT (not approved):` of their perimeter only; the
  runner commits prompts, handoffs, narratives, state and the `APPROVED <phase>` commit.
- **Commit only named paths** (`git commit -- <paths>`); never `git add -A` or `git commit -a` while
  a phase runs (two orchestrator commits swept in-progress code and do not build).
- Nobody pushes but the orchestrator, at the end of a run. Never push from a phase session.
- Phase perimeter: touch only the files the plan lists (plus its named test files); a new file is
  declared under **Deviations** in the handoff (master plan §3).
- No questions during a phase: an open choice is the smallest option consistent with the cited
  chapter, recorded under **Assumptions**; `BLOCKED` only for a plan/book contradiction or an
  impossible task (flow §6).

## Design and UX rules (`docs/intention/evidence/design_handoff.md`; `claude_design/`)
Field kinds, timings (undo toast 4.5 s, crossfade 700 ms), keyboard map, component set and tokens
come from the handoff; `styles/tokens.css` is copied from `claude_design/tokens/tokens.css`. Owner
2026-11-04/05: frontend implementation is not run on Fable; the Tools panel is a left collapsible
panel, the browser stays on the right (`10D`).
