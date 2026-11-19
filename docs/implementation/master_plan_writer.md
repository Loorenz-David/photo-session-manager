# Publication writer — master plan (v1, 2026-11-19)

Short on purpose. It adds to `docs/implementation/master_plan.md` (roles, standing rules §3, environment §7) and
`docs/implementation/master_plan_research.md` (research registry §6, storage §6.4, environment §7.2), and wins over
both on a name listed here. Process: `docs/process/flow.md` §10 (milestones of units), with the deltas of §3 below.
Planner: Claude Opus 5.5 (`pipeline-planner`), from book `08C_publication_writer.md` (RATIFIED 2026-11-19, A22).

## 1. Goal
Build book `08C_publication_writer.md`: research's two photo looks also keep a factual **product note**, which the
owner's review decisions override; a **writer** run (a second kind of run in the research engine) drafts the title,
the description, the condition report description and Shopify's search title and search description from the fields
the owner lets it see, the note and the titles of matching listings; a deterministic **post-check**; a typed
**safeguard** that never re-raises what the owner decided (HW3); **Settings → Writer**; the two **SEO fields** and
their publish step; **Restoration** as fixed choices. Semantics live in 08C and the evidence file
`docs/intention/evidence/publication_writer_research.md`; §5 below is the part 08C leaves to planning: the exact rules
of the mechanisms that fail silently. Where 08C and this file differ, 08C wins and this file is amended.

## 2. Milestones (one runner phase each; order in `docs/implementation/phases_writer.txt`)
| Id | Builds | Units | Criteria | Rows | Changes app files | Projection | depends_on | Useful on its own |
|---|---|---|---|---|---|---|---|---|
| `W1_product_note` | the note in stages 1 and 6, the owner's overrides, staleness, Look again, the note in the research panel | 3 | 14 | 54 | yes | no | — | the owner sees what research saw, corrected by their own decisions, and can have changed photos looked at again |
| `W2_writer_engine` | writer settings and the shipped guide, the input, the post-check, the run's life, the call and regenerate, the writes, the safeguard, cost and export; all behind the crate's fakes | 5 | 25 | 96 | three files (the `ResearchDeps` literal, session delete, the schema tripwire list) | no | W1 | the whole writer is proven in the research crate; nothing in the app runs it yet |
| `W3_writer_review` | the title suggestion, publish partition with the built-in fields, the `writer` settings slot and wiring, the commands, the triggers, the AI review tab's writer block, questions and answers, the research panel's writer block | 3 | 10 | 30 | yes | no | W2 | texts arrive in AI review after "Next to review" or "Write texts", with questions and notes; defaults only |
| `W4_writer_settings` | Settings → Writer: model and effort, language, automatic writing, the instruction with Preview, versions and Reset, visible fields, examples, banned words, lengths, safeguard modes with counts, questions, shop name | 2 | 10 | 21 | yes | no | W3 | the owner tunes the writer |
| `W5_seo_publish` | the two SEO fields (always present), reading them from the shop, the publish step writing Shopify's `seo` input; U4 carried fixes from the W1–W4 reviews | 4 | 14 | 27 | yes | **yes** | W4 | search texts are written, reviewed and published |
| `W6_restoration_choices` | Restoration as fixed choices (app-local field), used by the writer and checked by the post-check | 1 | 5 | 10 | yes | no | W5 | the workshop's steps are stated exactly; **gated on owner card 1** |
Totals (computed from the plan files): 6 milestones, 18 units, 78 criteria, 238 rows (W5 U4 added 2026-11-19); every unit has at most 8 criteria.
Each milestone needs the one before approved (its `depends_on`), gate = the **Tracker** line of that plan file.
Start each with `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase <id>` (the runner resolves any id as
`docs/implementation/plans/<id>.md`); `run_phase.sh all` reads `phases.txt`, not this list.

## 3. Process deltas (flow §10 applies; these add to it)
- **No mechanism-inventory gate.** §5 stands in for it: one contract per silent-failure mechanism (WC1–WC8), each
  with its invariant. 08C has no measurement ledger of its own; trace cells cite `HW1`–`HW7`, 08C sections, the
  contracts `WC1`–`WC8` of §5, and the research ids they extend (`MA5`, `MA7`, `MA9`, `MA10`, `MA15`, `MA16`, `MA17`;
  `M17`, `M18`, `M20` for publish). `[rule]` only for UI rendering rows. The orchestrator may backfill a lettered
  ledger section in book `14` from WC1–WC8 later; no row changes id when it does.
- **Milestones of units** (flow §10): one implementer session works the units in order with a
  `CHECKPOINT (not approved):` commit after each, one review, one fix budget, one narrative per milestone. Reason for
  more than 8 criteria per milestone: as for research, a phase cycle per unit costs more than the work.
