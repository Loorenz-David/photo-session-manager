import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { useState } from 'react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { CheckItem, LabelCheck, LabelFile, LabelRunResult } from '@/services/labels';

/**
 * L3·U4 (`plans/L3_generate_and_download.md` L3·C15, C16): the label dialog's done view and its downloads (09C §6.3–§6.5,
 * §10), with `@/services/labels` mocked.
 */

vi.mock('@/services/labels', () => ({
  labelsCheck: vi.fn(),
  labelsPreview: vi.fn(),
  labelsGenerate: vi.fn(),
  labelsPartial: vi.fn(),
  labelsDownload: vi.fn(),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  previewUrl: (path: string) => `asset://${path}`,
}));

import { LabelDialog } from '@/features/labels/LabelDialog';
import { labelsCheck, labelsDownload, labelsGenerate, labelsPreview } from '@/services/labels';

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
const FILE_70: LabelFile = { id: 'a1', filename: 'labels-1-70x100.pdf', width_mm: 70, height_mm: 100, page_count: 2 };
const FILE_50: LabelFile = { id: 'a2', filename: 'labels-1-50x30.pdf', width_mm: 50, height_mm: 30, page_count: 1 };

function item(id: string): CheckItem {
  return {
    product_id: id, sku: `TEST-${id}`, title: `Test product ${id}`, price_minor: 890_000, status: 'not_printed', printed_at: null,
    outcome: 'mark_printed', issues: [], notes: [],
  };
}

function check(ids: string[]): LabelCheck {
  return {
    id: 'c', groups: [{ template: T70, product_types: [], items: ids.map(item) }], unresolved: [],
    summary: { items: ids.length, ready: ids.length, failing: 0, mark_printed: ids.length, reprint: 0, no_price: 0 }, sizes: [],
  };
}

function result(change: Partial<LabelRunResult>): LabelRunResult {
  return { total: 1, rendered: 1, marked_printed: 0, reprinted: 0, no_price: 0, failures: [], not_run: [], artifacts: [], failed_product_ids: [], ...change };
}

/** Opens the dialog for `ids`, generates, and waits for the done view of `answer`; answers the `onClose` calls. */
async function done(ids: string[], answer: LabelRunResult) {
  vi.mocked(labelsCheck).mockResolvedValue(check(ids));
  vi.mocked(labelsGenerate).mockResolvedValue(answer);
  const closed: (LabelRunResult | null)[] = [];
  function Host() {
    const [shown, setShown] = useState(true);
    return shown ? <LabelDialog productIds={ids} onClose={r => { closed.push(r); setShown(false); }} /> : null;
  }
  render(<Host />);
  fireEvent.click(await screen.findByRole('button', { name: 'Generate PDF' }));
  await screen.findByText(/labels? (not )?generated$/i, { selector: 'h2' });
  return closed;
}

function title(): string {
  return within(screen.getByRole('dialog')).getByRole('heading').textContent ?? '';
}

function lines(): string[] {
  return [...document.querySelectorAll('.label-done-lines li')].map(li => li.textContent ?? '');
}

