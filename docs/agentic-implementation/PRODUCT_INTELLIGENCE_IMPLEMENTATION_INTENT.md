# Product Intelligence Platform — Implementation Intent

**Status:** Architecture and planning brief; not authorization to implement immediately  
**Audience:** Claude implementation/planning agent  
**Date:** 2026-11-12  
**Initial client:** Session Manager (macOS Tauri/Rust/React)  
**Future integrations:** Beyo central product domain, Shopify, other clients and knowledge providers

## 1. Mission

Build a reliable, modular **Product Intelligence Service (PIS)** that supports furniture photography sessions, progressively analyzes arriving photographs, researches product facts using pluggable sources, proposes evidence-backed values for an **existing, dynamically configured product form**, accepts human corrections, and prepares approved products for later batch publication.

The service must be independent of any particular product database, Shopify metafield naming scheme, camera vendor, AI model provider, or client UI. Session Manager is the first client, not the permanent home of the intelligence engine.

**Planning first:** Inspect the existing repositories, data contracts, Session Manager shot templates, product forms, image lifecycle, and central-domain plans before proposing changes. Produce a phased implementation plan, API/data contracts, risks, and test strategy. Do not assume the existing architecture needs replacing.

## 2. Explicit clarification: Do NOT rebuild product keys

Shopify and the future central product domain **already have field keys and schemas**. Preserve them as authoritative at their respective boundaries. Do **not** rename, migrate, duplicate, or rebuild existing product fields to adopt keys such as `furniture.material.primary_wood`.

The prior example of a semantic field identifier was an **optional internal alias/ontology**, not a proposed new Shopify or central-domain key. Only introduce semantic aliases if actual cross-provider mappings require them, and keep them in a versioned, configurable mapping registry. In the simplest implementation, carry the existing field key unchanged plus its source namespace/schema version.

Example field identity (illustrative only; discover real keys from the repository):

```json
{
  "source_system": "shopify",
  "source_field_key": "custom.wood_type",
  "field_schema_version": "existing-version",
  "semantic_alias": null,
  "type": "single_line_text_field"
}
```

**Principle:** The agent enriches the fields it is given; it does not own the schema. The client/form adapter supplies current field definitions, allowed values, validation, and field-level permissions. An output adapter maps approved values back to the exact original keys. A future move from Shopify to the central domain should require adapter/configuration changes, not rewriting reasoning workflows.

## 3. Scope: three complementary assistants

### A. Product intelligence (primary backend)
- Interpret images progressively as they arrive during a session.
- Extract observations (shape, material cues, extension boards, markings, damage, construction).
- Maintain field-level hypotheses, uncertainty, supporting/contradicting evidence, and source provenance.
- Retrieve comparable records from Beyo's existing data, future central domain, vector index, and/or controlled web research **through tool adapters**.
- Request specific missing evidence (e.g., underside label, end grain, damage close-up) when valuable.
- Never present a similar-looking item as proof of designer/manufacturer attribution.
- Let humans review, correct, attach supporting image(s), and approve each field.
- Preserve verified human values; new model runs must not silently overwrite them.

### B. Session photography assistant (local, responsive)
**Updated priority: NOT autonomous exposure/ISO/shutter tuning.** Camera settings are largely standardized already; do not build a camera-setting agent as a prerequisite.
- Understand the active product type and its **existing ordered shot template**.
- Classify incoming images by intended shot/angle and help assign them to the correct template slots and final product-image order.
- Detect duplicates, missing shots, unexpected angles, and possible multi-image composites; recommend corrections and allow drag/drop/manual overrides.
- Help with **alignment, horizon/vertical straightening, framing, crop, split-image layout**, and consistency between product images. Respect the user's 1:1 grid and other configured output presets.
- Prefer deterministic geometry/computer-vision operations for crop and alignment; use multimodal models to infer intent or propose candidate regions, not to fabricate product pixels.
- Non-destructive edits only; retain originals and edit recipes. Display preview, confidence, and allow one-click acceptance/undo.
- Keep latency-sensitive assistance local where practical; heavy analysis can be queued.
- Existing Sony SDK connection, Pixel Shift and bracketing work remain separate integrations. This initiative must not disrupt them.

