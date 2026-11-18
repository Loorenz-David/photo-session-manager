import '@testing-library/jest-dom/vitest';
import { act, cleanup, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { Form } from '@/services/attributes';
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

import { SearchWindow, browserBounds } from '@/features/search/SearchWindow';
import { getForm } from '@/services/attributes';
import { getSessionView } from '@/services/photos';
import * as search from '@/services/search';

// A ResizeObserver the test can fire, remembering what it watches.
const watched = new Set<Element>();
let fire: () => void = () => undefined;
class TestResizeObserver {
  constructor(callback: () => void) {
    fire = callback;
  }
  observe(target: Element) { watched.add(target); }
  unobserve() {}
  disconnect() { watched.clear(); }
}

const SESSION = 4;
function photo(id: number): Photo {
  return {
    id, session_id: SESSION, classification: 'main', position: id, captured_at: id, original_preview_path: null, adjusted_preview_path: `/p/${id}.jpg`, thumbnail_path: null,
    composition: { photo_id: id, kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: null, frame: null, frame_source: null, origin: 'default', revision: 0, state: 'ok', review_reason: null, updated_at: 0 },
    orientation: null, source: 'capture', integrity: 'ok', upload_state: null,
  };
}
const VIEW: SessionView = {
  session: { id: SESSION, item_id: 9, published_at: null, published_seq: 0, change_seq: 0, dirty: false, last_activity_at: 0, created_at: 0, arranged_at: null, photo_count: 1, unreviewed_count: 0, main_count: 1, condition_count: 0, publish: null, changes: null },
  item: { id: 9, sku: 'AC-0002', folder_key: 'AC-0002', barcode: null, item_type: 'Armchairs', title: 'test-2 armchair', origin: 'local', external_id: null, created_at: 0, updated_at: 0 },
  photos: [photo(21)],
};

let handlers: { page: (e: PageEvent) => void; lens: (e: LensEvent) => void; photo: (id: number) => void };
let areaRect = { left: 380.5, top: 41.2, width: 1059.4, height: 767.6 };

beforeEach(() => {
  watched.clear();
  globalThis.ResizeObserver = TestResizeObserver as unknown as typeof ResizeObserver;
  areaRect = { left: 380.5, top: 41.2, width: 1059.4, height: 767.6 };
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement) {
    if (this.dataset.testid === 'browser-area') return { ...areaRect, x: areaRect.left, y: areaRect.top, right: 0, bottom: 0, toJSON: () => ({}) } as DOMRect;
    return { left: 0, top: 0, width: 0, height: 0, x: 0, y: 0, right: 0, bottom: 0, toJSON: () => ({}) } as DOMRect;
  });
  vi.mocked(getSessionView).mockResolvedValue(VIEW);
  vi.mocked(getForm).mockResolvedValue({ item_id: 9, item_type: 'Armchairs', groups: [] } as unknown as Form);
  vi.mocked(search.savedPages).mockResolvedValue([]);
  vi.mocked(search.onSearchEvents).mockImplementation(async (_s, next) => {
    handlers = next;
    return () => undefined;
  });
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.clearAllMocks();
});

it('a13_browser_bounds_are_whole_css_pixels_never_device_pixels', () => {
  Object.defineProperty(window, 'devicePixelRatio', { value: 2, configurable: true });
  expect(browserBounds({ left: 380.5, top: 41.2, width: 1059.4, height: 767.6 })).toEqual({ x: 381, y: 41, width: 1059, height: 768 });
  expect(browserBounds({ left: Number.NaN, top: 0, width: 0, height: -5 })).toEqual({ x: 0, y: 0, width: 1, height: 1 });
});

it('a13_the_browser_is_placed_by_the_area_and_placed_again_only_when_it_moves', async () => {
  render(<SearchWindow sessionId={SESSION} initialPhotoId={21} />);
  await waitFor(() => expect(handlers).toBeDefined());
  await waitFor(() => expect(search.placeBrowser).toHaveBeenCalledWith(SESSION, 1, { x: 381, y: 41, width: 1059, height: 768 }, true));
  // Every row of the browser column is watched, so a bar appearing above the area moves it.
  const area = screen.getByTestId('browser-area');
  const column = area.parentElement!.parentElement!;
  for (const row of Array.from(column.children)) expect(watched.has(row)).toBe(true);
  const calls = vi.mocked(search.placeBrowser).mock.calls.length;
  // Nothing moved: nothing is sent.
  act(() => fire());
  expect(search.placeBrowser).toHaveBeenCalledTimes(calls);
  // A bar above appeared: the area moved down by its height (same size), and is placed again.
  areaRect = { ...areaRect, top: 69.2 };
  act(() => fire());
  expect(search.placeBrowser).toHaveBeenLastCalledWith(SESSION, 1, { x: 381, y: 69, width: 1059, height: 768 }, true);
});
