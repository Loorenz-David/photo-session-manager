---
plan: P69_faster_export_and_publish
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-12
actor: opus (pipeline-reviewer)
---
# Review — P69 faster export and publish (r1): APPROVED_WITH_NOTES

Gate: tracker `IMPLEMENTED`; checkpoint `b1129e1` present in `git log`. Perimeter: `git show --stat b1129e1` = the 25 paths the
handoff declares (16 production, 7 tests/harness, tracker + plan log). Nothing extra, nothing missing. Tree clean; HEAD `541a014` adds only
pipeline docs. No blocking findings.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
**Checks (reviewer run, code tree = `b1129e1`, one cargo command at a time, foreground).**
| Command | Result |
|---|---|
| `cargo test -p psm-testkit --no-fail-fast --test p69_… --test p27_… --test p56_… --test p09_… --test p44_… --test p61_… --test p62_… --test p64_… --test p65_…` | 90/90 (p69 7, p09 8, p27 11, p44 25, p56 4, p61 7, p62 12, p64 8, p65 8) |
| `cargo test -p psm-core -p psm-infra -p psm-integrations -p psm-desktop --lib` | 133/133 |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `npm run typecheck` | exit 0 |
| `npx vitest run src/features/publish src/features/settings` | 15 files, 63/63 |

The implementer's full-suite stamp (Rust 945/0 across the gate run plus the p56–p69 re-run; UI 466/466) is consumed. Nothing I ran contradicts it.

**Rows.**
- **C1(a).** `RecipeSpec::MASTER` → TIFF, `bits: 8`, `Uncompressed`. The test seeds a 16-bit `PSM Master` in the fake and observes the repair before the master is issued. The output is rendered from an 8-bit master (1 `render_output` job, 1 JPEG, the master's colour). The renderer decodes both `ImageRgb8` and `ImageRgb16`. The test fails if the bit depth stays 16.
- **C2(a).** `PrepareMastersService::tick` → `ExportService::prepare_master` asks for one master and refuses while the RAW is in `state.masters`/`waits`. `busy` reports `preparing` while a prepared master is out. On arrival, `master_ready` → `commit_prepared` puts the master in the cache when no run waits for it. A run that started meanwhile waits for it through `state.masters`. The test observes the masters asked for in order a, b, c with `paused("preparing")` between them, a publish that issues no master, `masters_reused=3/3`, and the masters deleted after the publish.
- **C2(b).** `busy` checks, in order: the app's pause check (capture or FTP upload, take-back gate, free space below the minimum), a prepared master out, an export run, any active publish item, renders in flight, and a stale preview still due. `preview_waiting` mirrors the preview scheduler, skipping failed-for-good and missing RAWs, so it cannot pause forever. The test observes `render`, `publish`, `off` (persisted `false`, applied live through `set_enabled` in the settings apply path) and `capture` (a real loopback FTP upload). After each, the master is requested once the condition ends. The UI test covers the checkbox → `{export:{prepare_masters:false}}`.
- **C2(c).** The master cache key is `(raw, adjusted_gen)` (export crop ignored); the output key is unchanged. The test observes that only the edited photo's master is rendered at publish, the manifest records the current gen and revision, the recomposed photo comes out at 600×400, budget-cleared masters are re-rendered by the publish, and nothing is re-prepared below the minimum (`disk`).
- **C3(a).** `prepare` gate: `earlier_in_prepare` on the same `run_id`, unless the item was started ahead. `start_next_in_batch` runs once an item leaves prepare. A gated item returns `Ok(false)` and the job is retried with no attempt cap and back-off ≤ 15 s (the same path as P65 waiting). The test fails both without overlap (the next item's masters are not asked for during the upload) and with the pre-P69 all-at-once behaviour (two items ahead). Completion order and galleries are asserted.
- **C4(a).** `log_timing` runs at done and writes every field. `activity().masters_ready` is filled only while the item runs. The test parses the INFO line (`masters_reused=1/2`) and reads the activity during the uploads and after. The UI test checks that the line is rendered when present and absent otherwise.
- **C5(a).** The stamp, plus the reviewer checks above.

**Seams read.** A prepared master still with Capture One at quit is an ordinary `render_master` job: `InterruptedExport` abandons it at restart and the job asks for it again (H4). `prepare_failed` is set only for a non-editor-wait failure. The take-back preconditions count prepared masters as running exports (see note 2).

**Assumptions.** A1–A10 are consistent with changelog 47, 10D and 10F. A2 is stricter than the plan: a publish of any session pauses the job, not only another session's. A10 (frame adoption on a prepared master) is the existing master behaviour applied earlier. None is B1.

**Hard constraints.** H1: the overlap only starts local exports of a batch that is already confirmed. H2: masters only, no RAW touched. H3: no new pixel code. H4: see seams. H5: the AppleScript channel is unchanged. H6: the job does not need commerce. H7: `shopify_ms` in core is plan-dictated (note 3). H8: only Tauri DTO fields were added. No live services in tests.

## Findings
None blocking (B1–B4: none).

## Notes written to backlog
1. **Book fold.** 10D MC-28 still says the master is a 16-bit TIFF and is rendered "only when an output is needed". Changelog 47 supersedes both.
2. **Take-back waits for one background master.** It reports "an export is running". The job pauses only while a take-back is held, not while an automatic take-back is pending. The delay is bounded by one render.
3. **H7 family.** The `shopify_ms` field and log key live in the core `PublishService`.

## Lessons for the plans
- A background job that uses Capture One should list the take-back **pending** state (the automatic take-back's "Waiting to take the cable") among its pause conditions, not only a take-back in progress.

## Write perimeter
`docs/implementation/backlog.md` (3 lines appended), `docs/implementation/master_plan.md` (P69 tracker row only),
`docs/implementation/plans/P69_faster_export_and_publish.md` (Review log entry), `docs/implementation/narratives/P69_faster_export_and_publish.md`
(new), this handoff. No production or test file touched; no probes (flow §4); nothing committed.