### C. Publication assistant (later, batch)
- Run only on approved or explicitly review-ready product states.
- Generate natural, brand-appropriate product titles, descriptions, condition text, SEO titles and meta descriptions from approved facts.
- Use existing Shopify/central-domain field definitions and validation.
- Support batching many products to reduce operational overhead and possibly inference cost; provider-side batch APIs are optional.
- Stage drafts for review; publishing and external writes require explicit authorization and safe conflict handling.
- Do not invent dimensions, provenance, designer, manufacturer, materials, restoration, or condition claims.

## 4. Workflow lifecycle and concurrency — Temporal from the start

Adopt **Temporal** as the durable workflow orchestrator from the initial implementation, rather than treating it as a later upgrade. Use the Python SDK with the existing FastAPI/backend ecosystem if appropriate. The implementation agent should evaluate operational requirements and propose a local development environment and production deployment approach before coding.

**One durable workflow per product photography session**, scoped by tenant/business and session ID. Many workflows must coexist independently. Temporal workflows coordinate durable activities; they must **not** hold an LLM process open continuously. Store large images, inference results and canonical evidence outside workflow history; use IDs/references in events.

Illustrative event vocabulary (adapt to existing app events):
- `session.started`
- `image.received`
- `image.updated`
- `image.deleted`
- `shot.assignment_changed`
- `capture.completed`
- `field.corrected`
- `field.approved`
- `session.review_requested`
- `publication.batch_requested`

A workflow should survive the photographer switching to another item, Session Manager quitting, Mac restart, network loss, model/API failure, and worker restart. New images should incrementally update relevant fields, not rerun the entire research pipeline by default.

Illustrative states:

```text
CREATED → COLLECTING → OBSERVING → RECONCILING → RESEARCHING
                     ↖              ↓                 ↓
                       NEW_EVIDENCE / WAITING_FOR_EVIDENCE
                                             ↓
                                   READY_FOR_HUMAN_REVIEW
                                             ↓
                                         APPROVED
                                             ↓
                                  PUBLICATION_ELIGIBLE
```

The workflow must have explicit stop/wait/resume conditions, cost/time/retry budgets, idempotency keys, per-session concurrency controls, and cancellation/compensation where relevant. Temporal activities that invoke external services must be idempotent or guarded against duplicate effects. Avoid unbounded workflow history; plan for continue-as-new/versioning if needed.

## 5. Product form and integration contracts

Define these **five versioned contracts**, grounded in existing app schemas:

1. **Product Form Contract** — product type, actual existing field IDs/keys, field types, allowed values, requiredness, validation, editability, provenance requirements, review policy, and schema version.
2. **Evidence Event Contract** — session/item identity, image ID, source/original/derivative IDs, capture slot, timestamps, content hash, processing status, user annotations, correction events.
3. **Knowledge Provider Contract** — structured search and fetch across Beyo historical products, central domain, pgvector, external web research, and future tools; include source identity, timestamp, trust level, permission/tenant scope, and citations.
4. **Intelligence Result Contract** — field-key-preserving suggestions, hypotheses, evidence links, contradictions, calibrated/uncalibrated confidence indication, verification status, model/prompt versions, and change history.
5. **Publication Contract** — approved fact snapshot, output schema mapping, validation, generated copy, draft status, review outcome, external write result, and concurrency/version guard.

Do not force every product category into a static universal form. Dynamic product forms and shot templates are first-class inputs. Use strict Pydantic validation and explicit unknown/not-observed states.

## 6. Ownership and persistence boundaries

