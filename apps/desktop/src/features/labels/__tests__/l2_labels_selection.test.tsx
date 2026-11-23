import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
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
import { labelsList, labelsSelectAll, labelsSummary } from '@/services/labels';
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

/** Renders the Sessions page and goes into the Labels box; waits for its first row when `rowsShown`. */
async function enterBox(navigate: (id: number) => void = () => undefined, rowsShown = true) {
  render(<SessionsPage navigate={navigate} />);
  await screen.findByTestId('session-row-90');
  fireEvent.click(screen.getByTestId('filter-labels'));
  if (rowsShown) await screen.findByTestId('label-row-1');
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

/** L2·U3 (`plans/L2_labels_box.md` L2·C13): the Labels box's selection and its bar (MC-L1.9). */

function bar() {
  return screen.queryByRole('toolbar', { name: 'Selected products' });
}

async function tickTwo() {
  await screen.findByTestId('label-row-1');
  fireEvent.click(checkbox('1'));
  fireEvent.click(checkbox('2'));
  await waitFor(() => expect(bar()).toHaveTextContent('2 selected'));
}

// Replaced by L3 (L3·C17(d)): Generate labels for N is now enabled.
it('l2_c13a_ticking_rows_shows_the_bar_with_generate', async () => {
  await enterBox();
  await tickTwo();
  expect(within(bar()!).getByRole('button', { name: /Generate labels for 2/ })).toBeEnabled();
  expect(within(bar()!).queryByRole('button', { name: /Review & publish/ })).not.toBeInTheDocument();
});

it('l2_c13b_the_header_box_ticks_the_rows_with_a_sku_and_offers_select_all', async () => {
  vi.mocked(labelsList).mockResolvedValue(standard({ rows: [...rows().slice(0, 24), product(25, { sku: null })] }));
  await enterBox();
  fireEvent.click(screen.getByRole('checkbox', { name: 'Select the products of this page' }));
  await waitFor(() => expect(bar()).toHaveTextContent('24 selected'));
  const ticked = [...document.querySelectorAll('[data-testid^="label-row-"] [role="checkbox"]')].filter(b => b.getAttribute('data-state') === 'checked');
  expect(ticked).toHaveLength(24);
  expect(within(bar()!).getByRole('button', { name: 'Select all 60' })).toBeInTheDocument();
});

it('l2_c13c_select_all_takes_exactly_what_pricing_answers', async () => {
  vi.mocked(labelsSelectAll).mockResolvedValue(Array.from({ length: 58 }, (_, i) => ({ product_id: String(i + 1), sku: `TEST-${i + 1}` })));
  await enterBox();
  fireEvent.click(screen.getByRole('checkbox', { name: 'Select the products of this page' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Select all 60' }));
  await waitFor(() => expect(bar()).toHaveTextContent('All 58 selected'));
  expect(vi.mocked(labelsSelectAll).mock.calls).toEqual([[{ filter: 'missing_price_tag', query: null }]]);
});

it('l2_c13d_the_selection_survives_page_changes', async () => {
  await enterBox();
  fireEvent.click(checkbox('1'));
  await waitFor(() => expect(bar()).toHaveTextContent('1 selected'));
  fireEvent.click(screen.getByRole('button', { name: 'Next' }));
  await screen.findByTestId('label-row-26');
  expect(bar()).toHaveTextContent('1 selected');
  fireEvent.click(screen.getByRole('button', { name: 'Previous' }));
  await screen.findByTestId('label-row-1');
  expect(checkbox('1')).toBeChecked();
  expect(bar()).toHaveTextContent('1 selected');
});

it('l2_c13e_a_filter_a_search_leaving_the_box_and_escape_clear_the_selection', async () => {
  await enterBox();
  await tickTwo();
  fireEvent.click(screen.getByTestId('label-filter-printed'));
  await waitFor(() => expect(bar()).not.toBeInTheDocument());

  await tickTwo();
  fireEvent.change(screen.getByLabelText('Search labels by SKU, title or type'), { target: { value: 'oak' } });
  await new Promise(resolve => setTimeout(resolve, 300));
  await waitFor(() => expect(bar()).not.toBeInTheDocument());

  await tickTwo();
  fireEvent.click(screen.getByTestId('filter-open'));
  await screen.findByTestId('session-row-90');
  fireEvent.click(screen.getByTestId('filter-labels'));
  await screen.findByTestId('label-row-1');
  expect(bar()).not.toBeInTheDocument();

  await tickTwo();
  fireEvent.keyDown(window, { key: 'Escape' });
  await waitFor(() => expect(bar()).not.toBeInTheDocument());
});
