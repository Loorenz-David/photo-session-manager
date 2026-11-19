import '@testing-library/jest-dom/vitest';
import { cleanup, render, screen, within } from '@testing-library/react';
import { afterEach, expect, it } from 'vitest';
import type { WriterStatus } from '@/services/writer';
import { WriterPanel } from '@/features/form/WriterPanel';

// W3·U3 rows W3·C10: the writer's block in the research panel (book 08C §11). Boundary: `WriterPanel` rendered with a
// writer status; the outcome is what it shows.

afterEach(() => cleanup());

function status(extra: Partial<WriterStatus> = {}): WriterStatus {
  return {
    reason: null, run: null, texts: [], attempts: [], regenerated: [], findings: [], steps: [], cost_micro_usd: 0,
    inputs: { fields: 0, note: 0, listings: [] }, stale_main: false, stale_condition: false, can_write: true,
    ...extra,
  };
}

it('w3_c10a_a_finished_write_run_shows_what_it_used_why_what_the_check_found_and_the_cost', () => {
  render(
    <WriterPanel
      status={status({
        run: { id: 9, state: 'finished', trigger: 'write', stage_reached: 'write', error: null },
        inputs: { fields: 12, note: 5, listings: ['Mogensen J39 chair', 'Beech dining chair', 'FDB J39'] },
        texts: [
          { key: 'product.title', label: 'Title', flagged: false, hits: [], reason: 'Maker and model from the fields.', facts: ['Designer', 'Model'], outcome: 'written' },
          { key: 'product.description', label: 'Description', flagged: false, hits: [], reason: 'Form and restoration from the note.', facts: ['Wood'], outcome: 'written' },
        ],
        attempts: [
          { attempt: 1, hits: [{ text: 'description', code: 'banned', detail: 'uses "stunning"' }] },
          { attempt: 2, hits: [] },
        ],
        regenerated: ['description'],
        findings: [{ id: 3, kind: 'finding', type: 'field_vs_field', keys: ['custom.era'], text_field: '', text: 'The era does not fit the designer', mode: 'note' }],
        cost_micro_usd: 21_000,
      })}
    />,
  );
  const block = screen.getByRole('region', { name: 'Writer' });
  expect(block.querySelector('[data-writer-inputs]')).toHaveTextContent('Used 12 fields · 5 observations · 3 listing titles');
  const titles = within(block).getByRole('list', { name: 'Listing titles' });
  for (const title of ['Mogensen J39 chair', 'Beech dining chair', 'FDB J39']) expect(within(titles).getByText(title)).toBeInTheDocument();
  const title = block.querySelector('[data-writer-text="product.title"]')!;
  expect(title).toHaveTextContent('Maker and model from the fields.');
  expect(title).toHaveTextContent('Facts: Designer, Model');
  const description = block.querySelector('[data-writer-text="product.description"]')!;
  expect(description).toHaveTextContent('Form and restoration from the note.');
  expect(description).toHaveTextContent('Facts: Wood');
  expect(block.querySelector('[data-writer-attempt="1"]')).toHaveTextContent('Check 1: description: uses "stunning"');
  expect(block.querySelector('[data-writer-attempt="2"]')).toBeNull();
  expect(within(block).getByText('Rewritten once')).toBeInTheDocument();
  expect(within(block).getByText('Note: The era does not fit the designer')).toBeInTheDocument();
  expect(block.querySelector('[data-writer-cost]')).toHaveTextContent('Cost $0.02');
});

it('w3_c10b_without_a_write_run_there_is_no_writer_block', () => {
  render(<WriterPanel status={status()} />);
  expect(screen.queryByRole('region', { name: 'Writer' })).not.toBeInTheDocument();
  cleanup();
  render(<WriterPanel status={null} />);
  expect(screen.queryByRole('region', { name: 'Writer' })).not.toBeInTheDocument();
});
