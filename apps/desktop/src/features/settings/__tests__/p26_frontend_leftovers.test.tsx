import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { SettingsPage } from '../SettingsPage';
import { getSettings, pickSessionFile, saveSettings, type Settings } from '@/services/settings';

vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickSessionFile: vi.fn(), pickFolder: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

const base: Settings = {
  camera: { receiver_port: 2121, passive_port_start: 50000, passive_port_end: 50009 }, document_id: '/Users/me/Pictures/Other', effective_document_id: '/Users/me/Pictures/Other',
  default_document_id: '/Users/me/Pictures/Session Manager', preview_recipe: 'PSM Preview',
  output_format: { long_edge: 2048, quality: 90 }, style_name: '', default_shape: '1:1', master_recipe: 'PSM Master', export_folder: '/tmp/export',
  preview_delay_s: 15, effective_preview_delay_s: 15, theme: 'light', sku_prefix: null, integration: null,
};

it('C1(a) shows the full app session path before save and the newly picked folder', async () => {
  vi.mocked(getSettings).mockResolvedValue(base);
  vi.mocked(pickSessionFile).mockResolvedValue('/Users/me/Pictures/Two/Two.cosessiondb');
  render(<SettingsPage />);
  expect(await screen.findByText('/Users/me/Pictures/Other')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: "Use the app's own session" }));
  expect(screen.getByText('Session Manager (created by the app)')).toBeInTheDocument();
  expect(screen.getByText('Effective path: /Users/me/Pictures/Session Manager')).toBeInTheDocument();
  expect(document.body.textContent).not.toContain('~/');
  expect(saveSettings).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Use another session…' }));
  await waitFor(() => expect(screen.getByText('Effective path: /Users/me/Pictures/Two')).toBeInTheDocument());
  expect(saveSettings).not.toHaveBeenCalled();
});
