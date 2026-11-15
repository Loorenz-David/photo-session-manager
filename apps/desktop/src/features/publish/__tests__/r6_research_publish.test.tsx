import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { PreviewConflict, PreviewDto } from '@/services/publish';

vi.mock('@/services/publish', async importOriginal => {
  const real = await importOriginal<typeof import('@/services/publish')>();
  return { ...real, publishPreview: vi.fn(), publishConfirm: vi.fn(), publishActivity: vi.fn(), publishCancel: vi.fn() };
});
vi.mock('@/services/attributes', () => ({ resolveConflict: vi.fn() }));
vi.mock('@/services/photos', () => ({ getSessionView: vi.fn() }));
vi.mock('@/services/sessions', () => ({ previewUrl: (path: string) => path }));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn() } }));

import { PublishPopover } from '../PublishPopover';
import { StatusPill } from '@/features/sessions/StatusPill';
import { publishConfirm, publishPreview } from '@/services/publish';
import { resolveConflict } from '@/services/attributes';
import { getSessionView } from '@/services/photos';
import { toast } from 'sonner';

type Item = PreviewDto['items'][number];

function item(session_id: number, extra: Partial<Item> = {}): Item {
  return { session_id, main: [1], condition: [], main_thumbnails: ['/1.jpg'], condition_thumbnails: [], details_filled: 3, details_total: 3, unreviewed: 0, announced_removals: [], create_item: false, n_withheld: 0, conflicts: [], researching: false, ...extra };
}
function preview(items: Item[], summary = 'Publish session 7: one image.'): PreviewDto {
  return { summary, items };
}
const DESIGNER: PreviewConflict = { key: 'designer', label: 'Designer', prior_value: '"Unknown"', prior_valid: true, prior_provenance: 'default', suggestion: '"Hans J. Wegner"', confidence: 'medium' };

