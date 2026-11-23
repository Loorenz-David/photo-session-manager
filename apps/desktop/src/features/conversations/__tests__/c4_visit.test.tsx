import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionRowResponse_Serialize } from '@/services/bindings';
import type { ConversationListRow, ConversationView as View } from '@/services/conversations';
import type { Session, SessionList } from '@/services/sessions';
import type { Settings } from '@/services/settings';

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
  onWorkspaceEvents: vi.fn(async () => () => undefined),
  onPhotoAdded: vi.fn(async () => () => undefined),
  onFileDrop: vi.fn(async () => () => undefined),
  onResearchChanged: vi.fn(async () => () => undefined),
  onConversationsChanged: vi.fn(async () => () => undefined),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
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
vi.mock('@/services/publish', () => ({ publishRetry: vi.fn() }));
vi.mock('@/services/health', () => ({ getHealth: vi.fn(() => new Promise(() => undefined)), getAppPaths: vi.fn(() => new Promise(() => undefined)) }));
vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), pickSessionFile: vi.fn() }));
vi.mock('@/services/styles', () => ({ listStyles: vi.fn(async () => null) }));
vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => null), onExportProgress: vi.fn(async () => () => undefined), openExportFolder: vi.fn(), runExport: vi.fn() }));
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
// The workspace's own content is not part of the jump; it shows which session is on screen.
vi.mock('@/features/workspace/WorkspacePage', () => ({
  WorkspacePage: ({ sessionId }: { sessionId: number }) => <main data-testid="workspace">Workspace of session {sessionId}</main>,
}));
vi.mock('sonner', () => ({ toast: Object.assign(vi.fn(), { error: vi.fn(), success: vi.fn(), loading: vi.fn(), dismiss: vi.fn() }) }));
vi.mock('@/components/ui/sonner', () => ({ Toaster: () => null }));

import App from '@/App';
import { conversationGet, conversationsList } from '@/services/conversations';
import { activateSession, activeSession, deactivateSession, listSessions } from '@/services/sessions';
import { getSettings } from '@/services/settings';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

