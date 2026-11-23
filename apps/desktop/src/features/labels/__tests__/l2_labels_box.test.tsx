import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionRowResponse_Serialize } from '@/services/bindings';
import type { LabelsChanged, PictureReady } from '@/services/events';
import type { LabelPage, LabelRow } from '@/services/labels';
import type { SessionPage } from '@/services/sessions';

const delivered = vi.hoisted(() => ({
  labels: [] as ((event: LabelsChanged) => void)[],
  pictures: [] as ((event: PictureReady) => void)[],
}));

vi.mock('@/services/labels', () => ({
  labelsStatus: vi.fn(async () => ({ access: { state: 'allowed', reason: null }, labels: [] })),
  labelsSummary: vi.fn(async () => ({ missing_price_tag: null })),
  labelsList: vi.fn(),
  labelsSelectAll: vi.fn(),
  labelsOpen: vi.fn(),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  sessionsPage: vi.fn(),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/research', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/research')>()), researchOverview: vi.fn(async () => []) }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async () => () => undefined),
  onPictureReady: vi.fn(async (handler: (event: PictureReady) => void) => {
    delivered.pictures.push(handler);
    return () => undefined;
  }),
  onLabelsChanged: vi.fn(async (handler: (event: LabelsChanged) => void) => {
    delivered.labels.push(handler);
    return () => undefined;
  }),
  onConversationsChanged: vi.fn(async () => () => undefined),
}));
vi.mock('@/services/conversations', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/conversations')>()),
  conversationsSessionBadges: vi.fn(async () => []),
  conversationsSettingsGet: vi.fn(async () => ({
    server: 'https://pricing.example', server_default: 'https://pricing.example', key_saved: false, sound: false, banner: false,
    connection: { state: 'no_key', store_name: null, shop: null, visible_stores: [] }, labels: 'off',
  })),
}));
vi.mock('@/features/settings/SettingsNav', async importOriginal => ({
  ...(await importOriginal<typeof import('@/features/settings/SettingsNav')>()),
  openSettingsSection: vi.fn(),
}));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }));

import { SessionsPage } from '@/features/sessions/SessionsPage';
import { openSettingsSection } from '@/features/settings/SettingsNav';
import { labelsList, labelsSummary } from '@/services/labels';
import { sessionsPage } from '@/services/sessions';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

const COUNTS = { total: 214, has_price: 180, missing_price: 34, missing_price_tag: 60, printed: 168, not_printed: 46 };
const MISSING_TAG = { status: 'not_printed', printed_at: null, has_price: true, reset: null } as const;

function product(n: number, change: Partial<LabelRow> = {}): LabelRow {
  return {
    product_id: String(n), commerce_id: `gid://shopify/Product/${n}`, sku: `TEST-${n}`, title: `Test table ${n}`, product_type: 'Test tables',
    price_minor: 890000, marker: MISSING_TAG, created_at: Date.now() - 60_000, session: null, picture_path: null, ...change,
  };
}

/** A page of 25 rows starting at product `first`. */
function rows(first = 1): LabelRow[] {
  return Array.from({ length: 25 }, (_, i) => product(first + i));
}

/** The standard answer: 25 rows, `next_cursor: "c1"`, the standard counts, `work_from: "2026-11-19"`. */
function standard(change: Partial<LabelPage> = {}): LabelPage {
  return { rows: rows(), next_cursor: 'c1', counts: COUNTS, work_from: '2026-11-19', ...change };
}

