import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { StockFields } from '../StockFields';
import { StockSection } from '@/features/settings/StockSection';
import { StatusPill } from '@/features/sessions/StatusPill';
import { itemStockGet, itemStockSet, stockDefaultGet, stockDefaultSet, stockLocations } from '@/services/stock';
import { publishActivity } from '@/services/publish';

vi.mock('@/services/stock', () => ({ stockLocations: vi.fn(), stockDefaultGet: vi.fn(), stockDefaultSet: vi.fn(), itemStockGet: vi.fn(), itemStockSet: vi.fn() }));
vi.mock('@/services/publish', () => ({ publishActivity: vi.fn(), publishCancel: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const LOCATIONS = [{ id: 'gid://shopify/Location/1', name: 'Warehouse' }, { id: 'gid://shopify/Location/2', name: 'Shop floor' }];

beforeEach(() => {
  vi.mocked(stockLocations).mockResolvedValue({ locations: LOCATIONS, missing_access: null });
  vi.mocked(itemStockSet).mockResolvedValue(undefined);
  vi.mocked(stockDefaultSet).mockImplementation(async choice => choice);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('C3(c) a new item\'s Product step shows Location and Quantity prefilled from the remembered default; a change is saved for the item', async () => {
  vi.mocked(itemStockGet).mockResolvedValue({ new_item: true, choice: { location_id: 'gid://shopify/Location/2', location_name: 'Shop floor', quantity: 3 } });
  render(<StockFields itemId={7} />);
  const location = await screen.findByRole('combobox', { name: 'Location' });
  expect(within(location).getAllByRole('option').map(o => o.textContent)).toEqual(['No location', 'Warehouse', 'Shop floor']);
  expect(location).toHaveValue('gid://shopify/Location/2');
  expect(screen.getByRole('spinbutton', { name: 'Quantity' })).toHaveValue(3);
  expect(screen.queryByText('Choose a location to track stock')).not.toBeInTheDocument();
  fireEvent.change(location, { target: { value: 'gid://shopify/Location/1' } });
  await waitFor(() => expect(itemStockSet).toHaveBeenLastCalledWith(7, { location_id: 'gid://shopify/Location/1', location_name: 'Warehouse', quantity: 3 }));
  fireEvent.change(screen.getByRole('spinbutton', { name: 'Quantity' }), { target: { value: '1' } });
  await waitFor(() => expect(itemStockSet).toHaveBeenLastCalledWith(7, { location_id: 'gid://shopify/Location/1', location_name: 'Warehouse', quantity: 1 }));
});

it('C3(c) without a default or a chosen location the form says "Choose a location to track stock" (quantity 1); an item already in the shop shows nothing', async () => {
  vi.mocked(itemStockGet).mockResolvedValue({ new_item: true, choice: null });
  render(<StockFields itemId={8} />);
  expect(await screen.findByText('Choose a location to track stock')).toBeInTheDocument();
  expect(screen.getByRole('combobox', { name: 'Location' })).toHaveValue('');
  expect(screen.getByRole('spinbutton', { name: 'Quantity' })).toHaveValue(1);
  cleanup();
  vi.mocked(itemStockGet).mockResolvedValue({ new_item: false, choice: null });
  const { container } = render(<StockFields itemId={9} />);
  await waitFor(() => expect(itemStockGet).toHaveBeenCalledWith(9));
  await waitFor(() => expect(container).toBeEmptyDOMElement());
});

it('C3(c) a shop without location access says the app needs the permissions, in the form and in Settings', async () => {
  const sentence = 'Your Shopify app needs the read_locations and write_inventory permissions';
  vi.mocked(stockLocations).mockResolvedValue({ locations: [], missing_access: sentence });
  vi.mocked(itemStockGet).mockResolvedValue({ new_item: true, choice: null });
  vi.mocked(stockDefaultGet).mockResolvedValue(null);
  render(<StockFields itemId={7} />);
  expect(await screen.findByText(sentence)).toBeInTheDocument();
  expect(screen.queryByRole('combobox', { name: 'Location' })).not.toBeInTheDocument();
  cleanup();
  render(<StockSection />);
  expect(await screen.findByText(sentence)).toBeInTheDocument();
});

it('C3(c) Settings → Shopify shows the remembered default and saves an edit', async () => {
  vi.mocked(stockDefaultGet).mockResolvedValue({ location_id: 'gid://shopify/Location/1', location_name: 'Warehouse', quantity: 1 });
  render(<StockSection />);
  const location = await screen.findByRole('combobox', { name: 'Location' });
  await waitFor(() => expect(location).toHaveValue('gid://shopify/Location/1'));
  expect(screen.getByRole('spinbutton', { name: 'Quantity' })).toHaveValue(1);
  fireEvent.change(location, { target: { value: 'gid://shopify/Location/2' } });
  fireEvent.change(screen.getByRole('spinbutton', { name: 'Quantity' }), { target: { value: '4' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save stock default' }));
  await waitFor(() => expect(stockDefaultSet).toHaveBeenCalledWith({ location_id: 'gid://shopify/Location/2', location_name: 'Shop floor', quantity: 4 }));
});

it('C2(a) a publish waiting for Capture One shows "Waiting for Capture One…" in the session row and its popover', async () => {
  vi.mocked(publishActivity).mockResolvedValue({ item_id: 3, step: 'prepare', text: 'Waiting for Capture One…', percent: 0, elapsed_ms: 20_000, error: null, retry: null, can_cancel: true });
  render(<StatusPill label={{ kind: 'waiting_for_editor' }} sessionId={5} />);
  expect(screen.getByText('Waiting for Capture One…')).toBeInTheDocument();
  fireEvent.mouseEnter(screen.getByRole('button', { name: 'Publish progress details' }));
  const panel = await screen.findByTestId('publish-activity');
  expect(within(panel).getByText('Waiting for Capture One…')).toBeInTheDocument();
  expect(within(panel).getByRole('button', { name: 'Cancel publish' })).toBeInTheDocument();
});
