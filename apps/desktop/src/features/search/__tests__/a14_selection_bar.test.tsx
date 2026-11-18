import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, renderHook, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { Definition, Form, FormField } from '@/services/attributes';
import type { Photo, SessionView } from '@/services/photos';
import type { LensEvent, PageEvent } from '@/services/search';

vi.mock('@/services/search', () => ({
  openSearchWindow: vi.fn(async () => undefined),
  placeBrowser: vi.fn(async () => undefined),
  goTo: vi.fn(async () => undefined),
  browserStep: vi.fn(async () => undefined),
  startLens: vi.fn(async () => undefined),
  giveUpLens: vi.fn(async () => undefined),
  lensResults: vi.fn(async () => false),
  readSelection: vi.fn(async () => ({ selection: null, refused: null })),
  researchMatches: vi.fn(async () => []),
  visualSearch: vi.fn(async () => []),
  savedPages: vi.fn(async () => []),
  savePage: vi.fn(),
  removePage: vi.fn(async () => undefined),
  putText: vi.fn(),
  putBack: vi.fn(async () => undefined),
  openAiReview: vi.fn(async () => undefined),
  onOpenAiReview: vi.fn(),
  askAi: vi.fn(),
  onSearchEvents: vi.fn(),
}));
vi.mock('@/services/photos', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/photos')>()), getSessionView: vi.fn() }));
vi.mock('@/services/attributes', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/attributes')>()), getForm: vi.fn() }));
vi.mock('@/services/sessions', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/sessions')>()), previewUrl: (path: string) => `asset://${path}` }));
vi.mock('sonner', () => ({ toast: Object.assign(vi.fn(), { success: vi.fn(), error: vi.fn(), message: vi.fn() }) }));
vi.mock('@/components/ui/sonner', () => ({ Toaster: () => null }));

import { SearchWindow } from '@/features/search/SearchWindow';
import { matchingFields } from '@/features/search/SelectionBar';
import { useReviewFromSearch } from '@/features/search/useReviewFromSearch';
import { getForm } from '@/services/attributes';
import { getSessionView } from '@/services/photos';
import * as search from '@/services/search';
import { toast } from 'sonner';

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

const SESSION = 3;
const ITEM_ID = 7;
const PAGE_URL = 'https://www.rusta.com/sv/vattenflaska';
const TEXT = 'Vattenflaska 500 ml Svart, design Rusta Studio';

function photo(id: number): Photo {
  return {
    id, session_id: SESSION, classification: 'main', position: id, captured_at: id, original_preview_path: null, adjusted_preview_path: `/p/${id}.jpg`, thumbnail_path: null,
    composition: { photo_id: id, kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: null, frame: null, frame_source: null, origin: 'default', revision: 0, state: 'ok', review_reason: null, updated_at: 0 },
    orientation: null, source: 'capture', integrity: 'ok', upload_state: null,
  };
}
function def(key: string, label: string, core_type = 'text'): Definition {
  return { integration: 'shopify', key, label, core_type, choices: [], validations: '{}', group: 'details', description: null, writable: true };
}
function field(definition: Definition): FormField {
  return { definition, value: null, is_default: false, is_severity: false, selected: null, default: null, saved: [] };
}
const FIELDS = [field(def('custom.wood', 'Wood')), field(def('custom.colour', 'Colour')), field(def('custom.volume', 'Volume')), field(def('custom.manufacturer', 'Manufacturer')), field(def('custom.designer', 'Designer'))];
const VIEW: SessionView = {
  session: { id: SESSION, item_id: ITEM_ID, published_at: null, published_seq: 0, change_seq: 0, dirty: false, last_activity_at: 0, created_at: 0, arranged_at: null, photo_count: 1, unreviewed_count: 0, main_count: 1, condition_count: 0, publish: null, changes: null },
  item: { id: ITEM_ID, sku: 'BT-0001', folder_key: 'BT-0001', barcode: null, item_type: 'Bottle', title: 'Bottle', origin: 'local', external_id: null, created_at: 0, updated_at: 0 },
  photos: [photo(11)],
};

let handlers: { page: (e: PageEvent) => void; lens: (e: LensEvent) => void; photo: (id: number) => void };

beforeEach(() => {
  vi.mocked(getSessionView).mockResolvedValue(VIEW);
  vi.mocked(getForm).mockResolvedValue({ item_id: ITEM_ID, item_type: 'Bottle', groups: [{ group: 'details', fields: FIELDS, filled: 0, total: 5 }] } as unknown as Form);
  vi.mocked(search.savedPages).mockResolvedValue([]);
  vi.mocked(search.readSelection).mockResolvedValue({ selection: { text: TEXT, url: PAGE_URL, title: 'Vattenflaska' }, refused: null });
  vi.mocked(search.putText).mockResolvedValue({ stored: {} as never, previous: '"Rusta"' });
  vi.mocked(search.askAi).mockResolvedValue({ suggested: [{ key: 'custom.volume', label: 'Volume', value: '500 ml' }], not_written: [], cost_micro_usd: 200 });
  vi.mocked(search.onSearchEvents).mockImplementation(async (_s, next) => {
    handlers = next;
    return () => undefined;
  });
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function renderBar() {
  render(<SearchWindow sessionId={SESSION} initialPhotoId={11} />);
  await waitFor(() => expect(handlers).toBeDefined());
  const bar = screen.getByLabelText('Selected text');
  await within(bar).findByTestId('selection-snippet');
  return bar;
}

it('a14_the_bar_shows_the_selection_as_a_quoted_snippet_with_its_whole_text_on_hover', async () => {
  const bar = await renderBar();
  const snippet = within(bar).getByTestId('selection-snippet');
  expect(snippet).toHaveTextContent(TEXT);
  expect(snippet).toHaveAttribute('title', TEXT);
  expect(within(bar).getByTestId('selection-bar')).toHaveAttribute('data-mode', 'actions');
  // The quick fields are labelled by name, Manufacturer and Designer first.
  expect(within(bar).getAllByRole('button', { name: /^Put in / }).map(b => b.textContent?.trim())).toEqual(['Manufacturer', 'Designer']);
});

it('a14_put_in_is_confirmed_by_a_toast_whose_undo_puts_the_previous_value_back', async () => {
  const bar = await renderBar();
  fireEvent.click(within(bar).getByRole('button', { name: 'Put in Designer' }));
  await waitFor(() => expect(search.putText).toHaveBeenCalledWith(SESSION, 'custom.designer', TEXT));
  await waitFor(() => expect(toast.success).toHaveBeenCalled());
  const [title, options] = vi.mocked(toast.success).mock.calls[0] as [string, { action: { label: string; onClick: () => void } }];
  expect(title).toBe('Put in Designer');
  expect(options.action.label).toBe('Undo');
  expect(search.putBack).not.toHaveBeenCalled();
  options.action.onClick();
  await waitFor(() => expect(search.putBack).toHaveBeenCalledWith(SESSION, 'custom.designer', '"Rusta"'));
  // A field that was empty is emptied again.
  vi.mocked(search.putText).mockResolvedValueOnce({ stored: {} as never, previous: null });
  vi.mocked(toast.success).mockClear();
  fireEvent.click(within(bar).getByRole('button', { name: 'Put in Manufacturer' }));
  await waitFor(() => expect(toast.success).toHaveBeenCalled());
  (vi.mocked(toast.success).mock.calls[0][1] as unknown as { action: { onClick: () => void } }).action.onClick();
  await waitFor(() => expect(search.putBack).toHaveBeenLastCalledWith(SESSION, 'custom.manufacturer', null));
});

it('a14_other_field_is_a_searchable_list_in_the_bar_with_arrow_keys_return_and_escape', async () => {
  const bar = await renderBar();
  fireEvent.click(within(bar).getByRole('button', { name: /Other field/ }));
  const find = within(bar).getByRole('combobox', { name: 'Find a field' });
  expect(find).toHaveFocus();
  const options = () => within(bar).getAllByRole('option').map(o => o.textContent);
  expect(options()).toEqual(['Wood', 'Colour', 'Volume']);
  fireEvent.change(find, { target: { value: 'o' } });
  expect(options()).toEqual(['Wood', 'Colour', 'Volume']);
  fireEvent.change(find, { target: { value: 'ol' } });
  expect(options()).toEqual(['Colour', 'Volume']);
  fireEvent.keyDown(find, { key: 'ArrowRight' });
  expect(within(bar).getByRole('option', { name: 'Volume' })).toHaveAttribute('aria-selected', 'true');
  fireEvent.keyDown(find, { key: 'Enter' });
  await waitFor(() => expect(search.putText).toHaveBeenCalledWith(SESSION, 'custom.volume', TEXT));
  expect(within(bar).getByTestId('selection-bar')).toHaveAttribute('data-mode', 'actions');
  // Escape goes back without putting anything.
  fireEvent.click(within(bar).getByRole('button', { name: /Other field/ }));
  fireEvent.keyDown(within(bar).getByRole('combobox', { name: 'Find a field' }), { key: 'Escape' });
  expect(within(bar).getByTestId('selection-bar')).toHaveAttribute('data-mode', 'actions');
  expect(search.putText).toHaveBeenCalledTimes(1);
});

it('a14_ask_ai_opens_a_prompt_in_the_bar_return_sends_escape_cancels_and_the_answer_links_to_ai_review', async () => {
  const bar = await renderBar();
  fireEvent.click(within(bar).getByRole('button', { name: /Ask AI/ }));
  const prompt = within(bar).getByLabelText('Instruction for AI');
  expect(prompt).toHaveFocus();
  fireEvent.keyDown(prompt, { key: 'Escape' });
  expect(within(bar).getByTestId('selection-bar')).toHaveAttribute('data-mode', 'actions');
  expect(search.askAi).not.toHaveBeenCalled();

  let answer: (value: Awaited<ReturnType<typeof search.askAi>>) => void = () => undefined;
  vi.mocked(search.askAi).mockImplementationOnce(() => new Promise(resolve => { answer = resolve; }));
  fireEvent.click(within(bar).getByRole('button', { name: /Ask AI/ }));
  fireEvent.change(within(bar).getByLabelText('Instruction for AI'), { target: { value: 'the volume' } });
  // The page's selection goes meanwhile: the prompt keeps the selection it was opened on.
  vi.mocked(search.readSelection).mockResolvedValue({ selection: null, refused: null });
  await act(async () => { await new Promise(r => setTimeout(r, 800)); });
  fireEvent.keyDown(within(bar).getByLabelText('Instruction for AI'), { key: 'Enter' });
  expect(search.askAi).toHaveBeenCalledWith(SESSION, { text: TEXT, url: PAGE_URL, title: 'Vattenflaska' }, 'the volume');
  expect(within(bar).getByTestId('selection-bar')).toHaveAttribute('data-mode', 'asking');
  expect(within(bar).getByText('Asking AI…')).toBeInTheDocument();
  await act(async () => answer({ suggested: [{ key: 'custom.volume', label: 'Volume', value: '500 ml' }], not_written: [], cost_micro_usd: 200 }));
  expect(within(bar).getByTestId('selection-bar')).toHaveAttribute('data-mode', 'done');
  expect(within(bar).getByText('1 suggestion sent to AI review')).toBeInTheDocument();
  fireEvent.click(within(bar).getByRole('button', { name: /Open AI review/ }));
  expect(search.openAiReview).toHaveBeenCalledWith(SESSION);
});

it('a14_with_nothing_selected_a_hint_pill_shows_and_dismiss_hides_the_bar_until_another_selection', async () => {
  const bar = await renderBar();
  fireEvent.click(within(bar).getByRole('button', { name: 'Dismiss selection' }));
  expect(within(bar).queryByTestId('selection-bar')).not.toBeInTheDocument();
  expect(within(bar).getByRole('status')).toHaveTextContent('Select text on the page');
  expect(within(bar).queryByRole('button')).not.toBeInTheDocument();
  // Another selection brings the bar back.
  vi.mocked(search.readSelection).mockResolvedValue({ selection: { text: 'Rusta Studio', url: PAGE_URL, title: 'Vattenflaska' }, refused: null });
  expect(await within(bar).findByTestId('selection-snippet', {}, { timeout: 2000 })).toHaveTextContent('Rusta Studio');
});

it('a14_other_fields_match_by_label', () => {
  expect(matchingFields(FIELDS, ' COL ').map(f => f.definition.label)).toEqual(['Colour']);
  expect(matchingFields(FIELDS, '').length).toBe(FIELDS.length);
});

it('a14_open_ai_review_from_a_search_window_opens_the_review_in_the_main_window', async () => {
  let fire: (sessionId: number) => void = () => undefined;
  vi.mocked(search.onOpenAiReview).mockImplementation(async handler => {
    fire = handler;
    return () => undefined;
  });
  const navigate = vi.fn();
  const openReview = vi.fn();
  const { rerender } = renderHook(({ id }) => useReviewFromSearch(id, navigate, openReview), { initialProps: { id: 3 as number | undefined } });
  await waitFor(() => expect(search.onOpenAiReview).toHaveBeenCalled());
  // The session shown: the review opens.
  act(() => fire(3));
  expect(openReview).toHaveBeenCalledTimes(1);
  expect(navigate).not.toHaveBeenCalled();
  // Another session: it is opened first, then its review.
  act(() => fire(8));
  expect(navigate).toHaveBeenCalledWith(8);
  expect(openReview).toHaveBeenCalledTimes(1);
  rerender({ id: 8 });
  expect(openReview).toHaveBeenCalledTimes(2);
  // Later visits to that session do not open it again.
  rerender({ id: 3 });
  rerender({ id: 8 });
  expect(openReview).toHaveBeenCalledTimes(2);
});
