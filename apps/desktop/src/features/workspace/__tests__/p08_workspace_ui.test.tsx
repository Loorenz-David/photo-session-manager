import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ItemDto_Serialize, SessionRowResponse_Serialize, SessionStatusResponse } from '@/services/bindings';
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
vi.mock('@/services/items', () => ({ searchItems: vi.fn(), nextSku: vi.fn(), createItem: vi.fn() }));
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
import { displayOrder } from '@/features/workspace/order';
import { cameraStatus } from '@/services/camera';
import { searchItems } from '@/services/items';
import {
  assignUnassigned,
  classifyPhoto,
  copyPhotos,
  deletePhoto,
  getSessionView,
  importPhotos,
  movePhotos,
  reorderPhoto,
  undoDelete,
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
const ITEM: ItemDto_Serialize = { id: 50, sku: 'OAK-1', folder_key: 'OAK-1', barcode: null, item_type: 'Chair', title: 'Oak chair', origin: 'local', created_at: 0, updated_at: 0 };

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
    },
    item: sessionId === 1 ? null : ITEM,
    photos: photos.map(p => ({ ...p, session_id: sessionId })),
  };
}

function sessionRow(id: number, title: string, status: SessionStatusResponse = 'in_progress'): SessionRowResponse_Serialize {
  return {
    id,
    sku: `SKU-${id}`,
    item_type: '',
    title,
    main_count: 1,
    condition_count: 0,
    unreviewed_count: 0,
    status,
    label: status === 'published' ? { kind: 'published' } : { kind: 'in_progress', value: 0 },
    progress: null,
    last_activity_at: 0,
    cover_preview_path: null,
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

/** The highlighted (active) tile of the photo panel. */
function activeTile(): number | null {
  const tile = document.querySelector('.photo-panel [aria-current="true"]');
  return tile ? Number(tile.getAttribute('data-photo-id')) : null;
}

function panelOrder(scope: ParentNode = document.querySelector('.photo-panel')!): number[] {
  return [...scope.querySelectorAll('[data-photo-id]')].map(tile => Number(tile.getAttribute('data-photo-id')));
}

function press(key: string, init: KeyboardEventInit = {}) {
  fireEvent.keyDown(window, { key, ...init });
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

/** Drags with the pointer from the centre of `from` to the centre of `to` and releases. */
async function drag(from: Element, to: [number, number]) {
  const start = from.getBoundingClientRect();
  const x = start.left + start.width / 2;
  const y = start.top + start.height / 2;
  fireEvent.pointerDown(from, { button: 0, isPrimary: true, clientX: x, clientY: y });
  await act(async () => {
    fireEvent.pointerMove(document, { clientX: x + 10, clientY: y + 10 });
  });
  await act(async () => {
    fireEvent.pointerMove(document, { clientX: to[0], clientY: to[1] });
  });
  await act(async () => {
    fireEvent.pointerUp(document, { clientX: to[0], clientY: to[1] });
  });
  // dnd-kit swallows the click that follows a drag for a short while; let that pass before the next interaction.
  await act(() => new Promise(resolve => setTimeout(resolve, 80)));
}

const tile = (id: number) => document.querySelector(`.photo-panel [data-photo-id="${id}"]`)!;

beforeEach(() => {
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

describe('C1 review keys and the next unreviewed photo', () => {
  it('C1(a) A on the active unreviewed photo classifies it into Main', async () => {
    await openWorkspace([view([photo(1, 'unreviewed')])]);
    press('a');
    await waitFor(() => expect(classifyPhoto).toHaveBeenCalledTimes(1));
    expect(classifyPhoto).toHaveBeenCalledWith(1, 'main');
  });

  it('C1(b) C classifies it into Condition', async () => {
    await openWorkspace([view([photo(1, 'unreviewed')])]);
    press('c');
    await waitFor(() => expect(classifyPhoto).toHaveBeenCalledTimes(1));
    expect(classifyPhoto).toHaveBeenCalledWith(1, 'condition');
  });

  it('C1(c) ⌫ deletes it', async () => {
    await openWorkspace([view([photo(1, 'unreviewed')])]);
    press('Backspace');
    await waitFor(() => expect(deletePhoto).toHaveBeenCalledTimes(1));
    expect(deletePhoto).toHaveBeenCalledWith(1);
  });

  it('C1(d) after each action the earliest-captured remaining unreviewed photo opens', async () => {
    const p = photo(1, 'unreviewed', { captured_at: 10 });
    const u1 = photo(9, 'unreviewed', { captured_at: 200 });
    const u2 = photo(12, 'unreviewed', { captured_at: 100 });
    await openWorkspace([
      view([p, u1, u2]),
      view([{ ...p, classification: 'main', position: 1 }, u1, u2]),
      view([{ ...p, classification: 'main', position: 1 }, u1, { ...u2, classification: 'main', position: 2 }]),
    ]);
    expect(activeTile()).toBe(1);

    press('a');
    await waitFor(() => expect(activeTile()).toBe(12));

    press('a');
    await waitFor(() => expect(classifyPhoto).toHaveBeenLastCalledWith(12, 'main'));
    await waitFor(() => expect(activeTile()).toBe(9));
  });
});

describe('Candidate C9 a capture arriving in the open session (ch04 S-C)', () => {
  it('C9 photo_added puts the new photo on the stage, and the next key acts on it', async () => {
    const first = photo(1, 'main', { position: 1 });
    const arrived = photo(2, 'unreviewed');
    await openWorkspace([view([first]), view([first, arrived])]);
    expect(activeTile()).toBe(1);

    await act(async () => delivered.workspace.forEach(handler => handler({ kind: 'photo_added', photo_id: 2, session_id: SESSION })));
    await waitFor(() => expect(activeTile()).toBe(2));

    press('c');
    await waitFor(() => expect(classifyPhoto).toHaveBeenCalledTimes(1));
    expect(classifyPhoto).toHaveBeenCalledWith(2, 'condition');
  });
});

describe('C2 undo', () => {
  const p = photo(1, 'main', { position: 1 });
  const q = photo(2, 'main', { position: 2 });

  it('C2(a) delete shows "Photo deleted · Undo ⌘Z", which closes at 4 500 ms', async () => {
    await openWorkspace([view([p, q]), view([q])]);
    vi.useFakeTimers();
    press('Backspace');
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    const toast = screen.getByText('Photo deleted · Undo ⌘Z').closest('[data-sonner-toast]')!;
    expect(toast).toHaveAttribute('data-removed', 'false');

    await act(async () => {
      await vi.advanceTimersByTimeAsync(4499);
    });
    expect(toast).toHaveAttribute('data-removed', 'false');

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });
    expect(toast).toHaveAttribute('data-removed', 'true');
    // Sonner's exit animation, then the toast is out of the document.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(200);
    });
    expect(screen.queryByText('Photo deleted · Undo ⌘Z')).toBeNull();
  });

  it('C2(b) ⌘Z at 6 000 ms, after the toast has gone, undoes and makes the restored photo active', async () => {
    vi.mocked(undoDelete).mockResolvedValue({ sessions: [], active_photo_id: 1 });
    await openWorkspace([view([p, q]), view([q]), view([p, q])]);
    vi.useFakeTimers();
    press('Backspace');
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(6000);
    });
    expect(screen.queryByText('Photo deleted · Undo ⌘Z')).toBeNull();
    expect(activeTile()).toBe(2);

    press('z', { metaKey: true });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(undoDelete).toHaveBeenCalledTimes(1);
    expect(undoDelete).toHaveBeenCalledWith(SESSION);
    expect(activeTile()).toBe(1);
  });

  it('C2(c) the toast Undo button undoes', async () => {
    vi.mocked(undoDelete).mockResolvedValue({ sessions: [], active_photo_id: 1 });
    await openWorkspace([view([p, q]), view([q]), view([p, q])]);
    press('Backspace');
    fireEvent.click(await screen.findByRole('button', { name: 'Undo' }));
    await waitFor(() => expect(undoDelete).toHaveBeenCalledTimes(1));
    expect(undoDelete).toHaveBeenCalledWith(SESSION);
  });
});

