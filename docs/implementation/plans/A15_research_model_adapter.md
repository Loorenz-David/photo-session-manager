---
phase: A15_research_model_adapter
title: The real model connection — the app can ask OpenAI to look at photos and read listing text, with instructions and answer formats per stage, correct cost accounting from the reported usage, and the right reaction to a rejected key, a rate limit or a refusal
difficulty: high
projection: yes
depends_on: [A11_research_call_gateway, A01_research_probes_and_search_benchmark]
---
# A15 — OpenAI model adapter (08A §4, §9; 08B MC-A2, MC-A8, MC-A9; RESEARCH-VERIFY-1…5, -10, -11)

## Goal
Implement `ModelProvider` for the OpenAI API behind an HTTP seam: per-stage instructions and strict output schemas
(version 1, starting from A02's version 0 when it exists), image input at the stage's size, the mapping of the
reported usage to the price-table units, the mapping of every failure to a `CallFailure`, refusals, and the
declared secret-bearing parts. **Every provider-owned shape in this phase comes from A01's captures**; nothing is
written from memory or from documentation prose. **Not in this phase:** visual search (A16), page reading (A17),
default model names (A24), the settings UI for the key (A19).

## Fold before dispatch (coordinator)
A01's evidence may rename or split the capture files. Before this phase is dispatched, check each fixture named in
the criteria against `crates/integrations/src/research/fixtures/README.md` and amend the cells. A row whose shape
A01 recorded as *documented only* is marked **structurally held** with its trigger ("the first such reply recorded
by a call record") instead of being implemented against prose. If A01's evidence contradicts MC-A8 or MC-A9, stop
and route the contradiction to the intention.

## Read first
`master_plan_research.md` §3 (rule 17, keys, HA5), §6.1, §6.6, §7.4 (RESEARCH-VERIFY-11) · book `14` (MA15, MA16),
`08A` §4, §5 (stage inputs and outputs), §9, `08B` MC-A2 (what the model is asked to return), MC-A4 (`sure` flag
only; never a confidence), MC-A8 (units), **MC-A9** (record, redaction layer 1, error classes), **§Grounding owed**
· evidence `research_providers.md` (A01's sections for RESEARCH-VERIFY-1…5 and -10) ·
`crates/integrations/src/research/fixtures/` · code `crates/integrations/src/commerce/shopify/transport.rs` (the
pattern for a private-runtime HTTP seam), `ports/research.rs`, `domain/research.rs` (`CallFailure`),
`crates/research-tool/src/instructions/` (A02's version 0, if A02 has run).

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/integrations/src/research/{http.rs, openai/{mod.rs, adapter.rs, usage.rs, errors.rs, instructions/…,
schemas/…}}`, `crates/testkit/src/fake_research_http.rs` (`FakeResearchHttp`: replays capture files, records the
requests it received), `crates/testkit/tests/suite/research/a15_research_model_adapter.rs` (+ `mod` line).
Edited: `crates/integrations/src/research/mod.rs`, `crates/integrations/src/lib.rs` (the adapter is built when
`research.providers.model` names it and the key exists), `crates/testkit/src/lib.rs`, own tracker row.

## Core mechanisms
None (adapter mapping is proven at the port boundary with captured replies).

## Ordered tasks
1. `ResearchHttp` (trait: one request → status, headers, body bytes, or a transport failure with `sent`) and
   `ReqwestResearchHttp`: `is_connect()` → `sent: false`; any other transport failure → `sent: true`; error text
   built after `without_url()`; timeout from `call_timeout_seconds.model`.
2. Request building per stage from `ModelRequest`: instructions (`INSTRUCTIONS_VERSION = 1`, in the requested
   language; choice fields carry the definition's own choice strings), the strict schema, the images.
3. Reply handling from the captures: structured output → `ModelOutput::Json`; the refusal item →
   `ModelOutput::Refusal(text)`; an incomplete reply or one that fails the schema → `CallFailure` kind `Schema`;
   usage → `(input_token, cached_input_token, output_token, web_search_call)` counts exactly as the capture
   reports them.
4. Errors from the captures → `CallFailure {status, kind: Http, quota flag, usage if reported}`.
5. `Exchange` for the record: `authorization` (and any other header A01 found to carry the key) as `<redacted>`.

## Criteria (6 criteria, 14 rows)
Boundary: `ModelProvider::call` on the adapter over `FakeResearchHttp`; outcome = the `ModelReply` / `CallFailure`
and the request the fake received. Fixture paths are under `crates/integrations/src/research/fixtures/openai/`.
| Row | Boundary | Input (fixture, probe) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | usage | the ok reply with an image and a strict schema (RESEARCH-VERIFY-1) | usage units and counts equal the capture's own usage figures, one unit per reported field the price table names | MA15 |
| C1(b) | usage | the reply of a call that used the web search tool (RESEARCH-VERIFY-1, -5) | `web_search_call` equals the number of tool calls the capture reports | MA15 |
| C1(c) | usage | the incomplete reply (RESEARCH-VERIFY-1, -3) | kind `Schema`; `usage` present or absent exactly as the capture shows (absent → the gateway charges the fallback) | MA15 |
| C2(a) | errors | the invalid-key reply (RESEARCH-VERIFY-2) | `CallFailure` with the capture's status; `classify` gives `key` | MC-A9 |
| C2(b) | errors | the rate-limit reply and the no-quota reply (RESEARCH-VERIFY-2; structurally held if A01 recorded them as documented only) | `transient` for the first; quota flag set and `quota` for the second | MC-A9 |
| C2(c) | errors | the oversized-input reply (RESEARCH-VERIFY-2) | `request` | MC-A9 |
| C3(a) | refusal | the refusal reply (RESEARCH-VERIFY-3; structurally held if not captured) | `ModelOutput::Refusal` with the capture's text | MC-A9 |
| C3(b) | schema | the ok reply with one required field removed from its output | kind `Schema` | MC-A9 |
| C4(a) | request | an observe request with one image at 1024 and choices `[Oak, Teak]`, language `Swedish` | the body the fake received carries the image bytes of that `ResearchImage`, both choice strings and the language; its top-level shape equals the request A01 recorded for an image + schema call (same keys) | MC-A2 |
| C4(b) | request | each of the stages observe, read, compare, labels, condition, decide | each is sent with its own instructions and schema, and the model and effort given in the request | MC-A7 (unit identity: model, effort, instructions version) |
| C5(a) | secrets | any call | the `Exchange` shows `authorization` as `<redacted>`; the key value appears nowhere in it | MA16 |
| C5(b) | transport | `ReqwestResearchHttp` against a closed port on `127.0.0.1` | `CallFailure {sent: false}`; its text contains no URL | MC-A9 (RESEARCH-VERIFY-11) |
| C5(c) | transport | `ReqwestResearchHttp` against a local listener that accepts and never answers, timeout 200 ms | `CallFailure {sent: true}`, kind `Timeout` | MC-A9 |
| C6(a) | end to end | A12's full-run script with this adapter (over `FakeResearchHttp` replaying captures for the model calls) and the fake search and page reader | the run finishes and writes its values; every call row's cost is non-zero and equals `call_cost` of the recorded usage | MA15 |

## Delegated choices
The wording of the instructions (kept short; each stage asks only for its output of 08A §5) · schema field names ·
how the adapter id is spelled in `research.providers.model`. Not delegated: a model never returns a confidence
(only `sure` on photo reads), and no shape is taken from memory.

## Manual try-out
Still nothing in the app (the key is entered in A19). A developer can see one real exchange in A01's recordings
(`~/psm-research-bench/a01/probes/`), which is what this adapter is tested against.

## Notes
- C5(b) and C5(c) use loopback sockets on port 0, as `p43_ftp_receiver` does; they are not external services.
- C6(a) needs one captured reply per stage; A02 writes them. If A02 has not run, the row uses the image + schema
  capture for every stage and says so under Assumptions.
- Review mode: this phase's module, `research::a11…::`, `research::a12…::`, clippy, fmt.

## Review log
