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
  putText: vi.fn(async () => ({ stored: {}, previous: null })),
  putBack: vi.fn(async () => undefined),
  putValue: vi.fn(),
  zoomPage: vi.fn(async () => 1),
  closeTab: vi.fn(async () => undefined),
  openAiReview: vi.fn(async () => undefined),
  askAi: vi.fn(async () => ({ suggested: [], not_written: [], answer: null, found: [], cost_micro_usd: 0 })),
  onSearchEvents: vi.fn(),
}));
vi.mock('@/services/photos', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/photos')>()), getSessionView: vi.fn() }));
vi.mock('@/services/attributes', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/attributes')>()), getForm: vi.fn() }));
vi.mock('@/services/sessions', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/sessions')>()), previewUrl: (path: string) => `asset://${path}` }));
vi.mock('sonner', () => ({ toast: Object.assign(vi.fn(), { success: vi.fn(), error: vi.fn(), message: vi.fn() }) }));
vi.mock('@/components/ui/sonner', () => ({ Toaster: () => null }));

import { SearchWindow, putFields, searchRoute } from '@/features/search/SearchWindow';
import { MediaColumn } from '@/features/form/MediaColumn';
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

function photo(id: number, classification: Photo['classification'], position: number, path: string | null = `/previews/${id}.jpg`): Photo {
  return {
    id,
    session_id: SESSION,
    classification,
    position,
    captured_at: id,
    original_preview_path: null,
    adjusted_preview_path: path,
    thumbnail_path: null,
    composition: { photo_id: id, kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: null, frame: null, frame_source: null, origin: 'default', revision: 0, state: 'ok', review_reason: null, updated_at: 0 },
    orientation: null,
    source: 'capture',
    integrity: 'ok',
    upload_state: null,
  };
}

function def(key: string, label: string, core_type = 'text', writable = true): Definition {
  return { integration: 'shopify', key, label, core_type, choices: [], validations: '{}', group: 'details', description: null, writable };
}
function field(definition: Definition): FormField {
  return { definition, value: null, is_default: false, is_severity: false, selected: null, default: null, saved: [] };
}

const FIELDS = [field(def('custom.wood', 'Wood')), field(def('custom.designer', 'Designer')), field(def('custom.manufacturer', 'Manufacturer')), field(def('custom.photos', 'Damage photos', 'media_list'))];

const VIEW: SessionView = {
  session: { id: SESSION, item_id: ITEM_ID, published_at: null, published_seq: 0, change_seq: 0, dirty: false, last_activity_at: 0, created_at: 0, arranged_at: null, photo_count: 3, unreviewed_count: 0, main_count: 2, condition_count: 1, publish: null, changes: null },
  item: { id: ITEM_ID, sku: 'VF-0042', folder_key: 'VF-0042', barcode: null, item_type: 'Sideboard', title: 'Teak sideboard', origin: 'local', external_id: null, created_at: 0, updated_at: 0 },
  photos: [photo(11, 'main', 1), photo(12, 'main', 2), photo(13, 'condition', 1)],
};

type Handlers = { page: (event: PageEvent) => void; lens: (event: LensEvent) => void; photo: (photoId: number) => void };
let handlers: Handlers;

beforeEach(() => {
  vi.mocked(search.researchMatches).mockResolvedValue([]);
  vi.mocked(search.visualSearch).mockResolvedValue([]);
  vi.mocked(search.savedPages).mockResolvedValue([]);
  vi.mocked(search.lensResults).mockResolvedValue(false);
  vi.mocked(search.readSelection).mockResolvedValue({ selection: null, refused: null });
  vi.mocked(getSessionView).mockResolvedValue(VIEW);
  vi.mocked(getForm).mockResolvedValue({ item_id: ITEM_ID, item_type: 'Sideboard', groups: [{ group: 'details', fields: FIELDS, filled: 0, total: 4 }] } as unknown as Form);
  vi.mocked(search.onSearchEvents).mockImplementation(async (_session, next) => {
    handlers = next;
    return () => undefined;
  });
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.useRealTimers();
});

async function renderWindow(photoId = 11) {
  render(<SearchWindow sessionId={SESSION} initialPhotoId={photoId} />);
  await waitFor(() => expect(screen.getByText('VF-0042', { exact: false })).toBeInTheDocument());
  await waitFor(() => expect(handlers).toBeDefined());
}

it('a12_route_names_the_session_and_photo', () => {
  expect(searchRoute('#/search/3/11')).toEqual({ sessionId: 3, photoId: 11 });
  expect(searchRoute('#/session/3')).toBeNull();
  expect(searchRoute('#/search/3')).toBeNull();
});

it('a12_pinned_photo_stays_and_another_photo_can_be_pinned_without_searching_again', async () => {
  await renderWindow(11);
  expect(screen.getByTestId('pinned-photo')).toHaveAttribute('data-photo-id', '11');
  expect(within(screen.getByTestId('pinned-photo')).getByRole('img')).toHaveAttribute('src', 'asset:///previews/11.jpg');
  await waitFor(() => expect(search.startLens).toHaveBeenCalledWith(SESSION, 1, 11));
  // A18: pinning another photo only changes the reference picture.
  fireEvent.click(screen.getByRole('button', { name: 'Pin photo 13' }));
  expect(screen.getByTestId('pinned-photo')).toHaveAttribute('data-photo-id', '13');
  await act(async () => { await new Promise(r => setTimeout(r, 20)); });
  expect(search.startLens).toHaveBeenCalledTimes(1);
  expect(screen.getAllByRole('tab')).toHaveLength(1);
  // The main window opening the search on another photo pins it and searches it in a new tab.
  act(() => handlers.photo(12));
  expect(screen.getByTestId('pinned-photo')).toHaveAttribute('data-photo-id', '12');
  await waitFor(() => expect(search.startLens).toHaveBeenLastCalledWith(SESSION, 2, 12));
  expect(screen.getAllByRole('tab')).toHaveLength(2);
});

it('a12_when_google_cannot_be_used_the_window_lists_matches_and_opens_them', async () => {
  vi.mocked(search.researchMatches).mockResolvedValue([{ url: 'https://e1.example/a', title: 'Teak sideboard by Kai K', source: 'E1', thumbnail: null }]);
  await renderWindow();
  act(() => handlers.lens({ session_id: SESSION, tab: 1, state: 'failed', why: "Google's page did not open" }));
  const list = await screen.findByTestId('match-list');
  fireEvent.click(within(list).getByRole('button', { name: /Teak sideboard by Kai K/ }));
  expect(search.goTo).toHaveBeenCalledWith(SESSION, 1, 'https://e1.example/a');
  expect(search.visualSearch).not.toHaveBeenCalled();
});

it('a12_with_no_research_matches_one_new_search_of_the_pinned_photo_runs', async () => {
  vi.mocked(search.visualSearch).mockResolvedValue([{ url: 'https://e2.example/b', title: 'B', source: 'e2.example', thumbnail: null }]);
  await renderWindow(12);
  act(() => handlers.lens({ session_id: SESSION, tab: 1, state: 'failed', why: 'x' }));
  await within(await screen.findByTestId('match-list')).findByRole('button', { name: /e2\.example/ });
  expect(search.visualSearch).toHaveBeenCalledWith(SESSION, 12);
});

it('a12_consent_is_explained_and_results_stop_the_wait', async () => {
  await renderWindow();
  act(() => handlers.lens({ session_id: SESSION, tab: 1, state: 'consent' }));
  expect(screen.getByText(/Google asks about cookies/)).toBeInTheDocument();
  act(() => handlers.lens({ session_id: SESSION, tab: 1, state: 'results', url: 'https://www.google.com/search?udm=26' }));
  expect(screen.queryByText(/Google asks about cookies/)).not.toBeInTheDocument();
});

it('a12_saved_pages_are_tabs_to_add_open_and_remove', async () => {
  vi.mocked(search.savedPages).mockResolvedValue([{ id: 1, url: 'https://e1.example/a', title: 'A', saved_at: 1 }]);
  vi.mocked(search.savePage).mockResolvedValue({ id: 2, url: 'https://e2.example/b', title: 'Kai K sideboard', saved_at: 2 });
  await renderWindow();
  const tabs = await screen.findByLabelText('Saved pages');
  expect(within(tabs).getByText('e1.example')).toBeInTheDocument();
  act(() => handlers.page({ session_id: SESSION, tab: 1, url: 'https://e2.example/b', title: 'Kai K sideboard', loading: false }));
  fireEvent.click(screen.getByRole('button', { name: /Save page/ }));
  await waitFor(() => expect(search.savePage).toHaveBeenCalledWith(ITEM_ID, 'https://e2.example/b', 'Kai K sideboard'));
  await waitFor(() => expect(within(screen.getByLabelText('Saved pages')).getByText('e2.example')).toBeInTheDocument());
  fireEvent.click(within(screen.getByLabelText('Saved pages')).getByText('e1.example'));
  expect(search.goTo).toHaveBeenCalledWith(SESSION, 1, 'https://e1.example/a');
  fireEvent.click(screen.getByRole('button', { name: 'Remove saved page e1.example' }));
  await waitFor(() => expect(within(screen.getByLabelText('Saved pages')).queryByText('e1.example')).not.toBeInTheDocument());
  expect(search.removePage).toHaveBeenCalledWith(1);
});

it('a12_results_goes_back_to_googles_results', async () => {
  vi.mocked(search.lensResults).mockResolvedValue(true);
  await renderWindow();
  fireEvent.click(screen.getByRole('button', { name: /Results/ }));
  await waitFor(() => expect(search.lensResults).toHaveBeenCalledWith(SESSION, 1));
});

it('a12_selected_text_can_go_into_a_field_or_to_ai_from_the_apps_own_bar', async () => {
  vi.mocked(search.readSelection).mockResolvedValue({ selection: { text: 'designed by Kai Kristiansen', url: 'https://e1.example/a', title: 'A' }, refused: null });
  vi.mocked(search.askAi).mockResolvedValue({ suggested: [{ key: 'custom.designer', label: 'Designer', value: 'Kai Kristiansen' }], not_written: [], answer: null, found: [], cost_micro_usd: 300 });
  await renderWindow();
  const bar = screen.getByLabelText('Selected text');
  await within(bar).findByText(/designed by Kai Kristiansen/);
  // Manufacturer and Designer come first; the list holds the rest (never a photo field).
  const quick = within(bar).getAllByRole('button', { name: /^Put in / }).map(b => b.getAttribute('aria-label'));
  expect(quick).toEqual(['Put in Designer', 'Put in Manufacturer']);
  fireEvent.click(within(bar).getByRole('button', { name: 'Put in Manufacturer' }));
  await waitFor(() => expect(search.putText).toHaveBeenCalledWith(SESSION, 'custom.manufacturer', 'designed by Kai Kristiansen'));
  // A14: the other fields are an inline list in the bar (anything drawn over the page would be hidden).
  fireEvent.click(within(bar).getByRole('button', { name: /Other field/ }));
  const list = within(bar).getByRole('listbox', { name: 'Fields' });
  expect(within(list).queryByText('Damage photos')).not.toBeInTheDocument();
  fireEvent.click(within(list).getByRole('option', { name: 'Wood' }));
  await waitFor(() => expect(search.putText).toHaveBeenCalledWith(SESSION, 'custom.wood', 'designed by Kai Kristiansen'));
  fireEvent.click(within(bar).getByRole('button', { name: /Ask AI/ }));
  fireEvent.change(within(bar).getByLabelText('Instruction for AI'), { target: { value: 'who designed it' } });
  fireEvent.click(within(bar).getByRole('button', { name: 'Send to AI' }));
  await waitFor(() => expect(search.askAi).toHaveBeenCalledWith(SESSION, { text: 'designed by Kai Kristiansen', url: 'https://e1.example/a', title: 'A' }, 'who designed it', 'answer'));
});

it('a12_nothing_selected_or_a_refused_selection_offers_no_action', async () => {
  vi.mocked(search.readSelection).mockResolvedValue({ selection: null, refused: 'The selection is too long — select at most 2,000 characters' });
  await renderWindow();
  const bar = screen.getByLabelText('Selected text');
  await within(bar).findByText(/too long/);
  expect(within(bar).queryByRole('button')).not.toBeInTheDocument();
  expect(search.putText).not.toHaveBeenCalled();
});

it('a12_the_page_is_placed_over_the_browser_area_and_hidden_while_the_list_shows', async () => {
  await renderWindow();
  await waitFor(() => expect(search.placeBrowser).toHaveBeenCalledWith(SESSION, 1, expect.any(Object), true));
  act(() => handlers.lens({ session_id: SESSION, tab: 1, state: 'failed', why: 'x' }));
  await waitFor(() => expect(search.placeBrowser).toHaveBeenLastCalledWith(SESSION, 1, expect.any(Object), false));
});

it('a12_put_fields_orders_and_filters', () => {
  const fields = putFields([...FIELDS, field(def('custom.locked', 'Maker code', 'text', false))]);
  expect(fields.map(f => f.definition.label)).toEqual(['Designer', 'Manufacturer', 'Wood']);
});

it('a12_a_photo_of_the_media_preview_opens_the_search_window', () => {
  render(<MediaColumn photos={VIEW.photos} step="details" />);
  fireEvent.click(screen.getAllByRole('button', { name: /Search this photo/ })[1]);
  expect(search.openSearchWindow).toHaveBeenCalledWith(SESSION, 12);
});

it('a12_without_results_in_time_the_window_gives_up_on_google_and_shows_its_list', async () => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  await renderWindow();
  expect(search.giveUpLens).not.toHaveBeenCalled();
  await act(async () => {
    vi.advanceTimersByTime(30_000);
  });
  await waitFor(() => expect(search.giveUpLens).toHaveBeenCalledWith(SESSION, 1, expect.any(String)));
  expect(await screen.findByTestId('match-list')).toBeInTheDocument();
});
