import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionRowResponse_Serialize } from '@/services/bindings';
import type { PictureReady } from '@/services/events';
import type { SessionList, SessionPage } from '@/services/sessions';
import type { SessionResearch } from '@/services/research';

/** H2·U3 (`plans/H2_list_pictures_pages_search.md`, rows H2·C9, C10): the Sessions page on `sessionsPage`. */

const delivered = vi.hoisted(() => ({ changed: [] as (() => void)[], pictures: [] as ((event: PictureReady) => void)[] }));

vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
  sessionsPage: vi.fn(),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/research', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/research')>()), researchOverview: vi.fn() }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async (refresh: () => void) => {
    delivered.changed.push(refresh);
    return () => undefined;
  }),
  onPictureReady: vi.fn(async (handler: (event: PictureReady) => void) => {
    delivered.pictures.push(handler);
    return () => undefined;
  }),
}));
vi.mock('@/services/conversations', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/conversations')>()),
  conversationsSessionBadges: vi.fn(async () => []),
}));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }));

import { SessionsPage } from '@/features/sessions/SessionsPage';
import { SessionsPanel } from '@/features/workspace/SessionsPanel';
import { listSessions, previewUrl, sessionsPage } from '@/services/sessions';
import { researchOverview } from '@/services/research';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

function row(id: number, cover: string | null = null, pending = false): SessionRowResponse_Serialize {
  return {
    id, sku: `SKU-${id}`, item_type: '', title: `Item ${id}`, main_count: 2, condition_count: 1, unreviewed_count: 0, status: 'in_progress',
    label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: Date.now(), created_at: Date.now(), cover_preview_path: cover,
    cover_pending: pending, arranged: false, publish: null, changes: null,
  };
}

function answer(rows: SessionRowResponse_Serialize[], page = 1, pages = 1, total = rows.length, groupTotal = rows.length): SessionPage {
  return { groups: [{ label: 'Today', total: groupTotal, rows }], counts: { open: total, published: 0, all: total }, unassigned_count: 0, page, pages, total };
}

const twentyFive = Array.from({ length: 25 }, (_, i) => row(100 - i));

/** The page-control button of page `n`. */
function pageButton(n: number) {
  return screen.getByRole('button', { name: `Page ${n}` });
}

function calls() {
  return vi.mocked(sessionsPage).mock.calls.map(([args]) => args);
}

/** Renders the page answering page 1 of 3, then goes to page 2 and forgets the calls so far. */
async function onPageTwo() {
  vi.mocked(sessionsPage).mockImplementation(async args => answer(twentyFive, args.page, 3, 60));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-100');
  fireEvent.click(pageButton(2));
  await waitFor(() => expect(pageButton(2)).toHaveAttribute('aria-current', 'page'));
  vi.mocked(sessionsPage).mockClear();
}

function rowImage(id: number) {
  return screen.getByTestId(`session-row-${id}`).querySelector('.row-thumb img');
}

