import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import type { SessionList } from '@/services/sessions';
import type { SessionResearch } from '@/services/research';

vi.mock('@/services/sessions', () => ({ listSessions: vi.fn(), arrangeNext: vi.fn() }));
vi.mock('@/services/research', () => ({ researchOverview: vi.fn() }));
vi.mock('@/services/events', () => ({ onSessionsChanged: vi.fn(async () => () => undefined) }));
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
import { listSessions } from '@/services/sessions';
import { researchOverview } from '@/services/research';

function show(research: Partial<SessionResearch>) {
  vi.mocked(listSessions).mockResolvedValue({ groups: [{ label: 'Today', rows: [{ id: 5, sku: 'SKU', item_type: 'Chair', title: 'Chair', arranged: true, label: { kind: 'ready' } }] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 0 } as SessionList);
  vi.mocked(researchOverview).mockResolvedValue([{ session_id: 5, state: 'done', position: null, to_review: 0, reason: null, ...research } as SessionResearch]);
  const onDetails = vi.fn();
  const onReview = vi.fn();
  render(<Header sessionId={5} theme="light" onThemeChange={() => undefined} onNewSession={() => undefined} onDetails={onDetails} onReview={onReview} />);
  return { onDetails, onReview };
}

afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('a11_4_the_header_ai_to_review_pill_opens_the_ai_review_tab', async () => {
  const { onDetails, onReview } = show({ state: 'to_review', to_review: 2 });
  fireEvent.click(await screen.findByRole('button', { name: 'AI · 2 to review' }));
  expect(onReview).toHaveBeenCalledTimes(1);
  expect(onDetails).not.toHaveBeenCalled();
});

it('a11_4_any_other_research_state_opens_details_as_before', async () => {
  const { onDetails, onReview } = show({ state: 'done' });
  fireEvent.click(await screen.findByRole('button', { name: 'AI done' }));
  expect(onDetails).toHaveBeenCalledTimes(1);
  expect(onReview).not.toHaveBeenCalled();
});
