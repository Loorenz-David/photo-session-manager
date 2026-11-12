# 08B — Mechanism contracts: research agent (MC-A1…MC-A12)

**Status: CONTRACTS RATIFIED** (owner David Loorenz, 2026-11-12; changelog entry 3). Written by the mechanism
inventory, 2026-11-12, on branch `feature/agentic` at `acd0323`. Chapter 08A (RATIFIED 2026-11-12) is the
authority for meaning; this chapter is the authority for the exact rule of each mechanism 08A relies on.
Conventions are as in `05A_mechanism_contracts.md`: "canonical" is the one stored and compared form, and
comparisons are byte-for-byte on canonical forms unless a contract says otherwise. Ids are `MC-A…` (the plain
`MC` numbers continue on `main`), and invariants are registered in chapter 14 as `MA10…`. Markers `(R-IAn)` trace
a rule to its owner card (R-IA1…R-IA5) or to the inventory resolution the owner ratified as a list
(R-IA6…R-IA26). **Shapes owned by an external service are not specified here**; each carries a
`RESEARCH-VERIFY-n` probe id (list at the end, charter rule 17).

## ⚠ OWNER DECISIONS REQUIRED (0)

None open. Cards 1–5 were answered by the owner on 2026-11-12. Card 1 was answered in the owner's own words:
"it should be a warning, and the job is stalled until resolved, but it doesn't block the others" (MC-A10). Cards 2
B, 3 B, 4 A and 5 B are folded into the contracts. R-IA6…R-IA26 were ratified as a list the same day ("the smaller
choices correct"). Changelog entries 2 and 3.

---

## Inventory (ranked by silent-failure risk)

| Rank | Mechanism | Contract | If subtly wrong… | Status |
|---|---|---|---|---|
| 1 | Confidence derivation for attribution, visible, measured and condition fields (§6 as a total function) | MC-A4 | false designer/maker claims published unchecked at medium, forever | defined; card 4 answered (A) |
| 2 | Write permission per stored state and the cleared set (HA1) | MC-A1 | checked values overwritten; cleared values return after a second clear | defined; conflicts per card 1 |
| 3 | Which values a publish sends (low withheld; conflicted fields frozen at their resolution, the item held until then) against the frozen confirmation | MC-A10 | weak guesses or unchecked replacements reach the live listing; a held item mutates early or never resumes | defined; card 1 answered (warning + held item) |
| 4 | Outbound image record, deleted/RAW exclusion, delete-during-run race | MC-A5 | a deleted photo leaves the Mac; no proof of what left | defined; card 3 answered (B) |
| 5 | Site identity: host and domain matching, the shop's hosts, page identity, own listing, reading order | MC-A3 | one dealer counted as two sites → false medium; the item's own listing used as evidence of itself | defined |
| 6 | Cost: units, price table, integer money, unreported usage, the cap check, the day boundary | MC-A8 | silent overspend; the day cap computed on the wrong day | defined; card 5 answered (B) |
| 7 | Stage units, input fingerprints, persistence, resume and reuse (H4) | MC-A7 | doubled steps and cost after a crash; follow-ups redo everything or miss changes | defined; card 2 answered (B) |
| 8 | Model output → stored value: choices, name normalisation, numbers and units | MC-A2 | invalid choices, "same value" mis-decided, coerced numbers | defined |
| 9 | Provider call record, retries, error classes, secret redaction | MC-A9 | a key written into the record or the export; retries that burn money on a dead key | defined; shapes RESEARCH-VERIFY |
| 10 | Retention, export contents, replay input identity | MC-A11 | export leaks a deleted photo or a secret; a replay that silently differs | defined |
| 11 | Run triggers, quiet period, queue, day-cap wait, offline, quit/resume | MC-A6 | runs on test frames, parallel runs past the cap, runs that never start | defined |
| 12 | Feedback rows and undo/redo around research writes | MC-A12 | undo silently erases a newer suggestion; evaluation data says "cleared" for an undone clear | defined |
| — | External shapes: OpenAI responses, usage and errors; visual search results; page reader; Shopify shop hosts and storefront URL | RESEARCH-VERIFY-1…11 | a fixture agrees with the code and with nothing else | probes owed |

Considered and left without a contract (trivial or already contracted): `research.language` (a string passed to
instructions); "Accept all" skipping `low` (exists, `accept_all`); value canonical forms (05A MC-9, reused);
"n fields filled" and the Finished label (derived for display only, never compared).

---

## MC-A1 — What a run may write (HA1) and the cleared set

**Inputs:** the stored `attribute_values` row for `(item_id, key)` **read inside the writing transaction**; the
definition and the item's current layout (presets of its item type, all three steps), also read in that transaction;
the cleared set (below); the proposed value already mapped by MC-A2.

**Fields never written by research** (checked first; the trace's last step names the field as "not researched"):
a key not in the current layout; any `product`-step field (title, description, tags and every other Product
field: the publication assistant's, deferred) (R-IA6); core types `media_list`, `reference`,
`rich_text`, `money`, `rating` (R-IA7); a definition with `writable = false`; `product.title`; SKU and
barcode (item fields, never attribute rows).

**Write table** (a row is *live* when `ai_status ≠ cleared`; the provenance set is the code's: `human`, `ai`,
`default`, `integration`):

| Stored row at write time | A run may |
|---|---|
| none | write |
| `default`, live | write, saving the replaced row as `ai_prior` (below) (R-IA1) |
| `integration`, live, valid or not | write, saving the replaced row as `ai_prior` (below) (R-IA1) |
| `ai` + `suggested` | replace with any value, keeping its `ai_prior` unchanged; or lower (below) |
| `ai` + `accepted` | never |
| `human`, live | never |
| `human` + `cleared` | never: a human clear is a human decision (R-IA8) |
| `ai` + `cleared` | write only a value not in the cleared set |
| `default` + `cleared`, `integration` + `cleared` | write only a value not in the cleared set (R-IA8) |
| any other combination (e.g. `ai` with no status) | never (fail closed) |

**Prior value and conflict (owner card 1, 2026-11-12) (R-IA1).** `attribute_values` gains `ai_prior`
(JSON `{value, valid, provenance}` or null). When research writes over a live `default` or `integration` row, that
row's value, validity and provenance are stored there. When research replaces its own suggestion, the prior is
kept; when the photographer acts on the field (MC-A12), it is dropped. A row is a **conflict** when it is `ai` +
`suggested` (any confidence), has an `ai_prior`, and not `S(value, ai_prior.value)`. A suggestion with no prior is
never a conflict. A conflict is resolved only by the photographer, at publish confirmation or in the form (MC-A10,
MC-A12).

**Cleared set** `C(item, key)` = the value of the stored row when it is cleared ∪ the `suggested value` of every
`research_feedback` row for `(item, key)` with action `cleared` or `kept_previous` (MC-A12). It grows. Clearing X, getting Y, and
clearing Y never lets X return (R-IA8). Membership uses **sameness** `S(a, b)`:

| Core type | `S(a, b)` holds when |
|---|---|
| text, textarea, url | `N(a) = N(b)` (MC-A2) |
| choice | canonical equality |
| multi_choice, tags | equal as sets (choice: canonical strings; tags: `N` keys) |
| dimension, weight | equal after conversion to one unit (MC-A2 numbers) |
| everything else | canonical equality (05A MC-9) |

A proposed value in `C` is treated as **not determined**, with the reason "matches a value you cleared". A
proposed value `S`-equal to a live `default` or `integration` row's value is not written: the field already
holds it.

**Lowering.** If a run decides *not determined* for a key whose row is `ai` + `suggested`, it writes the same
value with `ai_confidence = low`, keeps `ai_sources`, and sets `ai_reason` to "No longer supported: <reason>".
It never deletes a row, and it never writes a status other than `suggested` or a provenance other than `ai`.

**Code path.** `AttributesService::ai_suggest` applies this table inside its transaction. Today it refuses only
`human` (live or cleared) and **does not refuse `ai` + `accepted`**. The contract requires that refusal and
the cleared-set refusal. The existing same-content check stays: a write equal in value, confidence, status, reason
and sources is not written (idempotency, MC-A7).

**Invariant (MA10).** For every stored state of the table and every research write, the row changes only where
the table allows. No value in `C(item, key)` is ever written by research. No never-written field is ever written.

---

## MC-A2 — From model output to a stored value

**Name key `N(s)`** (version `RESEARCH_NORMALISE_VERSION = 1`, stored in `research_runs.settings_snapshot`):
NFKC → Unicode full case fold → every character of general category P\* or S\* replaced by a space →
whitespace runs collapsed to one space → trimmed. Letters with diacritics are kept (`Børge` ≠ `Borge`). No alias
table: `Hans J. Wegner` and `Hans Wegner` are different keys. Extraction instructions ask the model to answer
with the definition's own choice string when one names the same thing.

**Mapping by core type** (anything that fails gives *not determined* for that key, recorded in the trace):

| Core type | Accepted model output | Stored (05A MC-9 canonical) |
|---|---|---|
| choice | string | the unique choice byte-equal to it; else the unique choice `c` with `N(c) = N(s)`; zero or several → fail |
| multi_choice | array of strings | each element mapped as for choice; unmatched elements are dropped and named in the trace; result in the definition's choice order; empty → fail |
| tags | (Product step: never written, R-IA6) | — |
| text, textarea | string | MC-9 canonical; empty after trim → fail |
| url | string | absolute `http`/`https` URL, else fail |
| integer, decimal | JSON number or a string of digits with optional `.` | MC-9 canonical; exponent, thousands separators or a decimal comma → fail |
| dimension, weight | `{value, unit}` with unit in `mm, cm, m, in` / `g, kg, lb` | converted (1 in = 25.4 mm, 1 lb = 453.59237 g, exact decimals) into the definition's first listed unit and rounded half-even to one decimal place. The adapter maps these seven unit names to its own codes; a unit it cannot map → fail |
| boolean | `true` / `false` | as is |
| date | `YYYY-MM-DD` that exists in the calendar | as is |
| color | `#rrggbb` | lowercase |

**What a listing states.** Stage 3 returns, per page and per researched key, `{stated: text as on the page,
value: mapped value or null}`. A page that states two different values for one single-valued key states nothing
for it. A statement whose `value` is null (it names something that is not one of the choices) still counts in
the disagreement counts of MC-A4 as its own value `u(N(stated))`, but it can never be written (R-IA24).

**Invariant (part of MA11).** Every research value stored is valid for its definition at write time, and a choice
is byte-equal to one of the definition's current choices.

---

## MC-A3 — Sites, hosts and reading order

**Host `H(url)`.** The URL must be absolute `http`/`https`, else the candidate is discarded. The host is
lowercased, converted with IDNA to ASCII (punycode), stripped of a trailing dot, with the port ignored. IP-literal
hosts are never trusted.

**Trusted entry.** The user may type a domain or a URL. Stored form: `H` of it with one leading `www.` removed. It
must contain a dot. It is refused when it equals, is a parent of, or is a subdomain of an existing entry or a shop
host ("already covered by …"). Entries are therefore disjoint. `research.trusted_sites` is the ordered list;
priority is the index (0 first). A host `h` **matches** entry `e` iff `h = e` or `h` ends with `"." + e`.

**Shop hosts.** The connected shop's public hosts as the commerce port reports them: primary domain, other domains
and the platform domain (new port method; shape RESEARCH-VERIFY-9). They are normalised by `H` and snapshotted at
run start. A host matches the shop when it equals a shop host or is one of its subdomains.

**Site of a page.** Use the URL whose content was actually read: after redirects when the reader reports it
(RESEARCH-VERIFY-8), else the candidate URL. The first match wins:

| Site class | Identity |
|---|---|
| `shop` (matches a shop host) | `shop` |
| `trusted` (matches entry `e`) | `e` (at most one, by disjointness) |
| `untrusted` | `h` with one leading `www.` removed |

**Page identity `U(url)`** (dedupe): scheme lowercased, host `H`, default port dropped, empty path → `/`, fragment
dropped. Query parameters named `utm_*`, `srsltid`, `gclid`, `fbclid`, `mc_cid` or `mc_eid` are dropped; the
rest are sorted by name, then value. Candidates with equal `U` merge and keep the best rank.

**This item's own listing** is excluded from reading and from evidence, and the trace records it as "this item's
own listing". That covers a page whose `U` equals the item's storefront URL, or whose shop-host product handle
equals the item's handle (RESEARCH-VERIFY-9). Otherwise the item would be evidence for itself (R-IA9).

**Priority** is reading order only; every trusted site counts the same as evidence (08A §6, kept by the
orchestrator by the owner's delegation, 2026-11-12).

**Reading order.** First `shop` pages by provider rank; then `trusted` pages by entry priority, then by rank; then
`untrusted` pages by rank. Ties on rank break by `U` ascending (byte order). Pages are read in that order until
`research.max_pages` pages have been **obtained** (default 10, adjustable by the benchmark), or until the cost
rule of MC-A8 stops reading. A page that cannot be obtained is recorded as a `read` step "could not be read" and
does not count (R-IA10). The trusted list, shop hosts and `max_pages` are part of the run's settings
snapshot. Edits apply to runs that start later (08A §6).

**Invariant (part of MA11).** Two pages count as different sites only if their site identities differ. A page on
a subdomain of an entry counts as that entry. The item's own listing never counts.

---

## MC-A4 — Confidence derivation (§6 as a total function)

Value **and** confidence are computed by a pure core function over the stored stage outputs (MC-A7). A model never
sets a confidence. Its only inputs here are values, verdicts, statements, and a `sure` flag on photo reads.
`ai_reason` is model-written text in `research.language`. `ai_sources` = the `U` of the listing items supporting
the written value at the decided level, in reading order.

**Field class** (the first match wins): 1 never-written (MC-A1) → skip · 2 **condition** (the definition's group
is `condition`) · 3 **attribution** (key ∈ `research.attribution_keys`) · 4 **measured** (core type `dimension` or
`weight`) · 5 **visible** (everything else).

**Evidence items for key k:**
- *label item:* a stage 5 reading on photo p that names value v for k. Its source is photo p.
- *listing item:* a page read in stage 3 that states v for k (MC-A2), with its stage 4 verdict and its site (MC-A3).
  Only verdicts `match` and `similar` count. `different` and not-compared pages count for nothing.

**Levels for a value v:**
- **high(v):** a label item for v, or a `match` listing item on site `shop` stating v.
- **medium(v):** `match` listing items stating v on ≥ 2 distinct identities among `shop` and the trusted entries.
- **low(v):** ≥ 1 listing item (`match` or `similar`, any site) stating v.

**Support at a level:** high = distinct photos with a label item plus 1 if there is a shop `match`; medium =
distinct trusted-or-shop identities with `match`; low = distinct site identities of any class.

**Attribution decision.** `top` = the highest level at which some value qualifies. If exactly one value qualifies
there, it is written at that level. If several qualify, the unique value with the greatest support at `top` is
written at `low`, and the reason names the others. A tie at the greatest support → not determined. No level →
not determined. An unmapped value `u(…)` that wins → not determined, and the reason names it. Values that
qualify only below `top` are named in the reason and do not change the result (R-IA24).

| # | Evidence for k (enumerated rows; one exact outcome each) | Written |
|---|---|---|
| 1 | label names A | A, high |
| 2 | shop `match` states A | A, high |
| 3 | shop `similar` states A, nothing else | A, low |
| 4 | `match` on trusted e1 and e2, both A | A, medium |
| 5 | `match` on e1 twice (two pages, both A) | A, low |
| 6 | `match` on e1 = A, `match` on an untrusted site = A | A, low |
| 7 | label A; `match` on e1, e2 = B | A, high (B named) |
| 8 | label A; label B (two photos) | not determined (tie 1–1) |
| 9 | `match` e1, e2 = A; `match` e3, e4, e5 = B | B, low (3 > 2) |
| 10 | `match` e1, e2 = A; `match` e3, e4 = B | not determined |
| 11 | `similar` on 2 untrusted sites = A, `similar` on 1 = B | A, low |
| 12 | `match` e1 = A (unmapped), `match` e2 = A (unmapped), nothing else | not determined (the reason names A) |
| 13 | only `different` verdicts | not determined |
| 14 | nothing | not determined |

**Measured fields** (`dimension`, `weight`): never from our photos (08A §5). The attribution function runs over
listing items with verdict `match` only, then the result is capped at medium (high → medium). Sameness is
`S` of MC-A1.

**Visible fields:** v = the model's photo read (stage 1, or stage 5 for printed text), with `sure`. `w` = the
attribution function run over listing items only (R-IA11):

| Photo read | Listing result `w` | Written |
|---|---|---|
| v, sure | `w` at high or medium and `S(v, w)` | v, high |
| v, sure | `w` at high or medium, not `S(v, w)` | v, low (the reason names w) |
| v, sure | `w` at low, or none | v, medium |
| v, unsure | any | v, low |
| none | `w` at any level | w, low |
| none | none | not determined |

**Condition fields:** read only from stage 6 over the photos classified `condition`. With no Condition photo,
every condition field is not determined. A sure read → medium; an unsure read → low. Listings never raise a
condition field, because another piece's listing says nothing about this one's wear (R-IA12). With at
least one Condition photo, stage 6 may return `None` for a part no Condition photo shows, at medium when sure
(owner card 4 = A, 2026-11-12: every flaw is photographed). With no Condition photo at all, "None" is never
written (08A §6).

**Invariant (MA11).** For every enumerated evidence row above, the written value and confidence are that row's
single outcome. For attribution and measured fields, no research value is above `low` without a label item, a
shop `match`, or `match` items on two distinct trusted identities stating it; visible and condition fields follow
their own rows above (a sure photo read is `medium`).

---

## MC-A5 — Research images and the outbound record (HA3)

**Eligible photo**, evaluated in the transaction that records a call: the row is in the run's session, that
session's item is the run's item, and its classification is `main`, `condition` or `unreviewed`. A `deleted`
photo is never eligible, nor is a photo moved to another session since.

**Source** (owner card 3 = B, 2026-11-12; H3 as amended in chapter 01): the photo's adjusted preview for its
current `adjusted_gen`, framed by the 10D MC-27 pixel rule for its current composition revision (turn, flip,
straighten, cut). No master is rendered for research. It is resized to the stage's `research.stages.<stage>.image_long_edge` and never upscaled. Defaults: 1024 for
stages 1, 2 and 4; 2048 for stages 5 and 6 (R-IA13). Encoding: JPEG quality 85, sRGB, **all metadata
removed** (no EXIF, GPS, serial number or maker notes) (R-IA13). A photo with no adjusted preview yet
has no source (MC-A6 readiness); the original camera preview is never used.

**Type guard (rule 11).** The model and visual-search ports accept images only as a `ResearchImage` value, which
only the research-image renderer can construct from an eligible photo. No constructor takes a path or raw bytes,
so RAW and master bytes cannot be passed to a provider port. Within one run there is one stored copy per
`(photo id, source generation, composition revision, long edge)`, named by its SHA-256, and every call of the run
sends that copy.

**Order per call carrying photos.** (1) One transaction: re-read eligibility and drop photos that are no longer
eligible (the step detail says "photo removed since"); check the cost rule (MC-A8); insert the
`research_calls` row with state `sent`; insert one `research_outbound` row per photo (`run_id`, `call_id`,
`photo_id`, provider, `sent_at`, SHA-256 of the bytes); commit. (2) Only then send. A deletion committed before
(1) is never sent. A deletion committed after (1) may already be on its way, and it is excluded from every later
call. Third-party images fetched from listings are stored with the call record but are not outbound rows.

**Invariant (MA12).** The photos in every recorded provider request are exactly that call's outbound rows. Each
was eligible in the transaction that committed them, and that transaction precedes the send. No bytes reach a
provider port except as a `ResearchImage`.

---

## MC-A6 — When runs exist and start

**Scope.** A run belongs to one session (`research_runs.session_id`). "Research has not run for it yet" (08A §5)
means no `research_runs` row exists for this session. The Unassigned session (no item) is never researched
(R-IA23).

**Prerequisites** (MA7). The panel states the first one that fails, in this order: no integration configured;
no model key; no visual-search key; a configured provider or model missing from the price table (MC-A8); no
field in the layout that research may write (MC-A1); day limit reached; offline; `research.auto` off ("Research"
button shown). While any of the first five fails, no run row is created. Eligibility is evaluated on every photo
change of the session and on every save of a research setting or key.

**Rows.** For each session there is at most one run in state `running` and at most one in `queued` (two partial
unique indexes).

| Event | No queued or running run | A run is queued | A run is running |
|---|---|---|---|
| Session reaches ≥ 1 Main photo, no run row ever, `research.auto` on | create `queued`, trigger `auto` | — | — |
| Photo-derived input change (MC-A7 trigger set) after the latest run, `research.auto` on | create `queued`, trigger `follow_up` | keep it; its start moves (quiet period) | create `queued` `follow_up` unless one is queued |
| **Run again** | create `queued`, trigger `manual` | it becomes `manual` | create `queued` `manual`, or convert the queued follow-up |
| `research.auto` switched off (R-IA25) | — | queued `auto`/`follow_up` → `cancelled` | the running run finishes |

**Quiet period.** A queued `auto` or `follow_up` run may start when `research.quiet_seconds` have passed on the
monotonic clock (05A MC-4) since the session's last change. A change is a photo row created, a classification,
position or composition change (an MC-6 `change_seq` bump), or a photo's research source becoming available. At
launch, the last change is the launch instant. `manual` runs skip the quiet period.

**Readiness** (R-IA14). A queued `auto`/`follow_up` run does not start while an eligible
photo's source (MC-A5) is still missing, up to `research.source_wait_seconds` (default 120). After that wait it
starts without those photos, and their arrival is a change that queues a follow-up. If no eligible photo has a
source when the wait ends (Capture One closed), the run stays `queued` and the panel says "Waiting for Capture
One"; the camera's own preview is never used instead (owner card 3).

**Start order** (R-IA15). At most **one** run is `running` app-wide, so there is one research call in
flight (MA5) and the day spend is bounded (MC-A8). Among startable queued runs, `manual` goes first, then
`queued_at` ascending, then id ascending. The start transaction sets `running` and `started_at`, and records the
settings snapshot and the form snapshot.

**Day limit** (R-IA16). A queued run does not start while today's spend (MC-A8) is at or above
`research.max_cost_per_day`. It stays `queued`, the panel says "Day limit reached", and it may start from the
next local date.

**Offline.** A call that fails before any request byte is sent (MC-A9 class `offline`) holds the run in
`running` without counting an attempt. The panel says "Offline", and the call is retried with the jobs back-off
capped at 60 s.

**Quit and launch.** A `running` run stays `running` in the database. At launch the jobs runner re-queues it
(`requeue_running_jobs`), and it resumes per MC-A7. The work is a `jobs` row with `kind = research` (the column
is `kind`, not `type`), run by a dedicated runner (`JobRunner::only_kind("research")`) so a long run never
blocks preview or publish jobs.

**Invariant (MA13).** Never more than one `running` research run app-wide, nor more than one `queued` per
session. No `auto` or `follow_up` run starts within the quiet period of its session's last change. No run starts
while today's spend is at or above the day cap.

---

## MC-A7 — Stages, units, fingerprints, persistence, resume and reuse (H4)

**Units** (the unit of persistence; owner card 2 = B, 2026-11-12: each photo is looked at once, Compare uses the cover):

| Stage | Unit | Unit input identity |
|---|---|---|
| 1 Observe | one per eligible `main`/`unreviewed` photo | photo id, composition revision, long edge, model, effort, instructions version, language. Output includes `shows_marking: bool` |
| 2 Find matches | one | cover photo id (main position 1) and its composition revision; visual-search provider |
| 3 Read candidates | one per page read (MC-A3 order) | `U`, page-reader route, model, effort, instructions version, the researched keys with their definitions and choices |
| 4 Compare | one per page with images | `U`, SHA-256 of its stored images, cover identity |
| 5 Read labels | one per photo with `shows_marking` | photo identity, model, effort, instructions version |
| 6 Read condition | one | sorted ids and revisions of the `condition` photos, the condition definitions and choices |
| 7 Decide | one | fingerprints of every unit output used, the layout keys with definitions, `attribution_keys` |

**Fingerprint** `F(unit)` = SHA-256 (lowercase hex) of canonical JSON of
`{"v": RESEARCH_FINGERPRINT_VERSION, "stage": n, "unit": <identity fields>}`. Canonical JSON: objects with keys
sorted by UTF-8 byte order, no insignificant whitespace, strings NFC, integers as JSON integers, no floats
(decimals as MC-9 strings), arrays in the stated order. `RESEARCH_FINGERPRINT_VERSION = 1`; a change to any
identity field list bumps it.

**Storage.** `research_stage_outputs`: `run_id`, `stage`, `unit_key`, `fingerprint`, `output` (JSON),
`reused_from_run_id`, `committed_at`, unique `(run_id, stage, unit_key)`. These and the run's writes (stage 7
output: per key the value, confidence, reason, sources and the MC-A1 outcome) are kept with the run.

**Commit rule.** When a unit's last call returns, one transaction commits its output, its trace steps and its
calls' responses. Steps take `seq = max(seq of the run) + 1` inside that transaction, so seq is gapless and in
commit order. The panel shows committed steps only. Stage 7's attribute writes (MC-A1) and its `concluded` and
`not_determined` steps commit together. Every written key is named by at least one step (MA6).

**Resume.** On launch or retry the run continues at the first stage that has an uncommitted unit. A committed
unit never runs again. A call row still `sent` with no response is closed as `unknown` (charged per MC-A8) and
is re-issued as the next attempt. Stage 7 re-applies its writes through MC-A1, and the same-content check makes a
repeat a no-op.

**Reuse.** In an `auto` or `follow_up` run, a unit whose fingerprint equals that of a committed unit in the
latest earlier run of the same session is copied, with `reused_from_run_id`, no call and no step. The earlier
unit's output must still be complete, meaning every file it references is present (MC-A11 retention). A
`manual` run reuses nothing (R-IA18).

**Follow-up trigger set** (R-IA17). A follow-up is queued when, compared with the latest run's
snapshot, any of these changed: the set of stage 1 unit identities, the cover identity, or the stage 6 Condition
identity. This includes 08A's list: added photos, photos
reclassified to or from `deleted`, recomposed photos. It also includes main↔condition reclassification and a new
cover by reorder. It excludes Capture One retouches and settings changes: a setting applies to the next run that
happens, and never starts one.

**Invariant (MA14).** Killing and resuming at any instant ends with the same stage outputs, steps and attribute
writes as an uninterrupted run given the same provider answers. No committed unit is re-called, and no step
appears twice. A follow-up calls providers only for units whose fingerprint has no committed equal.

---

## MC-A8 — Cost, caps and the day

**Money** is an integer number of **micro-USD** (`i64`). Caps and prices in settings are MC-9 decimal strings
with at most 6 fractional digits, parsed exactly, never as floats.

**Price table** `research.prices`: rows `{provider, model ("" for non-model providers), unit, usd_per_million}`
plus, per `(provider, model)`, `fallback_call_usd`. Unit names are what the adapters report: `input_token`,
`cached_input_token`, `output_token`, `web_search_call`, `image_search_call`, `page_fetch` (exact usage fields:
RESEARCH-VERIFY-1, -6). Snapshotted at run start.

**Call cost** = `ceil( Σ_u usage_u × p_u / 10^6 )` micro-USD, with `p_u` = `usd_per_million × 10^6` as an
integer, computed in `i128`. A call whose usage is not reported (a timeout after send, a dropped connection after
send, `unknown` on resume, an error reply without usage) costs `fallback_call_usd` (R-IA19). A call
never sent (`not_sent`, class `offline`) costs 0. A reported unit that is missing from the table is charged the
fallback and logged. `research_runs.cost` is the sum of its calls, updated in the transaction that records each
reply.

**Stop rule** (owner card 5 = B, 2026-11-12; checked in the transaction of MC-A5 step 1, before every call):
with `reserve` = `fallback_call_usd` of the stage 7 model, a call of stages 1–6 is made only if
`run.cost + reserve < max_cost_per_run`. Otherwise the run skips its remaining reading units and goes to Decide on
the evidence committed so far. Decide is made if `run.cost < max_cost_per_run`, and the run ends
`stopped_at_cap` with its writes. If `run.cost ≥ max_cost_per_run` before Decide, the run ends `stopped_at_cap`
with no write.

**One call in flight.** The calls of a run are sequential, and with MC-A6's one running run there is at most one
research call in flight app-wide.

**The day** is the **local calendar date** of a call's `started_at`, in the Mac's time zone at evaluation (05A
MC-4: calendar dates, never "now − 24 h"). Today's spend = Σ cost of calls whose local date is today. It is
checked only when a run starts (MC-A6). Bound: today's spend ≤ day cap + one run cap + one call.
`research.record_days` has a minimum of 2, so today's calls are always present.

**Invariant (MA15).** Every call's recorded cost equals the integer formula above over its reported usage and
the run's price snapshot, or the fallback when no usage was reported. No run's cost exceeds its cap by more than
the one call in flight. Today's spend is the sum over calls whose local `started_at` date is today.

---

## MC-A9 — Provider calls: record, retries, error classes, secrets

**Record** `research_calls` (08A §10A, plus): `id`, `run_id`, `stage`, `unit_key`, `attempt`, provider, model,
effort, `state` (`sent` / `answered` / `failed` / `unknown` / `not_sent`), the request (method, redacted URL,
non-secret headers, body with each image replaced by `{"image_sha256": …}`), the response (status, non-secret
headers, body), usage, `cost_micro_usd`, `started_at`, `finished_at`, duration (monotonic), and the error class
and text.

**Redaction, two layers.** (1) The adapter declares its secret-bearing parts: the headers `authorization`,
`x-api-key`, `x-goog-api-key`, `cookie` and `set-cookie`, the query parameters `key` and `api_key`, plus any
others it lists. Their values are recorded as `"<redacted>"`. (2) Before every insert or update of a call record
and every research log line, the text is scanned for each secret value held by the secrets port (research
providers and the integration), plain and URL-encoded. A hit is replaced by `<redacted>`, the record gets
`redaction_hit = true`, and an error is logged without the value. The scan catches a key echoed anywhere,
including SerpApi's `api_key` query parameter (RESEARCH-VERIFY-7).

**Attempts** (08A §10 "a fixed count") (R-IA20): `RESEARCH_CALL_ATTEMPTS = 3` per call, waiting
`jobs::backoff` (1 s, then 2 s) between attempts. Call timeout `research.call_timeout_seconds`: 180 for model
calls, 60 for the others.

| Class (shapes: RESEARCH-VERIFY-2, -3, -11) | Action |
|---|---|
| `offline`: connect or DNS failure before any request byte | `not_sent`, no attempt counted, MC-A6 offline wait |
| `transient`: timeout after send, HTTP 408, 429 rate limit, 500–504, connection reset after send | attempt counted; retry |
| `key`: HTTP 401 or 403 from a provider | run `failed`, "key rejected by <provider>"; no retry |
| `quota`: the provider reports no credit or quota | run `failed`; no retry |
| `request`: any other 4xx | run `failed`; no retry |
| `schema`: reply does not match the stage's output schema | attempt counted; retry |
| `refusal`: the model refuses | the unit's output is "not determined" with the refusal text; no retry |

When the attempts run out, the run ends `failed`, naming the stage and the error. Values written earlier stay.

**Invariant (MA16, part).** No call record, research log line or research setting contains a value held by the
secrets port.

---

## MC-A10 — Suggestions in a publish: withheld values and conflicts (amends 07A MC-10 `frozen_attributes`)

**Conflicts are a warning; a confirmed item with conflicts waits (owner card 1, 2026-11-12: "it should be a warning,
and the job is stalled until resolved, but it doesn't block the others").** A conflict is defined in MC-A1: an
unchecked suggestion, at any confidence, that replaced a `default` or `integration` value and differs from it. The
confirmation popover shows each item's conflicts as a warning: the field, the previous value and where it came
from ("preset default" / "from Shopify"), the suggested value and its confidence, and the line "This item waits
until you resolve n conflicts". Each conflict offers **Keep previous**, **Use suggestion** and **Edit** (MC-A12),
there or in the form. **Confirm is never disabled by a conflict.** Chapter 08's "the form never blocks publishing"
holds: nothing blocks the form or the confirmation. Only the confirmed item's own job waits.

**What is frozen, and when.** The run, its items and every frozen list are written at confirmation in one
transaction, exactly as 07A MC-10 says, with three additions per item:
- `frozen_attributes` holds the canonical value of every live, valid row, except:
  - **withheld** rows (`ai` + `suggested` + `low`, no `ai_prior`), which are left out for good;
  - **pending** rows (conflicts at confirmation), which are left out until resolved.

  A row with an `ai_prior` that is not a conflict (its value `S`-equals the prior) freezes its value at any
  confidence.
- `pending_conflicts`: per pending key, the prior value and origin, the suggestion and the confidence, exactly as
  the popover showed them (part of `confirmed_summary`).
- `frozen_attribute_origin`: per frozen, withheld or pending key, the row's provenance, status, confidence and
  prior at confirmation. This is the evidence for MA8.

**Resolution freezes a key.** When the photographer resolves a pending key (MC-A12), the same transaction writes
that key's resolved canonical value into the item's `resolved_attributes`, with the action and time. A resolution
that leaves no live value (Clear) records "no value", and the key is never written (07A MC-13). From then on,
later writes to that field are not in this run: the principle of 08A §7, applied at the key's resolution. If the
photographer undoes the resolution while the item is still `held`, the undo removes that key's entry, and the key
is pending again.

**The stalled step.** An item confirmed with ≥ 1 pending key starts at step **`held`**, which is ordered before
`prepare`. No job is queued for it, no step runs, and no integration mutation is issued (no `ConfirmedRun` value is
handed to the adapter). The transaction that resolves its last pending key sets the step to `prepare`, records
`released_at`, and queues its job. From there the item runs as 07A MC-11 says, sending its frozen lists, with each
resolved key's frozen value in place of the pending one. Other items of the same confirmation start at `prepare` as
usual and never wait for it.

**How this sits with H1, M5 and M17.** Every integration mutation of a held item still carries a run committed with
`confirmed_at` before it. Everything it sends was recorded on that run before the first mutation: at confirmation
for all keys but the pending ones, and at their resolution for those. Each of those resolutions is itself an
explicit, recorded photographer action. The 07A MC-10 deleted/moved-photo skip still applies at every step,
however long the item waited.

**Conflicts that appear after confirmation.** A conflict that a later research run creates, on a key that was not
pending, neither stalls nor changes a confirmed item. The key is sent as frozen at confirmation (08A §7).

**Status and label** (05A MC-7, pointer added there). `held` counts as `publishing` in status rule 1, so the
session is not offered "Publish" again while it waits. Its row label, placed before `Publishing n%`, is
**"Waiting for you · n conflicts"**, where n = pending keys without a resolution (derived).

**Cancel.** On a `held` item, cancel sets `cancelled` at once. Nothing was sent, so nothing needs a safe point, and
the conflicts stay in the form. **Restart.** `held` is a persisted step. At launch a held item stays held: it is not
re-queued, because 07A MC-11's startup re-queue covers active steps only. Release happens only in a resolution's
transaction, so a crash leaves either held-with-pending or released-and-queued (H4).

**What is shown and re-checked.** `n_withheld` = the number of withheld keys. When it is > 0, the popover says "n
low-confidence AI suggestions will not be published". The UI passes, for each item, the `n_withheld` and the set of
conflict keys it showed. The confirming transaction recomputes both from the rows it reads. If either differs for
any item, the whole confirmation is refused ("AI suggestions changed while you were confirming") and the popover
rebuilds (R-IA22). A suggestion written after the commit is not in that run (08A §7).

**"n to review"** (derived, never stored) = live `ai` + `suggested` rows whose key is in the current layout, at
every confidence. Conflicts are among them, and the form marks them.

**Invariant (MA17).** For every confirmed item:
- no integration mutation precedes the resolution of its last pending key;
- it sends no `ai` + `suggested` + `low` value without a prior, and no unresolved conflict;
- each pending key is sent with the value recorded at its resolution, and every other key with the value frozen at
  confirmation;
- the stored origin and resolutions prove this;
- the withheld count and conflict set equal those shown;
- other items of the same confirmation never wait for it.

---

## MC-A11 — Retention, export and replay

**Retention.** A `research_calls` row and the image files only it references are deleted when
`now ≥ started_at + record_days × 86 400 000` ms. This is elapsed time, as in 10F's retention mechanism
(`mechanisms::retention`). The check runs at launch and hourly. Kept while the item exists: `research_runs`,
`research_steps`, `research_feedback`, `research_outbound` (the HA3 proof is small and must outlive the bytes)
and `research_stage_outputs` (R-IA26).

**Export selection.** Runs whose `started_at` local date is in `[from, to]` inclusive, or all runs of the chosen
items. Layout: `research-export-<local YYYYMMDD-HHMMSS>/<item id>-<folder key>/run-<id>/` containing
`run.json` (run row, settings snapshot, form snapshot), `steps.json`, `writes.json` (stage 7 writes and their
MC-A1 outcomes), `feedback.json` (the item's feedback rows at or after the run's start), `calls/<call id>.json`
and `images/<sha256>.jpg`. The folder is then zipped. Only `.json` and the stored `.jpg` copies are ever written.
A run whose calls have expired says "call record expired".

**Exclusions at export time.** The image of a photo that is now `deleted`, or whose row is gone, is omitted, and
the requests that referenced it carry `"withheld": "photo deleted"`. **Secret check, fail closed:** before
zipping, every file is scanned for every value held by the secrets port (all accounts, plain and URL-encoded).
Any hit aborts the export, deletes the partial folder and names the file.

**Replay identity** (dev-Mac tool). The tool rebuilds the recorded request body, resolving images by SHA-256 from
`images/`. It substitutes only the model, effort or instructions given on its command line, and every other byte
of the body is the recorded one. A call with a withheld image, a missing file or an expired record is reported
"not replayable: <reason>" and is never sent partially (R-IA26). The tool reads its own key from its
environment and never reads a Keychain. Replaying stage 3 through a model-side page tool cannot reproduce the
page content (RESEARCH-VERIFY-8).

**Invariant (MA16).** No export contains a value held by the secrets port, a RAW, a master, or an image of a photo
that is `deleted` at export time. A replayable call's rebuilt body differs from the recorded body only in the
substituted fields. Call records older than `record_days` are absent.

---

## MC-A12 — Feedback and undo around research writes

**Feedback rows** are written in the photographer action's own transaction, for each field whose row before the
action had provenance `ai`:

| Before | Action | Row after | `research_feedback.action` |
|---|---|---|---|
| `ai` + `suggested` | Looks right; Accept all (which skips `low` and every conflict (R-IA1)); **Use suggestion** on a conflict | `ai` + `accepted`, `ai_prior` dropped | `accepted` |
| `ai` + `suggested` with `ai_prior` | **Keep previous** (conflict) | the prior's value, validity and provenance; AI columns and `ai_prior` null | `kept_previous` (final value = the prior's; the suggestion joins the cleared set) |
| `ai` + `suggested` or `accepted` | Clear | `cleared`, `ai_prior` dropped (the shop keeps its value: 07A MC-13 never clears) | `cleared` (final value null) |
| `ai` + `suggested` or `accepted` | a human edit, including **Edit** on a conflict | `human`, `ai_prior` dropped | `edited` (final value = the new canonical value) |

Resolving a conflict in the popover is the same owner action as in the form: one undo step each (05A MC-30) and one
feedback row. If the key is pending on a `held` publish item, the same transaction records its resolution there and
releases the item when it was the last (MC-A10). Undoing it while the item is still held removes that record. Each row also stores the suggested value and confidence from before the action, the time, and
`step_token`, the history step's id. Research writes are never feedback and never undo steps (08A §7).

**Undo and redo** (amends 05A MC-30 for attribute steps) (R-IA21). Each field of an attribute step
applies only if the field's current row equals the expected row: the step's after-row for undo, its before-row
for redo, compared on every column except `updated_at`. If any field differs, because research or an integration
read changed it since, the whole step cannot apply. Following MC-30, nothing changes, the step is dropped, and
the notice says "that field was changed by research". Today's history code restores unconditionally. Undoing a
step deletes the feedback rows carrying its `step_token`, and redoing it re-inserts them. Steps that drop off the
25-step limit or are cleared at quit leave their rows in place.

**Invariant (MA18).** An undo or redo never overwrites a research write made after its step. Feedback rows exist
exactly for the photographer actions on `ai` rows that are currently applied, so the cleared set (MC-A1) never
holds a value from an undone clear.

---

## Grounding owed (external shapes, before the criteria of the phase that uses them are final)

From `evidence/research_providers.md` "Not verified" plus what these contracts consume. No fixture may invent any
of these shapes (charter rule 17).
- **RESEARCH-VERIFY-1:** OpenAI Responses `usage` fields (input, cached input, output and reasoning tokens) and
  how built-in tool calls (`web_search`) are counted. Whether an `incomplete` or failed response reports usage
  or is billed.
- **RESEARCH-VERIFY-2:** OpenAI error replies: HTTP status, `error.type`/`code` for a bad key, rate limit, no
  quota, server error and context length; the `Retry-After` header.
- **RESEARCH-VERIFY-3:** structured output with a strict JSON schema: the refusal item's shape, and `status:
  incomplete` with its reason.
- **RESEARCH-VERIFY-4:** image input limits: bytes, count per request, formats, and the `detail` setting with its
  token accounting.
- **RESEARCH-VERIFY-5:** image input and `web_search` in one request (evidence probe 5); `allowed_domains` and
  subdomains.
- **RESEARCH-VERIFY-6:** visual search result shape and rank for the chosen adapter (evidence probe 2): Cloud
  Vision `webDetection.pagesWithMatchingImages` order and fields, or SerpApi `visual_matches.position`, plus the
  usage units each reports.
- **RESEARCH-VERIFY-7:** SerpApi upload: `api_key` placement, `image_id` lifetime, formats and size limit
  (evidence probe 3).
- **RESEARCH-VERIFY-8:** page reader (evidence probe 1): per trusted site, the HTTP status, redirects, final URL,
  JS-rendered content and main images. For a model-side page tool: whether the final URL and page text are
  returned (MC-A3 site of a page, MC-A11 replay).
- **RESEARCH-VERIFY-9:** Shopify: the shop's public hosts (primary, other and platform domains) and a product's
  storefront URL or handle. Neither is in `CommerceAdapter` today (`item_links.external_url` is written as null).
  Shares the SHOPIFY-VERIFY process of 07A.
- **RESEARCH-VERIFY-10:** whether a request that timed out on our side is billed (if not observable,
  `fallback_call_usd` stands).
- **RESEARCH-VERIFY-11:** the installed `reqwest` version's error predicates that separate "no byte sent" from
  "sent, then failed" (from its installed source; MC-A9 `offline` versus `transient`).

---

## Resolved by the inventory, ratified by the owner (R-IA6…R-IA26; owner 2026-11-12, as a list)

Each resolves a gap or contradiction in 08A. Rationale in one line; the contract holds the rule.
- **R-IA6:** Product-step fields are never written. Rationale: ch08 puts title, description and tags there, and
  08A §12 defers the publication assistant.
- **R-IA7:** `media_list`, `reference`, `rich_text`, `money` and `rating` are never written. Rationale: these are
  not facts a listing or a photo establishes (§5 lists only facts).
- **R-IA8:** cleared rows. A human clear is never overwritten (as `ai_suggest` already does). A
  `default`/`integration` clear is treated like an `ai` clear. The cleared set includes earlier clears through
  feedback. Rationale: HA1's "never re-suggests a value the photographer cleared".
- **R-IA9:** the item's own shop listing is excluded. Rationale: otherwise it is circular evidence at `high`.
- **R-IA10:** `max_pages` defaults to 10 and pages that cannot be read do not count; untrusted pages are read
  after trusted ones. Rationale: §6 names "the run's page limit" without a value.
- **R-IA11:** visible-field cases §6 leaves open (listing only; photo and listing disagree). Also, "a match
  listing at medium or better" is read as listing evidence that reaches medium under the table.
- **R-IA12:** condition fields are never raised by listings, capped at medium. Rationale: another piece's
  listing cannot attest to this piece's wear.
- **R-IA13:** image sizes per stage (1024 / 2048) and all metadata removed. Rationale: §5's "unless the stage
  needs detail" names no size, and GPS and serial numbers should not leave the Mac.
- **R-IA14:** a wait of up to 120 s for the research source. Rationale: the 1 s quiet period is shorter than the
  15 s preview delay.
- **R-IA15:** one running run app-wide, `manual` first, then FIFO. Rationale: MA5's single call in flight and a
  bounded day spend.
- **R-IA16:** at the day cap, queued runs wait for the next local date rather than ending. Rationale: §9
  "stops new runs from starting".
- **R-IA17:** follow-ups are triggered by a change in photo-derived input identity (adds main↔condition and a
  cover change by reorder; excludes retouches and settings). Rationale: §5's examples imply both additions.
- **R-IA18:** a `manual` run reuses nothing. Rationale: "Run again" must be able to give a different answer.
- **R-IA19:** a call with unreported usage is charged `fallback_call_usd`. Rationale: HA6 cannot hold if unknown
  costs count as zero.
- **R-IA20:** three attempts, the error-class table and timeouts. Rationale: §10's "fixed count"; no retrying a
  rejected key.
- **R-IA21:** an attribute undo or redo is blocked when research changed the field since. Rationale: MC-30's
  "a step that can no longer apply changes nothing".
- **R-IA22:** confirmation is refused when the withheld count changed, and the origin is frozen. Rationale: H1's
  exact-text-shown rule, and MA8 needs evidence.
- **R-IA23:** runs are per session; Unassigned is never researched. Rationale: `research_runs.session_id`, and
  evidence is "the session's photos".
- **R-IA24:** disagreement counting (support per level, unmapped values count but never win, a page with two
  values states none) and lower-level values named in the reason. Rationale: §6 says "more matching listings"
  without saying what is counted.
- **R-IA25:** with `research.auto` off, follow-ups also stop and queued auto/follow-up runs are cancelled.
  Rationale: follow-ups are automatic starts.
- **R-IA26:** outbound rows and stage outputs outlive `record_days`; `record_days` has a minimum of 2; an export
  without a deleted photo's image marks those calls not replayable. Rationale: MA3's proof and MA9's exclusion
  win over replay.

## Deltas applied to 08A (2026-11-12, 08A changelog entry 7)

Applied at the orchestrator's direction after the owner answered cards 1–5. The R-IA resolutions they rest on
were ratified by the owner on 2026-11-12, and 08A carries `(R-IAn)` trace markers.
1. §5 "Photos used" (card 3) and §5 stage table and follow-up examples (card 2).
2. §6 visible fields: condition severities are no longer raised by listings (R-IA12); "listing evidence reaching
   medium" (R-IA11). §6 condition fields: card 4.
3. §8: `jobs.type` → `jobs.kind`; new tables and columns: `research_stage_outputs`, `research_calls.state`,
   `research_outbound.call_id`, `research_feedback.step_token` and the `kept_previous` action,
   `publish_items.frozen_attribute_origin`, `attribute_values.ai_prior` (card 1).
4. §9 cost cap: the Decide reserve (card 5).
5. §10A: `research_outbound` and stage outputs are kept beyond `record_days` (R-IA26).
6. §3: `ai_suggest` does not refuse `ai` + `accepted` today.
7. MA2 (lowered values) and MA9 (replay clause) in 08A §11 and chapter 14; MA8 with conflicts (card 1).
Also §5 "What a run may write" and §7 (card 1 conflicts), and H3 in chapter 01 (card 3). Pointer lines were added
to 07A MC-10 and 05A MC-30.

## Changelog

1. **2026-11-12 — Mechanism inventory of 08A** (pipeline-inventory, orchestrated). Twelve contracts MC-A1…MC-A12,
   invariants MA10…MA18 in chapter 14, eleven external probes, five owner cards (R-IA1…R-IA5), twenty-one
   resolutions for ratification (R-IA6…R-IA26), seven proposed deltas to 08A. 08A's text and status are
   untouched.
2. **2026-11-12 — Owner answers to cards 1–5** (relayed by the orchestrator). Card 1: a new branch, "conflicts to
   resolve at publish"; the orchestrator's reading is written into MC-A1, MC-A10 and MC-A12 and awaits the owner's
   confirmation (R-IA1, one card left). Card 2 B (MC-A7). Card 3 B (MC-A5; H3 amended in chapter 01). Card 4 A
   (MC-A4). Card 5 B (MC-A8). The deltas were applied to 08A (its entry 7), with pointer lines in 07A MC-10 and
   05A MC-30, and MA2, MA8, MA9 and MA17 reworded in chapter 14. R-IA6…R-IA26 still await ratification.
3. **2026-11-12 — Owner follow-up; contracts ratified.** (a) Card 1, corrected by the owner: "it should be a
   warning, and the job is stalled until resolved, but it doesn't block the others". MC-A10 was rewritten:
   Confirm is never disabled; a confirmed item with conflicts starts at the new step `held` and issues no
   mutation; each pending key is frozen at its resolution; the last resolution releases the item; other items run
   normally; a conflict that appears after confirmation does not stall. The card is removed.
   (b) R-IA6…R-IA26 were ratified by the owner as a list ("the smaller choices correct"), and the markers became
   `(R-IAn)` traces. (c) Trusted-site priority stays reading order only (08A §6). This is the orchestrator's choice,
   by the owner's delegation. (d) Inventory choices made to fit card 1 into existing contracts, decided without a
   new card because each is the smallest rule consistent with them:
   - `held` counts as `publishing` in 05A MC-7, with the label "Waiting for you · n conflicts" (pointer added in
     MC-7);
   - cancel on a held item is immediate;
   - a held item stays held across restarts and is not re-queued;
   - each pending key is frozen at its own resolution, not all at the last one, so a research write between two
     resolutions never enters the run;
   - the confirmation re-check compares the conflict set as well as the withheld count.

   Status: CONTRACTS RATIFIED.
4. **2026-11-12 — MC-A4 invariant scoped (orchestrator, from the R3 review).** The MA11 sentence "no research value
   is above `low` without …" contradicted MC-A4's own visible-field row (a sure photo read with no listing →
   `medium`) and condition paragraph (a sure read → `medium`). The enumerated rows win, as the invariant's first
   sentence already says; the second sentence now names the fields it covers (attribution and measured). Wording
   only: no outcome changes.
