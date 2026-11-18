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
}));
vi.mock('@/services/research', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/research')>()),
  researchStatus: vi.fn(),
  researchOverview: vi.fn(),
  runResearch: vi.fn(),
}));
vi.mock('@/services/events', () => ({ onResearchChanged: vi.fn(async () => () => undefined), onSessionsChanged: vi.fn(async () => () => undefined) }));
vi.mock('@/features/form/ItemTypePicker', () => ({ ItemTypePicker: () => null, useItemTypes: () => ({ types: [], failed: false }) }));
vi.mock('@/features/form/StockFields', () => ({ StockFields: () => null }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { openUrl } from '@tauri-apps/plugin-opener';
import { ExternalLink } from '@/components/ExternalLink';
import { FormDialog } from '@/features/form/FormDialog';
import { ResearchPanel, waitingLine } from '@/features/form/ResearchPanel';
import { ResearchPill, ordinal, researchLabel, useResearchOverview } from '@/features/sessions/ResearchPill';
import { acceptAllSuggestions, getForm } from '@/services/attributes';
import { researchOverview, researchStatus } from '@/services/research';

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
function field(definition: Definition, value: AttributeValue | null = null): FormField {
  return { definition, value, is_default: false, is_severity: false, selected: null, default: null, saved: [] };
}
function status(extra: Partial<ResearchStatus> = {}): ResearchStatus {
  return { reason: null, run: null, steps: [], fields_filled: 0, to_review: 0, can_run: true, wait_ends_at: null, queue: null, ...extra };
}
const run = (state: string, trigger = 'arranged') => ({ id: 1, state, trigger, stage_reached: null, error: null });
const panel = (s: ResearchStatus) =>
  render(<ResearchPanel status={s} fields={[]} focusKey={null} onFocus={() => undefined} onRun={async () => undefined} onAcceptAll={async () => undefined} onClose={() => undefined} />);

beforeEach(() => {
  vi.mocked(researchStatus).mockResolvedValue(status({ run: run('finished') }));
  vi.mocked(acceptAllSuggestions).mockResolvedValue(undefined as never);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

// 1. Links open in the default browser.
it('a6_1_a_source_link_opens_in_the_default_browser', () => {
  render(<ExternalLink href="https://www.pamono.com/wegner-ch24">Pamono</ExternalLink>);
  fireEvent.click(screen.getByRole('link', { name: 'Pamono' }));
  expect(openUrl).toHaveBeenCalledWith('https://www.pamono.com/wegner-ch24');
});

it('a6_1_a_research_step_link_goes_through_the_opener_and_a_non_web_address_is_not_a_link', () => {
  panel(status({ run: run('running'), steps: [{ seq: 1, stage: 'read', kind: 'read', summary: 'a page', detail: '', source_url: 'https://example.com/chair', source_title: 'Example · Chair', photo_ids: [], field_keys: [], unread: false }] }));
  fireEvent.click(screen.getByRole('link', { name: 'Example · Chair' }));
  expect(openUrl).toHaveBeenCalledWith('https://example.com/chair');
  cleanup();
  render(<ExternalLink href="file:///etc/passwd" />);
  expect(screen.queryByRole('link')).not.toBeInTheDocument();
});

// 2. "n to review" is locatable.
function twoSteps(): Form {
  const designer = field(def('designer', 'Designer'), ai('designer', 'Hans J. Wegner'));
  const wear = field(def('wear', 'Wear', 'condition'), ai('wear', 'Minor', { ai_confidence: 'low' }));
  const maker = field(def('maker', 'Maker'), ai('maker', 'Carl Hansen', { conflict: true, ai_prior: { value: JSON.stringify('Fritz Hansen'), provenance: 'integration' } as AttributeValue['ai_prior'] }));
  return { item_id: ITEM.id, item_type_from_integration: false, groups: [
    { group: 'details', fields: [designer, maker], filled: 2, total: 2 },
    { group: 'condition', fields: [wear], filled: 1, total: 1 },
    { group: 'product', fields: [], filled: 0, total: 0 },
  ] };
}
const renderForm = () => render(<FormDialog open onOpenChange={() => undefined} item={ITEM} photos={[]} sessionId={4} onItemChanged={() => undefined} />);

// A11: the chip opens the AI review tab, where the fields are listed (it replaced A6's popover list).
it('a6_2_the_review_list_shows_each_field_with_value_confidence_and_conflict', async () => {
  vi.mocked(getForm).mockResolvedValue(twoSteps());
  renderForm();
  fireEvent.click(await screen.findByRole('button', { name: '3 to review' }));
  expect(await screen.findByRole('tab', { name: /AI review/ })).toHaveAttribute('aria-selected', 'true');
  const list = await screen.findByRole('list', { name: 'Suggestions to confirm' });
  const rows = within(list).getAllByRole('listitem').filter(row => row.hasAttribute('data-review-row'));
  expect(rows.map(r => r.getAttribute('data-review-row'))).toEqual(['designer', 'maker', 'wear']);
  expect(rows[0]).toHaveTextContent(/Designer· Details.*High confidence.*Hans J\. Wegner/);
  expect(within(rows[1]).getByText('Conflict')).toBeInTheDocument();
  expect(rows[2]).toHaveTextContent(/Wear· Condition.*Low confidence · check.*Minor/);
  // Accept all from the tab takes only the high, non-conflict suggestion's count.
  const acceptAll = screen.getByRole('button', { name: 'Accept all (1)' });
  fireEvent.click(acceptAll);
  await waitFor(() => expect(acceptAllSuggestions).toHaveBeenCalledWith(ITEM.id));
});

it('a6_2_choosing_a_field_goes_to_its_step_and_opens_its_ai_popover', async () => {
  vi.mocked(getForm).mockResolvedValue(twoSteps());
  renderForm();
  fireEvent.click(await screen.findByRole('button', { name: '3 to review' }));
  const wearRow = await screen.findByRole('listitem', { name: 'Wear' });
  fireEvent.click(within(wearRow).getByRole('button', { name: 'Edit' }));
  expect(await screen.findByRole('region', { name: 'Condition fields' })).toBeInTheDocument();
  const popover = await screen.findByRole('dialog', { name: 'AI suggestion for Wear' });
  await new Promise(resolve => setTimeout(resolve, 50));
  expect(popover).toHaveAttribute('data-state', 'open');
  expect(within(popover).getByRole('button', { name: 'Looks right' })).toBeInTheDocument();
  expect(within(popover).getByRole('button', { name: 'Clear' })).toBeInTheDocument();
});

// 4. A waiting session says so.
it('a6_4_a_queued_run_says_whom_it_waits_for_and_its_place', () => {
  expect(ordinal(1)).toBe('1st');
  expect(ordinal(2)).toBe('2nd');
  expect(ordinal(3)).toBe('3rd');
  expect(ordinal(11)).toBe('11th');
  expect(ordinal(22)).toBe('22nd');
  panel(status({ run: run('queued'), queue: { position: 2, running_session_id: 9, running_name: 'Armchair · VF-0001', manual: false } }));
  expect(screen.getByText('Waiting — researching Armchair · VF-0001 first (2nd in line)')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Research this next' })).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Research' })).not.toBeInTheDocument();
});

it('a6_4_a_run_asked_for_by_hand_says_it_goes_first_and_offers_no_button', () => {
  panel(status({ run: run('queued', 'manual'), queue: { position: 1, running_session_id: 9, running_name: 'Armchair · VF-0001', manual: true } }));
  expect(screen.getByText('Waiting — researching Armchair · VF-0001 first (1st in line)')).toBeInTheDocument();
  expect(screen.getByText(/goes before the sessions marked arranged/)).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: /Research/ })).not.toBeInTheDocument();
  expect(waitingLine({ position: 1, running_session_id: null, running_name: null, manual: false })).toBe('Waiting to start (1st in line)');
});

