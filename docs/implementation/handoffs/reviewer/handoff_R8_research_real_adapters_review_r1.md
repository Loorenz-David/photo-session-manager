---
plan: R8_research_real_adapters
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-13
actor: opus (pipeline-reviewer, Opus 5.5)
---
# R8_research_real_adapters — review round 1: APPROVED_WITH_NOTES

First review of the milestone (3 units, 20 criteria, 35 rows). The authorities are flow.md §5 and §10, the plan's Review mode, and book 08B MC-A2, MC-A3, MC-A8 and MC-A9 with changelog 11. Every row has a test at its boundary that fails on a wrong outcome. No behaviour contradicts the book or the plan. The build, lint and format checks are green. Four non-blocking notes were written.

## Verified correct

**Gate.** Tracker `IMPLEMENTED`; checkpoints `0c6dec4` (U1), `c577f08` (U2), `d37e69e` (U3), `655d312` (gate fixes) and closing `b46d792` are in `git log`.

**Perimeter.** `git show --stat` of each checkpoint equals the handoff's declared list exactly, with no extra and no missing paths. Three paths are outside the plan's Files lists. Each is disclosed and each follows from a planned change:
- `p12_shopify_adapter.rs` c4i/c4j now expect `unit_names` (task 6);
- `research/r2_foundation.rs` f_c3a dropped its assertion of the empty `public_listing` default (task 5);
- `master_plan_research.md` §6.2 records the dependency pins.

`crates/research/fixtures/` has 0 diff lines over `0c6dec4^..b46d792`.

**Checks.** Plan Review mode, code tree `655d312`, one cargo command at a time, foreground:

| Command | Result |
|---|---|
| `cargo test -p psm-research --test engine adapter_openai::` | 15 passed |
| `cargo test -p psm-research --test engine a15_c` | 17 passed (incl. `lifecycle::a15_c7a`, `export::a15_c5d`) |
| `cargo test -p psm-research --test engine adapter_search::` | 7 passed |
| `cargo test -p psm-research --test engine adapter_pages::` | 9 passed |
| `cargo test -p psm-research --lib rules::calls` | 15 passed |
| `cargo test -p psm-testkit --test suite research::r8_shop_facts::` | 4 passed |
| `cargo test -p psm-testkit --test suite research::r7_wiring::` | 7 passed |
| `cargo test -p psm-testkit --test suite p12_shopify_adapter::` | 69 passed |
| `cargo clippy -p psm-research --all-targets -- -D warnings` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | exit 0, no output |
| `cargo check --workspace` | ok |

The implementer's full-suite stamp (874 suite tests, workspace minus testkit, vitest 488, typecheck) stands: it is not missing and nothing contradicts it. No real provider call was made.