- **Implementer Claude Opus** (`PSM_IMPLEMENTER=claude`), reviewer Opus. No plan sets `implementer:`.
- **Projection** only for `W5_seo_publish` (the publish step and Shopify's `seo` shapes: rule 17).
- **Tests** (flow §4, §4A, §10): one behaviour test per row at the boundary it names; unit tests only for the
  plan's **Core mechanisms**. While working, the crate's own tests (`cargo test -p psm-research --test engine
  <stem>::`, `--lib writer::<module>`), the touched app stems and UI files. The full suite runs **once, at the end of
  a milestone that changed app files**; W2 changes three app files (U1: one line of wiring; U5: session delete, the schema tripwire list) and runs it once.
- **Row ids** `W<n>·C<k>(<row>)`, numbered once per milestone across its units; tests named after their row
  (`w2_c7a_<slug>`). Reviewer notes: `docs/implementation/backlog_writer/<milestone id>.md`.
- **Scratch data only** until this branch joins `main`: the app built from it runs with
  `PSM_DATA_DIR=$HOME/psm-research-dev` (it writes a settings slot and run triggers a `main` build does not know).
- **Guide.** A milestone that changes a flow, table, command, setting or rule updates `docs/guide/` in the same
  change (flow §7A); W3 and W5 carry it as a task.
- **Join:** after W6 (or after W5 while card 1 waits) the orchestrator opens the PR into `main` as research did.

## 4. Names (registry; exact)
**Code layout.** The writer is a module of the research crate, not its own crate: it uses the gateway, the run
store, the cost rules and the fakes through crate-private paths, which a second crate would have to make public.
`crates/research/src/writer/{mod.rs, api.rs, config.rs, defaults.rs, note.rs, input.rs, postcheck.rs, safeguard.rs,
run.rs, writes.rs, schema.rs}`. Pure rules with inline unit tests: `writer::note::{decision, assemble, stale}`,
`writer::postcheck::check`, `writer::safeguard::filter`, `writer::writes::stored_state`.

**Enums** (`psm_research::domain`, snake_case): `RunTrigger` + `Look` (`"look"`, W1), `Write` (`"write"`, W2) ·
`Stage` + `Write` (number 8, `"write"`; **not** in `Stage::ALL`, which stays the seven research stages; `as_str`,
`parse` and `number` cover it) · `writer::note::Decision {Accepted, Corrected, KeptPrevious, Dismissed, Typed,
Unchecked, ShopDefault, Imported, Empty}` · `writer::safeguard::{FindingType {FieldVsPhotos, FieldVsField,
FieldVsListings, MissingInText, TextVsFacts}, Mode {Off, Note, Ask}, FindingState {Open, Answered, Dismissed,
Superseded}}` · `writer::run::Origin {NextToReview, AllChecked, WriteTexts, Answer}` · `writer::WriterBlock
{NoIntegration, NoModelKey, PriceMissing{provider, model}, NothingToWrite, ResearchBusy, DayLimit, Offline}` ·
`writer::note::LookBlock {NoIntegration, NoModelKey, PriceMissing{provider, model}, Busy, NotStale}` ·
`writer::postcheck::HitCode {Length, Banned, Dash, Punctuation, Pattern, UseSentences, Damage, Grade, Stability,
Names, Years, Models, Restoration, NotRechecked}`.

**Text keys** (the writer's fields, 08C §9) and their output names: `product.title` = `title` ·
`product.description` = `description` · `writer.condition_report_key` (default
`custom.condition_report_description`) = `condition_report` · `product.seo_title` = `seo_title` (W5) ·
`product.seo_description` = `seo_description` (W5). New built-in definitions (integration `core`): W5
`product.seo_title` (Text, "Search title"), `product.seo_description` (Textarea, "Search description"), both group
Product; W6 `local.restoration` (MultiChoice, "Restoration done", group Condition, choices §5 WC8-R).

**Storage.** No numbered migration. `psm_research::schema::ensure` (idempotent) adds in W2:
`writer_findings(id INTEGER PRIMARY KEY, item_id INTEGER NOT NULL, session_id INTEGER NOT NULL, run_id INTEGER NOT
NULL, kind TEXT NOT NULL /* 'finding' | 'question' */, type TEXT NOT NULL /* FindingType or 'question' */, keys TEXT
NOT NULL /* JSON array, sorted, deduplicated */, text_field TEXT NOT NULL DEFAULT '', identity TEXT NOT NULL, text
TEXT NOT NULL, mode TEXT NOT NULL /* 'note' | 'ask' */, state TEXT NOT NULL /* FindingState */, answer TEXT, created_at
INTEGER NOT NULL, resolved_at INTEGER)` with index `(item_id, identity)`. Writer and look runs are `research_runs`
rows (triggers `write`, `look`); their units are `research_stage_outputs` rows (writer units: stage `write`, unit keys
`texts`, `retry`, `result`); their calls are `research_calls` rows. The note is in the stage 1 and stage 6 unit
outputs (§5 WC1). `delete_session` (A17) already deletes a session's research runs with their rows; it also deletes
the session's `writer_findings` (W2 task).

**Settings.** A new opaque slot `writer` in `settings.json` (`psm_infra::Settings.writer: Option<Json>`, W3), shape
`psm_research::writer::config::WriterSettings`, read tolerantly by `WriterSettings::from_slot`, checked by
`validate`; saving it never rebuilds the App (the slot is nulled in the rebuild comparison of
`crates/integrations/src/lib.rs`, beside `theme`, W3). Fields and defaults: `auto` (true) · `language` ("English";
"Swedish") · `model` ("gpt-6.1-sol") · `effort` ("medium") · `instructions` (`{<language>: {text: Option<String>,
versions: [{saved_at, text}]}}`; `text: None` = the shipped default of §9; at most 20 versions per language, newest
first) · `examples` (`{<language>: Option<[Example {name, fields, title, description, condition_report, seo_title,
seo_description}]>}`; `None` = shipped §9) · `banned_extra` ([]) · `lengths` ({`title_max`: 70,
`description_min_words`: 60, `description_max_words`: 120, `condition_max_words`: 45, `seo_title_max`: 60,
`seo_description_min`: 120, `seo_description_max`: 155}) · `visible_keys` (`None` = the default rule of WC2) ·
`safeguard` ({`field_vs_photos`: "ask", `field_vs_field`: "note", `field_vs_listings`: "note", `missing_in_text`:
"ask", `text_vs_facts`: "ask"}) · `max_questions` (3) · `shop_name` ("") · `condition_report_key`
("custom.condition_report_description") · `damage_terms` ({}: `{<key>: {<language>: [term]}}`). The model's price
and the per-run and per-day caps are research's (`research.prices`, `max_cost_per_run`, `max_cost_per_day`). Writer
runs snapshot `ResearchConfig` with `writer: Option<WriterSnapshot>` (the resolved settings, the instruction text
and examples actually sent) and `stages.write = {model, effort}`.

**API.** `psm_research::writer::api::WriterApi: Send + Sync` implemented for `ResearchService`; the wiring hands the
same service as `Arc<dyn WriterApi>` beside `Arc<dyn ResearchApi>` (W1 adds the handle; `AppState` answers
`research_unavailable` without it, HA4). Methods: W1 `note(session_id) -> NoteView`, `look_again(session_id) ->
Requested` · W3 `status(session_id) -> WriterStatus`, `request(session_id, Origin) -> Requested`,
`answer(finding_id, &str)`, `dismiss(finding_id)` · W4 `settings_validate(&Json)`, `instruction_default(language) ->
String`, `preview(&Json settings, language) -> Preview {instructions, input, schema}`. `Requested {queued: bool,
reason: Option<Reason>}`. `ResearchDeps` gains `writer_settings: SettingsSource` (W2; the harness supplies it; W3
wires it to the `writer` slot).

**Commands** (`apps/desktop/src-tauri/src/commands/writer.rs`, registered in `specta_builder()`; wrapper
`apps/desktop/src/services/writer.ts`): W1 `writer_note`, `writer_look_again` · W3 `writer_status`, `writer_request
(session_id, origin)`, `writer_finding_answer(finding_id, answer)`, `writer_finding_dismiss(finding_id)` · W4
`writer_settings_get`, `writer_settings_save(value)`, `writer_instruction_reset(language)`, `writer_preview(language)`.
Origins on the wire: `next_to_review`, `all_checked`, `write_texts` (`answer` is internal).

**Events.** None new: every writer change emits the existing `AppEvent::ResearchChanged` → `research_changed
{session_id}`.

**Log lines.** `writer.requested session= origin= queued= reason=` · `writer.started run= session=` ·
`writer.post_check run= attempt= hits=` · `writer.finished run= written= flagged= regenerated= findings= questions=
cost_micro_usd=` · `writer.look_again session= queued= reason=` · `publish.seo item= written=`.

**Frontend.** W1 `features/form/ProductNote.tsx` (in `ResearchPanel.tsx`) · W3 `features/form/WriterBlock.tsx` (AI
review tab), `features/form/WriterPanel.tsx` (research panel), the "Write texts" button in the Product step,
`features/sessions/reviewQueue.ts` and `App.tsx` (the triggers) · W4 `features/settings/WriterSection.tsx`,
`WriterInstruction.tsx`, `WriterExamples.tsx` and the nav entry in `SettingsNav.tsx`.

**Tests.** Engine stems `crates/research/tests/engine/{w1_note.rs, w2_writer_run.rs, w2_writer_writes.rs,
w2_safeguard.rs}` (each with its `mod` line in `main.rs`); app-seam stems `crates/testkit/tests/suite/research/
{w1_note_seam.rs, w3_title.rs, w3_writer_wiring.rs, w5_seo_publish.rs, w6_restoration.rs}` (`mod` lines in
`research/mod.rs`); desktop command tests in `apps/desktop/src-tauri/src/lib.rs`; UI
`features/<f>/__tests__/w<n>_<slug>.test.tsx`.

## 5. Contracts (stand in for the mechanism inventory; plans implement them verbatim)

Notation: `C(item, k)` the cleared set of 08B MC-A1; `S` its sameness; `N` 08B MC-A2's name key; "layout" =
`layout_keys` of the item's type (all three steps); "row" = the `attribute_values` row of `(item, k)`.

### WC1 — The product note: stored shape, tags, identity and reuse, staleness (08C §4)
1. **Request.** Stage 1 input gains `note_fields: [{key, label}]` = every key of the item's layout in the Details and
   Condition steps (not limited to `fill_keys`). Stage 6 input gains `photos: [{index}]` (1-based, in the order the
   images are attached) and `note_fields` = the Condition-step keys. Instructions (`stages.rs` `OBSERVE`,
   `CONDITION`) gain one sentence each, §9.3. `config::INSTRUCTIONS_VERSION` 2 → 3 (every stage identity changes once;
   `RESEARCH_FINGERPRINT_VERSION` stays 2: no identity field is added).
2. **Schema** (`adapters/openai/schema.rs`): Observe (the `photos` form) gains `note: array of {text: string, keys:
   array of string}`; Condition gains `note: array of {photo: integer, text: string, keys: array of string}`. All
   properties required, `additionalProperties: false`.
3. **Stored shape.** Stage 1 unit output gains `"note": [{"text", "keys"}]`; stage 6 output gains `"note": [{"text",
   "keys", "photo_id"}]`. Parsing: text trimmed, empty dropped, cut at 300 characters (char boundary); `keys`
   filtered to the request's `note_fields` keys, duplicates removed, order kept; at most 20 observations per unit (the
   rest dropped); stage 6 `photo` i maps to the i-th image actually sent (images dropped by the gateway removed, as
   `marking_photos` does), out of range → `photo_id: null`. A model reply without `note` (a fake or an older replay)
   stores `"note": []`.
4. **Identity and reuse.** Unchanged MC-A7: the note is part of its unit's output and is reused with it whenever the
   fingerprint is equal. No note table.
5. **Which note.** `N1` = the stage 1 output of the latest run (by id) of the session with trigger in {`arranged`,
   `manual`, `auto`, `follow_up`, `look`} that has a committed stage 1 unit whose output has the key `note`; `N6`
   likewise for stage 6. Chosen independently. An output without the key (before W1) does not count.
6. **Photo numbers.** An `N6` observation with `photo_id` p is shown as "condition photo n", n = 1-based index of p in
   the session's selected condition photos (07A MC-14 order: Condition by `(position, id)`, minus the damage report's
   `excluded`). If p is not among them the observation is left out of the writer input (the panel shows it, marked
   "photo no longer in the condition photos").
