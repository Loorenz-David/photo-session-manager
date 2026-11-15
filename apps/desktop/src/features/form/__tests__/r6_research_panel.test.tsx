import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { AttributeValue, Definition, Form, FormField } from '@/services/attributes';
import type { Item } from '@/services/items';
import type { ResearchStatus, ResearchStep } from '@/services/research';

vi.mock('@/services/attributes', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/attributes')>()),
  getForm: vi.fn(),
  listDefinitions: vi.fn(),
  setAttributeValue: vi.fn(),
  clearAttribute: vi.fn(),
  acceptAllSuggestions: vi.fn(),
  acceptSuggestion: vi.fn(),
}));
vi.mock('@/services/research', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/research')>()),
  researchStatus: vi.fn(),
  runResearch: vi.fn(),
}));
vi.mock('@/services/events', () => ({ onResearchChanged: vi.fn() }));
vi.mock('@/features/form/ItemTypePicker', () => ({ ItemTypePicker: () => null, useItemTypes: () => ({ types: [], failed: false }) }));
vi.mock('@/features/form/StockFields', () => ({ StockFields: () => null }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { FormDialog } from '@/features/form/FormDialog';
import { ResearchPanel } from '@/features/form/ResearchPanel';
import { acceptSuggestion, getForm } from '@/services/attributes';
import { onResearchChanged } from '@/services/events';
import { researchStatus, runResearch } from '@/services/research';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};
Element.prototype.scrollIntoView ??= function scrollIntoView() {};
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;

const SESSION = 4;
const ITEM: Item = { id: 7, sku: 'VF-0042', folder_key: 'VF-0042', barcode: null, item_type: 'Chair', title: 'Oak chair', origin: 'local', external_id: null, created_at: 0, updated_at: 0 };

function def(key: string, label = key): Definition {
  return { integration: 'shopify', key, label, core_type: 'text', choices: [], validations: '{}', group: 'details', description: null, writable: true };
}
function ai(key: string, value: string, extra: Partial<AttributeValue> = {}): AttributeValue {
  return { item_id: ITEM.id, key, value: JSON.stringify(value), valid: true, provenance: 'ai', ai_confidence: 'high', ai_status: 'suggested', ai_reason: null, ai_sources: null, ai_prior: null, updated_at: 1, conflict: false, ...extra };
}
function field(definition: Definition, value: AttributeValue | null = null): FormField {
  return { definition, value, is_default: false, is_severity: false, selected: null, default: null, saved: [] };
}
function form(fields: FormField[]): Form {
  return { item_id: ITEM.id, item_type_from_integration: false, groups: [{ group: 'details', fields, filled: fields.length, total: fields.length }, { group: 'condition', fields: [], filled: 0, total: 0 }, { group: 'product', fields: [], filled: 0, total: 0 }] };
}
function step(seq: number, kind: string, extra: Partial<ResearchStep> = {}): ResearchStep {
  return { seq, stage: 'observe', kind, summary: `summary ${seq}`, detail: `detail ${seq}`, source_url: null, source_title: null, photo_ids: [], field_keys: [], ...extra };
}
function status(extra: Partial<ResearchStatus> = {}): ResearchStatus {
  return { reason: null, run: null, steps: [], fields_filled: 0, to_review: 0, can_run: false, wait_ends_at: null, ...extra };
}
const run = (state: string, extra: Partial<NonNullable<ResearchStatus['run']>> = {}) => ({ id: 1, state, trigger: 'arranged', stage_reached: null, error: null, ...extra });

const panel = (s: ResearchStatus | null, props: Partial<Parameters<typeof ResearchPanel>[0]> = {}) =>
  render(<ResearchPanel status={s} fields={[]} focusKey={null} onFocus={() => undefined} onRun={async () => undefined} onAcceptAll={async () => undefined} onClose={() => undefined} {...props} />);

let researchHandler: ((sessionId: number) => void) | null = null;

beforeEach(() => {
  researchHandler = null;
  vi.mocked(onResearchChanged).mockImplementation(async handler => {
    researchHandler = handler;
    return () => undefined;
  });
  vi.mocked(runResearch).mockResolvedValue(undefined);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); vi.useRealTimers(); });

