# Session Manager — build flow (v1, 2026-10-24)

This file is the project's process authority. The global pipeline doctrine in `~/agent-skills/`
still applies, **except where this file says otherwise; here, this file wins.** The master plan
imports it as standing rules, so every role session reads it. It will change as we learn; every
change is a dated entry in the log at the end.

## 1. Goal of the flow
Ship the MVP in days, not weeks, with the owner involved only at phase ends. Each phase runs
unattended from first prompt to approval. Questions that come up during a phase are answered
from the intention book or decided by the implementer and recorded, never asked. The owner reads
one plain-language narrative per phase and can try the result by hand.

## 2. Roles, runners, models
| Role | How it runs | Model |
|---|---|---|
| Orchestrator | this Claude session; starts runs, reads results, answers blocked sessions from the book | Fable |
| Mechanism inventory | `claude -p --agent pipeline-inventory` | Opus 5.5 |
| Planner | `claude -p --agent pipeline-planner` | Fable |
| Implementer | `claude -p --agent pipeline-implementer --model opus`, effort `medium`, or `high` when the plan marks the phase `difficulty: high` (owner 2026-10-25: Opus replaces Codex 6 Sol; `PSM_IMPLEMENTER=codex` restores `codex exec -m gpt-6-sol`) | Opus 5.5 |
| Fix | `claude -p --agent pipeline-implementer --model sonnet`, effort `high` (owner 2026-10-25: Sonnet replaces Codex 6 Luna; `PSM_IMPLEMENTER=codex` restores `codex exec -m gpt-6-luna`) | Sonnet 5.5 |
| Reviewer, re-reviewer | `claude -p --agent pipeline-reviewer` | Opus 5.5 |
| Projection | off by default; `projection: yes` in a plan's frontmatter turns it on (`pipeline-projectionist`, Opus) | Opus 5.5 |
| Tester | **not declared.** The implementer writes the behaviour tests; the reviewer audits them. | — |

The orchestrator never polls. It starts `tools/pipeline/run_phase.sh` in the background and is
woken once, when the runner exits.

## 2A. The orchestrator
A fresh Claude session woken by the owner with `docs/process/prompts/wake_orchestrator.md`. Its
standing instructions are `docs/process/orchestrator.md`; its memory between sessions is
`docs/implementation/STATUS.md`.

## 3. The mechanical bridge
`tools/pipeline/run_phase.sh` is a shell state machine. It spends no tokens between sessions.
For one phase it: fills a prompt template from the plan file, commits the prompt, launches the
session, waits, reads the handoff's frontmatter (the chat line is the fallback), and moves on:

```
implement (Sol) → review (Opus) → APPROVED? done
                               → CHANGES_REQUESTED → fix 1 (Luna) → re-review (Opus)
                                                       → APPROVED / APPROVED_WITH_NOTES → done
                                                       → CHANGES_REQUESTED with MUST items → stop, orchestrator
```
Exit codes: `0` approved (approval commit made), `10` the orchestrator must look (blocked
session, owner cards, MUST findings after fix 1, projection amendments), `20` runner error.
`run_phase.sh all` runs `docs/implementation/phases.txt` in order and stops at the first
non-zero exit. Logs and raw session output live in `.pipeline/runs/` (not committed); prompts,
handoffs, narratives and state live in `docs/implementation/` (committed by the runner).

## 4. Tests: inputs and outcomes
This section corrects the most expensive part of earlier projects.
- **One behavioural test per criterion row**, at the boundary the row names: a service call, a
  command handler, a produced file, stored state, the calls a fake adapter received. It asserts
  the exact outcome for a given input. That is the whole proof.
- **Granular unit tests only for core mechanisms the plan lists by name** (status derivation
  order, idempotency keys, RAW hash, position math, gallery reconciliation diff, attribute type
  mapping). Still input → output, on the real function.
- **Never:** internal call counts, private helpers, generated query text, mocks of internal
  collaborators, snapshot of internals, "tests for confidence".
- **The named-mutation regime is off.** No mutation ledgers, no arming, no `executed == declared`
  gates, and reviewers never edit production code, not even temporarily, to see a test fail. A
  reviewer judges a test by reading the code it exercises and, if in doubt, by running the test
  with a changed input.
