import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ConversationsSettings } from '@/services/conversations';

vi.mock('@/services/conversations', () => ({
  conversationsSettingsGet: vi.fn(),
  conversationsSettingsSave: vi.fn(),
  conversationsKeySet: vi.fn(),
  conversationsTest: vi.fn(),
}));
vi.mock('@/services/events', () => ({ onConversationsChanged: vi.fn().mockResolvedValue(() => undefined) }));

import { PricingSection } from '@/features/settings/PricingSection';
import { SettingsNav } from '@/features/settings/SettingsNav';
import { conversationsKeySet, conversationsSettingsGet, conversationsSettingsSave, conversationsTest } from '@/services/conversations';

const DEFAULT_SERVER = 'https://pricing.beyoworkaroundtheclock.com';
const SHOP = 'shop-a.myshopify.com';

function settings(extra: Partial<ConversationsSettings> = {}, connection: Partial<ConversationsSettings['connection']> = {}): ConversationsSettings {
  return {
    server: DEFAULT_SERVER, server_default: DEFAULT_SERVER, key_saved: false, sound: true, banner: true,
    connection: { state: 'no_key', store_name: null, shop: SHOP, visible_stores: [], last_ok_at: null, last_error: null, last_error_at: null, ...connection },
    labels: 'off',
    ...extra,
  };
}

function section() { return screen.getByRole('region', { name: 'Pricing' }); }
async function shown() { await waitFor(() => expect(within(section()).getByRole('heading', { name: 'Pricing' })).toBeInTheDocument()); await screen.findByText('Test connection'); }
function connectionText() { return screen.queryByTestId('pricing-connection')?.textContent ?? null; }

beforeEach(() => { vi.mocked(conversationsSettingsGet).mockResolvedValue(settings()); });
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('c1_c20a_a_pricing_entry_after_shopify_and_the_defaults', async () => {
  for (const showResearch of [true, false]) {
    const { unmount } = render(<SettingsNav active="capture-one" showResearch={showResearch} onSelect={() => undefined} />);
    const labels = within(screen.getByRole('navigation', { name: 'Settings sections' })).getAllByRole('button').map(b => b.textContent);
    expect(labels.indexOf('Pricing')).toBe(labels.indexOf('Shopify') + 1);
    unmount();
  }
  render(<PricingSection />);
  await shown();
  expect(screen.getByLabelText('Server address')).toHaveValue(DEFAULT_SERVER);
  expect(screen.getByLabelText('API key')).toHaveValue('');
  expect(screen.getByRole('button', { name: 'Save key' })).toBeInTheDocument();
  expect(screen.getByRole('switch', { name: 'Sound for new questions and replies' })).toHaveAttribute('aria-checked', 'true');
  expect(screen.getByRole('switch', { name: 'Mac banner when Session Manager is in the background' })).toHaveAttribute('aria-checked', 'true');
});

it('c1_c20b_a_saved_key_shows_saved_with_replace_and_remove', async () => {
  vi.mocked(conversationsSettingsGet).mockResolvedValue(settings({ key_saved: true }));
  render(<PricingSection />);
  await shown();
  expect(screen.getByTestId('pricing-key-state')).toHaveTextContent('Saved');
  expect(screen.getByRole('button', { name: 'Replace' })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Remove' })).toBeInTheDocument();
  expect(screen.queryByLabelText('API key', { selector: 'input' })).toBeNull();
  expect(document.querySelector('input[type="password"]')).toBeNull();
});

it('c1_c20c_saving_a_typed_key', async () => {
  vi.mocked(conversationsKeySet).mockResolvedValue(settings({ key_saved: true }, { state: 'ready', store_name: 'Beyo' }));
  render(<PricingSection />);
  await shown();
  fireEvent.change(screen.getByLabelText('API key'), { target: { value: 'bp_test_key' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save key' }));
  await waitFor(() => expect(screen.getByTestId('pricing-key-state')).toHaveTextContent('Saved'));
  expect(conversationsKeySet).toHaveBeenCalledTimes(1);
  expect(conversationsKeySet).toHaveBeenCalledWith('bp_test_key');
  expect(document.querySelector('input[type="password"]')).toBeNull();
});

it('c1_c20d_connected_line', async () => {
  vi.mocked(conversationsSettingsGet).mockResolvedValue(settings({ key_saved: true }, { state: 'ready', store_name: 'Beyo' }));
  render(<PricingSection />);
  await shown();
  expect(connectionText()).toBe(`Connected to Beyo (${SHOP})`);
});

it('c1_c20e_unmatched_and_no_shop_lines', async () => {
  vi.mocked(conversationsSettingsGet).mockResolvedValue(settings({ key_saved: true }, { state: 'store_unmatched', visible_stores: ['One', 'Two'] }));
  const { unmount } = render(<PricingSection />);
  await shown();
  expect(connectionText()).toBe(`This Pricing key isn't tied to ${SHOP}. It can see: One, Two.`);
  unmount();
  vi.mocked(conversationsSettingsGet).mockResolvedValue(settings({ key_saved: true }, { state: 'no_shop', shop: null }));
  render(<PricingSection />);
  await shown();
  expect(connectionText()).toBe('Connect Shopify first; conversations are matched to its products.');
});

it('c1_c20f_test_connection_and_the_sound_switch', async () => {
  vi.mocked(conversationsTest).mockResolvedValue(settings({ key_saved: true }, { state: 'ready', store_name: 'Beyo' }));
  vi.mocked(conversationsSettingsSave).mockResolvedValue(settings({ sound: false }));
  render(<PricingSection />);
  await shown();
  fireEvent.click(screen.getByRole('button', { name: 'Test connection' }));
  await waitFor(() => expect(connectionText()).toBe(`Connected to Beyo (${SHOP})`));
  expect(conversationsTest).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole('switch', { name: 'Sound for new questions and replies' }));
  await waitFor(() => expect(conversationsSettingsSave).toHaveBeenCalledWith({ sound: false }));
});

it('c1_c20g_last_checked_time_then_a_newer_error', async () => {
  const okAt = new Date(2026, 10, 20, 14, 2, 5).getTime();
  vi.mocked(conversationsSettingsGet).mockResolvedValue(settings({ key_saved: true }, { state: 'ready', store_name: 'Beyo', last_ok_at: okAt }));
  const { unmount } = render(<PricingSection />);
  await shown();
  expect(screen.getByTestId('pricing-last-checked')).toHaveTextContent('Last checked: 14:02:05');
  unmount();
  vi.mocked(conversationsSettingsGet).mockResolvedValue(settings({ key_saved: true }, {
    state: 'ready', store_name: 'Beyo', last_ok_at: okAt, last_error: 'Could not reach Pricing', last_error_at: okAt + 30_000,
  }));
  render(<PricingSection />);
  await shown();
  expect(screen.getByTestId('pricing-last-checked')).toHaveTextContent('Last checked: Could not reach Pricing');
});

it('c1_c20h_refused_line', async () => {
  vi.mocked(conversationsSettingsGet).mockResolvedValue(settings({ key_saved: true }, { state: 'refused' }));
  render(<PricingSection />);
  await shown();
  expect(connectionText()).toBe('Pricing refused the key: check Settings → Pricing');
});