7. **Staleness** (computed at read time, never stored). `stale_main` ⇔ `N1` exists and `photo_plan(usable photos now,
   observe_max_photos).chosen` as `[id, revision, role]` ≠ the `photos` list of `N1`'s unit identity. `stale_condition`
   ⇔ a Condition field is in `research.fill_keys` ∧ ((`N6` exists and the sorted `(id, revision)` of the usable Condition photos now ≠ `N6`'s identity `photos`) or
   (no `N6` and usable Condition photos exist)) (amended 2026-11-19, W5·U4: no banner Look again cannot clear). `missing_main` ⇔ no `N1`
   and a usable Main photo exists. "Usable" is `stages::usable_photos` (eligible and with a source).
8. **Look again** (`look_again`). Queued (trigger `look`) iff: the session has an item; no run of the session with
   trigger ≠ `browse` is `queued` or `running` (else `Busy`); `stale_main ∨ stale_condition ∨ missing_main` (else
   `NotStale`); integration, model key and prices of the observe and condition models present (else that block, first
   failing in this order). A day limit does not block creation (the run waits queued, MC-A6). A `look` run executes
   stage 1 over the current `photo_plan` and stage 6 when usable Condition photos exist and a Condition field is
   chosen; nothing else: no stage 2–5 or 7, no attribute write, ends `finished`. It reuses a unit whose fingerprint
   equals one committed in the latest earlier run of the session with trigger in the set of item 5. It starts like a
   `manual` run (no quiet period; the source wait applies). Nothing is chained after it.
9. **Research views exclude** `look` and `write` runs as they exclude `browse`: `store::runs`, the two SQL queries of
   `ResearchService::overview`, `status`'s latest run, `arranged`'s check, the reuse lookup of a research run
   (`stages::execute` `earlier_run`). The start order and `tick` include them (`store::runs_in_state`). `run_again`
   when the session's queued run has trigger `look` or `write`: that run becomes `cancelled` with error "research
   started again" and a `manual` run is queued, in one transaction (the per-session queued index allows one).
**Invariant WC1.** Every observation the writer receives is in a committed stage 1 or 6 output of the run named by
item 5, with tags ⊆ that request's `note_fields`; `stale_*` is true exactly when the current photo identity differs
from the note's; a `look` run writes no attribute row and calls no stage other than 1 and 6.

### WC2 — Owner decisions over observations, and exactly what the writer sees (08C §4, §6, HW2, HW3)
**Decision of a key** (`writer::note::decision`, first match wins; `F` = the latest `research_feedback` row of
`(item, k)` by `(at, id)`; for `product.title` the row is WC4's stored state):
1. row exists, `ai_status = cleared` (any provenance) → `Dismissed`
2. row exists, `valid = false` → `Empty`
3. row `ai` + `suggested` → `Unchecked` (with its confidence)
4. row `ai` + `accepted` → `Accepted`
5. row `human`, live: `F.action = edited` → `Corrected`; otherwise → `Typed`
6. row `default` or `integration`, live: `F.action = kept_previous` → `KeptPrevious`; otherwise `ShopDefault`
   (default) or `Imported` (integration)
7. no row → `Empty`
**Owner's decision** = {`Accepted`, `Corrected`, `KeptPrevious`, `Dismissed`, `Typed`} (08C §4 list).
**Reviewed** (HW3, the never-re-raise predicate): `reviewed(item, k)` ⇔ row `ai_status = cleared` ∨ row `ai` +
`accepted` ∨ (some `research_feedback` row of `(item, k)` exists ∧ row is not `ai` + `suggested`). Feedback rows
exist only for applied actions (MC-A12), so an undone decision is not reviewed. A new research suggestion after a
decision makes the key unreviewed again.
**Undecided** = {`Typed`, `Unchecked`, `ShopDefault`, `Imported`} (08C §6: typed values research never looked at,
shop defaults, imported values; plus unchecked suggestions).
**Visible keys.** `writer.visible_keys` when set (∩ layout); otherwise every layout key in the Details and Condition
steps whose core type is not `money`, `media_list` or `reference`. The writer's own text keys are never visible.
**Input the writer receives** (`writer::input::build`, stored as the run's `input_snapshot`, byte for byte what the
request carries besides the instructions):
- `fields: [{key, label, value, decision, fact}]` for each visible key in layout order; `value` = the display text of
  the row's value (null for `Empty` and `Dismissed`); `fact` = decision ∉ {`Empty`, `Dismissed`} ∧ ¬(`Unchecked` with
  confidence `low`). Only `fact` values may be stated (HW2).
- `note: [{ref, text, keys, photo, owner}]`: `N1` observations (`ref` "p1.<i>") then `N6` (`ref` "c1.<i>", `photo`
  "condition photo n"). An observation one of whose keys has an owner's decision is **replaced**: `text` = those keys
  as "<label>: <value> (owner's decision)" joined by "; " (`Dismissed`: "<label>: not stated (the owner dismissed
  <values of C(item,k)>)"), `owner: true`; otherwise the text is kept, `owner: false`.
- `research_saw: [{key, text}]`: for every observation replaced only because of a `Typed` key, or tagged with a key
  whose decision is undecided, the original text per such key (the safeguard's comparison input; never text
  material).
- `listings: [{title, verdict}]`: titles of the stage 3 pages of the latest research run with a stage 4 verdict
  `match` or `similar`, `match` first, then by stage 3 order, at most 8.
- `answers: [{type, keys, question, answer}]`: the item's `writer_findings` with state `answered`, oldest first.
- `rejected: [{text, value}]`: per requested text, the values of `C(item, k)` (so it writes differently).
- `texts`: the requested output names (WC4), `language`, `lengths`, `shop_name` (omitted when blank).
**Invariant WC2.** For every key, the writer input states a value only where `fact` is true; an observation tagged
with an owner-decided key never reaches the writer in its original words except inside `research_saw` for a `Typed`
key; `reviewed` is computed from stored rows and feedback only.

### WC3 — The trigger (08C §5.1, A19)
`writer::run::request(session, origin)`, one transaction, in this order:
1. Unassigned session → refused (`NothingToWrite`).
2. The session has a run with trigger ∉ {`browse`, `write`} in state `queued` or `running` (research or look) →
   refused `ResearchBusy`.
3. Creation-blocking prerequisite, first failing: `NoIntegration`, `NoModelKey` (the research model key), 
   `PriceMissing` (no `fallback_call_usd` for (`research.providers.model`, `writer.model`)), `NothingToWrite` (no
   text key of WC4 is requestable) → refused with it.
4. Automatic origins (`next_to_review`, `all_checked`): `writer.auto = false` → nothing, no reason shown; the session
   has a `write` run in state `queued`, `running`, `finished` or `stopped_at_cap` → nothing (at most one automatic
   writing per session; a `failed` or `cancelled` one may be retried).
5. A `write` run of the session is `queued` → nothing new; answers `queued: true` (an `answer` origin's answer is
   already stored and the queued run reads it at its start).
6. Otherwise insert a `queued` run, trigger `write`, with `input_snapshot` null until start and the origin in the
   log line and the run's settings snapshot. (With a `write` run `running`, this is the one queued beside it.)
Repeat presses are therefore idempotent; "Write texts" while texts are being written queues exactly one more.
**Start.** Start order (`rules::run_rules::start_order`): tier 0 = triggers `manual`, `write`, `look`; tier 1 the
rest; then `queued_at`, then id. `may_start` for `write`: no run running app-wide, today's spend below the day cap,
no creation-blocking writer prerequisite; no quiet period, no source wait. Offline holds the run as MC-A6. The start
transaction snapshots settings (WC7), the form and the writer input (WC2).
**Frontend calls.** "Next to review →" (AI review tab) and the header's "Next to review" call `writer_request(current
session, next_to_review)` before moving on. Leaving a session's page or closing Details while that item's AI review
tab shows "All checked ✓" calls `writer_request(session, all_checked)`. "Write texts" (Product step) calls
`writer_request(session, write_texts)`. A refusal reason is shown only for `write_texts` (toast) and in the writer
block; automatic origins never show a toast.
**Invariant WC3.** Per session at most one `write` run is queued, none is created while a research or look run of the
session is queued or running, and automatic origins create at most one `write` run per session unless the earlier
ones all failed or were cancelled.

### WC4 — The writer's write rules (HW1; 08B MC-A1 applied to the five fields)
1. **Requestable texts.** `title` always (built-in, mandatory in the Product step); each other text key when it is in
   the layout, has an active definition with `writable = true` and core type `text`, `textarea` or `rich_text`. The
   request's schema has exactly the requestable texts.
2. **Fail closed.** `writer::writes` writes only the requestable text keys; research's `never_written` is unchanged
   (it still refuses all five), so research and the writer own disjoint keys.
3. **Stored state** `stored_state(item, k)` (read in the writing transaction): for `k ≠ product.title` the row; for
   `product.title`: the `product.title` attribute row if one exists; else none when `items.title` is blank; else a
   synthetic live row with value `items.title` and provenance `integration` when the item's link title base equals
   `items.title` (the rule `form_get` uses today), otherwise provenance `human`.
4. **Decision** = `rules::write_table::decide_write(definition, stored_state, proposed, C(item, k))`, the table
   unchanged: write over none, live `default`/`integration` (saving `ai_prior`), own `ai` + `suggested` (keeping its
   prior); never over `ai` + `accepted` or `human`; over a cleared row only a value not in `C`; everything else
   refused. `S` for `text`/`textarea` is `N(a) = N(b)`; for `rich_text` canonical equality.
5. **Row written.** Provenance `ai`, status `suggested`; confidence `medium` when the text has no post-check hit left,
   `low` when flagged (WC5); `ai_reason` = "<reason>\nFacts: <labels of the facts used>" plus, when flagged,
   "\nCheck: <hit details joined by '; '>"; `ai_sources` = []. Value: `description` = paragraphs joined by "\n\n"
   (canonical rich text); the others trimmed. An empty text is not written. The same-content check of
   `writes::suggest_in` applies (a repeat is a no-op). `product.title` is written as an attribute row with key
   `product.title`; `items.title` is never written by the writer.
6. **Commit.** All writes, the run's steps and its findings commit in one transaction (the `result` unit).
7. **Title in core** (W3): the form shows a live `product.title` `ai` row when it is `suggested`, or `accepted` with
   value equal to `items.title`; otherwise `items.title` as today. Accept / Use suggestion / Accept all: row → `ai` +
   `accepted` **and** `items.title` := its value, one undo step, feedback `accepted`. Keep previous: the row is
   deleted, `items.title` unchanged, feedback `kept_previous`. Dismiss (title was empty): row → `cleared`. A human
   title edit deletes the row in the same transaction (feedback `edited` when it was `ai`). Undo and redo restore the
   row and `items.title` together (MC-A12 conditional undo applies to both).
8. **Publish partition** (W3): `partition_in` passes `builtin_definitions()` ∪ the shop's definitions, so a
   description or search text with a prior is a real conflict (held, MC-A10) instead of being left out; rows with key
   `product.title` are skipped by the partition (never frozen, withheld or pending): the title sent is `items.title`
   frozen at confirmation, so a title suggestion is published only after the owner accepts it.
**Invariant WC4 (extends MA10).** For every stored state and every writer write, the row changes only where the table
allows; no value in `C(item, k)` is written; no key outside the requestable texts is written; `items.title` changes
only by the owner's action.

### WC5 — The deterministic post-check (08C §5.4, §7; evidence §7.5)
`writer::postcheck::check(texts, ctx) -> Vec<Hit {text, code, detail}>`, per text after trimming; `L` the run's
language; "contains" is case-insensitive on NFC text; a word match uses Unicode word boundaries; a phrase match is a
substring match.
1. **Length** (`Length`). Characters = Unicode scalar values; words = whitespace-separated tokens containing a letter
   or digit. `title` ≤ `title_max`; `description` words in [`description_min_words`, `description_max_words`];
   `condition_report` words ≤ `condition_max_words`; `seo_title` ≤ `seo_title_max`; `seo_description` characters in
   [`seo_description_min`, `seo_description_max`].
2. **Banned** (`Banned`), both languages always plus `banned_extra`. Words: timeless, elevate, elevates, testament,
   boasts, stunning, iconic, nestled, vibrant, showcase, showcases, showcasing, seamless, effortless, versatile,
   must-have, exquisite, impeccable, meticulous, curated, delve, delves, tapestry, embodies, exemplifies, beloved,
   gem, treasure, tidlös, ikonisk, fantastisk, otrolig, unik. Phrases: "statement piece", "perfect for", "adds a touch
   of", "warmth and character", "piece of history", "whether you", "look no further", "en riktig pärla", "ett måste",
   "perfekt för", "lyfter rummet", "tillför värme". "iconic", "ikonisk", "unik" are exempt when a `fact` value
   contains the word. ("rich (as praise)" is not decidable without meaning and stays in the instruction only.)
3. **Dash** (`Dash`): U+2014 anywhere; U+2013 unless both neighbours are digits; " - " (a hyphen with a space on
   both sides).
4. **Punctuation** (`Punctuation`): "!" or "?".
5. **Pattern** (`Pattern`, English): "not just" followed later in the same sentence by "but"; "it's not" or "it is
   not" followed later in the same sentence by "it's" or "it is"; "serves as"; ", " followed by one of highlighting,
   adding, showcasing, offering, featuring, creating, bringing, providing, reflecting, ensuring, making, giving,
   lending, boasting. (Three comma-separated adjectives need a part-of-speech tagger and stay in the instruction.)
6. **Use sentences** (`UseSentences`, `description`): sentences (split after ". ", "! ", "? " or the end) containing a
   marker: EN " suits ", " works at ", " works in ", " works as ", " use it ", " use them ", " ideal for ", " fits ",
   " can be used "; SV " passar ", " fungerar ", " kan användas ". More than one → hit.
7. **Damage coverage** (`Damage`, `condition_report`): for each visible Condition-step key that is a severity choice
   (`is_severity_set`) whose value is a `fact` not in {"None", "N/A"}: covered iff the report contains one of its
   terms. Terms = `damage_terms[key][L]` when set; else for English the label after removing a leading "Condition."
   split on non-letters, words of at least 4 letters, each with a trailing "es" or "s" removed; for Swedish those
   English words mapped by: scratch→rep, stain→fläck, discoloration→missfärg, veneer→fanér, dent→buckl, chip→nagg,
   crack→spricka, water→vatten, damage→skad, stability→stadig|glapp, missing→saknas, part→del,
   upholstery→klädsel|stoppning, odor→lukt, mold→mögel, function→funktion|fungerar, wear→slit (a word with no mapping
   contributes no term). A key with no term is never a hit.
8. **Grade word** (`Grade`, `condition_report`): when `research.detailed_condition.key` is visible and holds a
   `fact`: the grade = the value's text before the first " - " (trimmed); Swedish maps Excellent→Utmärkt, Very
   Good→Mycket gott, Good→Gott, Worn→Slitet (case-insensitive lookup; unmapped → the English grade). The report must
   start with the grade (case-insensitive). No fact → no check.
9. **Stability** (`Stability`, `condition_report`): contains one of EN stable, sturdy, firm, sound, wobble, wobbles,
   loose; SV stadig, stabil, glapp.
10. **Names only from fields** (`Names`, `Years`, `Models`, every text). `A` = the `fact` values plus the note texts
    the writer received (after replacement), each `N`-keyed. (i) `Names`: every choice string of every definition
    whose key is in `research.attribution_keys`, and every value of those keys stored on any item, that the text
    contains (by `N` substring) must be an `N` substring of `A`. (ii) `Years`: every `\b(18|19|20)\d\d\b` in the text
    must occur as a 4-digit number in `A`. (iii) `Models`: every token `\b[A-Z]{1,4}[- ]?\d{1,4}[A-Z]?\b`, except
    tokens whose letters are a single W, D, H, B, L, A or I, tokens followed by a unit (`\s?(mm|cm|m|kg|%)\b`), and except the line starting "Dimensions:" or "Mått:", must
    occur in `A` after removing spaces and hyphens and upper-casing.
11. **Restoration** (`Restoration`, W6): each selected `local.restoration` choice must appear by one of its phrases
    for `L` (WC8-R) in the condition report (owner card 1, 2026-11-19: the customer reads the restoration in the
    condition report description; the product description may mention it but does not count).
**Regenerate once.** After the first call, hits → one more call (unit `retry`) asking to rewrite only the texts with
hits, naming each hit; clean texts of the first call are kept. After the retry, every text with a hit is
**flagged** (WC4.5). If the retry is not made (cost cap, WC7), each text with a hit is flagged with an added hit
`NotRechecked` "not checked again: run cost limit". Hits of code `Damage` become `MissingInText` findings and hits
of codes `Names`, `Years`, `Models` become `TextVsFacts` findings (WC6), keys = the field key (damage) or the text key.
**Invariant WC5.** For the same texts and context `check` returns the same hits; no text reaches the form unflagged
while it has a hit; at most two writer calls per run.

### WC6 — Safeguard types, modes, and never-re-raise (08C §6, HW3)
Types and default modes: `field_vs_photos` ask, `field_vs_field` note, `field_vs_listings` note, `missing_in_text`
ask, `text_vs_facts` ask. Model findings arrive in the call's `findings: [{type, keys, text}]` and `questions:
[{keys, text}]`; post-check findings per WC5.
**Filter** (`writer::safeguard::filter`, deterministic, before storing; per candidate `f` with type `t`, keys `K`
(layout keys only; others removed), text key `tf`):
1. `mode(t) = off` → dropped.
2. Subject: `K` for `field_vs_*` and questions; `{tf}` for `missing_in_text` and `text_vs_facts`. Any subject key
   `reviewed` (WC2) → dropped. A `missing_in_text` or `text_vs_facts` finding whose text `tf` was not written or
   unchanged in this run (write outcome other than `written` / `unchanged`, e.g. the owner's own text) → dropped
   (amended 2026-11-19, W5·U4).
3. `field_vs_photos`: dropped unless every key of `K` is undecided (WC2) and holds a value.
4. `field_vs_*` with `K` empty → dropped.
5. `identity(f)` = `t|<K sorted, comma-joined>|tf`, and for questions `question|<K>||<N(text)>`. Equal to the identity
   of a `writer_findings` row of the item with state `answered` or `dismissed` → dropped; a duplicate within the run
   → dropped.
6. **Cap.** Asks are the surviving `ask`-mode findings in the type order of the table, then by identity, then the
   questions in reply order; at most `max_questions` in total: asks beyond it are stored as `note`, questions beyond it
   are dropped (the run's step lists them).
**Storage** (in the `result` transaction): one `writer_findings` row per survivor, state `open`; every earlier `open`
row of the item becomes `superseded`. **Answer**: state `answered`, `answer`, `resolved_at`, then `request(session,
Answer)`. **Dismiss**: state `dismissed`, `resolved_at`, no run. Counts per type (answered, dismissed) are a query.
**Invariant WC6 (HW3 as a predicate).** For every `writer_findings` row `r` with state `open` committed by run `R`: no
subject key of `r` was `reviewed` in `R`'s commit transaction, `mode(r.type) ≠ off` in `R`'s snapshot, and no row of
the item with state `answered` or `dismissed` committed before `R`'s commit has `identity = r.identity`.

### WC7 — Cost, the day cap and the call record (HW4, HW6; extends MC-A8, MC-A9, MC-A11)
- Every writer call goes through `CallGateway::call` with `Stage::Write`, unit keys `texts` and `retry`; it is a
  `research_calls` row with provider `research.providers.model` (resolved), model `writer.model`, effort
  `writer.effort`; its cost follows MC-A8 from the run's price snapshot; it adds to `research_runs.cost` and to
  today's spend; the day cap is checked at start (MC-A6), shared with research and browse.
- **Stop rule:** a `Stage::Write` call is made iff `run.cost < max_cost_per_run` (`rules::cost::next_call` treats
  `Write` like `Decide`).
- **HW4:** a writer request carries no image (`images` and `listing_images` empty); no `research_outbound` row is ever
  written for a `write` run.
- **Record and export:** writer and look runs are exported by `export` (it already lists all runs): `run.json` with
  their trigger, `steps.json`, `calls/`, `feedback.json`; `writes.json` = the `result` unit output for a `write` run
  (`{texts, hits: [{attempt, hits}], writes: [{key, outcome}], findings, questions}`), null for a `look` run. Retention
  as MC-A11. Replay: `Stage::parse("write")` resolves, so the tool can rebuild a writer call.
**Invariant WC7 (extends MA5, MA15, MA16).** Every writer call is recorded and costed like a research call; the day's
spend includes writer calls; no writer request contains an image; every writer run of a range is in its export.

### WC8 — The SEO publish step (08C §9; idempotent and resumable like 07A MC-11, MC-13)
1. **Fields.** `product.seo_title` (Text), `product.seo_description` (Textarea): built-in, group Product, **always
   present**: `layout_keys` and `form_get` append each to the end of the Product step when a stored preset lacks it
   (W5 mechanism; W6 uses it for `local.restoration` in the Condition step).
2. **Read from the shop** (P67 remote read): `RemoteValues` gains `seo_title`, `seo_description` (`Option<String>`,
   blank → `None`); stored as `integration` rows exactly as the description is.
3. **Partition.** As every key (WC4.8): a `low` suggestion without prior is withheld; one with a prior is a conflict
   (held until resolved).
4. **Update step** (`PublishService::update`, after the description): `F` = `item.run_attributes()`. When `F` holds
   neither key with a non-blank value → no read, no write. Else read the remote `seo {title, description}` (null ≡
   blank). Desired = `F`'s non-blank value per key, else the remote value unchanged. Equal (trimmed) to remote → no
   mutation. Else one `productUpdate {id, seo: {title, description}}` with both fields (an unchanged one sends the
   remote value as read, null stays null), carrying the confirmed run (H1). Read back; a changed key whose read-back
   (trimmed) differs → `Error::AttributeVerification {keys}` naming `product.seo_title` / `product.seo_description`.
5. **Resume.** The step re-reads before writing, so a kill after the mutation and before the step commits ends with
   no second mutation (M18); a publish never clears an SEO field the owner has no value for (M20).
6. **Shopify shapes** (schema read 2026-11-19 through the Shopify Admin schema tool): `Product.seo: SEO!`,
   `SEO {title: String, description: String}`; `ProductUpdateInput.seo: SEOInput {title: String, description:
   String}`. Values for an unset SEO field (null or "") and the handling of a title over 70 characters are captured by
   W5·U2 (fixture task; projection checks it).
**WC8-R (Restoration, W6, owner card 1 answered A, 2026-11-19).** `local.restoration`: built-in MultiChoice, group Condition, always
present; choices (canonical English) with phrases per language for the post-check: Cleaned (cleaned / rengjord,
rengjorda) · Sanded (sanded / slipad, slipade, slipat) · Oiled (oiled / oljad, oljade, oljat) · Waxed (waxed /
vaxad, vaxat) · Lacquered (lacquered / lackad, lackerad, lackat) · Joints re-glued (re-glued / limmad, limmade) ·
Veneer repaired (veneer repaired, repaired the veneer / lagad fanér, fanéren lagad) · New foam (new foam / ny
stoppning) · New webbing (new webbing / nya gjordar) · New springs (new springs / nya fjädrar) · New papercord (new
papercord, rewove / nytt pappersgarn, nyflätad) · New fabric (new fabric, reupholstered / nyklädd, nytt tyg) · New
leather (new leather / nytt läder) · Rewired (rewired / nya elkablar, omdragen el). Never published (no definition
link: `write_set` skips it); research never writes it (`never_written` names it); the free-text `custom.restoration`
stays an ordinary visible field.
**Invariant WC8 (extends M17, M18, M20).** Every SEO mutation carries a confirmed run and sends only frozen or
resolved values; resuming at any instant issues no second identical mutation; no SEO field is cleared by a publish.

## 6. Delegated choices (decided here; each is one sentence in a narrative's "Choices made for you")
- **One crate.** The writer is a module of `crates/research` (reasons in §4).
- **Settings slot `writer`,** not a sub-object of `research`: the research DTO round-trip would otherwise drop it.
- **Default model** `gpt-6.1-sol`, effort `medium` (text only; about two cents a write). The owner tests others.
- **Clean texts are `medium`, flagged texts `low`.** So Accept all takes clean texts, skips flagged ones, and a
  flagged text with no prior is withheld at publish (MC-A10 unchanged).
- **A title suggestion is never published before it is accepted** (the title's home is the item's title; WC4.7–8).
  The description and the search texts follow MC-A10 like every research suggestion.
- **Look again chains nothing:** after it, the writer block says the photos were looked at again; "Write texts"
  uses the new note.
- **Stage 6 is not made to run for the note alone:** with no Condition field chosen, there is no condition note.
- **The Swedish instruction is written in Swedish** (§9.2; evidence §4: match the prompt's style to the output). The
  Swedish example set starts with example D only; the owner adds more (08C §13).
- **Example adaptations** (§9.4): C opens with "Worn" (08C's grade words); D gains "och raka ben" to reach 60 words;
  B and C get search texts written to §7.6; D's search title drops "| {Butik}".
- **Shop name** is a setting (`writer.shop_name`, empty by default): no shop name is read from Shopify.
- **Post-check limits of evidence §7.5:** "rich (as praise)" and "three adjectives" stay in the instruction only.
- **Findings are stored per item,** superseded by the next writer run; answered and dismissed ones are kept for the
  never-re-raise rule and the counts.
- **Restoration choices** are app-local (owner card 1 default), listed in WC8-R.

## 7. Environment pointers
Commands and caveats: `master_plan_research.md` §7.2, `master_plan.md` §7, `docs/guide/05_working.md`. While working:
`cargo test -p psm-research --lib writer::<module>` · `cargo test -p psm-research --test engine <stem>::` ·
`cargo test -p psm-testkit --test suite research::<stem>::` · `cargo test -p psm-desktop` (regenerates
`apps/desktop/src/services/bindings.ts` through `export_bindings`; never hand-edit) · `cd apps/desktop && npx vitest
run <files>` · `npm run typecheck` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo check
--workspace`. Closing gate of a milestone that changed app files (once): `cargo test --workspace` · `cargo clippy
--workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` ·
`npm test`. One cargo command at a time, never in the background, never piped. Known intermittent tests:
`docs/guide/04_change_impact.md` §Fragile spots.