function renderForm(photos: Parameters<typeof FormDialog>[0]['photos'] = []) {
  return render(<FormDialog open onOpenChange={() => undefined} item={ITEM} photos={photos} sessionId={SESSION} onItemChanged={() => undefined} />);
}

it('a18_c2a_steps_in_seq_order_with_verbs_and_read_link', () => {
  panel(status({
    run: run('running'),
    steps: [
      step(3, 'read', { source_url: 'https://www.pamono.com/wegner-ch24', source_title: 'Pamono · CH24' }),
      step(1, 'looked'),
      step(5, 'concluded'),
      step(2, 'searched'),
      step(4, 'compared'),
    ],
  }));
  const items = within(screen.getByRole('list', { name: 'Research steps' })).getAllByRole('listitem');
  expect(items.map(item => item.getAttribute('data-kind'))).toEqual(['looked', 'searched', 'read', 'compared', 'concluded']);
  expect(items.map(item => item.textContent?.split(' ')[0])).toEqual(['Looked', 'Searched', 'Read', 'Compared', 'Concluded']);
  expect(within(items[2]).getByRole('link', { name: 'Pamono · CH24' })).toHaveAttribute('href', 'https://www.pamono.com/wegner-ch24');
  expect(within(items[0]).queryByRole('link')).not.toBeInTheDocument();
});

it('a18_c2b_running_run_appends_the_new_step_on_research_changed', async () => {
  vi.mocked(getForm).mockResolvedValue(form([field(def('designer', 'Designer'))]));
  vi.mocked(researchStatus).mockResolvedValue(status({ run: run('running'), steps: [step(1, 'looked')] }));
  renderForm();
  fireEvent.click(await screen.findByRole('button', { name: 'Research' }));
  const steps = await screen.findByRole('list', { name: 'Research steps' });
  await waitFor(() => expect(within(steps).getAllByRole('listitem')).toHaveLength(1));
  expect(within(screen.getByRole('complementary', { name: 'Research' })).getByText('Researching…')).toHaveAttribute('data-live');
  vi.mocked(researchStatus).mockResolvedValue(status({ run: run('running'), steps: [step(1, 'looked'), step(2, 'searched', { summary: 'for matching listings' })] }));
  await waitFor(() => expect(researchHandler).not.toBeNull());
  act(() => researchHandler!(SESSION));
  await waitFor(() => expect(within(steps).getAllByRole('listitem')).toHaveLength(2));
  expect(within(steps).getAllByRole('listitem')[1]).toHaveTextContent('Searched for matching listings');
  expect(within(screen.getByRole('complementary', { name: 'Research' })).getByText('Researching…')).toBeInTheDocument();
});

it('a18_c2c_finished_line_counts_filled_and_to_review', () => {
  panel(status({ run: run('finished'), fields_filled: 3, to_review: 2 }));
  expect(screen.getByText('Finished · 3 fields filled · 2 to review')).toBeInTheDocument();
  expect(document.querySelector('[data-live]')).toBeNull();
  cleanup();
  // A follow-up that confirmed the values already there wrote nothing: the line does not read as "0 fields filled".
  panel(status({ run: run('finished'), fields_filled: 0, to_review: 3 }));
  expect(screen.getByText('Finished · no fields changed · 3 to review')).toBeInTheDocument();
  cleanup();
  panel(status({ run: run('stopped_at_cap'), fields_filled: 1, to_review: 1 }));
  expect(screen.getByText('Finished · 1 field filled · 1 to review · stopped at the cost limit')).toBeInTheDocument();
});

it('a18_c2d_failed_run_shows_stage_error_and_run_again', () => {
  panel(status({ run: run('failed', { stage_reached: 'read', error: 'The page reader refused the key' }), can_run: true }));
  expect(screen.getByText('Stopped at read: The page reader refused the key')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Run again' })).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Research' })).not.toBeInTheDocument();
});

it('a18_c3a_research_button_starts_a_run_for_the_session', async () => {
  vi.mocked(getForm).mockResolvedValue(form([field(def('designer', 'Designer'))]));
  vi.mocked(researchStatus).mockResolvedValue(status({ can_run: true, reason: { code: 'AutoOff', sentence: '' } }));
  renderForm();
  fireEvent.click(await screen.findByRole('button', { name: 'Research' }));
  const aside = await screen.findByRole('complementary', { name: 'Research' });
  fireEvent.click(await within(aside).findByRole('button', { name: 'Research' }));
  await waitFor(() => expect(runResearch).toHaveBeenCalledWith(SESSION));
  expect(runResearch).toHaveBeenCalledTimes(1);
});

