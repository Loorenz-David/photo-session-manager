// W1·U3 (rows W1·C14(a)–(c), book 08C §4): the product note in the research panel, with `services/writer.ts` mocked.
import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { NoteObservation, ProductNote } from '@/services/writer';

vi.mock('@/services/writer', () => ({ writerNote: vi.fn(), writerLookAgain: vi.fn() }));

import { ResearchPanel } from '@/features/form/ResearchPanel';
import { writerLookAgain, writerNote } from '@/services/writer';

const SESSION = 9;

function observation(ref: string, text: string, owner: boolean, photo: string | null = null): NoteObservation {
  return { ref, original: text, text, keys: [], photo, owner, decisions: [], in_writer_input: true };
}
function note(extra: Partial<ProductNote> = {}): ProductNote {
  return { observations: [], research_saw: [], stale_main: false, stale_condition: false, missing_main: false, ...extra };
}
const panel = () =>
  render(<ResearchPanel sessionId={SESSION} status={null} fields={[]} focusKey={null} onFocus={() => undefined} onRun={async () => undefined} onAcceptAll={async () => undefined} onClose={() => undefined} />);

beforeEach(() => {
  vi.mocked(writerLookAgain).mockResolvedValue({ queued: true, reason: null });
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('w1_c14a lists every observation and marks only the owner decision', async () => {
  vi.mocked(writerNote).mockResolvedValue(note({
    observations: [
      observation('p1.1', 'Wood: Teak (owner\'s decision)', true),
      observation('c1.1', 'Scratch on the left arm', false, 'condition photo 1'),
    ],
  }));
  panel();
  const block = await screen.findByRole('region', { name: 'What research saw' });
  expect(within(block).getByRole('heading', { name: 'What research saw' })).toBeInTheDocument();
  const items = within(block).getAllByRole('listitem');
  expect(items).toHaveLength(2);
  expect(items[0]).toHaveTextContent('Wood: Teak (owner\'s decision)');
  expect(within(items[0]).getByText('Your decision')).toBeInTheDocument();
  expect(items[1]).toHaveTextContent('Scratch on the left arm');
  expect(items[1]).toHaveTextContent('condition photo 1');
  expect(within(items[1]).queryByText('Your decision')).toBeNull();
  expect(writerNote).toHaveBeenCalledWith(SESSION);
});

it('w1_c14b shows the changed-photos banner and Look again asks once for this session', async () => {
  vi.mocked(writerNote).mockResolvedValue(note({ stale_main: true, observations: [observation('p1.1', 'Frame in oak', false)] }));
  panel();
  expect(await screen.findByText('Photos changed since they were looked at')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Look again' }));
  await waitFor(() => expect(writerLookAgain).toHaveBeenCalledTimes(1));
  expect(writerLookAgain).toHaveBeenCalledWith(SESSION);
});

it('w1_c14c with no note yet offers Look again', async () => {
  vi.mocked(writerNote).mockResolvedValue(note({ missing_main: true }));
  panel();
  const block = await screen.findByRole('region', { name: 'What research saw' });
  expect(within(block).getByText('No product note yet')).toBeInTheDocument();
  expect(within(block).getByRole('button', { name: 'Look again' })).toBeInTheDocument();
  expect(within(block).queryByRole('listitem')).toBeNull();
});
