import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { SettingsPage } from '../SettingsPage';
import { getSettings, saveSettings, type Settings } from '@/services/settings';

vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), pickSessionFile: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

const shown: Settings = {
  camera: { receiver_port: 2121, passive_port_start: 50000, passive_port_end: 50009 }, document_id: null, effective_document_id: '/tmp/doc', default_document_id: '/tmp/doc',
  preview_recipe: 'PSM Preview', master_recipe: 'PSM Master', output_format: { long_edge: 2048, quality: 90 }, style_name: '', default_shape: '1:1',
  export_folder: '/tmp/export', preview_delay_s: 15, effective_preview_delay_s: 15, theme: 'light', sku_prefix: null, integration: null,
};

it('C4(f) Settings trades the web recipe for the output size and quality', async () => {
  vi.mocked(getSettings).mockResolvedValue(shown);
  vi.mocked(saveSettings).mockResolvedValue(shown);
  render(<SettingsPage />);
  const size = await screen.findByRole('spinbutton', { name: 'Output size (long edge, px)' });
  const quality = screen.getByRole('spinbutton', { name: 'Output JPEG quality' });
  expect(size).toHaveValue(2048);
  expect(quality).toHaveValue(90);
  expect(screen.queryByRole('textbox', { name: 'Web recipe' })).not.toBeInTheDocument();
  fireEvent.change(size, { target: { value: '1600' } });
  fireEvent.change(quality, { target: { value: '85' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(saveSettings).mock.calls[0][0].output_format).toEqual({ long_edge: 1600, quality: 85 });
});