it('a18_c3b_reason_sentences_and_no_button_at_the_day_limit', () => {
  panel(status({ reason: { code: 'DayLimit', sentence: 'Day limit reached' }, run: run('queued'), can_run: false }));
  expect(screen.getByText('Day limit reached')).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Research' })).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Run again' })).not.toBeInTheDocument();
  cleanup();
  panel(status({ reason: { code: 'WaitingForEditor', sentence: 'Waiting for Capture One' }, run: run('queued') }));
  expect(screen.getByText('Waiting for Capture One')).toBeInTheDocument();
  cleanup();
  // MC-A6: the sentence is said when the wait for Capture One's previews ends, not before (R4 review).
  vi.useFakeTimers();
  panel(status({ reason: { code: 'WaitingForEditor', sentence: 'Waiting for Capture One' }, run: run('queued'), wait_ends_at: Date.now() + 60_000 }));
  expect(screen.queryByText('Waiting for Capture One')).not.toBeInTheDocument();
  act(() => { vi.advanceTimersByTime(60_000); });
  expect(screen.getByText('Waiting for Capture One')).toBeInTheDocument();
});

it('a18_c4a_focus_from_a_field_popover_dims_other_steps', async () => {
  vi.mocked(getForm).mockResolvedValue(form([field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner'))]));
  vi.mocked(researchStatus).mockResolvedValue(status({
    run: run('finished'),
    steps: [step(1, 'looked'), step(2, 'read', { field_keys: ['designer', 'maker'] }), step(3, 'concluded', { field_keys: ['maker'] })],
  }));
  renderForm();
  fireEvent.click(await screen.findByRole('button', { name: 'AI suggestion, high confidence' }));
  fireEvent.click(await screen.findByRole('button', { name: 'See research' }));
  const steps = await screen.findByRole('list', { name: 'Research steps' });
  await waitFor(() => expect(within(steps).getAllByRole('listitem')).toHaveLength(3));
  const dimmed = () => within(steps).getAllByRole('listitem').map(item => item.hasAttribute('data-dimmed'));
  expect(dimmed()).toEqual([true, false, true]);
  fireEvent.click(screen.getByRole('button', { name: 'Focused on Designer ×' }));
  expect(dimmed()).toEqual([false, false, false]);
});

it('a18_c5a_header_to_review_opens_the_panel_and_hides_at_zero', async () => {
  vi.mocked(getForm).mockResolvedValue(form([
    field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner')),
    field(def('maker', 'Maker'), ai('maker', 'Carl Hansen', { ai_confidence: 'low' })),
    field(def('model', 'Model'), ai('model', 'CH24', { ai_status: 'accepted' })),
  ]));
  vi.mocked(researchStatus).mockResolvedValue(status({ run: run('finished') }));
  renderForm();
  const label = await screen.findByRole('button', { name: '2 to review' });
  expect(screen.queryByRole('complementary', { name: 'Research' })).not.toBeInTheDocument();
  fireEvent.click(label);
  expect(await screen.findByRole('complementary', { name: 'Research' })).toBeInTheDocument();
  vi.mocked(getForm).mockResolvedValue(form([
    field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner', { ai_status: 'accepted' })),
    field(def('maker', 'Maker'), ai('maker', 'Carl Hansen', { ai_status: 'accepted' })),
    field(def('model', 'Model'), ai('model', 'CH24', { ai_status: 'accepted' })),
  ]));
  vi.mocked(acceptSuggestion).mockResolvedValue(ai('designer', 'Hans J. Wegner', { ai_status: 'accepted' }));
  fireEvent.click(screen.getAllByRole('button', { name: /AI suggestion, high confidence|AI suggestion, low confidence/ })[0]);
  fireEvent.click(await screen.findByRole('button', { name: 'Looks right' }));
  await waitFor(() => expect(screen.queryByRole('button', { name: /to review$/ })).not.toBeInTheDocument());
});
