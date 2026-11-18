# 08C — Publication writer (titles, descriptions, condition report, search texts)

**Status: DRAFT, shaping** (owner David Loorenz; shaper Claude; 2026-11-18). Not ratified: nothing
here may be planned or built until the owner ratifies it (§13). Branch `feature/agentic`.
This is the "publication assistant" that 08A §12 deferred to its own chapter. Evidence for the
writing rules: `docs/intention/evidence/publication_writer_research.md` (research 2026-11-18,
sources cited there).

## 1. Outcome

When the photographer has confirmed a session's photos and form and moves on to the next session,
Session Manager writes the item's texts in the background:
- the **title**
- the **description**
- the **condition report description**
- Shopify's **search title** and **search description**

Each text arrives in the AI review tab as a suggestion with its reasoning and the facts it used,
to Accept, Keep previous or Edit, like every research suggestion. The texts read like a careful
dealer's catalogue entry, not like AI. They contain only facts the form and the photos support,
and they follow search rules for this kind of product.

The writer **does not look at the photos**. Research already looks at them once, and from this
chapter on it also writes down what it saw (§4). The writer reads that note, so photos are
analysed once per item.

## 2. Hard constraints (added to H1–H8 and HA1–HA6, same standing)

| # | Constraint | Why |
|---|---|---|
| HW1 | The writer never overwrites a text that holds a `human` value or an accepted `ai` value, and never re-suggests a text the photographer cleared. The rules are 08B MC-A1's write table, applied to the writer's fields. | A checked text must stay checked. |
| HW2 | Every claim in a written text comes from a field the writer may see (§8) or from the product note (§4). A designer, model, maker, year or material absent from both is never written. | A false claim to a buyer. |
| HW3 | What the photographer decided in the research review is final. The writer and its safeguard never question it again (§6). | Owner 2026-11-18: "otherwise the user is denying twice". |
| HW4 | The writer sends no image to any provider by default. | Photos are analysed once (owner 2026-11-18). |
| HW5 | Texts never mention AI, the writing process or the shop's values. | Evidence §4: saying "AI" lowers purchase intent. |
| HW6 | Writer calls count toward the same day cap as research and are recorded in the same call record and export. | One budget, one record (08A §9, §10A). |
| HW7 | With no key, no network or no integration, everything else works as before. The writer does not run and says why. | H6, HA4. |

## 3. Grounding (what exists on `feature/agentic` at `9aed4c8`)

- **Research stages** (`crates/research/src/stages.rs`): stage 1 Observe looks at up to
  `observe_max_photos` Main photos and answers per field. Stage 6 Condition reads the condition
  photos per condition field. Neither keeps a free-text description of what it saw.
- **Matching listings:** stages 2–4 find, read and compare listing pages. Each read page is
  stored as a step with its URL and title, and stage 4 gives a Match / Similar / Different
  verdict. These titles are the "words buyers use" the writer recycles (§7).
- **Write rules:** 08B MC-A1 write table (`crates/research/src/rules/write_table.rs`). Product-group
  fields, rich text and the built-in title are never written by research. They are the writer's
  fields.
- **Review:** the AI review tab (A11, A18): Accept / Keep previous / Dismiss / Edit, reasoning,
  sources, Accept all. The review queue and "Next to review" (A19).
- **Settings pattern:** per-stage model and effort, language (English), cost caps, the price table
  (`crates/research/src/config.rs`). The provider adapter is OpenAI.
- **Shop fields on the work Mac:** `product.title`, `product.description`,
  `custom.condition_report_description` (the only condition text in use, owner 2026-11-18), and the
  condition choice fields (scratches, veneer damage, stability…) research fills in.
- **Not existing:** a product note, any writer code, Shopify SEO title/description support (the
  Shopify adapter writes no `seo` input today), an editable instruction.

## 4. The product note (research change)

Research's two photo looks each also return a short factual note:

