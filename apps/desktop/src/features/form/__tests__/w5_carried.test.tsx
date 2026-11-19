import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import type { WriterStatus } from '@/services/writer';

// W5·U4 rows W5·C12(a) (an answer that queues nothing says why) and W5·C13(a) (money below one cent). Boundary: the
// `WriterBlock` rendered with `services/writer` and Sonner mocked; `money` called directly.

vi.mock('@/services/writer', () => ({
  writerStatus: vi.fn(),
  writerFindingAnswer: vi.fn(),
  writerFindingDismiss: vi.fn(),
}));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { toast } from 'sonner';
import { WriterBlock } from '@/features/form/WriterBlock';
import { money } from '@/features/form/WriterPanel';
import { writerFindingAnswer } from '@/services/writer';

const QUESTION = 'Is the frame teak or oak?';
const BUSY = 'Research is running on this session.';

function status(): WriterStatus {
  return {
    reason: null, run: { id: 9, state: 'finished', trigger: 'write', stage_reached: 'write', error: null }, texts: [], attempts: [], regenerated: [],
    findings: [{ id: 11, kind: 'question', type: 'question', keys: ['wood'], text_field: '', text: QUESTION, mode: 'ask' }],
    steps: [], cost_micro_usd: 0, inputs: { fields: 0, note: 0, listings: [] }, stale_main: false, stale_condition: false, can_write: true, finished_at: null, not_offered: [],
  };
}

afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('w5_c12a_an_answer_that_queues_nothing_shows_the_reason_as_a_toast', async () => {
  vi.mocked(writerFindingAnswer).mockResolvedValue({ queued: false, reason: { code: 'research_busy', sentence: BUSY } });
  render(<WriterBlock status={status()} onChanged={async () => undefined} />);
  fireEvent.change(screen.getByRole('textbox', { name: `Answer: ${QUESTION}` }), { target: { value: 'Teak' } });
  fireEvent.click(screen.getByRole('button', { name: 'Answer' }));
  await waitFor(() => expect(toast.error).toHaveBeenCalledWith(BUSY));
  expect(writerFindingAnswer).toHaveBeenCalledWith(11, 'Teak');
});

it('w5_c13a_money_below_one_cent_is_less_than_a_cent_and_zero_is_zero', () => {
  expect(money(4_000)).toBe('< $0.01');
  expect(money(0)).toBe('$0.00');
  expect(money(21_000)).toBe('$0.02');
});
