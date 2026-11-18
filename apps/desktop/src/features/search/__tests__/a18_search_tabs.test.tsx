import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { Definition, Form, FormField } from '@/services/attributes';
import type { Photo, SessionView } from '@/services/photos';
import type { LensEvent, PageEvent, ZoomAction } from '@/services/search';

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
  zoomPage: vi.fn(),
  closeTab: vi.fn(async () => undefined),
  openAiReview: vi.fn(async () => undefined),
  askAi: vi.fn(),
  onSearchEvents: vi.fn(),
}));
vi.mock('@/services/photos', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/photos')>()), getSessionView: vi.fn() }));
vi.mock('@/services/attributes', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/attributes')>()), getForm: vi.fn() }));
vi.mock('@/services/sessions', async importOriginal => ({ ...(await importOriginal<typeof import('@/services/sessions')>()), previewUrl: (path: string) => `asset://${path}` }));
vi.mock('sonner', () => ({ toast: Object.assign(vi.fn(), { success: vi.fn(), error: vi.fn(), message: vi.fn() }) }));
vi.mock('@/components/ui/sonner', () => ({ Toaster: () => null }));

import { SearchWindow, tabLabel, type SearchTab } from '@/features/search/SearchWindow';
import { FIT, keepInside, zoomAround } from '@/features/search/ZoomablePhoto';
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
const PAGE = 'https://e1.example/sideboards';
const TABLE = 'Sideboard | Kai Kristiansen | Feldballes Møbelfabrik | 1960s';

function photo(id: number): Photo {
  return {
    id, session_id: SESSION, classification: 'main', position: id, captured_at: id, original_preview_path: null, adjusted_preview_path: `/p/${id}.jpg`, thumbnail_path: null,
    composition: { photo_id: id, kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: null, frame: null, frame_source: null, origin: 'default', revision: 0, state: 'ok', review_reason: null, updated_at: 0 },
    orientation: null, source: 'capture', integrity: 'ok', upload_state: null,
  };
}
function def(key: string, label: string): Definition {
  return { integration: 'shopify', key, label, core_type: 'text', choices: [], validations: '{}', group: 'details', description: null, writable: true };
}
function field(definition: Definition): FormField {
  return { definition, value: null, is_default: false, is_severity: false, selected: null, default: null, saved: [] };
}
const FIELDS = [field(def('custom.designer', 'Designer')), field(def('custom.manufacturer', 'Manufacturer'))];
const VIEW: SessionView = {
  session: { id: SESSION, item_id: ITEM_ID, published_at: null, published_seq: 0, change_seq: 0, dirty: false, last_activity_at: 0, created_at: 0, arranged_at: null, photo_count: 2, unreviewed_count: 0, main_count: 2, condition_count: 0, publish: null, changes: null },
  item: { id: ITEM_ID, sku: 'VF-0042', folder_key: 'VF-0042', barcode: null, item_type: 'Sideboard', title: 'Teak sideboard', origin: 'local', external_id: null, created_at: 0, updated_at: 0 },
  photos: [photo(11), photo(12)],
};

type Handlers = { page: (e: PageEvent) => void; lens: (e: LensEvent) => void; photo: (id: number) => void; zoom?: (a: ZoomAction) => void };
let handlers: Handlers;

beforeEach(() => {
  vi.mocked(getSessionView).mockResolvedValue(VIEW);
  vi.mocked(getForm).mockResolvedValue({ item_id: ITEM_ID, item_type: 'Sideboard', groups: [{ group: 'details', fields: FIELDS, filled: 0, total: 2 }] } as unknown as Form);
  vi.mocked(search.savedPages).mockResolvedValue([]);
  vi.mocked(search.researchMatches).mockResolvedValue([]);
  vi.mocked(search.visualSearch).mockResolvedValue([]);
  vi.mocked(search.readSelection).mockResolvedValue({ selection: null, refused: null });
  vi.mocked(search.zoomPage).mockImplementation(async (_s, _t, current, action) => (action === 'in' ? Math.round(current * 110) / 100 : action === 'out' ? 0.9 : 1));
  vi.mocked(search.onSearchEvents).mockImplementation(async (_s, next) => {
    handlers = next;
    return () => undefined;
  });
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function renderWindow() {
  render(<SearchWindow sessionId={SESSION} initialPhotoId={11} />);
  await waitFor(() => expect(handlers).toBeDefined());
  await waitFor(() => expect(search.startLens).toHaveBeenCalledWith(SESSION, 1, 11));
}
const tabs = () => screen.getAllByRole('tab');
const activeTab = () => tabs().find(t => t.getAttribute('aria-selected') === 'true');

it('a18_search_this_photo_on_the_pinned_photo_opens_a_new_tab_and_keeps_the_first', async () => {
  await renderWindow();
  act(() => handlers.lens({ session_id: SESSION, tab: 1, state: 'results', url: 'https://www.google.com/search?udm=26' }));
  act(() => handlers.page({ session_id: SESSION, tab: 1, url: 'https://www.google.com/search?udm=26', title: 'Google', loading: false }));
  // Pin the other photo (no search), then search it from the pinned photo's hover button.
  fireEvent.click(screen.getByRole('button', { name: 'Pin photo 12' }));
  expect(search.startLens).toHaveBeenCalledTimes(1);
  fireEvent.click(within(screen.getByTestId('pinned-photo')).getByRole('button', { name: /Search this photo/ }));
  await waitFor(() => expect(search.startLens).toHaveBeenLastCalledWith(SESSION, 2, 12));
  expect(tabs()).toHaveLength(2);
  expect(activeTab()).toHaveAccessibleName(/Searching/);
  // The new tab is the one placed over the browser area (the app hides the others).
  await waitFor(() => expect(search.placeBrowser).toHaveBeenLastCalledWith(SESSION, 2, expect.any(Object), true));
  // The first tab kept its page: back to it, its results show again without a new search.
  fireEvent.click(tabs()[0]);
  expect(activeTab()).toHaveAccessibleName(/Google results/);
  await waitFor(() => expect(search.placeBrowser).toHaveBeenLastCalledWith(SESSION, 1, expect.any(Object), true));
  expect(search.startLens).toHaveBeenCalledTimes(2);
  // Only the current tab's selection is read.
  await waitFor(() => expect(search.readSelection).toHaveBeenLastCalledWith(SESSION, 1));
});

it('a18_a_tab_routes_its_own_events_and_closing_it_keeps_the_others', async () => {
  await renderWindow();
  fireEvent.click(screen.getByRole('button', { name: 'New search of the pinned photo' }));
  await waitFor(() => expect(search.startLens).toHaveBeenLastCalledWith(SESSION, 2, 11));
  // Google fails in tab 2 only: tab 2 shows the list, tab 1 is still searching on Google.
  act(() => handlers.lens({ session_id: SESSION, tab: 2, state: 'failed', why: 'x' }));
  expect(await screen.findByTestId('match-list')).toBeInTheDocument();
  fireEvent.click(tabs()[0]);
  expect(screen.queryByTestId('match-list')).not.toBeInTheDocument();
  expect(screen.getByText('Searching Google for this photo…')).toBeInTheDocument();
  // Close tab 2: the app closes its page; tab 1 stays. The last tab cannot be closed.
  fireEvent.click(screen.getByRole('button', { name: 'Close search Matches' }));
  await waitFor(() => expect(search.closeTab).toHaveBeenCalledWith(SESSION, 2));
  expect(tabs()).toHaveLength(1);
  expect(screen.queryByRole('button', { name: /^Close search/ })).not.toBeInTheDocument();
});

it('a18_page_zoom_from_the_toolbar_and_the_menu_keys_is_kept_per_tab', async () => {
  await renderWindow();
  fireEvent.click(screen.getByRole('button', { name: 'Zoom in' }));
  await waitFor(() => expect(screen.getByTestId('zoom-level')).toHaveTextContent('110%'));
  expect(search.zoomPage).toHaveBeenLastCalledWith(SESSION, 1, 1, 'in');
  // ⌘+ from the View menu zooms the current tab further.
  await act(async () => handlers.zoom?.('in'));
  await waitFor(() => expect(screen.getByTestId('zoom-level')).toHaveTextContent('121%'));
  expect(search.zoomPage).toHaveBeenLastCalledWith(SESSION, 1, 1.1, 'in');
  // A new tab starts at 100 %; the first keeps its zoom.
  fireEvent.click(screen.getByRole('button', { name: 'New search of the pinned photo' }));
  await waitFor(() => expect(screen.getByTestId('zoom-level')).toHaveTextContent('100%'));
  fireEvent.click(tabs()[0]);
  expect(screen.getByTestId('zoom-level')).toHaveTextContent('121%');
  // Actual size.
  fireEvent.click(screen.getByRole('button', { name: 'Actual size' }));
  await waitFor(() => expect(screen.getByTestId('zoom-level')).toHaveTextContent('100%'));
  expect(search.zoomPage).toHaveBeenLastCalledWith(SESSION, 1, 1.21, 'reset');
});

it('a18_an_answer_in_words_offers_values_to_put_in_and_stays_on_its_tab_until_dismissed', async () => {
  vi.mocked(search.readSelection).mockResolvedValue({ selection: { text: TABLE, url: PAGE, title: 'Sideboards' }, refused: null });
  vi.mocked(search.askAi).mockResolvedValue({
    suggested: [], not_written: [], cost_micro_usd: 400,
    answer: 'Kai Kristiansen is the designer; Feldballes Møbelfabrik is the manufacturer.',
    found: [
      { key: 'custom.designer', label: 'Designer', value: 'Kai Kristiansen', value_json: '"Kai Kristiansen"', quote: 'Kai Kristiansen' },
      { key: 'custom.manufacturer', label: 'Manufacturer', value: 'Feldballes Møbelfabrik', value_json: '"Feldballes Møbelfabrik"', quote: 'Feldballes Møbelfabrik' },
    ],
  });
  vi.mocked(search.putValue).mockResolvedValue({ stored: {} as never, previous: null });
  await renderWindow();
  act(() => handlers.page({ session_id: SESSION, tab: 1, url: PAGE, title: 'Sideboards', loading: false }));
  const bar = screen.getByLabelText('Selected text');
  await within(bar).findByTestId('selection-snippet');
  fireEvent.click(within(bar).getByRole('button', { name: /Ask AI/ }));
  expect(within(bar).getByRole('radio', { name: /Answer me/ })).toHaveAttribute('aria-checked', 'true');
  fireEvent.change(within(bar).getByLabelText('Instruction for AI'), { target: { value: 'which is the designer and which the manufacturer?' } });
  fireEvent.click(within(bar).getByRole('button', { name: 'Send to AI' }));
  await waitFor(() => expect(search.askAi).toHaveBeenCalledWith(SESSION, { text: TABLE, url: PAGE, title: 'Sideboards' }, 'which is the designer and which the manufacturer?', 'answer'));
  const card = await screen.findByRole('region', { name: 'AI answer' });
  expect(within(card).getByTestId('ai-answer-text')).toHaveTextContent('Kai Kristiansen is the designer');
  // Nothing was written: only a tap puts a value in, exactly like "Put in" (with Undo).
  expect(search.putValue).not.toHaveBeenCalled();
  expect(search.putText).not.toHaveBeenCalled();
  fireEvent.click(within(card).getByRole('button', { name: /Put “Feldballes Møbelfabrik” in Manufacturer/ }));
  await waitFor(() => expect(search.putValue).toHaveBeenCalledWith(SESSION, 'custom.manufacturer', '"Feldballes Møbelfabrik"'));
  await waitFor(() => expect(toast.success).toHaveBeenCalled());
  const [title, options] = vi.mocked(toast.success).mock.calls[0] as [string, { action: { label: string; onClick: () => void } }];
  expect(title).toBe('Put in Manufacturer');
  options.action.onClick();
  await waitFor(() => expect(search.putBack).toHaveBeenCalledWith(SESSION, 'custom.manufacturer', null));
  // The answer belongs to its tab: another tab does not show it, coming back does.
  fireEvent.click(screen.getByRole('button', { name: 'New search of the pinned photo' }));
  await waitFor(() => expect(screen.queryByRole('region', { name: 'AI answer' })).not.toBeInTheDocument());
  fireEvent.click(tabs()[0]);
  const again = await screen.findByRole('region', { name: 'AI answer' });
  fireEvent.click(within(again).getByRole('button', { name: 'Dismiss answer' }));
  expect(screen.queryByRole('region', { name: 'AI answer' })).not.toBeInTheDocument();
});

it('a18_the_pinned_photo_zooms_with_the_wheel_and_double_click_fits_it_again', async () => {
  await renderWindow();
  const zoomable = screen.getByTestId('zoomable-photo');
  expect(zoomable).toHaveAttribute('data-zoom', '1.00');
  fireEvent.wheel(zoomable, { deltaY: -200, clientX: 10, clientY: 10 });
  await waitFor(() => expect(Number(zoomable.getAttribute('data-zoom'))).toBeGreaterThan(1.2));
  expect(screen.getByRole('button', { name: 'Fit the photo' })).toBeInTheDocument();
  fireEvent.doubleClick(zoomable);
  expect(zoomable).toHaveAttribute('data-zoom', '1.00');
  // Zooming never searches.
  expect(search.startLens).toHaveBeenCalledTimes(1);
});

it('a18_zoom_math_keeps_the_point_under_the_pointer_and_never_pans_past_the_edges', () => {
  const box = { width: 400, height: 300 };
  // Zoom 2× at the centre: no pan.
  expect(zoomAround(FIT, 2, { x: 0, y: 0 }, box)).toEqual({ scale: 2, x: 0, y: 0 });
  // Zoom 2× at (100, 50) from the centre: that point stays put.
  const at = zoomAround(FIT, 2, { x: 100, y: 50 }, box);
  expect(at).toEqual({ scale: 2, x: -100, y: -50 });
  // Back to 1×: fitted and centred.
  expect(zoomAround(at, 0.5, { x: 100, y: 50 }, box)).toEqual(FIT);
  // Never beyond 6× nor past the photo's edges.
  expect(zoomAround(FIT, 50, { x: 0, y: 0 }, box).scale).toBe(6);
  expect(keepInside({ scale: 2, x: 999, y: -999 }, box)).toEqual({ scale: 2, x: 200, y: -150 });
});

it('a18_a_tab_is_named_by_what_it_shows', () => {
  const base: SearchTab = { id: 1, photoId: 11, page: null, lens: null, waiting: false, mode: 'web', matches: null, matchesOrigin: null, searching: false, zoom: 1, answer: null };
  expect(tabLabel({ ...base, waiting: true })).toBe('Searching…');
  expect(tabLabel({ ...base, page: { url: 'https://www.google.com/search?udm=26', title: '', loading: false } })).toBe('Google results');
  expect(tabLabel({ ...base, page: { url: 'https://www.pamono.com/x', title: '', loading: false } })).toBe('pamono.com');
  expect(tabLabel({ ...base, mode: 'list' })).toBe('Matches');
});
