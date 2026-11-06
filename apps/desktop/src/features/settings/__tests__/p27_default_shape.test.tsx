import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { SettingsPage } from '../SettingsPage';
import { HealthPage } from '@/features/health/HealthPage';
import { getSettings, saveSettings, type Settings } from '@/services/settings';
import { getAppPaths, getHealth } from '@/services/health';

vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), pickSessionFile: vi.fn() }));
vi.mock('@/services/health', () => ({ getHealth: vi.fn(), getAppPaths: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

const shown: Settings = {
  camera: { receiver_port: 2121, passive_port_start: 50000, passive_port_end: 50009 }, document_id: null, effective_document_id: '/tmp/doc', default_document_id: '/tmp/doc',
  preview_recipe: 'PSM Preview', master_recipe: 'PSM Master', output_format: { long_edge: 2048, quality: 90 }, style_name: '', default_shape: '1:1',
  export_folder: '/tmp/export', preview_delay_s: 15, effective_preview_delay_s: 15, theme: 'light', sku_prefix: null, integration: null,
};

it('C4(b) retired crop health code displays its raw name', async () => {
  vi.mocked(getHealth).mockResolvedValue({ items: [{ code: 'crop_not_applied', ok: false, detail: 'x' }] });
  vi.mocked(getAppPaths).mockResolvedValue({ data_dir: '/tmp/psm', database: '/tmp/psm/db', settings: '/tmp/psm/settings' });
  render(<HealthPage />);
  const detail = await screen.findByText('x');
  expect(detail.closest('.health-row')?.querySelector('.health-label')).toHaveTextContent('crop_not_applied');
});

it('C5(a) Settings shows the seven default shapes and saves the master recipe', async () => {
  vi.mocked(getSettings).mockResolvedValue(shown);
  vi.mocked(saveSettings).mockResolvedValue(shown);
  render(<SettingsPage />);
  const shape = await screen.findByRole('combobox', { name: 'Default shape' });
  expect(shape).toHaveValue('1:1');
  expect(Array.from(shape.querySelectorAll('option')).map(option => [option.value, option.textContent])).toEqual([
    ['original', 'Original'], ['1:1', 'Square (1:1)'], ['4:5', '4:5'], ['3:2', '3:2'],
    ['2:3', '2:3'], ['16:9', '16:9'], ['free', 'Free'],
  ]);
  expect(screen.queryByLabelText('Shape on arrival')).not.toBeInTheDocument();
  expect(screen.getByRole('textbox', { name: 'Master recipe' })).toHaveValue('PSM Master');
  fireEvent.change(shape, { target: { value: '4:5' } });
  fireEvent.change(screen.getByRole('textbox', { name: 'Master recipe' }), { target: { value: 'My Master' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
  const patch = vi.mocked(saveSettings).mock.calls[0][0];
  expect(patch.default_shape).toBe('4:5');
  expect(patch.master_recipe).toBe('My Master');
  expect(patch).not.toHaveProperty('crop_on_arrival');
});
