import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { AttributeValue, Definition, Form, FormField, Group } from '@/services/attributes';
import type { Item } from '@/services/items';

vi.mock('@/services/attributes', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/attributes')>()),
  getForm: vi.fn(), listDefinitions: vi.fn(), setAttributeValue: vi.fn(), clearAttribute: vi.fn(),
  acceptAllSuggestions: vi.fn(), updateIdentity: vi.fn(),
}));
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn() } }));

import { FormDialog } from '@/features/form/FormDialog';
import { IdentityEditor } from '@/features/form/IdentityEditor';
import { getForm, setAttributeValue, updateIdentity } from '@/services/attributes';
import { toast } from 'sonner';

globalThis.ResizeObserver ??= class { observe() {} unobserve() {} disconnect() {} };
Element.prototype.scrollIntoView ??= function scrollIntoView() {};
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;

const ITEM_ID = 7;
const item: Item = { id: ITEM_ID, sku: 'VF-0042', folder_key: 'VF-0042', barcode: '4006381333931', item_type: 'Chair', title: 'Oak chair', origin: 'local', external_id: null, created_at: 0, updated_at: 0 };
const def = (key: string, core_type: string, label: string, group: Group = 'details'): Definition => ({ integration: 'shopify', key, label, core_type, choices: [], validations: '{}', group, description: null, writable: true });
const stored = (key: string, value: unknown, extra: Partial<AttributeValue> = {}): AttributeValue => ({ item_id: ITEM_ID, key, value: JSON.stringify(value), valid: true, provenance: 'human', ai_confidence: null, ai_status: null, ai_reason: null, ai_sources: null, updated_at: 1, ...extra });
const field = (definition: Definition, value: AttributeValue | null): FormField => ({ definition, value, is_default: false, is_severity: false, selected: null });
const form = (groups: Partial<Record<Group, FormField[]>>): Form => ({ item_id: ITEM_ID, groups: (['details', 'condition', 'product'] as Group[]).map(group => ({ group, fields: groups[group] ?? [], filled: 0, total: groups[group]?.length ?? 0 })) });
const renderForm = (onOpenChange = vi.fn(), onItemChanged = vi.fn()) => render(<FormDialog open onOpenChange={onOpenChange} item={item} photos={[]} onItemChanged={onItemChanged} />);

beforeEach(() => { vi.mocked(setAttributeValue).mockImplementation(async (_id, key, value) => stored(key, value)); });
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.clearAllMocks(); });

it('C4(a) queues two AI accept clicks and reloads even when the second action fails', async () => {
  const ai = { provenance: 'ai', ai_confidence: 'high', ai_status: 'suggested', ai_reason: 'Seen in photo', ai_sources: [] };
  vi.mocked(getForm).mockResolvedValue(form({ details: [
    field(def('custom.era', 'text', 'Era'), stored('custom.era', '1960s', ai)),
    field(def('custom.maker', 'text', 'Maker'), stored('custom.maker', 'Oak & Co', ai)),
  ] }));
  let resolveA!: (value: AttributeValue) => void;
  let rejectB!: (reason: Error) => void;
  vi.mocked(setAttributeValue)
    .mockReturnValueOnce(new Promise(resolve => { resolveA = resolve; }))
    .mockReturnValueOnce(new Promise((_resolve, reject) => { rejectB = reject; }));
  renderForm();
  const era = await screen.findByLabelText('Era');
  fireEvent.click(within(era.closest('[data-field]') as HTMLElement).getByRole('button', { name: /AI suggestion/ }));
  fireEvent.click(await screen.findByRole('button', { name: 'Looks right' }));
  const maker = screen.getByLabelText('Maker');
  fireEvent.click(within(maker.closest('[data-field]') as HTMLElement).getByRole('button', { name: /AI suggestion/ }));
  fireEvent.click(await screen.findByRole('button', { name: 'Looks right' }));
  expect(setAttributeValue).toHaveBeenCalledTimes(1);
  expect(setAttributeValue).toHaveBeenNthCalledWith(1, ITEM_ID, 'custom.era', '1960s');
  await act(async () => { resolveA(stored('custom.era', '1960s')); });
  await waitFor(() => expect(setAttributeValue).toHaveBeenCalledTimes(2));
  expect(setAttributeValue).toHaveBeenNthCalledWith(2, ITEM_ID, 'custom.maker', 'Oak & Co');
  const beforeFailure = vi.mocked(getForm).mock.calls.length;
  await act(async () => { rejectB(new Error('nope')); });
  await waitFor(() => expect(toast.error).toHaveBeenCalledWith(expect.stringContaining('nope')));
  expect(vi.mocked(getForm).mock.calls.length).toBeGreaterThan(beforeFailure);
});

it('C4(b) seeds invalid decimal text and writes the corrected value on blur', async () => {
  const invalid = form({ details: [field(def('custom.price', 'decimal', 'Price'), stored('custom.price', '12,5', { valid: false }))] });
  const corrected = form({ details: [field(def('custom.price', 'decimal', 'Price'), stored('custom.price', '12.5'))] });
  vi.mocked(getForm).mockImplementation(async () => vi.mocked(setAttributeValue).mock.calls.length ? corrected : invalid);
  renderForm();
  const input = await screen.findByLabelText('Price');
  expect(input).toHaveValue('12,5');
  expect(document.querySelector('[data-invalid]')).toHaveTextContent('Invalid · stored as received: 12,5');
  fireEvent.change(input, { target: { value: '12.5' } });
  fireEvent.blur(input);
  await waitFor(() => expect(setAttributeValue).toHaveBeenCalledExactlyOnceWith(ITEM_ID, 'custom.price', '12.5'));
  await waitFor(() => expect(document.querySelector('[data-invalid]')).toBeNull());
});

it('C4(c) Escape after editing Title saves it and refreshes the workspace before closing', async () => {
  vi.mocked(getForm).mockResolvedValue(form({ product: [field(def('product.title', 'text', 'Title', 'product'), stored('product.title', 'Oak chair'))] }));
  const closed = vi.fn();
  const changed = vi.fn();
  renderForm(closed, changed);
  fireEvent.click(await screen.findByRole('tab', { name: /Product/ }));
  const title = await screen.findByLabelText('Title');
  fireEvent.change(title, { target: { value: 'Oak chair deluxe' } });
  fireEvent.keyDown(title, { key: 'Escape' });
  await waitFor(() => expect(closed).toHaveBeenCalledExactlyOnceWith(false));
  expect(setAttributeValue).toHaveBeenCalledExactlyOnceWith(ITEM_ID, 'product.title', 'Oak chair deluxe');
  expect(changed).toHaveBeenCalledTimes(1);
  expect(changed.mock.invocationCallOrder[0]).toBeLessThan(closed.mock.invocationCallOrder[0]);
});

it('C4(d) clearing barcode sends an empty string and closes the identity editor', async () => {
  const cleared = { ...item, barcode: null };
  vi.mocked(updateIdentity).mockResolvedValue(cleared);
  const saved = vi.fn();
  render(<IdentityEditor item={item} onSaved={saved} />);
  fireEvent.click(screen.getByRole('button', { name: 'Edit SKU and barcode' }));
  const barcode = await screen.findByLabelText('Barcode');
  fireEvent.change(barcode, { target: { value: '' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save' }));
  await waitFor(() => expect(updateIdentity).toHaveBeenCalledExactlyOnceWith(ITEM_ID, null, ''));
  await waitFor(() => expect(saved).toHaveBeenCalledExactlyOnceWith(cleared));
  await waitFor(() => expect(screen.queryByLabelText('Barcode')).toBeNull());
});
