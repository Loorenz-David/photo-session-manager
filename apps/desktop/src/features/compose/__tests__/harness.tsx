/**
 * Shared harness of the P31 behaviour tests (copied from `workspace/__tests__/p22_workspace_tidy.test.tsx`): the
 * service mocks, jsdom polyfills, photo/view fixtures, `openWorkspace`, the compose-stage layout, pointer / wheel /
 * gesture dispatch and the hand-flushed animation frame.
 */
import '@testing-library/jest-dom/vitest';
import { act, cleanup, createEvent, fireEvent, render, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, vi } from 'vitest';
import { toast } from 'sonner';
import type { ItemDto_Serialize } from '@/services/bindings';
import type { Composition, CompositionValues } from '@/services/compositions';
import type { FileDropEvent, PhotoEvent, WorkspaceEvent } from '@/services/events';
import type { Photo, SessionView } from '@/services/photos';

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
vi.mock('@/services/compositions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/compositions')>()),
  setComposition: vi.fn(),
  applyComposition: vi.fn(),
  resetComposition: vi.fn(),
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
import { setComposition } from '@/services/compositions';
import { classifyPhoto, copyPhotos, deletePhoto, getSessionView, importPhotos, movePhotos, reorderPhoto } from '@/services/photos';
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

export const SESSION = 5;
const ITEM: ItemDto_Serialize = { id: 50, sku: 'OAK-1', folder_key: 'OAK-1', barcode: null, item_type: 'Chair', title: 'Oak chair', origin: 'local', external_id: null, created_at: 0, updated_at: 0 };

/** The 2560 × 1707 preview frame and its default square (P28 case `d1`). */
export const FRAME = { w: 2560, h: 1707 };
export const DEFAULT_RECT = { x: 0.16660156250000002, y: 0, w: 0.666796875, h: 1 };
export const DEFAULT_VALUES: CompositionValues = { kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: DEFAULT_RECT };

export function composition(photoId: number, over: Partial<Composition> = {}): Composition {
  return { photo_id: photoId, kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: DEFAULT_RECT, frame: FRAME, origin: 'default', revision: 0, state: 'ok', review_reason: null, updated_at: 0, ...over };
}

/** What the backend answers to a save: the sent values stored under `revision`. */
export function saved(photoId: number, values: CompositionValues, revision = 1): Composition {
  return composition(photoId, { ...values, origin: 'user', revision, updated_at: 1 });
}

export function photo(id: number, classification: Photo['classification'], extra: Partial<Photo> = {}): Photo {
  return {
    id,
    session_id: SESSION,
    classification,
    position: null,
    captured_at: id * 10,
    original_preview_path: `/previews/${id}/original.jpg`,
    adjusted_preview_path: `/previews/${id}/adjusted-1.jpg`,
    thumbnail_path: `/previews/${id}/thumb-1.jpg`,
    composition: composition(id),
    orientation: 1,
    source: 'capture',
    integrity: 'ok',
    upload_state: null,
    ...extra,
  };
}