function sessionAnswer(): SessionPage {
  const row: SessionRowResponse_Serialize = {
    id: 90, sku: 'SKU-90', item_type: '', title: 'Item 90', main_count: 2, condition_count: 1, unreviewed_count: 0, status: 'in_progress',
    label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: Date.now(), created_at: Date.now(), cover_preview_path: null,
    cover_pending: false, arranged: false, publish: null, changes: null,
  };
  return { groups: [{ label: 'Today', total: 1, rows: [row] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 3, page: 1, pages: 1, total: 1 };
}

function failure(code: string, message: string, data: Record<string, string> = {}) {
  return Object.assign(new Error(message), { code, data });
}

function listCalls() {
  return vi.mocked(labelsList).mock.calls.map(([args]) => args);
}

/** Renders the Sessions page and goes into the Labels box; waits for its first row when `rowsShown`. */
async function enterBox(navigate: (id: number) => void = () => undefined, rowsShown = true) {
  render(<SessionsPage navigate={navigate} />);
  await screen.findByTestId('session-row-90');
  fireEvent.click(screen.getByTestId('filter-labels'));
  if (rowsShown) await screen.findByTestId('label-row-1');
}

async function labelsChanged(event: LabelsChanged = { product_ids: ['1'], access: false }) {
  await act(async () => delivered.labels.forEach(handler => handler(event)));
}

function checkbox(productId: string) {
  return within(screen.getByTestId(`label-row-${productId}`)).getByRole('checkbox');
}

beforeEach(() => {
  delivered.labels.length = 0;
  delivered.pictures.length = 0;
  vi.mocked(sessionsPage).mockImplementation(async () => sessionAnswer());
  vi.mocked(labelsList).mockImplementation(async args => (args.cursor === 'c1' ? standard({ rows: rows(26), next_cursor: 'c2' }) : standard()));
  vi.mocked(labelsSummary).mockImplementation(async () => ({ missing_price_tag: null }));
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.clearAllMocks();
});

/**
 * L2·U3 (`plans/L2_labels_box.md` L2·C10, C12, C14): the Labels box on the Sessions page: its number, the toolbar, the
 * off and error bodies, Previous / Next, and the refresh, with the services and events mocked.
 */

it('l2_c10a_the_labels_box_shows_its_number_from_one_summary', async () => {
  vi.mocked(labelsSummary).mockResolvedValue({ missing_price_tag: 12 });
  render(<SessionsPage navigate={() => undefined} />);
  await waitFor(() => expect(screen.getByTestId('count-labels').textContent).toBe('12'));
  const boxes = [...document.querySelectorAll('[data-testid^="filter-"]')].map(b => b.getAttribute('data-testid'));
  expect(boxes).toEqual(['filter-open', 'filter-published', 'filter-all', 'filter-labels']);
  expect(screen.getByTestId('filter-labels')).toHaveTextContent('Labels');
  expect(vi.mocked(labelsSummary).mock.calls).toEqual([[{ refresh: true }]]);
  expect(labelsList).not.toHaveBeenCalled();
  cleanup();
  vi.mocked(labelsSummary).mockResolvedValue({ missing_price_tag: null });
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-90');
  await waitFor(() => expect(labelsSummary).toHaveBeenCalledTimes(2));
  expect(screen.getByTestId('count-labels').textContent).toBe('');
});

it('l2_c10b_entering_the_box_asks_the_first_page_and_shows_its_toolbar', async () => {
  await enterBox();
  expect(listCalls()).toEqual([{ filter: 'missing_price_tag', query: null, cursor: null }]);
  const subFilters = [['missing_price_tag', 'Missing price tag60'], ['not_printed', 'Not printed46'], ['printed', 'Printed168'], ['all', 'All214']];
  for (const [value, text] of subFilters) expect(screen.getByTestId(`label-filter-${value}`)).toHaveTextContent(text);
  expect(screen.getByTestId('label-filter-missing_price_tag')).toHaveAttribute('data-state', 'on');
  const header = within(screen.getByRole('table', { name: 'Labels' })).getAllByRole('row')[0];
  expect(header.textContent?.trim()).toBe('ItemPriceLabelCreated');
  expect(screen.queryByRole('button', { name: 'Filter by creation time' })).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: /To review/ })).not.toBeInTheDocument();
  expect(screen.queryByTestId('unassigned-card')).not.toBeInTheDocument();
  expect(document.querySelector('.group-header')).toBeNull();
  expect(screen.getByTestId('count-labels')).toHaveTextContent('60');
});

