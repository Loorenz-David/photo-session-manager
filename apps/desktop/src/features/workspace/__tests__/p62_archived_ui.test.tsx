import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import type { SessionView } from '@/services/photos';
import type { ItemArchive } from '@/services/retention';

vi.mock('@/services/photos', () => ({
  getSessionView: vi.fn(),
  classifyPhoto: vi.fn(),
  reorderPhoto: vi.fn(),
  deletePhoto: vi.fn(),
  movePhotos: vi.fn(),
  copyPhotos: vi.fn(),
  importPhotos: vi.fn(),
  assignUnassigned: vi.fn(),
  pickImportFiles: vi.fn(),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/retention', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/retention')>()),
  getItemArchive: vi.fn(),
  setKeepFiles: vi.fn(),
  getRetentionDays: vi.fn(),
  setRetentionDays: vi.fn(),
}));
vi.mock('@/services/events', () => ({
  onWorkspaceEvents: vi.fn(async () => () => undefined),
  onPhotoAdded: vi.fn(async () => () => undefined),
  onFileDrop: vi.fn(async () => () => undefined),
  onSessionsChanged: vi.fn(async () => () => undefined),
}));
vi.mock('@/services/history', () => ({
  historyGet: vi.fn(async () => ({ undo: null, redo: null })),
  historyUndo: vi.fn(),
  historyRedo: vi.fn(),
  editMenuSet: vi.fn(async () => undefined),
  onHistoryChanged: vi.fn(async () => () => undefined),
  onEditMenu: vi.fn(async () => () => undefined),
}));
vi.mock('@/services/previews', () => ({ setPreviewFocus: vi.fn() }));
vi.mock('sonner', () => ({ toast: Object.assign(vi.fn(), { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() }) }));

import { StatusPill } from '@/features/sessions/StatusPill';
import { RetentionSetting } from '@/features/settings/StorageSection';
import { ItemDetailsCard } from '@/features/workspace/ItemDetailsCard';
import { WorkspacePage } from '@/features/workspace/WorkspacePage';
import { getSessionView } from '@/services/photos';
import { FILES_CLEARED, getItemArchive, getRetentionDays, setRetentionDays } from '@/services/retention';

afterEach(() => { cleanup(); vi.clearAllMocks(); });

const item = { id: 7, sku: 'ARC-1', folder_key: 'ARC-1', barcode: null, item_type: 'Sideboard', title: 'Oak sideboard', origin: 'local', external_id: 'gid://shopify/Product/1', created_at: 0, updated_at: 0 };
const view = {
  session: { id: 5, item_id: 7, published_at: 1, published_seq: 3, change_seq: 3, dirty: false, last_activity_at: 0, created_at: 0, photo_count: 2, unreviewed_count: 0, main_count: 2, condition_count: 0, publish: null },
  item,
  photos: [],
} as unknown as SessionView;
const url = (id: string) => `https://cdn.fake-shop.test/files/${id}.jpg`;
const cleared: ItemArchive = {
  item_id: 7, keep_files: false, cleared: true,
  pictures: [
    { role: 'gallery', position: 1, url: url('m1'), path: '/data/previews/archived/7/gallery-01-a.jpg' },
    { role: 'gallery', position: 2, url: url('m2'), path: '/data/previews/archived/7/gallery-02-b.jpg' },
    { role: 'condition', position: 1, url: url('f1'), path: null },
  ],
};

it('C3(a) the session list labels a cleared item "Archived in Shopify"', () => {
  render(<StatusPill label={{ kind: 'archived' }} />);
  expect(screen.getByText('Archived in Shopify')).toBeInTheDocument();
});

it('C3(a)/(b) a cleared item opens on its Shopify photos, cached, with no review controls', async () => {
  vi.mocked(getSessionView).mockResolvedValue(view);
  vi.mocked(getItemArchive).mockResolvedValue(cleared);
  render(<WorkspacePage sessionId={5} assignOpen={false} onAssignOpenChange={() => undefined} navigate={() => undefined} />);
  const archived = await screen.findByRole('main', { name: 'Archived in Shopify' });
  expect(getItemArchive).toHaveBeenCalledWith(5);
  const main = within(archived).getByRole('region', { name: 'Main' });
  expect(within(main).getAllByRole('img').map(img => [img.getAttribute('alt'), img.getAttribute('src')])).toEqual([
    ['Main 01', 'asset:///data/previews/archived/7/gallery-01-a.jpg'],
    ['Main 02', 'asset:///data/previews/archived/7/gallery-02-b.jpg'],
  ]);
  const condition = within(archived).getByRole('region', { name: 'Condition' });
  expect(within(condition).getByRole('img', { name: 'Condition 01 not downloaded' })).toBeInTheDocument();
  expect(within(archived).getByRole('note')).toHaveTextContent(FILES_CLEARED);
  expect(screen.queryByRole('button', { name: /approve|condition|delete|compose/i })).not.toBeInTheDocument();
  expect(screen.queryByRole('complementary', { name: 'Photos' })).not.toBeInTheDocument();
});

it('C4(a) the item card toggles "Keep files on this Mac"', () => {
  const onKeep = vi.fn();
  render(<ItemDetailsCard view={view} onClose={() => undefined} keepFiles={false} onKeepFilesChange={onKeep} />);
  fireEvent.click(screen.getByRole('checkbox', { name: 'Keep files on this Mac' }));
  expect(onKeep).toHaveBeenCalledWith(true);
});

it('Settings → Storage saves the days published items keep their files', async () => {
  vi.mocked(getRetentionDays).mockResolvedValue(4);
  vi.mocked(setRetentionDays).mockResolvedValue(0);
  render(<RetentionSetting />);
  const input = await screen.findByLabelText('Keep files of published items (days)');
  await waitFor(() => expect(input).toHaveValue(4));
  fireEvent.change(input, { target: { value: '0' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save days' }));
  await waitFor(() => expect(setRetentionDays).toHaveBeenCalledWith(0));
  expect(screen.getByLabelText('Keep files of published items (days)')).toHaveValue(0);
});
