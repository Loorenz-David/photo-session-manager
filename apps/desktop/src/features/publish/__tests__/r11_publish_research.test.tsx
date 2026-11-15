import '@testing-library/jest-dom/vitest';
import { act, cleanup, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import type { PreviewDto } from '@/services/publish';

vi.mock('@/services/publish', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/publish')>()), publishPreview: vi.fn(), publishConfirm: vi.fn(),
}));
vi.mock('@/services/events', () => ({ onResearchChanged: vi.fn() }));
vi.mock('@/services/sessions', () => ({ previewUrl: (path: string) => path }));
vi.mock('sonner', () => ({ toast: { error: vi.fn() } }));

import { PublishPopover } from '../PublishPopover';
import { publishPreview } from '@/services/publish';
import { onResearchChanged } from '@/services/events';

const line = 'Research is still running for this item. Its suggestions are not in this publish unless you wait.';
function preview(researching: boolean, n_withheld = 0): PreviewDto {
  return { summary: 'Publish one image', items: [{ session_id: 5, main: [1], condition: [], main_thumbnails: ['/1.jpg'],
    condition_thumbnails: [], details_filled: 0, details_total: 1, unreviewed: 0, announced_removals: [],
    create_item: true, n_withheld, conflicts: [], researching }] };
}
let deliver: ((id: number) => void) | undefined;
afterEach(() => { cleanup(); vi.clearAllMocks(); deliver = undefined; });

it('a25_c26a_research_line_does_not_disable_publish', async () => {
  vi.mocked(publishPreview).mockResolvedValue(preview(true));
  vi.mocked(onResearchChanged).mockImplementation(async handler => { deliver = handler; return () => undefined; });
  render(<PublishPopover sessionIds={[5]} onClose={() => undefined} />);
  const item = await screen.findByText(line);
  expect(item.closest('[data-session="5"]')).not.toBeNull();
  expect(screen.getByRole('button', { name: 'Publish 1 images' })).toBeEnabled();
});

it('a25_c26b_research_event_reloads_and_shows_withheld', async () => {
  vi.mocked(publishPreview).mockResolvedValueOnce(preview(true)).mockResolvedValue(preview(false, 1));
  vi.mocked(onResearchChanged).mockImplementation(async handler => { deliver = handler; return () => undefined; });
  render(<PublishPopover sessionIds={[5]} onClose={() => undefined} />);
  await screen.findByText(line);
  await waitFor(() => expect(deliver).toBeDefined());
  act(() => deliver!(5));
  await screen.findByText('1 low-confidence AI suggestion will not be published');
  expect(screen.queryByText(line)).not.toBeInTheDocument();
  expect(publishPreview).toHaveBeenCalledTimes(2);
  expect(within(screen.getByRole('dialog')).getByRole('button', { name: 'Publish 1 images' })).toBeEnabled();
});
