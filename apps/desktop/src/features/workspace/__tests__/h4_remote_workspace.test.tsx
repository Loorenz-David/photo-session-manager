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
import { setComposition } from '@/services/compositions';
import { classifyPhoto, copyPhotos, deletePhoto, getSessionView, movePhotos, reorderPhoto } from '@/services/photos';
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
// H4·C10 — the standard workspace with the Shopify pictures

it('H4·C10(a) remotePhotos: one photo object per picture with negative ids and a neutral composition', () => {
  const photos = remotePhotos(remoteView('archived', [picture('gallery', 1, '/a'), picture('gallery', 2, null), picture('condition', 1, '/c')]));
  expect(photos.map(p => [p.id, p.classification, p.position, p.original_preview_path, p.adjusted_preview_path])).toEqual([
    [-1, 'main', 1, '/a', '/a'],
    [-2, 'main', 2, null, null],
    [-3, 'condition', 1, '/c', '/c'],
  ]);
  for (const p of photos) {
    expect(p.composition.frame).not.toBeNull();
    expect(p.composition.rect).not.toBeNull();
  }
});

it('H4·C10(b) an archived product opens in the standard workspace with its pictures', async () => {
  await opened();
  expect(getRemote).toHaveBeenCalledWith(SESSION);
  const stage = screen.getByRole('region', { name: 'Stage' });
  expect(images(stage)).toContain('asset:///p/g1.jpg');
  expect(panelImages('Main images')).toEqual(['asset:///p/g1.jpg', 'asset:///p/g2.jpg', 'asset:///p/g3.jpg']);
  expect(panelImages('Condition')).toEqual(['asset:///p/c1.jpg', 'asset:///p/c2.jpg']);
  expect(screen.queryByRole('main', { name: 'Archived in Shopify' })).not.toBeInTheDocument();
});

it('H4·C10(c) a picture arriving in the background replaces its placeholder without asking again', async () => {
  await opened(remoteView('archived', [FIVE[0], picture('gallery', 2, null), FIVE[2], FIVE[3], FIVE[4]]));
  const second = () => document.querySelector('.photo-panel [data-photo-id="-2"]')!;
  expect(images(second())).toEqual([]);
  expect(getRemote).toHaveBeenCalledTimes(1);
  act(() => delivered.pictures.forEach(handler => handler({ kind: 'remote', session_id: SESSION, role: 'gallery', position: 2, path: '/b' })));
  await waitFor(() => expect(images(second())).toEqual(['asset:///b']));
  expect(getRemote).toHaveBeenCalledTimes(1);
});

it('H4·C10(d) the note under the header with Bring back now', async () => {
  await opened();
  const note = screen.getByRole('note');
  expect(note).toHaveTextContent('Photos are on Shopify. Editing a photo brings them back to this Mac first.');
  expect(within(note).getByRole('button', { name: 'Bring back now' })).toBeInTheDocument();
});

it('H4·C10(e) a shop-only product opens the same way', async () => {
  await opened(remoteView('shop_only', [picture('gallery', 1, '/s1.jpg'), picture('gallery', 2, '/s2.jpg')]));
  expect(panelImages('Main images')).toEqual(['asset:///s1.jpg', 'asset:///s2.jpg']);
  expect(screen.getByRole('note')).toHaveTextContent('Photos are on Shopify. Editing a photo brings them back to this Mac first.');
});

it('H4·C10(f) checking Shopify, then the failed check with Try again', async () => {
  open(remoteView('unknown', [], { check_failed: false }));
  expect(await screen.findByText('Checking Shopify…')).toBeInTheDocument();
  vi.mocked(getRemote).mockResolvedValue(remoteView('unknown', [], { check_failed: true }));
  act(() => delivered.workspace.forEach(handler => handler({ kind: 'session_changed', session_id: SESSION })));
  expect(await screen.findByText("Could not check Shopify for this product's photos.")).toBeInTheDocument();
  const calls = vi.mocked(getRemote).mock.calls.length;
  fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
  await waitFor(() => expect(getRemote).toHaveBeenCalledTimes(calls + 1));
});

it('H4·C10(g) the details form opens without bringing anything back', async () => {
  await opened();
  act(() => { fireEvent.keyDown(window, { key: 'd' }); });
  expect(onDetails).toHaveBeenCalledWith(true);
  expect(dialog()).toBeNull();
});

it('H4·C10(h) a local session is the workspace as before', async () => {
  vi.mocked(getSessionView).mockResolvedValue(sessionView([
    { id: 11, session_id: SESSION, classification: 'main', position: 1, captured_at: 0, original_preview_path: '/l/1.jpg', adjusted_preview_path: '/l/1-adj.jpg', thumbnail_path: null, composition: remotePhotos(remoteView('archived', [FIVE[0]]))[0].composition, orientation: null, source: 'capture', integrity: 'ok', upload_state: null },
  ] as unknown as SessionView['photos']));
  open(remoteView('local', []));
  await waitFor(() => expect(panelImages('Main images')).toEqual(['asset:///l/1-adj.jpg']));
  expect(screen.queryByRole('note')).toBeNull();
});

// ---------------------------------------------------------------------------------------------
// H4·C11 — every photo edit asks first

