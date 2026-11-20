import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { AttributeValue, Definition, Form, FormField, Group } from '@/services/attributes';
import type { Item } from '@/services/items';

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
import { getForm, setAttributeValue } from '@/services/attributes';
import { listItemTypes } from '@/services/items';
import { itemStockGet, stockLocations } from '@/services/stock';

globalThis.ResizeObserver ??= class { observe() {} unobserve() {} disconnect() {} };
Element.prototype.scrollIntoView ??= function scrollIntoView() {};
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;

const ITEM_ID = 7;
const baseItem: Item = { id: ITEM_ID, sku: 'SOFA-1', folder_key: 'SOFA-1', barcode: null, item_type: 'Sofa', title: 'Velvet sofa', origin: 'linked', external_id: 'gid://shopify/Product/1', created_at: 0, updated_at: 0 };
const def = (key: string, core_type: string, label: string, group: Group = 'details', choices: string[] = []): Definition => ({ integration: key.startsWith('product.') ? 'core' : 'shopify', key, label, core_type, choices, validations: '{}', group, description: null, writable: true });
const stored = (key: string, value: unknown, provenance = 'human'): AttributeValue => ({ item_id: ITEM_ID, key, value: JSON.stringify(value), valid: true, provenance, ai_confidence: null, ai_status: null, ai_reason: null, ai_sources: null, ai_prior: null, updated_at: 1, conflict: false });
const field = (definition: Definition, value: AttributeValue | null, extra: Partial<FormField> = {}): FormField => ({ definition, value, is_default: value?.provenance === 'default', is_severity: false, selected: null, default: null, saved: [], ...extra });
const form = (groups: Partial<Record<Group, FormField[]>>, itemTypeFromIntegration = false): Form => ({ item_id: ITEM_ID, item_type_from_integration: itemTypeFromIntegration, groups: (['details', 'condition', 'product'] as Group[]).map(group => ({ group, fields: groups[group] ?? [], filled: 0, total: groups[group]?.length ?? 0 })) });

beforeEach(() => {
  vi.mocked(setAttributeValue).mockImplementation(async (_id, key, value) => stored(key, value));
  vi.mocked(listItemTypes).mockResolvedValue(['Chair', 'Sofa']);
  vi.mocked(stockLocations).mockResolvedValue({ locations: [], missing_access: null });
  vi.mocked(itemStockGet).mockResolvedValue({ new_item: false, choice: null });
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

// A26 (owner 2026-11-20): the description's "AI agent writes it / Write manually" switch predates the publication
// writer. The description is a plain text box like the title, also when empty; the writer's texts arrive in AI review.
it('A26 an empty description is an open text box, with no AI-agent switch, and what is typed is saved', async () => {
  const local = { ...baseItem, origin: 'local' as const, external_id: null };
  vi.mocked(getForm).mockResolvedValue(form({ product: [
    field(def('product.title', 'text', 'Title', 'product'), null),
    field(def('product.description', 'rich_text', 'Description', 'product'), null),
  ] }));
  render(<FormDialog open onOpenChange={vi.fn()} item={local} photos={[]} onItemChanged={vi.fn()} />);
  fireEvent.click(await screen.findByRole('tab', { name: /Product/ }));
  const description = await screen.findByLabelText('Description');
  expect(description.tagName).toBe('TEXTAREA');
  expect(description).toHaveValue('');
  expect(screen.queryByText(/AI agent writes it/)).toBeNull();
  expect(screen.queryByText(/Write manually/)).toBeNull();
  fireEvent.change(description, { target: { value: 'Deep seat.' } });
  fireEvent.blur(description);
  await waitFor(() => expect(setAttributeValue).toHaveBeenCalledExactlyOnceWith(ITEM_ID, 'product.description', 'Deep seat.'));
});