- **Existing Shopify or central domain:** canonical product fields and business records; preserve existing keys.
- **Session Manager:** capture UI, shot templates, local working session, edits, previews and user actions.
- **Product Intelligence Service:** durable *intelligence working state* (observations, hypotheses, evidence graph, research jobs, corrections, model versions). This is not a second authoritative product catalogue.
- **Temporal:** workflow orchestration and execution history, not the only product/evidence database.
- **PostgreSQL:** structured intelligence state, claims, sources, correction events, agent runs and publication drafts.
- **Object storage:** source images, previews, derived images and evidence crops, with retention and access controls.
- **Optional pgvector:** similarity search over authorized reference products/images; combine with structured filters and full-text search. Vector similarity is candidate retrieval, never authoritative attribution.

Preserve original RAWs and distinguish source evidence from presentation derivatives. Cropping or color edits should not automatically invalidate all observations; removing the sole source of a claim should flag that claim for reevaluation. Edited images and AI annotations must never silently change the underlying source evidence.

## 7. Model and agent orchestration

Use **bounded, typed activities**, not one unrestricted autonomous agent:

1. Image observation (vision model; extract structured observations with image-region references).
2. Field reconciliation (update existing hypotheses; detect contradictions).
3. Research planning (decide which unresolved fields merit retrieval).
4. Tool retrieval (Beyo, central domain, vector, authorized browser/web).
5. Evidence adjudication (distinguish confirmed, probable, unknown).
6. Missing-evidence request (optional shot suggestion).
7. Human review/correction handling.
8. Publication drafting and validation (separate workflow).

Abstract model providers; benchmark rather than preselect a single multimodal model. Prefer deterministic validation and workflow logic around model calls. Confidence values must not be presented as calibrated probabilities without empirical calibration. Designer/manufacturer claims require stronger provenance than basic product category or visible extension count.

Corrections should immediately update the product's verified working state and produce reusable evaluation examples. **Do not claim that every correction automatically fine-tunes model weights.** Later training or fine-tuning must be a deliberate, evaluated process. Enforce tenant separation and avoid leaking one business's private references into another's outputs.

## 8. Camera assistant boundary (revised)

Prior discussions proposed an AI that dynamically changes camera exposure between angles. **This is no longer the primary requirement.** The user now reports that camera settings rarely need adjustment during normal photography.

Initial session assistant should focus on:
- Matching images to ordered product-type shot templates.
- Helping with framing, alignment, crop and split/grid compositions.
- Detecting missing/duplicate/misordered shots.
- Supporting automatic grouping of multi-shot captures when available.

Do not introduce autonomous camera-setting changes in the MVP. A future optional camera-control plugin may use validated, deterministic capture profiles and Sony SDK operations, with user override and safety constraints.

## 9. Batch publication lifecycle

Research starts at `session.started` and progresses as evidence arrives, even when other sessions are active. Publication is intentionally decoupled: the photographer may finish several sessions, edit and color-correct them later, and request publication in a batch.

A publication batch must:
- Freeze/version the approved fact snapshot for each product.
- Generate copy/SEO in parallel within cost/rate limits.
- Validate against facts and required form fields.
- Return drafts for review.
- Map fields through the active output adapter (Shopify today; central domain later).
- Avoid duplicate writes, detect stale versions, and respect explicit publish permissions.

## 10. Reliability and security requirements

- Durable progress and recoverability across process restarts.
- Idempotent event ingestion, activity retries with backoff, dead-letter/escalation for persistent failures.
- Field-level provenance, human-verification locks, explicit unknowns, contradictory-evidence handling.
- Audit logs for model calls, retrieval, suggestions, corrections and publication changes.
- Model/prompt/schema versioning; deterministic regression/evaluation datasets.
- Configurable per-product research budgets, timeouts, token/cost caps and tool permissions.
- Observability: Temporal UI, OpenTelemetry traces, metrics for queue age, inference cost, completion latency, review acceptance and false-attribution rate.
- Tenant and source access control, secret management, minimal data sent to model providers, retention and deletion policies.
- Human approval before publishing or overwriting externally authoritative records.

