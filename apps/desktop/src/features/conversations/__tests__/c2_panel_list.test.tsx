import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ConversationsChanged } from '@/services/events';
import type { ConversationList, ConversationListRow, ConversationsSettings } from '@/services/conversations';

const delivered = vi.hoisted(() => ({ changed: [] as ((event: ConversationsChanged) => void)[] }));

vi.mock('@/services/conversations', () => ({
  conversationsCounts: vi.fn(async () => ({ here: 0, elsewhere: 0 })),
  conversationsList: vi.fn(),
  conversationsSettingsGet: vi.fn(),
  conversationsSettingsSave: vi.fn(),
  conversationGet: vi.fn(),
  conversationMarkRead: vi.fn(async () => undefined),
}));
vi.mock('@/services/events', () => ({
  onConversationsChanged: vi.fn(async (handler: (event: ConversationsChanged) => void) => {
    delivered.changed.push(handler);
    return () => undefined;
  }),
}));

import { ConversationsContext, useConversations } from '../useConversations';
import { ConversationsButton } from '../ConversationsButton';
import { ConversationsPanel } from '../ConversationsPanel';
import { ConversationRow } from '../ConversationRow';
import {
  conversationGet,
  conversationsList,
  conversationsSettingsGet,
  conversationsSettingsSave,
  type ConversationView,
} from '@/services/conversations';

function row(extra: Partial<ConversationListRow> = {}): ConversationListRow {
  return {
    id: 'c1',
    sku: 'BEYO-1880',
    product_title: 'Vintage Brass Wall Mirror, Round',
    topic: 'Condition',
    status: 'open',
    blocking: true,
    waiting: true,
    last_line: 'Is the glass original?',
    last_activity_at: Date.now() - 5 * 60_000,
    has_news: true,
    session_id: 7,
    ...extra,
  };
}

function list(extra: Partial<ConversationList> = {}): ConversationList {
  return { state: 'ready', problem: null, rows: [], news_elsewhere: 0, ...extra };
}

function settings(sound: boolean): ConversationsSettings {
  return { sound } as ConversationsSettings;
}

function Harness({ sessionId }: { sessionId?: number }) {
  const conversations = useConversations(sessionId);
  return (
    <ConversationsContext.Provider value={conversations}>
      <ConversationsButton />
      <ConversationsPanel />
    </ConversationsContext.Provider>
  );
}

async function openPanel(sessionId?: number) {
  render(<Harness sessionId={sessionId} />);
  fireEvent.click(screen.getByRole('button', { name: 'Conversations' }));
  return screen.findByRole('dialog', { name: 'Conversations' });
}

function deliver() {
  act(() => delivered.changed.forEach(handler => handler({ toasts: [], more: 0 })));
}

beforeEach(() => {
  delivered.changed = [];
  vi.mocked(conversationsList).mockResolvedValue(list());
  vi.mocked(conversationsSettingsGet).mockResolvedValue(settings(true));
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  window.location.hash = '';
});

// ---------- C2·C9 opening, tabs, closing ----------

it('c2_c9a_the_icon_opens_the_panel_on_the_open_tab', async () => {
  const panel = await openPanel();
  expect(within(panel).getByRole('tab', { name: 'Open' })).toHaveAttribute('aria-selected', 'true');
  await waitFor(() => expect(conversationsList).toHaveBeenCalledWith({ kind: 'all' }, 'open'));
});

it('c2_c9b_the_resolved_tab_lists_the_resolved_rows', async () => {
  const panel = await openPanel();
  vi.mocked(conversationsList).mockResolvedValue(list({ rows: [row({ id: 'r1', sku: 'BEYO-7', status: 'resolved', waiting: false })] }));
  fireEvent.click(within(panel).getByRole('tab', { name: 'Resolved' }));
  await waitFor(() => expect(conversationsList).toHaveBeenCalledWith({ kind: 'all' }, 'resolved'));
  expect(await within(panel).findByText('BEYO-7')).toBeInTheDocument();
});

it('c2_c9c_the_close_button_closes_the_panel', async () => {
  const panel = await openPanel();
  fireEvent.click(within(panel).getByRole('button', { name: 'Close' }));
  expect(screen.queryByRole('dialog', { name: 'Conversations' })).toBeNull();
});

