import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionRowResponse_Serialize } from '@/services/bindings';
import type { LabelsChanged } from '@/services/events';
import type { LabelsStatus, SessionLabel } from '@/services/labels';
import type { SessionList, SessionPage } from '@/services/sessions';

/**
 * L1·U4 (`plans/L1_labels_provider_and_marker.md` L1·C18): the label markers on the Sessions page's rows and beside the
 * session page's title (MC-L3.6, MC-L3.7), with `labelsStatus` and the events mocked.
 */

const delivered = vi.hoisted(() => ({ changed: [] as (() => void)[], labels: [] as ((event: LabelsChanged) => void)[] }));

vi.mock('@/services/labels', () => ({ labelsStatus: vi.fn() }));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
  sessionsPage: vi.fn(),
  arrangeNext: vi.fn(),
}));
vi.mock('@/services/research', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/research')>()), researchOverview: vi.fn(async () => []) }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async (refresh: () => void) => {
    delivered.changed.push(refresh);
    return () => undefined;
  }),
  onPictureReady: vi.fn(async () => () => undefined),
  onLabelsChanged: vi.fn(async (handler: (event: LabelsChanged) => void) => {
    delivered.labels.push(handler);
    return () => undefined;
  }),
  onConversationsChanged: vi.fn(async () => () => undefined),
}));
vi.mock('@/services/conversations', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/conversations')>()),
  conversationsSessionBadges: vi.fn(async () => []),
  conversationsCounts: vi.fn(async () => ({ here: 0, elsewhere: 0 })),
}));
vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => null), onExportProgress: vi.fn(async () => () => undefined), openExportFolder: vi.fn(), runExport: vi.fn() }));
vi.mock('@/features/publish/usePublish', () => ({ usePublish: () => ({ failedCount: 0, label: null, notifications: [], refresh: async () => undefined }) }));
vi.mock('@/features/publish/NotificationsDrawer', () => ({ NotificationsDrawer: () => null }));
vi.mock('@/features/publish/PublishProgress', () => ({ PublishProgress: () => null }));
vi.mock('@/features/settings/StorageSection', () => ({ DiskWarning: () => null }));
vi.mock('@/features/history/HistoryButtons', () => ({ HistoryButtons: () => null }));
vi.mock('@/features/shell/CameraDot', () => ({ CameraDot: () => null }));
vi.mock('@/features/shell/ThemeToggle', () => ({ ThemeToggle: () => null }));
vi.mock('@/features/conversations/ConversationsButton', () => ({ ConversationsButton: () => null }));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }));

import { toast } from 'sonner';
import { SessionsPage } from '@/features/sessions/SessionsPage';
import { Header } from '@/features/shell/Header';
import { labelsStatus } from '@/services/labels';
import { listSessions, sessionsPage } from '@/services/sessions';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

const NOV_4 = new Date(new Date().getFullYear(), 10, 4, 9, 12).getTime();

function row(id: number): SessionRowResponse_Serialize {
  return {
    id, sku: `SKU-${id}`, item_type: '', title: `Item ${id}`, main_count: 2, condition_count: 1, unreviewed_count: 0, status: 'in_progress',
    label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: Date.now(), created_at: Date.now(), cover_preview_path: null,
    cover_pending: false, arranged: false, publish: null, changes: null,
  };
}

/** A fresh answer object each time (as the backend's), page 1 = rows 1–3 of two pages, page 2 = rows 4–5. */
function answer(page: number): SessionPage {
  const rows = page === 1 ? [row(1), row(2), row(3)] : [row(4), row(5)];
  return { groups: [{ label: 'Today', total: rows.length, rows }], counts: { open: 5, published: 0, all: 5 }, unassigned_count: 0, page, pages: 2, total: 5 };
}

function printed(id: number): SessionLabel {
  return { session_id: id, product_id: `P${id}`, marker: { status: 'printed', printed_at: NOV_4, has_price: true, reset: null } };
}

function missingTag(id: number): SessionLabel {
  return { session_id: id, product_id: `P${id}`, marker: { status: 'not_printed', printed_at: null, has_price: true, reset: null } };
}

function status(labels: SessionLabel[]): LabelsStatus {
  return { access: { state: 'allowed', reason: null }, labels };
}

function calls() {
  return vi.mocked(labelsStatus).mock.calls.map(([args]) => args);
}

function pill(id: number): string | null {
  return within(screen.getByTestId(`session-row-${id}`)).queryByTestId('label-pill')?.textContent ?? null;
}