## 8. Log
- **v1, 2026-11-19 (planner).** Written from 08C. One owner card (Restoration: app-local field, or a Shopify field the
  owner creates), gating W6 only; see `handoffs/reviewer/handoff_writer_plan_r1.md`.

## 9. The shipped writing guide (08C §7; evidence §7, §8)
### 9.1 English instruction (`writer::defaults::INSTRUCTION_EN`, verbatim)
> You write product texts for a small Swedish shop that sells restored Scandinavian mid-century furniture. You
> receive the item's fields, a short note of what the photos show, the titles of matching listings from other
> sellers, and the owner's earlier answers. Write the texts listed in `texts`: a title, a description, a condition
> report, a search title and a search description.
>
> Use only facts in the fields marked as facts and in the photo note. A value marked as the owner's decision wins
> over anything else. If a field is empty, leave that fact out. Never guess a designer, model, date or material. If
> the designer field says the attribution is uncertain, write "attributed to". Use the era field as it stands. Copy
> names, model numbers and manufacturer names exactly, including Danish letters. From the matching listings take
> only nouns, such as the maker's name, a model number or a type word, at most once in the title and once in the
> description, and never their adjectives or spellings that differ from the fields.
>
> Write like a careful dealer writing a catalogue entry: plain sentences, concrete nouns, verbs like "is", "has" and
> "was made". Prefer one precise detail to a general claim. Describe shape, material and what was done to the piece.
> Use only adjectives a buyer could check in the photos.
>
> Title: follow the title formula. Commas only, no words of praise, at most 70 characters.
>
> Description: 60 to 120 words in short paragraphs, in this order. First, what it is, who made it, when and where.
> Second, one or two sentences on form and materials. Third, what our workshop did, naming each restoration step
> exactly as the fields give it. Fourth, at most one sentence on where it can be used. End with the dimensions line
> in the given format, with the seat height for seating.
>
> Condition report: at most 45 words. Begin with the grade word of the Detailed condition field. Then the
> restoration in one sentence. Then each remaining flaw with its location and approximate size, citing the
> condition photo number if given. End with a sentence on stability. Never soften a flaw and never omit one the
> condition fields list.
>
> Search title: at most 60 characters, with the designer or the country, the type, the material and the decade; add
> " | " and the shop name only if it still fits. Search description: 120 to 155 characters, one or two sentences
> with what it is, the main restoration fact and the condition grade; end with "Ships from Sweden." if it fits.
>
> Use commas and full stops. No dashes as punctuation, no exclamation marks or questions, no strings of three
> adjectives. Do not mention AI, the writing process or the shop's values.
>
> Follow the style of the examples below but never reuse their sentences. Report contradictions you notice as
> findings, and ask a question only when a fact you need is missing or unclear.

