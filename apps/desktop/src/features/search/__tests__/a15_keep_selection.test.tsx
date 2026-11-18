import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
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
  putValue: vi.fn(),
  zoomPage: vi.fn(async () => 1),
  closeTab: vi.fn(async () => undefined),
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

import { keptSelection, samePage, SearchWindow } from '@/features/search/SearchWindow';
import { getForm } from '@/services/attributes';
import { getSessionView } from '@/services/photos';
import * as search from '@/services/search';

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
  vi.mocked(search.askAi).mockResolvedValue({ suggested: [{ key: 'custom.volume', label: 'Volume', value: '500 ml' }], not_written: [], answer: null, found: [], cost_micro_usd: 200 });
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

/** The page reports nothing selected from now on (focus went into the app's own bar). */
async function pageSelectionGoes() {
  vi.mocked(search.readSelection).mockResolvedValue({ selection: null, refused: null });
  await act(async () => { await new Promise(r => setTimeout(r, 800)); });
}
const mode = (bar: HTMLElement) => within(bar).getByTestId('selection-bar').getAttribute('data-mode');

it('a15_back_from_other_field_without_a_field_keeps_the_selection_and_its_actions', async () => {
  const bar = await renderBar();
  fireEvent.click(within(bar).getByRole('button', { name: /Other field/ }));
  await pageSelectionGoes();
  // Back with the arrow.
  fireEvent.click(within(bar).getByRole('button', { name: 'Back' }));
  expect(mode(bar)).toBe('actions');
  expect(within(bar).getByTestId('selection-snippet')).toHaveTextContent(TEXT);
  // Back with Escape, after typing in the search box.
  fireEvent.click(within(bar).getByRole('button', { name: /Other field/ }));
  const find = within(bar).getByRole('combobox', { name: 'Find a field' });
  fireEvent.change(find, { target: { value: 'wo' } });
  await act(async () => { await new Promise(r => setTimeout(r, 800)); });
  fireEvent.keyDown(find, { key: 'Escape' });
  expect(within(bar).getByTestId('selection-snippet')).toHaveTextContent(TEXT);
  // The actions still work on it.
  fireEvent.click(within(bar).getByRole('button', { name: 'Put in Designer' }));
  await waitFor(() => expect(search.putText).toHaveBeenCalledWith(SESSION, 'custom.designer', TEXT));
  // A put keeps the selection for a second action.
  expect(within(bar).getByTestId('selection-snippet')).toHaveTextContent(TEXT);
});

it('a15_escape_from_ask_ai_keeps_the_selection_and_after_an_answer_it_is_still_there', async () => {
  const bar = await renderBar();
  fireEvent.click(within(bar).getByRole('button', { name: /Ask AI/ }));
  await pageSelectionGoes();
  fireEvent.keyDown(within(bar).getByLabelText('Instruction for AI'), { key: 'Escape' });
  expect(mode(bar)).toBe('actions');
  expect(within(bar).getByTestId('selection-snippet')).toHaveTextContent(TEXT);
  // Cancel with ✕ too.
  fireEvent.click(within(bar).getByRole('button', { name: /Ask AI/ }));
  fireEvent.click(within(bar).getByRole('button', { name: 'Cancel' }));
  expect(within(bar).getByTestId('selection-snippet')).toHaveTextContent(TEXT);
  // Ask, then close the answer: the selection is still offered.
  fireEvent.click(within(bar).getByRole('button', { name: /Ask AI/ }));
  fireEvent.click(within(bar).getByRole('radio', { name: /Fill fields/ }));
  fireEvent.change(within(bar).getByLabelText('Instruction for AI'), { target: { value: 'the volume' } });
  fireEvent.keyDown(within(bar).getByLabelText('Instruction for AI'), { key: 'Enter' });
  await waitFor(() => expect(mode(bar)).toBe('done'));
  fireEvent.click(within(bar).getByRole('button', { name: 'Close' }));
  expect(within(bar).getByTestId('selection-snippet')).toHaveTextContent(TEXT);
  // A18: after an answer in words the bar is straight back on the selection's actions.
  fireEvent.click(within(bar).getByRole('button', { name: /Ask AI/ }));
  fireEvent.click(within(bar).getByRole('radio', { name: /Answer me/ }));
  fireEvent.change(within(bar).getByLabelText('Instruction for AI'), { target: { value: 'which is the designer?' } });
  fireEvent.keyDown(within(bar).getByLabelText('Instruction for AI'), { key: 'Enter' });
  await waitFor(() => expect(search.askAi).toHaveBeenLastCalledWith(SESSION, expect.any(Object), 'which is the designer?', 'answer'));
  await waitFor(() => expect(mode(bar)).toBe('actions'));
  expect(within(bar).getByTestId('selection-snippet')).toHaveTextContent(TEXT);
});

