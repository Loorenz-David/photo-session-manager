import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { AttributeValue, Definition, Form, FormField, FormRead, Group } from '@/services/attributes';
import type { Item } from '@/services/items';
import type { SessionView } from '@/services/photos';

vi.mock('@/services/attributes', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/attributes')>()),
  getForm: vi.fn(), listDefinitions: vi.fn(), setAttributeValue: vi.fn(), clearAttribute: vi.fn(), acceptAllSuggestions: vi.fn(),
  readShopValues: vi.fn(), setFieldDefault: vi.fn(), addSavedValue: vi.fn(), removeSavedValue: vi.fn(), fillDefaults: vi.fn(),
}));
vi.mock('@/services/items', () => ({ listItemTypes: vi.fn(), setItemType: vi.fn(), refreshItem: vi.fn() }));
vi.mock('@/services/photos', () => ({ getSessionView: vi.fn() }));
vi.mock('@/services/stock', () => ({ stockLocations: vi.fn(), itemStockGet: vi.fn(), itemStockSet: vi.fn() }));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn() } }));

import { FormDialog } from '@/features/form/FormDialog';
import { BatchQueue } from '@/features/publish/BatchQueue';
import { addSavedValue, fillDefaults, getForm, readShopValues, removeSavedValue, setAttributeValue, setFieldDefault } from '@/services/attributes';
import { listItemTypes, setItemType } from '@/services/items';
import { getSessionView } from '@/services/photos';
import { itemStockGet, stockLocations } from '@/services/stock';

globalThis.ResizeObserver ??= class { observe() {} unobserve() {} disconnect() {} };
Element.prototype.scrollIntoView ??= function scrollIntoView() {};
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;

const ITEM_ID = 7;
const baseItem: Item = { id: ITEM_ID, sku: 'SOFA-1', folder_key: 'SOFA-1', barcode: null, item_type: 'Sofa', title: 'Velvet sofa', origin: 'linked', external_id: 'gid://shopify/Product/1', created_at: 0, updated_at: 0 };
const def = (key: string, core_type: string, label: string, group: Group = 'details', choices: string[] = []): Definition => ({ integration: key.startsWith('product.') ? 'core' : 'shopify', key, label, core_type, choices, validations: '{}', group, description: null, writable: true });
const stored = (key: string, value: unknown, provenance = 'human'): AttributeValue => ({ item_id: ITEM_ID, key, value: JSON.stringify(value), valid: true, provenance, ai_confidence: null, ai_status: null, ai_reason: null, ai_sources: null, updated_at: 1 });
const field = (definition: Definition, value: AttributeValue | null, extra: Partial<FormField> = {}): FormField => ({ definition, value, is_default: value?.provenance === 'default', is_severity: false, selected: null, default: null, saved: [], ...extra });
const form = (groups: Partial<Record<Group, FormField[]>>, itemTypeFromIntegration = false): Form => ({ item_id: ITEM_ID, item_type_from_integration: itemTypeFromIntegration, groups: (['details', 'condition', 'product'] as Group[]).map(group => ({ group, fields: groups[group] ?? [], filled: 0, total: groups[group]?.length ?? 0 })) });
const read = (state: string, extra: Partial<FormRead> = {}): FormRead => ({ item_id: ITEM_ID, state, error: null, item_type: 'Sofa', title: 'Velvet sofa', ...extra });

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(r => { resolve = r; });
  return { promise, resolve };
}

