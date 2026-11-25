import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { CheckGroup, CheckItem, LabelCheck } from '@/services/labels';

/**
 * L4·U2 (`plans/L4_owner_corrections.md` L4·C6–C7): the check dialog's layout and its product cards (09C §14.1), with
 * `@/services/labels` and the opener mocked.
 */

vi.mock('@/services/labels', () => ({
  labelsCheck: vi.fn(),
  labelsPreview: vi.fn(),
  labelsGenerate: vi.fn(),
  labelsPartial: vi.fn(),
  labelsDownload: vi.fn(),
}));
vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn(async () => undefined) }));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/conversations', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/conversations')>()),
  conversationsSettingsGet: vi.fn(),
}));

import { openUrl } from '@tauri-apps/plugin-opener';
import { LabelDialog } from '@/features/labels/LabelDialog';
import { labelsCheck, labelsPreview } from '@/services/labels';
import { previewUrl } from '@/services/sessions';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

const ADDR = 'https://shop.example/search?q=TEST-1802';
const SENT = 'Not published to the Online Store yet; the QR code opens https://shop.example/search?q=TEST-1802.';
const T70 = { id: 't70', name: 'Label 70×100', width_mm: 70, height_mm: 100 };
const T50 = { id: 't50', name: 'Small tag', width_mm: 50, height_mm: 30 };

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

async function open(answer: LabelCheck, productIds = answer.groups.flatMap(g => g.items.map(i => i.product_id))) {
  vi.mocked(labelsCheck).mockResolvedValue(answer);
  render(<LabelDialog productIds={productIds} onClose={() => undefined} />);
  await screen.findByTestId('label-check-scroll');
}

/** One card of a batch of two (the second a plain product), so the dialog shows the batch layout. */
async function card(subject: CheckItem): Promise<HTMLElement> {
  await open(check([group(T70, [subject, item('9', 'mark_printed')])]));
  return screen.getByTestId(`label-card-${subject.product_id}`);
}

function pills(scope: HTMLElement): { text: string; tone: string | null }[] {
  const row = scope.querySelector('.label-card-warnings');
  if (!row) return [];
  return Array.from(row.querySelectorAll('.pill')).map(p => ({ text: p.textContent ?? '', tone: p.getAttribute('data-tone') }));
}

