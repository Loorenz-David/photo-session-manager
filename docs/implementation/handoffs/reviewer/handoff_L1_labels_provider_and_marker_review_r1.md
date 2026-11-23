---
plan: L1_labels_provider_and_marker
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-23
actor: opus
---
# L1 review r1: CHANGES_REQUESTED (1 blocking finding)

The labels provider, the access rule, the display copy, the label events in the pull, the command, the Settings line,
the markers and the disabled ⋯ entry are built as the plan and contracts say. All checks are green. One defect blocks:
the session page asks for its marker **without** refresh when the same session is reopened after another page,
against MC-L3.7 and 09C §7 ("asks for its one product when opened"). The fix is a few lines in the marker hook or the
header plus one extra input in an existing test.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Gate
- Plan Tracker line read `IMPLEMENTED`. Checkpoints `28395bc` (U1), `793950f` (U2), `499b734` (U3), `8870b3f` (U4),
  `06318b5` (close) are all in `git log`.
- Review tree: `167d95d` (the close checkpoint plus the runner's handoff and prompt commits, docs only).
  `git status --porcelain` was clean at entry and at exit.

## Perimeter
- `git diff --stat 28395bc^ 06318b5`: 64 paths. They equal the handoff's declared write perimeter, including the
  declared closing-gate fix `crates/testkit/tests/suite/p01_storage_identity.rs` (outside the plan's Files, column list
  only; see notes). `test_app.rs` and `services/conversations.ts` were listed in the plan but needed no change
  (declared). No undeclared path.

## Checks (Review mode, each run once, one cargo command at a time)
| Command | Result |
|---|---|
| `cargo test -p psm-core --lib mechanisms::labels` | 8 passed |
| `cargo test -p psm-integrations --lib conversations::beyo_pricing` | 95 passed |
| `cargo test -p psm-testkit --test suite l1_labels_access::` | 9 passed |
| `cargo test -p psm-testkit --test suite l1_labels_markers::` | 15 passed |
| `cargo test -p psm-testkit --test suite c1_conversations_connection::` | 19 passed |
| `cargo test -p psm-desktop` | 97 passed (bindings regenerated: no diff) |
| `npx vitest run src/features/{labels,sessions,shell,settings,workspace,publish,conversations,pending,compose}` | 104 files, 704 passed |
| `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit -p psm-desktop --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | 0 errors |

The implementer's full-suite stamp (`cargo test --workspace` 1358 + p01 fix re-run, `npm test` 129 files / 867)
stands: nothing contradicts it.

## Verified correct
- **U1 (C1–C6).** `label_error` is the only error function of label calls and equals master plan §4 "Status mapping"
  row by row (401/403, 404 by call, 409, 410, 422 `no_products` / other, 503 by code, other 5xx, other status with /
  without an envelope). Transport failures map to `Unreachable`. No label path calls `read_error` / `write_error` or
  returns `ProviderError` (ACC.5). Query encoding, `filter` omitted for `All`, `status` asks nothing for `[]` and
  refuses a short answer. `labelStatus` strict, reasons tolerant. `describe` words exact. `product_id_of` /
  `commerce_id_of` accept one or more ASCII digits only. Each test asserts the whole value or the exact request list;
  C10(b)'s inputs are each refused by their own predicate (prefix, `?`, `..`), so the row can fail.
- **U2 (C7–C10).** `plan` through `send`; `preview`, `run`, `download` through `fetch` with `FILE_TIMEOUT` /
  `RUN_TIMEOUT`; bodies carry `productId` only (plus the key for runs); `%PDF-` check on both file answers;
  `lifecycleResult` tolerant (D4), run `status` and plan `outcome` strict; the download path check matches MC-L2.12
  and refuses a backslash and `%2e` segments too (assumption 3, only stricter). `run`, `plan`, `preview` and
  `download` have no caller outside the adapter's tests (M-L2).
- **U3 (C11–C14).** `label.printed` / `label.reset` map by MC-L4.1 (data id, else envelope id, else `Other`); label
  events are never `for_us`, so the pull treats them as "not for us" (cursor moves in the lane, no mark, no
  `applied`). `pull` is now a wrapper around `pull_page` that hands the collected changes to `LabelsLink::changed`
  on every way out (MC-L4.3); `first_connection` is unchanged. `connect` calls `connected()` after the lane guard's
  scope ends, only when `Ready`; `key_set` calls `key_changed()` before anything else, on both branches (C12(i)).
  `connected()` applies only ACC.2 and emits exactly one `LabelsChanged` (D2); every other label call goes through
  `learn` (ACC.4). `LabelCopy` gets `clock.mono()` (D9); a `None` answer keeps its answered-at time (D10);
  `clear()` drops answers to requests started before it (assumption 5). The only schema change is
  `conversation_sync.labels_access` (default `unknown`); nothing deletes that row. `session_products` joins sessions
  to `item_links` of the commerce integration. Core names no Pricing, Shopify or `gid` (H7). Logs carry counts and the
  access word only. Fixtures carry neutral values and a `_source` key. The service tests run on the real wiring
  (`TestApp::with_pricing`) and assert exact request lines, events, cursor and stored value.
- **U4 (C15–C19).** `labels_status` is thin (`spawn_blocking`, one service call plus `access()`); DTOs and error
  codes match master plan §4 Commands; `settings_dto.labels` comes from `access()`; the bridge emits
  `labels_changed {product_ids, access}`. `markerText` follows MC-L3.5's first-match order with fixed English months
  (D13). Rows render before the markers answer; the id-list rule on the Sessions page (C18(c)); `labels_changed`
  re-asks without / with refresh (C18(d)); a failed call leaves rows and no toast (C18(e)). `LabelMenuItem` follows
  D6's first-match order and D5's hint source. The UI tests assert exact call lists and exact words.
- **Existing tests.** Changed only where allowed: `c1_c9a` request list gains the templates line (D3); `p27` gains
  the `DROP COLUMN` line; `p01` gains the column; two typed settings fixtures gain `labels: 'off'` (D7). The adapter's
  inline test module was split into `testing` (the recording transport) and `tests` with no assertion changed.
- **H1–H8.** H1: the only writing call has no caller. H6: without a ready connection the service answers `Off` and
  makes no request. H7 as above. H8: business logic in `psm-core`, the command is thin. H2–H5 untouched.
- **Orphan tests.** None: every new test is named after a row, and the 8 unit tests are the plan's core mechanisms.

## Findings

### F1 (blocking, B1) · route: production
- **What is wrong.** The session page asks for its marker with `refresh: false` when the same session is opened again
  after another page. `Header` is one persistent element across routes (`App.tsx` renders it for both routes), and
  `useSessionLabels` keeps the Sessions page's id-list rule ("refresh only when the id list differs from this
  mount's last refreshing call"). Open session 5 → `labelsStatus({[5], refresh: true})`; go to the list (ids
  `null`, no call); open session 5 again → `labelsStatus({[5], refresh: false})`. Verified with a temporary vitest
  probe (copied the `l1_markers` mocks, rendered `Header` with 5 → undefined → 5, printed the calls:
  `[{"session_ids":[5],"refresh":true},{"session_ids":[5],"refresh":false}]`; file deleted after the run).
  With `refresh: false` the backend answers only the display copy and never asks Pricing, so a change that sends no
  event (a price set on a "No price yet" product), or a first read that failed, is not picked up on reopening.
- **Authority.** `master_plan_labels.md` §5 MC-L3.7: "The session page calls it for its one session with
  `refresh = true` when opened." Book 09C §7: "The session page asks for its one product when opened." Row L1·C18(f)
  states the first open only.
- **Correction.** Make the session page ask with `refresh: true` every time a session is opened, including a reopen
  of the same session (for example: the hook forgets its last refreshing id list when its ids become `null`, or the
  header passes a per-open flag); keep the Sessions page's id-list rule (L1·C18(c)) as it is. Extend `l1_c18f` with
  the reopen input (5 → no session → 5: the second call has `refresh: true`).
- **Files the fix may touch.** `apps/desktop/src/features/labels/useSessionLabels.ts`,
  `apps/desktop/src/features/shell/Header.tsx`, `apps/desktop/src/features/labels/__tests__/l1_markers.test.tsx`;
  the plan's Tracker line and Review log; the fix handoff.

## Assumptions (handoff) assessed
None contradicts 09C or the contracts. For the owner, in plain words (narrative at approval):
1. A Pricing error without its usual error body is treated as "Pricing answered in a way this version does not
   understand".
2. The label file download refuses a few more disguised addresses than the plan listed (backslash, encoded dots).
3. A label in a generation result whose outcome word is unknown counts as not rendered (it shows as failed, never as
   done).
4. Test connection switching stores while markers are being read: the late answer is dropped.
5. Removing or replacing the key, and learning that a key may read labels, do not refresh the markers on screen by
   themselves (see backlog note 1).
6. The ⋯ hint is a second line under "Generate label"; printed markers are neutral, the others amber; the header
   marker follows the title's other pills.
Assumptions 1, 6 and 11 (where code lives, how the fake logs) are technical.

## Notes written to backlog
`docs/implementation/backlog_labels/L1_labels_provider_and_marker.md` (the plan's and master plan §3's reviewer-notes
path for this track; the prompt's generic `backlog.md` was not used):
1. Markers stale after key removal / server change / a failed connect: no `LabelsChanged` (ACC.3 gap) → orchestrator
   fold before L2.
2. `useSessionLabels`' try/catch exists for test mocks → L2·U3, L3·U4 tests mock explicitly.
3. Hidden ids re-asked on `labels_changed {access: true}` after a page empties → L2·U3.
4. `p01_storage_identity` is the second `ADD COLUMN` tripwire (guide updated) → L2/L3 plans list it.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1 | orchestrator (ACC.3 fold) before L2 |
| 2 | L2·U3, L3·U4 |
| 3 | L2·U3 |
| 4 | L2, L3 plans (Files) |

## Lessons for the plans
- A UI row whose contract says "when opened" names the reopen input too; L1·C18(f) had only the first open, so the
  hook's reuse of the list's rule passed its test.
- ACC.3 should say whether `key_changed()` tells the screen (note 1).

## Probe declaration
- One temporary test file, `apps/desktop/src/features/labels/__tests__/zz_review_probe.test.tsx`, created, run once,
  deleted; `git status --porcelain` clean afterwards. No production file touched, no database or state changed.

## Write perimeter (this session)
- `docs/implementation/plans/L1_labels_provider_and_marker.md` (Tracker line, Review log entry)
- `docs/implementation/backlog_labels/L1_labels_provider_and_marker.md` (new)
- this handoff
- No narrative (written at approval). No commits (the runner commits).
