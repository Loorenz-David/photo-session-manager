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

import { toast } from 'sonner';
import { SessionsPage } from '@/features/sessions/SessionsPage';
import { labelsList, labelsOpen, labelsSummary } from '@/services/labels';
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

/** Renders the Sessions page and goes into the Labels box; waits for its first row when `rowsShown`. */
async function enterBox(navigate: (id: number) => void = () => undefined, rowsShown = true) {
  render(<SessionsPage navigate={navigate} />);
  await screen.findByTestId('session-row-90');
  fireEvent.click(screen.getByTestId('filter-labels'));
  if (rowsShown) await screen.findByTestId('label-row-1');
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

/** L2·U3 (`plans/L2_labels_box.md` L2·C11): the Labels rows, with and without a session on this Mac. */

const NOV_4 = new Date(new Date().getFullYear(), 10, 4, 9, 12).getTime();

function withSession(): LabelRow {
  const session: SessionRowResponse_Serialize = {
    id: 7, sku: 'LOC-7', item_type: 'Chair', title: 'Oak chair', main_count: 2, condition_count: 0, unreviewed_count: 0, status: 'in_progress',
    label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: Date.now(), created_at: Date.now(), cover_preview_path: '/p/a.jpg',
    cover_pending: false, arranged: true, publish: null, changes: null,
  };
  return product(1, { session, price_minor: 890000, marker: { status: 'printed', printed_at: NOV_4, has_price: true, reset: null } });
}

function unmatched(): LabelRow {
  return product(3, { title: 'Test table', sku: 'TEST-1802', commerce_id: 'gid://shopify/Product/3', picture_path: null, marker: MISSING_TAG });
}

it('l2_c11a_a_row_with_a_session_looks_and_opens_like_it', async () => {
  vi.mocked(labelsList).mockResolvedValue(standard({ rows: [withSession()] }));
  const navigate = vi.fn();
  await enterBox(navigate);
  const row = screen.getByTestId('label-row-1');
  expect(within(row).getByText('Chair')).toBeInTheDocument();
  expect(within(row).getByText('LOC-7')).toBeInTheDocument();
  expect(row.querySelector('.row-thumb img')).toHaveAttribute('src', 'asset:///p/a.jpg');
  expect(within(row).getByText('Arranged')).toBeInTheDocument();
  expect(within(row).getByTestId('label-pill')).toHaveTextContent('Label printed · 4 Nov');
  expect(row.querySelector('.row-price')?.textContent).toBe('8 900 kr');
  expect(row.querySelector('.row-time')?.textContent).not.toBe('');
  fireEvent.click(row.querySelector('.row-main')!);
  expect(navigate.mock.calls).toEqual([[7]]);
  expect(labelsOpen).not.toHaveBeenCalled();
});

it('l2_c11b_a_row_without_a_session_shows_pricings_title_and_its_picture_when_it_lands', async () => {
  vi.mocked(labelsList).mockResolvedValue(standard({ rows: [product(1), unmatched()] }));
  await enterBox();
  const row = screen.getByTestId('label-row-3');
  expect(within(row).getByText('Test table')).toBeInTheDocument();
  expect(within(row).getByText('TEST-1802')).toBeInTheDocument();
  expect(within(row).getByTestId('thumb-placeholder')).toBeInTheDocument();
  expect(row.querySelector('img')).toBeNull();
  expect(row.querySelector('.row-label')?.textContent).toBe('Missing price tag');
  vi.mocked(labelsList).mockClear();
  await act(async () => delivered.pictures.forEach(handler => handler({ kind: 'search', external_id: 'gid://shopify/Product/3', path: '/p/s.jpg' })));
  await waitFor(() => expect(screen.getByTestId('label-row-3').querySelector('.row-thumb img')).toHaveAttribute('src', 'asset:///p/s.jpg'));
  expect(labelsList).not.toHaveBeenCalled();
});

it('l2_c11c_opening_a_row_without_a_session_adopts_then_navigates_or_says_why_not', async () => {
  vi.mocked(labelsList).mockResolvedValue(standard({ rows: [product(1), unmatched()] }));
  vi.mocked(labelsOpen).mockResolvedValue({ session_id: 31 });
  const navigate = vi.fn();
  await enterBox(navigate);
  fireEvent.click(screen.getByTestId('label-row-3').querySelector('.row-main')!);
  await waitFor(() => expect(navigate.mock.calls).toEqual([[31]]));
  expect(vi.mocked(labelsOpen).mock.calls).toEqual([[{ product_id: '3' }]]);
  cleanup();
  vi.mocked(labelsOpen).mockRejectedValue(failure('sku_collision', 'M'));
  const stay = vi.fn();
  await enterBox(stay);
  fireEvent.click(screen.getByTestId('label-row-3').querySelector('.row-main')!);
  await waitFor(() => expect(vi.mocked(toast.error).mock.calls).toEqual([['M']]));
  expect(stay).not.toHaveBeenCalled();
});

it('l2_c11d_no_price_no_created_and_no_tick_without_a_sku', async () => {
  vi.mocked(labelsList).mockResolvedValue(standard({ rows: [product(1, { price_minor: null, created_at: null, sku: null })] }));
  await enterBox();
  const row = screen.getByTestId('label-row-1');
  expect(row.querySelector('.row-price')?.textContent).toBe('No price');
  expect(row.querySelector('.row-time')?.textContent).toBe('');
  expect(within(row).queryByRole('checkbox')).not.toBeInTheDocument();
});