- **Stage 1 (Main photos):** what the piece is, its form, construction, materials and visible
  details. Each observation is tagged with the field keys it bears on (e.g. "frame in a reddish
  wood with straight grain" → `custom.color_furniture`, material).
- **Stage 6 (condition photos):** per photo, the wear it shows and where, tagged with the condition
  field it bears on and the photo number.

The note is stored with the run's stage outputs. Its identity is the same photo identity the
stages already use (photo id and revision), so it is reused, not redone, when nothing changed.
Cost: a paragraph of extra output on calls that already happen, well under a cent per item.

**The photographer's review updates the note.** Each tagged observation is overridden by the
photographer's decision on its field: accepted, corrected, kept previous, dismissed or typed. The
writer sees the decision, marked as the owner's, not the original observation. Example: research
saw "oak", the photographer set Teak, so the note now says teak (owner's decision).

**Stale note:** if photos change after research looked (another Main photo, a recompose, other
condition photos), the writer shows "photos changed since they were looked at" and offers
"Look again" for stages 1 and 6 only. This is manual, consistent with A19.

## 5. Workflow

1. **Trigger (owner 2026-11-18):** the photographer finishes reviewing a session and presses
   "Next to review" (or leaves the session with AI review "All checked"). The writer queues for
   that session. A manual "Write texts" button in the Product tab also queues it. Nothing writes
   while research still runs on the session.
2. **Inputs:**
   - the fields the writer may see (§8), with their values and provenance
   - the product note with the owner's overrides
   - the titles of matching listings (stage 4 Match or Similar)
   - the writing guide (§7), the style examples, and the language
   - the photographer's answers to earlier questions, if any
3. **One call** returns, as structured output:
   - the five texts
   - per text, the facts used (field keys and note references) and a short reason
   - the safeguard findings (§6)
   - at most `writer.max_questions` questions
4. **Post-check** (deterministic, no model):
   - banned words and patterns
   - lengths
   - every damage field set must appear in the condition report
   - every designer, model, maker and year in a text must match a field

   On a hit, the writer regenerates once, naming the hits. On a second hit, the texts arrive
   flagged with what failed.
5. **Output:** suggestions in the AI review tab, written through the HW1 rules, each with its
   reasoning, facts used and the previous text. Questions and safeguard findings appear above
   them.
6. **Answers:** answering a question or a finding re-runs the write (text only, no photos), and the
   answer is kept as an owner decision on the item.

## 6. The safeguard (contradiction checker)

A guard against tired or hurried approvals. The checks are split by type from the start, each type
with its own setting **off / note / ask**. A note shows on the item; an ask is a question
that waits for an answer. Every finding records whether it was answered or dismissed, so a type
that annoys shows up in the counts.

| Type | Checks | Default |
|---|---|---|
| Field vs photos | an undecided field contradicts the product note | ask |
| Field vs field | e.g. seat height above total height; an era that cannot fit the designer | note |
| Field vs listings | a Match listing credits another designer or maker | note |
| Missing in text | a condition field marks damage but the condition text omits it (also the post-check) | ask |
| Text vs facts | a sentence claims what no field or note supports (also the post-check) | ask |

**Never re-raised (HW3):** a field the photographer accepted, corrected, kept or dismissed in
research review, or a question already answered. The safeguard looks only at undecided things:
- values typed by hand that research never looked at
- shop defaults
- imported values
- the writer's own sentences

The default modes are a starting point for the owner's experiments.

## 7. The writing guide (default, editable)

The default guide is the evidence file's §7, adapted. In short:
- **Instruction:** about 330 words. Plain catalogue register, only facts from fields and note,
  exact names, a fixed order, and what to write rather than what not to write. Evidence: few-shot
  examples and structure are the reliable levers; a ban list in the prompt alone is unproven.
- **Title formula** (≤70 characters): designer known, designer unknown, attributed, and Swedish
  variants, e.g. "Børge Mogensen J39 Chair in Beech for FDB Møbler, 1960s".
- **Description:** 60–120 words, in this order:
  1. identity
  2. form and materials
  3. what the workshop did
  4. at most one use sentence
  5. the dimensions line with seat height
- **Condition report:**
  - at most 45 words
  - opens with a grade word, then the restoration
  - each flaw with location, size and photo number
  - always states stability
- **Search title** ≤60 characters. **Search description** 120–155 characters.
- **Banned words and patterns** (English and Swedish) run as the post-check (§5.4), not only in the
  prompt.
- **Words from matching listings:** nouns only (canonical maker, model number, type words), once in
  the title and once in the description. Spelling variants are never written into visible text.
- **Style examples:** 3–5 per language, shown to the model as examples, never copied. They start
  from the evidence file's examples and the owner's own published texts edited to the guide (§12).

## 8. Settings (Settings → Writer)

- **Model and effort**, with the price table shared with research.
- **Instruction:** editable, with "Preview what is sent" (the exact request with an example item),
  earlier versions kept, and "Reset to default".
- **Fields the writer may see:** a checklist over the layout's fields. Default: all Details and
  Condition fields. Prices are excluded by default.
- **Style examples** per language, banned words (added to the built-in list), and target lengths.
- **Language:** English or Swedish, default English (owner 2026-11-18).
- **Safeguard:** one off/note/ask switch per type (§6); `writer.max_questions` (default 3).
- **Automatic writing** on "Next to review": on by default.

## 9. Fields the writer writes

| Field | Shop target |
|---|---|
| `product.title` | product title |
| `product.description` | product body (HTML from paragraphs) |
| `custom.condition_report_description` | the existing metafield |
| `product.seo_title` (new) | Shopify product SEO title |
| `product.seo_description` (new) | Shopify product SEO description |

The two SEO fields are new built-in Product fields and need a publish step that writes Shopify's
`seo` input (owner 2026-11-18: yes). Every other field stays research's or the photographer's.

## 10. Where it lives

A second kind of run inside the research engine (`writer` next to `research`). It reuses:
- the job queue
- the run, step and call record
- the cost and day cap
- the provider gateway and secrets
- export and replay

It is the same service-ready shape as 08A §4. The planner may split it into its own crate behind
the same ports if the research crate grows too large. That is a planning decision, not an
intention change.

## 11. Observability

The writer's run appears in the research panel as its own block: inputs used (fields, note,
listing titles), the reasoning per text, post-check hits and regenerations, findings, questions,
cost. It is exported with Export research.

## 12. Scope ladder

**Must ship:**
- the product note (§4) with owner overrides
- the writer run (§5) with post-check
- the five texts
- review integration
- questions and the safeguard types
- the settings in §8
- the two SEO fields and their publish step
- the panel and export

**Only if cheap:**
- per-text "Write again"
- a word-level comparison against the previous text in review

**Deferred (owner 2026-11-18 or later):**
- learning from the owner's edits (record, show, suggest instruction changes)
- Google Search Console and DataForSEO keyword sources
- Google Merchant Center feed (`structured_title` disclosure)
- per-item-type instructions
- batch writing for many items
- sending one photo to the writer

**Non-goals:** fine-tuning; AI-text detectors or "humanizer" rewrites (evidence §4: unsupported);
the writer changing any non-text field.

## 13. ⚠ OWNER DECISIONS REQUIRED

1. **Grade scale:** the condition report opens with Excellent / Very good / Good / Fair (Swedish:
   Utmärkt / Mycket gott / Gott / Godtagbart skick). Adopt it? It could be a new condition choice
   field or derived from the existing condition fields.
2. **"Attributed to":** allowed when the designer field is marked attributed, or never?
3. **Era:** production decade, design year, or both when known?
4. **Restoration:** keep the free text (Restoration field), or add fixed choices (sanded, oiled,
   lacquered, new foam, webbing, papercord, fabric) so the writer states them exactly?
5. **Use sentence** ("suits a desk or reading corner"): keep, at most one, or drop?
6. **Search texts:** add the shop name to search titles when it fits? "Ships from Sweden" in search
   descriptions?
7. **Style examples:** start from the evidence file's four invented examples plus 3 of your
   published listings that the writer rewrites to the guide for your approval?
8. **Provider:** the same OpenAI provider as research (one key), or also allow a second provider
   for writing?

## Shaping changelog

1. 2026-11-18: draft from the owner's brief:
   - texts: title, description, condition report; more fields later
   - inputs: evidence fields and photos, with photos analysed once by research
   - not AI-sounding, and search-aware, starting from matching-listing words
   - settings: model, effort, instruction, visible fields
   - reasoning shown; questions as a typed safeguard
   - language setting, English default
   - trigger after review on moving to the next session
   - SEO fields yes; condition text only in "Condition Report Description"
   - review decisions final (HW3); learning from edits deferred

   Research: `evidence/publication_writer_research.md`.
