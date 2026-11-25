import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { useState } from 'react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { SessionRowResponse_Serialize } from '@/services/bindings';
import type { CheckItem, LabelCheck, LabelFile, LabelPage, LabelRow, LabelRunResult } from '@/services/labels';
import type { SessionPage } from '@/services/sessions';

/**
 * L4·U3 (`plans/L4_owner_corrections.md` L4·C8–C9): Generate goes straight to Preview (09C §14.2, MC-L2.17): every file
 * of the result is saved and opened by the dialog itself; everything generated → a toast and the dialog closes;
 * otherwise the result with the problems as cards. `@/services/labels` and `sonner`'s toast mocked.
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
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn(), loading: vi.fn() } }));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(),
  sessionsPage: vi.fn(),
  sessionFilesMisplaced: vi.fn(async () => 0),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/conversations', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/conversations')>()),
  conversationsSettingsGet: vi.fn(),
  conversationsCounts: vi.fn(async () => ({ here: 0, elsewhere: 0 })),
  conversationsSessionBadges: vi.fn(async () => []),
}));
vi.mock('@/services/research', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/research')>()), researchOverview: vi.fn(async () => []) }));
vi.mock('@/services/events', () => ({
  onSessionsChanged: vi.fn(async () => () => undefined),
  onLabelsChanged: vi.fn(async () => () => undefined),
  onConversationsChanged: vi.fn(async () => () => undefined),
  onPictureReady: vi.fn(async () => () => undefined),
}));
vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => null), onExportProgress: vi.fn(async () => () => undefined), openExportFolder: vi.fn(), runExport: vi.fn() }));
vi.mock('@/features/settings/SettingsNav', async importOriginal => ({
  ...(await importOriginal<typeof import('@/features/settings/SettingsNav')>()),
  openSettingsSection: vi.fn(),
}));

import { toast } from 'sonner';
import { LabelDialog } from '@/features/labels/LabelDialog';
import { SessionsPage } from '@/features/sessions/SessionsPage';
import { labelsCheck, labelsDownload, labelsGenerate, labelsList, labelsPartial, labelsPreview } from '@/services/labels';
import { sessionsPage } from '@/services/sessions';

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

const T70 = { id: 't70', name: 'Label 70×100', width_mm: 70, height_mm: 100 };
const A1: LabelFile = { id: 'a1', filename: 'labels-1-70x100.pdf', width_mm: 70, height_mm: 100, page_count: 2 };
const A2: LabelFile = { id: 'a2', filename: 'labels-1-50x30.pdf', width_mm: 50, height_mm: 30, page_count: 1 };
const PATH = '/h/Documents/Session Manager Labels/a.pdf';

function item(id: string, change: Partial<CheckItem> = {}): CheckItem {
  return {
    product_id: id, sku: `TEST-${id}`, title: `Test product ${id}`, price_minor: 890_000, status: 'not_printed', printed_at: null,
    outcome: 'mark_printed', issues: [], notes: [], picture_path: null, ...change,
  };
}

function checkOf(items: CheckItem[], change: Partial<LabelCheck> = {}): LabelCheck {
  return {
    id: 'c', groups: [{ template: T70, product_types: [], items }], unresolved: [],
    summary: { items: items.length, ready: items.length, failing: 0, mark_printed: items.length, reprint: 0, no_price: 0 },
    sizes: [], ...change,
  };
}

function result(change: Partial<LabelRunResult> = {}): LabelRunResult {
  return { total: 1, rendered: 1, marked_printed: 1, reprinted: 0, no_price: 0, failures: [], not_run: [], artifacts: [], failed_product_ids: [], ...change };
}

function saved(path: string) {
  return { path, opened: true };
}

function never<T>(): Promise<T> {
  return new Promise<T>(() => undefined);
}

/** Renders the dialog in a host that unmounts it when it closes; answers the `onClose` calls. */
async function open(check: LabelCheck, productIds = check.groups.flatMap(g => g.items.map(i => i.product_id))) {
  vi.mocked(labelsCheck).mockResolvedValue(check);
  const closed: (LabelRunResult | null)[] = [];
  function Host() {
    const [shown, setShown] = useState(true);
    return shown ? <LabelDialog productIds={productIds} onClose={r => { closed.push(r); setShown(false); }} /> : null;
  }
  render(<Host />);
  await screen.findByRole('button', { name: 'Generate PDF' });
  return closed;
}

