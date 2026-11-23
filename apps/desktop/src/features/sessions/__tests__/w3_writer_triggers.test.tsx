import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { AttributeValue, Definition, Form, FormField } from '@/services/attributes';
import type { Item } from '@/services/items';
import type { ResearchStatus, SessionResearch } from '@/services/research';
import type { SessionRowResponse_Serialize } from '@/services/bindings';
import type { WriterStatus } from '@/services/writer';

// W3·U3 rows W3·C7: what asks the writer for a session's texts (WC3 "Frontend calls"). Boundaries: `AiReviewTab`,
// `FormDialog` (closing Details, the Product step's Write texts) and `Header`, rendered with `services/writer`,
// `services/research` and `services/attributes` mocked; the outcome is the service calls made and what is shown.

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
  researchOverview: vi.fn(),
}));
vi.mock('@/services/writer', () => ({
  writerStatus: vi.fn(),
  writerRequest: vi.fn(),
  writerFindingAnswer: vi.fn(),
  writerFindingDismiss: vi.fn(),
  writerNote: vi.fn(async () => { throw new Error('no note'); }),
  writerLookAgain: vi.fn(),
}));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  listSessions: vi.fn(), sessionsPage: vi.fn(),
  arrangeNext: vi.fn(),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/events', () => ({ onResearchChanged: vi.fn(async () => () => undefined), onSessionsChanged: vi.fn(async () => () => undefined) }));
vi.mock('@/services/export', () => ({ integrationKind: vi.fn(async () => null), onExportProgress: vi.fn(async () => () => undefined), openExportFolder: vi.fn(), runExport: vi.fn() }));
vi.mock('@/features/publish/usePublish', () => ({ usePublish: () => ({ failedCount: 0, label: null, notifications: [], refresh: async () => undefined }) }));
vi.mock('@/features/publish/NotificationsDrawer', () => ({ NotificationsDrawer: () => null }));
vi.mock('@/features/publish/PublishProgress', () => ({ PublishProgress: () => null }));
vi.mock('@/features/settings/StorageSection', () => ({ DiskWarning: () => null }));
vi.mock('@/features/history/HistoryButtons', () => ({ HistoryButtons: () => null }));
vi.mock('@/features/shell/CameraDot', () => ({ CameraDot: () => null }));
vi.mock('@/features/shell/ThemeToggle', () => ({ ThemeToggle: () => null }));
vi.mock('@/features/form/ItemTypePicker', () => ({ ItemTypePicker: () => null, useItemTypes: () => ({ types: [], failed: false }) }));
vi.mock('@/features/form/StockFields', () => ({ StockFields: () => null }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn(), loading: vi.fn() } }));

import { toast } from 'sonner';
import { AiReviewTab } from '@/features/form/AiReviewTab';
import { FormDialog } from '@/features/form/FormDialog';
import { Header } from '@/features/shell/Header';
import { getForm } from '@/services/attributes';
import { researchOverview, researchStatus } from '@/services/research';
import { listSessions, sessionsPage } from '@/services/sessions';
import { answerFromList } from '@/features/sessions/__tests__/pageFromList';
import { writerRequest, writerStatus } from '@/services/writer';

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
const BUSY = 'Research is still working on this session; write texts when it finishes';

function def(key: string, label: string, group: Definition['group']): Definition {
  return { integration: 'shopify', key, label, core_type: 'text', choices: [], validations: '{}', group, description: null, writable: true };
}
function ai(key: string, value: string, status: 'suggested' | 'accepted'): AttributeValue {
  return { item_id: ITEM.id, key, value: JSON.stringify(value), valid: true, provenance: 'ai', ai_confidence: 'medium', ai_status: status, ai_reason: null, ai_sources: [], ai_prior: null, updated_at: 1, conflict: false };
}
function field(definition: Definition, value: AttributeValue | null = null): FormField {
  return { definition, value, is_default: false, is_severity: false, selected: null, default: null, saved: [] };
}
function form(details: FormField[], product: FormField[] = []): Form {
  const g = (group: Definition['group'], fields: FormField[]) => ({ group, fields, filled: fields.filter(f => f.value).length, total: fields.length });
  return { item_id: ITEM.id, item_type_from_integration: false, groups: [g('details', details), g('condition', []), g('product', product)] };
}
function research(): ResearchStatus {
  return { reason: null, run: { id: 1, state: 'finished', trigger: 'arranged', stage_reached: null, error: null }, steps: [], fields_filled: 0, to_review: 0, can_run: true, wait_ends_at: null, queue: null };
}
function writer(): WriterStatus {
  return {
    reason: null, run: null, texts: [], attempts: [], regenerated: [], findings: [], steps: [], cost_micro_usd: 0,
    inputs: { fields: 0, note: 0, listings: [] }, stale_main: false, stale_condition: false, can_write: true, finished_at: null, not_offered: [],
  };
}
const checked = () => form([field(def('designer', 'Designer', 'details'), ai('designer', 'Børge Mogensen', 'accepted'))]);
const unchecked = () => form([
  field(def('designer', 'Designer', 'details'), ai('designer', 'Børge Mogensen', 'accepted')),
  field(def('era', 'Era', 'details'), ai('era', '1960s', 'suggested')),
]);
const renderForm = () =>
  render(<FormDialog open onOpenChange={() => undefined} item={ITEM} photos={[]} sessionId={SESSION} onItemChanged={() => undefined} review={{ left: 1, onNext: () => undefined }} />);