beforeEach(() => {
  vi.mocked(getSessionView).mockImplementation(async id => ({ session: { id, item_id: 11 }, item: { id: 11 }, photos: [] }) as unknown as Awaited<ReturnType<typeof getSessionView>>);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('a20_c1a_withheld_line_once_under_the_item_that_has_withheld_suggestions', async () => {
  vi.mocked(publishPreview).mockResolvedValue(preview([item(1, { n_withheld: 2 }), item(2)]));
  render(<PublishPopover sessionIds={[1, 2]} onClose={() => undefined} />);
  const lines = await screen.findAllByText('2 low-confidence AI suggestions will not be published');
  expect(lines).toHaveLength(1);
  expect(lines[0].closest('[data-session]')).toHaveAttribute('data-session', '1');
  expect(screen.queryByText(/low-confidence AI suggestion/, { selector: '[data-session="2"] *' })).not.toBeInTheDocument();
});

it('a20_c2a_conflict_block_shows_prior_origin_suggestion_actions_and_confirm_stays_enabled', async () => {
  vi.mocked(publishPreview).mockResolvedValue(preview([item(7, { conflicts: [DESIGNER] })]));
  render(<PublishPopover sessionIds={[7]} onClose={() => undefined} />);
  const block = await screen.findByRole('group', { name: 'Conflicts' });
  const conflict = within(block).getByText('Designer').closest('[data-conflict]') as HTMLElement;
  expect(conflict).toHaveTextContent('Previous: Unknown · preset default');
  expect(conflict).toHaveTextContent('Suggested: Hans J. Wegner · medium confidence');
  for (const name of ['Keep previous', 'Use suggestion', 'Edit']) expect(within(conflict).getByRole('button', { name })).toBeEnabled();
  expect(within(block).getByText('This item waits until you resolve 1 conflict')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Publish 1 images' })).toBeEnabled();
});

it('a20_c2b_prior_from_the_shop_reads_from_shopify', async () => {
  vi.mocked(publishPreview).mockResolvedValue(preview([item(7, { conflicts: [{ ...DESIGNER, prior_provenance: 'integration' }] })]));
  render(<PublishPopover sessionIds={[7]} onClose={() => undefined} />);
  const block = await screen.findByRole('group', { name: 'Conflicts' });
  expect(block).toHaveTextContent('Previous: Unknown · from Shopify');
  expect(block).not.toHaveTextContent('preset default');
});

it('a20_c3a_keep_previous_resolves_and_rereads_the_preview', async () => {
  vi.mocked(publishPreview).mockResolvedValueOnce(preview([item(7, { conflicts: [DESIGNER] })])).mockResolvedValue(preview([item(7)]));
  vi.mocked(resolveConflict).mockResolvedValue({} as Awaited<ReturnType<typeof resolveConflict>>);
  render(<PublishPopover sessionIds={[7]} onClose={() => undefined} />);
  const block = await screen.findByRole('group', { name: 'Conflicts' });
  fireEvent.click(within(block).getByRole('button', { name: 'Keep previous' }));
  await waitFor(() => expect(resolveConflict).toHaveBeenCalledWith(11, 'designer', 'keep_previous', undefined));
  expect(getSessionView).toHaveBeenCalledWith(7);
  await waitFor(() => expect(screen.queryByRole('group', { name: 'Conflicts' })).not.toBeInTheDocument());
  expect(publishPreview).toHaveBeenCalledTimes(2);
  expect(resolveConflict).toHaveBeenCalledTimes(1);
});

it('a20_c3b_confirm_receives_exactly_the_shown_set_and_the_shown_lines', async () => {
  const summary = 'Publish session 7: one image.';
  vi.mocked(publishPreview).mockResolvedValue(preview([item(7, { n_withheld: 1, conflicts: [DESIGNER] })], summary));
  vi.mocked(publishConfirm).mockResolvedValue(1);
  const onClose = vi.fn();
  render(<PublishPopover sessionIds={[7]} onClose={onClose} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Publish 1 images' }));
  await waitFor(() => expect(publishConfirm).toHaveBeenCalledTimes(1));
  expect(publishConfirm).toHaveBeenCalledWith(
    [7],
    `${summary}\n1 low-confidence AI suggestion will not be published\nThis item waits until you resolve 1 conflict`,
    [{ session_id: 7, n_withheld: 1, conflict_keys: ['designer'] }],
  );
  await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
});

it('a20_c3c_changed_suggestions_refusal_is_shown_and_the_preview_rebuilt', async () => {
  const sentence = 'AI suggestions changed while you were confirming';
  vi.mocked(publishPreview).mockResolvedValueOnce(preview([item(7, { n_withheld: 1 })])).mockResolvedValue(preview([item(7, { n_withheld: 2 })]));
  vi.mocked(publishConfirm).mockRejectedValue(Object.assign(new Error(sentence), { code: 'suggestions_changed' }));
  const onClose = vi.fn();
  render(<PublishPopover sessionIds={[7]} onClose={onClose} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Publish 1 images' }));
  expect(await screen.findByRole('alert')).toHaveTextContent(sentence);
  await waitFor(() => expect(publishPreview).toHaveBeenCalledTimes(2));
  expect(await screen.findByText('2 low-confidence AI suggestions will not be published')).toBeInTheDocument();
  expect(screen.getByRole('dialog', { name: 'Review publish' })).toBeInTheDocument();
  expect(onClose).not.toHaveBeenCalled();
  expect(toast.error).not.toHaveBeenCalled();
});

it('a20_c4a_waiting_for_you_row_label', () => {
  render(<StatusPill label={{ kind: 'waiting_for_you', value: 2 }} sessionId={5} />);
  expect(screen.getByText('Waiting for you · 2 conflicts')).toBeInTheDocument();
  cleanup();
  render(<StatusPill label={{ kind: 'waiting_for_you', value: 1 }} sessionId={5} />);
  expect(screen.getByText('Waiting for you · 1 conflict')).toBeInTheDocument();
});