describe('C3 stage: Original / Adjusted', () => {
  const original = () => screen.getByAltText('Camera original');

  it('C3(a) with no adjusted render the original is shown and the pill spins', async () => {
    await openWorkspace([view([photo(1, 'unreviewed', { adjusted_preview_path: null })])]);
    expect(original()).toHaveAttribute('data-visible', 'true');
    expect(screen.queryByAltText('Adjusted')).toBeNull();
    expect(within(document.querySelector('.toggle-pill')!).getByRole('status', { name: 'Processing' })).toBeInTheDocument();
  });

  it('C3(b) photo_changed delivering an adjusted path renders it with the 700 ms crossfade', async () => {
    const pending = photo(1, 'unreviewed', { adjusted_preview_path: null });
    await openWorkspace([view([pending]), view([{ ...pending, adjusted_preview_path: '/previews/1/adjusted-2.jpg' }])]);
    expect(screen.queryByAltText('Adjusted')).toBeNull();

    await act(async () => delivered.workspace.forEach(handler => handler({ kind: 'photo_changed', photo_id: 1, session_id: SESSION })));

    const adjusted = await screen.findByAltText('Adjusted');
    expect(adjusted).toHaveAttribute('src', 'asset:///previews/1/adjusted-2.jpg');
    expect(adjusted).toHaveClass('crossfade');
    expect(adjusted).toHaveStyle({ transitionProperty: 'opacity', transitionDuration: '700ms' });
  });

  it('C3(c) Space switches to Original for the whole session view, so the next photo shows its original too', async () => {
    await openWorkspace([view([photo(1, 'main', { position: 1 }), photo(2, 'main', { position: 2 })])]);
    fireEvent.load(screen.getByAltText('Adjusted'));
    expect(original()).toHaveAttribute('data-visible', 'false');

    press(' ');
    expect(original()).toHaveAttribute('data-visible', 'true');
    press('ArrowRight');
    await waitFor(() => expect(activeTile()).toBe(2));
    fireEvent.load(screen.getByAltText('Adjusted'));

    expect(screen.getByAltText('Camera original')).toHaveAttribute('src', 'asset:///previews/2/original.jpg');
    expect(screen.getByAltText('Camera original')).toHaveAttribute('data-visible', 'true');
    expect(screen.getByAltText('Adjusted')).toHaveAttribute('data-visible', 'false');
    expect(document.querySelector('.toggle-pill')).toHaveAttribute('aria-pressed', 'true');
  });

  it('C3(d) orientation 6 rotates the original by 90° and leaves the adjusted unrotated', async () => {
    await openWorkspace([view([photo(1, 'main', { position: 1, orientation: 6 })])]);
    expect(original().style.transform).toBe('rotate(90deg)');
    expect(screen.getByAltText('Adjusted').style.transform).toBe('');
  });
});