## 11. Suggested implementation sequence

**Milestone 0 — Repository discovery and design (do first)**
- Locate and inspect existing Session Manager form/template APIs, product models, image events, Shopify mapping, central-domain design, and worker conventions.
- Identify which repo owns the new service and what should remain in Session Manager.
- Produce architecture decision records, integration diagrams, exact schema mappings and unresolved questions.

**Milestone 1 — Contracts and Temporal foundation**
- Define the five versioned contracts and event idempotency scheme.
- Create local Temporal dev stack, worker, FastAPI endpoints and one durable session workflow.
- Demonstrate four concurrent workflows surviving a worker restart.

**Milestone 2 — Progressive product intelligence vertical slice**
- Start a dining-table session with the real dynamic form.
- Ingest images incrementally; produce structured observations and field suggestions with image references.
- Persist state and expose progress to Session Manager.
- Accept corrections with supporting images; protect verified fields.

**Milestone 3 — Knowledge adapters and research**
- Implement a first Beyo data adapter; add pgvector only if useful.
- Add bounded retrieval, source provenance and contradiction handling.
- Benchmark no-retrieval versus retrieval on verified products.

**Milestone 4 — Session photography assistant**
- Map images to existing ordered shot templates.
- Prototype crop/alignment suggestions and non-destructive edit recipes.
- Measure accuracy and review time; preserve manual override.

**Milestone 5 — Batch publication**
- Generate fact-grounded titles, descriptions and SEO for approved products.
- Implement current Shopify adapter and review-first draft flow.
- Keep central-domain output adapter replaceable.

**Milestone 6 — Evaluation and optimization**
- Build a representative, human-verified furniture test set.
- Measure per-field precision, abstention, provenance quality, correction rate, cost and latency.
- Tune prompts, retrieval, models and workflows based on evidence.

## 12. Minimum acceptance tests

1. Start four photography sessions sequentially; each continues independent analysis in the background.
2. Close Session Manager and restart Temporal workers; all workflows resume without lost or duplicate work.
3. Feed three images per session; new evidence updates only relevant hypotheses.
4. Use the real dynamic product form and preserve exact Shopify/central-domain field keys end-to-end.
5. Human-correct a field, attach an evidence image, then ingest another photo; verified correction remains authoritative.
6. Retrieve a similar but different designer's chair; system must not claim an attribution based on similarity alone.
7. Detect missing or misordered template shots and propose (not silently impose) corrections.
8. Produce a non-destructive alignment/crop suggestion and preserve the original.
9. Publish-preparation batch for multiple approved products produces drafts with no invented claims and no unapproved external writes.
10. Retry duplicate/out-of-order image events and transient model/tool failures without duplicate facts or publication writes.

## 13. Required deliverables from the implementation agent

Before writing substantial production code, provide:

1. **Current-state inventory** — actual existing repositories, product field keys, forms, templates, and event paths; cite file paths.
2. **Proposed boundaries** — service ownership, data flow, Temporal workflow topology, and adapter interfaces.
3. **Versioned contracts** — sample requests/events/responses using *real* keys where discoverable.
4. **Data model** — migrations and evidence/claim/correction lifecycle.
5. **Temporal deployment plan** — local and production topology, persistence, operational costs, monitoring and recovery.
6. **Phased implementation tickets** — small, testable tasks with acceptance criteria and dependencies.
7. **Risks and open questions** — especially source quality, confidence calibration, image privacy, model cost, and schema changes.
8. **First vertical-slice test plan** — runnable end-to-end tests including four concurrent sessions and human corrections.

**Do not begin with an agent framework, model fine-tuning, schema migration, or autonomous camera-setting changes. Begin with existing-system discovery, contracts, Temporal durability, and one evidence-backed product session.**