it('l2_c10c_a_sub_filter_and_the_search_ask_through_pricing', async () => {
  await enterBox();
  fireEvent.click(screen.getByTestId('label-filter-printed'));
  await waitFor(() => expect(listCalls()).toHaveLength(2));
  expect(listCalls()[1]).toEqual({ filter: 'printed', query: null, cursor: null });
  fireEvent.change(screen.getByLabelText('Search labels by SKU, title or type'), { target: { value: 'oak' } });
  await new Promise(resolve => setTimeout(resolve, 300));
  await waitFor(() => expect(listCalls()).toHaveLength(3));
  expect(listCalls()[2]).toEqual({ filter: 'printed', query: 'oak', cursor: null });
});

it('l2_c10d_the_work_from_line', async () => {
  await enterBox();
  expect(screen.getByText('Products created since 19 Nov 2026')).toBeInTheDocument();
  cleanup();
  vi.mocked(labelsList).mockResolvedValue(standard({ work_from: null }));
  await enterBox();
  expect(screen.queryByText(/Products created since/)).not.toBeInTheDocument();
});

it('l2_c10e_off_and_denied_bodies_point_to_settings', async () => {
  vi.mocked(labelsList).mockRejectedValue(failure('labels_off', '', { reason: 'no_key' }));
  await enterBox(undefined, false);
  expect(await screen.findByText('Connect Pricing in Settings → Pricing')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Open Settings → Pricing' }));
  expect(vi.mocked(openSettingsSection).mock.calls).toEqual([['pricing']]);
  expect(document.querySelector('[data-testid^="label-row-"]')).toBeNull();
  cleanup();
  vi.mocked(openSettingsSection).mockClear();
  vi.mocked(labelsList).mockRejectedValue(failure('labels_denied', "Session Manager isn't allowed to print labels. Ask a Pricing admin."));
  await enterBox(undefined, false);
  expect(await screen.findByText("This key can't read labels. Ask a Pricing admin to add labels:read and labels:write.")).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Open Settings → Pricing' }));
  expect(vi.mocked(openSettingsSection).mock.calls).toEqual([['pricing']]);
  expect(document.querySelector('[data-testid^="label-row-"]')).toBeNull();
});

it('l2_c10f_a_failed_refresh_keeps_the_rows_and_offers_try_again', async () => {
  await enterBox();
  vi.mocked(labelsList).mockClear();
  vi.mocked(labelsList).mockRejectedValueOnce(failure('labels_unreachable', "Pricing can't be reached right now."));
  await labelsChanged();
  expect(await screen.findByText("Pricing can't be reached right now.")).toBeInTheDocument();
  expect(document.querySelectorAll('[data-testid^="label-row-"]')).toHaveLength(25);
  vi.mocked(labelsList).mockClear();
  fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
  await waitFor(() => expect(listCalls()).toEqual([{ filter: 'missing_price_tag', query: null, cursor: null }]));
});

it('l2_c10g_an_unavailable_list_says_so_with_try_again', async () => {
  vi.mocked(labelsList).mockRejectedValue(failure('labels_list_unavailable', "The product list isn't available right now."));
  await enterBox(undefined, false);
  expect(await screen.findByText("The product list isn't available right now.")).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument();
});

it('l2_c12a_previous_next_and_the_page_line', async () => {
  await enterBox();
  expect(screen.getByText('Page 1 of 3')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Next' })).toBeEnabled();
  expect(screen.getByRole('button', { name: 'Previous' })).toBeDisabled();
  expect(screen.queryByRole('button', { name: /^Page \d+$/ })).not.toBeInTheDocument();
});

it('l2_c12b_next_asks_the_seen_cursor_and_previous_the_one_before', async () => {
  await enterBox();
  fireEvent.click(screen.getByRole('button', { name: 'Next' }));
  await screen.findByTestId('label-row-26');
  expect(listCalls()[1]).toEqual({ filter: 'missing_price_tag', query: null, cursor: 'c1' });
  expect(screen.getByText('Page 2 of 3')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Previous' }));
  await screen.findByTestId('label-row-1');
  expect(listCalls()[2]).toEqual({ filter: 'missing_price_tag', query: null, cursor: null });
  expect(screen.getByText('Page 1 of 3')).toBeInTheDocument();
});

it('l2_c12c_a_sub_filter_goes_to_the_first_page', async () => {
  await enterBox();
  fireEvent.click(screen.getByRole('button', { name: 'Next' }));
  await screen.findByTestId('label-row-26');
  fireEvent.click(screen.getByTestId('label-filter-not_printed'));
  await waitFor(() => expect(listCalls()).toHaveLength(3));
  expect(listCalls()[2]).toEqual({ filter: 'not_printed', query: null, cursor: null });
  await waitFor(() => expect(screen.getByText(/^Page 1 of /)).toBeInTheDocument());
});

it('l2_c14a_a_label_change_refreshes_the_page_shown_keeping_ticks_and_page', async () => {
  await enterBox();
  fireEvent.click(screen.getByRole('button', { name: 'Next' }));
  await screen.findByTestId('label-row-26');
  fireEvent.click(checkbox('26'));
  vi.mocked(labelsList).mockClear();
  await labelsChanged();
  await waitFor(() => expect(listCalls()).toEqual([{ filter: 'missing_price_tag', query: null, cursor: 'c1' }]));
  await act(async () => undefined);
  expect(checkbox('26')).toBeChecked();
  expect(screen.getByText('Page 2 of 3')).toBeInTheDocument();
});

it('l2_c14b_label_changes_refresh_at_most_once_per_two_seconds', async () => {
  await enterBox();
  vi.mocked(labelsList).mockClear();
  vi.useFakeTimers();
  await labelsChanged();
  await act(async () => { vi.advanceTimersByTime(500); });
  await labelsChanged();
  await act(async () => { vi.advanceTimersByTime(500); });
  await labelsChanged();
  expect(listCalls()).toHaveLength(1);
  await act(async () => { vi.advanceTimersByTime(999); });
  expect(listCalls()).toHaveLength(1);
  await act(async () => { vi.advanceTimersByTime(1); });
  expect(listCalls()).toHaveLength(2);
  await act(async () => { vi.advanceTimersByTime(5000); });
  expect(listCalls()).toHaveLength(2);
});

it('l2_c14c_outside_the_box_a_label_change_asks_only_the_number', async () => {
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-90');
  await waitFor(() => expect(labelsSummary).toHaveBeenCalledTimes(1));
  vi.mocked(labelsSummary).mockResolvedValue({ missing_price_tag: 11 });
  await labelsChanged();
  await waitFor(() => expect(screen.getByTestId('count-labels')).toHaveTextContent('11'));
  expect(vi.mocked(labelsSummary).mock.calls).toEqual([[{ refresh: true }], [{ refresh: true }]]);
  expect(labelsList).not.toHaveBeenCalled();
});

it('l2_c14d_entering_the_box_again_asks_the_first_page', async () => {
  await enterBox();
  fireEvent.click(screen.getByRole('button', { name: 'Next' }));
  await screen.findByTestId('label-row-26');
  fireEvent.click(screen.getByTestId('filter-open'));
  await screen.findByTestId('session-row-90');
  vi.mocked(labelsList).mockClear();
  fireEvent.click(screen.getByTestId('filter-labels'));
  await screen.findByTestId('label-row-1');
  expect(listCalls()).toEqual([{ filter: 'missing_price_tag', query: null, cursor: null }]);
});