export function view(photos: Photo[], sessionId = SESSION): SessionView {
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
export async function openWorkspace(views: SessionView[], sessionId = SESSION) {
  const mocked = vi.mocked(getSessionView);
  views.forEach(v => mocked.mockResolvedValueOnce(v));
  mocked.mockResolvedValue(views[views.length - 1]);
  window.location.hash = `#/session/${sessionId}`;
  render(<App />);
  await waitFor(() => expect(document.querySelectorAll('.photo-panel [data-photo-id]').length).toBe(views[0].photos.length));
}

/** Two main photos, photo 1 on the stage. */
export const twoPhotos = () => view([photo(1, 'main', { position: 1 }), photo(2, 'main', { position: 2 })]);

export const stage = () => document.querySelector<HTMLElement>('.compose-stage');
export const image = () => document.querySelector<HTMLElement>('.compose-image');
export const tile = (id: number, scope: ParentNode = document) => scope.querySelector<HTMLElement>(`[data-photo-id="${id}"]`)!;

/** The compose stage measures 1000 × 700 at (0, 0) (plan P31 fixture); everything else has no box. */
export function layoutStage() {
  vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(function (this: Element) {
    if (this.matches('.compose-stage')) {
      return { left: 0, top: 0, width: 1000, height: 700, right: 1000, bottom: 700, x: 0, y: 0, toJSON: () => ({}) } as DOMRect;
    }
    return { left: 0, top: 0, width: 0, height: 0, right: 0, bottom: 0, x: 0, y: 0, toJSON: () => ({}) } as DOMRect;
  });
}

/** Enters Compose with R and waits for the editor. */
export async function compose() {
  fireEvent.keyDown(window, { key: 'r' });
  await waitFor(() => expect(stage()).not.toBeNull());
}

/** Dispatches a pointer event on the compose stage: `pointerId 1`, `isPrimary true`, `button 0` unless `extra` says otherwise. */
export function pointer(type: 'pointerdown' | 'pointermove' | 'pointerup' | 'pointercancel', x: number, y: number, extra: PointerEventInit = {}, target: Element = stage()!) {
  const init: PointerEventInit = { pointerId: 1, isPrimary: true, button: 0, clientX: x, clientY: y, bubbles: true, cancelable: true, ...extra };
  fireEvent(target, new PointerEvent(type, init));
}

/** Dispatches a wheel event on the compose stage and returns it (for `defaultPrevented`). */
export function wheel(init: { x?: number; y?: number; deltaX?: number; deltaY?: number; ctrlKey?: boolean; metaKey?: boolean }) {
  const target = stage()!;
  const event = createEvent.wheel(target, { clientX: init.x ?? 500, clientY: init.y ?? 350, deltaX: init.deltaX ?? 0, deltaY: init.deltaY ?? 0, ctrlKey: init.ctrlKey ?? false, metaKey: init.metaKey ?? false });
  fireEvent(target, event);
  return event;
}

/** Fake timers for the wheel settle — the clock and intervals only, so `captureFrames` keeps `requestAnimationFrame`. */
export function fakeTimers() {
  vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'setInterval', 'clearInterval', 'Date'] });
}

/** Dispatches a WebKit gesture event (`gesturestart` / `gesturechange` / `gestureend`) and returns it. */
export function gesture(type: 'gesturestart' | 'gesturechange' | 'gestureend', init: { x?: number; y?: number; scale?: number } = {}, target: Element = stage()!) {
  const event = new MouseEvent(type, { clientX: init.x ?? 500, clientY: init.y ?? 350, bubbles: true, cancelable: true });
  Object.defineProperty(event, 'scale', { value: init.scale ?? 1 });
  fireEvent(target, event);
  return event;
}

/** Captures `requestAnimationFrame` so the test flushes frames by hand and counts the calls. */
export function captureFrames() {
  const frames: FrameRequestCallback[] = [];
  const spy = vi.spyOn(window, 'requestAnimationFrame').mockImplementation(callback => {
    frames.push(callback);
    return frames.length;
  });
  vi.spyOn(window, 'cancelAnimationFrame').mockImplementation(() => undefined);
  return {
    calls: () => spy.mock.calls.length,
    flush: () => act(() => { frames.splice(0).forEach(callback => callback(0)); }),
  };
}

/** `translate(Xpx, Ypx) rotate(Ddeg)` of the compose image. */
export function transformOf(element: HTMLElement | null = image()): { x: number; y: number; deg: number } {
  const match = /translate\((-?[\d.]+)px, (-?[\d.]+)px\) rotate\((-?[\d.]+)deg\)/.exec(element?.style.transform ?? '');
  if (!match) throw new Error(`no transform: ${element?.style.transform ?? '(no element)'}`);
  return { x: Number(match[1]), y: Number(match[2]), deg: Number(match[3]) };
}

export const savedValues = (call: number): CompositionValues => vi.mocked(setComposition).mock.calls[call][1];

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
  vi.mocked(setComposition).mockImplementation(async (photoId, values) => saved(photoId, values as CompositionValues, 1));
  window.localStorage.setItem('compose.hintShown', '1');
  window.location.hash = '#/';
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.clearAllMocks();
  vi.mocked(getSessionView).mockReset();
  vi.mocked(setComposition).mockReset();
});

export { delivered };
