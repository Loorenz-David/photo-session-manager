import '@testing-library/jest-dom/vitest';
import { cleanup, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionList } from '@/services/sessions';
import type { ConversationsChanged } from '@/services/events';

vi.mock('@/services/conversations', () => ({ conversationsSessionBadges: vi.fn() }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async () => () => undefined),
  onConversationsChanged: vi.fn(async (_handler: (event: ConversationsChanged) => void) => () => undefined),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(), sessionsPage: vi.fn(),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/research', () => ({ researchOverview: vi.fn(async () => []) }));

import { SessionsPage } from '../SessionsPage';
import { listSessions, sessionsPage } from '@/services/sessions';
import { answerFromList } from '@/features/sessions/__tests__/pageFromList';
import { conversationsSessionBadges } from '@/services/conversations';

function session(id: number, sku: string) {
  return {
    id, sku, item_type: 'Mirror', title: 'Mirror', arranged: false, status: 'in_progress', label: { kind: 'in_progress' },
    main_count: 0, condition_count: 0, created_at: Date.now(), cover_preview_path: null, changes: null, publish: null,
  };
}

beforeEach(() => {
  answerFromList(sessionsPage, listSessions);
  vi.mocked(listSessions).mockResolvedValue({
    groups: [{ label: 'Today', rows: [session(7, 'BEYO-7'), session(8, 'BEYO-8')] }],
    counts: { open: 2, published: 0, all: 2 },
    unassigned_count: 0,
  } as unknown as SessionList);
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('c2_c14a_a_session_with_open_conversations_shows_n_open', async () => {
  vi.mocked(conversationsSessionBadges).mockResolvedValue([{ session_id: 7, open: 2, waiting: false }]);
  render(<SessionsPage navigate={() => undefined} />);
  const row7 = await screen.findByTestId('session-row-7');
  await waitFor(() => expect(within(row7).getByText('2 open')).toBeInTheDocument());
  const row8 = screen.getByTestId('session-row-8');
  expect(within(row8).queryByText(/open$/)).toBeNull();
  expect(within(row8).queryByText('Pricing is waiting')).toBeNull();
});

it('c2_c14b_a_blocking_conversation_shows_pricing_is_waiting', async () => {
  vi.mocked(conversationsSessionBadges).mockResolvedValue([{ session_id: 7, open: 1, waiting: true }]);
  render(<SessionsPage navigate={() => undefined} />);
  const row7 = await screen.findByTestId('session-row-7');
  await waitFor(() => expect(within(row7).getByText('Pricing is waiting')).toBeInTheDocument());
  expect(within(row7).queryByText('1 open')).toBeNull();
});
