import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { SettingsPage } from '../SettingsPage';
import { HealthPage } from '@/features/health/HealthPage';
import { getSettings, saveSettings, type Settings } from '@/services/settings';
import { getAppPaths, getHealth } from '@/services/health';

vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), listCropRatios: vi.fn(async () => []) }));
vi.mock('@/services/health', () => ({ getHealth: vi.fn(), getAppPaths: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

const shown: Settings = {
  crop_on_arrival: 'Square',
  document_id: '/Users/me/Pictures/session test 1', preview_recipe: 'PSM Preview', web_recipe: 'PSM Web', style_name: 'PSM House',
  export_folder: '/Users/me/Pictures/Session Manager Export', preview_delay_s: 40, effective_preview_delay_s: 30, theme: 'light', sku_prefix: null,
  integration: { kind: 'shopify', shop_domain: 'shop.myshopify.com', api_version: '2026-10', client_id: 'client-id', client_secret: '••••1234', condition_photos_key: 'custom.damage_report' },
};

it('C1(b) settings screen shows the masked secret and the stored Shopify fields, and saves the masked secret back unchanged', async () => {
  vi.mocked(getSettings).mockResolvedValue(shown);
  vi.mocked(saveSettings).mockResolvedValue(shown);
  render(<SettingsPage />);
  expect(await screen.findByLabelText('Client secret')).toHaveValue('••••1234');
  expect(screen.getByLabelText('Shop domain')).toHaveValue('shop.myshopify.com');
  expect(screen.getByLabelText('Client ID')).toHaveValue('client-id');
  expect(screen.getByLabelText('Condition photos key')).toHaveValue('custom.damage_report');
  expect(screen.getByText('In effect: 30 s (10–30 s)')).toBeInTheDocument();
  fireEvent.change(screen.getByLabelText('Preview refresh delay (seconds)'), { target: { value: '20' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
  const patch = vi.mocked(saveSettings).mock.calls[0][0];
  expect(patch.preview_delay_s).toBe(20);
  expect(patch.integration).toEqual(shown.integration);
});

it('C3(b) health screen shows the failing items with their detail and "Retry checks" asks again', async () => {
  vi.mocked(getHealth).mockResolvedValue({ items: [
    { code: 'nudge_listening', ok: true, detail: 'Listening' },
    { code: 'document_not_open', ok: false, detail: 'document_not_open' },
    { code: 'integration_token_invalid', ok: false, detail: 'The Session Manager app is not installed on this store' },
    { code: 'condition_field_missing', ok: false, detail: 'Condition photos field custom.nope not found in the shop' },
  ] });
  vi.mocked(getAppPaths).mockResolvedValue({ data_dir: '/tmp/psm', database: '/tmp/psm/session_manager.db', settings: '/tmp/psm/settings.json' });
  render(<HealthPage />);
  const token = await screen.findByText('The Session Manager app is not installed on this store');
  const field = screen.getByText('Condition photos field custom.nope not found in the shop');
  expect(token.closest('.health-row')).toHaveAttribute('data-ok', 'false');
  expect(field.closest('.health-row')).toHaveAttribute('data-ok', 'false');
  expect(screen.getByText('Listening').closest('.health-row')).toHaveAttribute('data-ok', 'true');
  expect(getHealth).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole('button', { name: 'Retry checks' }));
  await waitFor(() => expect(getHealth).toHaveBeenCalledTimes(2));
});