function sessionRow(id: number, sku: string, title: string): SessionRowResponse_Serialize {
  return {
    id, sku, item_type: '', title, main_count: 2, condition_count: 1, unreviewed_count: 0, status: 'in_progress',
    label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: Date.now(), created_at: Date.now(), cover_preview_path: null,
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

/** `App` in session 3 (its activation already made), then **Open session** on a conversation of session 7. */
async function visit7FromSession3() {
  window.location.hash = '#/session/3';
  render(<App />);
  await waitFor(() => expect(activateSession).toHaveBeenCalledWith(3));
  resetCaptureCalls();
  await openConversationOf(7);
  fireEvent.click(await openSessionButton());
  await workspaceShows(7);
}

/** `App` on the Sessions page, then **Open session** on a conversation of session 7. */
async function visit7FromSessions() {
  render(<App />);
  await screen.findByText('Chair');
  await openConversationOf(7);
  fireEvent.click(await openSessionButton());
  await workspaceShows(7);
}

function bar() {
  return screen.queryByRole('status', { name: 'Visit' });
}

/** Lets pending hash events and effects run, so a call that would follow is made before the assertion. */
async function settle() {
  await new Promise(resolve => setTimeout(resolve, 20));
}

it('c4_c3a_open_session_from_session_3_shows_session_7_without_moving_the_capture_folder', async () => {
  await visit7FromSession3();
  expect(window.location.hash).toBe('#/session/7');
  await settle();
  expect(captureCalls()).toEqual([]);
});

it('c4_c3b_open_session_from_the_sessions_page_shows_session_7_with_its_back_link_and_no_call', async () => {
  await visit7FromSessions();
  expect(screen.getByRole('button', { name: 'Back to Sessions' })).toBeInTheDocument();
  await settle();
  expect(captureCalls()).toEqual([]);
});

it('c4_c3c_after_the_jump_the_panel_is_still_open_on_the_same_conversation', async () => {
  await visit7FromSession3();
  const panel = screen.getByRole('dialog', { name: 'Conversations' });
  expect(within(panel).getByTestId('conversation-view')).toBeInTheDocument();
  expect(within(panel).getByText('Product 7', { selector: '.conversation-title' })).toBeInTheDocument();
  // The conversation's session is now on screen: no Open session.
  expect(within(panel).queryByRole('button', { name: 'Open session' })).toBeNull();
});

async function dirtySettingsThenOpenSession() {
  window.location.hash = '#/settings';
  render(<App />);
  fireEvent.change(await screen.findByLabelText('Preview recipe'), { target: { value: 'PSM Preview 2' } });
  await openConversationOf(7);
  fireEvent.click(await openSessionButton());
  const dialog = await screen.findByRole('dialog', { name: 'Save your changes?' });
  expect(screen.getByText('Settings', { selector: 'h1' })).toBeInTheDocument();
  expect(window.location.hash).toBe('#/settings');
  return dialog;
}

it('c4_c3d_from_dirty_settings_open_session_asks_first_and_discard_opens_the_visit_without_a_call', async () => {
  const dialog = await dirtySettingsThenOpenSession();
  fireEvent.click(within(dialog).getByRole('button', { name: 'Discard' }));
  await workspaceShows(7);
  expect(window.location.hash).toBe('#/session/7');
  await waitFor(() => expect(bar()).toBeInTheDocument());
  expect(within(bar()!).getByRole('button', { name: '← Back to Settings' })).toBeInTheDocument();
  await settle();
  expect(captureCalls()).toEqual([]);
});

it('c4_c3d_from_dirty_settings_cancel_keeps_settings_and_makes_no_call', async () => {
  const dialog = await dirtySettingsThenOpenSession();
  fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
  await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Save your changes?' })).toBeNull());
  await settle();
  expect(screen.getByText('Settings', { selector: 'h1' })).toBeInTheDocument();
  expect(screen.queryByTestId('workspace')).toBeNull();
  expect(window.location.hash).toBe('#/settings');
  expect(captureCalls()).toEqual([]);
});

it('c4_c4a_the_bar_names_the_visited_session_and_the_session_the_backend_reports_active', async () => {
  vi.mocked(activeSession).mockResolvedValue({ id: 3 } as Session);
  await visit7FromSession3();
  await waitFor(() => expect(within(bar()!).getByText('Visiting BEYO-7: new photos still go to BEYO-3.')).toBeInTheDocument());
  expect(within(bar()!).getByRole('button', { name: '← Back to BEYO-3' })).toBeInTheDocument();
  expect(within(bar()!).getByRole('button', { name: 'Shoot here instead' })).toBeInTheDocument();
});

it('c4_c4b_from_the_sessions_page_the_bar_says_unassigned_and_back_to_sessions', async () => {
  await visit7FromSessions();
  await waitFor(() => expect(within(bar()!).getByText('Visiting BEYO-7: new photos still go to Unassigned.')).toBeInTheDocument());
  expect(within(bar()!).getByRole('button', { name: '← Back to Sessions' })).toBeInTheDocument();
});

it('c4_c4c_in_a_session_without_a_visit_there_is_no_bar', async () => {
  window.location.hash = '#/session/3';
  render(<App />);
  await workspaceShows(3);
  await settle();
  expect(bar()).toBeNull();
});

it('c4_c5a_back_returns_to_session_3_and_makes_no_call', async () => {
  await visit7FromSession3();
  fireEvent.click(within(bar()!).getByRole('button', { name: /^← Back to/ }));
  await workspaceShows(3);
  expect(window.location.hash).toBe('#/session/3');
  expect(bar()).toBeNull();
  await settle();
  expect(captureCalls()).toEqual([]);
});

it('c4_c5b_back_returns_to_the_sessions_page_and_makes_no_call', async () => {
  await visit7FromSessions();
  fireEvent.click(within(bar()!).getByRole('button', { name: '← Back to Sessions' }));
  await waitFor(() => expect(screen.queryByTestId('workspace')).toBeNull());
  expect(screen.getByRole('table', { name: 'Sessions' })).toBeInTheDocument();
  expect(bar()).toBeNull();
  await settle();
  expect(captureCalls()).toEqual([]);
});

it('c4_c5c_back_after_two_jumps_returns_to_session_3_without_a_call', async () => {
  await visit7FromSession3();
  const panel = screen.getByRole('dialog', { name: 'Conversations' });
  fireEvent.click(within(panel).getByRole('button', { name: 'Back to conversations' }));
  fireEvent.click(await within(panel).findByText('Topic 9'));
  await within(panel).findByText('Product 9', { selector: '.conversation-title' });
  fireEvent.click(await openSessionButton());
  await workspaceShows(9);
  fireEvent.click(within(bar()!).getByRole('button', { name: /^← Back to/ }));
  await workspaceShows(3);
  expect(bar()).toBeNull();
  await settle();
  expect(captureCalls()).toEqual([]);
});

it('c4_c6a_shoot_here_activates_the_visited_session_once_and_ends_the_visit', async () => {
  await visit7FromSession3();
  fireEvent.click(within(bar()!).getByRole('button', { name: 'Shoot here instead' }));
  await waitFor(() => expect(bar()).toBeNull());
  await settle();
  expect(captureCalls()).toEqual(['activate 7']);
  await workspaceShows(7);
});

it('c4_c7a_the_header_back_link_to_sessions_ends_the_visit_and_deactivates_once', async () => {
  await visit7FromSession3();
  fireEvent.click(screen.getByRole('button', { name: 'Back to Sessions' }));
  await waitFor(() => expect(screen.queryByTestId('workspace')).toBeNull());
  expect(bar()).toBeNull();
  await settle();
  expect(captureCalls()).toEqual(['deactivate']);
});

it('c4_c7b_the_notifications_drawer_open_ends_the_visit_and_activates_that_session', async () => {
  await visit7FromSessions();
  fireEvent.click(screen.getByRole('button', { name: 'Notifications' }));
  const drawer = await screen.findByRole('dialog', { name: 'Notifications' });
  fireEvent.click(within(drawer).getByRole('button', { name: 'Open' }));
  await workspaceShows(5);
  await waitFor(() => expect(bar()).toBeNull());
  await settle();
  expect(captureCalls()).toEqual(['activate 5']);
});

it('c4_c7c_without_a_visit_opening_and_leaving_a_session_moves_the_capture_folder_as_before', async () => {
  render(<App />);
  fireEvent.click(await screen.findByText('Desk'));
  await workspaceShows(5);
  fireEvent.click(screen.getByRole('button', { name: 'Back to Sessions' }));
  await waitFor(() => expect(screen.queryByTestId('workspace')).toBeNull());
  await settle();
  expect(captureCalls()).toEqual(['activate 5', 'deactivate']);
});
