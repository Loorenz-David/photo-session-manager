import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { BringBack, PictureReady, WorkspaceEvent } from '@/services/events';
import type { SessionView } from '@/services/photos';
import type { RemoteView } from '@/services/remote';

/** Handlers registered with the (mocked) event services, so a test can deliver events. */
const delivered = vi.hoisted(() => ({
  workspace: [] as ((event: WorkspaceEvent) => void)[],
  pictures: [] as ((event: PictureReady) => void)[],
  bringBack: [] as ((event: BringBack) => void)[],
}));

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
vi.mock('@/services/remote', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/remote')>()),
  getRemote: vi.fn(),
  refreshRemote: vi.fn(),
  bringBack: vi.fn(),
}));
vi.mock('@/services/compositions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/compositions')>()),
  setComposition: vi.fn(),
  applyComposition: vi.fn(),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(async () => ({ groups: [], counts: { open: 0, published: 0, all: 0 }, unassigned_count: 0 })),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/events', () => ({
  onWorkspaceEvents: vi.fn(async (handler: (event: WorkspaceEvent) => void) => { delivered.workspace.push(handler); return () => undefined; }),
  onPictureReady: vi.fn(async (handler: (event: PictureReady) => void) => { delivered.pictures.push(handler); return () => undefined; }),
  onBringBack: vi.fn(async (handler: (event: BringBack) => void) => { delivered.bringBack.push(handler); return () => undefined; }),
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
vi.mock('@/services/items', () => ({ searchItems: vi.fn(async () => []), searchItemsShop: vi.fn(async () => []), adoptItem: vi.fn() }));
vi.mock('@/features/form/FormDialog', () => ({ FormDialog: ({ open }: { open: boolean }) => (open ? <div role="dialog" aria-label="Product details" /> : null) }));
vi.mock('sonner', () => ({ toast: Object.assign(vi.fn(), { success: vi.fn(), error: vi.fn(), dismiss: vi.fn() }) }));

import { remotePhotos } from '@/features/workspace/remote/remotePhotos';
import { WorkspacePage } from '@/features/workspace/WorkspacePage';
import { getSessionView } from '@/services/photos';
import { bringBack, getRemote } from '@/services/remote';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};
Element.prototype.scrollIntoView ??= function scrollIntoView() {};
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;
if (!('PointerEvent' in window)) {
  class PointerEventPolyfill extends MouseEvent {
    readonly pointerId: number;
    readonly isPrimary: boolean;
    readonly pointerType: string;
    constructor(type: string, init: PointerEventInit = {}) {
      super(type, init);
      this.pointerId = init.pointerId ?? 1;
      this.isPrimary = init.isPrimary ?? true;
      this.pointerType = init.pointerType ?? 'mouse';
    }
  }
  Object.assign(window, { PointerEvent: PointerEventPolyfill });
}

const SESSION = 5;
const item = { id: 7, sku: 'ARC-1', folder_key: 'ARC-1', barcode: null, item_type: 'Sideboard', title: 'Oak sideboard', origin: 'local', external_id: 'gid://shopify/Product/1', created_at: 0, updated_at: 0 };
function sessionView(photos: SessionView['photos'] = []): SessionView {
  return {
    session: { id: SESSION, item_id: 7, published_at: 1, published_seq: 3, change_seq: 3, dirty: false, last_activity_at: 0, created_at: 0, photo_count: 0, unreviewed_count: 0, main_count: 0, condition_count: 0, publish: null },
    item,
    photos,
  } as unknown as SessionView;
}
const picture = (role: 'gallery' | 'condition', position: number, path: string | null) => ({ role, position, media_id: `${role}-${position}`, path, gone: false });
function remoteView(state: RemoteView['state'], pictures: RemoteView['pictures'], extra: Partial<RemoteView> = {}): RemoteView {
  return { item_id: 7, state, keep_files: false, pictures, bring_back_bytes: 7_600_000, check_failed: false, bring_back: null, ...extra };
}
const FIVE = [
  picture('gallery', 1, '/p/g1.jpg'), picture('gallery', 2, '/p/g2.jpg'), picture('gallery', 3, '/p/g3.jpg'),
  picture('condition', 1, '/p/c1.jpg'), picture('condition', 2, '/p/c2.jpg'),
];