beforeEach(() => {
  answerFromList(sessionsPage, listSessions);
  vi.mocked(researchStatus).mockResolvedValue(research());
  vi.mocked(writerStatus).mockResolvedValue(writer());
  vi.mocked(writerRequest).mockResolvedValue({ queued: true, reason: null });
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('w3_c7a_next_to_review_in_ai_review_asks_the_writer_then_moves_on_even_when_it_fails', () => {
  // A19 hazard: an automatic origin never blocks the owner's flow, so the request fails here.
  vi.mocked(writerRequest).mockRejectedValue(new Error('offline'));
  const onNext = vi.fn();
  render(
    <AiReviewTab entries={[]} acceptable={0} status={research()} anyAi onLooksRight={vi.fn()} onClear={vi.fn()} onKeepPrevious={vi.fn()}
      conflictActions={() => undefined} onSeeResearch={vi.fn()} onShowField={vi.fn()} onAcceptAll={vi.fn()}
      nextToReview={{ left: 2, onNext }} sessionId={SESSION} />,
  );
  expect(screen.getByText('All checked ✓')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: /Next to review/ }));
  return waitFor(() => {
    expect(writerRequest).toHaveBeenCalledTimes(1);
    expect(writerRequest).toHaveBeenCalledWith(SESSION, 'next_to_review');
    expect(onNext).toHaveBeenCalledTimes(1);
    expect(vi.mocked(writerRequest).mock.invocationCallOrder[0]).toBeLessThan(onNext.mock.invocationCallOrder[0]);
  });
});

it('w3_c7b_closing_details_while_all_checked_asks_once_and_not_with_a_suggestion_left', async () => {
  vi.mocked(getForm).mockResolvedValue(checked());
  renderForm();
  fireEvent.click(await screen.findByRole('tab', { name: /AI review/ }));
  expect(await screen.findByText('All checked ✓')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Close' }));
  await waitFor(() => expect(writerRequest).toHaveBeenCalledTimes(1));
  expect(writerRequest).toHaveBeenCalledWith(SESSION, 'all_checked');
  cleanup();
  expect(writerRequest).toHaveBeenCalledTimes(1);

  vi.mocked(writerRequest).mockClear();
  vi.mocked(getForm).mockResolvedValue(unchecked());
  renderForm();
  fireEvent.click(await screen.findByRole('tab', { name: /AI review/ }));
  expect(await screen.findByRole('listitem', { name: 'Era' })).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Close' }));
  cleanup();
  await new Promise(resolve => setTimeout(resolve, 0));
  expect(writerRequest).not.toHaveBeenCalled();
});

it('w3_c7c_write_texts_in_the_product_step_asks_once_and_shows_a_refusal_as_a_toast', async () => {
  vi.mocked(getForm).mockResolvedValue(form([], [field(def('product.title', 'Title', 'product'))]));
  vi.mocked(writerRequest).mockResolvedValue({ queued: false, reason: { code: 'ResearchBusy', sentence: BUSY } });
  renderForm();
  fireEvent.click(await screen.findByRole('tab', { name: /Product/ }));
  const region = await screen.findByRole('region', { name: 'Product fields' });
  fireEvent.click(within(region).getByRole('button', { name: 'Write texts' }));
  await waitFor(() => expect(toast.error).toHaveBeenCalledWith(BUSY));
  expect(writerRequest).toHaveBeenCalledTimes(1);
  expect(writerRequest).toHaveBeenCalledWith(SESSION, 'write_texts');
});

it('w3_c7d_the_headers_next_to_review_asks_the_writer_before_the_queue_moves_on', async () => {
  const row = {
    id: 3, sku: 'SKU-3', item_type: '', title: 'Chair', main_count: 2, condition_count: 1, unreviewed_count: 0, status: 'in_progress',
    label: { kind: 'in_progress', value: 0 }, progress: null, last_activity_at: 0, created_at: 0, cover_preview_path: null,
    arranged: true, publish: null, changes: null,
  } as SessionRowResponse_Serialize;
  vi.mocked(listSessions).mockResolvedValue({ groups: [{ label: 'Today', rows: [row] }], counts: { open: 1, published: 0, all: 1 }, unassigned_count: 0 });
  vi.mocked(researchOverview).mockResolvedValue([{ session_id: 3, state: 'done', position: null, to_review: 0, reason: null } as SessionResearch]);
  const onNext = vi.fn();
  render(<Header sessionId={3} theme="light" onThemeChange={() => undefined} onNewSession={() => undefined} reviewQueue={{ left: 2, onNext }} />);
  fireEvent.click(await screen.findByRole('button', { name: /Next to review \(2 left\)/ }));
  await waitFor(() => expect(writerRequest).toHaveBeenCalledTimes(1));
  expect(writerRequest).toHaveBeenCalledWith(3, 'next_to_review');
  expect(onNext).toHaveBeenCalledTimes(1);
  expect(vi.mocked(writerRequest).mock.invocationCallOrder[0]).toBeLessThan(onNext.mock.invocationCallOrder[0]);
});
