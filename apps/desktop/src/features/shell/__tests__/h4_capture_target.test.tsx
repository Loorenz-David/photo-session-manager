import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionRowResponse_Serialize } from '@/services/bindings';
import type { ConversationListRow, ConversationView as View } from '@/services/conversations';
import type { BringBack, PictureReady, WorkspaceEvent } from '@/services/events';
import type { PreviewDto } from '@/services/publish';
import type { RemoteView } from '@/services/remote';
import type { SessionList } from '@/services/sessions';
import type { Settings } from '@/services/settings';

/**
 * H4·U3 (MC-H3.4, 10H §6): the capture-target gate, the visit bar's Shoot here instead, Refresh from Shopify and the
 * details-only publish confirmation, through `App` with the services mocked. Session 3 is a local session; 7 is the
 * product under test (its remote state is each test's `getRemote` answer).
 */

const delivered = vi.hoisted(() => ({
  workspace: [] as ((event: WorkspaceEvent) => void)[],
  pictures: [] as ((event: PictureReady) => void)[],
  bringBack: [] as ((event: BringBack) => void)[],
}));

vi.mock('@/services/conversations', () => ({
  conversationsCounts: vi.fn(async () => ({ here: 0, elsewhere: 0 })),
  conversationsList: vi.fn(),
  conversationsSettingsGet: vi.fn(async () => ({ sound: true })),
  conversationsSettingsSave: vi.fn(),
  conversationsSessionBadges: vi.fn(async () => []),
  conversationGet: vi.fn(),
  // A30: one question per product unless a test says otherwise: the thread wraps conversationGet.
  conversationThread: vi.fn(async (id: string) => {
    const view = await (await import("@/services/conversations")).conversationGet(id);
    return { product_id: "p", sku: view.sku, product_title: view.product_title, session_id: view.session_id, questions: [view] };
  }),
  conversationMarkRead: vi.fn(async () => undefined),
  conversationAttachable: vi.fn(async () => ({ photos: [], reason: 'nothing_published' })),
  conversationReply: vi.fn(),
  conversationRetry: vi.fn(),
  conversationDiscard: vi.fn(),
  conversationMarkAnswered: vi.fn(),
}));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async () => () => undefined),
  onCameraStatus: vi.fn(async () => () => undefined),
  onWorkspaceEvents: vi.fn(async (handler: (event: WorkspaceEvent) => void) => { delivered.workspace.push(handler); return () => undefined; }),
  onPictureReady: vi.fn(async (handler: (event: PictureReady) => void) => { delivered.pictures.push(handler); return () => undefined; }),
  onBringBack: vi.fn(async (handler: (event: BringBack) => void) => { delivered.bringBack.push(handler); return () => undefined; }),
  onPhotoAdded: vi.fn(async () => () => undefined),
  onFileDrop: vi.fn(async () => () => undefined),
  onResearchChanged: vi.fn(async () => () => undefined),
  onConversationsChanged: vi.fn(async () => () => undefined),
}));
vi.mock('@/services/remote', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/remote')>()),
  getRemote: vi.fn(),
  refreshRemote: vi.fn(async () => undefined),
  bringBack: vi.fn(),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
  sessionsPage: vi.fn(),
  arrangeNext: vi.fn(),
  activeSession: vi.fn(),
  activateSession: vi.fn(async () => undefined),
  deactivateSession: vi.fn(async () => undefined),
  sessionFilesMisplaced: vi.fn(async () => 0),
}));
vi.mock('@/services/history', () => ({
  historyGet: vi.fn(() => new Promise(() => undefined)),
  historyUndo: vi.fn(),
  historyRedo: vi.fn(),
  editMenuSet: vi.fn(async () => undefined),
  onEditMenu: vi.fn(async () => () => undefined),
  onHistoryChanged: vi.fn(async () => () => undefined),
}));
vi.mock('@/services/search', () => ({ onOpenAiReview: vi.fn(async () => () => undefined) }));
vi.mock('@/services/writer', () => ({ writerRequest: vi.fn(async () => undefined) }));
vi.mock('@/services/research', () => ({ researchOverview: vi.fn(async () => []) }));
vi.mock('@/services/publish', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/publish')>()),
  publishRetry: vi.fn(),
  publishPreview: vi.fn(),
  publishConfirm: vi.fn(async () => 1),
}));
vi.mock('@/services/health', () => ({ getHealth: vi.fn(() => new Promise(() => undefined)), getAppPaths: vi.fn(() => new Promise(() => undefined)) }));
vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), pickSessionFile: vi.fn() }));
vi.mock('@/services/styles', () => ({ listStyles: vi.fn(async () => null) }));
vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => 'shopify'), onExportProgress: vi.fn(async () => () => undefined), openExportFolder: vi.fn(), runExport: vi.fn() }));
// A11's publish notifications: one "Open" for session 5 in the (real) notifications drawer.
vi.mock('@/features/publish/usePublish', () => ({
  usePublish: () => ({
    failedCount: 0,
    label: null,
    notifications: [{ id: 1, item_id: 50, session_id: 5, kind: 'published', text: 'Published BEYO-5', created_at: 1_795_176_000_000 }],
    refresh: async () => undefined,
  }),
}));
vi.mock('@/features/publish/PublishProgress', () => ({ PublishProgress: () => null }));
vi.mock('@/features/settings/StorageSection', () => ({ DiskWarning: () => null, StorageSection: () => null }));
vi.mock('@/features/settings/CameraSection', () => ({ CameraSection: () => null }));
vi.mock('@/features/settings/PricingSection', () => ({ PricingSection: () => null }));
vi.mock('@/features/settings/TemplatesSection', () => ({ TemplatesSection: () => null }));
vi.mock('@/features/history/HistoryButtons', () => ({ HistoryButtons: () => null }));
vi.mock('@/features/shell/CameraDot', () => ({ CameraDot: () => null }));
vi.mock('@/features/shell/ThemeToggle', () => ({ ThemeToggle: () => null, useTheme: () => ['light', () => undefined] }));
vi.mock('@/features/sessions/HoverPreview', () => ({ HoverPreview: ({ children }: { children: React.ReactNode }) => children, Thumb: () => null }));
// The workspace's own content is not under test here; it shows which session is on screen and starts a bring back
// through the shared remote item (as the workspace's dialog does).
vi.mock('@/features/workspace/WorkspacePage', async () => {
  const { useContext } = await import('react');
  const { RemoteItemContext } = await import('@/features/workspace/remote/useRemoteItem');
  return {
    WorkspacePage: ({ sessionId }: { sessionId: number }) => {
      const remote = useContext(RemoteItemContext);
      return <main data-testid="workspace">Workspace of session {sessionId}<button type="button" onClick={() => void remote?.bringBack()}>Start bring back</button></main>;
    },
  };
});
vi.mock('sonner', () => ({ toast: Object.assign(vi.fn(), { error: vi.fn(), success: vi.fn(), loading: vi.fn(), dismiss: vi.fn() }) }));
vi.mock('@/components/ui/sonner', () => ({ Toaster: () => null }));