Followed by the title formula (evidence §7.2, English lines: designer known, designer unknown, attributed; title
case), the dimensions line format `Dimensions: W 68 × D 72 × H 76 cm, seat height 42 cm.`, and the examples.

### 9.2 Swedish instruction (`writer::defaults::INSTRUCTION_SV`, verbatim)
> Du skriver produkttexter för en liten svensk butik som säljer renoverade skandinaviska möbler från mitten av
> 1900-talet. Du får föremålets fält, en kort anteckning om vad bilderna visar, rubriker från andra säljares
> matchande annonser och ägarens tidigare svar. Skriv texterna i `texts`: en rubrik, en beskrivning, en
> skickrapport, en sökrubrik och en sökbeskrivning.
>
> Använd bara fakta från fält som är markerade som fakta och från bildanteckningen. Ett värde som är ägarens beslut
> går före allt annat. Är ett fält tomt utelämnar du den uppgiften. Gissa aldrig formgivare, modell, årtal eller
> material. Säger formgivarfältet att attributionen är osäker skriver du "troligen". Använd tidsfältet som det står.
> Skriv namn, modellnummer och tillverkarnamn exakt, med danska bokstäver, i originalstavning. Ta bara substantiv
> från de matchande annonserna, som tillverkarens namn, ett modellnummer eller ett typord, högst en gång i rubriken
> och en gång i beskrivningen, aldrig deras adjektiv eller stavningar som skiljer sig från fälten.
>
> Skriv som en noggrann handlare som skriver en katalogtext: enkla meningar, konkreta substantiv, verb som "är",
> "har" och "tillverkades". Välj en exakt detalj framför ett allmänt påstående. Beskriv form, material och vad som
> har gjorts med möbeln. Använd bara adjektiv som en köpare kan kontrollera på bilderna. Skriv som en svensk
> handlare, inte som en översättning.
>
> Rubrik: följ rubrikformeln. Bara kommatecken, inga berömmande ord, högst 70 tecken, versal bara i början och i
> namn.
>
> Beskrivning: 60 till 120 ord i korta stycken, i denna ordning. Först vad det är, vem som tillverkade den, när och
> var. Sedan en eller två meningar om form och material. Sedan vad vår verkstad har gjort, med varje
> renoveringssteg exakt som fälten anger det. Sedan högst en mening om var den kan användas. Avsluta med måttraden i
> angivet format, med sitthöjd för sittmöbler.
>
> Skickrapport: högst 45 ord. Börja med skickordet från fältet Detailed condition (Utmärkt, Mycket gott, Gott eller
> Slitet skick). Sedan renoveringen i en mening. Sedan varje kvarvarande skada med plats och ungefärlig storlek, med
> bildnumret om det finns. Avsluta med en mening om stabiliteten. Förminska aldrig en skada och utelämna aldrig en
> som skickfälten anger.
>
> Sökrubrik: högst 60 tecken, med formgivaren eller landet, typen, materialet och decenniet; lägg till " | " och
> butikens namn bara om det ryms. Sökbeskrivning: 120 till 155 tecken, en eller två meningar med vad det är, den
> viktigaste renoveringen och skicket; avsluta med "Skickas från Sverige." om det ryms.
>
> Använd kommatecken och punkt. Inga tankstreck, inga utropstecken eller frågor, inga rader med tre adjektiv. Nämn
> aldrig AI, hur texten skrevs eller butikens värderingar.
>
> Följ stilen i exemplen nedan men återanvänd aldrig deras meningar. Rapportera motsägelser du ser som fynd, och
> ställ en fråga bara när en uppgift du behöver saknas eller är oklar.

