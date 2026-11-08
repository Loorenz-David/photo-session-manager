---
plan: P51b_fingerprint_exact_confirm
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-08
actor: opus
---

# P51b review r1: APPROVED_WITH_NOTES

**Gate.** Tracker row `IMPLEMENTED`; checkpoint `6a06110` ("CHECKPOINT (not approved): P51b exact fingerprint confirmation") is in `git log`. Passed.

## Verified correct

- **Perimeter.** `git show --stat 6a06110` lists 25 paths, exactly the handoff's declared list. Two are outside the plan's estimate and disclosed as deviations; both are necessary: `native/camera-sidecar/tests/test_transport_manager.cpp` declares and calls the policy checks, and `apps/desktop/src-tauri/src/lib.rs` forwards `replace` in `camera_fingerprint_pending` and updates its P50 payload test. No SDK file, build output, `.env` or real fingerprint is committed. The tree was clean after the stub build.
- **Stamp (reviewer run, code tree `6a06110`).** `cargo test --workspace`: 820 passed, 0 failed (p51b 4/4, p50 green). `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `npm run typecheck`: exit 0. `npm test`: 53 files, 404/404. `tools/sidecar/build.sh --stub --test`: `21 tests, 0 failed`.
- **Helper decision** (`fingerprint_policy.cpp`), read against the contract's full case table:
  - a pin equal to the raw key gives `Match` before the expectation is touched, so it survives;
  - otherwise the expectation is taken and reset in every branch, which consumes it on `Pin`/`Replace` and drops it on `Mismatch`/`Unknown`;
  - a differing pin gives `Replace` only when `replace && fp == display`;
  - with no pin, the result is `Pin` only when `fp == display` (either `replace`) or the CLI flag is set.

  The CLI flag cannot override a differing pin, as before. The mutex covers both `arm` and `decide`.
- **`check_fingerprint`.**
  - It writes the pin file only for `Pin`/`Replace`, and the old unconditional `unlink` on `replace` is gone.
  - It keeps the raw pin-file format and emits `match`/`MISMATCH`/`unknown`/`pinned` with the display form.
  - The success flags are unchanged: Match is true, Mismatch and Unknown are false, and a pin is true.
- **Command parsing.** `main.cpp` decodes `fp` once with `percent_decode(c.get("fp"))`, the same pattern as `user`/`password`; `c.get` returns raw text, so nothing is decoded twice. An empty value gives `cmd.rejected` with reason `fingerprint required`. The weak stub (`main.cpp`) and the strong definition (`camera_connection.cpp`) have the same parameter list `(const std::string&, bool)` and the same return type.
- **Rust.**
  - Wire format: `percent_encode` turns `:` into `%3A`, giving `accept-fingerprint fp=… [replace=1] cmd_id=…`.
  - `confirm_fingerprint(shown)`: with nothing pending it returns `FingerprintNotPending`. If `shown` differs from the pending value it returns `FingerprintChanged` before any send, and the pending value stays. If they are equal it sends with the pending `replace`.
  - `replace` comes from `status == "MISMATCH"`, both in the adapter's state and in `StepOutcome.fingerprint_pending`. The runtime emits `AppEvent::FingerprintPending { fingerprint, replace }`, and `items.rs` maps the new error to `fingerprint_changed`.
- **UI.** `CameraSection`'s Confirm passes the rendered `fingerprint` state. The card is cleared only after the awaited command resolves, so a `fingerprint_changed` rejection shows the exact toast and leaves the card. The event sets the value, `fingerprintChanged = replace` and the needs-action flag.
- **Criterion rows (13), each with a boundary test that fails on a wrong outcome:**
  - C1(a)–(d) and C2(a) call the pure policy, and each consume or drop is proven by a follow-up `decide`.
  - C3(a)–(d) assert:
    - the exact stdin prefix `accept-fingerprint fp=ab%3Acd%3Aef cmd_id=`, with no `replace=1` anywhere;
    - `Err(FingerprintChanged)`, with no `accept-fingerprint` line, and that the value is still pending (the later exact confirm succeeds);
    - `fp=ef%3A12 replace=1`;
    - both event payloads.
  - C4(a)–(c) assert the exact argument `'ab:cd:ef'`, the exact toast with the value still shown, and the title `Fingerprint changed` with the value and both buttons after an event that arrives after mount.
  - C4(c) mounts with status `disconnected` instead of the row's "connected". This is equivalent: neither is `needs_action`, so only the event can set the title.
  - C5(a) is green per the stamp.
- **Assumptions.** None contradicts 10E MC-31.
  - The CLI `--accept-fingerprint` first-pin is in the contract.
  - "A matching pin leaves the expectation" is the contract's `Match (expectation untouched)`.
  - "A failed pin-file save still accepts this connection" is the existing behaviour, which this phase does not change.
  - The display-versus-raw comparison is the plan's hazard note.
- **H1–H8.**
  - H7: core gains only a generic `replace: bool` on an existing event and one error code beside the existing `FingerprintNotPending`.
  - H8: the Tauri command stays a pass-through.
  - H2 and H5 are untouched. No live service is used, and the suite needs no camera.
- **MC-31** ("pinned only on their confirmation") now holds end to end. The shown value is compared in the app and in the helper, and a different key is never pinned or replaced.

## Findings

None blocking (B1–B4: none).

## Notes written to backlog

1. **Test structure.** The five policy checks run inside the existing C++ case "credentials percent-decoding reverses the app's encoding", because P45's stub-build test hardcodes `21 tests, 0 failed`. A policy failure would therefore be labelled as a percent-decoding failure, and the fixed count blocks later phases from adding cases. Make P45 check `0 failed` and give the policy its own `TEST_CASE`s.
2. **Link-time hazard.** `accept_fingerprint_on_next_connect` has no shared header. If one side's parameter list changed alone, the SDK build would still link and silently keep the weak no-op, so Confirm would arm nothing. Declare it in a `crlab/` header that both files include.

## Lessons for the plans

- When a phase adds C++ cases, the plan should say whether P45's fixed case count may be updated. This phase had to bend its test layout around it.

## Probe declaration

No probes. No production file was edited, even temporarily. Files this session wrote:
- `docs/implementation/backlog.md` (2 lines appended)
- `docs/implementation/master_plan.md` (P51b row only)
- `docs/implementation/plans/P51b_fingerprint_exact_confirm.md` (Review log)
- `docs/implementation/narratives/P51b_fingerprint_exact_confirm.md` (new)
- this handoff

Build output from `tools/sidecar/build.sh --stub --test` stays in its ignored build folder, and `git status` was clean apart from these files.

## Carry-forward

| Note | Destination |
|---|---|
| P45 fixed case count / policy checks inside the percent-decoding case | next phase that touches `native/camera-sidecar/tests` (P52 or later) |
| shared declaration for `accept_fingerprint_on_next_connect` | next phase that touches the helper |
| N6 "Not mine" keeps retrying (pre-existing, out of scope here) | P52 / backend |

## ⚠ OWNER DECISIONS REQUIRED (0)

none