beforeEach(() => {
  delivered.workspace.length = 0;
  delivered.pictures.length = 0;
  delivered.bringBack.length = 0;
  vi.mocked(getSessionView).mockResolvedValue(sessionView());
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

const onDetails = vi.fn();
function open(view: RemoteView) {
  vi.mocked(getRemote).mockResolvedValue(view);
  render(<WorkspacePage sessionId={SESSION} assignOpen={false} onAssignOpenChange={() => undefined} onDetailsOpenChange={onDetails} navigate={() => undefined} />);
}
/** The `<img>` sources in a panel group (the tiles' pictures carry `alt=""`). */
const images = (scope: Element) => [...scope.querySelectorAll('img')].map(img => img.getAttribute('src'));
const panelImages = (region: string) => images(screen.getByRole('region', { name: region }));
const dialog = () => screen.queryByRole('dialog', { name: 'Bring back the photos' });
async function opened(view = remoteView('archived', FIVE)) {
  open(view);
  await screen.findByText('Photos are on Shopify. Editing a photo brings them back to this Mac first.');
}

// ---------------------------------------------------------------------------------------------
// H4·C12 — Bring back: progress, success, failure, one at a time

const progress = (done: number, total: number) =>
  act(() => delivered.bringBack.forEach(handler => handler({ session_id: SESSION, done, total, state: 'running', error: null })));
async function askAndBringBack() {
  await opened();
  fireEvent.click(screen.getByRole('button', { name: 'Bring back now' }));
  const shown = await screen.findByRole('dialog', { name: 'Bring back the photos' });
  fireEvent.click(within(shown).getByRole('button', { name: 'Bring back' }));
  return shown;
}

it('H4·C12(a) Bring back starts once and shows its progress', async () => {
  vi.mocked(bringBack).mockReturnValue(new Promise(() => undefined));
  const shown = await askAndBringBack();
  progress(4, 10);
  await waitFor(() => expect(within(shown).getByText('Bringing back 4 of 10 photos…')).toBeInTheDocument());
  expect(within(shown).queryByRole('button', { name: 'Bring back' })).toBeNull();
  expect(bringBack).toHaveBeenCalledTimes(1);
  expect(bringBack).toHaveBeenCalledWith(SESSION);
});

it('H4·C12(b) after a bring back the product is an ordinary session', async () => {
  const own = remotePhotos(remoteView('archived', FIVE.map(p => ({ ...p, path: `/own/${p.role}-${p.position}.jpg` }))))
    .map((photo, index) => ({ ...photo, id: 100 + index, session_id: SESSION, upload_state: null }));
  vi.mocked(bringBack).mockImplementation(async () => {
    vi.mocked(getRemote).mockResolvedValue(remoteView('local', []));
    vi.mocked(getSessionView).mockResolvedValue(sessionView(own));
    return { photo_ids: own.map(p => p.id), synced: true };
  });
  await askAndBringBack();
  await waitFor(() => expect(dialog()).toBeNull());
  await waitFor(() => expect(screen.queryByRole('note')).toBeNull());
  await waitFor(() => expect(panelImages('Main images')).toEqual(['asset:///own/gallery-1.jpg', 'asset:///own/gallery-2.jpg', 'asset:///own/gallery-3.jpg']));
  expect(panelImages('Condition')).toEqual(['asset:///own/condition-1.jpg', 'asset:///own/condition-2.jpg']);
  expect(document.querySelectorAll('.photo-panel [data-photo-id]')).toHaveLength(5);
});

it('H4·C12(c) a failed bring back says why and offers Try again', async () => {
  vi.mocked(bringBack).mockRejectedValue(new Error('Could not bring back the photos: 1 photo could not be downloaded'));
  const shown = await askAndBringBack();
  expect(await within(shown).findByText('Could not bring back the photos: 1 photo could not be downloaded')).toBeInTheDocument();
  fireEvent.click(within(shown).getByRole('button', { name: 'Try again' }));
  await waitFor(() => expect(bringBack).toHaveBeenCalledTimes(2));
});

it('H4·C12(d) opened while a bring back runs: its progress, nothing started', async () => {
  await opened(remoteView('archived', FIVE, { bring_back: { done: 2, total: 5 } }));
  fireEvent.click(screen.getByRole('button', { name: 'Move to Condition' }));
  const shown = await screen.findByRole('dialog', { name: 'Bring back the photos' });
  expect(within(shown).getByText('Bringing back 2 of 5 photos…')).toBeInTheDocument();
  expect(bringBack).not.toHaveBeenCalled();
});