Followed by the Swedish title formula (evidence §7.2, Swedish lines; sentence case), `Mått: B 68 × D 72 × H 76 cm,
sitthöjd 42 cm.`, and the examples. Note: the post-check grade word for Swedish is the mapped word of WC5.8 ("Slitet",
the instruction's "Slitet skick" starts with it).

### 9.3 Note sentences added to the stage instructions (part of `INSTRUCTIONS_VERSION` 3)
- `OBSERVE` + " Also write `note`: short factual observations of what the piece is, its form, construction, materials
  and visible details, each tagged with the keys from `note_fields` it bears on (none when it bears on no field)."
- `CONDITION` + " Also write `note`: per photo, by its index in `photos`, the wear it shows and where, tagged with the
  condition keys from `note_fields` it bears on."

### 9.4 Shipped examples (`writer::defaults::EXAMPLES_EN` = A, B, C; `EXAMPLES_SV` = D)
Evidence §8 A–D verbatim (fields line, title, description, condition, search title, search description), with these
changes only: **A** unchanged. **B** search title "Danish Teak Nesting Tables, Set of 3, 1960s"; search description
"Set of three Danish teak nesting tables from the 1960s, cleaned and re-oiled with the original finish kept. Good
condition. Ships from Sweden." **C** condition opens "Worn, cleaned only." (was "Fair, cleaned only."); search title
"Danish Teak Sideboard with Sliding Doors, 1960s"; search description "Danish teak sideboard from the 1960s with
sliding doors, shelves and four drawers. Sold as found and cleaned only. Worn condition. Ships from Sweden." **D**
fields line gains "raka ben"; description's second paragraph begins "Stommen är i massiv teak med svängda armstöd och
raka ben."; search title "Dansk teakfåtölj i ullbouclé, 1960-tal". Measured by the planner against WC5's lengths
(words: A 65/29, B 74/36, C 67/45, D 60/25; search characters: A 51/148, B 43/142, C 47/149, D 38/136): every
example passes the post-check.