import App from '@/App';
import { conversationGet, conversationsList } from '@/services/conversations';
import { publishConfirm, publishPreview } from '@/services/publish';
import { bringBack, getRemote, refreshRemote } from '@/services/remote';
import { activateSession, activeSession, deactivateSession, listSessions, sessionsPage } from '@/services/sessions';
import { answerFromList } from '@/features/sessions/__tests__/pageFromList';
import { getSettings } from '@/services/settings';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

function sessionRow(id: number, sku: string, title: string): SessionRowResponse_Serialize {
  return {
    id, sku, item_type: '', title, main_count: 2, condition_count: 1, unreviewed_count: 0, status: 'in_progress',
    label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: Date.now(), created_at: Date.now(), cover_preview_path: null, cover_pending: false,
    arranged: false, publish: null, changes: null,
  };
}
const LIST: SessionList = {
  groups: [{ label: 'Today', rows: [sessionRow(9, 'BEYO-9', 'Lamp'), sessionRow(7, 'BEYO-7', 'Mirror'), sessionRow(5, 'BEYO-5', 'Desk'), sessionRow(3, 'BEYO-3', 'Chair')] }],
  counts: { open: 4, published: 0, all: 4 },
  unassigned_count: 0,
};

function conversationRow(id: string, sessionId: number): ConversationListRow {
  return {
    product_id: `p-${sessionId}`, questions: 1,
    id, sku: `BEYO-${sessionId}`, product_title: `Product ${sessionId}`, topic: `Topic ${sessionId}`, status: 'open', blocking: false, waiting: false,
    last_line: 'Is the glass original?', last_activity_at: 1_795_176_000_000, has_news: true, session_id: sessionId,
  };
}
function conversation(id: string, sessionId: number): View {
  return {
    id, sku: `BEYO-${sessionId}`, product_title: `Product ${sessionId}`, topic: `Topic ${sessionId}`, blocking: false, status: 'open', resolved_by: null,
    session_id: sessionId, messages: [], pending: [], can_send: true, off_reason: null,
  };
}