function generate() {
  fireEvent.click(screen.getByRole('button', { name: 'Generate PDF' }));
}

function buttons(): string[] {
  return within(screen.getByRole('dialog')).getAllByRole('button').map(b => b.textContent?.trim() ?? '');
}

function downloads() {
  return vi.mocked(labelsDownload).mock.calls.map(([args]) => args);
}

function noDownloadOrDoneButton() {
  expect(screen.queryByRole('button', { name: /Download PDF/ })).toBeNull();
  expect(screen.queryByRole('button', { name: 'Done' })).toBeNull();
}

beforeEach(() => {
  vi.mocked(labelsPreview).mockImplementation(async ({ product_id }) => ({ path: `/d/previews/labels/c-${product_id}.png` }));
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('l4_c8a_one_label_is_saved_and_opened_and_the_dialog_closes_with_a_toast', async () => {
  const done = result({ artifacts: [A1] });
  vi.mocked(labelsGenerate).mockResolvedValue(done);
  vi.mocked(labelsDownload).mockResolvedValue(saved(PATH));
  const closed = await open(checkOf([item('1')]));
  noDownloadOrDoneButton();
  generate();
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(downloads()).toEqual([{ check_id: 'c', artifact_id: 'a1' }]);
  expect(vi.mocked(toast.success).mock.calls).toEqual([['Label generated · opened in Preview']]);
  expect(closed).toEqual([done]);
});

it('l4_c8b_every_file_of_a_batch_is_saved_in_order_and_the_toast_counts_no_price', async () => {
  vi.mocked(labelsGenerate).mockResolvedValue(result({ total: 3, rendered: 3, marked_printed: 2, no_price: 1, artifacts: [A1, A2] }));
  vi.mocked(labelsDownload).mockImplementation(async ({ artifact_id }) => saved(`/h/${artifact_id}.pdf`));
  await open(checkOf([item('1'), item('2'), item('3', { outcome: 'no_price', price_minor: null })]));
  generate();
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(downloads()).toEqual([{ check_id: 'c', artifact_id: 'a1' }, { check_id: 'c', artifact_id: 'a2' }]);
  expect(vi.mocked(toast.success).mock.calls).toEqual([['3 labels generated · opened in Preview · 1 stay Not printed (no price)']]);
});

it('l4_c8c_the_dialog_cannot_be_closed_while_the_files_are_saved', async () => {
  vi.mocked(labelsGenerate).mockResolvedValue(result({ artifacts: [A1] }));
  vi.mocked(labelsDownload).mockReturnValue(never());
  await open(checkOf([item('1')]));
  generate();
  const dialog = screen.getByRole('dialog');
  expect(await within(dialog).findByText('Opening in Preview…')).toBeInTheDocument();
  fireEvent.keyDown(dialog, { key: 'Escape' });
  fireEvent.pointerDown(document.body);
  fireEvent.click(document.body);
  expect(screen.getByRole('dialog')).toBeInTheDocument();
  expect(within(screen.getByRole('dialog')).getAllByRole('button').filter(b => !(b as HTMLButtonElement).disabled)).toEqual([]);
  expect(toast.success).not.toHaveBeenCalled();
  expect(downloads()).toHaveLength(1);
  noDownloadOrDoneButton();
});

// ---------- the Labels box (L4·C8(d)) ----------

const MISSING_TAG = { status: 'not_printed', printed_at: null, has_price: true, reset: null } as const;
const COUNTS = { total: 214, has_price: 180, missing_price: 34, missing_price_tag: 60, printed: 168, not_printed: 46 };

function product(n: number): LabelRow {
  return {
    product_id: String(n), commerce_id: `gid://shopify/Product/${n}`, sku: `TEST-${n}`, title: `Test table ${n}`, product_type: 'Test tables',
    price_minor: 890000, marker: MISSING_TAG, created_at: Date.now() - 60_000, session: null, picture_path: null,
  };
}

function sessionAnswer(): SessionPage {
  const row: SessionRowResponse_Serialize = {
    id: 90, sku: 'SKU-90', item_type: '', title: 'Item 90', main_count: 2, condition_count: 1, unreviewed_count: 0, status: 'in_progress',
    label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: Date.now(), created_at: Date.now(), cover_preview_path: null,
    cover_pending: false, arranged: false, publish: null, changes: null,
  };
  return { groups: [{ label: 'Today', total: 1, rows: [row] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 3, page: 1, pages: 1, total: 1 };
}

it('l4_c8d_a_batch_from_the_labels_box_closes_by_itself_and_clears_the_selection', async () => {
  const page: LabelPage = { rows: Array.from({ length: 5 }, (_, i) => product(i + 1)), next_cursor: null, counts: COUNTS, work_from: '2026-11-19' };
  vi.mocked(sessionsPage).mockImplementation(async () => sessionAnswer());
  vi.mocked(labelsList).mockImplementation(async () => page);
  vi.mocked(labelsCheck).mockImplementation(async ({ product_ids }) => checkOf(product_ids.map(id => item(id))));
  vi.mocked(labelsGenerate).mockResolvedValue(result({ total: 2, rendered: 2, marked_printed: 2, artifacts: [A1] }));
  vi.mocked(labelsDownload).mockResolvedValue(saved(PATH));
  render(<SessionsPage navigate={() => undefined} />);
  await screen.findByTestId('session-row-90');
  fireEvent.click(screen.getByTestId('filter-labels'));
  await screen.findByTestId('label-row-1');
  for (const id of ['3', '1']) fireEvent.click(within(screen.getByTestId(`label-row-${id}`)).getByRole('checkbox'));
  const bar = () => screen.queryByRole('toolbar', { name: 'Selected products' });
  fireEvent.click(await within(bar()!).findByRole('button', { name: /Generate labels for 2/ }));
  await screen.findByRole('button', { name: 'Generate PDF' });
  const before = vi.mocked(labelsList).mock.calls.length;
  generate();
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  await waitFor(() => expect(vi.mocked(labelsList).mock.calls.length).toBe(before + 1));
  expect(vi.mocked(labelsList).mock.calls.slice(before)).toEqual([[{ filter: 'missing_price_tag', query: null, cursor: null }]]);
  for (const id of ['1', '3']) expect(within(screen.getByTestId(`label-row-${id}`)).getByRole('checkbox')).not.toBeChecked();
  expect(bar()?.textContent ?? '').not.toMatch(/selected/);
  expect(toast.success).toHaveBeenCalledTimes(1);
  await new Promise(resolve => setTimeout(resolve, 50));
  expect(vi.mocked(labelsList).mock.calls.length).toBe(before + 1);
});

// ---------- L4·C9: when anything went wrong ----------

it('l4_c9a_a_failed_product_keeps_the_dialog_on_the_result_with_its_card', async () => {
  const done = result({
    total: 3, rendered: 2, marked_printed: 1, reprinted: 1,
    failures: [{ product_id: '3', sku: 'TEST-3', title: 'Test lamp', message: 'Barcode: no barcode.' }],
    artifacts: [A1], failed_product_ids: ['3'],
  });
  vi.mocked(labelsGenerate).mockResolvedValue(done);
  vi.mocked(labelsDownload).mockResolvedValue(saved(PATH));
  const closed = await open(checkOf([
    item('1'), item('2', { outcome: 'reprint', status: 'printed' }),
    item('3', { title: 'Test lamp', picture_path: '/d/previews/p/3.jpg' }),
  ]));
  generate();
  expect(await screen.findByRole('heading', { name: '2 of 3 labels generated' })).toBeInTheDocument();
  expect(downloads()).toEqual([{ check_id: 'c', artifact_id: 'a1' }]);
  for (const line of ['✓ 1 marked Printed', '↻ 1 reprinted · stay Printed', '✕ 1 failed']) expect(screen.getByText(line)).toBeInTheDocument();
  const card = screen.getByTestId('label-card-3');
  expect(card.querySelector('.label-card-sku')).toHaveTextContent('TEST-3');
  expect(within(card).getByText('Test lamp')).toBeInTheDocument();
  expect(card.querySelector('.label-card-thumb img')).toHaveAttribute('src', 'asset:///d/previews/p/3.jpg');
  expect(Array.from(card.querySelectorAll('.label-card-warnings .pill')).map(p => [p.textContent, p.getAttribute('data-tone')]))
    .toEqual([["Can't generate: Barcode: no barcode.", 'red']]);
  expect(screen.getByText('Failed products stay selected so you can fix and retry.')).toBeInTheDocument();
  expect(screen.getByText(PATH).parentElement).toHaveTextContent(`Saved and opened in Preview: ${PATH}`);
  expect(buttons()).toEqual(['Close']);
  expect(toast.success).not.toHaveBeenCalled();
  noDownloadOrDoneButton();
  fireEvent.click(screen.getByRole('button', { name: 'Close' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(closed).toEqual([done]);
});

it('l4_c9b_a_product_pricing_did_not_find_keeps_the_dialog_open', async () => {
  vi.mocked(labelsGenerate).mockResolvedValue(result({ total: 2, rendered: 2, marked_printed: 1, no_price: 1, artifacts: [A1] }));
  vi.mocked(labelsDownload).mockResolvedValue(saved(PATH));
  await open(
    checkOf([item('1'), item('2', { outcome: 'no_price', price_minor: null })], { unresolved: [{ product_id: '99', sku: null, message: 'No product.' }] }),
    ['1', '2', '99'],
  );
  generate();
  expect(await screen.findByRole('heading', { name: '2 of 3 labels generated' })).toBeInTheDocument();
  expect(downloads()).toEqual([{ check_id: 'c', artifact_id: 'a1' }]);
  expect(screen.getByText('○ 1 stay Not printed · No selling price')).toBeInTheDocument();
  expect(screen.getByText('99 · Not found in Pricing')).toBeInTheDocument();
  expect(buttons()).toEqual(['Close']);
  expect(toast.success).not.toHaveBeenCalled();
});

it('l4_c9c_one_failed_product_saves_nothing_and_shows_its_card', async () => {
  vi.mocked(labelsGenerate).mockResolvedValue(result({
    total: 1, rendered: 0, marked_printed: 0,
    failures: [{ product_id: '3', sku: 'TEST-3', title: 'Test lamp', message: 'Barcode: no barcode.' }],
    failed_product_ids: ['3'],
  }));
  await open(checkOf([item('3', { title: 'Test lamp' })]));
  generate();
  expect(await screen.findByRole('heading', { name: 'Label not generated' })).toBeInTheDocument();
  expect(labelsDownload).not.toHaveBeenCalled();
  expect(within(screen.getByTestId('label-card-3')).getByText("Can't generate: Barcode: no barcode.")).toHaveAttribute('data-tone', 'red');
  expect(buttons()).toEqual(['Close']);
  expect(screen.queryByText('Failed products stay selected so you can fix and retry.')).toBeNull();
});

it('l4_c9d_leaving_after_a_failed_generate_saves_what_was_made', async () => {
  const notRun = Array.from({ length: 47 }, (_, i) => String(i + 301));
  vi.mocked(labelsGenerate).mockRejectedValue({ code: 'labels_unreachable', message: "Pricing can't be reached right now." });
  vi.mocked(labelsPartial).mockResolvedValue(result({ total: 347, rendered: 300, marked_printed: 300, not_run: notRun, artifacts: [A1], failed_product_ids: notRun }));
  vi.mocked(labelsDownload).mockResolvedValue(saved(PATH));
  await open(checkOf([item('1'), item('2'), item('3')]));
  generate();
  fireEvent.click(await screen.findByRole('button', { name: 'Cancel' }));
  expect(await screen.findByRole('heading', { name: '300 of 347 labels generated' })).toBeInTheDocument();
  expect(vi.mocked(labelsPartial).mock.calls).toEqual([[{ check_id: 'c' }]]);
  expect(downloads()).toEqual([{ check_id: 'c', artifact_id: 'a1' }]);
  expect(screen.getByText('✕ 47 not generated')).toBeInTheDocument();
  expect(buttons()).toEqual(['Close']);
  expect(toast.success).not.toHaveBeenCalled();
  cleanup();
  vi.mocked(labelsDownload).mockClear();

  vi.mocked(labelsPartial).mockRejectedValue({ code: 'not_found', message: 'Not found' });
  const closed = await open(checkOf([item('1'), item('2'), item('3')]));
  generate();
  fireEvent.click(await screen.findByRole('button', { name: 'Cancel' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(closed).toEqual([null]);
  expect(labelsDownload).not.toHaveBeenCalled();
});

it('l4_c9e_a_file_that_could_not_be_saved_offers_try_again', async () => {
  vi.mocked(labelsGenerate).mockResolvedValue(result({ artifacts: [A1] }));
  vi.mocked(labelsDownload)
    .mockRejectedValueOnce({ code: 'labels_save_failed', message: "Couldn't save the PDF: disk full." })
    .mockResolvedValueOnce(saved('/h/x/a.pdf'));
  const closed = await open(checkOf([item('1')]));
  generate();
  expect(await screen.findByRole('heading', { name: 'Label generated' })).toBeInTheDocument();
  const line = screen.getByTestId('label-file-a1');
  expect(within(line).getByText("Couldn't save the PDF: disk full.")).toBeInTheDocument();
  expect(toast.success).not.toHaveBeenCalled();
  fireEvent.click(within(line).getByRole('button', { name: 'Try again' }));
  expect(await within(line).findByText('/h/x/a.pdf')).toBeInTheDocument();
  expect(downloads()).toEqual([{ check_id: 'c', artifact_id: 'a1' }, { check_id: 'c', artifact_id: 'a1' }]);
  expect(line).toHaveTextContent('Saved and opened in Preview: /h/x/a.pdf');
  expect(within(line).queryByRole('button', { name: 'Try again' })).toBeNull();
  expect(screen.getByRole('dialog')).toBeInTheDocument();
  expect(closed).toEqual([]);
  expect(toast.success).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Close' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(toast.success).not.toHaveBeenCalled();
});

it('l4_c9f_an_expired_file_has_no_try_again', async () => {
  vi.mocked(labelsGenerate).mockResolvedValue(result({ artifacts: [A1] }));
  vi.mocked(labelsDownload).mockRejectedValue({ code: 'labels_expired', message: 'This file has expired. Generate the labels again.' });
  await open(checkOf([item('1')]));
  generate();
  expect(await screen.findByText('This file has expired. Generate the labels again.')).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Try again' })).toBeNull();
  expect(buttons()).toEqual(['Close']);
});

it('l4_c9g_a_file_preview_could_not_open_says_where_it_is_and_is_not_asked_again', async () => {
  const where = '/Users/x/Documents/Session Manager Labels/a.pdf';
  vi.mocked(labelsGenerate).mockResolvedValue(result({ artifacts: [A1] }));
  vi.mocked(labelsDownload).mockResolvedValue({ path: where, opened: false });
  await open(checkOf([item('1')]));
  generate();
  expect(await screen.findByText(where)).toBeInTheDocument();
  expect(screen.getByTestId('label-file-a1')).toHaveTextContent(`Saved, but Preview couldn't open it: ${where}`);
  expect(screen.queryByRole('button', { name: 'Try again' })).toBeNull();
  await new Promise(resolve => setTimeout(resolve, 30));
  expect(downloads()).toHaveLength(1);
  expect(toast.success).not.toHaveBeenCalled();
});

it('l4_c9h_every_file_is_asked_even_when_one_fails', async () => {
  vi.mocked(labelsGenerate).mockResolvedValue(result({ total: 2, rendered: 2, marked_printed: 2, artifacts: [A1, A2] }));
  vi.mocked(labelsDownload).mockImplementation(async ({ artifact_id }) => {
    if (artifact_id === 'a1') throw { code: 'labels_save_failed', message: "Couldn't save the PDF: disk full." };
    return saved('/h/a2.pdf');
  });
  await open(checkOf([item('1'), item('2')]));
  generate();
  expect(await screen.findByRole('heading', { name: '2 labels generated' })).toBeInTheDocument();
  expect(downloads()).toEqual([{ check_id: 'c', artifact_id: 'a1' }, { check_id: 'c', artifact_id: 'a2' }]);
  const first = screen.getByTestId('label-file-a1');
  expect(within(first).getByText("Couldn't save the PDF: disk full.")).toBeInTheDocument();
  expect(within(first).getByRole('button', { name: 'Try again' })).toBeInTheDocument();
  expect(screen.getByTestId('label-file-a2')).toHaveTextContent('Saved and opened in Preview: /h/a2.pdf');
  expect(screen.getByText('Print at 100 % (not Fit to page).')).toBeInTheDocument();
  expect(toast.success).not.toHaveBeenCalled();
});
