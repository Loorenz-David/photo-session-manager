---
plan: P11_form_ui
role: fix
round: 2
state: DONE
date: 2026-10-26
actor: sonnet
---
# Handoff — P11_form_ui, fix 1

Gate: tracker showed P11_form_ui `CHANGES_REQUESTED`. Passed.

## Finding → what changed → files
**F1 (B1, production): the form's Title was a second copy (`product.title`), not the item's title.**
- `form_get` now presents `items.title` as the `product.title` field's value (human, valid; an empty title is no value).
- A human `attribute_values_set` on `product.title` sets `items.title` (canonical text, unchanged title not written,
  sessions' activity set, `change_seq` untouched) and stores no value row. `attribute_ai_clear` empties `items.title`.
- New `StoreTx::update_item_title`, SQLite implementation.
- New service test: set the Title → `session_get`'s item, the item row and `form_get` all show it, no `product.title`
  row exists, activity moved; clearing empties it.
- Form: the placeholder workaround is gone. A Title save (blur, Save & close) calls `onItemChanged`, which reloads the
  workspace, so the header follows. One form test covers it (value shown, no placeholder, write, reload).
- Files: `crates/core/src/services/attributes.rs`, `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`,
  `crates/testkit/tests/p11_title_binding.rs` (new), `apps/desktop/src/features/form/FormDialog.tsx`,
  `apps/desktop/src/features/form/__tests__/p11_form_ui.test.tsx`.

## Divergences
None from the correction. Three small choices inside it (also in the plan's Review log): a non-human `set_value` on
`product.title` is refused; `prefill_defaults` skips a preset default for `product.title`; a non-text value is refused
(`invalid_value`). `WorkspacePage.tsx` (optional in the finding) needed no edit. `ai_suggest` on `product.title` is not
changed (it would store a row that `form_get` ignores); noted for the reviewer, not a finding I was asked to fix.
No P10 test pinned `product.title` as a stored row, so no P10 test changed.

## Files changed (this session)
The six files above, plus `docs/implementation/master_plan.md` (P11 tracker row only) and
`docs/implementation/plans/P11_form_ui.md` (Review log "fix 1"), and this handoff. No probe files (no mutation regime).

## Checkpoint SHA
`4fbc0f9` (code, tracker, plan log; this handoff is committed by the runner).

## Stamp
Tree clean after the commit. `npm run typecheck` exit 0 · `npm test` 4 files, 78/78 · `cargo test --workspace` 236 passed,
0 failed (was 235; +1) · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all -- --check` exit 0.
The desktop package has no separate lint script; clippy and fmt are the lint.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