describe('C4 order, reorder and groups', () => {
  const a = photo(1, 'main', { position: 1 });
  const b = photo(2, 'main', { position: 2 });
  const u = photo(3, 'unreviewed');
  const d = photo(4, 'condition', { position: 1 });

  it('C4(a) the grid shows Main by position, then the unreviewed photo, then the Condition section', async () => {
    await openWorkspace([view([b, u, d, a])]);
    expect(panelOrder()).toEqual([1, 2, 3, 4]);
    const condition = screen.getByRole('region', { name: 'Condition' });
    expect(panelOrder(condition)).toEqual([4]);
    expect(panelOrder(screen.getByRole('region', { name: 'Main images' }))).toEqual([1, 2, 3]);
  });

  it('C4(b) dragging a from index 0 to index 2 in Main reorders it to position 3', async () => {
    const c = photo(5, 'main', { position: 3 });
    await openWorkspace([view([a, b, c])]);
    layout({ '[data-photo-id="1"]': [0, 0], '[data-photo-id="2"]': [110, 0], '[data-photo-id="5"]': [0, 85] });
    await drag(tile(1), [50, 122]);
    await waitFor(() => expect(reorderPhoto).toHaveBeenCalledWith(1, 3));
    expect(classifyPhoto).not.toHaveBeenCalled();
  });

  it('C4(c) dragging d from Condition onto Main index 0 classifies it into Main (no reorder)', async () => {
    await openWorkspace([view([a, b, d])]);
    layout({ '[data-photo-id="1"]': [0, 0], '[data-photo-id="2"]': [110, 0], '[data-photo-id="4"]': [0, 300] });
    await drag(tile(4), [50, 37]);
    await waitFor(() => expect(classifyPhoto).toHaveBeenCalledWith(4, 'main'));
    expect(reorderPhoto).not.toHaveBeenCalled();
  });

  it('C4(d) the folded strip shows the same order with the COND divider before d', async () => {
    await openWorkspace([view([b, u, d, a])]);
    press('f');
    const strip = await screen.findByRole('complementary', { name: 'Photo strip' });
    const sequence = [...strip.querySelectorAll('[data-photo-id], [role="separator"]')].map(node =>
      node.getAttribute('role') === 'separator' ? node.textContent : Number(node.getAttribute('data-photo-id')),
    );
    expect(sequence).toEqual([1, 2, 3, 'COND', 4]);
  });
});