beforeEach(() => {
  vi.mocked(labelsPreview).mockResolvedValue({ path: '/d/previews/labels/c-1.png' });
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('l3_c15a_a_batch_fully_generated', async () => {
  await done(['1', '2', '3'], result({ total: 3, rendered: 3, marked_printed: 2, no_price: 1 }));
  expect(title()).toBe('3 labels generated');
  expect(lines()).toEqual(['✓ 2 marked Printed', '○ 1 stay Not printed · No selling price']);
});

it('l3_c15b_a_batch_with_a_failure_says_so_and_lists_it', async () => {
  await done(['1', '2', '3'], result({
    total: 3, rendered: 2, marked_printed: 1, no_price: 1,
    failures: [{ product_id: '3', sku: 'TEST-3', title: 'Test lamp', message: 'Barcode: no barcode.' }], failed_product_ids: ['3'],
  }));
  expect(title()).toBe('2 of 3 labels generated');
  expect(lines()).toContain('✕ 1 failed');
  const failed = screen.getByText('TEST-3 · Test lamp').closest('li')!;
  expect(within(failed).getByText('Barcode: no barcode.')).toBeInTheDocument();
  expect(screen.getByText('Failed products stay selected so you can fix and retry.')).toBeInTheDocument();
});

it('l3_c15c_one_product_generated_or_not', async () => {
  await done(['1'], result({ total: 1, rendered: 1, marked_printed: 1 }));
  expect(title()).toBe('Label generated');
  cleanup();
  await done(['1'], result({
    total: 1, rendered: 0, failures: [{ product_id: '1', sku: 'TEST-1', title: 'Test product 1', message: 'Barcode: no barcode.' }],
    failed_product_ids: ['1'],
  }));
  expect(title()).toBe('Label not generated');
  expect(screen.getByText('Barcode: no barcode.')).toBeInTheDocument();
});

it('l3_c15d_reprints_stay_printed', async () => {
  await done(['1', '2'], result({ total: 2, rendered: 2, reprinted: 2 }));
  expect(lines()).toEqual(['↻ 2 reprinted · stay Printed']);
});

it('l3_c15e_products_not_run_are_never_reported_generated', async () => {
  await done(['1', '2'], result({
    total: 347, rendered: 300, marked_printed: 300, not_run: Array.from({ length: 47 }, (_, i) => String(i + 301)),
  }));
  expect(title()).toBe('300 of 347 labels generated');
  expect(lines()).toContain('✕ 47 not generated');
});

it('l3_c16a_one_file_is_one_download_that_closes_the_dialog', async () => {
  vi.mocked(labelsDownload).mockResolvedValue({ path: '/Users/x/Documents/Session Manager Labels/labels-1-70x100.pdf', opened: true });
  const closed = await done(['1'], result({ marked_printed: 1, artifacts: [FILE_70] }));
  expect(screen.getAllByRole('button', { name: /Download PDF/ })).toHaveLength(1);
  expect(screen.getByText('Print at 100 % (not Fit to page).')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: /Download PDF/ }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(vi.mocked(labelsDownload).mock.calls).toEqual([[{ check_id: 'c', artifact_id: 'a1' }]]);
  expect(closed).toHaveLength(1);
});

it('l3_c16b_several_files_are_rows_with_their_own_download_and_done', async () => {
  vi.mocked(labelsDownload).mockResolvedValue({ path: '/Users/x/Documents/Session Manager Labels/labels-1-70x100.pdf', opened: true });
  const closed = await done(['1', '2', '3'], result({ total: 3, rendered: 3, marked_printed: 3, artifacts: [FILE_70, FILE_50] }));
  const first = screen.getByTestId('label-file-a1');
  const second = screen.getByTestId('label-file-a2');
  expect(within(first).getByText('labels-1-70x100.pdf')).toBeInTheDocument();
  expect(within(first).getByText('2 labels')).toBeInTheDocument();
  expect(within(first).getByText('70 × 100 mm')).toBeInTheDocument();
  expect(within(second).getByText('labels-1-50x30.pdf')).toBeInTheDocument();
  expect(within(second).getByText('1 label')).toBeInTheDocument();
  expect(within(second).getByText('50 × 30 mm')).toBeInTheDocument();
  expect(within(first).getByRole('button', { name: /Download PDF/ })).toBeInTheDocument();
  expect(within(second).getByRole('button', { name: /Download PDF/ })).toBeInTheDocument();
  fireEvent.click(within(first).getByRole('button', { name: /Download PDF/ }));
  await waitFor(() => expect(vi.mocked(labelsDownload)).toHaveBeenCalledTimes(1));
  await screen.findByText(/Saved and opened in Preview/);
  expect(screen.getByRole('dialog')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Done' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(closed).toHaveLength(1);
});

it('l3_c16c_no_file_is_close', async () => {
  await done(['1', '2'], result({ total: 2, rendered: 0, failures: [{ product_id: '1', sku: 'A', title: 'B', message: 'M' }] }));
  expect(screen.queryByRole('button', { name: /Download PDF/ })).toBeNull();
  expect(screen.getByRole('button', { name: 'Close' })).toBeInTheDocument();
});

it('l3_c16d_a_failed_download_says_why_and_saving_can_be_tried_again', async () => {
  vi.mocked(labelsDownload).mockRejectedValueOnce({ code: 'labels_expired', message: 'This file has expired. Generate the labels again.' });
  await done(['1'], result({ marked_printed: 1, artifacts: [FILE_70] }));
  fireEvent.click(screen.getByRole('button', { name: /Download PDF/ }));
  expect(await screen.findByText('This file has expired. Generate the labels again.')).toBeInTheDocument();
  cleanup();

  vi.mocked(labelsDownload).mockRejectedValueOnce({ code: 'labels_save_failed', message: "Couldn't save the PDF: disk full." });
  vi.mocked(labelsDownload).mockResolvedValueOnce({ path: '/Users/x/Documents/Session Manager Labels/labels-1-70x100.pdf', opened: false });
  await done(['1'], result({ marked_printed: 1, artifacts: [FILE_70] }));
  fireEvent.click(screen.getByRole('button', { name: /Download PDF/ }));
  const message = await screen.findByText("Couldn't save the PDF: disk full.");
  fireEvent.click(within(message.closest('[role="alert"]') as HTMLElement).getByRole('button', { name: 'Try again' }));
  await waitFor(() => expect(vi.mocked(labelsDownload)).toHaveBeenCalledTimes(3));
  expect(vi.mocked(labelsDownload).mock.calls.slice(1)).toEqual([[{ check_id: 'c', artifact_id: 'a1' }], [{ check_id: 'c', artifact_id: 'a1' }]]);
});

it('l3_c16e_a_file_preview_did_not_open_shows_where_it_was_saved', async () => {
  const path = '/Users/x/Documents/Session Manager Labels/a.pdf';
  vi.mocked(labelsDownload).mockResolvedValue({ path, opened: false });
  const closed = await done(['1'], result({ marked_printed: 1, artifacts: [FILE_70] }));
  fireEvent.click(screen.getByRole('button', { name: /Download PDF/ }));
  expect(await screen.findByText(path)).toBeInTheDocument();
  expect(screen.getByText(/Saved, but Preview couldn't open it/)).toBeInTheDocument();
  expect(screen.getByRole('dialog')).toBeInTheDocument();
  expect(closed).toEqual([]);
  fireEvent.click(screen.getByRole('button', { name: 'Close' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
});