// ---------- C2·C10 the row ----------

it('c2_c10a_a_row_shows_its_product_topic_line_waiting_and_unread', () => {
  render(<ConversationRow row={row()} onSelect={() => undefined} />);
  expect(screen.getByText('BEYO-1880')).toBeInTheDocument();
  expect(screen.getByText('Vintage Brass Wall Mirror, Round')).toBeInTheDocument();
  expect(screen.getByText('Condition')).toBeInTheDocument();
  expect(screen.getByRole('img', { name: 'Open' })).toBeInTheDocument();
  expect(screen.getByText('Is the glass original?')).toBeInTheDocument();
  expect(screen.getByText('Pricing is waiting')).toBeInTheDocument();
  expect(screen.getByRole('img', { name: 'Unread' })).toBeInTheDocument();
  expect(screen.queryByText('Not on this Mac')).toBeNull();
});

it('c2_c10b_no_waiting_and_no_news_show_neither', () => {
  render(<ConversationRow row={row({ waiting: false, has_news: false })} onSelect={() => undefined} />);
  expect(screen.queryByText('Pricing is waiting')).toBeNull();
  expect(screen.queryByRole('img', { name: 'Unread' })).toBeNull();
});

it('c2_c10c_a_row_without_a_session_is_not_on_this_mac', () => {
  render(<ConversationRow row={row({ session_id: null })} onSelect={() => undefined} />);
  expect(screen.getByText('Not on this Mac')).toBeInTheDocument();
});

it('c2_c10d_a_row_without_a_sku_shows_the_title_alone', () => {
  render(<ConversationRow row={row({ sku: null })} onSelect={() => undefined} />);
  const product = screen.getByText('Vintage Brass Wall Mirror, Round').parentElement!;
  expect(product.textContent).toBe('Vintage Brass Wall Mirror, Round');
});

// ---------- C2·C11 inside a session ----------

it('c2_c11a_inside_a_session_the_list_is_that_session_s', async () => {
  await openPanel(7);
  await waitFor(() => expect(conversationsList).toHaveBeenCalledWith({ kind: 'session', session_id: 7 }, 'open'));
  expect(conversationsList).not.toHaveBeenCalledWith({ kind: 'all' }, 'open');
});

it('c2_c11b_the_footer_switches_the_list_to_all_conversations', async () => {
  vi.mocked(conversationsList).mockImplementation(async scope =>
    scope.kind === 'all' ? list({ rows: [row({ id: 'b1', sku: 'BEYO-2' })] }) : list({ news_elsewhere: 2 }),
  );
  const panel = await openPanel(7);
  const footer = await within(panel).findByRole('button', { name: '2 new in other sessions → All conversations' });
  fireEvent.click(footer);
  await waitFor(() => expect(conversationsList).toHaveBeenCalledWith({ kind: 'all' }, 'open'));
  expect(await within(panel).findByText('BEYO-2')).toBeInTheDocument();
});

it('c2_c11c_no_news_elsewhere_no_footer', async () => {
  vi.mocked(conversationsList).mockResolvedValue(list({ rows: [row()] }));
  const panel = await openPanel(7);
  await within(panel).findByText('BEYO-1880');
  expect(within(panel).queryByText(/new in other sessions/)).toBeNull();
});

// ---------- C2·C12 states ----------

it('c2_c12a_without_a_key_the_panel_asks_to_connect', async () => {
  vi.mocked(conversationsList).mockResolvedValue(list({ state: 'no_key' }));
  const panel = await openPanel();
  expect(await within(panel).findByText('Connect Pricing in Settings → Pricing')).toBeInTheDocument();
  expect(within(panel).queryByRole('tab')).toBeNull();
  fireEvent.click(within(panel).getByRole('button', { name: 'Open Settings → Pricing' }));
  expect(window.location.hash).toBe('#/settings');
});