const SETTINGS: Settings = {
  default_shape: '1:1', master_recipe: 'PSM Master',
  camera: { receiver_port: 2121, passive_port_start: 50000, passive_port_end: 50009 }, document_id: '/Users/me/Pictures/Beyo Studio', effective_document_id: '/Users/me/Pictures/Beyo Studio', default_document_id: '/Users/me/Pictures/Session Manager', preview_recipe: 'PSM Preview', output_format: { long_edge: 2048, quality: 90 }, style_name: '',
  export_folder: '/Users/me/Pictures/Session Manager Export', preview_delay_s: 15, effective_preview_delay_s: 15, theme: 'light', sku_prefix: null, integration: null,
} as Settings;

beforeEach(() => {
  answerFromList(sessionsPage, listSessions);
  vi.mocked(listSessions).mockResolvedValue(LIST);
  vi.mocked(activeSession).mockResolvedValue(null);
  vi.mocked(conversationsList).mockResolvedValue({ state: 'ready', problem: null, rows: [conversationRow('c7', 7), conversationRow('c9', 9)], news_elsewhere: 0 });
  vi.mocked(conversationGet).mockImplementation(async id => conversation(id, id === 'c9' ? 9 : 7));
  vi.mocked(getSettings).mockResolvedValue(SETTINGS);
});

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
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;
Element.prototype.scrollIntoView ??= function scrollIntoView() {};

/** Each session's remote view, as `getRemote` answers it (unlisted sessions are local). */
let remote: Record<number, RemoteView | null> = {};
function remoteView(state: RemoteView['state'], extra: Partial<RemoteView> = {}): RemoteView {
  const pictures = state === 'archived' || state === 'shop_only'
    ? [{ role: 'gallery', position: 1, media_id: 'm1', path: '/p/m1.jpg', gone: false }]
    : [];
  return { item_id: 70, state, keep_files: false, pictures, bring_back_bytes: 1_500_000, check_failed: false, bring_back: null, ...extra };
}
const ARCHIVED = remoteView('archived');
const LOCAL = remoteView('local');

beforeEach(() => {
  delivered.workspace.length = 0;
  delivered.pictures.length = 0;
  delivered.bringBack.length = 0;
  remote = { 7: ARCHIVED };
  vi.mocked(getRemote).mockImplementation(async id => remote[id] ?? LOCAL);
  answerFromList(sessionsPage, listSessions);
  vi.mocked(listSessions).mockResolvedValue(LIST);
  vi.mocked(activeSession).mockResolvedValue(null);
  vi.mocked(conversationsList).mockResolvedValue({ state: 'ready', problem: null, rows: [conversationRow('c7', 7), conversationRow('c9', 9)], news_elsewhere: 0 });
  vi.mocked(conversationGet).mockImplementation(async id => conversation(id, id === 'c9' ? 9 : 7));
  vi.mocked(getSettings).mockResolvedValue(SETTINGS);
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  window.location.hash = '';
});

/** Every `activateSession` / `deactivateSession` call, in order. */
function captureCalls(): string[] {
  const calls = [
    ...vi.mocked(activateSession).mock.calls.map((args, i) => ({ at: vi.mocked(activateSession).mock.invocationCallOrder[i], text: `activate ${args[0]}` })),
    ...vi.mocked(deactivateSession).mock.calls.map((_, i) => ({ at: vi.mocked(deactivateSession).mock.invocationCallOrder[i], text: 'deactivate' })),
  ];
  return calls.sort((a, b) => a.at - b.at).map(call => call.text);
}
function resetCaptureCalls() {
  vi.mocked(activateSession).mockClear();
  vi.mocked(deactivateSession).mockClear();
}

/** Opens the panel from the header icon and picks the conversation of `sessionId`. */
async function openConversationOf(sessionId: number) {
  fireEvent.click(await screen.findByRole('button', { name: 'Conversations' }));
  const panel = await screen.findByRole('dialog', { name: 'Conversations' });
  fireEvent.click(await within(panel).findByText(`Topic ${sessionId}`));
  await within(panel).findByTestId('conversation-view');
  await within(panel).findByText(`Product ${sessionId}`, { selector: '.conversation-title' });
}

async function openSessionButton() {
  return within(screen.getByRole('dialog', { name: 'Conversations' })).findByRole('button', { name: 'Open session' });
}

