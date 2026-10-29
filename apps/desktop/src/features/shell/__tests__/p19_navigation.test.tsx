import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ItemDto_Serialize } from '@/services/bindings';
import type { FileDropEvent, PhotoEvent, WorkspaceEvent } from '@/services/events';
import type { Photo, SessionView } from '@/services/photos';

/** Handlers the app registered with the (mocked) Tauri event services, so a test can deliver events. */
const delivered = vi.hoisted(() => ({
  workspace: [] as ((event: WorkspaceEvent) => void)[],
  photoAdded: [] as ((event: PhotoEvent) => void)[],
  fileDrop: [] as ((event: FileDropEvent) => void)[],
}));

vi.mock('@/services/health', () => ({ getHealth: vi.fn(), getAppPaths: vi.fn() }));
vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), listCropRatios: vi.fn(async () => []) }));
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
import { HealthPage } from '@/features/health/HealthPage';
import { SettingsPage } from '@/features/settings/SettingsPage';
import { getHealth, getAppPaths } from '@/services/health';
import { getSettings, type Settings } from '@/services/settings';
import { cameraStatus } from '@/services/camera';
import { searchItems } from '@/services/items';
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

const shown: Settings = {
  crop_on_arrival: 'Square',
  document_id: null, preview_recipe: 'PSM Preview', web_recipe: 'PSM Web', style_name: 'PSM House',
  export_folder: '/tmp/export', preview_delay_s: 15, effective_preview_delay_s: 15, theme: 'light', sku_prefix: null,
  integration: { kind: 'shopify', shop_domain: 'shop.myshopify.com', api_version: '2026-10', client_id: 'client', client_secret: 'masked', condition_photos_key: 'custom.damage_report' },
};

beforeEach(() => {
  delivered.workspace.length = 0;
  delivered.photoAdded.length = 0;
  delivered.fileDrop.length = 0;
  vi.mocked(getHealth).mockResolvedValue({ items: [] });
  vi.mocked(getAppPaths).mockResolvedValue({ data_dir: '/tmp/psm', database: '/tmp/psm/db', settings: '/tmp/psm/settings' });
  vi.mocked(getSettings).mockResolvedValue(shown);
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



it('C1(a) Health starts with Back to Settings and keeps Retry checks as its only header action', async () => {
  window.location.hash = '#/health';
  render(<HealthPage />);
  await screen.findByRole('region', { name: 'System status' });
  const header = screen.getByRole('banner');
  expect(within(header).getAllByRole('button')[0]).toHaveAccessibleName('Back to Settings');
  expect(within(header).getByRole('button', { name: 'Retry checks' })).toBeInTheDocument();
  expect(within(header).queryByRole('button', { name: 'Sessions' })).toBeNull();
  expect(within(header).queryByRole('button', { name: 'Settings' })).toBeNull();
  fireEvent.click(within(header).getAllByRole('button')[0]);
  expect(window.location.hash).toBe('#/settings');
});

it('C1(b) Settings starts with Back to Sessions and Health is the child action', async () => {
  window.location.hash = '#/settings';
  render(<SettingsPage />);
  await screen.findByLabelText('Client secret');
  const buttons = within(screen.getByRole('banner')).getAllByRole('button');
  expect(buttons[0]).toHaveAccessibleName('Back to Sessions');
  expect(buttons.filter(button => button.getAttribute('aria-label')?.includes('Sessions') || button.textContent?.includes('Sessions'))).toHaveLength(1);
  expect(within(screen.getByRole('banner')).getByRole('button', { name: 'Health' })).toBeInTheDocument();
  fireEvent.click(buttons[0]);
  expect(window.location.hash).toBe('#/');
});

it('C1(c) both item and Unassigned workspaces return to the Sessions page', async () => {
  for (const [id, photos] of [[5, [photo(1, 'main', { position: 1 }), photo(2, 'main', { position: 2 })]], [1, []]] as const) {
    await openWorkspace([view([...photos], id)], id);
    const header = screen.getByRole('banner');
    expect(within(header).getAllByRole('button')[0]).toHaveAccessibleName('Back to Sessions');
    if (id === 1) expect(within(header).getByText('Unassigned photos')).toBeInTheDocument();
    fireEvent.click(within(header).getAllByRole('button')[0]);
    await waitFor(() => expect(screen.getByRole('button', { name: /New session/i })).toBeInTheDocument());
    expect(window.location.hash).toBe('#/');
    expect(document.querySelector('.photo-panel')).toBeNull();
    cleanup();
    vi.mocked(getSessionView).mockReset();
  }
});

it('C1(d) the Sessions root has a title first and no Back button', async () => {
  render(<App />);
  const header = screen.getByRole('banner');
  expect(header.firstElementChild).toHaveTextContent('Session Manager');
  expect(header.firstElementChild?.tagName).toBe('H1');
  expect(within(header).queryByRole('button', { name: /^Back/ })).toBeNull();
  expect(screen.getByRole('button', { name: /New session/i })).toBeInTheDocument();
});

it('C2(a) Settings and Health Back keys respect the page parent and typing', async () => {
  window.location.hash = '#/settings';
  render(<SettingsPage />);
  await screen.findByLabelText('Client secret');
  fireEvent.keyDown(window, { key: '[', metaKey: true });
  expect(window.location.hash).toBe('#/');
  cleanup();
  window.location.hash = '#/settings';
  render(<SettingsPage />);
  await screen.findByLabelText('Client secret');
  fireEvent.keyDown(window, { key: 'Escape' });
  expect(window.location.hash).toBe('#/');
  cleanup();
  window.location.hash = '#/settings';
  render(<SettingsPage />);
  const secret = await screen.findByLabelText('Client secret');
  fireEvent.keyDown(secret, { key: 'Escape' });
  expect(window.location.hash).toBe('#/settings');
  cleanup();
  window.location.hash = '#/health';
  render(<HealthPage />);
  fireEvent.keyDown(window, { key: '[', metaKey: true });
  expect(window.location.hash).toBe('#/settings');
  cleanup();
  window.location.hash = '#/health';
  render(<HealthPage />);
  fireEvent.keyDown(window, { key: 'Escape' });
  expect(window.location.hash).toBe('#/settings');
});

it('C2(b) workspace Escape keeps its layer behavior and command Back waits for the picker to close', async () => {
  vi.mocked(searchItems).mockResolvedValue([]);
  await openWorkspace([view([], 1)], 1);
  fireEvent.keyDown(window, { key: 'Escape' });
  expect(window.location.hash).toBe('#/session/1');
  fireEvent.click(screen.getByRole('button', { name: 'Assign product' }));
  expect(await screen.findByRole('dialog')).toBeInTheDocument();
  fireEvent.keyDown(window, { key: '[', metaKey: true });
  expect(window.location.hash).toBe('#/session/1');
  fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  fireEvent.keyDown(window, { key: '[', metaKey: true });
  expect(window.location.hash).toBe('#/');
});