async function shown() {
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-3');
}

beforeEach(() => {
  delivered.changed.length = 0;
  delivered.labels.length = 0;
  vi.mocked(sessionsPage).mockImplementation(async args => answer(args.page));
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('l1_c18a_rows_render_before_the_markers_answer', async () => {
  vi.mocked(labelsStatus).mockReturnValue(new Promise(() => undefined));
  await shown();
  expect(screen.getByTestId('session-row-1')).toBeInTheDocument();
  expect(screen.getByTestId('session-row-2')).toBeInTheDocument();
  await waitFor(() => expect(calls()).toEqual([{ session_ids: [1, 2, 3], refresh: true }]));
});

it('l1_c18b_each_row_shows_its_marker', async () => {
  vi.mocked(labelsStatus).mockResolvedValue(status([printed(1), missingTag(2), { session_id: 3, product_id: 'P3', marker: null }]));
  await shown();
  await waitFor(() => expect(pill(1)).toBe('Label printed · 4 Nov'));
  expect(pill(2)).toBe('Missing price tag');
  expect(pill(3)).toBeNull();
});

it('l1_c18c_a_reload_of_the_same_page_does_not_refresh_another_page_does', async () => {
  vi.mocked(labelsStatus).mockResolvedValue(status([]));
  await shown();
  await waitFor(() => expect(calls()).toHaveLength(1));
  vi.mocked(labelsStatus).mockClear();
  await act(async () => delivered.changed.forEach(refresh => refresh()));
  await waitFor(() => expect(calls()).toEqual([{ session_ids: [1, 2, 3], refresh: false }]));
  fireEvent.click(screen.getByRole('button', { name: 'Page 2' }));
  await screen.findByTestId('session-row-5');
  await waitFor(() => expect(calls()).toEqual([
    { session_ids: [1, 2, 3], refresh: false },
    { session_ids: [4, 5], refresh: true },
  ]));
});

it('l1_c18d_labels_changed_asks_again_and_the_pills_follow', async () => {
  vi.mocked(labelsStatus).mockResolvedValue(status([missingTag(1)]));
  await shown();
  await waitFor(() => expect(pill(1)).toBe('Missing price tag'));
  vi.mocked(labelsStatus).mockClear();
  vi.mocked(labelsStatus).mockResolvedValue(status([printed(1)]));
  await act(async () => delivered.labels.forEach(handler => handler({ product_ids: ['P1'], access: false })));
  await waitFor(() => expect(calls()).toEqual([{ session_ids: [1, 2, 3], refresh: false }]));
  await waitFor(() => expect(pill(1)).toBe('Label printed · 4 Nov'));
  vi.mocked(labelsStatus).mockResolvedValue(status([{ session_id: 1, product_id: 'P1', marker: null }]));
  await act(async () => delivered.labels.forEach(handler => handler({ product_ids: [], access: true })));
  await waitFor(() => expect(calls()).toEqual([
    { session_ids: [1, 2, 3], refresh: false },
    { session_ids: [1, 2, 3], refresh: true },
  ]));
  await waitFor(() => expect(pill(1)).toBeNull());
});

it('l1_c18e_a_failed_markers_call_leaves_the_rows_alone', async () => {
  vi.mocked(labelsStatus).mockRejectedValue(new Error('Pricing can\'t be reached right now.'));
  await shown();
  await waitFor(() => expect(calls()).toHaveLength(1));
  await act(async () => undefined);
  expect(screen.getByTestId('session-row-1')).toBeInTheDocument();
  expect(screen.getByTestId('session-row-3')).toBeInTheDocument();
  expect(screen.queryByTestId('label-pill')).not.toBeInTheDocument();
  expect(toast.error).not.toHaveBeenCalled();
});

it('l1_c18f_the_session_page_shows_the_marker_beside_its_title', async () => {
  vi.mocked(listSessions).mockResolvedValue({ groups: [{ label: 'Today', rows: [{ ...row(5), item_type: 'Chair', label: { kind: 'ready' } }] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 0 } as SessionList);
  vi.mocked(labelsStatus).mockResolvedValue(status([printed(5)]));
  render(<Header sessionId={5} theme="light" onThemeChange={() => undefined} onNewSession={() => undefined} />);
  const label = await screen.findByTestId('label-pill');
  expect(label).toHaveTextContent('Label printed · 4 Nov');
  expect(label.closest('.page-header, header')).not.toBeNull();
  expect(calls()).toEqual([{ session_ids: [5], refresh: true }]);
});