**Rows, by reading the production code and then its test.**
- **A15·C1(a)–(c), usage.**
  - Mapping. `usage.rs` gives `input_tokens − cached − cache_write`, then cached, cache write, output, and `tool_usage.web_search.num_requests`, counted only when above 0. It is `None` only when there is no `usage` object.
  - Capture values agree: 1316−1313 = 3 / 1313 / 26 → 3549 µUSD (the capture's own `cost_micro_usd`); 25461−5746 = 19715 / 5746 / 561 / 3; incomplete 3 / 1313 cached / 16.
  - The tests take the usage from a Schema failure, because the captured outputs are R1's version-0 shapes. `usage_of` runs before the success/failure branch, so it is the same path as `Ok`.
- **A15·C2(a)–(c), errors.** `errors::status_failure` sets `quota: false` always, with the provider's `error.message`. `classify` gives Key for 401, Transient for 429, Request for 400. Fixture statuses 401 and 400 are confirmed.
- **A15·C3(a), refusal.** It is checked before the status check, so a refusal is never mistaken for a schema failure. The exact text and usage 3 / 1555 / 77 match the observe envelope.
- **A15·C3(b), schema check.** `schema::check` handles `anyOf`, refuses missing keys and extra keys (`additionalProperties: false`), and checks enums and kinds. Without `shows_marking` → Schema; with it → `Ok` with the exact output.
- **A15·C4(a)/(b), request.**
  - Exactly `input, instructions, max_output_tokens, model, store, text`, plus `reasoning` only when effort is not empty. `instructions` and `model` verbatim, `store: false`, 4000.
  - Images are JPEG data URLs with no `detail`. A missing listing image is skipped.
  - The schema is built from `request.input["fields"]`. Every stage input carries `fields`: observe (`stages.rs` 649), read (823), labels (971), condition (1023), decide (`decide.rs` 236). The per-stage keys equal what the parsers read (`reads`/`sure`/`shows_marking`, `keys` via `statements(…, "keys")`, `verdict`, `labels` + `text`, `reasons`).
- **A15·C5(a), (d), secrets.** `authorization` shows `<redacted>` in the `Exchange`. The key is read per call from `research-model-key`. `json_escaped` was added to `rules::calls::redact`. The export scan calls `redact` (`export.rs` 349), so both halves of C5(d) depend on the new form. The serialized text never holds the plain key `sk-a"b\c`, so both tests fail without it.
- **A15·C5(b)/(c), transport.** Closed port → `is_connect` → (false, Connect), with the text built after `without_url()` and the URL, path and `api_key` all absent. Silent listener with 200 ms → (true, Timeout).
- **A15·C6(a), end to end.** The `once` rules are consumed in order, so a mix-up in stage order would break both the parse and the exact costs 234 / 243 / 194 / 4076 / 194. Each cost equals its capture's `cost_micro_usd`.
- **A15·C7(a), `wait_ends_at`.** Automatic → now + 90 000; manual → now; no waiting run → `None`.
- **A16·C1–C6, Lens.**
  - Upload (multipart: image bytes unchanged + `api_key`), then search by the returned `image_id` with `engine=google_lens&type=all&hl=en&country=se`. `visual_matches` in reply order, ranks over results with a link; an absent or empty list → `Ok` with no candidates.
  - One `image_search_call`. A 400 on upload → Request, and no search is sent.
  - `api_key` shows `<redacted>` in the multipart record and in the URL. SHA-256 of the sent bytes = the stored copy.
  - Full run: read order beyovintage.se → auctionet.com → pamono, with exactly one outbound row for the cover.
- **A17·C1–C3, page reader.**
  - `read` never returns `Err`.
  - Redirects: one request per hop; `redirects <= 5` follows, and the 6th → not obtained. C1(b) chain(5) and chain(6) pin both sides.
  - `final_url` = the URL that answered 2xx.
  - Pace: in memory, per host, after a `powered-by: Shopify`, `x-shopid` or 429 answer, 2 000 ms before every later request to that host. There is none before the first page request. The `Retry-After` retry adds no pace and happens only when n ≤ `call_timeout_seconds.other`; a second 429 → not obtained.
  - Usage `page_fetch` 1 when anything left the Mac; `[]` when the guard refused or the connection failed before sending.
  - Steps in the full runs: one "could not be read" plus two titled, `Finished`. C3(d) holds no run on a dead domain.
- **Address guard (not delegated; read by the reviewer).**
  - Half 1 (`allowed_host`): only http/https, never an IP literal, never `localhost` / `*.localhost`. It is applied in `fetch`, so it covers the page, every hop and every image.
  - Half 2 (`PublicResolver`): drops loopback, private, link-local, unspecified, broadcast and 0/8 addresses (IPv4-mapped IPv6 unwrapped; ULA `fc00::/7`, link-local `fe80::/10`) and refuses a host with nothing left as `blocked`. The live client also refuses non-public IP literals.
  - The DNS lookup is made by the client itself, so the address it connects to is the one that was checked (no rebinding window).
  - `build` gives the reader `PublicOnly` and the providers `Any`.
- **A17·C4(a)/(b), shop hosts.** `PublicListing($id)` / `ShopHosts`, with no `webPresences` and no `product` in the unlinked query. Hosts deduplicated, `onlineStoreUrl` null → `None`, tail `products/<handle>`. The id is a gid, consistent with `PRODUCT_READ`.
- **A17·C5(a), unit names.** `unit_names` (seven names → codes) only on dimension and weight. 10 in → `{"value":"254","unit":"MILLIMETERS"}`. `refresh_schema` runs at every startup (`lib.rs` 847, `health.rs` 36), so existing shops gain `unit_names` without a migration.
- **A17·C6(a), wired app.** The own listing with a locale prefix is excluded by the handle tail. It is never requested and is listed as "this item's own listing". The other page is read with site class `shop`, and the run is `Finished`.
- **A17·C7(a), selection.** Empty secret store in every case. Default ids, and `""` resolved to them in both `build` and the snapshot; `"nope"` → model `None`.

**Seams probed by reading.**
- Deviation 1 (lazy client) is correct and sufficient. reqwest's panic on building a client inside a runtime is debug-only (`blocking/wait.rs` `enter()`). Dropping a blocking client only joins its runtime thread, so the old adapters dropped by a settings save cannot panic. The research runner is a plain `std::thread`, so sends never run inside a runtime.
- `ResearchFiles::new(paths.root)` matches the service's `data_dir`, so listing images are found.
- A missing search key is already a prerequisite (`runs.rs` 190).

**Assumptions (flow §6).** None contradicts the intention:
- Integer and decimal answers allow numbers only, which is narrower than MC-A2's accepted output and allowed.
- An unreadable research copy fails the run as Request, which is consistent with MC-A9's classes.
- The rest are delegated choices, recorded in plain words in the narrative.

**Hard constraints.**
- H1: Shopify is read-only here.
- H2–H5: not touched.
- H6: research stays optional.
- H7 / HA5: no core file changed; vendor names only in `research/src/{adapters, config.rs}`.
- H8: no Tauri in business logic.
- Rule 9: only loopback listeners on port 0. Keys: test constants only, none in fixtures or commits.

**Over-engineering test.** No test asserts call counts, private helpers or query text. The `webPresences` and `product` substring checks are the row's own stated outcomes.

## Findings
None blocking (B1–B4). No should-fix.

## Notes written to backlog
In `docs/implementation/backlog_research/R8_research_real_adapters.md` (pointer line in `docs/implementation/backlog.md`):
1. The guard could also refuse CGNAT `100.64/10`, `192.0.0/24`, `198.18/15`, multicast, `fec0::/10`, NAT64 and 6to4. The contract is met; this is cheap hardening.
2. reqwest reads `HTTP(S)_PROXY` / `ALL_PROXY` by default. Through a proxy, half 2 of the guard does not apply (half 1 still does). `.no_proxy()` on the `PublicOnly` client closes it.
3. Page bodies are decoded as UTF-8 whatever their charset, and any content type is parsed as HTML. Older `windows-1252` dealer pages would read garbled.
4. Plan lesson: `master_plan_research.md` §3 "Isolation" lists only `{writes, fakes}` for the testkit, but R8's rows need `adapters::page_reader`, `rules::value_map` and `store` in the app suite.

Also recorded: R7's notes on the escaped secret form and the `wait_ends_at` backend test are closed by A15·C5(d) and A15·C7(a).

## Lessons for the plans
- Name the testkit's R8 test surface in `master_plan_research.md` §3 (note 4).
- For the address guard, the next contract text should list the ranges explicitly (note 1), or say "every non-global address".

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1 guard ranges | orchestrator, with the guard contract (`master_plan_research.md` §8) |
| 2 env proxies | same as 1 |
| 3 page charset | R9 (live benchmark will show whether dealer pages are affected) |
| 4 isolation list | orchestrator (master plan text) |

## Write perimeter of this session
- `docs/implementation/plans/R8_research_real_adapters.md`: Tracker line, Review log entry
- `docs/implementation/backlog_research/R8_research_real_adapters.md` (new)
- `docs/implementation/backlog.md`: one pointer line appended
- `docs/implementation/narratives/R8_research_real_adapters.md` (new)
- this handoff

No production code, test or fixture was touched. No probes: flow §4 forbids edits, and none were made. No database or state side effects. No commits.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
