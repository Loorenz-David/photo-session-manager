import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';

vi.mock('@/services/publish', () => ({ publishActivity: vi.fn(), publishCancel: vi.fn() }));

import { PublishProgress } from '../PublishProgress';
import { publishActivity, type PublishActivityDto } from '@/services/publish';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

afterEach(() => { cleanup(); vi.clearAllMocks(); });

const activity = (over: Partial<PublishActivityDto> = {}): PublishActivityDto => ({
  item_id: 7, step: 'upload', text: 'Uploading main photos 1 of 2', percent: 42, elapsed_ms: 12_000,
  error: null, retry: null, can_cancel: true, ...over,
});

async function openPopover() {
  render(<PublishProgress value={42} sessionId={4} />);
  fireEvent.click(screen.getByRole('button', { name: 'Publish progress details' }));
  return screen.findByTestId('publish-activity');
}

it('C4(a) the publish popover shows how many masters were ready in advance', async () => {
  vi.mocked(publishActivity).mockResolvedValue(activity({ masters_ready: 'Masters ready in advance: 1 of 2' }));
  const panel = await openPopover();
  expect(await screen.findByText('Masters ready in advance: 1 of 2')).toBeInTheDocument();
  expect(panel).toHaveTextContent('Uploading main photos 1 of 2');
});

it('C4(a) no masters line before the export of the item is known', async () => {
  vi.mocked(publishActivity).mockResolvedValue(activity({ step: 'prepare', text: 'Preparing outputs' }));
  await openPopover();
  expect(await screen.findByText('Preparing outputs')).toBeInTheDocument();
  expect(screen.queryByText(/Masters ready in advance/)).toBeNull();
});