- **The over-engineering test:** if a developer changing this behaviour would have to change
  this test for any reason other than the behaviour changing, the test is the defect.

## 5. Review verdicts and the fix budget
Blocking (CHANGES_REQUESTED) **only** for:
- **B1** behaviour contradicts the intention or the plan at a boundary a user or an integration can observe;
- **B2** a criterion row has no test at its boundary, or its test cannot fail on a wrong outcome (asserts nothing, asserts the fixture, accepts several outcomes);
- **B3** a hard constraint H1–H8 or the secrets / no-live-services rules are violated;
- **B4** the build, suite, lint or typecheck is red.
Everything else is a **note** written to `docs/implementation/backlog.md`. Notes never trigger a
fix cycle. Verdicts: `APPROVED`, `APPROVED_WITH_NOTES`, `CHANGES_REQUESTED`.

**Fix budget: one automatic fix per phase.** If the re-review still finds blocking items, the
re-reviewer triages each: **MUST** (user-visible wrong outcome, hard constraint, red suite) or
**DROP** (style, more tests, refactor, speculative robustness). No MUST left → the verdict is
`APPROVED_WITH_NOTES`. Any MUST → the runner stops and the orchestrator decides: one more fix,
authorised by writing `FIX2_AUTHORISED: <reason>` in the plan's Review log and relaunching, or a
card for the owner. Never more than two fixes.

## 6. No questions during a phase
- The implementer never asks. A judgment call the plan leaves open is decided as the smallest
  option consistent with the intention chapter the plan cites, and recorded under
  **Assumptions** in the handoff. Only a contradiction between plan and intention, or a task that
  cannot be done as written, is `BLOCKED`.
- A blocked session or an owner card goes to the orchestrator, who answers from the book when the
  book answers it, and otherwise puts a decision card in the phase narrative. Owner decisions are
  batched at phase ends.
- The reviewer reads the assumptions: one that contradicts the intention is a B1 finding; the
  rest are reported to the owner in plain words.

## 6A. Hardware in sessions
Sessions may fire the shutter themselves through Capture One's `capture` command (the same
channel the adapter uses) to produce real captures for probes and manual checks. Preconditions
the owner sets once: camera tethered and awake (auto power-off disabled), a Capture One session
document open. Scripted frames land in the active capture folder and are never deleted (H2).
**The automated suite never depends on the camera or on Capture One being present**; live
behaviour is checked by probes whose results are recorded in the evidence doc and the Review log.

## 7. What the owner receives
At approval the reviewer writes `docs/implementation/narratives/<phase>.md`:
**What you can do now** · **Try it yourself** (numbered steps on this Mac) · **Choices made for
you** (each assumption in one sentence, and what to say to change it) · **Not yet** · **Needs you**
(cards verbatim, or "Nothing"). No plan references, no finding ids, no file paths outside the
steps. The orchestrator relays it verbatim. The owner reviews phases at their own pace; the
runner does not wait for them unless started with `--pause`.

## 8. Commits
Implementer and fix sessions make `CHECKPOINT (not approved):` commits of their perimeter only.
The runner commits prompts, handoffs, narratives and state, and makes the `APPROVED <phase>`
commit on exit 0. Pushing is the orchestrator's act at the end of a run.

## 9. Planning rules that make unattended phases possible
Phases ≤ 8 criteria; ordered so the **local workflow** (shoot, sort, live previews, export) is
usable before Shopify; each plan file carries frontmatter `phase`, `title`, `difficulty`,
`projection`, `depends_on`, a **Core mechanisms** list (the only place unit tests are allowed), a
**Delegated choices** section, and a **Manual try-out** section the narrative is built from.
Criteria rows name boundary, input, exact outcome and trace (M1–M7 or a mechanism contract).

## Log
- **2026-10-24 v1.2.** Orchestrator entry point (§2A), `archive` command, second fix by explicit authorisation.
- **2026-10-24 v1.1.** §6A: sessions may trigger captures through Capture One; suite stays hardware-free.
- **2026-10-24 v1.** Created from the owner's brief: Rust/Tauri stack, no tester role, no
  mutation regime, behaviour tests at boundaries, one automatic fix then triage, mechanical
  bridge, narratives for the owner, models per role.
