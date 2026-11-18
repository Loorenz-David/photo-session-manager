# 08A — Research agent (product facts)

**Status: RATIFIED** (owner David Loorenz, 2026-11-12; shaping changelog entry 6). Shaper Claude.
Branch `feature/agentic`. Raw idea: `docs/agentic-implementation/PRODUCT_INTELLIGENCE_IMPLEMENTATION_INTENT.md`.
This chapter amends chapter 08 §AI agent. It is the first of three assistants; the other two get
their own lettered chapters later and are non-goals here (§Scope ladder).

## 1. Outcome

While the photographer shoots, Session Manager researches the piece the way the owner does by hand
today: it looks at the photos, finds visually matching listings on the web, reads them, compares
them with the piece, and fills the item's form with **suggested** values, each with a confidence,
a one-paragraph reason and the links it relied on. The photographer checks them in the form
(Looks right / Clear / edit) and can open the research panel to see every step the agent took.
A suggestion the photographer has not checked reaches the shop only at medium or high confidence, inside a confirmed publish; a low one never does (§7).

## 2. Hard constraints (added to H1–H8, same standing)

| # | Constraint | Why |
|---|---|---|
| HA1 | The agent never writes a field that holds a `human` value or an `ai` value with status `accepted`, and never re-suggests a value the photographer cleared. | A checked value must stay checked. |
| HA2 | A designer, manufacturer or other attribution value is never above `low` confidence on visual similarity alone (§6). | A wrong attribution on a listing is a false claim to a buyer. |
| HA3 | No RAW and no photo classified `deleted` is ever sent to a model or search provider. Every image that leaves the Mac for research is recorded against a research run. | Owner decision 3 (2026-11-12); H2; M3's "no deleted photo has left the Mac". |
| HA4 | With no provider key, no network or no integration configured, every other part of the app works exactly as before; research simply does not run and says why. | H6. |
| HA5 | Provider-, model- and site-specific concepts (a model name, a search service, a reseller domain) never appear in core modules or core tables. They live in adapters and settings. | H7. |
| HA6 | A research run never spends more than its cost cap (§9). | Unattended runs on every item. |

## 3. Grounding (what exists on `main` at `a00b7ee`)

- **Values and provenance:** `attribute_values` already carries provenance `human` / `ai` / `default`
  / `integration`, `ai_confidence` (high / medium / low), `ai_status` (suggested / accepted /
  cleared), `ai_reason`, `ai_sources` (`crates/core/src/domain/attribute.rs`).
- **Write path:** `AttributesService::ai_suggest`, `accept_all`, clear and "Looks right" exist
  (`crates/core/src/services/attributes.rs`); `ai_suggest` already refuses a human-owned field (live or
  cleared) but does **not** refuse an `ai`+`accepted` one; 08B MC-A1 requires that refusal (inventory
  2026-11-12). Nothing calls `ai_suggest` in production.
- **UI:** AI pill, popover (`AiPopover.tsx`) and research panel (`ResearchPanel.tsx`) exist. The
  panel is a static list built from stored provenance (one block per AI field). The design mock-up
  shows more: a live timeline of steps (looked, searched, read a listing, compared, concluded) with
  a Finished state and "n fields filled · n to review".
- **Which fields:** the form of an item is the Customize-fields layout of its item type
  (`presets`, chapter 08). The item type comes from the shop's product Type (changelog 42).
- **Durable work:** the `jobs` table and in-process runner (chapter 05) carry preview, export and
  publish work across restarts.
- **Layers:** chapter 11 reserves "AI orchestration (later)" in the services layer. HTTP from Rust
  is already used by the Shopify adapter (`reqwest`). Secrets go through `ports/secrets` (Keychain).
- **Not existing:** shot templates, any model or search adapter, any research table.

## 4. Where it lives (owner decision 1 = C, 2026-11-12)

The engine runs **inside Session Manager**: a `research` application service in the Rust services
layer, its work carried by the existing durable job queue, its state in SQLite. No Temporal, no
Python, no PostgreSQL, no separate server, no tenants (replaces §4, §6 and Milestone 1 of the raw
idea).

