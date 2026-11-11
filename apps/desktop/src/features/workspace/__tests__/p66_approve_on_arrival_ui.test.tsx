import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ItemDto_Serialize, SessionRowResponse_Serialize } from '@/services/bindings';
import type { Photo, SessionView } from '@/services/photos';

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
  listSessions: vi.fn(),
  activeSession: vi.fn(),
  createSession: vi.fn(),
  activateSession: vi.fn(),
  deactivateSession: vi.fn(),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/items', () => ({ searchItems: vi.fn(), searchItemsShop: vi.fn(async () => []), nextSku: vi.fn(), createItem: vi.fn(), adoptItem: vi.fn(), listItemTypes: vi.fn(async () => { throw { code: 'no_integration' }; }), refreshItem: vi.fn() }));
vi.mock('@/services/camera', () => ({ cameraStatus: vi.fn(), cameraRetry: vi.fn() }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async () => () => undefined),
  onCameraStatus: vi.fn(async () => () => undefined),
  onWorkspaceEvents: vi.fn(async () => () => undefined),
  onPhotoAdded: vi.fn(async () => () => undefined),
  onFileDrop: vi.fn(async () => () => undefined),
}));
vi.mock('@/services/history', () => ({
  historyGet: vi.fn(async () => ({ undo: null, redo: null })),
  historyUndo: vi.fn(),
  historyRedo: vi.fn(),
  editMenuSet: vi.fn(async () => undefined),
  onHistoryChanged: vi.fn(async () => () => undefined),
  onEditMenu: vi.fn(async () => () => undefined),
}));

import App from '@/App';
import { SessionsPage } from '@/features/sessions/SessionsPage';
import { cameraStatus } from '@/services/camera';
import { classifyPhoto, deletePhoto, getSessionView } from '@/services/photos';
import { activateSession, activeSession, deactivateSession, listSessions } from '@/services/sessions';

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
const MINUTE = 60_000;
const DAY = 24 * 60 * MINUTE;

function photo(id: number, classification: Photo['classification'], position: number | null): Photo {
  return {
    id,
    session_id: SESSION,
    classification,
    position,
    captured_at: id * 10,
    original_preview_path: `/previews/${id}/original.jpg`,
    adjusted_preview_path: `/previews/${id}/adjusted-1.jpg`,
    thumbnail_path: null,
    composition: { photo_id: id, kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: null, frame: null, frame_source: null, origin: 'default', revision: 0, state: 'ok', review_reason: null, updated_at: 0 },
    orientation: 1,
    source: 'capture',
    integrity: 'ok',
    upload_state: null,
  };
}