it('a15_an_empty_reading_in_the_main_bar_keeps_the_selection', async () => {
  const bar = await renderBar();
  await pageSelectionGoes();
  expect(mode(bar)).toBe('actions');
  expect(within(bar).getByTestId('selection-snippet')).toHaveTextContent(TEXT);
});

it('a15_a_new_selection_replaces_the_kept_one_and_a_too_long_one_shows_why', async () => {
  const bar = await renderBar();
  await pageSelectionGoes();
  vi.mocked(search.readSelection).mockResolvedValue({ selection: { text: 'Rusta Studio', url: PAGE_URL, title: 'Vattenflaska' }, refused: null });
  expect(await within(bar).findByText('Rusta Studio', {}, { timeout: 2000 })).toBeInTheDocument();
  vi.mocked(search.readSelection).mockResolvedValue({ selection: null, refused: 'The selection is too long — select at most 2,000 characters' });
  expect(await within(bar).findByText(/too long/, {}, { timeout: 2000 })).toBeInTheDocument();
  expect(within(bar).queryByTestId('selection-bar')).not.toBeInTheDocument();
});

it('a15_another_page_drops_the_selection_but_a_fragment_change_does_not', async () => {
  const bar = await renderBar();
  await pageSelectionGoes();
  act(() => handlers.page({ tab: 1, url: `${PAGE_URL}#reviews`, title: 'Vattenflaska', loading: false } as PageEvent));
  expect(within(bar).getByTestId('selection-snippet')).toHaveTextContent(TEXT);
  act(() => handlers.page({ tab: 1, url: 'https://www.rusta.com/sv/termos', title: 'Termos', loading: true } as PageEvent));
  await waitFor(() => expect(within(bar).queryByTestId('selection-bar')).not.toBeInTheDocument());
  expect(within(bar).getByRole('status')).toHaveTextContent('Select text on the page');
});

it('a15_dismiss_lets_go_and_the_same_text_selected_again_shows_again', async () => {
  const bar = await renderBar();
  fireEvent.click(within(bar).getByRole('button', { name: 'Dismiss selection' }));
  expect(within(bar).queryByTestId('selection-bar')).not.toBeInTheDocument();
  // The page still reports it: stays dismissed.
  await act(async () => { await new Promise(r => setTimeout(r, 800)); });
  expect(within(bar).queryByTestId('selection-bar')).not.toBeInTheDocument();
  // Deselected, then the same text selected again: it shows.
  await pageSelectionGoes();
  vi.mocked(search.readSelection).mockResolvedValue({ selection: { text: TEXT, url: PAGE_URL, title: 'Vattenflaska' }, refused: null });
  expect(await within(bar).findByTestId('selection-snippet', {}, { timeout: 2000 })).toHaveTextContent(TEXT);
});

it('a15_kept_selection_rule', () => {
  const a = { text: 'a', url: 'https://x.example/p', title: '' };
  const b = { text: 'b', url: 'https://x.example/p', title: '' };
  const dismissed = { current: null as string | null };
  expect(keptSelection(a, { selection: null, refused: null }, dismissed)).toBe(a);
  expect(keptSelection(a, { selection: { ...a }, refused: null }, dismissed)).toBe(a);
  expect(keptSelection(a, { selection: b, refused: null }, dismissed)).toBe(b);
  expect(keptSelection(a, { selection: null, refused: 'too long' }, dismissed)).toBeNull();
  dismissed.current = `${b.url}\n${b.text}`;
  expect(keptSelection(null, { selection: b, refused: null }, dismissed)).toBeNull();
  expect(keptSelection(null, { selection: null, refused: null }, dismissed)).toBeNull();
  expect(dismissed.current).toBeNull();
  expect(samePage('https://x.example/p#a', 'https://x.example/p')).toBe(true);
  expect(samePage('https://x.example/p?q=1', 'https://x.example/p')).toBe(false);
});