beforeEach(() => {
  vi.mocked(labelsPreview).mockImplementation(async ({ product_id }) => ({ path: `/d/previews/labels/c-${product_id}.png` }));
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

it('l4_c6a_the_list_scrolls_and_the_footer_holds_the_sentences_and_the_buttons', async () => {
  await open(check([group(T70, [item('1', 'mark_printed'), item('2', 'mark_printed')])]));
  const scroll = screen.getByTestId('label-check-scroll');
  const footer = screen.getByTestId('label-check-footer');
  expect(within(scroll).getByTestId('label-card-1')).toBeInTheDocument();
  expect(within(scroll).getByTestId('label-card-2')).toBeInTheDocument();
  expect(within(scroll).queryByRole('button', { name: 'Generate PDF' })).toBeNull();
  expect(within(scroll).queryByRole('button', { name: 'Cancel' })).toBeNull();
  expect(within(footer).getByText('2 products will be marked Printed')).toBeInTheDocument();
  expect(within(footer).getByRole('button', { name: 'Generate PDF' })).toBeInTheDocument();
  expect(within(footer).getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
  expect(scroll.contains(footer)).toBe(false);
  const pane = document.querySelector('.label-preview-pane') as HTMLElement;
  expect(pane).not.toBeNull();
  expect(scroll.contains(pane)).toBe(false);
  expect(footer.contains(pane)).toBe(false);
});

it('l4_c6b_a_group_header_is_two_lines_with_their_full_text_as_tooltip', async () => {
  await open(check([
    group(T70, [item('1', 'mark_printed')], ['Tables', 'Chairs']),
    group(T50, [item('2', 'mark_printed')]),
    group(null, [item('3', 'fail', { issues: ['No template applies.'] })]),
  ]));
  const header = (name: string) => screen.getByRole('region', { name }).querySelector('.label-group-header') as HTMLElement;
  const lines = (name: string) => Array.from(header(name).children).map(line => ({ text: line.textContent, title: line.getAttribute('title') }));
  expect(lines('Label 70×100')).toEqual([
    { text: 'Label 70×100', title: 'Label 70×100' },
    { text: 'Tables, Chairs · 70 × 100 mm', title: 'Tables, Chairs · 70 × 100 mm' },
  ]);
  expect(lines('Small tag')).toEqual([
    { text: 'Small tag', title: 'Small tag' },
    { text: '50 × 30 mm', title: '50 × 30 mm' },
  ]);
  expect(lines('No template')).toEqual([{ text: 'No template', title: 'No template' }]);
});

it('l4_c6c_one_card_per_product_with_picture_sku_title_price_and_outcome', async () => {
  await open(check([group(T70, [
    item('1', 'mark_printed', { sku: 'TEST-1', title: 'Test table', price_minor: 890_000, picture_path: '/d/previews/p/1.jpg' }),
    item('2', 'reprint', { status: 'printed' }),
    item('3', 'no_price', { price_minor: null }),
    item('4', 'fail', { issues: ['Barcode: no barcode.'] }),
  ])]));
  const first = screen.getByTestId('label-card-1');
  const thumb = first.querySelector('.label-card-thumb') as HTMLElement;
  expect(thumb.querySelector('img')).toHaveAttribute('src', previewUrl('/d/previews/p/1.jpg'));
  const side = thumb.parentElement as HTMLElement;
  expect(Array.from(side.children).indexOf(thumb)).toBeLessThan(Array.from(side.children).findIndex(c => c.textContent === 'TEST-1'));
  expect(within(first).getByText('Test table')).toHaveAttribute('title', 'Test table');
  expect(first.querySelector('.label-card-price')?.textContent).toBe('8\u2009900\u00A0kr');
  expect(within(first).getByText('Will be marked Printed')).toBeInTheDocument();
  expect(within(screen.getByTestId('label-card-2')).getByText('Reprint · stays Printed')).toBeInTheDocument();
  const third = screen.getByTestId('label-card-3');
  expect(within(third).getByText('No price · stays Not printed')).toBeInTheDocument();
  expect(third.querySelector('.label-card-price')).toHaveTextContent('No price');
  expect(within(screen.getByTestId('label-card-4')).getByText("Can't generate")).toBeInTheDocument();
});

it('l4_c6d_no_picture_shows_the_empty_tile_and_no_sku_shows_the_product_id', async () => {
  await open(check([group(T70, [item('1', 'mark_printed'), item('77', 'mark_printed', { sku: null })])]));
  const first = screen.getByTestId('label-card-1');
  expect(first.querySelector('img')).toBeNull();
  expect(within(first).getByTestId('thumb-placeholder')).toHaveClass('label-card-thumb');
  const other = screen.getByTestId('label-card-77');
  expect(other.querySelector('.label-card-sku')).toHaveTextContent(/^77$/);
});

it('l4_c6e_clicking_a_card_selects_it_and_shows_its_preview', async () => {
  await open(check([group(T70, [item('1', 'mark_printed'), item('2', 'mark_printed'), item('3', 'mark_printed')])]));
  const current = () => ['1', '2', '3'].filter(id => screen.getByTestId(`label-card-${id}`).getAttribute('aria-current') === 'true');
  expect(current()).toEqual(['1']);
  fireEvent.click(screen.getByTestId('label-card-3'));
  await waitFor(() => expect(current()).toEqual(['3']));
  await waitFor(() => expect(vi.mocked(labelsPreview).mock.calls.filter(([args]) => args.product_id === '3')).toHaveLength(1));
  expect(screen.getByRole('button', { name: '‹' })).toBeEnabled();
  expect(screen.getByRole('button', { name: '›' })).toBeDisabled();
});

it('l4_c7a_a_card_with_nothing_to_say_has_no_warnings_row', async () => {
  const subject = await card(item('1', 'mark_printed'));
  expect(subject.querySelector('.label-card-warnings')).toBeNull();
});

it('l4_c7b_no_price_is_an_amber_pill', async () => {
  const subject = await card(item('1', 'no_price', { price_minor: null }));
  expect(pills(subject)).toEqual([{ text: 'No price', tone: 'amber' }]);
});

it('l4_c7c_each_issue_is_a_red_pill', async () => {
  const subject = await card(item('1', 'fail', { issues: ['Barcode: no barcode.', 'No template applies.'] }));
  expect(pills(subject)).toEqual([
    { text: "Can't generate: Barcode: no barcode.", tone: 'red' },
    { text: "Can't generate: No template applies.", tone: 'red' },
  ]);
});

it('l4_c7d_qr_link_shows_and_hides_the_address_and_never_opens_it_by_itself', async () => {
  const subject = await card(item('1', 'mark_printed', { notes: [{ kind: 'not_published', message: SENT, link: ADDR }] }));
  expect(pills(subject)).toEqual([
    { text: 'Not published yet', tone: 'amber' },
    { text: 'QR link', tone: 'link' },
  ]);
  const qr = within(subject).getByRole('button', { name: 'QR link' });
  expect(screen.queryByText(SENT)).toBeNull();
  expect(screen.queryByText(ADDR)).toBeNull();
  fireEvent.click(qr);
  const address = within(subject).getByText(ADDR);
  expect(subject.querySelector('.label-card-warnings')?.contains(address)).toBe(false);
  expect(openUrl).not.toHaveBeenCalled();
  fireEvent.click(address);
  await waitFor(() => expect(vi.mocked(openUrl).mock.calls).toEqual([[ADDR]]));
  fireEvent.click(qr);
  expect(screen.queryByText(ADDR)).toBeNull();
  expect(vi.mocked(openUrl)).toHaveBeenCalledTimes(1);
});

it('l4_c7e_a_not_published_note_without_an_address_has_no_qr_link', async () => {
  const subject = await card(item('1', 'mark_printed', { notes: [{ kind: 'not_published', message: 'M', link: null }] }));
  expect(pills(subject)).toEqual([{ text: 'Not published yet', tone: 'amber' }]);
  expect(within(subject).queryByRole('button', { name: 'QR link' })).toBeNull();
});

it('l4_c7f_any_other_note_is_a_grey_pill_with_its_text', async () => {
  const subject = await card(item('1', 'mark_printed', { notes: [{ kind: 'other', message: 'Something to know.', link: null }] }));
  expect(pills(subject)).toEqual([{ text: 'Something to know.', tone: 'grey' }]);
});

it('l4_c7g_the_pills_keep_the_lists_order', async () => {
  const subject = await card(item('1', 'fail', {
    price_minor: null,
    issues: ['Barcode: no barcode.'],
    notes: [{ kind: 'not_published', message: SENT, link: ADDR }],
  }));
  expect(pills(subject).map(p => p.text)).toEqual(['No price', "Can't generate: Barcode: no barcode.", 'Not published yet', 'QR link']);
});

it('l4_c7h_a_product_pricing_did_not_find_is_a_line_in_the_list_without_a_card', async () => {
  await open(
    check([group(T70, [item('1', 'mark_printed')])], { unresolved: [{ product_id: '99', sku: null, message: 'No product.' }] }),
    ['1', '99'],
  );
  const line = screen.getByText('99 · Not found in Pricing');
  expect(screen.getByTestId('label-check-scroll').contains(line)).toBe(true);
  expect(line.closest('.label-group')).toBeNull();
  expect(screen.queryByTestId('label-card-99')).toBeNull();
});