describe('C5 Unassigned workspace', () => {
  const p = photo(1, 'unreviewed');
  const q = photo(2, 'unreviewed');

  async function openUnassigned(rows: SessionRowResponse_Serialize[]) {
    vi.mocked(listSessions).mockResolvedValue({ groups: [{ label: 'Today', rows }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 2 });
    await openWorkspace([view([p, q], 1)], 1);
    await waitFor(() => expect(document.querySelectorAll('.sessions-panel-row').length).toBe(rows.length));
  }

  it('C5(a) the sessions panel is open, the header reads "No product" and offers "Assign product"', async () => {
    await openUnassigned([sessionRow(7, 'Oak chair')]);
    expect(screen.getByRole('complementary', { name: 'Sessions' })).toBeInTheDocument();
    expect(screen.getByText('No product')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Assign product' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /Publish/ })).toBeNull();
  });

  it('C5(b) dropping the selected photos p and q on session S moves both there', async () => {
    await openUnassigned([sessionRow(7, 'Oak chair')]);
    fireEvent.click(tile(1), { metaKey: true });
    fireEvent.click(tile(2), { metaKey: true });
    layout({ '[data-photo-id="1"]': [0, 0], '[data-photo-id="2"]': [110, 0], '[data-session-id="7"]': [400, 0] }, [100, 40]);
    await drag(tile(1), [450, 20]);
    await waitFor(() => expect(movePhotos).toHaveBeenCalledWith([1, 2], 7));
  });

  it('C5(c) a published session row is not a drop target', async () => {
    await openUnassigned([sessionRow(8, 'Walnut desk', 'published')]);
    const row = document.querySelector('[data-session-id="8"]')!;
    expect(row).toHaveAttribute('data-droppable', 'false');
    layout({ '[data-photo-id="1"]': [0, 0], '[data-session-id="8"]': [400, 0] }, [100, 40]);
    await drag(tile(1), [450, 20]);
    await act(async () => undefined);
    expect(movePhotos).not.toHaveBeenCalled();
  });

  it('C5(d) "Assign product" then picking an item assigns the Unassigned set to it', async () => {
    vi.mocked(searchItems).mockResolvedValue([ITEM]);
    vi.mocked(assignUnassigned).mockResolvedValue({ sessions: [], active_photo_id: null });
    await openUnassigned([]);
    fireEvent.click(screen.getByRole('button', { name: 'Assign product' }));
    fireEvent.change(await screen.findByLabelText('Search items by title or SKU'), { target: { value: 'Oak' } });
    fireEvent.click(await screen.findByText('Oak chair'));
    await waitFor(() => expect(assignUnassigned).toHaveBeenCalledWith(ITEM.id));
  });
});

describe('C6 import', () => {
  it('C6(a) a file drop of /x.ARW imports that path', async () => {
    await openWorkspace([view([photo(1, 'main', { position: 1 })])]);
    await act(async () => delivered.fileDrop.forEach(handler => handler({ type: 'drop', paths: ['/x.ARW'] })));
    expect(importPhotos).toHaveBeenCalledWith(['/x.ARW']);
  });
});

describe('C7 arrivals elsewhere', () => {
  it('C7(a) a photo arriving in Unassigned while on the Sessions page raises a toast whose Open goes there', async () => {
    vi.mocked(getSessionView).mockResolvedValue(view([photo(1, 'unreviewed')], 1));
    render(<App />);
    await waitFor(() => expect(delivered.photoAdded.length).toBeGreaterThan(0));

    await act(async () => delivered.photoAdded.forEach(handler => handler({ photo_id: 1, session_id: 1 })));

    expect(await screen.findByText('Photos arrived in Unassigned')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Open' }));
    await waitFor(() => expect(window.location.hash).toBe('#/session/1'));
  });
});

describe('C8 photo menu', () => {
  it('C8(a) ⋯ "Copy to session…" then S copies the photo to S', async () => {
    vi.mocked(listSessions).mockResolvedValue({ groups: [{ label: 'Today', rows: [sessionRow(7, 'Oak chair')] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 0 });
    await openWorkspace([view([photo(1, 'main', { position: 1 })])]);
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Photo actions' }), { button: 0, ctrlKey: false });
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Copy to session…' }));
    const dialog = await screen.findByRole('dialog', { name: 'Copy to session' });
    fireEvent.click(await within(dialog).findByText('Oak chair'));
    await waitFor(() => expect(copyPhotos).toHaveBeenCalledWith([1], 7));
  });
});

describe('Core mechanism: displayOrder (MC-5 display rule)', () => {
  it('orders Main and Condition by position and unreviewed by captured_at, then id', () => {
    const order = displayOrder([
      photo(1, 'condition', { position: 2 }),
      photo(2, 'main', { position: 2 }),
      photo(3, 'unreviewed', { captured_at: 500 }),
      photo(4, 'unreviewed', { captured_at: 100 }),
      photo(5, 'unreviewed', { captured_at: 100 }),
      photo(6, 'main', { position: 1 }),
      photo(7, 'condition', { position: 1 }),
      photo(8, 'deleted'),
    ]);
    expect(order.main.map(p => p.id)).toEqual([6, 2]);
    expect(order.unreviewed.map(p => p.id)).toEqual([4, 5, 3]);
    expect(order.condition.map(p => p.id)).toEqual([7, 1]);
  });
});