beforeEach(() => {
  delivered.changed.length = 0;
  delivered.pictures.length = 0;
  vi.mocked(researchOverview).mockResolvedValue([]);
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('h2_c9a_rows_and_page_numbers', async () => {
  vi.mocked(sessionsPage).mockResolvedValue(answer(twentyFive, 1, 3, 60));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-100');
  expect(document.querySelectorAll('[data-testid^="session-row-"]')).toHaveLength(25);
  const control = screen.getByRole('navigation', { name: 'Pages' });
  const numbers = within(control).getAllByRole('button', { name: /^Page \d$/ });
  expect(numbers.map(b => [b.textContent, b.getAttribute('aria-current')])).toEqual([['1', 'page'], ['2', null], ['3', null]]);
  cleanup();
  vi.mocked(sessionsPage).mockResolvedValue(answer([row(1)], 1, 1));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-1');
  expect(screen.queryByRole('navigation', { name: 'Pages' })).not.toBeInTheDocument();
});

it('h2_c9b_a_page_number_asks_for_that_page', async () => {
  vi.mocked(sessionsPage).mockImplementation(async args => answer(twentyFive, args.page, 3, 60));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-100');
  fireEvent.click(pageButton(2));
  await waitFor(() => expect(calls().at(-1)).toEqual({ filter: 'open', query: null, created_from: null, only_ids: null, page: 2 }));
});

it('h2_c9c_a_filter_or_a_search_goes_to_page_one', async () => {
  await onPageTwo();
  fireEvent.click(screen.getByTestId('filter-published'));
  await waitFor(() => expect(calls()).toHaveLength(1));
  expect(calls()).toEqual([{ filter: 'published', query: null, created_from: null, only_ids: null, page: 1 }]);

  fireEvent.click(pageButton(2));
  await waitFor(() => expect(pageButton(2)).toHaveAttribute('aria-current', 'page'));
  vi.mocked(sessionsPage).mockClear();
  fireEvent.change(screen.getByLabelText('Search sessions by title or SKU'), { target: { value: 'oak' } });
  await waitFor(() => expect(calls()).toHaveLength(1));
  await new Promise(resolve => setTimeout(resolve, 300));
  expect(calls()).toEqual([{ filter: 'published', query: 'oak', created_from: null, only_ids: null, page: 1 }]);
});

it('h2_c9d_the_time_filter_and_to_review_go_to_page_one', async () => {
  vi.mocked(researchOverview).mockResolvedValue([
    { session_id: 9, state: 'to_review', position: null, to_review: 2, reason: null } as SessionResearch,
    { session_id: 4, state: 'to_review', position: null, to_review: 1, reason: null } as SessionResearch,
  ]);
  await onPageTwo();
  fireEvent.click(screen.getByRole('button', { name: 'Filter by creation time' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Last 7 days' }));
  await waitFor(() => expect(calls()).toHaveLength(1));
  const start = new Date();
  start.setHours(0, 0, 0, 0);
  start.setDate(start.getDate() - 6);
  expect(calls()).toEqual([{ filter: 'open', query: null, created_from: start.getTime(), only_ids: null, page: 1 }]);

  fireEvent.click(pageButton(2));
  await waitFor(() => expect(pageButton(2)).toHaveAttribute('aria-current', 'page'));
  vi.mocked(sessionsPage).mockClear();
  fireEvent.click(screen.getByRole('button', { name: /To review · 2/ }));
  await waitFor(() => expect(calls()).toHaveLength(1));
  expect(calls()).toEqual([{ filter: 'open', query: null, created_from: start.getTime(), only_ids: [4, 9], page: 1 }]);
});

it('h2_c9e_sessions_changed_asks_for_the_page_shown', async () => {
  await onPageTwo();
  act(() => delivered.changed.forEach(refresh => refresh()));
  await waitFor(() => expect(calls()).toHaveLength(1));
  expect(calls()[0].page).toBe(2);
});

it('h2_c9f_a_clamped_answer_is_the_current_page', async () => {
  vi.mocked(sessionsPage).mockImplementation(async args => (args.page === 3 ? answer(twentyFive.slice(0, 5), 2, 2, 30) : answer(twentyFive, args.page, 3, 60)));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-100');
  fireEvent.click(pageButton(3));
  await waitFor(() => expect(pageButton(2)).toHaveAttribute('aria-current', 'page'));
  expect(screen.queryByRole('button', { name: 'Page 3' })).not.toBeInTheDocument();
});

it('h2_c9g_a_group_header_shows_its_total', async () => {
  vi.mocked(sessionsPage).mockResolvedValue(answer(twentyFive.slice(0, 5), 2, 2, 30, 30));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-100');
  expect(screen.getByRole('heading', { level: 2 })).toHaveTextContent('Today 30');
});

it('h2_c10a_a_row_shows_its_picture', async () => {
  vi.mocked(sessionsPage).mockResolvedValue(answer([row(1, '/p/a.jpg')]));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-1');
  expect(rowImage(1)).toHaveAttribute('src', previewUrl('/p/a.jpg'));
});

it('h2_c10b_a_row_without_a_picture_shows_the_placeholder', async () => {
  vi.mocked(sessionsPage).mockResolvedValue(answer([row(1, null, true), row(2, null, false)]));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-1');
  for (const id of [1, 2]) {
    const thumb = screen.getByTestId(`session-row-${id}`).querySelector('.row-thumb')!;
    expect(thumb).toHaveClass('row-thumb-empty');
    expect(thumb.querySelector('img')).toBeNull();
  }
});

it('h2_c10c_a_picture_ready_cover_updates_its_row_without_a_reload', async () => {
  vi.mocked(sessionsPage).mockResolvedValue(answer([row(1, null, true)]));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-1');
  await waitFor(() => expect(delivered.pictures.length).toBeGreaterThan(0));
  vi.mocked(sessionsPage).mockClear();
  act(() => delivered.pictures.forEach(handler => handler({ kind: 'cover', session_id: 1, path: '/p/c.jpg' })));
  await waitFor(() => expect(rowImage(1)).toHaveAttribute('src', previewUrl('/p/c.jpg')));
  expect(sessionsPage).not.toHaveBeenCalled();
});

it('h2_c10d_the_hover_card_shows_the_rows_picture', async () => {
  vi.mocked(sessionsPage).mockResolvedValue(answer([row(1, '/p/a.jpg'), row(2, null, true)]));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-1');
  await waitFor(() => expect(delivered.pictures.length).toBeGreaterThan(0));
  act(() => delivered.pictures.forEach(handler => handler({ kind: 'cover', session_id: 2, path: '/p/c.jpg' })));
  await waitFor(() => expect(rowImage(2)).toHaveAttribute('src', previewUrl('/p/c.jpg')));
  for (const [id, path] of [[1, '/p/a.jpg'], [2, '/p/c.jpg']] as const) {
    fireEvent.pointerEnter(screen.getByTestId(`session-row-${id}`).querySelector('.row-main')!);
    const card = await waitFor(() => {
      const found = document.querySelector('.hover-preview .hover-image img');
      expect(found).not.toBeNull();
      return found!;
    });
    expect(card).toHaveAttribute('src', previewUrl(path));
    fireEvent.pointerLeave(screen.getByTestId(`session-row-${id}`).querySelector('.row-main')!);
    await waitFor(() => expect(document.querySelector('.hover-preview')).toBeNull());
  }
});

it('h2_c10e_the_workspace_sessions_panel_shows_pictures_or_the_placeholder', async () => {
  const list: SessionList = { groups: [{ label: 'Today', rows: [row(1, '/p/a.jpg'), row(2)] }], counts: { open: 2, published: 0, all: 2 }, unassigned_count: 0 };
  vi.mocked(listSessions).mockResolvedValue(list);
  const { container } = render(<SessionsPanel currentSessionId={99} onClose={() => undefined} />);
  await waitFor(() => expect(container.querySelectorAll('.sessions-panel-row')).toHaveLength(2));
  const [withPicture, without] = [...container.querySelectorAll('.sessions-panel-row')];
  expect(withPicture.querySelector('.row-thumb img')).toHaveAttribute('src', previewUrl('/p/a.jpg'));
  expect(without.querySelector('.row-thumb')).toHaveClass('row-thumb-empty');
  expect(without.querySelector('img')).toBeNull();
});
