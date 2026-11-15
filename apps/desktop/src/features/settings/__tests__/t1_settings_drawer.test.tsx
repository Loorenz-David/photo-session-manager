import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { SettingsPage } from '../SettingsPage';
import { getSettings, saveSettings, type Settings } from '@/services/settings';

vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), pickSessionFile: vi.fn() }));
vi.mock('@/services/styles', () => ({ listStyles: vi.fn().mockResolvedValue({ user: [], built_in: [] }) }));
vi.mock('../CameraSection', () => ({ CameraSection: () => <h2>Camera</h2> }));
vi.mock('../StockSection', () => ({ StockSection: () => <p>Stock for new products</p> }));
vi.mock('../StorageSection', () => ({ StorageSection: () => <h2>Storage</h2> }));
vi.mock('../ResearchSection', () => ({ ResearchSection: () => <h2>Research</h2> }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const shown = {
  camera: { receiver_port: 2121, passive_port_start: 50000, passive_port_end: 50009 },
  document_id: null, effective_document_id: '/tmp/doc', default_document_id: '/tmp/doc',
  preview_recipe: 'PSM Preview', master_recipe: 'PSM Master', style_name: '', default_shape: '1:1',
  output_format: { long_edge: 2048, quality: 90 }, export_folder: '/tmp/export', preview_delay_s: 15,
  effective_preview_delay_s: 15, theme: 'light', sku_prefix: null,
  integration: { kind: 'shopify', shop_domain: 'example.myshopify.com', api_version: '2026-10', client_id: 'id', client_secret: 'secret', condition_photos_key: 'custom.damage_report' },
  research: {},
} as Settings;

beforeEach(() => {
  vi.mocked(getSettings).mockResolvedValue(shown);
  vi.mocked(saveSettings).mockImplementation(async patch => ({ ...shown, ...patch } as Settings));
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

function nav() { return screen.getByRole('navigation', { name: 'Settings sections' }); }
function pane(name: string) { return document.querySelector(`[data-section="${name}"]`); }

it('t1_c5a_lists_sections_in_order_and_opens_capture_one', async () => {
  render(<SettingsPage />);
  await screen.findByLabelText('Preview recipe');
  expect(within(nav()).getAllByRole('button').map(button => button.textContent)).toEqual([
    'Capture One', 'Camera', 'Export and previews', 'Shopify', 'Storage', 'Research', 'Shooting templates',
  ]);
  expect(within(nav()).getByRole('button', { name: 'Capture One' })).toHaveAttribute('aria-current', 'page');
  for (const name of ['capture-one','camera','export-and-previews','shopify','storage','research','shooting-templates']) {
    expect(pane(name)).toHaveAttribute('data-active', name === 'capture-one' ? 'true' : 'false');
  }
});

it('t1_c5b_shows_only_the_selected_pane', async () => {
  render(<SettingsPage />);
  await screen.findByLabelText('Preview recipe');
  fireEvent.click(within(nav()).getByRole('button', { name: 'Shopify' }));
  expect(screen.getByLabelText('Shop domain').closest('[data-section]')).toHaveAttribute('data-active', 'true');
  expect(screen.getByLabelText('Preview recipe').closest('[data-section]')).toHaveAttribute('data-active', 'false');
});

it('t1_c5c_saves_values_typed_across_two_sections_together', async () => {
  render(<SettingsPage />);
  fireEvent.change(await screen.findByLabelText('Preview recipe'), { target: { value: 'New preview' } });
  fireEvent.click(within(nav()).getByRole('button', { name: 'Shopify' }));
  fireEvent.change(screen.getByLabelText('Shop domain'), { target: { value: 'new.myshopify.com' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(saveSettings).mock.calls[0][0]).toMatchObject({
    preview_recipe: 'New preview', integration: { shop_domain: 'new.myshopify.com' },
  });
});

it('t1_c5d_hides_the_shared_save_footer_for_templates', async () => {
  render(<SettingsPage />);
  await screen.findByLabelText('Preview recipe');
  fireEvent.click(within(nav()).getByRole('button', { name: 'Shooting templates' }));
  expect(screen.getByTestId('settings-footer')).toHaveAttribute('data-active', 'false');
  fireEvent.click(within(nav()).getByRole('button', { name: 'Camera' }));
  expect(screen.getByTestId('settings-footer')).toHaveAttribute('data-active', 'true');
});
