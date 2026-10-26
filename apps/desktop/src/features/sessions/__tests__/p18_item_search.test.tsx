import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { NewSessionDialog, ITEM_SEARCH_DEBOUNCE_MS, SHOP_SEARCH_DEBOUNCE_MS } from '../NewSessionDialog';
import { nextSku, searchItems, searchItemsShop, type Item } from '@/services/items';

vi.mock('@/services/items', () => ({ nextSku: vi.fn(), searchItems: vi.fn(), searchItemsShop: vi.fn(), createItem: vi.fn(), adoptItem: vi.fn() }));
globalThis.ResizeObserver ??= class { observe() {} unobserve() {} disconnect() {} };
Element.prototype.scrollIntoView ??= function scrollIntoView() {};

function item(id: number, title: string, origin: 'local' | 'linked'): Item {
  return { id, title, sku: `SKU-${title}`, folder_key: title, barcode: null, item_type: '', origin, external_id: origin === 'linked' ? `remote-${title}` : null, created_at: 0, updated_at: 0 };
}
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(yes => { resolve = yes; });
  return { promise, resolve };
}
async function advance(ms: number) { await act(async () => { await vi.advanceTimersByTimeAsync(ms); }); }
afterEach(() => { cleanup(); vi.useRealTimers(); vi.clearAllMocks(); });

it('C3(b) a local match appears before the 300 ms shop request and the shop result joins it', async () => {
  vi.useFakeTimers();
  vi.mocked(nextSku).mockResolvedValue(null);
  vi.mocked(searchItems).mockResolvedValue([item(1, 'local A', 'local')]);
  const shop = deferred<Item[]>();
  vi.mocked(searchItemsShop).mockReturnValue(shop.promise);
  render(<NewSessionDialog open onOpenChange={() => undefined} onOpened={() => undefined} />);
  fireEvent.change(screen.getByLabelText('Search items by title or SKU'), { target: { value: 'ab' } });
  await advance(ITEM_SEARCH_DEBOUNCE_MS);
  expect(screen.getByText('local A')).toBeInTheDocument();
  expect(screen.getByText('1 found · searching the shop…')).toBeInTheDocument();
  expect(searchItemsShop).not.toHaveBeenCalled();
  await advance(SHOP_SEARCH_DEBOUNCE_MS - ITEM_SEARCH_DEBOUNCE_MS);
  expect(searchItemsShop).toHaveBeenCalledWith('ab');
  await act(async () => shop.resolve([item(0, 'shop B', 'linked')]));
  expect(screen.getByText('local A')).toBeInTheDocument();
  expect(screen.getByText('shop B')).toBeInTheDocument();
  expect(screen.getByText('2 found · 1 in the shop')).toBeInTheDocument();
});

it('C3(c) an older shop response cannot replace the newer query result', async () => {
  vi.useFakeTimers();
  vi.mocked(nextSku).mockResolvedValue(null);
  vi.mocked(searchItems).mockResolvedValue([]);
  const old = deferred<Item[]>();
  const latest = deferred<Item[]>();
  vi.mocked(searchItemsShop).mockImplementation(query => query === 'a' ? old.promise : latest.promise);
  render(<NewSessionDialog open onOpenChange={() => undefined} onOpened={() => undefined} />);
  const input = screen.getByLabelText('Search items by title or SKU');
  fireEvent.change(input, { target: { value: 'a' } });
  await advance(SHOP_SEARCH_DEBOUNCE_MS);
  expect(searchItemsShop).toHaveBeenCalledWith('a');
  fireEvent.change(input, { target: { value: 'ab' } });
  await advance(SHOP_SEARCH_DEBOUNCE_MS);
  expect(searchItemsShop).toHaveBeenCalledWith('ab');
  await act(async () => latest.resolve([item(0, 'shop AB', 'linked')]));
  await act(async () => old.resolve([item(0, 'shop A', 'linked')]));
  expect(screen.getByText('shop AB')).toBeInTheDocument();
  expect(screen.queryByText('shop A')).not.toBeInTheDocument();
});