// 3. Research status on the list and the session page.
it('a6_3_every_state_has_its_words', () => {
  const row = (state: string, extra = {}) => ({ session_id: 1, state, position: null, to_review: 0, reason: null, ...extra });
  expect(researchLabel(row('researching'))).toBe('AI researching…');
  expect(researchLabel(row('waiting', { position: 3 }))).toBe('AI waiting · 3rd in line');
  expect(researchLabel(row('to_review', { to_review: 2 }))).toBe('AI · 2 to review');
  expect(researchLabel(row('done'))).toBe('AI done');
  expect(researchLabel(row('failed'))).toBe('AI stopped');
  expect(researchLabel(row('blocked', { reason: { code: 'NoFieldChosen', sentence: 'Choose the fields research fills in Settings → Research' } }))).toBe("AI can't run · no fields chosen");
  render(<ResearchPill research={row('blocked', { reason: { code: 'NoFieldChosen', sentence: 'Choose the fields research fills in Settings → Research' } })} />);
  expect(screen.getByText("AI can't run · no fields chosen")).toHaveAttribute('title', 'Choose the fields research fills in Settings → Research');
});

function Overview({ id }: { id: number | null | undefined }) {
  const rows = useResearchOverview(id);
  return <ul>{[...rows.values()].map(r => <li key={r.session_id}>{r.session_id}:{r.state}</li>)}</ul>;
}

it('a6_3_the_list_reads_every_session_in_one_call_and_a_header_reads_only_its_own', async () => {
  vi.mocked(researchOverview).mockResolvedValue([
    { session_id: 4, state: 'researching', position: null, to_review: 0, reason: null },
    { session_id: 5, state: 'waiting', position: 1, to_review: 0, reason: null },
  ]);
  render(<Overview id={null} />);
  expect(await screen.findByText('5:waiting')).toBeInTheDocument();
  expect(researchOverview).toHaveBeenCalledTimes(1);
  expect(researchOverview).toHaveBeenCalledWith(null);
  cleanup();
  vi.mocked(researchOverview).mockClear();
  render(<Overview id={undefined} />);
  expect(researchOverview).not.toHaveBeenCalled();
});