It is **service-ready**: the research service talks to the rest of the app only through three
plain data shapes, so a later remote engine (the Beyo central domain) can replace the in-app one
behind one interface without changing the form, the tables or the UI.

1. **Form snapshot (in):** the item's type and the ordered fields of its layout, each with key,
   label, core type, choices, description, current value and provenance. Keys are the app's
   existing attribute keys, unchanged (raw idea §2).
2. **Evidence (in):** the session's photos that may be used (§5), each with id, classification,
   position and the rendered file.
3. **Result (out):** per field, a suggested value or "not determined", confidence, reason, sources;
   and the ordered trace of steps.

Three provider interfaces (ports in core, adapters in `crates/integrations`):

| Port | Does | First adapter |
|---|---|---|
| model provider | looks at images and text, returns structured answers | OpenAI API (owner 2026-11-12: "Codex"; key already held) |
| visual search provider | given a photo, returns visually matching web pages | **chosen by benchmark (owner 2026-11-12, option A; §12A).** Candidates in `evidence/research_providers.md`: Google Lens through a vendor (closest to the owner's manual method; Google disputes its legality) or Google Cloud Vision Web Detection (first-party; weaker on look-alikes by report). Both take the photo's bytes, so no photo needs a public address. |
| page reader | fetches one listing page and returns its text and main images | open until probe 1 of the evidence doc: the model provider's page-open tool restricted to the trusted sites, or a plain HTTP fetch from Rust |

The shop's own catalogue needs no extra port in this stage: the owner's listings are public pages
and are found by the visual search like any other (the owner lands on their own shop's page in
about 40 % of manual searches).

## 5. Workflow

**Start (owner decision: automatic, with a switch).** A research run is queued for an item when
its session has at least one Main photo and research has not run for it yet. Setting
`research.auto` (default on) turns this off; the form's research panel has **Research** /
**Run again** for a manual run. Photos are approved on arrival (changelog 46), so the first capture is the first Main photo.

**Quiet period (owner 2026-11-12).** A run, first or follow-up, starts when no photo has arrived or
changed in the session for `research.quiet_seconds`, default **1**. A studio with a photographer
still learning raises it so test frames are not researched. While a run is `running`, further
changes queue at most one follow-up, which starts when the run ends and the quiet period has passed.

**Photos used.** Main and Condition photos, and unreviewed ones; never `deleted` (HA3). What is
sent is a reduced copy of the photo as composed in the app, cut from its **adjusted preview** by the
composition rule (10D MC-27; H3 as amended, owner card 3, 2026-11-12), longest side 1024 px, 2048 for
reading labels and condition (R-IA13), metadata removed; never the RAW or the master, and no
master is rendered for research (08B MC-A5).

**Stages of one run.** Each is a bounded step with a typed input and output, its own model and
effort setting (§9), and one or more trace steps (§8). No stage is a free-running agent.

| # | Stage | Input | Output |
|---|---|---|---|
| 1 | Observe | each Main photo, looked at once and remembered (owner card 2, 2026-11-12) | what is visible: piece type, materials, construction, markings; whether the photo shows a marking |
| 2 | Find matches | cover photo | candidate pages from the visual search provider |
| 3 | Read candidates | top candidate pages (trusted sites first, §6) | per page: title, attributed designer / maker / period, its images |
| 4 | Compare | the cover photo vs each candidate's images (owner card 2) | match / similar / different, with the features compared |
| 5 | Read labels | photos stage 1 found showing a label, stamp or marking | transcribed text |
| 6 | Read condition | Condition photos | values for the layout's condition fields |
| 7 | Decide | all of the above + the form snapshot | per field: value or not determined, confidence, reason, sources |

**New photos after a run (progressive).** A photo added, reclassified to or from `deleted`, or
recomposed after the last run queues a **follow-up run**, which redoes only the stages whose input
changed (a new Main photo → stage 1 for that photo only, 5 if it shows a marking, and 7; a new Condition
photo → 6 and 7; a new cover → all). The exact trigger set and unit identities are 08B MC-A7.
Follow-ups obey the quiet period above; the day cap (§9) bounds what a long session can spend.

**What a run may write (HA1).** For each field in the layout: empty, `default`, `integration` or
`ai`+`suggested` → may be written or replaced. `human`, `ai`+`accepted` → never. `ai`+`cleared` →
may be written only with a value different from the cleared one. A field not in the layout is
never written. **Replacing a `default` or `integration` value (owner card 1, 2026-11-12; R-IA1):**
the replaced value is kept with the suggestion (`ai_prior`); while that suggestion is unchecked and differs
from it, the field is a **conflict** the photographer resolves; a confirmed item waits for it (§7). The full write table is 08B
MC-A1. A run never deletes a value; when a follow-up no longer supports an earlier
suggestion it lowers that suggestion to `low` and says why.

**Field rules.** `choice` / `multi_choice`: only values from the definition's choices; no match →
not determined. `media_list`, `reference`, SKU, barcode: never written. Title and description:
not in this stage (§Scope ladder). Numbers measured from a photo (dimensions, weight): never
suggested from our photos; may be suggested from a matched listing, at most `medium`.

**Not determined.** No value is written; the trace's final step lists the field with the reason
("no label visible; the two closest listings disagree"). The form shows the field empty.

## 6. Evidence rule for attribution (owner 2026-11-12: "correct")

Attribution fields are the layout fields named in the setting `research.attribution_keys`
(initially the shop's designer and manufacturer keys; the owner may add period, model name).

| Confidence | Requires |
|---|---|
| high | a label, stamp or marking read in the item's own photos that names it; **or** a `match` (stage 4) with a listing on the owner's own shop |
| medium | `match` with two or more listings on different trusted sites that state the same value |
| low | one listing, or `similar` only, or untrusted sites only. Shown, never accepted by "Accept all" |

Complete order when several rows apply: high wins over medium over low; listings that disagree at
the same level → the value with more matching listings at `low`, and the disagreement stated in the
reason; a tie → not determined.

**Trusted sites (owner 2026-11-12).** Settings → Research → Trusted sites: the user adds a site
(a domain; its subdomains are included), removes it, and gives each a priority by ordering the
list (first = highest). Stored as the setting `research.trusted_sites`; the app ships with an empty
list and no site is built in (HA5). The connected shop's own domain is always trusted and always
first; it is not in the list and cannot be removed. Priority decides **reading order**: among the
candidates of a run, pages on trusted sites are read before any other, in list order, until the
run's page limit or cost cap. Priority does not change the evidence table: every trusted site
counts the same toward `medium`. Sites outside the list may be read after them and count only
toward `low`. A change to the list applies to runs started after it.

**Reading pages (owner correction A9, 2026-11-18).** The connected shop's own product pages are read through
Shopify's API, not its web pages (the shop's web pages answer automated reading with "too many requests"). A site
that refuses automated reading is asked once per run; its other pages are skipped with "Skipped: <site> is blocking
automated reading". The same product in several languages or collections is read once, and pages that only list
products (collections, categories) are not read.

Other (visible) fields — wood, colour, upholstery (owner 2026-11-12):
`medium` when the model reads it from the photos; `high` only when listing evidence that reaches
medium under the table above states the same (R-IA11); `low` when the model marks itself
unsure. Condition severities follow the same photo rule but are never raised by a listing: another
piece's listing says nothing about this one's wear (R-IA12). Full table: 08B MC-A4.

**Condition fields** (the layout's Condition step) are read from the photos the photographer
classified Condition. Main photos, detail shots included, are not used to set a condition field
(owner 2026-11-12: a detail shot is Main for its beauty even when it shows wear; Condition is the
photographer's manual choice). With no Condition photo at all, condition fields are not determined and the
agent never writes "None". With at least one Condition photo, the agent may write "None" for a part no
Condition photo shows (owner card 4, 2026-11-12: every flaw is photographed).

## 7. Publishing and the form

- Chapter 08 stands: values save as typed, a human edit makes the field human-owned, "Accept all"
  skips `low`.
- **Unchecked suggestions and publish (owner 2026-11-12, option B).** A publish sends `human`,
  `default`, `integration` and `ai`+`accepted` values, and `ai`+`suggested` values of `medium` or
  `high` confidence. An `ai`+`suggested` value of `low` confidence is never sent; the confirmation
  popover says "n low-confidence AI suggestions will not be published". The values sent are frozen
  at confirmation like every other (07A MC-10); a suggestion written after confirmation is not in
  that run.
- **Conflicts (owner card 1, 2026-11-12: "it should be a warning, and the job is stalled until resolved, but it
  doesn't block the others").** A conflict is a suggestion that replaced a preset default or a value from Shopify
  and is still unchecked. The confirmation popover shows each item's conflicts as a warning: the field, the
  previous value and its origin, the suggestion and its confidence, with Keep previous / Use suggestion / Edit.
  Confirm stays possible. A confirmed item that still has conflicts waits ("Waiting for you · n conflicts"),
  sending nothing to the shop, until every one is resolved; then it continues on its own. Other items of the same
  publish run normally. Each resolved field is sent as resolved, and everything else as frozen at confirmation. A
  conflict that appears after confirmation does not stall the item. Resolving is an owner action (undo step,
  feedback row). Chapter 08's "the form never blocks publishing" still holds. Exact rule: 08B MC-A10, MC-A12.
- Suggestions written by a run are not undo steps (they are the system's, like arrival); Looks
  right, Clear, Accept all and edits are, as today (MC-30).
- The form header shows "n to review" and opens the research panel (design mock-up).
  **Amended (owner, 2026-11-17, changelog A6):** the chip opens the **list of fields to review** (label, step,
  suggested value, confidence, conflicts marked); choosing one goes to its step, scrolls to it and opens its AI
  popover (Looks right / Clear, or the conflict's choices). Accept all and See research are in the list too.
  Source links (panel steps, popover sources) open in the default browser. A **queued** run says so in the panel:
  "Waiting — researching *name · SKU* first (*n*th in line)"; research still runs one session at a time. Pressing
  "Research this next" moves it to the front, and the panel then says it goes before the sessions marked arranged.
  The **sessions list** (beside "Arranged") and the **session header** show each session's research state:
  AI researching…, AI waiting · *n*th in line, AI · *n* to review, AI done, AI stopped, or AI can't run · *reason*
  (an arranged session whose research cannot start); in the header a tap opens Details.
  **Amended (owner, 2026-11-17, changelog A7):** in the panel each kind of step has its own icon, label and rail
  (never colour alone): Looked and Searched quiet; Read and Compared in the AI colour; **Concluded** stands out most
  (tinted, bold); a page that could not be read is a muted warning, "Could not read" with its link; Not determined
  dashed.

## 8. Domain model (additions; owner S unless stated)

- `research_runs`: id, item_id, session_id, trigger (`auto` / `manual` / `follow_up`), state
  (`queued` → `running` → `finished` / `failed` / `stopped_at_cap` / `cancelled`), stage reached,
  started_at, finished_at, cost (provider-reported usage, summed), settings snapshot (models,
  efforts, language), error.
- `research_steps`: run_id, seq, stage, kind (`looked` / `searched` / `read` / `compared` /
  `concluded` / `not_determined`), summary (one line, shown), detail (shown under it), source url
  and title (for `read`), photo ids used, field keys it led to. This is what the research panel
  shows, in order, live while the run is `running`.
- `research_outbound`: run_id, call_id, photo_id, provider, sent_at, bytes hash. The proof for HA3.
- `research_feedback`: item_id, key, suggested value, confidence, final value, action (`accepted`
  / `cleared` / `edited` / `kept_previous`), at, step_token. Written when the photographer acts on a
  suggestion; kept as evaluation examples. It trains nothing by itself (raw idea §7).
- `research_calls` (§10A) carries a state (`sent` / `answered` / `failed` / `unknown` / `not_sent`);
  `research_stage_outputs` holds each committed stage unit's output and input fingerprint (08B MC-A7).
- `jobs.kind` (the column's real name) gains `research`.
- `attribute_values` gains `ai_prior` (the replaced `default`/`integration` value, owner card 1);
  `ai_sources` stays the list of URLs the Decide stage relied on.
- `publish_items` gains `frozen_attribute_origin` (provenance, status, confidence and prior per key at
  confirmation; the evidence for MA8), `pending_conflicts`, `resolved_attributes` and `released_at`; its step
  gains `held`, before `prepare` (08B MC-A10).

Facts: steps, outbound records, feedback, run states, costs. Derived, never stored: "n to
review", "n fields filled", the panel's Finished label.

## 9. Models, effort, cost, language, secrets

- Each stage has its own model and reasoning-effort setting (`research.stages.<stage>.model`,
  `.effort`), so stages can be benchmarked and changed independently (owner 2026-11-12). Defaults
  are chosen by the first benchmark, not by this chapter.
- **Cost cap:** `research.max_cost_per_run`, default USD 0.50, and `research.max_cost_per_day`,
  default USD 15. Cost is computed from the usage each provider reports and a price table in
  settings. A run keeps back the price of its Decide stage (owner card 5, 2026-11-12): when only that
  reserve is left it stops reading, decides from what it has, and ends `stopped_at_cap` with those
  suggestions; it never starts a call once its cap is reached. The day cap stops new runs from
  starting (HA6). Exact rule: 08B MC-A8.
- **Language:** setting `research.language`, default English: reasons, trace text and free-text
  values are written in it. Choice values are the definition's own strings.
- **Secrets:** provider keys in the Keychain through the existing secrets port; never in logs,
  settings files or the repo (M39's rule).

## 10. Crash and repeat (H4)

Each finished stage's output is stored before the next starts. After a kill at any point, the run
resumes at the first unfinished stage and ends with the same suggestions and trace as an
uninterrupted run given the same provider answers; no step and no suggestion is written twice. A
provider failure is retried with backoff up to a fixed count, then the run ends `failed` with the
stage and error, and "Run again" is offered. Quitting the app pauses research; it resumes at
launch (accepted cost of decision C).

## 10A. Observability: the research record and its export (owner 2026-11-12)

The app is used on the work Mac and improved on the dev Mac. Everything needed to see where a run
went right or wrong is recorded on the work Mac and can be carried to the dev Mac as files. No
server, no telemetry service.

**Recorded for every provider call** (`research_calls`, owner S): run_id, stage, provider, model,
effort, the exact request sent (instructions and inputs, images by reference to the stored copy),
the exact response received, usage, cost, duration, attempt number, error. The reduced images sent
(HA3) are kept as files beside it. Kept for `research.record_days` (default 30, minimum 2), then removed;
`research_runs`, `research_steps`, `research_feedback`, `research_outbound` and the stage outputs are kept
(R-IA26).

**Export.** Settings → Research → **Export research**, for a date range or chosen items, writes one
folder (zipped) holding, per run: the run and its steps; every call as above; the images sent; the
form snapshot the run started from; the values it wrote; and what the photographer then did with
each (accepted, cleared, edited to what: `research_feedback`), plus the settings snapshot. Plain
JSON and JPEG, one sub-folder per item, readable without the app. **It never contains a secret, a
RAW, or a photo that is `deleted` at export time.**

**On the dev Mac.** A command-line tool in the repo reads an export and prints, per stage and
overall: suggestions accepted unchanged, edited, cleared, not determined; attribution right or
wrong against the photographer's final value; cost and time. It can **replay** one stage of an
exported run from its recorded inputs with a different model, effort or instructions and show the
difference, without touching the work Mac or its data.

**Only if cheap:** an export-folder setting that writes each finished run's export automatically
(pointed at a synced folder, the files reach the dev Mac with no hand carrying).

## 11. Measurement ledger (proposed; ids `MA…` until ratified into chapter 14)

| ID | Observable outcome, measured true ⇒ shipped | Defect family it guards |
|---|---|---|
| **MA1** | After any run, every field that was `human` or `ai`+`accepted` before it holds the same value and provenance; no cleared value came back. | checked values overwritten |
| **MA2** | Every attribution value written by a run has the confidence the §6 table (08B MC-A4) gives for the evidence recorded in its steps, except a value a later run lowered to `low` because it lost its support (the reason says so); none is above `low` without a read label or the required matching listings. | false designer or maker claims |
| **MA3** | Every image sent to a provider has a `research_outbound` row; none belongs to a `deleted` photo; no RAW or master bytes are sent. | photos leaking |
| **MA4** | Kill-and-resume at any point of a run ends with the same values, confidences and steps as an uninterrupted run with the same provider answers; nothing is duplicated. | half-finished or doubled research |
| **MA5** | No run's recorded cost exceeds its cap by more than the one call in flight; no run starts after the day cap is reached. | runaway spend |
| **MA6** | Every value a run writes is in the item's layout and valid for its definition (a listed choice for choice fields); every `ai` value has at least one step naming its key. | invalid values, unexplained suggestions |
| **MA8** | After a publish, the shop holds no value that was `ai`+`suggested` at `low` confidence (with no replaced value) at confirmation, and no unchecked suggestion that replaced a preset default or a value from Shopify; an item confirmed with conflicts issued no shop change before its last conflict was resolved, and sent each conflicted field as resolved; other items of that publish were not delayed (owner card 1, 2026-11-12). | unchecked weak guesses or replacements on a live listing |
| **MA9** | An export contains, for every run in its range, every step, every provider call with its exact request and response, every image sent, the values written and the photographer's action on each; replaying a stage from it sends the same inputs the original call sent, for every call whose images are in the export and whose record has not expired (any other call is reported not replayable). It contains no secret, no RAW and no `deleted` photo. | a wrong suggestion that cannot be explained or reproduced; secrets or deleted photos leaving in an export |
| **MA7** | With no key, no network or no integration, sessions, review, export and publish behave as on `main`; the panel states why research is not running. | core workflow coupled to AI |

Operational measurement (recorded, not a test): on a set of owner-verified items, share of
attribution suggestions at medium or high that are correct, share left not determined, cost and
time per item, share of suggestions accepted unchanged.

## 12. Scope ladder

**Must ship (this chapter):** stages 1–7; automatic and manual runs; follow-up runs; the live
research panel from stored steps; cost caps; settings for models, language and the trusted-sites list with priorities; feedback
records; the call record, Export research, and the dev-Mac report and replay tool (§10A).

**Only if cheap:** cancel a running run from the panel; per-field "research this field again";
showing the cost of a run in the panel.

**Deferred, each its own later chapter:** publication assistant (titles, descriptions, SEO,
batch); session photography assistant (shot templates, slot assignment, missing and duplicate
shots, framing suggestions as 10D composition values).

**Non-goals:** Temporal, Python, PostgreSQL, object storage, tenants; a similarity index of the
shop's images; fine-tuning; any camera setting; the agent adding fields to a layout or new choices
to a definition; calibrated probabilities.

## 12A. Before any plan (pre-implementation protocol)

1. **Provider benchmark.** On the owner's 20–30 known pieces: Lens through a vendor against Cloud
   Vision Web Detection (is the right listing in the first 10 results), and the candidate models
   per stage. Result recorded in `evidence/research_providers.md`; the owner picks the visual
   search adapter from it. The plan builds one adapter; the port keeps the other possible.
2. **Page-reading probe** (evidence doc, probe 1): can the trusted sites be read, and by which route.
3. The other probes listed in the evidence doc.

## ⚠ OWNER DECISIONS REQUIRED (0, plus two inputs)

None open.

Inputs owed by the owner (collecting): the first trusted sites (entered in the setting); 20–30 already-researched items.
The benchmark of §12A waits for them; ratification does not.

## Shaping changelog

1. **2026-11-12 — First grounded draft.** From the raw idea and the owner's answers of the day.
   Resolved by the owner: engine in-app and service-ready (C); product facts first; photos may go
   to a cloud provider, deleted never; automatic start with a switch; evidence rule of §6; OpenAI
   as first model provider with per-stage model and effort; English by default with a language
   setting. Resolved by the shaper from the repo: the raw idea's five contracts reduce to the three
   data shapes of §4 because form, keys and provenance already exist; "preserve field keys" is
   already how chapter 08 works; Temporal's durability is covered by the existing job queue and
   H4; the raw idea's workflow states become run states plus stages. Dropped as non-goals:
   Temporal, Python, PostgreSQL, tenants, pgvector.
2. **2026-11-12 — Owner round 1.** Quiet period becomes a setting, default 1 s (owner: an
   experienced photographer makes no test frames; a learning one raises it). Unchecked suggestions
   publish at medium and high, never at low (option B; §1, §7, new MA8). Visual search provider is
   chosen by a benchmark of both candidates (option A; §12A). Visible-field confidence as proposed
   (option A). Owner's note on condition: Condition is the photographer's manual classification,
   detail shots stay Main; folded into §6 as a shaper reading awaiting confirmation. Evidence doc
   `evidence/research_providers.md` added from the web research.
3. **2026-11-12 — Owner round 2.** Confirmed: with no Condition photo, condition fields stay not
   determined (never "None"). Status set to READY_FOR_RATIFICATION; ratification surface presented
   in chat.
4. **2026-11-12 — Owner round 3: observability.** Owner: the app runs on the work Mac and must be
   analysed and improved from the dev Mac, perhaps by exporting content. Added §10A (call record,
   Export research, report and replay tool), `research_calls`, MA9; status back to COLLABORATING.
5. **2026-11-12 — Owner round 4.** §10A approved: 30-day call record, replay tool in must-ship.
   Status READY_FOR_RATIFICATION; surface of round 2 plus §10A and MA9 presented in chat.
6. **2026-11-12 — RATIFIED by the owner, David Loorenz** ("after that modification I approve /
   ratify this intention"). Modification folded first: trusted sites are a setting the user edits,
   with priorities (§6). Shaper reading recorded with it: priority is reading order and does not
   change the evidence table. Surface presented: the outcome; measurements MA1–MA9 with the defect
   family of each; what ships and the non-goals; observability (§10A); no open decisions; two
   deferred inputs (trusted sites, test set) and the provider benchmark of §12A. Owner's note on
   process: the faster test runs being built on `main` are merged into this branch before
   implementation starts.
7. **2026-11-12 — Post-ratification amendments by owner cards** (mechanism inventory, chapter 08B; owner
   answers relayed by the orchestrator). Card 1: suggestions over a preset default or a Shopify value keep the
   replaced value and are conflicts the photographer resolves at publish (§5, §7, §8, MA8); the exact reading
   awaits the owner's confirmation (ratify R-IA1). Card 2 B: each photo looked at once, Compare uses the cover
   (§5 stage table and follow-up examples). Card 3 B: research images are cut from the adjusted preview (§5;
   H3 amended in chapter 01). Card 4 A: with Condition photos, "None" may be set for a part no Condition photo
   shows (§6). Card 5 B: the Decide reserve (§9). Inventory deltas folded with them: §3 `ai_suggest` fact; §6
   visible-field wording and condition never raised by listings (ratify R-IA11, R-IA12); §8 `jobs.kind`, new
   columns and tables; §10A what is kept (ratify R-IA26); MA2 and MA9 wording. Contracts: 08B. Status line
   unchanged (RATIFIED); R-IA6…R-IA26 await the owner's ratification as a list.
8. **2026-11-12 — Owner follow-up** (relayed by the orchestrator). Card 1 corrected: conflicts are a warning at
   confirmation, and a confirmed item with conflicts waits until they are resolved without holding up the others
   (§7, §8, MA8; 08B MC-A10). Inventory resolutions R-IA6…R-IA26 ratified as a list; their markers are now
   `(R-IAn)` traces. Trusted-site priority stays reading order only (§6), as the orchestrator's choice by the owner's
   delegation. Status line unchanged (RATIFIED).
