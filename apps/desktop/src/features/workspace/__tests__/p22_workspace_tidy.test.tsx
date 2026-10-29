import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { toast } from 'sonner';
import type { ItemDto_Serialize } from '@/services/bindings';
import type { FileDropEvent, PhotoEvent, WorkspaceEvent } from '@/services/events';
import type { Photo, SessionView } from '@/services/photos';

/** Handlers the app registered with the (mocked) Tauri event services, so a test can deliver events. */
const delivered = vi.hoisted(() => ({
  workspace: [] as ((event: WorkspaceEvent) => void)[],
  photoAdded: [] as ((event: PhotoEvent) => void)[],
  fileDrop: [] as ((event: FileDropEvent) => void)[],
}));

vi.mock('@/services/photos', () => ({
  getSessionView: vi.fn(),
  classifyPhoto: vi.fn(),
  reorderPhoto: vi.fn(),
  deletePhoto: vi.fn(),
  undoDelete: vi.fn(),
  movePhotos: vi.fn(),
  copyPhotos: vi.fn(),
  importPhotos: vi.fn(),
  assignUnassigned: vi.fn(),
  pickImportFiles: vi.fn(),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
  activeSession: vi.fn(),
  createSession: vi.fn(),
  activateSession: vi.fn(),
  deactivateSession: vi.fn(),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/items', () => ({ searchItems: vi.fn(), searchItemsShop: vi.fn(async () => []), nextSku: vi.fn(), createItem: vi.fn(), adoptItem: vi.fn() }));
vi.mock('@/services/camera', () => ({ cameraStatus: vi.fn(), cameraRetry: vi.fn() }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async () => () => undefined),
  onCameraStatus: vi.fn(async () => () => undefined),
  onWorkspaceEvents: vi.fn(async (handler: (event: WorkspaceEvent) => void) => {
    delivered.workspace.push(handler);
    return () => undefined;
  }),
  onPhotoAdded: vi.fn(async (handler: (event: PhotoEvent) => void) => {
    delivered.photoAdded.push(handler);
    return () => undefined;
  }),
  onFileDrop: vi.fn(async (handler: (event: FileDropEvent) => void) => {
    delivered.fileDrop.push(handler);
    return () => undefined;
  }),
}));

import App from '@/App';
import { cameraStatus } from '@/services/camera';
import {
  classifyPhoto,
  copyPhotos,
  deletePhoto,
  getSessionView,
  importPhotos,
  movePhotos,
  reorderPhoto,
} from '@/services/photos';
import { activateSession, activeSession, deactivateSession, listSessions } from '@/services/sessions';

// jsdom lacks these browser APIs, which cmdk, Radix and dnd-kit use.
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
const ITEM: ItemDto_Serialize = { id: 50, sku: 'OAK-1', folder_key: 'OAK-1', barcode: null, item_type: 'Chair', title: 'Oak chair', origin: 'local', external_id: null, created_at: 0, updated_at: 0 };

function photo(id: number, classification: Photo['classification'], extra: Partial<Photo> = {}): Photo {
  return {
    id,
    session_id: SESSION,
    classification,
    position: null,
    captured_at: id * 10,
    original_preview_path: `/previews/${id}/original.jpg`,
    adjusted_preview_path: `/previews/${id}/adjusted-1.jpg`,
    orientation: 1,
    source: 'capture',
    integrity: 'ok',
    upload_state: null,
    ...extra,
  };
}

function view(photos: Photo[], sessionId = SESSION): SessionView {
  return {
    session: {
      id: sessionId,
      item_id: sessionId === 1 ? null : ITEM.id,
      published_at: null,
      published_seq: 0,
      change_seq: 0,
      dirty: false,
      last_activity_at: 0,
      created_at: 0,
      photo_count: photos.length,
      unreviewed_count: photos.filter(p => p.classification === 'unreviewed').length,
      main_count: photos.filter(p => p.classification === 'main').length,
      condition_count: photos.filter(p => p.classification === 'condition').length,
      publish: null,
    },
    item: sessionId === 1 ? null : ITEM,
    photos: photos.map(p => ({ ...p, session_id: sessionId })),
  };
}

const reviewed = { sessions: [], active_photo_id: null };

/** Opens the workspace of `sessionId`; `getSessionView` answers with each view in turn, then keeps the last. */
async function openWorkspace(views: SessionView[], sessionId = SESSION) {
  const mocked = vi.mocked(getSessionView);
  views.forEach(v => mocked.mockResolvedValueOnce(v));
  mocked.mockResolvedValue(views[views.length - 1]);
  window.location.hash = `#/session/${sessionId}`;
  render(<App />);
  await waitFor(() => expect(document.querySelectorAll('.photo-panel [data-photo-id]').length).toBe(views[0].photos.length));
}

function panelOrder(scope: ParentNode = document.querySelector('.photo-panel')!): number[] {
  return [...scope.querySelectorAll('[data-photo-id]')].map(tile => Number(tile.getAttribute('data-photo-id')));
}

/** Lays out elements for dnd-kit, which measures with getBoundingClientRect (zero everywhere in jsdom). */
function layout(rects: Record<string, [number, number]>, size: [number, number] = [100, 75]) {
  vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(function (this: Element) {
    for (const [selector, [left, top]] of Object.entries(rects)) {
      if (this.matches(selector)) {
        const [width, height] = size;
        return { left, top, width, height, right: left + width, bottom: top + height, x: left, y: top, toJSON: () => ({}) } as DOMRect;
      }
    }
    return { left: 0, top: 0, width: 0, height: 0, right: 0, bottom: 0, x: 0, y: 0, toJSON: () => ({}) } as DOMRect;
  });
}

const tile = (id: number) => document.querySelector(`.photo-panel [data-photo-id="${id}"]`)!;

beforeEach(() => {
  vi.spyOn(toast, 'error');
  delivered.workspace.length = 0;
  delivered.photoAdded.length = 0;
  delivered.fileDrop.length = 0;
  vi.mocked(listSessions).mockResolvedValue({ groups: [], counts: { open: 0, published: 0, all: 0 }, unassigned_count: 0 });
  vi.mocked(activeSession).mockResolvedValue(null);
  vi.mocked(activateSession).mockResolvedValue();
  vi.mocked(deactivateSession).mockResolvedValue();
  vi.mocked(cameraStatus).mockResolvedValue({ kind: 'connected', name: 'Sony ZV-E10' });
  vi.mocked(classifyPhoto).mockResolvedValue(reviewed);
  vi.mocked(reorderPhoto).mockResolvedValue(reviewed);
  vi.mocked(deletePhoto).mockResolvedValue(reviewed);
  vi.mocked(movePhotos).mockResolvedValue(reviewed);
  vi.mocked(copyPhotos).mockResolvedValue(reviewed);
  vi.mocked(importPhotos).mockResolvedValue();
  window.location.hash = '#/';
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.clearAllMocks();
  vi.mocked(getSessionView).mockReset();
});


async function releaseAt(from: Element, to: [number, number]) {
  const start = from.getBoundingClientRect();
  const x = start.left + start.width / 2;
  const y = start.top + start.height / 2;
  fireEvent.pointerDown(from, { button: 0, isPrimary: true, clientX: x, clientY: y });
  await act(async () => { fireEvent.pointerMove(document, { clientX: x + 10, clientY: y + 10 }); });
  await act(async () => { fireEvent.pointerMove(document, { clientX: to[0], clientY: to[1] }); });
  act(() => { fireEvent.pointerUp(document, { clientX: to[0], clientY: to[1] }); });
}

const mainOrder = () => panelOrder(screen.getByRole('region', { name: 'Main images' }));
const conditionOrder = () => panelOrder(screen.getByRole('region', { name: 'Condition' }));
const original = view([photo(1, 'main', { position: 1 }), photo(2, 'main', { position: 2 }), photo(5, 'main', { position: 3 })]);
const stored = view([photo(1, 'main', { position: 3 }), photo(2, 'main', { position: 1 }), photo(5, 'main', { position: 2 })]);

it('C3(a) an event during a pending reorder leaves the dropped order visible until the command reloads', async () => {
  let resolve!: (value: typeof reviewed) => void;
  vi.mocked(reorderPhoto).mockReturnValue(new Promise(done => { resolve = done; }));
  vi.mocked(getSessionView).mockResolvedValue(original);
  await openWorkspace([original]);
  layout({ '[data-photo-id="1"]': [0, 0], '[data-photo-id="2"]': [110, 0], '[data-photo-id="5"]': [0, 85] });
  await releaseAt(tile(1), [50, 122]);
  expect(mainOrder()).toEqual([2, 5, 1]);
  const beforeEvent = vi.mocked(getSessionView).mock.calls.length;
  await act(async () => delivered.workspace.forEach(handler => handler({ kind: 'photo_changed', photo_id: 2, session_id: SESSION })));
  expect(vi.mocked(getSessionView).mock.calls.length).toBe(beforeEvent);
  expect(mainOrder()).toEqual([2, 5, 1]);
  vi.mocked(getSessionView).mockResolvedValue(stored);
  await act(async () => { resolve(reviewed); });
  expect(mainOrder()).toEqual([2, 5, 1]);
  expect(vi.mocked(getSessionView).mock.calls.length).toBeGreaterThan(beforeEvent);
});

it('C3(b) the held overlay shows the destination number and Cover badge before release', async () => {
  const withCondition = view([...original.photos, photo(4, 'condition', { position: 1 }), photo(6, 'unreviewed')]);
  await openWorkspace([withCondition]);
  layout({ '[data-photo-id="1"]': [0, 0], '[data-photo-id="2"]': [110, 0], '[data-photo-id="5"]': [0, 85], '[data-photo-id="6"]': [110, 85], '[data-photo-id="4"]': [0, 300] });
  const start = tile(1);
  fireEvent.pointerDown(start, { button: 0, isPrimary: true, clientX: 50, clientY: 37 });
  await act(async () => { fireEvent.pointerMove(document, { clientX: 60, clientY: 47 }); });
  await act(async () => { fireEvent.pointerMove(document, { clientX: 50, clientY: 122 }); });
  const overlay = () => document.querySelector('.drag-overlay')!;
  expect(overlay().querySelector('.tile-number')).toHaveTextContent('03');
  expect(overlay().querySelector('.tile-cover')).toBeNull();
  await act(async () => { fireEvent.pointerMove(document, { clientX: 50, clientY: 337 }); });
  expect(overlay().querySelector('.tile-number')).toHaveTextContent('02');
  expect(overlay().querySelector('.tile-cover')).toBeNull();
  await act(async () => { fireEvent.pointerMove(document, { clientX: 50, clientY: 37 }); });
  expect(overlay().querySelector('.tile-number')).toHaveTextContent('01');
  expect(overlay().querySelector('.tile-cover')).not.toBeNull();
  act(() => { fireEvent.pointerUp(document, { clientX: 50, clientY: 37 }); });
  expect(reorderPhoto).not.toHaveBeenCalled();
  expect(classifyPhoto).not.toHaveBeenCalled();
  expect(mainOrder()).toEqual([1, 2, 5, 6]);
  expect(conditionOrder()).toEqual([4]);
  const next = tile(5);
  fireEvent.pointerDown(next, { button: 0, isPrimary: true, clientX: 50, clientY: 122 });
  await act(async () => { fireEvent.pointerMove(document, { clientX: 60, clientY: 132 }); });
  await act(async () => { fireEvent.pointerMove(document, { clientX: 50, clientY: 37 }); });
  expect(overlay().querySelector('.tile-number')).toHaveTextContent('01');
  expect(overlay().querySelector('.tile-cover')).not.toBeNull();
  act(() => { fireEvent.pointerUp(document, { clientX: 50, clientY: 37 }); });
  const afterOwnSlot = tile(1);
  fireEvent.pointerDown(afterOwnSlot, { button: 0, isPrimary: true, clientX: 50, clientY: 37 });
  await act(async () => { fireEvent.pointerMove(document, { clientX: 60, clientY: 47 }); });
  await act(async () => { fireEvent.pointerMove(document, { clientX: 160, clientY: 122 }); });
  expect(overlay().querySelector('.tile-number')).toHaveTextContent('03');
  expect(overlay().querySelector('.tile-cover')).toBeNull();
  act(() => { fireEvent.pointerUp(document, { clientX: 160, clientY: 122 }); });
  await waitFor(() => expect(reorderPhoto).toHaveBeenCalledWith(1, 3));
});
