import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { AttributeValue, Definition, Form, FormField } from '@/services/attributes';
import type { Item } from '@/services/items';
import type { ResearchStatus } from '@/services/research';
import type { WriterStatus } from '@/services/writer';

// W3·U3 rows W3·C8 (the writer block in AI review) and W3·C9 (the writer's text rows). Boundary: the form's AI review
// tab rendered with `services/writer`, `services/research` and `services/attributes` mocked; the outcome is what is
// shown and the service calls made.

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
vi.mock('@/services/writer', () => ({
  writerStatus: vi.fn(),
  writerRequest: vi.fn(),
  writerFindingAnswer: vi.fn(),
  writerFindingDismiss: vi.fn(),
  writerNote: vi.fn(async () => { throw new Error('no note'); }),
  writerLookAgain: vi.fn(),
}));
vi.mock('@/services/events', () => ({ onResearchChanged: vi.fn(async () => () => undefined), onSessionsChanged: vi.fn(async () => () => undefined) }));
vi.mock('@/features/form/ItemTypePicker', () => ({ ItemTypePicker: () => null, useItemTypes: () => ({ types: [], failed: false }) }));
vi.mock('@/features/form/StockFields', () => ({ StockFields: () => null }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { FormDialog } from '@/features/form/FormDialog';
import { acceptSuggestion, getForm } from '@/services/attributes';
import { researchStatus } from '@/services/research';
import { writerFindingAnswer, writerFindingDismiss, writerStatus } from '@/services/writer';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};
Element.prototype.scrollIntoView ??= function scrollIntoView() {};
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;

const ITEM: Item = { id: 7, sku: 'VF-0042', folder_key: 'VF-0042', barcode: null, item_type: 'Chair', title: 'Shop chair', origin: 'local', external_id: null, created_at: 0, updated_at: 0 };
const SESSION = 4;
const QUESTION = 'Is the frame teak or oak?';

function def(key: string, label: string, group: Definition['group'], core_type: Definition['core_type'] = 'text'): Definition {
  return { integration: 'shopify', key, label, core_type, choices: [], validations: '{}', group, description: null, writable: true };
}
function ai(key: string, value: string, extra: Partial<AttributeValue> = {}): AttributeValue {
  return { item_id: ITEM.id, key, value: JSON.stringify(value), valid: true, provenance: 'ai', ai_confidence: 'medium', ai_status: 'suggested', ai_reason: null, ai_sources: [], ai_prior: null, updated_at: 1, conflict: false, ...extra };
}
function field(definition: Definition, value: AttributeValue | null = null): FormField {
  return { definition, value, is_default: false, is_severity: false, selected: null, default: null, saved: [] };
}
const prior = (value: string, provenance: string) => ({ value: JSON.stringify(value), provenance }) as AttributeValue['ai_prior'];
function form(details: FormField[], condition: FormField[] = [], product: FormField[] = []): Form {
  const g = (group: Definition['group'], fields: FormField[]) => ({ group, fields, filled: fields.filter(f => f.value).length, total: fields.length });
  return { item_id: ITEM.id, item_type_from_integration: false, groups: [g('details', details), g('condition', condition), g('product', product)] };
}
function research(): ResearchStatus {
  return { reason: null, run: { id: 1, state: 'finished', trigger: 'arranged', stage_reached: null, error: null }, steps: [], fields_filled: 0, to_review: 0, can_run: true, wait_ends_at: null, queue: null };
}
function writer(extra: Partial<WriterStatus> = {}): WriterStatus {
  return {
    reason: null, run: { id: 9, state: 'finished', trigger: 'write', stage_reached: 'write', error: null }, texts: [], attempts: [], regenerated: [],
    findings: [], steps: [], cost_micro_usd: 0, inputs: { fields: 0, note: 0, listings: [] }, stale_main: false, stale_condition: false, can_write: true, finished_at: null, not_offered: [],
    ...extra,
  };
}
const ask = { id: 11, kind: 'question', type: 'question', keys: ['wood'], text_field: '', text: QUESTION, mode: 'ask' };
const note = { id: 12, kind: 'finding', type: 'field_vs_field', keys: ['era'], text_field: '', text: 'The era does not fit the designer', mode: 'note' };

/** A Details suggestion, so the tab has suggestions to show under the writer block. */
const designer = () => field(def('designer', 'Designer', 'details'), ai('designer', 'Børge Mogensen'));

const renderForm = () =>
  render(<FormDialog open onOpenChange={() => undefined} item={ITEM} photos={[]} sessionId={SESSION} onItemChanged={() => undefined} />);

async function openReviewTab() {
  renderForm();
  fireEvent.click(await screen.findByRole('tab', { name: /AI review/ }));
  return screen.findByRole('region', { name: 'AI review' });
}

beforeEach(() => {
  vi.mocked(researchStatus).mockResolvedValue(research());
  vi.mocked(writerStatus).mockResolvedValue(writer());
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('w3_c8a_an_open_question_shows_above_the_suggestions_and_its_answer_is_sent', async () => {
  vi.mocked(getForm).mockResolvedValue(form([designer()]));
  vi.mocked(writerStatus).mockResolvedValue(writer({ findings: [ask] }));
  vi.mocked(writerFindingAnswer).mockResolvedValue({ queued: true, reason: null });
  const region = await openReviewTab();
  const question = await within(region).findByText(QUESTION);
  const suggestions = within(region).getByRole('list', { name: 'Suggestions to confirm' });
  // The question comes before the suggestions in the tab.
  expect(question.compareDocumentPosition(suggestions) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  fireEvent.change(within(region).getByRole('textbox', { name: `Answer: ${QUESTION}` }), { target: { value: 'Teak' } });
  fireEvent.click(within(region).getByRole('button', { name: 'Answer' }));
  await waitFor(() => expect(writerFindingAnswer).toHaveBeenCalledTimes(1));
  expect(writerFindingAnswer).toHaveBeenCalledWith(11, 'Teak');
});

it('w3_c8b_a_dismissed_note_leaves_after_the_re_read', async () => {
  vi.mocked(getForm).mockResolvedValue(form([designer()]));
  let dismissed = false;
  vi.mocked(writerStatus).mockImplementation(async () => writer({ findings: dismissed ? [] : [note] }));
  vi.mocked(writerFindingDismiss).mockImplementation(async () => { dismissed = true; });
  const region = await openReviewTab();
  const shown = await within(region).findByText(note.text);
  fireEvent.click(within(shown.closest('li')!).getByRole('button', { name: 'Dismiss' }));
  await waitFor(() => expect(within(region).queryByText(note.text)).not.toBeInTheDocument());
  expect(writerFindingDismiss).toHaveBeenCalledTimes(1);
  expect(writerFindingDismiss).toHaveBeenCalledWith(12);
});

it('w3_c8c_while_the_writer_writes_the_tab_says_so', async () => {
  vi.mocked(getForm).mockResolvedValue(form([field(def('designer', 'Designer', 'details'), ai('designer', 'Børge Mogensen', { ai_status: 'accepted' }))]));
  vi.mocked(writerStatus).mockResolvedValue(writer({ run: { id: 9, state: 'running', trigger: 'write', stage_reached: null, error: null } }));
  const region = await openReviewTab();
  expect(await within(region).findByText('Writing texts…')).toBeInTheDocument();
  // The tab's empty reason names the writer as the cause.
  expect(within(region).getByText('The writer is writing the texts. They appear here when they are ready.')).toBeInTheDocument();
});

it('w3_c9a_a_description_shows_its_previous_text_and_every_suggested_paragraph_and_accept_accepts_it', async () => {
  const description = field(
    def('product.description', 'Description', 'product', 'rich_text'),
    ai('product.description', 'A dining chair in solid beech.\n\nWe cleaned and oiled the frame.', { conflict: true, ai_prior: prior('Old shop text.', 'integration') }),
  );
  vi.mocked(getForm).mockResolvedValue(form([], [], [description]));
  vi.mocked(acceptSuggestion).mockResolvedValue(description.value!);
  const region = await openReviewTab();
  const row = await within(region).findByRole('listitem', { name: 'Description' });
  expect(within(row).getByText('Old shop text.')).toBeInTheDocument();
  expect(within(row).getByText('A dining chair in solid beech.')).toBeInTheDocument();
  expect(within(row).getByText('We cleaned and oiled the frame.')).toBeInTheDocument();
  fireEvent.click(within(row).getByRole('button', { name: 'Accept' }));
  await waitFor(() => expect(acceptSuggestion).toHaveBeenCalledWith(ITEM.id, 'product.description'));
  expect(acceptSuggestion).toHaveBeenCalledTimes(1);
});

it('w3_c9b_a_flagged_condition_report_shows_low_and_its_check_line', async () => {
  const report = field(
    def('custom.condition_report_description', 'Condition report', 'condition', 'textarea'),
    ai('custom.condition_report_description', 'Good. Light wear on the seat.', {
      ai_confidence: 'low',
      ai_reason: 'From the condition fields.\nFacts: Wear\nCheck: does not name Condition.Stains/Discoloration',
    }),
  );
  vi.mocked(getForm).mockResolvedValue(form([], [report]));
  const region = await openReviewTab();
  const row = await within(region).findByRole('listitem', { name: 'Condition report' });
  expect(within(row).getByText('Low confidence · check')).toBeInTheDocument();
  expect(within(row).getByText('Check: does not name Condition.Stains/Discoloration')).toBeInTheDocument();
});

it('w3_c9c_a_title_suggestion_shows_the_shops_title_and_the_suggestion_with_its_actions', async () => {
  const suggestion = 'Børge Mogensen J39 Chair in Beech, 1960s';
  const title = field(def('product.title', 'Title', 'product'), ai('product.title', suggestion, { conflict: true, ai_prior: prior('Shop chair', 'integration') }));
  vi.mocked(getForm).mockResolvedValue(form([], [], [title]));
  const region = await openReviewTab();
  const row = await within(region).findByRole('listitem', { name: 'Title' });
  const before = row.querySelector('[data-conflict-prior]')!;
  const after = row.querySelector('[data-suggested]')!;
  expect(before).toHaveTextContent('Shop chair');
  expect(after).toHaveTextContent(suggestion);
  expect(before.compareDocumentPosition(after) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  for (const action of ['Accept', 'Keep previous', 'Edit']) expect(within(row).getByRole('button', { name: action })).toBeInTheDocument();
});
