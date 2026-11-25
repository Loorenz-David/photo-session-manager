import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { useState } from 'react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { CheckGroup, CheckItem, LabelCheck, LabelRunResult } from '@/services/labels';

/**
 * L3·U4 (`plans/L3_generate_and_download.md` L3·C12–C14): the label dialog's check view, its preview and Generate
 * (09C §6.1, §6.2, §10), with `@/services/labels` mocked.
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
vi.mock('@/services/conversations', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/conversations')>()),
  conversationsSettingsGet: vi.fn(),
}));
vi.mock('@/features/settings/SettingsNav', async importOriginal => ({
  ...(await importOriginal<typeof import('@/features/settings/SettingsNav')>()),
  openSettingsSection: vi.fn(),
}));

import { LabelDialog } from '@/features/labels/LabelDialog';
import { labelsCheck, labelsGenerate, labelsPartial, labelsPreview } from '@/services/labels';

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

function item(id: string, outcome: CheckItem['outcome'], change: Partial<CheckItem> = {}): CheckItem {
  return {
    product_id: id, sku: `TEST-${id}`, title: `Test product ${id}`, price_minor: 890_000, status: 'not_printed', printed_at: null,
    outcome, issues: [], notes: [], picture_path: null, ...change,
  };
}

function group(template: CheckGroup['template'], items: CheckItem[], productTypes: string[] = []): CheckGroup {
  return { template, product_types: productTypes, items };
}

function check(groups: CheckGroup[], change: Partial<LabelCheck> = {}): LabelCheck {
  const all = groups.flatMap(g => g.items);
  const n = (o: CheckItem['outcome']) => all.filter(i => i.outcome === o).length;
  return {
    id: 'c', groups, unresolved: [],
    summary: { items: all.length, ready: all.length - n('fail'), failing: n('fail'), mark_printed: n('mark_printed'), reprint: n('reprint'), no_price: n('no_price') },
    sizes: [], ...change,
  };
}

function result(change: Partial<LabelRunResult> = {}): LabelRunResult {
  return { total: 1, rendered: 1, marked_printed: 1, reprinted: 0, no_price: 0, failures: [], not_run: [], artifacts: [], failed_product_ids: [], ...change };
}

/** Renders the dialog in a host that unmounts it when it closes; answers the `onClose` calls. */
function open(productIds: string[]) {
  const closed: (LabelRunResult | null)[] = [];
  function Host() {
    const [shown, setShown] = useState(true);
    return shown ? <LabelDialog productIds={productIds} onClose={r => { closed.push(r); setShown(false); }} /> : null;
  }
  render(<Host />);
  return closed;
}

function never<T>(): Promise<T> {
  return new Promise<T>(() => undefined);
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(r => {
    resolve = r;
  });
  return { promise, resolve };
}

function buttons(): string[] {
  return within(screen.getByRole('dialog')).getAllByRole('button').map(b => b.textContent?.trim() ?? '');
}

beforeEach(() => {
  vi.mocked(labelsPreview).mockImplementation(async ({ product_id }) => ({ path: `/d/previews/labels/c-${product_id}.png` }));
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('l3_c12a_the_dialog_opens_at_once_with_a_busy_state', async () => {
  vi.mocked(labelsCheck).mockReturnValue(never());
  open(['1']);
  const dialog = await screen.findByRole('dialog');
  expect(dialog).toHaveAttribute('aria-busy', 'true');
  expect(within(dialog).getByRole('status')).toHaveTextContent('Checking…');
  expect(vi.mocked(labelsCheck).mock.calls).toEqual([[{ product_ids: ['1'] }]]);
});

it('l3_c12c_products_pricing_did_not_find_are_listed_outside_the_groups', async () => {
  vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, [item('1', 'mark_printed')])], {
    unresolved: [{ product_id: '99', sku: null, message: 'No product.' }, { product_id: null, sku: 'X-1', message: 'No product.' }],
  }));
  open(['1', '99', 'x']);
  for (const text of ['99 · Not found in Pricing', 'X-1 · Not found in Pricing']) {
    const entry = await screen.findByText(text);
    expect(entry.closest('.label-group')).toBeNull();
  }
});

it('l3_c12d_the_batch_footer_in_pricings_wording', async () => {
  const items = Array.from({ length: 5 }, (_, i) => item(String(i + 1), 'mark_printed'));
  vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, items)]));
  open(['1', '2', '3', '4', '5']);
  expect(await screen.findByText('5 products will be marked Printed')).toBeInTheDocument();
  expect(screen.queryByText(/can't be generated with the current template/)).toBeNull();
  expect(screen.queryByText(/already printed/)).toBeNull();
  cleanup();
  vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, [
    item('1', 'mark_printed'), item('2', 'fail', { issues: ['x'] }), item('3', 'reprint'), item('4', 'reprint'),
  ])]));
  open(['1', '2', '3', '4']);
  expect(await screen.findByText('1 products will be marked Printed')).toBeInTheDocument();
  expect(screen.getByText("1 can't be generated with the current template. They're marked below.")).toBeInTheDocument();
  expect(screen.getByText("2 already printed. They're reprinted and stay Printed.")).toBeInTheDocument();
});