function view(photos: Photo[]): SessionView {
  return {
    session: {
      id: SESSION,
      item_id: ITEM.id,
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
    item: ITEM,
    photos,
  };
}

const reviewed = { sessions: [], active_photo_id: null };

/** Opens the workspace; `getSessionView` answers with each view in turn, then keeps the last. */
async function openWorkspace(views: SessionView[]) {
  const mocked = vi.mocked(getSessionView);
  views.forEach(v => mocked.mockResolvedValueOnce(v));
  mocked.mockResolvedValue(views[views.length - 1]);
  window.location.hash = `#/session/${SESSION}`;
  render(<App />);
  await waitFor(() => expect(document.querySelectorAll('.photo-panel [data-photo-id]').length).toBe(views[0].photos.length));
}

function activeTile(): number | null {
  const tile = document.querySelector('.photo-panel [aria-current="true"]');
  return tile ? Number(tile.getAttribute('data-photo-id')) : null;
}

function press(key: string) {
  fireEvent.keyDown(window, { key });
}

function reviewButtons(): string[] {
  return within(document.querySelector('.review-buttons') as HTMLElement).getAllByRole('button').map(button => button.getAttribute('aria-label') ?? '');
}

/** Drags the stage photo by (dx, dy) with the pointer and releases. */
async function swipe(dx: number, dy: number) {
  const layer = document.querySelector('.swipe-layer')!;
  fireEvent.pointerDown(layer, { button: 0, isPrimary: true, clientX: 300, clientY: 300 });
  for (const step of [0.25, 0.5, 0.75, 1]) {
    await act(async () => {
      fireEvent.pointerMove(document, { clientX: 300 + dx * step, clientY: 300 + dy * step, buttons: 1 });
      await new Promise(resolve => setTimeout(resolve, 20));
    });
  }
  await act(async () => {
    fireEvent.pointerUp(document, { clientX: 300 + dx, clientY: 300 + dy });
    await new Promise(resolve => setTimeout(resolve, 20));
  });
}

function sessionRow(id: number, title: string, createdAt: number, lastActivityAt: number): SessionRowResponse_Serialize {
  return {
    id,
    sku: `SKU-${id}`,
    item_type: '',
    title,
    main_count: 1,
    condition_count: 0,
    unreviewed_count: 0,
    status: 'in_progress',
    label: { kind: 'in_progress', value: 0 },
    progress: null,
    last_activity_at: lastActivityAt,
    created_at: createdAt,
    cover_preview_path: null,
    publish: null,
  };
}

beforeEach(() => {
  vi.mocked(listSessions).mockResolvedValue({ groups: [], counts: { open: 0, published: 0, all: 0 }, unassigned_count: 0 });
  vi.mocked(activeSession).mockResolvedValue(null);
  vi.mocked(activateSession).mockResolvedValue();
  vi.mocked(deactivateSession).mockResolvedValue();
  vi.mocked(cameraStatus).mockResolvedValue({ kind: 'connected', name: 'Sony a7R V', link: 'cable' });
  vi.mocked(classifyPhoto).mockResolvedValue(reviewed);
  vi.mocked(deletePhoto).mockResolvedValue(reviewed);
  window.location.hash = '#/';
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.clearAllMocks();
  vi.mocked(getSessionView).mockReset();
});

describe('C3(b) review of photos that arrive approved', () => {
  const arrived = [photo(1, 'main', 1), photo(2, 'main', 2), photo(3, 'main', 3)];

  it('an arrived photo offers Deny and Condition, no Approve; C marks it Condition and the next photo in order opens', async () => {
    await openWorkspace([view(arrived), view([photo(2, 'main', 1), photo(3, 'main', 2), photo(1, 'condition', 1)])]);
    expect(activeTile()).toBe(1);
    expect(reviewButtons()).toEqual(['Deny', 'Move to Condition']);

    press('c');
    await waitFor(() => expect(classifyPhoto).toHaveBeenCalledTimes(1));
    expect(classifyPhoto).toHaveBeenCalledWith(1, 'condition');
    await waitFor(() => expect(activeTile()).toBe(2));
  });

  it('⌫ denies it: "Photo denied · Undo ⌘Z", and the next photo in order opens', async () => {
    await openWorkspace([view(arrived), view([photo(1, 'main', 1), photo(3, 'main', 2)])]);
    fireEvent.click(document.querySelector('.photo-panel [data-photo-id="2"]')!);
    await waitFor(() => expect(activeTile()).toBe(2));

    press('Backspace');
    await waitFor(() => expect(deletePhoto).toHaveBeenCalledTimes(1));
    expect(deletePhoto).toHaveBeenCalledWith(2);
    expect(await screen.findByText('Photo denied · Undo ⌘Z')).toBeInTheDocument();
    await waitFor(() => expect(activeTile()).toBe(3));
  });

  it('swiping the stage photo left denies it', async () => {
    await openWorkspace([view(arrived)]);
    await swipe(-200, 0);
    await waitFor(() => expect(deletePhoto).toHaveBeenCalledWith(1));
    expect(classifyPhoto).not.toHaveBeenCalled();
  });

  it('swiping the stage photo up marks it Condition', async () => {
    await openWorkspace([view(arrived)]);
    await swipe(0, -200);
    await waitFor(() => expect(classifyPhoto).toHaveBeenCalledWith(1, 'condition'));
    expect(deletePhoto).not.toHaveBeenCalled();
  });

  it('A on an arrived photo changes nothing (no approve step); a photo unreviewed from before keeps Approve', async () => {
    await openWorkspace([view([photo(1, 'main', 1), photo(9, 'unreviewed', null)])]);
    fireEvent.click(document.querySelector('.photo-panel [data-photo-id="1"]')!);
    await waitFor(() => expect(activeTile()).toBe(1));
    press('a');
    await waitFor(() => expect(activeTile()).toBe(9));
    expect(classifyPhoto).not.toHaveBeenCalled();

    expect(reviewButtons()).toEqual(['Deny', 'Move to Condition', 'Approve']);
    press('a');
    await waitFor(() => expect(classifyPhoto).toHaveBeenCalledWith(9, 'main'));
  });
});

describe('C3(a) the sessions list keeps creation order', () => {
  it('rows render in the delivered creation order, show their creation time, and the time filter uses it', async () => {
    const now = Date.now();
    vi.mocked(listSessions).mockResolvedValue({
      groups: [
        // C and B were created today; B was last active 40 days ago (impossible but telling), A just now.
        { label: 'Today', rows: [sessionRow(3, 'Gamma lamp', now - 2 * MINUTE, now - 2 * MINUTE), sessionRow(2, 'Beta desk', now - 3 * MINUTE, now - 40 * DAY)] },
        { label: 'Earlier', rows: [sessionRow(1, 'Alpha chair', now - 40 * DAY, now - 1 * MINUTE)] },
      ],
      counts: { open: 3, published: 0, all: 3 },
      unassigned_count: 0,
    });
    render(<SessionsPage navigate={() => undefined} />);
    await screen.findByText('Alpha chair');
    const rows = () => [...document.querySelectorAll('[data-testid^="session-row-"]')].map(row => row.getAttribute('data-testid'));
    expect(rows()).toEqual(['session-row-3', 'session-row-2', 'session-row-1']);
    const time = (id: number) => document.querySelector(`[data-testid="session-row-${id}"] time`)!;
    expect(time(1)).toHaveAttribute('datetime', new Date(now - 40 * DAY).toISOString());
    expect(time(3)).toHaveTextContent('2 min ago');

    fireEvent.click(screen.getByRole('button', { name: 'Filter by creation time' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Today' }));
    await waitFor(() => expect(rows()).toEqual(['session-row-3', 'session-row-2']));
  });
});