beforeEach(() => {
  vi.mocked(setAttributeValue).mockImplementation(async (_id, key, value) => stored(key, value));
  vi.mocked(listItemTypes).mockResolvedValue(['Chair', 'Sofa']);
  vi.mocked(stockLocations).mockResolvedValue({ locations: [], missing_access: null });
  vi.mocked(itemStockGet).mockResolvedValue({ new_item: false, choice: null });
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

const shopForm = () => form({
  details: [
    field(def('custom.color_furniture', 'text', 'Color'), stored('custom.color_furniture', 'Green', 'integration')),
    field(def('custom.country', 'choice', 'Country', 'details', ['Denmark', 'Sweden']), stored('custom.country', 'Sweden', 'integration')),
  ],
  condition: [field(def('custom.condition_scratch', 'choice', 'Scratch', 'condition', ['N/A', 'None', 'Minor', 'Moderate', 'Major']), stored('custom.condition_scratch', 'Minor', 'integration'), { is_severity: true })],
  product: [
    field(def('product.title', 'text', 'Title', 'product'), stored('product.title', 'Velvet sofa', 'integration')),
    field(def('product.description', 'rich_text', 'Description', 'product'), stored('product.description', 'Deep seat.', 'integration')),
    field(def('product.tags', 'tags', 'Tags', 'product'), stored('product.tags', ['velvet', 'green'], 'integration')),
  ],
}, true);

const fieldBox = (label: string) => screen.getByLabelText(label).closest('[data-field]') as HTMLElement;

it('C1(a) a linked item\'s form waits quietly for the product, then shows its values labelled from Shopify', async () => {
  const pending = deferred<FormRead[]>();
  vi.mocked(readShopValues).mockReturnValue(pending.promise);
  vi.mocked(getForm).mockResolvedValue(shopForm());
  render(<FormDialog open onOpenChange={vi.fn()} item={baseItem} photos={[]} onItemChanged={vi.fn()} />);
  await waitFor(() => expect(getForm).toHaveBeenCalled());
  // While reading: a loading state, never empty fields.
  expect(document.querySelector('[aria-busy="true"]')).not.toBeNull();
  expect(screen.queryByLabelText('Color')).toBeNull();
  await act(async () => { pending.resolve([read('read')]); });
  expect(readShopValues).toHaveBeenCalledExactlyOnceWith([ITEM_ID]);
  expect(await screen.findByLabelText('Color')).toHaveValue('Green');
  expect(within(fieldBox('Color')).getByText('from Shopify')).toBeInTheDocument();
  expect(within(fieldBox('Country')).getByText('from Shopify')).toBeInTheDocument();
  expect(screen.getByLabelText('Item type')).toHaveValue('Sofa');
  expect(screen.getByLabelText('Item type').parentElement!.querySelector('[data-from-integration]')).toHaveTextContent('from Shopify');
  fireEvent.click(screen.getByRole('tab', { name: /Condition/ }));
  expect(within(await screen.findByText('Scratch').then(label => label.closest('[data-field]') as HTMLElement)).getByText('from Shopify')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('tab', { name: /Product/ }));
  expect(await screen.findByLabelText('Title')).toHaveValue('Velvet sofa');
  for (const label of ['Title', 'Description']) expect(within(fieldBox(label)).getByText('from Shopify')).toBeInTheDocument();
  const tags = screen.getByText('Tags').closest('[data-field]') as HTMLElement;
  expect(within(tags).getByText('velvet')).toBeInTheDocument();
  expect(within(tags).getByText('from Shopify')).toBeInTheDocument();
});

it('C1(b) a failed product read says so and the form stays editable with the local values', async () => {
  vi.mocked(readShopValues).mockResolvedValue([read('failed', { error: 'provider: shop unavailable' })]);
  vi.mocked(getForm).mockResolvedValue(form({ details: [field(def('custom.color_furniture', 'text', 'Color'), stored('custom.color_furniture', 'Blue'))] }));
  render(<FormDialog open onOpenChange={vi.fn()} item={baseItem} photos={[]} onItemChanged={vi.fn()} />);
  expect(await screen.findByRole('alert')).toHaveTextContent("Couldn't read this product from Shopify");
  const color = await screen.findByLabelText('Color');
  expect(color).toHaveValue('Blue');
  fireEvent.change(color, { target: { value: 'Teal' } });
  fireEvent.blur(color);
  await waitFor(() => expect(setAttributeValue).toHaveBeenCalledExactlyOnceWith(ITEM_ID, 'custom.color_furniture', 'Teal'));
});

it('C2(b) saved values are one-click picks (most recent first), removable; Save value and Save as default are per item type', async () => {
  const local = { ...baseItem, origin: 'local' as const, external_id: null };
  vi.mocked(getForm).mockResolvedValue(form({ details: [
    field(def('custom.country', 'choice', 'Country', 'details', ['Denmark', 'Sweden', 'Norway']), null, { saved: ['"Denmark"', '"Sweden"'] }),
    field(def('custom.color_furniture', 'text', 'Color'), stored('custom.color_furniture', 'Oak'), { saved: ['"Walnut"'] }),
  ] }));
  vi.mocked(addSavedValue).mockResolvedValue([]);
  vi.mocked(removeSavedValue).mockResolvedValue([]);
  vi.mocked(setFieldDefault).mockResolvedValue('Oak');
  render(<FormDialog open onOpenChange={vi.fn()} item={local} photos={[]} onItemChanged={vi.fn()} />);
  const picks = await screen.findByRole('list', { name: 'Saved values of Country' });
  expect(within(picks).getAllByRole('button', { name: /^Use / }).map(b => b.textContent)).toEqual(['Denmark', 'Sweden']);
  fireEvent.click(within(picks).getByRole('button', { name: 'Use Sweden' }));
  await waitFor(() => expect(setAttributeValue).toHaveBeenCalledExactlyOnceWith(ITEM_ID, 'custom.country', 'Sweden'));
  fireEvent.click(within(picks).getByRole('button', { name: 'Remove saved Denmark' }));
  await waitFor(() => expect(removeSavedValue).toHaveBeenCalledExactlyOnceWith('Sofa', 'custom.country', 'Denmark'));
  fireEvent.click(screen.getByRole('button', { name: 'Save Color value' }));
  await waitFor(() => expect(addSavedValue).toHaveBeenCalledExactlyOnceWith('Sofa', 'custom.color_furniture', 'Oak'));
  fireEvent.click(screen.getByRole('button', { name: 'Save Color as default' }));
  await waitFor(() => expect(setFieldDefault).toHaveBeenCalledExactlyOnceWith('Sofa', 'custom.color_furniture', 'Oak'));
});

it('C2(c) Fill defaults asks the backend to fill the empty fields and shows the result', async () => {
  const local = { ...baseItem, origin: 'local' as const, external_id: null };
  const before = form({ details: [
    field(def('custom.designer', 'text', 'Designer'), null, { default: '"Wegner"' }),
    field(def('custom.country', 'text', 'Country'), stored('custom.country', 'Sweden'), { default: '"Denmark"' }),
  ] });
  const after = form({ details: [
    field(def('custom.designer', 'text', 'Designer'), stored('custom.designer', 'Wegner', 'default'), { default: '"Wegner"' }),
    field(def('custom.country', 'text', 'Country'), stored('custom.country', 'Sweden'), { default: '"Denmark"' }),
  ] });
  vi.mocked(fillDefaults).mockResolvedValue(1);
  vi.mocked(getForm).mockImplementation(async () => (vi.mocked(fillDefaults).mock.calls.length ? after : before));
  render(<FormDialog open onOpenChange={vi.fn()} item={local} photos={[]} onItemChanged={vi.fn()} />);
  expect(await screen.findByLabelText('Designer')).toHaveValue('');
  fireEvent.click(screen.getByRole('button', { name: /Fill defaults/ }));
  await waitFor(() => expect(fillDefaults).toHaveBeenCalledExactlyOnceWith(ITEM_ID));
  await waitFor(() => expect(screen.getByLabelText('Designer')).toHaveValue('Wegner'));
  expect(screen.getByLabelText('Country')).toHaveValue('Sweden');
  expect(within(fieldBox('Designer')).getByText('Default')).toBeInTheDocument();
});

it('C3(a) a new item without a type opens with an empty picker that is not focused, and nothing is saved until the photographer confirms', async () => {
  const untyped = { ...baseItem, item_type: '', origin: 'local' as const, external_id: null };
  vi.mocked(getForm).mockResolvedValue(form({ details: [field(def('custom.color_furniture', 'text', 'Color'), null)] }));
  render(<FormDialog open onOpenChange={vi.fn()} item={untyped} photos={[]} onItemChanged={vi.fn()} />);
  await screen.findByLabelText('Color');
  await waitFor(() => expect(listItemTypes).toHaveBeenCalled());
  const picker = screen.getByLabelText('Item type');
  expect(picker).toHaveValue('');
  expect(picker).not.toHaveFocus();
  fireEvent.keyDown(document.activeElement ?? document.body, { key: 'ArrowDown' });
  fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Enter' });
  // A suggestion shown in the field is not a choice.
  fireEvent.change(picker, { target: { value: 'Sofa' } });
  await new Promise(resolve => setTimeout(resolve, 50));
  expect(setItemType).not.toHaveBeenCalled();
  fireEvent.blur(picker);
  await waitFor(() => expect(setItemType).toHaveBeenCalledExactlyOnceWith(ITEM_ID, 'Sofa'));
});

it('C4(a) in a batch the next form opens on the step of the previous one; a single form opens on Details', async () => {
  const items = [1, 2, 3].map(n => ({ ...baseItem, id: n, sku: `SOFA-${n}`, title: `Sofa ${n}` }));
  vi.mocked(getSessionView).mockImplementation(async id => ({ session: { id }, item: items[id - 1], photos: [] }) as unknown as SessionView);
  vi.mocked(readShopValues).mockImplementation(async ids => ids.map(id => read('read', { item_id: id, title: `Sofa ${id}` })));
  vi.mocked(getForm).mockImplementation(async id => ({ ...form({ details: [field(def('custom.color_furniture', 'text', 'Color'), null)], product: [field(def('product.title', 'text', 'Title', 'product'), stored('product.title', `Sofa ${id}`))] }), item_id: id }));
  render(<BatchQueue ids={[1, 2, 3]} onClose={vi.fn()} />);
  fireEvent.click(await screen.findByRole('tab', { name: /Product/ }));
  expect(await screen.findByLabelText('Title')).toHaveValue('Sofa 1');
  // The batch read every linked product once, together.
  expect(readShopValues).toHaveBeenCalledExactlyOnceWith([1, 2, 3]);
  fireEvent.click(screen.getByRole('button', { name: 'Confirm & next' }));
  await waitFor(() => expect(screen.getByLabelText('Title')).toHaveValue('Sofa 2'));
  expect(screen.getByRole('tab', { name: /Product/ })).toHaveAttribute('aria-selected', 'true');
  expect(screen.getByRole('tab', { name: /Details/ })).toHaveAttribute('aria-selected', 'false');
  cleanup();
  render(<FormDialog open onOpenChange={vi.fn()} item={items[0]} photos={[]} onItemChanged={vi.fn()} />);
  expect(await screen.findByRole('tab', { name: /Details/ })).toHaveAttribute('aria-selected', 'true');
  expect(await screen.findByLabelText('Color')).toBeInTheDocument();
});
