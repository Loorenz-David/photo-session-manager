import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionRowResponse_Serialize } from '@/services/bindings';
import type { ConversationsSettings } from '@/services/conversations';
import type { LabelCheck, LabelPage, LabelRow, LabelsAccess, SessionLabel } from '@/services/labels';
import type { SessionList, SessionPage } from '@/services/sessions';

/**
 * L3·U4 (`plans/L3_generate_and_download.md` L3·C17): the ⋯ entry on session rows, the session page and Labels rows
 * without a session, and the batch bar's Generate labels for N, opening the label dialog; the selection after a run
 * (09C §6, §6.5; MC-L1.9).
 */

vi.mock('@/services/labels', () => ({
  labelsStatus: vi.fn(),
  labelsSummary: vi.fn(async () => ({ missing_price_tag: null })),
  labelsList: vi.fn(),
  labelsSelectAll: vi.fn(),
  labelsOpen: vi.fn(),
  labelsCheck: vi.fn(),
  labelsPreview: vi.fn(),
  labelsGenerate: vi.fn(),
  labelsPartial: vi.fn(),
  labelsDownload: vi.fn(),
}));
vi.mock('@/services/conversations', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/conversations')>()),
  conversationsSettingsGet: vi.fn(),
  conversationsCounts: vi.fn(async () => ({ here: 0, elsewhere: 0 })),
  conversationsSessionBadges: vi.fn(async () => []),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
  sessionsPage: vi.fn(),
  sessionFilesMisplaced: vi.fn(async () => 0),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/research', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/research')>()), researchOverview: vi.fn(async () => []) }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async () => () => undefined),
  onLabelsChanged: vi.fn(async () => () => undefined),
  onConversationsChanged: vi.fn(async () => () => undefined),
  onPictureReady: vi.fn(async () => () => undefined),
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
vi.mock('@/features/settings/SettingsNav', async importOriginal => ({
  ...(await importOriginal<typeof import('@/features/settings/SettingsNav')>()),
  openSettingsSection: vi.fn(),
}));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }));

import { SessionActionsMenu } from '@/features/sessions/DeleteSession';
import { SessionsPage } from '@/features/sessions/SessionsPage';
import { Header } from '@/features/shell/Header';
import { conversationsSettingsGet } from '@/services/conversations';
import { labelsCheck, labelsGenerate, labelsList, labelsPreview, labelsStatus } from '@/services/labels';
import { listSessions, sessionsPage } from '@/services/sessions';

if (!('PointerEvent' in window)) {
  class PointerEventPolyfill extends MouseEvent {
    readonly pointerId: number;
    readonly pointerType: string;
    constructor(type: string, init: PointerEventInit = {}) {
      super(type, init);
      this.pointerId = init.pointerId ?? 1;
      this.pointerType = init.pointerType ?? 'mouse';
    }
  }
  Object.assign(window, { PointerEvent: PointerEventPolyfill });
}
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

const ALLOWED: LabelsAccess = { state: 'allowed', reason: null };
const UNKNOWN: LabelsAccess = { state: 'unknown', reason: null };
const PRINTED = { status: 'printed', printed_at: 1_793_782_800_000, has_price: true, reset: null } as const;
const MISSING_TAG = { status: 'not_printed', printed_at: null, has_price: true, reset: null } as const;

function label(productId: string | null, marker: SessionLabel['marker'] = null): SessionLabel {
  return { session_id: 2, product_id: productId, marker };
}

function checkOf(ids: string[]): LabelCheck {
  return {
    id: 'c',
    groups: [{
      template: { id: 't70', name: 'Label 70×100', width_mm: 70, height_mm: 100 }, product_types: [],
      items: ids.map(id => ({ product_id: id, sku: `TEST-${id}`, title: `Test table ${id}`, price_minor: 890_000, status: 'not_printed', printed_at: null, outcome: 'mark_printed', issues: [], notes: [] })),
    }],
    unresolved: [],
    summary: { items: ids.length, ready: ids.length, failing: 0, mark_printed: ids.length, reprint: 0, no_price: 0 },
    sizes: [],
  };
}

