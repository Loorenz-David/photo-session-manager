import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { NewSessionDialog } from '../NewSessionDialog';
import { createItem, listItemTypes, nextSku, searchItems, searchItemsShop } from '@/services/items';
import { createSession } from '@/services/sessions';

vi.mock('@/services/items', () => ({ nextSku: vi.fn(), searchItems: vi.fn(), searchItemsShop: vi.fn(), createItem: vi.fn(), adoptItem: vi.fn(), refreshItem: vi.fn(), listItemTypes: vi.fn() }));
vi.mock('@/services/sessions', async original => ({ ...(await original<typeof import('@/services/sessions')>()), createSession: vi.fn() }));
globalThis.ResizeObserver ??= class { observe() {} unobserve() {} disconnect() {} };
Element.prototype.scrollIntoView ??= function scrollIntoView() {};

afterEach(() => { cleanup(); vi.clearAllMocks(); });

async function openCreate() {
  vi.mocked(nextSku).mockResolvedValue('LAMP-2');
  vi.mocked(searchItems).mockResolvedValue([]);
  vi.mocked(searchItemsShop).mockResolvedValue([]);
  vi.mocked(createItem).mockResolvedValue({ id: 2, sku: 'LAMP-2', title: 'Lamp', item_type: 'Lamps', folder_key: 'LAMP-2', barcode: null, origin: 'local', external_id: null, created_at: 0, updated_at: 0 });
  vi.mocked(createSession).mockResolvedValue({ id: 3 } as Awaited<ReturnType<typeof createSession>>);
  render(<NewSessionDialog open onOpenChange={() => undefined} onOpened={() => undefined} />);
  fireEvent.change(screen.getByLabelText('Search items by title or SKU'), { target: { value: 'Lamp' } });
  fireEvent.click(await screen.findByText('Create “Lamp”'));
  await waitFor(() => expect(screen.getByLabelText('SKU')).toHaveValue('LAMP-2'));
}

it('C2(b) shop types are searchable and a listed choice is required', async () => {
  vi.mocked(listItemTypes).mockResolvedValue(['Chairs', 'Lamps']);
  await openCreate();
  expect(screen.getByRole('button', { name: 'Create session' })).toBeDisabled();
  fireEvent.change(screen.getByLabelText('Item type'), { target: { value: 'Other' } });
  expect(screen.getByRole('button', { name: 'Create session' })).toBeDisabled();
  fireEvent.change(screen.getByLabelText('Item type'), { target: { value: 'Lamps' } });
  await waitFor(() => expect(screen.getByRole('button', { name: 'Create session' })).not.toBeDisabled());
  fireEvent.click(screen.getByRole('button', { name: 'Create session' }));
  await waitFor(() => expect(createItem).toHaveBeenCalledWith({ sku: 'LAMP-2', title: 'Lamp', itemType: 'Lamps' }));
});

it('C2(c) absent integration permits a typed type', async () => {
  vi.mocked(listItemTypes).mockRejectedValue({ code: 'no_integration' });
  await openCreate();
  fireEvent.change(screen.getByLabelText('Item type'), { target: { value: 'Ceramics' } });
  await waitFor(() => expect(screen.getByRole('button', { name: 'Create session' })).not.toBeDisabled());
  fireEvent.click(screen.getByRole('button', { name: 'Create session' }));
  await waitFor(() => expect(createItem).toHaveBeenCalledWith({ sku: 'LAMP-2', title: 'Lamp', itemType: 'Ceramics' }));
  cleanup();
  vi.mocked(listItemTypes).mockRejectedValue({ code: 'provider' });
  await openCreate();
  expect(await screen.findByText("Couldn't load the shop's types — type one")).toBeInTheDocument();
  fireEvent.change(screen.getByLabelText('Item type'), { target: { value: 'Glass' } });
  await waitFor(() => expect(screen.getByRole('button', { name: 'Create session' })).not.toBeDisabled());
  fireEvent.click(screen.getByRole('button', { name: 'Create session' }));
  await waitFor(() => expect(createItem).toHaveBeenCalledWith({ sku: 'LAMP-2', title: 'Lamp', itemType: 'Glass' }));
});
