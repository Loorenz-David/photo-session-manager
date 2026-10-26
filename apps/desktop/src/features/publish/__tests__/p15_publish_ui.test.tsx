import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';

vi.mock('@/services/publish', () => ({ publishPreview: vi.fn(), publishConfirm: vi.fn(), publishRetry: vi.fn(), notificationsList: vi.fn(), onPublishProgress: vi.fn() }));
const delivered = vi.hoisted(() => ({ sessions: [] as (() => void)[], progress: [] as (() => void)[] }));
vi.mock('@/services/sessions', () => ({ previewUrl: (path: string) => path, listSessions: vi.fn(), createSession: vi.fn() }));
vi.mock('@/services/events', () => ({ onSessionsChanged: vi.fn(async (handler: () => void) => { delivered.sessions.push(handler); return () => undefined; }), onWorkspaceEvents: vi.fn(async () => () => undefined), onFileDrop: vi.fn(async () => () => undefined) }));
vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => 'shopify'), onExportProgress: vi.fn(async () => () => undefined) }));
vi.mock('@/features/workspace/Stage', () => ({ Stage: () => <div /> }));
vi.mock('@/features/workspace/ImportDropZone', () => ({ ImportDropZone: () => <div /> }));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn(), dismiss: vi.fn() } }));
vi.mock('@/services/photos', () => ({ getSessionView: vi.fn() }));
vi.mock('@/services/attributes', () => ({ getForm: vi.fn() }));
vi.mock('@/features/form/FormDialog', () => ({ FormDialog: ({ queueStrip, queueConfirm, publishSlot }: { queueStrip: React.ReactNode; queueConfirm: () => void; publishSlot: React.ReactNode }) => <div>{queueStrip}<button onClick={queueConfirm}>Confirm &amp; next</button>{publishSlot}</div> }));

import { PublishPopover } from '../PublishPopover';
import { PublishProgress } from '../PublishProgress';
import { publishPreview, publishConfirm, publishRetry, notificationsList, onPublishProgress } from '@/services/publish';
import { getSessionView } from '@/services/photos';
import { getForm } from '@/services/attributes';
import { BatchQueue } from '../BatchQueue';
import { SessionsPage } from '@/features/sessions/SessionsPage';
import { WorkspacePage } from '@/features/workspace/WorkspacePage';
import { NotificationsDrawer } from '../NotificationsDrawer';
import { usePublish } from '../usePublish';
import { listSessions } from '@/services/sessions';
import { toast } from 'sonner';

