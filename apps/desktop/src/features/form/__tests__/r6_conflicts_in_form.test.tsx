import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { AttributeValue, Definition, Form, FormField } from '@/services/attributes';
import type { Item } from '@/services/items';

vi.mock('@/services/attributes', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/attributes')>()),
  getForm: vi.fn(),
  setAttributeValue: vi.fn(),
  clearAttribute: vi.fn(),
  acceptSuggestion: vi.fn(),
  resolveConflict: vi.fn(),
}));
vi.mock('@/services/research', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/research')>()),
  researchStatus: vi.fn(async () => { throw Object.assign(new Error('Research is not available'), { code: 'research_unavailable' }); }),
}));
vi.mock('@/services/events', () => ({ onResearchChanged: vi.fn(async () => () => undefined) }));
vi.mock('@/features/form/ItemTypePicker', () => ({ ItemTypePicker: () => null, useItemTypes: () => ({ types: [], failed: false }) }));
vi.mock('@/features/form/StockFields', () => ({ StockFields: () => null }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { FormDialog } from '@/features/form/FormDialog';
import { SAVE_DEBOUNCE_MS } from '@/features/form/useFormState';
import { getForm, resolveConflict, setAttributeValue } from '@/services/attributes';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

const ITEM: Item = { id: 7, sku: 'VF-0042', folder_key: 'VF-0042', barcode: null, item_type: 'Chair', title: 'Oak chair', origin: 'local', external_id: null, created_at: 0, updated_at: 0 };

function def(key: string, label: string): Definition {
  return { integration: 'shopify', key, label, core_type: 'text', choices: [], validations: '{}', group: 'details', description: null, writable: true };
}
function ai(key: string, value: string, extra: Partial<AttributeValue> = {}): AttributeValue {
  return { item_id: ITEM.id, key, value: JSON.stringify(value), valid: true, provenance: 'ai', ai_confidence: 'medium', ai_status: 'suggested', ai_reason: null, ai_sources: null, ai_prior: null, updated_at: 1, conflict: false, ...extra };
}
function field(definition: Definition, value: AttributeValue | null): FormField {
  return { definition, value, is_default: false, is_severity: false, selected: null, default: null, saved: [] };
}
function form(fields: FormField[]): Form {
  return { item_id: ITEM.id, item_type_from_integration: false, groups: [{ group: 'details', fields, filled: fields.length, total: fields.length }, { group: 'condition', fields: [], filled: 0, total: 0 }, { group: 'product', fields: [], filled: 0, total: 0 }] };
}
const designerConflict = () => field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner', { ai_prior: { value: '"Unknown"', valid: true, provenance: 'default' }, conflict: true }));

function renderForm() {
  return render(<FormDialog open onOpenChange={() => undefined} item={ITEM} photos={[]} sessionId={4} onItemChanged={() => undefined} />);
}

beforeEach(() => {
  vi.mocked(resolveConflict).mockResolvedValue(ai('designer', 'Walnut', { provenance: 'human', ai_status: null, ai_confidence: null }));
});
afterEach(() => { cleanup(); vi.clearAllMocks(); vi.useRealTimers(); });

it('a20_c5a_conflict_field_is_marked_offers_the_three_actions_and_edit_saves_once_as_typed', async () => {
  vi.mocked(getForm).mockResolvedValue(form([designerConflict()]));
  renderForm();
  const container = await waitFor(() => {
    const element = document.querySelector('[data-field="designer"]');
    expect(element).not.toBeNull();
    return element as HTMLElement;
  });
  expect(container).toHaveAttribute('data-conflict');
  fireEvent.click(screen.getByRole('button', { name: 'AI suggestion, medium confidence' }));
  const popover = await screen.findByLabelText('AI suggestion for Designer');
  expect(popover).toHaveTextContent('Replaces Unknown · preset default');
  expect(screen.getByRole('button', { name: 'Keep previous' })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Use suggestion' })).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Looks right' })).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Clear' })).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Edit' }));
  const input = screen.getByLabelText('Designer') as HTMLInputElement;
  await waitFor(() => expect(input).toHaveFocus());
  expect(input).toHaveValue('Hans J. Wegner');
  // Typing never saves on the debounce: on a held publish item the first save of a pending key resolves it (N1).
  for (const prefix of ['W', 'Wa', 'Wal', 'Waln', 'Walnu', 'Walnut']) fireEvent.change(input, { target: { value: prefix } });
  await act(async () => { await new Promise(resolve => setTimeout(resolve, SAVE_DEBOUNCE_MS + 100)); });
  expect(resolveConflict).not.toHaveBeenCalled();
  expect(setAttributeValue).not.toHaveBeenCalled();
  fireEvent.keyDown(input, { key: 'Enter' });
  await waitFor(() => expect(resolveConflict).toHaveBeenCalledWith(ITEM.id, 'designer', 'edit', 'Walnut'));
  fireEvent.blur(input);
  await act(async () => { await new Promise(resolve => setTimeout(resolve, SAVE_DEBOUNCE_MS + 100)); });
  expect(resolveConflict).toHaveBeenCalledTimes(1);
  expect(setAttributeValue).not.toHaveBeenCalled();
});

it('a20_c5b_header_counts_plain_suggestions_and_conflicts', async () => {
  vi.mocked(getForm).mockResolvedValue(form([
    field(def('maker', 'Maker'), ai('maker', 'Carl Hansen')),
    field(def('model', 'Model'), ai('model', 'CH24', { ai_confidence: 'low' })),
    designerConflict(),
    field(def('era', 'Era'), ai('era', '1950s', { ai_status: 'accepted' })),
  ]));
  renderForm();
  expect(await screen.findByRole('button', { name: '3 to review' })).toBeInTheDocument();
  expect(document.querySelectorAll('[data-conflict]')).toHaveLength(1);
});
