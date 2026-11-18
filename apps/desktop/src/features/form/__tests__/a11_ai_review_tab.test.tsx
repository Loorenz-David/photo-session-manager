import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
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

import { openUrl } from '@tauri-apps/plugin-opener';
import { FormDialog, type FormPosition } from '@/features/form/FormDialog';
import { emptyReason } from '@/features/form/AiReviewTab';
import { acceptSuggestion, clearAttribute, getForm, resolveConflict } from '@/services/attributes';
import { researchStatus } from '@/services/research';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};
Element.prototype.scrollIntoView ??= function scrollIntoView() {};
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;

const ITEM: Item = { id: 7, sku: 'VF-0042', folder_key: 'VF-0042', barcode: null, item_type: 'Chair', title: 'Oak chair', origin: 'local', external_id: null, created_at: 0, updated_at: 0 };

function def(key: string, label: string, group: Definition['group'] = 'details'): Definition {
  return { integration: 'shopify', key, label, core_type: 'text', choices: [], validations: '{}', group, description: null, writable: true };
}
function ai(key: string, value: string, extra: Partial<AttributeValue> = {}): AttributeValue {
  return { item_id: ITEM.id, key, value: JSON.stringify(value), valid: true, provenance: 'ai', ai_confidence: 'high', ai_status: 'suggested', ai_reason: null, ai_sources: null, ai_prior: null, updated_at: 1, conflict: false, ...extra };
}
function human(key: string, value: string): AttributeValue {
  return { ...ai(key, value), provenance: 'human', ai_confidence: null, ai_status: null };
}
function field(definition: Definition, value: AttributeValue | null = null): FormField {
  return { definition, value, is_default: false, is_severity: false, selected: null, default: null, saved: [] };
}
function status(extra: Partial<ResearchStatus> = {}): ResearchStatus {
  return { reason: null, run: null, steps: [], fields_filled: 0, to_review: 0, can_run: true, wait_ends_at: null, queue: null, ...extra };
}
const run = (state: string) => ({ id: 1, state, trigger: 'arranged', stage_reached: null, error: null });
const prior = (value: string, provenance: string) => ({ value: JSON.stringify(value), provenance }) as AttributeValue['ai_prior'];

function form(details: FormField[], condition: FormField[] = [], product: FormField[] = []): Form {
  const g = (group: Definition['group'], fields: FormField[]) => ({ group, fields, filled: fields.filter(f => f.value).length, total: fields.length });
  return { item_id: ITEM.id, item_type_from_integration: false, groups: [g('details', details), g('condition', condition), g('product', product)] };
}

/** Designer (high, with sources and a reason), Maker (conflict over a Shopify value), Wear (low, condition), Price (product, medium). */
function mixed(): Form {
  return form(
    [
      field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner', { ai_reason: 'Two dealers name him', ai_sources: ['https://www.pamono.com/wegner-ch24'] })),
      field(def('maker', 'Maker'), ai('maker', 'Carl Hansen', { conflict: true, ai_prior: prior('Fritz Hansen', 'integration') })),
      field(def('era', 'Era'), ai('era', '1950s', { ai_status: 'accepted' })),
      field(def('color', 'Color'), human('color', 'Oak')),
      field(def('notes', 'Notes')),
    ],
    [field(def('wear', 'Wear', 'condition'), ai('wear', 'Minor', { ai_confidence: 'low' }))],
    [field(def('model', 'Model', 'product'), ai('model', 'CH24', { ai_confidence: 'medium', ai_prior: prior('Chair', 'preset') }))],
  );
}

const renderForm = (position?: FormPosition) =>
  render(<FormDialog open onOpenChange={() => undefined} item={ITEM} photos={[]} sessionId={4} position={position} onItemChanged={() => undefined} />);

async function openReviewTab() {
  fireEvent.click(await screen.findByRole('tab', { name: /AI review/ }));
  return screen.findByRole('region', { name: 'AI review' });
}
const rowKeys = () => [...document.querySelectorAll('[data-review-row]')].map(row => row.getAttribute('data-review-row'));
const count = () => document.querySelector('[data-review-count]')?.textContent;