it('c2_c12b_a_refused_key_shows_its_problem_above_the_rows', async () => {
  vi.mocked(conversationsList).mockResolvedValue(list({
    state: 'refused',
    problem: 'Pricing refused the key: check Settings → Pricing',
    rows: [row({ id: 'a', sku: 'BEYO-1' }), row({ id: 'b', sku: 'BEYO-2' })],
  }));
  const panel = await openPanel();
  expect(await within(panel).findByRole('alert')).toHaveTextContent('Pricing refused the key: check Settings → Pricing');
  expect(within(panel).getByText('BEYO-1')).toBeInTheDocument();
  expect(within(panel).getByText('BEYO-2')).toBeInTheDocument();
});

it('c2_c12c_an_empty_open_tab_says_so', async () => {
  const panel = await openPanel();
  expect(await within(panel).findByText('No open conversations')).toBeInTheDocument();
});

// ---------- C2·C13 mute ----------

it('c2_c13a_mute_saves_the_sound_off', async () => {
  vi.mocked(conversationsSettingsSave).mockResolvedValue(settings(false));
  const panel = await openPanel();
  const mute = within(panel).getByRole('button', { name: 'Mute' });
  await waitFor(() => expect(mute).toBeEnabled());
  expect(mute).toHaveAttribute('aria-pressed', 'false');
  fireEvent.click(mute);
  await waitFor(() => expect(mute).toHaveAttribute('aria-pressed', 'true'));
  expect(conversationsSettingsSave).toHaveBeenCalledTimes(1);
  expect(conversationsSettingsSave).toHaveBeenCalledWith({ sound: false });
});

it('c2_c13b_a_muted_sound_reads_muted_and_unmutes', async () => {
  vi.mocked(conversationsSettingsGet).mockResolvedValue(settings(false));
  vi.mocked(conversationsSettingsSave).mockResolvedValue(settings(true));
  const panel = await openPanel();
  const mute = within(panel).getByRole('button', { name: 'Mute' });
  await waitFor(() => expect(mute).toHaveAttribute('aria-pressed', 'true'));
  fireEvent.click(mute);
  await waitFor(() => expect(conversationsSettingsSave).toHaveBeenCalledWith({ sound: true }));
});

// ---------- C2·C15 live list, selecting a row ----------

it('c2_c15a_conversations_changed_adds_the_new_row_to_the_open_panel', async () => {
  vi.mocked(conversationsList).mockResolvedValue(list({ rows: [row({ id: 'a', sku: 'BEYO-1' })] }));
  const panel = await openPanel();
  await within(panel).findByText('BEYO-1');
  vi.mocked(conversationsList).mockResolvedValue(list({ rows: [row({ id: 'a', sku: 'BEYO-1' }), row({ id: 'b', sku: 'BEYO-2' })] }));
  deliver();
  expect(await within(panel).findByText('BEYO-2')).toBeInTheDocument();
  expect(screen.getByRole('dialog', { name: 'Conversations' })).toBe(panel);
});

it('c2_c15c_a_row_opens_its_conversation_and_the_back_arrow_returns', async () => {
  vi.mocked(conversationsList).mockResolvedValue(list({ rows: [row({ id: 'a', sku: 'BEYO-1', topic: 'Dimensions' })] }));
  vi.mocked(conversationGet).mockResolvedValue({
    id: 'a',
    sku: 'BEYO-1',
    product_title: 'Vintage Brass Wall Mirror, Round',
    topic: 'Dimensions',
    blocking: false,
    status: 'open',
    resolved_by: null,
    session_id: 7,
    messages: [],
    pending: [],
    can_send: true,
    off_reason: null,
  } satisfies ConversationView);
  const panel = await openPanel();
  fireEvent.click(await within(panel).findByTestId('conversation-row-a'));
  expect(within(panel).getByTestId('conversation-view')).toBeInTheDocument();
  expect(await within(panel).findByText('Dimensions')).toBeInTheDocument();
  expect(within(panel).getByText('BEYO-1')).toBeInTheDocument();
  expect(vi.mocked(conversationGet)).toHaveBeenCalledWith('a');
  fireEvent.click(within(panel).getByRole('button', { name: 'Back to conversations' }));
  expect(within(panel).getByTestId('conversation-row-a')).toBeInTheDocument();
  expect(within(panel).queryByTestId('conversation-view')).toBeNull();
});