async function workspaceShows(sessionId: number) {
  await waitFor(() => expect(screen.getByTestId('workspace')).toHaveTextContent(`Workspace of session ${sessionId}`));
}

/** `App` in session 3 (its activation already made), then **Open session** on a conversation of `visited`. */
async function visitFromSession3(visited = 7) {
  window.location.hash = '#/session/3';
  render(<App />);
  await waitFor(() => expect(activateSession).toHaveBeenCalledWith(3));
  resetCaptureCalls();
  await openConversationOf(visited);
  fireEvent.click(await openSessionButton());
  await workspaceShows(visited);
}


/** Lets pending hash events, effects and remote reads run, so a call that would follow is made before the assertion. */
async function settle() {
  await act(() => new Promise(resolve => setTimeout(resolve, 30)));
}

/** An ordinary navigation (the sessions panel, a link): the hash changes. */
async function go(hash: string) {
  await act(async () => {
    window.location.hash = hash;
    window.dispatchEvent(new HashChangeEvent('hashchange'));
  });
}

function sessionChanged(sessionId: number) {
  act(() => delivered.workspace.forEach(handler => handler({ kind: 'session_changed', session_id: sessionId })));
}

/** Session 3 open with its activation made, then archived session 7 opened by an ordinary navigation. */
async function inSession3ThenOpen7() {
  window.location.hash = '#/session/3';
  render(<App />);
  await waitFor(() => expect(activateSession).toHaveBeenCalledWith(3));
  resetCaptureCalls();
  await go('#/session/7');
  await workspaceShows(7);
  await waitFor(() => expect(getRemote).toHaveBeenCalledWith(7));
  await settle();
}

/** A bring back of session 7 that answers `local` afterwards. */
function bringBackSucceeds() {
  vi.mocked(bringBack).mockImplementation(async () => {
    remote[7] = LOCAL;
    return { photo_ids: [71], synced: true };
  });
}

// ---------------------------------------------------------------------------------------------
// H4·C13 — the capture target never follows a remote product

it('H4·C13(a) opening an archived product from a session moves nothing', async () => {
  await inSession3ThenOpen7();
  expect(captureCalls()).toEqual([]);
});

it('H4·C13(b) opening an archived product from the Sessions page and going back makes no call', async () => {
  render(<App />);
  await screen.findByText('Chair');
  await go('#/session/7');
  await workspaceShows(7);
  await waitFor(() => expect(getRemote).toHaveBeenCalledWith(7));
  await settle();
  await go('#/');
  await waitFor(() => expect(screen.queryByTestId('workspace')).toBeNull());
  await settle();
  expect(captureCalls()).toEqual([]);
});

it('H4·C13(c) after its bring back the product becomes the capture target once (without session_changed)', async () => {
  bringBackSucceeds();
  await inSession3ThenOpen7();
  fireEvent.click(screen.getByRole('button', { name: 'Start bring back' }));
  await waitFor(() => expect(activateSession).toHaveBeenCalledWith(7));
  await settle();
  expect(captureCalls()).toEqual(['activate 7']);
});

it('H4·C13(c) after its bring back the product becomes the capture target once (with session_changed after it)', async () => {
  bringBackSucceeds();
  await inSession3ThenOpen7();
  fireEvent.click(screen.getByRole('button', { name: 'Start bring back' }));
  await waitFor(() => expect(bringBack).toHaveBeenCalledTimes(1));
  await settle();
  sessionChanged(7);
  await settle();
  expect(captureCalls()).toEqual(['activate 7']);
});

it('H4·C13(d) a visited archived product brought back stays a visit: no call', async () => {
  bringBackSucceeds();
  await visitFromSession3(7);
  fireEvent.click(screen.getByRole('button', { name: 'Start bring back' }));
  await waitFor(() => expect(bringBack).toHaveBeenCalledTimes(1));
  sessionChanged(7);
  await settle();
  expect(vi.mocked(activateSession).mock.calls).toEqual([]);
});

async function openAdopted(answers: RemoteView[]) {
  let n = 0;
  vi.mocked(getRemote).mockImplementation(async id => (id === 7 ? answers[Math.min(n++, answers.length - 1)] : LOCAL));
  window.location.hash = '#/session/7';
  render(<App />);
  await workspaceShows(7);
  await waitFor(() => expect(getRemote).toHaveBeenCalledWith(7));
  await settle();
  expect(captureCalls()).toEqual([]);
  sessionChanged(7);
  await settle();
}