it('l3_c12e_one_product_says_what_happens_to_it', async () => {
  vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, [item('1', 'mark_printed')])]));
  open(['1']);
  expect(await screen.findByText('The product will be marked Printed.')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Generate PDF' })).toBeEnabled();
  cleanup();

  const printedAt = new Date(new Date().getFullYear(), 10, 4, 12).getTime();
  vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, [item('1', 'reprint', { status: 'printed', printed_at: printedAt })])]));
  open(['1']);
  expect(await screen.findByText('Label status stays Printed.')).toBeInTheDocument();
  expect(screen.getByText('Printed 4 Nov. Reprinting keeps it marked Printed.')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Reprint PDF' })).toBeEnabled();
  cleanup();

  vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, [item('1', 'no_price', { price_minor: null })])]));
  open(['1']);
  expect(await screen.findByText('Label status stays Not printed.')).toBeInTheDocument();
  expect(screen.getByText('This product has no selling price. You can generate its label, but it stays marked Not printed.')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Generate PDF' })).toBeEnabled();
});

it('l3_c12f_every_product_failing_disables_generate_and_cancel_closes', async () => {
  vi.mocked(labelsCheck).mockResolvedValue(check([group(null, [item('1', 'fail', { issues: ['a'] }), item('2', 'fail', { issues: ['b'] })])]));
  const closed = open(['1', '2']);
  expect(await screen.findByRole('button', { name: 'Generate PDF' })).toBeDisabled();
  fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(closed).toEqual([null]);
  expect(labelsGenerate).not.toHaveBeenCalled();
});

it('l3_c12g_a_refused_check_shows_its_message_and_only_close', async () => {
  for (const failure of [
    { code: 'labels_no_products', message: 'None of these products were found in Pricing.' },
    { code: 'labels_denied', message: "Session Manager isn't allowed to print labels. Ask a Pricing admin." },
  ]) {
    vi.mocked(labelsCheck).mockRejectedValue(failure);
    open(['1']);
    expect(await screen.findByText(failure.message)).toBeInTheDocument();
    expect(buttons()).toEqual(['Close']);
    cleanup();
  }
});

it('l3_c13a_the_preview_loads_on_a_placeholder_then_shows_on_white', async () => {
  const answer = deferred<{ path: string }>();
  vi.mocked(labelsPreview).mockReturnValue(answer.promise);
  vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, [item('1', 'mark_printed'), item('2', 'mark_printed')])]));
  open(['1', '2']);
  const placeholder = await screen.findByTestId('label-preview-placeholder');
  expect(placeholder).toHaveAttribute('data-ratio', '70 / 100');
  expect(vi.mocked(labelsPreview).mock.calls).toEqual([[{ check_id: 'c', product_id: '1' }]]);
  answer.resolve({ path: '/d/previews/labels/c-1.png' });
  const picture = await screen.findByRole('img', { name: 'Label preview' });
  expect(picture).toHaveAttribute('src', 'asset:///d/previews/labels/c-1.png');
  expect(picture.parentElement).toHaveClass('label-preview');
  expect(vi.mocked(labelsPreview)).toHaveBeenCalledTimes(1);
});

it('l3_c13b_the_arrows_step_through_a_batch', async () => {
  vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, [item('1', 'mark_printed'), item('2', 'mark_printed'), item('3', 'mark_printed')])]));
  open(['1', '2', '3']);
  const back = await screen.findByRole('button', { name: '‹' });
  const forward = screen.getByRole('button', { name: '›' });
  expect(back).toBeDisabled();
  fireEvent.click(forward);
  await waitFor(() => expect(vi.mocked(labelsPreview)).toHaveBeenCalledTimes(2));
  fireEvent.click(forward);
  await waitFor(() => expect(vi.mocked(labelsPreview)).toHaveBeenCalledTimes(3));
  expect(forward).toBeDisabled();
  expect(back).toBeEnabled();
  expect(vi.mocked(labelsPreview).mock.calls.map(([args]) => args.product_id)).toEqual(['1', '2', '3']);
});

it('l3_c13c_a_refused_preview_shows_its_message_and_generate_stays_on', async () => {
  for (const failure of [
    { code: 'labels_rejected', message: 'No template applies.' },
    { code: 'labels_preview_failed', message: "The label preview can't be shown." },
  ]) {
    vi.mocked(labelsPreview).mockRejectedValue(failure);
    vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, [item('1', 'mark_printed')])]));
    open(['1']);
    const message = await screen.findByText(failure.message);
    expect(message.closest('.label-preview-pane')).not.toBeNull();
    expect(screen.queryByRole('img', { name: 'Label preview' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Generate PDF' })).toBeEnabled();
    cleanup();
  }
});