beforeEach(() => {
  vi.mocked(researchStatus).mockResolvedValue(status({ run: run('finished') }));
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('a11_1_the_tab_lists_exactly_the_pending_suggestions_of_every_step_with_their_count', async () => {
  vi.mocked(getForm).mockResolvedValue(mixed());
  renderForm();
  const tab = await screen.findByRole('tab', { name: /AI review/ });
  await waitFor(() => expect(count()).toBe('4'));
  expect(tab).toHaveAttribute('aria-selected', 'false');
  await openReviewTab();
  // Accepted, human and empty fields are not listed; suggestions of Condition and Product are.
  expect(rowKeys()).toEqual(['designer', 'maker', 'wear', 'model']);
  const designer = screen.getByRole('listitem', { name: 'Designer' });
  expect(designer).toHaveTextContent('· Details');
  expect(designer).toHaveTextContent('High confidence');
  expect(designer).toHaveTextContent('Hans J. Wegner');
  expect(within(designer).getByText('The field was empty')).toBeInTheDocument();
  expect(within(designer).getByText('Two dealers name him')).toBeInTheDocument();
  // Sources open in the browser.
  fireEvent.click(within(designer).getByRole('link'));
  expect(openUrl).toHaveBeenCalledWith('https://www.pamono.com/wegner-ch24');
  const model = screen.getByRole('listitem', { name: 'Model' });
  expect(model).toHaveTextContent('· Product');
  expect(model).toHaveTextContent('Medium confidence');
  expect(model).toHaveTextContent('Replaces Chair · preset default');
  // A18: a suggestion with a prior (not a conflict) offers Keep previous; one over an empty field, Dismiss.
  expect(within(model).getByRole('button', { name: 'Keep previous' })).toBeInTheDocument();
  expect(within(designer).getByRole('button', { name: 'Dismiss' })).toBeInTheDocument();
  expect(within(designer).queryByRole('button', { name: 'Keep previous' })).not.toBeInTheDocument();
  const maker = screen.getByRole('listitem', { name: 'Maker' });
  expect(within(maker).getByText('Conflict')).toBeInTheDocument();
  expect(maker).toHaveTextContent('Replaces Fritz Hansen · from Shopify');
  expect(screen.getByRole('listitem', { name: 'Wear' })).toHaveTextContent('Low confidence · check');
  // The Back / Next step buttons belong to the steps, not to AI review.
  expect(screen.queryByRole('button', { name: /Condition$/ })).not.toBeInTheDocument();
});

it('a11_2_looks_right_and_clear_use_the_pill_commands_and_the_row_leaves_with_the_count', async () => {
  vi.mocked(getForm).mockResolvedValue(mixed());
  renderForm();
  await openReviewTab();
  await waitFor(() => expect(count()).toBe('4'));

  // Confirm: the same command as the pill's Looks right; the re-read form no longer holds the suggestion.
  const after = mixed();
  after.groups[0].fields[0] = field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner', { ai_status: 'accepted' }));
  vi.mocked(getForm).mockResolvedValue(after);
  fireEvent.click(within(screen.getByRole('listitem', { name: 'Designer' })).getByRole('button', { name: 'Accept' }));
  await waitFor(() => expect(acceptSuggestion).toHaveBeenCalledExactlyOnceWith(ITEM.id, 'designer'));
  await waitFor(() => expect(rowKeys()).toEqual(['maker', 'wear', 'model']));
  expect(count()).toBe('3');

  // Deny over an empty field: Dismiss is the pill's Clear.
  const cleared = structuredClone(after);
  cleared.groups[1].fields[0] = field(def('wear', 'Wear', 'condition'));
  vi.mocked(getForm).mockResolvedValue(cleared);
  fireEvent.click(within(screen.getByRole('listitem', { name: 'Wear' })).getByRole('button', { name: 'Dismiss' }));
  await waitFor(() => expect(clearAttribute).toHaveBeenCalledExactlyOnceWith(ITEM.id, 'wear'));
  await waitFor(() => expect(rowKeys()).toEqual(['maker', 'model']));
  expect(count()).toBe('2');
  // The tab stays open on its own list.
  expect(screen.getByRole('tab', { name: /AI review/ })).toHaveAttribute('aria-selected', 'true');
});

it('a11_2_a_conflict_offers_the_conflict_actions_and_edit_goes_to_the_field_in_edit_mode', async () => {
  vi.mocked(getForm).mockResolvedValue(mixed());
  renderForm();
  await openReviewTab();
  const maker = await screen.findByRole('listitem', { name: 'Maker' });
  expect(within(maker).queryByRole('button', { name: 'Dismiss' })).not.toBeInTheDocument();
  fireEvent.click(within(maker).getByRole('button', { name: 'Keep previous' }));
  await waitFor(() => expect(resolveConflict).toHaveBeenCalledExactlyOnceWith(ITEM.id, 'maker', 'keep_previous'));
  fireEvent.click(within(screen.getByRole('listitem', { name: 'Maker' })).getByRole('button', { name: 'Edit' }));
  expect(await screen.findByRole('region', { name: 'Details fields' })).toBeInTheDocument();
  await waitFor(() => expect(document.activeElement?.id).toBe('field-maker'));
});

it('a11_2_edit_goes_to_the_field_on_its_own_step', async () => {
  vi.mocked(getForm).mockResolvedValue(mixed());
  renderForm();
  await openReviewTab();
  fireEvent.click(within(await screen.findByRole('listitem', { name: 'Model' })).getByRole('button', { name: 'Edit' }));
  expect(await screen.findByRole('region', { name: 'Product fields' })).toBeInTheDocument();
  expect(await screen.findByRole('dialog', { name: 'AI suggestion for Model' })).toBeInTheDocument();
});

it('a11_3_the_tab_stays_at_zero_and_says_why_it_is_empty', async () => {
  vi.mocked(getForm).mockResolvedValue(form([field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner', { ai_status: 'accepted' })), field(def('notes', 'Notes'))]));
  renderForm();
  await openReviewTab();
  expect(count()).toBe('0');
  expect(screen.getByText('Nothing to confirm')).toBeInTheDocument();
  expect(screen.getByText('Every AI suggestion on this item has been checked.')).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: /to review$/ })).not.toBeInTheDocument();

  expect(emptyReason(status(), false)).toBe("Research hasn't run for this item yet.");
  expect(emptyReason(status({ reason: { code: 'NoFieldChosen', sentence: 'Choose the fields research fills in Settings → Research' } as ResearchStatus['reason'] }), false))
    .toBe("Research hasn't run for this item yet. Choose the fields research fills in Settings → Research");
  expect(emptyReason(null, false)).toBe("Research hasn't run for this item yet.");
  expect(emptyReason(status({ run: run('running') }), false)).toBe('Research is running. Its suggestions appear here as they come in.');
  expect(emptyReason(status({ run: run('queued') }), true)).toBe('Research is running. Its suggestions appear here as they come in.');
  expect(emptyReason(status({ run: run('finished') }), false)).toBe('The last research run suggested nothing to confirm.');
});

it('a11_4_the_to_review_chip_and_an_opening_position_land_on_the_tab', async () => {
  vi.mocked(getForm).mockResolvedValue(mixed());
  renderForm();
  fireEvent.click(await screen.findByRole('button', { name: '4 to review' }));
  expect(await screen.findByRole('region', { name: 'AI review' })).toBeInTheDocument();
  expect(screen.getByRole('tab', { name: /AI review/ })).toHaveAttribute('aria-selected', 'true');
  cleanup();
  // The header's "AI · n to review" opens the form with this position.
  renderForm({ step: 'review', scroll: 0 });
  expect(await screen.findByRole('region', { name: 'AI review' })).toBeInTheDocument();
  await waitFor(() => expect(rowKeys()).toEqual(['designer', 'maker', 'wear', 'model']));
});

it('a18_keep_previous_and_accept_on_a_suggestion_with_a_prior_that_is_not_a_conflict', async () => {
  vi.mocked(getForm).mockResolvedValue(mixed());
  renderForm();
  await openReviewTab();
  const model = await screen.findByRole('listitem', { name: 'Model' });
  fireEvent.click(within(model).getByRole('button', { name: 'Keep previous' }));
  await waitFor(() => expect(resolveConflict).toHaveBeenCalledExactlyOnceWith(ITEM.id, 'model', 'keep_previous'));
  fireEvent.click(within(screen.getByRole('listitem', { name: 'Model' })).getByRole('button', { name: 'Accept' }));
  await waitFor(() => expect(acceptSuggestion).toHaveBeenCalledWith(ITEM.id, 'model'));
  // A conflict's Accept is its Use suggestion.
  fireEvent.click(within(screen.getByRole('listitem', { name: 'Maker' })).getByRole('button', { name: 'Accept' }));
  await waitFor(() => expect(resolveConflict).toHaveBeenCalledWith(ITEM.id, 'maker', 'use_suggestion'));
});

it('a18_a_row_reads_as_before_arrow_after_with_confidence_in_words_and_sources_as_site_chips', async () => {
  const long = 'Two dealers name him as the designer of this exact chair and both show the same carved back rail, paper cord seat and steam-bent top, and a third lists it under his name.';
  const f = mixed();
  f.groups[0].fields[0] = field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner', { ai_reason: long, ai_sources: ['https://www.pamono.com/wegner-ch24', 'https://www.1stdibs.com/x'] }));
  vi.mocked(getForm).mockResolvedValue(f);
  renderForm();
  await openReviewTab();
  const designer = await screen.findByRole('listitem', { name: 'Designer' });
  expect(within(designer).getByText('High confidence')).toBeInTheDocument();
  expect(designer.querySelector('[data-change]')).toHaveTextContent('The field was empty');
  expect(designer.querySelector('[data-suggested]')).toHaveTextContent('Hans J. Wegner');
  const sources = within(designer).getByLabelText('Sources');
  expect(within(sources).getAllByRole('link').map(a => a.textContent)).toEqual(['pamono.com', '1stdibs.com']);
  // A long reason opens with More and closes with Less.
  fireEvent.click(within(designer).getByRole('button', { name: 'More' }));
  expect(within(designer).getByRole('button', { name: 'Less' })).toBeInTheDocument();
  const maker = screen.getByRole('listitem', { name: 'Maker' });
  expect(maker.querySelector('[data-change]')).toHaveTextContent('Fritz Hansen');
  expect(maker.querySelector('[data-suggested]')).toHaveTextContent('Carl Hansen');
});

const renderDetails = (review?: { left: number; onNext: () => void }, onOpenChange: (open: boolean) => void = () => undefined) =>
  render(<FormDialog open onOpenChange={onOpenChange} item={ITEM} photos={[]} sessionId={4} reviewFirst review={review} onItemChanged={() => undefined} />);

it('a19_details_opens_on_ai_review_when_suggestions_wait_and_on_details_otherwise', async () => {
  vi.mocked(getForm).mockResolvedValue(mixed());
  renderDetails();
  expect(await screen.findByRole('region', { name: 'AI review' })).toBeInTheDocument();
  expect(screen.getByRole('tab', { name: /AI review/ })).toHaveAttribute('aria-selected', 'true');
  cleanup();
  vi.mocked(getForm).mockResolvedValue(form([field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner', { ai_status: 'accepted' }))]));
  renderDetails();
  expect(await screen.findByRole('region', { name: 'Details fields' })).toBeInTheDocument();
  // Without reviewFirst (a publish batch's form) the form keeps opening on Details.
  cleanup();
  vi.mocked(getForm).mockResolvedValue(mixed());
  renderForm();
  expect(await screen.findByRole('region', { name: 'Details fields' })).toBeInTheDocument();
});

it('a19_the_last_suggestion_checked_offers_next_to_review', async () => {
  vi.mocked(getForm).mockResolvedValue(form([field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner'))]));
  const onNext = vi.fn();
  const onOpenChange = vi.fn();
  renderDetails({ left: 2, onNext }, onOpenChange);
  const designer = await screen.findByRole('listitem', { name: 'Designer' });
  vi.mocked(getForm).mockResolvedValue(form([field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner', { ai_status: 'accepted' }))]));
  fireEvent.click(within(designer).getByRole('button', { name: 'Accept' }));
  expect(await screen.findByText('All checked ✓')).toBeInTheDocument();
  expect(screen.getByText('2 more sessions wait for review.')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: /Next to review/ }));
  expect(onNext).toHaveBeenCalledTimes(1);
  expect(onOpenChange).toHaveBeenCalledWith(false);
});

it('a19_with_no_other_session_waiting_it_goes_back_to_the_sessions', async () => {
  vi.mocked(getForm).mockResolvedValue(form([field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner', { ai_status: 'accepted' }))]));
  const onNext = vi.fn();
  render(<FormDialog open onOpenChange={() => undefined} item={ITEM} photos={[]} sessionId={4} position={{ step: 'review', scroll: 0 }} review={{ left: 0, onNext }} onItemChanged={() => undefined} />);
  expect(await screen.findByText('No other session waits for review.')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: /Back to sessions/ }));
  expect(onNext).toHaveBeenCalledTimes(1);
});