it('H4·C13(e) an adopted product read as shop-only is never activated', async () => {
  await openAdopted([remoteView('unknown'), remoteView('shop_only')]);
  expect(captureCalls()).toEqual([]);
});

it('H4·C13(e) an adopted product whose read finds no pictures is activated once', async () => {
  await openAdopted([remoteView('unknown'), LOCAL]);
  expect(captureCalls()).toEqual(['activate 7']);
});

it('H4·C13(e) an adopted product whose read failed is activated once', async () => {
  await openAdopted([remoteView('unknown'), remoteView('unknown', { check_failed: true })]);
  expect(captureCalls()).toEqual(['activate 7']);
});

it('H4·C13(f) the visit bar offers Shoot here instead only for a product that can be the capture target', async () => {
  await visitFromSession3(7);
  await settle();
  const bar = screen.getByRole('status', { name: 'Visit' });
  expect(within(bar).queryByRole('button', { name: 'Shoot here instead' })).toBeNull();
  cleanup();
  vi.clearAllMocks();
  vi.mocked(getRemote).mockImplementation(async id => remote[id] ?? LOCAL);
  vi.mocked(listSessions).mockResolvedValue(LIST);
  vi.mocked(activeSession).mockResolvedValue(null);
  vi.mocked(conversationsList).mockResolvedValue({ state: 'ready', problem: null, rows: [conversationRow('c7', 7), conversationRow('c9', 9)], news_elsewhere: 0 });
  vi.mocked(conversationGet).mockImplementation(async id => conversation(id, id === 'c9' ? 9 : 7));
  vi.mocked(getSettings).mockResolvedValue(SETTINGS);
  answerFromList(sessionsPage, listSessions);
  await visitFromSession3(9);
  await waitFor(() => expect(within(screen.getByRole('status', { name: 'Visit' })).getByRole('button', { name: 'Shoot here instead' })).toBeInTheDocument());
});

// ---------------------------------------------------------------------------------------------
// H4·C14 — Refresh from Shopify and the details-only publish confirmation

it('H4·C14(a) Refresh from Shopify in the header menu, for a remote product only', async () => {
  window.location.hash = '#/session/7';
  render(<App />);
  await workspaceShows(7);
  await waitFor(() => expect(getRemote).toHaveBeenCalledWith(7));
  await settle();
  const reads = vi.mocked(getRemote).mock.calls.length;
  fireEvent.pointerDown(screen.getByRole('button', { name: 'Session actions' }), { button: 0 });
  fireEvent.click(await screen.findByRole('menuitem', { name: 'Refresh from Shopify' }));
  await waitFor(() => expect(refreshRemote).toHaveBeenCalledTimes(1));
  expect(refreshRemote).toHaveBeenCalledWith(7);
  await waitFor(() => expect(vi.mocked(getRemote).mock.calls.length).toBe(reads + 1));
  expect(vi.mocked(refreshRemote).mock.invocationCallOrder[0]).toBeLessThan(vi.mocked(getRemote).mock.invocationCallOrder[reads]);

  await go('#/session/3');
  await workspaceShows(3);
  await settle();
  fireEvent.pointerDown(screen.getByRole('button', { name: 'Session actions' }), { button: 0 });
  await screen.findByRole('menuitem', { name: 'Delete session…' });
  expect(screen.queryByRole('menuitem', { name: 'Refresh from Shopify' })).toBeNull();
});

it('H4·C14(b) Publish of an archived product confirms its details only', async () => {
  const preview = {
    summary: 'Publish details of BEYO-7',
    items: [{
      session_id: 7, main: [], condition: [], main_thumbnails: [], condition_thumbnails: [], details_filled: 3, details_total: 5,
      unreviewed: 0, announced_removals: [], create_item: false, n_withheld: 0, conflicts: [], researching: false, details_only: true,
    }],
  } as PreviewDto;
  vi.mocked(publishPreview).mockResolvedValue(preview);
  window.location.hash = '#/session/7';
  render(<App />);
  await workspaceShows(7);
  fireEvent.click(await screen.findByRole('button', { name: /Publish/ }));
  const popover = await screen.findByRole('dialog', { name: 'Review publish' });
  expect(await within(popover).findByText('Details only: the photos stay as they are on Shopify.')).toBeInTheDocument();
  fireEvent.click(within(popover).getByRole('button', { name: 'Publish details' }));
  await waitFor(() => expect(publishConfirm).toHaveBeenCalledTimes(1));
  expect(vi.mocked(publishConfirm).mock.calls[0][0]).toEqual([7]);
});
