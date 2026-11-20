import '@testing-library/jest-dom/vitest';
import { act, cleanup, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionList } from '@/services/sessions';
import type { ConversationsChanged } from '@/services/events';

/** The `conversations_changed` handlers the app registered, so a test can deliver the event. */
const delivered = vi.hoisted(() => ({ changed: [] as ((event: ConversationsChanged) => void)[] }));

vi.mock('@/services/conversations', () => ({
  conversationsCounts: vi.fn(),
  conversationsList: vi.fn(async () => ({ state: 'ready', problem: null, rows: [], news_elsewhere: 0 })),
  conversationsSettingsGet: vi.fn(async () => ({ sound: true })),
  conversationsSettingsSave: vi.fn(),
  conversationsSessionBadges: vi.fn(async () => []),
}));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async () => () => undefined),
  onCameraStatus: vi.fn(async () => () => undefined),
  onWorkspaceEvents: vi.fn(async () => () => undefined),
  onPhotoAdded: vi.fn(async () => () => undefined),
  onFileDrop: vi.fn(async () => () => undefined),
  onConversationsChanged: vi.fn(async (handler: (event: ConversationsChanged) => void) => {
    delivered.changed.push(handler);
    return () => undefined;
  }),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
  arrangeNext: vi.fn(),
  activeSession: vi.fn(async () => null),
  activateSession: vi.fn(async () => undefined),
  deactivateSession: vi.fn(async () => undefined),
}));
vi.mock('@/services/research', () => ({ researchOverview: vi.fn(async () => []) }));
vi.mock('@/services/health', () => ({ getHealth: vi.fn(() => new Promise(() => undefined)), getAppPaths: vi.fn(() => new Promise(() => undefined)) }));
vi.mock('@/services/settings', () => ({ getSettings: vi.fn(() => new Promise(() => undefined)), saveSettings: vi.fn(), pickFolder: vi.fn(), pickSessionFile: vi.fn() }));
vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => null), onExportProgress: vi.fn(async () => () => undefined), openExportFolder: vi.fn(), runExport: vi.fn() }));
vi.mock('@/features/publish/usePublish', () => ({ usePublish: () => ({ failedCount: 0, label: null, notifications: [], refresh: async () => undefined }) }));
vi.mock('@/features/publish/NotificationsDrawer', () => ({ NotificationsDrawer: () => null }));
vi.mock('@/features/publish/PublishProgress', () => ({ PublishProgress: () => null }));
vi.mock('@/features/settings/StorageSection', () => ({ DiskWarning: () => null, StorageSection: () => null }));
vi.mock('@/features/history/HistoryButtons', () => ({ HistoryButtons: () => null }));
vi.mock('@/features/shell/CameraDot', () => ({ CameraDot: () => null }));
vi.mock('@/features/shell/ThemeToggle', () => ({ ThemeToggle: () => null, useTheme: () => ['light', () => undefined] }));
vi.mock('sonner', () => ({ toast: Object.assign(vi.fn(), { error: vi.fn(), success: vi.fn(), loading: vi.fn(), dismiss: vi.fn() }) }));
vi.mock('@/components/ui/sonner', () => ({ Toaster: () => null }));

import App from '@/App';
import { Header } from '@/features/shell/Header';
import { ConversationsContext, useConversations } from '../useConversations';
import { ConversationsPanel } from '../ConversationsPanel';
import { conversationsCounts } from '@/services/conversations';
import { listSessions } from '@/services/sessions';

function Harness({ sessionId }: { sessionId?: number }) {
  const conversations = useConversations(sessionId);
  return (
    <ConversationsContext.Provider value={conversations}>
      <Header sessionId={sessionId} theme="light" onThemeChange={() => undefined} onNewSession={() => undefined} />
      <ConversationsPanel />
    </ConversationsContext.Provider>
  );
}

function button() {
  return screen.getByRole('button', { name: 'Conversations' });
}

beforeEach(() => {
  delivered.changed = [];
  vi.mocked(listSessions).mockResolvedValue({ groups: [{ label: 'Today', rows: [{ id: 7, sku: 'BEYO-1880', item_type: 'Mirror', title: 'Mirror', arranged: false, label: { kind: 'ready' } }] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 0 } as SessionList);
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  window.location.hash = '';
});

it('c2_c8a_the_sessions_page_icon_shows_the_count_of_all_sessions', async () => {
  vi.mocked(conversationsCounts).mockResolvedValue({ here: 3, elsewhere: 0 });
  render(<Harness />);
  await waitFor(() => expect(button()).toHaveTextContent('3'));
  expect(conversationsCounts).toHaveBeenCalledWith(null);
});

it('c2_c8b_no_news_shows_no_count_and_no_dot', async () => {
  vi.mocked(conversationsCounts).mockResolvedValue({ here: 0, elsewhere: 0 });
  render(<Harness />);
  await waitFor(() => expect(conversationsCounts).toHaveBeenCalled());
  expect(button()).toHaveTextContent(/^$/);
  expect(screen.queryByTestId('conversations-dot')).toBeNull();
});

it('c2_c8c_inside_a_session_the_count_is_its_own_and_a_dot_tells_of_others', async () => {
  vi.mocked(conversationsCounts).mockResolvedValue({ here: 1, elsewhere: 2 });
  render(<Harness sessionId={7} />);
  await waitFor(() => expect(button()).toHaveTextContent('1'));
  expect(within(button()).getByTestId('conversations-dot')).toBeInTheDocument();
  expect(conversationsCounts).toHaveBeenCalledWith(7);
});

it.each(['#/settings', '#/health'])('c2_c8d_the_icon_is_on_%s', async hash => {
  vi.mocked(conversationsCounts).mockResolvedValue({ here: 2, elsewhere: 0 });
  window.location.hash = hash;
  render(<App />);
  await waitFor(() => expect(button()).toHaveTextContent('2'));
});

it('c2_c15b_the_count_follows_conversations_changed', async () => {
  vi.mocked(conversationsCounts).mockResolvedValue({ here: 1, elsewhere: 0 });
  render(<Harness />);
  await waitFor(() => expect(button()).toHaveTextContent('1'));
  vi.mocked(conversationsCounts).mockResolvedValue({ here: 2, elsewhere: 0 });
  await waitFor(() => expect(delivered.changed.length).toBeGreaterThan(0));
  act(() => delivered.changed.forEach(handler => handler({ toasts: [], more: 0 })));
  await waitFor(() => expect(button()).toHaveTextContent('2'));
});

