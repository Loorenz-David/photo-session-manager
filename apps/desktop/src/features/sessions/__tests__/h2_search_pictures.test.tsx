import '@testing-library/jest-dom/vitest';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ReactElement } from 'react';
import type { PictureReady } from '@/services/events';

/** H2·U4 (`plans/H2_list_pictures_pages_search.md`, rows H2·C14): result tiles of New session and Assign product. */

const delivered = vi.hoisted(() => ({ pictures: [] as ((event: PictureReady) => void)[] }));

vi.mock('@/services/items', () => ({ nextSku: vi.fn(), searchItems: vi.fn(), searchItemsShop: vi.fn(), createItem: vi.fn(), adoptItem: vi.fn(), listItemTypes: vi.fn(async () => { throw { code: 'no_integration' }; }), refreshItem: vi.fn() }));
vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  previewUrl: (path: string) => `asset://${path}`,
}));
vi.mock('@/services/events', () => ({
  onPictureReady: vi.fn(async (handler: (event: PictureReady) => void) => {
    delivered.pictures.push(handler);
    return () => undefined;
  }),
}));

import { NewSessionDialog } from '../NewSessionDialog';
import { ItemPickerDialog } from '@/features/workspace/PickerDialogs';
import { nextSku, searchItems, searchItemsShop, type Item } from '@/services/items';
import { previewUrl } from '@/services/sessions';

globalThis.ResizeObserver ??= class { observe() {} unobserve() {} disconnect() {} };
Element.prototype.scrollIntoView ??= function scrollIntoView() {};

function item(id: number, title: string, extra: Partial<Item> = {}): Item {
  return { id, title, sku: `SKU-${id}`, folder_key: title, barcode: null, item_type: '', origin: 'local', external_id: null, created_at: 0, updated_at: 0, ...extra };
}

/** The result tile of the row titled `title`. */
function tile(title: string) {
  return screen.getByText(title).closest('.result-row')!.querySelector('.result-thumb')!;
}

function deliver(event: PictureReady) {
  act(() => delivered.pictures.forEach(handler => handler(event)));
}

beforeEach(() => {
  delivered.pictures.length = 0;
  vi.mocked(nextSku).mockResolvedValue(null);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

const dialogs: [string, () => ReactElement][] = [
  ['NewSessionDialog', () => <NewSessionDialog open onOpenChange={() => undefined} onOpened={() => undefined} />],
  ['ItemPickerDialog', () => <ItemPickerDialog open title="Assign product" onOpenChange={() => undefined} onPick={() => undefined} />],
];

async function searchFor(dialog: () => ReactElement, query: string) {
  render(dialog());
  fireEvent.change(screen.getByLabelText('Search items by title or SKU'), { target: { value: query } });
}

/** H2·C14(a): a result with a picture shows it; one without shows its first letter. */
async function picturesOrLetters(dialog: () => ReactElement) {
  vi.mocked(searchItems).mockResolvedValue([item(1, 'Oak chair', { picture_path: '/p/x.jpg' }), item(2, 'Walnut desk')]);
  vi.mocked(searchItemsShop).mockResolvedValue([]);
  await searchFor(dialog, 'o');
  await screen.findByText('Walnut desk');
  expect(tile('Oak chair').querySelector('img')).toHaveAttribute('src', previewUrl('/p/x.jpg'));
  expect(tile('Walnut desk').querySelector('img')).toBeNull();
  expect(tile('Walnut desk')).toHaveTextContent('W');
}

/** H2·C14(b): a shop result's picture arrives as `picture_ready` kind `search`. */
async function shopPictureArrives(dialog: () => ReactElement) {
  vi.mocked(searchItems).mockResolvedValue([]);
  vi.mocked(searchItemsShop).mockResolvedValue([item(0, 'Teak bench', { origin: 'linked', external_id: 'E' })]);
  await searchFor(dialog, 'teak');
  await screen.findByText('Teak bench', {}, { timeout: 2000 });
  expect(tile('Teak bench')).toHaveTextContent('T');
  deliver({ kind: 'search', external_id: 'E', path: '/p/s.jpg' });
  await waitFor(() => expect(tile('Teak bench').querySelector('img')).toHaveAttribute('src', previewUrl('/p/s.jpg')));
}

it('h2_c14a_new_session_tiles_show_pictures_or_letters', async () => {
  await picturesOrLetters(dialogs[0][1]);
});

it('h2_c14b_new_session_a_shop_picture_replaces_the_letter', async () => {
  await shopPictureArrives(dialogs[0][1]);
});

it('h2_c14c_assign_product_shows_the_same_three_outcomes', async () => {
  const picker = dialogs[1][1];
  await picturesOrLetters(picker);
  cleanup();
  await shopPictureArrives(picker);
  cleanup();
  vi.mocked(searchItems).mockResolvedValue([item(3, 'Pine shelf', { session_id: 7 })]);
  vi.mocked(searchItemsShop).mockResolvedValue([]);
  await searchFor(picker, 'pine');
  await screen.findByText('Pine shelf');
  expect(tile('Pine shelf')).toHaveTextContent('P');
  deliver({ kind: 'cover', session_id: 7, path: '/p/cover7.jpg' });
  await waitFor(() => expect(tile('Pine shelf').querySelector('img')).toHaveAttribute('src', previewUrl('/p/cover7.jpg')));
});
