import '@testing-library/jest-dom/vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { AttributeValue, Definition, Form, FormField } from '@/services/attributes';
import type { Item } from '@/services/items';
import type { ResearchStatus } from '@/services/research';

vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn() }));
vi.mock('@/services/attributes', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/attributes')>()),
  getForm: vi.fn(),
  listDefinitions: vi.fn(),
  setAttributeValue: vi.fn(),
  clearAttribute: vi.fn(),
  acceptAllSuggestions: vi.fn(),
  acceptSuggestion: vi.fn(),
  resolveConflict: vi.fn(),
}));
vi.mock('@/services/research', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/research')>()),
  researchStatus: vi.fn(),
  runResearch: vi.fn(),
}));
vi.mock('@/services/events', () => ({ onResearchChanged: vi.fn(async () => () => undefined), onSessionsChanged: vi.fn(async () => () => undefined) }));
vi.mock('@/features/form/ItemTypePicker', () => ({ ItemTypePicker: () => null, useItemTypes: () => ({ types: [], failed: false }) }));
vi.mock('@/features/form/StockFields', () => ({ StockFields: () => null }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { FormDialog, type FormFieldJump } from '@/features/form/FormDialog';
import { getForm } from '@/services/attributes';
import { researchStatus } from '@/services/research';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};
Element.prototype.scrollIntoView ??= function scrollIntoView() {};
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;

const ITEM: Item = { id: 7, sku: 'BEYO-7', folder_key: 'BEYO-7', barcode: null, item_type: 'Chair', title: 'Oak chair', origin: 'local', external_id: null, created_at: 0, updated_at: 0 };

function def(key: string, label: string, group: Definition['group']): Definition {
  return { integration: 'shopify', key, label, core_type: 'text', choices: [], validations: '{}', group, description: null, writable: true };
}
function field(definition: Definition, value: AttributeValue | null = null): FormField {
  return { definition, value, is_default: false, is_severity: false, selected: null, default: null, saved: [] };
}
function suggestion(key: string, value: string): AttributeValue {
  return { item_id: ITEM.id, key, value: JSON.stringify(value), valid: true, provenance: 'ai', ai_confidence: 'high', ai_status: 'suggested', ai_reason: null, ai_sources: null, ai_prior: null, updated_at: 1, conflict: false };
}
function status(): ResearchStatus {
  return { reason: null, run: null, steps: [], fields_filled: 0, to_review: 0, can_run: true, wait_ends_at: null, queue: null } as ResearchStatus;
}

/** Details: Era, Designer (with `designerValue`); Condition: Wear, the condition report text; Product: Model. */
function form(designerValue: AttributeValue | null = null): Form {
  const g = (group: Definition['group'], fields: FormField[]) => ({ group, fields, filled: fields.filter(f => f.value).length, total: fields.length });
  return {
    item_id: ITEM.id,
    item_type_from_integration: false,
    groups: [
      g('details', [field(def('custom.era', 'Era', 'details')), field(def('custom.designer', 'Designer', 'details'), designerValue)]),
      g('condition', [field(def('custom.wear', 'Wear', 'condition')), field(def('custom.condition_report_description', 'Condition text', 'condition'))]),
      g('product', [field(def('custom.model', 'Model', 'product'))]),
    ],
  };
}

const renderForm = (open: boolean, jump: FormFieldJump | null) => (
  <FormDialog open={open} onOpenChange={() => undefined} item={ITEM} photos={[]} sessionId={4} reviewFirst field={jump} onItemChanged={() => undefined} />
);

let scrolled: ReturnType<typeof vi.spyOn>;
/** The fields whose box was scrolled to (MetafieldInput's reveal). */
function revealed(): string[] {
  return (scrolled.mock.contexts as Element[]).map(el => el.getAttribute?.('data-field')).filter((key): key is string => Boolean(key));
}

beforeEach(() => {
  scrolled = vi.spyOn(Element.prototype, 'scrollIntoView');
  vi.mocked(researchStatus).mockResolvedValue(status());
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  scrolled.mockRestore();
});

it('c4_c12a_opened_at_a_details_field_shows_details_and_reveals_it', async () => {
  vi.mocked(getForm).mockResolvedValue(form());
  render(renderForm(true, { key: 'custom.designer', n: 1 }));
  expect(await screen.findByRole('region', { name: 'Details fields' })).toBeInTheDocument();
  await waitFor(() => expect(revealed()).toEqual(['custom.designer']));
});

it('c4_c12b_opened_at_a_condition_field_then_reopened_at_an_unknown_one', async () => {
  vi.mocked(getForm).mockResolvedValue(form());
  const { rerender } = render(renderForm(true, { key: 'custom.condition_report_description', n: 1 }));
  expect(await screen.findByRole('region', { name: 'Condition fields' })).toBeInTheDocument();
  await waitFor(() => expect(revealed()).toEqual(['custom.condition_report_description']));
  rerender(renderForm(false, null));
  await waitFor(() => expect(screen.queryByRole('region', { name: 'Condition fields' })).toBeNull());
  scrolled.mockClear();
  rerender(renderForm(true, { key: 'custom.unknown', n: 1 }));
  expect(await screen.findByRole('region', { name: 'Details fields' })).toBeInTheDocument();
  await screen.findByLabelText('Designer');
  await new Promise(resolve => setTimeout(resolve, 20));
  expect(screen.getByRole('region', { name: 'Details fields' })).toBeInTheDocument();
  expect(revealed()).toEqual([]);
});

it('c4_c12c_with_a_waiting_suggestion_the_field_wins_over_the_ai_review_landing', async () => {
  vi.mocked(getForm).mockResolvedValue(form(suggestion('custom.designer', 'Hans J. Wegner')));
  render(renderForm(true, { key: 'custom.designer', n: 1 }));
  await waitFor(() => expect(revealed()).toEqual(['custom.designer']));
  await new Promise(resolve => setTimeout(resolve, 20));
  expect(screen.getByRole('region', { name: 'Details fields' })).toBeInTheDocument();
  expect(screen.queryByRole('region', { name: 'AI review' })).toBeNull();
});
