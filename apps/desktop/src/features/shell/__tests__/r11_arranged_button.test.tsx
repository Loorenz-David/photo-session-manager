import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionList } from '@/services/sessions';

vi.mock('@/services/sessions', () => ({ listSessions: vi.fn(), arrangeNext: vi.fn() }));
vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => null), onExportProgress: vi.fn(async () => () => undefined), openExportFolder: vi.fn(), runExport: vi.fn() }));
vi.mock('@/features/publish/usePublish', () => ({ usePublish: () => ({ failedCount: 0, label: null, notifications: [], refresh: async () => undefined }) }));
vi.mock('@/features/publish/NotificationsDrawer', () => ({ NotificationsDrawer: () => null }));
vi.mock('@/features/publish/PublishProgress', () => ({ PublishProgress: () => null }));
vi.mock('@/features/settings/StorageSection', () => ({ DiskWarning: () => null }));
vi.mock('@/features/history/HistoryButtons', () => ({ HistoryButtons: () => null }));
vi.mock('@/features/shell/CameraDot', () => ({ CameraDot: () => null }));
vi.mock('@/features/shell/ThemeToggle', () => ({ ThemeToggle: () => null }));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }));

import { Header } from '../Header';
import { arrangeNext, listSessions } from '@/services/sessions';
import { toast } from 'sonner';

const onNewSession = vi.fn();

function show(id = 5, arranged = false, archived = false) {
  vi.mocked(listSessions).mockResolvedValue({ groups: [{ label: 'Today', rows: [{ id, sku: 'SKU', item_type: 'Chair', title: 'Chair', arranged, label: { kind: archived ? 'archived' : 'ready' } }] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 0 } as SessionList);
  return render(<Header sessionId={id} theme="light" onThemeChange={() => undefined} onNewSession={onNewSession} />);
}

beforeEach(() => { window.location.hash = '#/session/5'; });
afterEach(() => { cleanup(); vi.clearAllMocks(); });

// A19 (owner 2026-11-18): arranging goes back to the Sessions page with the New session field ready, never to the
// next unarranged session.
it('a19_c24a_arrange_goes_home_with_the_new_session_field_ready', async () => {
  vi.mocked(arrangeNext).mockResolvedValue({ next_session_id: 7, research_queued: true });
  show();
  fireEvent.click(await screen.findByRole('button', { name: 'Arranged — next' }));
  await waitFor(() => expect(arrangeNext).toHaveBeenCalledExactlyOnceWith(5));
  await waitFor(() => expect(window.location.hash).toBe('#/'));
  expect(onNewSession).toHaveBeenCalledTimes(1);
  expect(toast.success).toHaveBeenCalledWith('Arranged · AI research queued');
});

it('a19_c24b_without_research_queued_it_still_goes_home', async () => {
  vi.mocked(arrangeNext).mockResolvedValue({ next_session_id: null, research_queued: false });
  show(); fireEvent.click(await screen.findByRole('button', { name: 'Arranged — next' }));
  await waitFor(() => expect(window.location.hash).toBe('#/'));
  expect(onNewSession).toHaveBeenCalledTimes(1);
  expect(toast.success).toHaveBeenCalledWith('Arranged');
});

it('a25_c24c_marked_session_button_says_arranged', async () => {
  show(5, true);
  expect(await screen.findByRole('button', { name: 'Arranged ✓ — next' })).toBeInTheDocument();
});

it('a25_c24d_unassigned_and_archived_have_no_button', async () => {
  show(1); expect(screen.queryByRole('button', { name: /Arranged/ })).not.toBeInTheDocument();
  cleanup(); show(5, false, true);
  await waitFor(() => expect(listSessions).toHaveBeenCalled());
  expect(screen.queryByRole('button', { name: /Arranged/ })).not.toBeInTheDocument();
});

it('a25_c24e_failure_shows_message_and_keeps_route', async () => {
  vi.mocked(arrangeNext).mockRejectedValue(new Error('Could not mark session'));
  show(); fireEvent.click(await screen.findByRole('button', { name: 'Arranged — next' }));
  await waitFor(() => expect(toast.error).toHaveBeenCalledWith('Could not mark session'));
  expect(window.location.hash).toBe('#/session/5');
});
