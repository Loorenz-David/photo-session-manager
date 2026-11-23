import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionRowResponse_Serialize } from '@/services/bindings';
import type { SessionList } from '@/services/sessions';
import type { SessionResearch } from '@/services/research';

vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
  arrangeNext: vi.fn(),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/research', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/research')>()), researchOverview: vi.fn() }));
vi.mock('@/services/events', () => ({ onSessionsChanged: vi.fn(async () => () => undefined) }));
vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => null), onExportProgress: vi.fn(async () => () => undefined), openExportFolder: vi.fn(), runExport: vi.fn() }));
vi.mock('@/features/publish/usePublish', () => ({ usePublish: () => ({ failedCount: 0, label: null, notifications: [], refresh: async () => undefined }) }));
vi.mock('@/features/publish/NotificationsDrawer', () => ({ NotificationsDrawer: () => null }));
vi.mock('@/features/publish/PublishProgress', () => ({ PublishProgress: () => null }));
vi.mock('@/features/settings/StorageSection', () => ({ DiskWarning: () => null }));
vi.mock('@/features/history/HistoryButtons', () => ({ HistoryButtons: () => null }));
vi.mock('@/features/shell/CameraDot', () => ({ CameraDot: () => null }));
vi.mock('@/features/shell/ThemeToggle', () => ({ ThemeToggle: () => null }));
vi.mock('@/features/sessions/HoverPreview', () => ({ HoverPreview: ({ children }: { children: React.ReactNode }) => children, Thumb: () => null }));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }));

import { Header } from '@/features/shell/Header';
import { SessionsPage } from '@/features/sessions/SessionsPage';
import { nextToReview, showsNextToReview } from '@/features/sessions/reviewQueue';
import { listSessions } from '@/services/sessions';
import { researchOverview } from '@/services/research';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

function row(id: number, title: string, arranged = true): SessionRowResponse_Serialize {
  return {
    id, sku: `SKU-${id}`, item_type: '', title, main_count: 2, condition_count: 1, unreviewed_count: 0, status: 'in_progress',
    label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: Date.now(), created_at: Date.now(), cover_preview_path: null, cover_pending: false,
    arranged, publish: null, changes: null,
  };
}
function research(session_id: number, state: SessionResearch['state'], to_review = 0): SessionResearch {
  return { session_id, state, position: null, to_review, reason: null } as SessionResearch;
}
const LIST: SessionList = { groups: [{ label: 'Today', rows: [row(4, 'Desk'), row(3, 'Chair'), row(2, 'Lamp'), row(1, 'Unassigned')] }], counts: { open: 4, published: 0, all: 4 }, unassigned_count: 0 };

beforeEach(() => {
  vi.mocked(listSessions).mockResolvedValue(LIST);
  vi.mocked(researchOverview).mockResolvedValue([research(4, 'to_review', 3), research(3, 'researching'), research(2, 'to_review', 1)]);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('a19_next_to_review_follows_the_sessions_list_order_after_the_current_one_and_wraps', () => {
  const order = [9, 8, 7, 6, 5];
  const waiting = (id: number) => [9, 7, 5].includes(id);
  expect(nextToReview(order, waiting, 8)).toBe(7);
  expect(nextToReview(order, waiting, 7)).toBe(5);
  expect(nextToReview(order, waiting, 5)).toBe(9);
  expect(nextToReview(order, waiting, undefined)).toBe(9);
  expect(nextToReview(order, id => id === 7, 7)).toBeNull();
  expect(nextToReview(order, () => false, 8)).toBeNull();
  // A session not in the list (another filter): the first waiting one.
  expect(nextToReview(order, waiting, 42)).toBe(9);
});

it('a19_the_header_rule_reviewing_shows_next_to_review_otherwise_arranged_next', () => {
  expect(showsNextToReview(true, research(5, 'done'), 2)).toBe(true);
  expect(showsNextToReview(true, research(5, 'to_review', 3), 1)).toBe(true);
  expect(showsNextToReview(true, research(5, 'done'), 0)).toBe(false);
  expect(showsNextToReview(false, research(5, 'done'), 2)).toBe(false);
  for (const state of ['researching', 'waiting', 'failed', 'blocked'] as const) expect(showsNextToReview(true, research(5, state), 2)).toBe(false);
  expect(showsNextToReview(true, undefined, 2)).toBe(false);
});

it('a19_the_sessions_page_lists_only_the_sessions_waiting_for_review_and_opens_their_page', async () => {
  const navigate = vi.fn();
  render(<SessionsPage navigate={navigate} />);
  const toggle = await screen.findByRole('button', { name: /To review · 2/ });
  expect(toggle).toHaveAttribute('aria-pressed', 'false');
  await screen.findByText('Chair');
  fireEvent.click(toggle);
  expect(toggle).toHaveAttribute('aria-pressed', 'true');
  const table = screen.getByRole('table', { name: 'Sessions' });
  await waitFor(() => expect(within(table).queryByText('Chair')).not.toBeInTheDocument());
  expect(within(table).getByText('Desk')).toBeInTheDocument();
  expect(within(table).getByText('Lamp')).toBeInTheDocument();
  // Opening one goes to its session page (to retouch), not into Details.
  fireEvent.click(within(table).getByText('Desk'));
  expect(navigate).toHaveBeenCalledWith(4);
  // Off again: every session.
  fireEvent.click(toggle);
  expect(await within(table).findByText('Chair')).toBeInTheDocument();
});

it('a19_with_nothing_waiting_there_is_no_to_review_control', async () => {
  vi.mocked(researchOverview).mockResolvedValue([research(4, 'done'), research(3, 'researching')]);
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByText('Chair');
  expect(screen.queryByRole('button', { name: /To review/ })).not.toBeInTheDocument();
});

it('a19_a_reviewed_session_header_offers_next_to_review_with_how_many_are_left', async () => {
  vi.mocked(researchOverview).mockResolvedValue([research(3, 'done')]);
  const onNext = vi.fn();
  render(<Header sessionId={3} theme="light" onThemeChange={() => undefined} onNewSession={() => undefined} reviewQueue={{ left: 2, onNext }} />);
  fireEvent.click(await screen.findByRole('button', { name: /Next to review \(2 left\)/ }));
  expect(onNext).toHaveBeenCalledTimes(1);
  expect(screen.queryByRole('button', { name: /Arranged/ })).not.toBeInTheDocument();
});

it('a19_while_shooting_the_header_keeps_arranged_next', async () => {
  vi.mocked(researchOverview).mockResolvedValue([research(3, 'researching')]);
  render(<Header sessionId={3} theme="light" onThemeChange={() => undefined} onNewSession={() => undefined} reviewQueue={{ left: 2, onNext: () => undefined }} />);
  expect(await screen.findByRole('button', { name: /Arranged ✓ — next/ })).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: /Next to review/ })).not.toBeInTheDocument();
});
