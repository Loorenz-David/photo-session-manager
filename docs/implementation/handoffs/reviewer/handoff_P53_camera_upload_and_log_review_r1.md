---
plan: P53_camera_upload_and_log
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-08
actor: opus
---
# P53 review r1: APPROVED_WITH_NOTES

Gate passed: the tracker row was `IMPLEMENTED` and checkpoint `75b249b` is in `git log`. No blocking finding (B1–B4). Seven notes went to the backlog. The phase is approved.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Perimeter.** `git show --stat 75b249b` lists exactly the 15 paths the handoff declares: `crates/core/src/mechanisms/{camera_status,capture_ledger}.rs`, `crates/core/src/services/camera_ledger.rs`, `crates/infra/src/ftp/{server,session}.rs`, `crates/infra/src/osascript.rs`, `crates/integrations/src/camera/sidecar/client.rs`, `crates/testkit/tests/{p16_operations,p43_ftp_receiver,p44_capture_ledger,p46_direct_camera_adapter,p53_camera_upload_and_log}.rs`, plus the tracker, the plan and the handoff. All of it is inside the plan's write perimeter. No frontend change was needed: `CameraCard` renders `needs.message` from core, and the health page shows the core detail. The older tests change only where the contract changed an asserted value: `settle` after `226` and the `.done` written path (p43), the new sentence (p44, p46), `c6d` count 1→2 under the max rule (p44), and the subscriber at DEBUG so P16's provider-call assertions still see their lines (p16).
- **Evidence.** Reviewer run on the clean tree `75b249b`. `cargo test --workspace` ran to its last doc-test with no failing binary (p53 7/7). `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `npm run typecheck` and `npm test` (53 files, 404 tests) were all green. Variation: `c3a` run alone is green, so its upload lines come from its own app.
- **C1(a).** `upload` returns after flush, size, then the `.part`→`.done` rename. `serve` replies `226`, logs `completed … reply_ms`, then `tokio::spawn(file_completed)`, which hashes in `spawn_blocking` and then calls `sink.delivery`. Under the old order the test's `stor` would block on the gated sink, so the test discriminates. After release the test checks size and SHA-256.
- **C1(b).** `server.rs` re-files every `.<name>.done` in `<root>/.incoming` before the accept loop. In the test, `stop()` cancels the first filing at its hash await, the marker survives, and the second receiver delivers once with the correct hash. No code path deletes a `.done` file. If filing fails after `226`, the marker stays for the next start (contract).
- **C1(c).** `completed()` appends `.done` files to the names that LIST, NLST, SIZE and MDTM report, so the test gets `213 <size>` while delivery is blocked.
- **Waiting-file rules.** A same-name `STOR` while `.done` waits gets 450, the MC-32 in-flight answer, and the waiting file is never truncated. `upload` refuses to overwrite an existing marker. The `active` reservation (`UploadGuard`) is dropped before the reply and before hashing, so a different name is never refused with 450 (plan hazard). Before the reply, the code waits only on flush, metadata and rename after the data connection closes.
- **C2(a)–(c).**
  - The gap's age is measured from the arrival of its later file, so a partly filled gap keeps its age (`c2b` sees count 1 and then a clear).
  - Bounds are `> WATCHDOG_MIN_AGE_MS` and `< WATCHDOG_MAX_AGE_MS`, and dismissed keys are excluded.
  - The count reaches the header through `WatchdogState.pending_count` → `runtime.rs` → `set_files_not_received`, which is the watchdog's own path.
  - A counter reset means a number below every number of that prefix in the window. Without that rule, `9998, 0002` reports 9995 missing, so `c2c` discriminates.
  - The sentence equals the plan's text in both `status_message` and the health detail.
- **C3(a).** Every parsed sidecar stdout event becomes `camera.event <name> k=v …` at INFO on target `psm::camera`. Top-level `password`, `user`, `pass`, `credentials` and `fingerprint` become `***` before the registered-secret redaction. The protocol carries the SSH fingerprint only in the `fingerprint` key, and `camera.ftp_state` fields are property names and values. The receiver logs `started`, `completed name size ms reply_ms`, `failed name reason` and `filed name outcome hash_ms`. `camera.sequence_gap` is logged only when the gap list changes. In osascript, `provider call` is DEBUG and failures and timeouts are WARN.
- **Assumptions** (handoff §Assumptions), read against 10E MC-32/MC-33: the `.done` marker is a delegated choice; the gap clock, the reset rule and the max count are judgment calls inside "no double counting"; redaction is as the plan lists. None contradicts the intention. They are explained to the owner in the narrative.
- **H1–H8.**
  - H1 and H6: no external write.
  - H2: only the receiver's own staging file is renamed. No RAW in a candidate folder is touched, and nothing is deleted.
  - H3: no pixels are processed.
  - H4: a crash after `226` recovers at the next start, and a repeat delivery is absorbed by first-copy-wins.
  - H5: no change.
  - H7: the domain enum stays generic. See note 2 on the remedy text.
  - H8: logic lives in core and infra, with no Tauri dependency.
  - No secret is logged, and no live service is used in the suite.

## Findings
None blocking. Every row has a test at its boundary that fails on a wrong outcome.

## Notes written to backlog
1. Capture One provider calls at DEBUG leave 12 §Logs "every provider call with duration" untrue for the INFO-filtered file log. Route: plan/book. The owner asked for the change in P52 F2, so the book line should follow.
2. The Sony remedy text ("switch FTP Function off and on") sits in core `camera_status` and `camera_ledger`, as the plan contract dictates. Route: plan. The adapter should supply device remedies.
3. The `max(exposures, missing numbers)` count undercounts when the two are distinct shots.
4. A `.done` recovered at start is delivered with `completed_at` = the recovery time, which affects FIFO association.
5. Filing now overlaps the next transfer, so two filings can take the worker lock out of arrival order (FIFO swap risk under load).
6. Dismissal is keyed by the gap's ends, so a partly filled dismissed gap raises again.
7. `c3a` uses a global subscriber shared with other tests in the binary. `c1b` would hang if the hash finished before `stop()` (blocked sink plus runtime drop).

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1, 2 | orchestrator: book 12 §Logs amendment / adapter-owned remedy text at the next camera planning act |
| 3, 6 | P52 run 2 observation; plan only if the owner sees a wrong count |
| 4, 5 | P54 or a later camera backend phase (marker mtime; one filing queue) |
| 7 | the next phase touching `p53_camera_upload_and_log.rs` |

## Lessons for the plans
- A plan that changes a log level should name the book line it amends. Here 12 §Logs now disagrees with the code, and the plan said "no other book change".
- A user-facing remedy that names a device menu belongs to the adapter's text, not to core's `status_message`. The contract placed it in core.

## Write perimeter (this session)
- `docs/implementation/master_plan.md`: P53 tracker row only.
- `docs/implementation/plans/P53_camera_upload_and_log.md`: Review log entry.
- `docs/implementation/backlog.md`: 7 appended notes.
- `docs/implementation/narratives/P53_camera_upload_and_log.md`: new.
- `docs/implementation/handoffs/reviewer/handoff_P53_camera_upload_and_log_review_r1.md`: this file.
- No production or test file was edited. No probes, no state changes beyond `target/` build artifacts. No commits.