const QUESTION = 'Bring back 5 photos from Shopify to edit them? About 8 MB.';
async function asked() {
  const shown = await screen.findByRole('dialog', { name: 'Bring back the photos' });
  expect(within(shown).getByText(QUESTION)).toBeInTheDocument();
  return shown;
}
/** Closes the question with Cancel (clicked again while it is still shown: a click in the tick the dialog mounts
 *  after a window keydown can be lost in jsdom). */
async function closeDialog() {
  await asked();
  await waitFor(() => {
    const shown = dialog();
    if (shown) fireEvent.click(within(shown).getByRole('button', { name: 'Cancel' }));
    expect(dialog()).toBeNull();
  });
}
const openStageMenu = () => fireEvent.pointerDown(screen.getByRole('button', { name: 'Photo actions' }), { button: 0, ctrlKey: false });

it('H4·C11(a) the stage Condition button asks first and classifies nothing', async () => {
  await opened();
  fireEvent.click(screen.getByRole('button', { name: 'Move to Condition' }));
  const shown = await asked();
  expect(within(shown).getByRole('button', { name: 'Bring back' })).toBeInTheDocument();
  expect(within(shown).getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
  expect(classifyPhoto).not.toHaveBeenCalled();
});

it('H4·C11(b) the Condition key asks first', async () => {
  await opened();
  act(() => { fireEvent.keyDown(window, { key: 'c' }); });
  await asked();
  expect(classifyPhoto).not.toHaveBeenCalled();
});

it('H4·C11(c) Deny in the photo menu asks first', async () => {
  await opened();
  openStageMenu();
  fireEvent.click(await screen.findByRole('menuitem', { name: /^Deny/ }));
  await asked();
  expect(deletePhoto).not.toHaveBeenCalled();
});

/** Lays out the panel's tiles for dnd-kit, which measures with getBoundingClientRect (zero in jsdom). */
function layout(rects: Record<string, [number, number]>) {
  return vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(function (this: Element) {
    for (const [selector, [left, top]] of Object.entries(rects)) {
      if (this.matches(selector)) return { left, top, width: 100, height: 75, right: left + 100, bottom: top + 75, x: left, y: top, toJSON: () => ({}) } as DOMRect;
    }
    return { left: 0, top: 0, width: 0, height: 0, right: 0, bottom: 0, x: 0, y: 0, toJSON: () => ({}) } as DOMRect;
  });
}

it('H4·C11(d) dragging a picture to another place asks first and reorders nothing', async () => {
  await opened();
  const measured = layout({ '.photo-panel [data-photo-id="-1"]': [0, 0], '.photo-panel [data-photo-id="-2"]': [110, 0], '.photo-panel [data-photo-id="-3"]': [220, 0] });
  const from = document.querySelector('.photo-panel [data-photo-id="-3"]')!;
  fireEvent.pointerDown(from, { button: 0, isPrimary: true, clientX: 270, clientY: 37 });
  await act(async () => { fireEvent.pointerMove(document, { clientX: 260, clientY: 47 }); });
  await act(async () => { fireEvent.pointerMove(document, { clientX: 20, clientY: 37 }); });
  act(() => { fireEvent.pointerUp(document, { clientX: 20, clientY: 37 }); });
  await asked();
  expect(reorderPhoto).not.toHaveBeenCalled();
  measured.mockRestore();
});

it('H4·C11(e) R and the menu Split ask first and open no editor', async () => {
  await opened();
  act(() => { fireEvent.keyDown(window, { key: 'r' }); });
  await closeDialog();
  expect(screen.queryByRole('button', { name: 'Done' })).toBeNull();
  openStageMenu();
  fireEvent.click(await screen.findByRole('menuitem', { name: /^Split/ }));
  await asked();
  expect(screen.queryByRole('button', { name: 'Done' })).toBeNull();
  expect(setComposition).not.toHaveBeenCalled();
});

it('H4·C11(f) Move, Copy and Assign in the menu ask first and open no picker', async () => {
  await opened();
  for (const name of ['Move to session…', 'Copy to session…', 'Assign to product…']) {
    openStageMenu();
    fireEvent.click(await screen.findByRole('menuitem', { name }));
    await closeDialog();
    expect(screen.queryByRole('dialog', { name: /Move to session|Copy to session|Assign to product/ })).toBeNull();
  }
  expect(movePhotos).not.toHaveBeenCalled();
  expect(copyPhotos).not.toHaveBeenCalled();
});

it('H4·C11(g) Bring back now opens the same question', async () => {
  await opened();
  fireEvent.click(screen.getByRole('button', { name: 'Bring back now' }));
  await asked();
});

it('H4·C11(h) Cancel closes the question and brings nothing back', async () => {
  await opened();
  fireEvent.click(screen.getByRole('button', { name: 'Bring back now' }));
  await closeDialog();
  expect(bringBack).not.toHaveBeenCalled();
  expect(panelImages('Main images')).toEqual(['asset:///p/g1.jpg', 'asset:///p/g2.jpg', 'asset:///p/g3.jpg']);
});

it('H4·C11(i) one picture: the singular question, at least 1 MB', async () => {
  open(remoteView('archived', [FIVE[0]], { bring_back_bytes: 300_000 }));
  fireEvent.click(await screen.findByRole('button', { name: 'Bring back now' }));
  const shown = await screen.findByRole('dialog', { name: 'Bring back the photos' });
  expect(within(shown).getByText('Bring back 1 photo from Shopify to edit it? About 1 MB.')).toBeInTheDocument();
});
