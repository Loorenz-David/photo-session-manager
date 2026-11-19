import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionResearch } from '@/services/research';
import type { WriterStatus } from '@/services/writer';

// A24 (owner 2026-11-19, "perfect, we can build that"): the owner can tell the writer is working, finished or failed.
// Boundary: the writer's block in AI review, its block in the research panel, the session pill and the app-wide toasts,
// rendered with `services/writer`, `services/research` and `services/events` mocked; the outcome is what is shown, the
// toasts raised and the service calls made.

let refresh: (() => void) | undefined;
vi.mock('@/services/writer', () => ({ writerRequest: vi.fn(), writerStatus: vi.fn(), writerFindingAnswer: vi.fn(), writerFindingDismiss: vi.fn() }));
vi.mock('@/services/research', () => ({ researchOverview: vi.fn() }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async (handler: () => void) => {
    refresh = handler;
    return () => undefined;
  }),
}));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { toast } from 'sonner';
import { WriterBlock, notOfferedSentence } from '@/features/form/WriterBlock';
import { WriterPanel, lastWritten } from '@/features/form/WriterPanel';
import { WriterPill, WriterToasts } from '@/features/sessions/ResearchPill';
import { researchOverview } from '@/services/research';
import { writerRequest } from '@/services/writer';

beforeEach(() => {
  vi.clearAllMocks();
  refresh = undefined;
});
afterEach(() => cleanup());

function status(extra: Partial<WriterStatus> = {}): WriterStatus {
  return {
    reason: null, run: null, texts: [], attempts: [], regenerated: [], findings: [], steps: [], cost_micro_usd: 0,
    inputs: { fields: 0, note: 0, listings: [] }, stale_main: false, stale_condition: false, can_write: true,
    finished_at: null, not_offered: [], ...extra,
  };
}

const run = (state: string, error: string | null = null) => ({ id: 32, state, trigger: 'write', stage_reached: 'write', error });

it('a24_a_failed_write_says_why_and_try_again_asks_for_texts', async () => {
  vi.mocked(writerRequest).mockResolvedValue({ queued: true, reason: null });
  const reload = vi.fn(async () => undefined);
  render(<WriterBlock status={status({ run: run('failed', 'the model did not answer') })} onChanged={reload} sessionId={7} />);
  const alert = screen.getByRole('alert');
  expect(alert).toHaveTextContent('Writing texts failed: the model did not answer.');
  fireEvent.click(within(alert).getByRole('button', { name: 'Try again' }));
  await waitFor(() => expect(writerRequest).toHaveBeenCalledWith(7, 'write_texts'));
  await waitFor(() => expect(reload).toHaveBeenCalled());
  expect(toast.success).toHaveBeenCalledWith('Writing texts. They arrive in AI review.');
});

it('a24_a_finished_write_lists_the_texts_it_did_not_offer_in_plain_words', () => {
  render(
    <WriterBlock
      status={status({
        run: run('finished'),
        not_offered: [
          { key: 'product.description', label: 'Description', outcome: 'the photographer cleared this field' },
          { key: 'custom.condition_report_description', label: 'Condition Report Description', outcome: 'the photographer accepted this value' },
        ],
      })}
      onChanged={async () => undefined}
    />,
  );
  const list = screen.getByRole('list', { name: 'Texts not rewritten' });
  expect(within(list).getByText('Description was not rewritten: you cleared it. Press Write texts to get a new one.')).toBeInTheDocument();
  expect(within(list).getByText('Condition Report Description was not rewritten: you accepted the current text. Press Write texts to get a new one.')).toBeInTheDocument();
  expect(notOfferedSentence('Title', 'already holds this text')).toBe('Title was not rewritten: it came out the same as the current text.');
});

it('a24_while_writing_nothing_stale_is_listed', () => {
  render(<WriterBlock status={status({ run: run('running'), not_offered: [{ key: 'k', label: 'Description', outcome: 'no text' }] })} onChanged={async () => undefined} />);
  expect(screen.getByText('Writing texts…')).toBeInTheDocument();
  expect(screen.queryByRole('list', { name: 'Texts not rewritten' })).not.toBeInTheDocument();
  expect(screen.queryByRole('alert')).not.toBeInTheDocument();
});

it('a24_the_research_panel_says_when_the_writer_last_wrote_and_how_it_ended', () => {
  const at = Date.UTC(2026, 10, 19, 14, 30);
  render(<WriterPanel status={status({ run: run('failed', 'x'), finished_at: at })} />);
  const line = document.querySelector('[data-writer-last]')!;
  expect(line).toHaveTextContent(/^Last written: .+ · failed$/);
  expect(lastWritten('finished', at)).toMatch(/ · finished$/);
  expect(lastWritten('stopped_at_cap', at)).toMatch(/ · stopped$/);
  expect(lastWritten('running', null)).toBe('Writing now…');
});

function research(writer: SessionResearch['writer'], session_id = 4): SessionResearch {
  return { session_id, state: 'done', position: null, to_review: 0, reason: null, writer };
}

it('a24_the_session_pill_shows_writing_and_failed_only', () => {
  const { rerender } = render(<WriterPill research={research({ state: 'writing', sku: 'Sdt1-191126' })} />);
  expect(screen.getByText('Writing texts…')).toBeInTheDocument();
  rerender(<WriterPill research={research({ state: 'failed', sku: 'Sdt1-191126' })} />);
  expect(screen.getByText('Writing failed')).toBeInTheDocument();
  rerender(<WriterPill research={research({ state: 'done', sku: 'Sdt1-191126' })} />);
  expect(screen.queryByText(/Writing/)).not.toBeInTheDocument();
});

it('a24_a_toast_only_when_a_run_seen_writing_ends', async () => {
  // On load the run is already over: no toast.
  vi.mocked(researchOverview).mockResolvedValue([research({ state: 'done', sku: 'A-1' }, 3), research({ state: 'writing', sku: 'Sdt1-191126' }, 4), research({ state: 'writing', sku: 'B-2' }, 5)]);
  render(<WriterToasts />);
  await waitFor(() => expect(refresh).toBeDefined());
  await waitFor(() => expect(researchOverview).toHaveBeenCalledTimes(1));
  expect(toast.success).not.toHaveBeenCalled();
  vi.mocked(researchOverview).mockResolvedValue([research({ state: 'done', sku: 'A-1' }, 3), research({ state: 'done', sku: 'Sdt1-191126' }, 4), research({ state: 'failed', sku: 'B-2' }, 5)]);
  await act(async () => refresh!());
  await waitFor(() => expect(toast.success).toHaveBeenCalledWith('Texts ready for Sdt1-191126 — see AI review'));
  expect(toast.success).toHaveBeenCalledTimes(1);
  expect(toast.error).toHaveBeenCalledWith('Writing texts failed for B-2 — open AI review to try again');
  // The same states again: nothing new.
  await act(async () => refresh!());
  await waitFor(() => expect(researchOverview).toHaveBeenCalledTimes(3));
  expect(toast.success).toHaveBeenCalledTimes(1);
  expect(toast.error).toHaveBeenCalledTimes(1);
});
