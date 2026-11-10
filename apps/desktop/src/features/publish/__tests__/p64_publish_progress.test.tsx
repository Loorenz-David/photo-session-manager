import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';

vi.mock('@/services/publish', () => ({ publishActivity: vi.fn(), publishCancel: vi.fn() }));

import { PublishProgress } from '../PublishProgress';
import { StatusPill } from '@/features/sessions/StatusPill';
import { publishActivity, publishCancel, type PublishActivityDto } from '@/services/publish';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

afterEach(() => { cleanup(); vi.clearAllMocks(); });

const activity = (over: Partial<PublishActivityDto> = {}): PublishActivityDto => ({
  item_id: 7, step: 'upload', text: 'Uploading condition photos 2 of 3', percent: 56, elapsed_ms: 65_000,
  error: null, retry: null, can_cancel: true, ...over,
});

async function openPopover() {
  render(<PublishProgress value={56} sessionId={4} />);
  expect(screen.getByText('Publishing 56%')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Publish progress details' }));
  return screen.findByTestId('publish-activity');
}

it('C3(a) the progress popover names the current step and the elapsed time', async () => {
  vi.mocked(publishActivity).mockResolvedValue(activity());
  await openPopover();
  expect(await screen.findByText('Uploading condition photos 2 of 3')).toBeInTheDocument();
  expect(screen.getByText('56% · Elapsed 1:05')).toBeInTheDocument();
  expect(publishActivity).toHaveBeenCalledWith(4);
  expect(screen.queryByRole('alert')).toBeNull();
});

it('C3(a) a failed upload shows the error and its retry state', async () => {
  vi.mocked(publishActivity).mockResolvedValue(activity({ error: 'provider: upload unavailable', retry: 'retrying' }));
  await openPopover();
  const alert = await screen.findByRole('alert');
  expect(alert).toHaveTextContent('Last error: provider: upload unavailable');
  expect(alert).toHaveTextContent('Retrying after this error');
});

it('C3(a) a publish waiting for Retry says so', async () => {
  vi.mocked(publishActivity).mockResolvedValue(activity({ step: 'failed', text: 'Publish failed', error: 'provider: upload unavailable', retry: 'retry_available', can_cancel: false }));
  await openPopover();
  expect(await screen.findByRole('alert')).toHaveTextContent('Stopped · press Retry to continue');
  expect(screen.queryByRole('button', { name: 'Cancel publish' })).toBeNull();
});

it('C3(b) Cancel publish asks to confirm, then cancels the item', async () => {
  vi.mocked(publishActivity).mockResolvedValue(activity({ text: 'Uploading main photos 3 of 8' }));
  vi.mocked(publishCancel).mockResolvedValue();
  await openPopover();
  fireEvent.click(await screen.findByRole('button', { name: 'Cancel publish' }));
  expect(publishCancel).not.toHaveBeenCalled();
  expect(screen.getByRole('group', { name: 'Confirm cancel' })).toHaveTextContent('Photos already on Shopify stay there');
  fireEvent.click(screen.getByRole('button', { name: 'Keep publishing' }));
  expect(screen.queryByRole('group', { name: 'Confirm cancel' })).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Cancel publish' }));
  fireEvent.click(screen.getByRole('button', { name: 'Stop publish' }));
  await waitFor(() => expect(publishCancel).toHaveBeenCalledWith(7));
});

it('C3(b) a cancelled publish reads "Publish cancelled · Publish"', () => {
  const onPublish = vi.fn();
  render(<StatusPill label={{ kind: 'publish_cancelled' }} onPublish={onPublish} />);
  expect(screen.getByText('Publish cancelled')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Publish' }));
  expect(onPublish).toHaveBeenCalled();
});