function storeGone(): ConversationsSettings {
  return {
    server: 'https://pricing.example', server_default: 'https://pricing.example', key_saved: true, sound: true, banner: true,
    connection: { state: 'store_gone', store_name: 'Dev store', shop: 'shop-a.myshopify.com', visible_stores: [], last_ok_at: null, last_error: null, last_error_at: null },
    labels: 'off',
  };
}

async function openMenu(name = 'Session actions') {
  fireEvent.pointerDown(screen.getByRole('button', { name }), { button: 0 });
  await screen.findByRole('menu');
}

/** The label item: its words and whether it is off. */
function labelItem(): { disabled: boolean; text: string } | null {
  const item = screen.queryByRole('menuitem', { name: /(Generate|Reprint) label/ });
  if (!item) return null;
  return { disabled: item.getAttribute('aria-disabled') === 'true', text: item.textContent ?? '' };
}

async function menuOf(sessionLabel: SessionLabel, access: LabelsAccess) {
  render(<SessionActionsMenu sessionId={2} onDeleted={() => undefined} label={sessionLabel} labelsAccess={access} />);
  await openMenu();
}

beforeEach(() => {
  vi.mocked(conversationsSettingsGet).mockResolvedValue(storeGone());
  vi.mocked(labelsCheck).mockImplementation(async ({ product_ids }) => checkOf(product_ids));
  vi.mocked(labelsPreview).mockResolvedValue({ path: '/d/previews/labels/c-1.png' });
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('l3_c17a_the_session_menu_entry_generates_or_reprints', async () => {
  for (const access of [ALLOWED, UNKNOWN]) {
    await menuOf(label('1'), access);
    expect(labelItem()).toEqual({ disabled: false, text: 'Generate label' });
    cleanup();
  }
  await menuOf(label('1', PRINTED), ALLOWED);
  expect(labelItem()).toEqual({ disabled: false, text: 'Reprint label' });
  cleanup();

  await menuOf(label('1', MISSING_TAG), ALLOWED);
  fireEvent.click(screen.getByRole('menuitem', { name: 'Generate label' }));
  await screen.findByRole('dialog');
  await waitFor(() => expect(vi.mocked(labelsCheck).mock.calls).toEqual([[{ product_ids: ['1'] }]]));
});

it('l3_c17b_the_session_page_menu_opens_the_same_dialog', async () => {
  vi.mocked(listSessions).mockResolvedValue({ groups: [{ label: 'Today', rows: [{ id: 2, sku: 'SKU-2', item_type: 'Chair', title: 'Chair', arranged: false, label: { kind: 'ready' } }] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 0 } as SessionList);
  vi.mocked(labelsStatus).mockResolvedValue({ access: ALLOWED, labels: [label('1', MISSING_TAG)] });
  render(<Header sessionId={2} theme="light" onThemeChange={() => undefined} onNewSession={() => undefined} />);
  await waitFor(() => expect(labelsStatus).toHaveBeenCalled());
  await openMenu();
  await waitFor(() => expect(labelItem()).toEqual({ disabled: false, text: 'Generate label' }));
  fireEvent.click(screen.getByRole('menuitem', { name: 'Generate label' }));
  await screen.findByRole('dialog');
  await waitFor(() => expect(vi.mocked(labelsCheck).mock.calls).toEqual([[{ product_ids: ['1'] }]]));
});

// ---------- the Labels box ----------

const COUNTS = { total: 214, has_price: 180, missing_price: 34, missing_price_tag: 60, printed: 168, not_printed: 46 };

function product(n: number): LabelRow {
  return {
    product_id: String(n), commerce_id: `gid://shopify/Product/${n}`, sku: `TEST-${n}`, title: `Test table ${n}`, product_type: 'Test tables',
    price_minor: 890000, marker: MISSING_TAG, created_at: Date.now() - 60_000, session: null, picture_path: null,
  };
}

function page(): LabelPage {
  return { rows: Array.from({ length: 5 }, (_, i) => product(i + 1)), next_cursor: null, counts: COUNTS, work_from: '2026-11-19' };
}

function sessionAnswer(): SessionPage {
  const row: SessionRowResponse_Serialize = {
    id: 90, sku: 'SKU-90', item_type: '', title: 'Item 90', main_count: 2, condition_count: 1, unreviewed_count: 0, status: 'in_progress',
    label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: Date.now(), created_at: Date.now(), cover_preview_path: null,
    cover_pending: false, arranged: false, publish: null, changes: null,
  };
  return { groups: [{ label: 'Today', total: 1, rows: [row] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 3, page: 1, pages: 1, total: 1 };
}

async function enterBox() {
  vi.mocked(sessionsPage).mockImplementation(async () => sessionAnswer());
  vi.mocked(labelsList).mockImplementation(async () => page());
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-90');
  fireEvent.click(screen.getByTestId('filter-labels'));
  await screen.findByTestId('label-row-1');
}

function bar() {
  return screen.queryByRole('toolbar', { name: 'Selected products' });
}

function tick(productId: string) {
  fireEvent.click(within(screen.getByTestId(`label-row-${productId}`)).getByRole('checkbox'));
}

it('l3_c17c_a_labels_row_without_a_session_has_a_menu_with_only_the_label_entry', async () => {
  await enterBox();
  const row = screen.getByTestId('label-row-2');
  fireEvent.pointerDown(within(row).getByRole('button', { name: 'Label actions' }), { button: 0 });
  const menu = await screen.findByRole('menu');
  expect(within(menu).getAllByRole('menuitem').map(i => i.textContent)).toEqual(['Generate label']);
  fireEvent.click(within(menu).getByRole('menuitem', { name: 'Generate label' }));
  await screen.findByRole('dialog');
  await waitFor(() => expect(vi.mocked(labelsCheck).mock.calls).toEqual([[{ product_ids: ['2'] }]]));
});

it('l3_c17d_generate_labels_for_n_opens_the_dialog_in_selection_order', async () => {
  await enterBox();
  tick('3');
  tick('1');
  const button = await within(bar()!).findByRole('button', { name: /Generate labels for 2/ });
  expect(button).toBeEnabled();
  fireEvent.click(button);
  await screen.findByRole('dialog');
  await waitFor(() => expect(vi.mocked(labelsCheck).mock.calls).toEqual([[{ product_ids: ['3', '1'] }]]));
});

it('l3_c17e_after_a_run_the_failed_products_stay_selected_and_the_page_is_asked_again', async () => {
  vi.mocked(labelsGenerate).mockResolvedValue({
    total: 2, rendered: 1, marked_printed: 1, reprinted: 0, no_price: 0,
    failures: [{ product_id: '3', sku: 'TEST-3', title: 'Test table 3', message: 'Barcode: no barcode.' }],
    not_run: [], artifacts: [], failed_product_ids: ['3'],
  });
  await enterBox();
  tick('3');
  tick('1');
  fireEvent.click(await within(bar()!).findByRole('button', { name: /Generate labels for 2/ }));
  fireEvent.click(await screen.findByRole('button', { name: 'Generate PDF' }));
  await screen.findByRole('heading', { name: '1 of 2 labels generated' });
  const before = vi.mocked(labelsList).mock.calls.length;
  fireEvent.click(screen.getByRole('button', { name: 'Close' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  await waitFor(() => expect(bar()).toHaveTextContent('1 selected'));
  expect(within(screen.getByTestId('label-row-3')).getByRole('checkbox')).toBeChecked();
  expect(within(screen.getByTestId('label-row-1')).getByRole('checkbox')).not.toBeChecked();
  await waitFor(() => expect(vi.mocked(labelsList).mock.calls.length).toBe(before + 1));
  expect(vi.mocked(labelsList).mock.calls.slice(before)).toEqual([[{ filter: 'missing_price_tag', query: null, cursor: null }]]);
  await new Promise(resolve => setTimeout(resolve, 50));
  expect(vi.mocked(labelsList).mock.calls.length).toBe(before + 1);
});

it('l3_c17f_without_a_shop_product_or_a_key_the_entry_stays_off', async () => {
  await menuOf(label(null), ALLOWED);
  expect(labelItem()).toEqual({ disabled: true, text: 'Generate labelPublish first so Pricing can see it' });
  cleanup();
  await menuOf(label('1'), { state: 'off', reason: 'no_key' });
  expect(labelItem()).toEqual({ disabled: true, text: 'Generate labelConnect Pricing in Settings → Pricing' });
});
