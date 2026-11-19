---
plan: W4_writer_settings
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-19
actor: opus (pipeline-reviewer)
---
# Handoff — W4_writer_settings, review r1

**Gate.** The plan's Tracker line read IMPLEMENTED. Checkpoints `23a9668` (U1) and `82da541` (U2) and the closing commit
`0effd5b` are all in `git log`. Gate passed. Process: `docs/process/flow.md` wins (no mutation probes, blocking only
B1–B4). No production code was edited, not even temporarily.

**Perimeter.** `git diff --stat 23a9668^ 0effd5b` lists 21 paths. They equal the handoff's Files changed, including the
declared deviations (`writer/input.rs`, `research/runs.rs`, `research/api_impl.rs`, `t1_settings_drawer.test.tsx`,
`docs/guide/01_product.md`, `docs/guide/02_architecture.md`). Each deviation is needed by the plan's own tasks.

**Checks** (run once, as the plan's Review mode says, on tree `0855e6c`):

| Command | Result |
|---|---|
| `cargo test -p psm-research --lib writer::` | 72 passed |
| `cargo test -p psm-testkit --test suite research::w3_writer_wiring::` | 8 passed |
| `cargo test -p psm-desktop` | 61 passed (`w4_tests` 9/9); `bindings.ts` regenerated with no diff |
| `cargo clippy -p psm-research -p psm-desktop --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | clean |
| `npx vitest run src/features/settings` | 19 files, 96 passed |

The implementer's full-suite stamp (1829 Rust, 659 UI) stands.

## Verified correct
I read the production code behind each row, then its test.
- **C1 (get).** `settings_dto` fills every field from `from_slot`. It resolves each language's instruction (the saved
  text or the shipped one, `is_default` meaning no saved text) and examples. The counts come from `writer_counts`
  (answered and dismissed per type; open rows ignored). C1(a) pins every default. C1(b) also seeds an open row.
- **C2 (versions).**
  - `saved_over` keeps the stored versions and ignores the caller's.
  - It pushes the stored effective text only when the resolved text changes.
  - It stores an instruction or example list equal to the shipped one as "shipped".
  - `push_version` puts the new version first, keeps at most 20, and does nothing when the text is unchanged.
  - C2(c) refuses `language` "German" before anything is written; the test compares the file's bytes.
- **C3 (reset).** `reset_instruction` pushes the saved text into the versions and clears `text`.
- **C4 (preview), one request builder.** The preview and a run build their request through the same three functions:
  `WriterSnapshot::resolve`, `input::assemble` and `request_of`.
  - The run's start uses `resolve`, and `writer_input` ends in `assemble`.
  - `execute` takes its instructions from `request_of`.
  - The OpenAI adapter builds the write schema with the same `schema_for_write(requested(input))` that `request_of` uses.
  - C4(b): the preview makes no run row, no call row and no model request.
- **C5 (the run uses the saved values).** A saved `visible_keys` means the input fields are only the designer. A saved
  `max_questions` 1 means one ask out of three. The saved model and effort appear on the call rows.
- **The save never sends a partial slot.** `save_slot` serialises the full `WriterSettings`. It has no
  `skip_serializing_if`, so `null`s are written too. `SettingsService::set` then replaces the slot's top-level fields.
  The UI sends the whole DTO, and C6(b) asserts the full value.
- **U2 (C6–C10), the Settings → Writer page.**
  - The nav entry sits after Research and is shown with it.
  - The model list has the priced models of the research model provider, plus a saved model with no price, marked
    "(no price)".
  - The editor shows the saved text, or the shipped one when nothing is saved.
  - **Use this** changes only the form. **Reset to default** calls the command and merges that language's instruction
    back into the form.
  - The checklist's default follows WC2's rule (Details and Condition, but no money, media or reference fields, none of
    the writer's own texts, not the condition report key). It stays `null` until a box is touched.
  - Every test asserts the exact saved value or the exact text shown.
- **Assumptions 1–9.** None contradicts 08C §8, WC2 or WC6. All are in the narrative's "Choices made for you".
- **H1–H8 hold.** Nothing is written to an external platform. No device or platform concept was added to core. The
  versions rule lives in the research crate; the commands only read, call and save.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
`docs/implementation/backlog_writer/W4_writer_settings.md`, the reviewer-notes file the writer master plan §3 names
(the prompt's general `backlog.md` is overridden by that plan):
1. The preview shows only saved settings, but the plan's try-out step 1 reads as if it previews an unsaved edit. Put to
   the owner as a choice.
2. **Reset to default** drops an unsaved edit of the same instruction without warning.
3. The plan's "no price" model hazard is handled in code but has no test.
4. Save and reset read the slot, then write it, outside the settings lock. Two saves at the same moment could lose one
   earlier version.
5. Guide `04_change_impact.md` row 27 (the writer command row) does not list the new settings readers.
6. Three notes from W3 are still open: an answer that queues nothing gives no message, sub-cent costs show "$0.00", and
   guide row 28 is out of date.

## Write perimeter (this session)
- `docs/implementation/plans/W4_writer_settings.md` (the Tracker line, and the Review log entry appended)
- `docs/implementation/backlog_writer/W4_writer_settings.md` (new)
- `docs/implementation/narratives/W4_writer_settings.md` (new)
- this handoff

I edited no production or test file and ran no probes. No tool-recorded state was changed. I committed nothing; the
runner commits these files.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