it('l3_c13d_one_product_has_no_arrows', async () => {
  vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, [item('1', 'mark_printed')])]));
  open(['1']);
  await screen.findByRole('img', { name: 'Label preview' });
  expect(screen.queryByRole('button', { name: '‹' })).toBeNull();
  expect(screen.queryByRole('button', { name: '›' })).toBeNull();
});

async function checked(productIds = ['1']) {
  vi.mocked(labelsCheck).mockResolvedValue(check([group(T70, productIds.map(id => item(id, 'mark_printed')))]));
  const closed = open(productIds);
  await screen.findByRole('button', { name: 'Generate PDF' });
  return closed;
}

it('l3_c14a_generating_cannot_be_pressed_twice_nor_closed', async () => {
  vi.mocked(labelsGenerate).mockReturnValue(never());
  await checked();
  const generate = screen.getByRole('button', { name: 'Generate PDF' });
  fireEvent.click(generate);
  fireEvent.click(generate);
  const dialog = screen.getByRole('dialog');
  fireEvent.keyDown(dialog, { key: 'Escape' });
  fireEvent.pointerDown(document.body);
  fireEvent.click(document.body);
  expect(vi.mocked(labelsGenerate).mock.calls).toEqual([[{ check_id: 'c' }]]);
  expect(within(dialog).getByText('Generating…')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Generate PDF' })).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Cancel' })).toBeDisabled();
  expect(screen.getByRole('dialog')).toBeInTheDocument();
});

it('l3_c14b_a_dropped_connection_offers_try_again_with_the_same_check', async () => {
  vi.mocked(labelsGenerate).mockRejectedValueOnce({ code: 'labels_unreachable', message: "Pricing can't be reached right now." });
  vi.mocked(labelsGenerate).mockReturnValueOnce(never());
  await checked();
  fireEvent.click(screen.getByRole('button', { name: 'Generate PDF' }));
  expect(await screen.findByText("Pricing can't be reached right now.")).toBeInTheDocument();
  expect(buttons()).toEqual(['Cancel', 'Try again']);
  fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
  expect(vi.mocked(labelsGenerate).mock.calls).toEqual([[{ check_id: 'c' }], [{ check_id: 'c' }]]);
});

it('l3_c14c_a_busy_renderer_offers_try_again', async () => {
  vi.mocked(labelsGenerate).mockRejectedValue({ code: 'labels_renderer_unavailable', message: "Labels can't be generated right now. Try again in a moment." });
  await checked();
  fireEvent.click(screen.getByRole('button', { name: 'Generate PDF' }));
  expect(await screen.findByText("Labels can't be generated right now. Try again in a moment.")).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Try again' })).toBeEnabled();
});

it('l3_c14d_leaving_after_a_failed_generate_shows_what_was_generated', async () => {
  vi.mocked(labelsGenerate).mockRejectedValue({ code: 'labels_unreachable', message: "Pricing can't be reached right now." });
  vi.mocked(labelsPartial).mockResolvedValue(result({
    total: 50, rendered: 3, marked_printed: 3, not_run: Array.from({ length: 47 }, (_, i) => String(i + 4)),
    artifacts: [{ id: 'a1', filename: 'labels-1-70x100.pdf', width_mm: 70, height_mm: 100, page_count: 3 }],
    failed_product_ids: Array.from({ length: 47 }, (_, i) => String(i + 4)),
  }));
  await checked(['1', '2', '3']);
  fireEvent.click(screen.getByRole('button', { name: 'Generate PDF' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Cancel' }));
  expect(await screen.findByRole('heading', { name: '3 of 50 labels generated' })).toBeInTheDocument();
  expect(screen.getByText('✕ 47 not generated')).toBeInTheDocument();
  expect(vi.mocked(labelsPartial).mock.calls).toEqual([[{ check_id: 'c' }]]);
  cleanup();

  vi.mocked(labelsPartial).mockRejectedValue({ code: 'not_found', message: 'Not found' });
  const closed = await checked(['1', '2', '3']);
  fireEvent.click(screen.getByRole('button', { name: 'Generate PDF' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Cancel' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  expect(closed).toEqual([null]);
});

it('l3_c14e_a_refused_run_offers_only_close', async () => {
  vi.mocked(labelsGenerate).mockRejectedValue({ code: 'labels_denied', message: "Session Manager isn't allowed to print labels. Ask a Pricing admin." });
  await checked();
  fireEvent.click(screen.getByRole('button', { name: 'Generate PDF' }));
  expect(await screen.findByText("Session Manager isn't allowed to print labels. Ask a Pricing admin.")).toBeInTheDocument();
  expect(buttons()).toEqual(['Close']);
  expect(screen.queryByRole('button', { name: 'Try again' })).toBeNull();
});
