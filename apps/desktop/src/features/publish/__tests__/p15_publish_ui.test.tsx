import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';

vi.mock('@/services/publish', () => ({ publishPreview: vi.fn(), publishConfirm: vi.fn(), publishRetry: vi.fn(), notificationsList: vi.fn(), onPublishProgress: vi.fn() }));
vi.mock('@/services/sessions', () => ({ previewUrl: (path: string) => path }));
vi.mock('@/services/photos', () => ({ getSessionView: vi.fn() }));
vi.mock('@/services/attributes', () => ({ getForm: vi.fn() }));
vi.mock('@/features/form/FormDialog', () => ({ FormDialog: ({ queueStrip, queueConfirm, publishSlot }: { queueStrip: React.ReactNode; queueConfirm: () => void; publishSlot: React.ReactNode }) => <div>{queueStrip}<button onClick={queueConfirm}>Confirm &amp; next</button>{publishSlot}</div> }));

import { PublishPopover } from '../PublishPopover';
import { PhotoUploadBadge } from '../PhotoUploadBadge';
import { PublishProgress } from '../PublishProgress';
import { publishPreview, publishConfirm } from '@/services/publish';
import { getSessionView } from '@/services/photos';
import { getForm } from '@/services/attributes';
import { BatchQueue } from '../BatchQueue';
import { StatusPill } from '@/features/sessions/StatusPill';

afterEach(() => { cleanup(); vi.clearAllMocks(); });

const preview = { summary: 'Publish session 7: four selected images; remove one old image.', items: [{ session_id: 7, main: [4, 2, 3], condition: [9], main_thumbnails: ['/4.jpg', '/2.jpg', '/3.jpg'], condition_thumbnails: ['/9.jpg'], details_filled: 3, details_total: 8, unreviewed: 2, announced_removals: ['old'], create_item: true }] };

it('C1(a) renders ordered thumbnails, counts, notes and exact summary', async () => {
  vi.mocked(publishPreview).mockResolvedValue(preview);
  render(<PublishPopover sessionIds={[7]} onClose={() => undefined} />);
  expect((await screen.findAllByAltText(/^Main /)).map(img => img.getAttribute('src'))).toEqual(['/4.jpg', '/2.jpg', '/3.jpg']);
  expect(screen.getByAltText('Condition 1')).toHaveAttribute('src', '/9.jpg');
  expect(screen.getByText('Product details 3/8')).toBeInTheDocument();
  expect(screen.getByText('2 unreviewed photos will not be published')).toBeInTheDocument();
  expect(screen.getByText('1 gallery removals')).toBeInTheDocument();
  expect(screen.getByText('A new product will be created')).toBeInTheDocument();
  expect(screen.getByTestId('publish-summary')).toHaveTextContent(preview.summary);
  expect(screen.getByRole('button', { name: 'Publish 4 images' })).toBeInTheDocument();
});

it('C1(b) sends the exact summary displayed in the confirmation surface', async () => {
  vi.mocked(publishPreview).mockResolvedValue(preview);
  vi.mocked(publishConfirm).mockResolvedValue(1);
  render(<PublishPopover sessionIds={[7]} onClose={() => undefined} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Publish 4 images' }));
  await waitFor(() => expect(publishConfirm).toHaveBeenCalledWith([7], preview.summary));
});

it('C2(a) displays the backend-derived publishing percentage in the header progress', () => {
  render(<PublishProgress value={42} />);
  expect(screen.getByText('Publishing 42%')).toBeInTheDocument();
  expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '42');
});

it.each([
  ['C4(a)', 'ready', 'On Shopify'],
  ['C4(b)', 'none', 'Not uploaded'],
  ['C4(c)', 'uploaded', 'Uploading'],
])('%s maps %s to %s', (_row, state, expected) => {
  render(<PhotoUploadBadge state={state} />);
  expect(screen.getByText(expected)).toBeInTheDocument();
});

it('C3(b) describes a failed description and offers Retry', () => {
  const retry = vi.fn();
  render(<StatusPill label={{ kind: 'description_failed' }} onRetry={retry} />);
  expect(screen.getByText('Description failed')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
  expect(retry).toHaveBeenCalledOnce();
});

function queueFixtures() {
  vi.mocked(getSessionView).mockImplementation(async id => ({ session: { id }, item: { id, title: `Item ${id}` }, photos: [] }) as unknown as Awaited<ReturnType<typeof getSessionView>>);
  vi.mocked(getForm).mockResolvedValue({ groups: [{ filled: 1, total: 3 }] } as Awaited<ReturnType<typeof getForm>>);
}

it('C5(a) publishes only the first two confirmed sessions with their fresh preview summary', async () => {
  queueFixtures();
  vi.mocked(publishPreview).mockResolvedValue({ items: [], summary: 'T_AB' });
  vi.mocked(publishConfirm).mockResolvedValue(9);
  render(<BatchQueue ids={[1, 2, 3]} onClose={() => undefined} />);
  fireEvent.click(await screen.findByText('Confirm & next'));
  fireEvent.click(screen.getByText('Confirm & next'));
  fireEvent.click(screen.getByRole('button', { name: 'Publish 2 confirmed' }));
  await waitFor(() => expect(publishConfirm).toHaveBeenCalledWith([1, 2], 'T_AB'));
  expect(publishPreview).toHaveBeenCalledWith([1, 2]);
});

it('C5(b) marks a confirmed queue item and shows its form fill count', async () => {
  queueFixtures();
  render(<BatchQueue ids={[1, 2, 3]} onClose={() => undefined} />);
  await screen.findByText(/Item 1 · 1\/3 filled/);
  fireEvent.click(screen.getByText('Confirm & next'));
  expect(screen.getByText(/✓ Item 1 · 1\/3 filled/)).toBeInTheDocument();
});