afterEach(() => { cleanup(); vi.clearAllMocks(); delivered.sessions.length = 0; delivered.progress.length = 0; delete (window as Window & { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__; });

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

const listRow = (label: { kind: 'description_failed' } | { kind: 'publishing'; value: number }, publish: unknown = null) => ({ id: 4, sku: 'P15', item_type: 'Chair', title: 'Oak chair', main_count: 1, condition_count: 0, unreviewed_count: 0, status: label.kind === 'publishing' ? 'publishing' : 'failed', label, progress: label.kind === 'publishing' ? label.value : null, last_activity_at: Date.now(), cover_preview_path: null, publish });
const list = (row: ReturnType<typeof listRow>) => ({ groups: [{ label: 'Today', rows: [row] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 0 });

it('C2(a) refreshes the row after publish progress and session change to Publishing 42%', async () => {
  let percent = 10;
  vi.mocked(listSessions).mockImplementation(async () => list(listRow({ kind: 'publishing', value: percent })) as Awaited<ReturnType<typeof listSessions>>);
  vi.mocked(notificationsList).mockResolvedValue([]);
  vi.mocked(onPublishProgress).mockImplementation(async handler => { delivered.progress.push(() => handler({ session_id: 4, item_id: 7, step: 'upload', uploaded: 2, total: 4 })); return () => undefined; });
  (window as Window & { __TAURI_INTERNALS__?: object }).__TAURI_INTERNALS__ = {};
  function HeaderProgress() { const current = usePublish(4); return current.label?.kind === 'publishing' ? <PublishProgress value={current.label.value} /> : null; }
  render(<><SessionsPage navigate={() => undefined} /><HeaderProgress /></>);
  await waitFor(() => expect(screen.getAllByText('Publishing 10%')).toHaveLength(2));
  percent = 42;
  await waitFor(() => expect(delivered.progress).toHaveLength(1));
  await act(async () => delivered.progress.forEach(handler => handler()));
  await act(async () => delivered.sessions.forEach(handler => handler()));
  await waitFor(() => expect(screen.getAllByText('Publishing 42%')).toHaveLength(2));
  expect(screen.getAllByRole('progressbar').map(bar => bar.getAttribute('aria-valuenow'))).toEqual(['42', '42']);
});

it('C3(b) retries the failed publish item from its session row', async () => {
  vi.mocked(listSessions).mockResolvedValue(list(listRow({ kind: 'description_failed' }, { item_id: 9, step: 'failed' })) as Awaited<ReturnType<typeof listSessions>>);
  vi.mocked(publishRetry).mockResolvedValue();
  render(<SessionsPage navigate={() => undefined} />);
  expect(await screen.findByText('Description failed')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
  await waitFor(() => expect(publishRetry).toHaveBeenCalledWith(9));
});

function workspaceView(uploadState: string | null, failed = true) {
  return { session: { id: 4, item_id: 40, published_at: null, published_seq: 0, change_seq: 0, dirty: false, last_activity_at: 0, created_at: 0, photo_count: 1, unreviewed_count: 0, main_count: 1, condition_count: 0, publish: failed ? { item_id: 7, step: 'failed', failed_step: 'upload', error: 'Upload timed out', uploaded: 0, total: 1 } : null }, item: { id: 40, sku: 'P15', folder_key: 'P15', barcode: null, item_type: 'Chair', title: 'Oak chair', origin: 'local', external_id: null, created_at: 0, updated_at: 0 }, photos: [{ id: 1, session_id: 4, classification: 'main', position: 1, captured_at: 0, original_preview_path: '/a.jpg', adjusted_preview_path: '/a.jpg', orientation: 1, source: 'capture', integrity: 'ok', upload_state: uploadState }] };
}

it('C3(a) shows the failed reason and sticky Retry for the workspace item', async () => {
  vi.mocked(getSessionView).mockResolvedValue(workspaceView(null) as Awaited<ReturnType<typeof getSessionView>>);
  vi.mocked(publishRetry).mockResolvedValue();
  render(<WorkspacePage sessionId={4} assignOpen={false} onAssignOpenChange={() => undefined} navigate={() => undefined} />);
  expect(await screen.findByText('Upload timed out')).toBeInTheDocument();
  expect(screen.getByText('Nothing was overwritten')).toBeInTheDocument();
  expect(toast.error).toHaveBeenCalledWith('Publish failed', expect.objectContaining({ duration: Infinity, action: expect.objectContaining({ label: 'Retry' }) }));
  fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
  await waitFor(() => expect(publishRetry).toHaveBeenCalledWith(7));
});

it.each([['C4(a)', 'ready', 'On Shopify'], ['C4(b)', 'none', 'Not uploaded'], ['C4(c)', 'uploaded', 'Uploading']])('%s shows %s on a workspace photo', async (_row, state, label) => {
  vi.mocked(getSessionView).mockResolvedValue(workspaceView(state) as Awaited<ReturnType<typeof getSessionView>>);
  render(<WorkspacePage sessionId={4} assignOpen={false} onAssignOpenChange={() => undefined} navigate={() => undefined} />);
  expect(await screen.findByText(label)).toBeInTheDocument();
});

it('C6(a) opens the failed session and retries its publish item from notification history', async () => {
  vi.mocked(notificationsList).mockResolvedValue([]);
  vi.mocked(publishRetry).mockResolvedValue();
  const opened = vi.fn();
  render(<NotificationsDrawer onClose={() => undefined} onOpen={opened} notifications={[{ id: 1, item_id: 10, session_id: 3, run_id: 2, kind: 'published', text: 'Published', created_at: 1 }, { id: 2, item_id: 11, session_id: 4, run_id: 3, kind: 'failed', text: 'Publish failed', created_at: 2 }]} />);
  expect(screen.getAllByText(/Published|Publish failed/)).toHaveLength(2);
  fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
  expect(publishRetry).toHaveBeenCalledWith(11);
  fireEvent.click(screen.getAllByRole('button', { name: 'Open' })[1]);
  expect(window.location.hash).toBe('#/session/4');
  expect(opened).toHaveBeenCalledWith(4);
});
