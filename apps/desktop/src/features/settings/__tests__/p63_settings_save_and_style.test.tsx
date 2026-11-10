import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { SettingsPage } from '../SettingsPage';
import { getSettings, saveSettings, type Settings } from '@/services/settings';
import { listStyles } from '@/services/styles';

vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), pickSessionFile: vi.fn() }));
vi.mock('@/services/styles', () => ({ listStyles: vi.fn() }));
vi.mock('./../CameraSection', () => ({ CameraSection: () => null }));
vi.mock('./../StorageSection', () => ({ StorageSection: () => null }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const shown: Settings = {
  default_shape: '1:1', master_recipe: 'PSM Master',
  camera: { receiver_port: 2121, passive_port_start: 50000, passive_port_end: 50009 }, document_id: '/Users/me/Pictures/Beyo Studio', effective_document_id: '/Users/me/Pictures/Beyo Studio', default_document_id: '/Users/me/Pictures/Session Manager', preview_recipe: 'PSM Preview', output_format: { long_edge: 2048, quality: 90 }, style_name: '',
  export_folder: '/Users/me/Pictures/Session Manager Export', preview_delay_s: 15, effective_preview_delay_s: 15, theme: 'light', sku_prefix: null, integration: null,
};

beforeEach(() => {
  window.location.hash = '#/settings';
  vi.mocked(getSettings).mockResolvedValue(shown);
  vi.mocked(saveSettings).mockImplementation(async patch => ({ ...shown, ...patch, document_id: patch.document_id || null } as Settings));
  vi.mocked(listStyles).mockResolvedValue({ user: ['test-style'], built_in: ['Vivid'] });
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

async function changed() {
  render(<SettingsPage />);
  fireEvent.change(await screen.findByLabelText('Preview recipe'), { target: { value: 'PSM Preview 2' } });
}

it('C4(a) the Save button sits in the sticky footer, in view at any scroll position', async () => {
  await changed();
  const button = screen.getByRole('button', { name: 'Save settings' });
  const footer = screen.getByTestId('settings-footer');
  expect(footer).toContainElement(button);
  expect(getComputedStyle(footer).position).toBe('sticky');
  expect(getComputedStyle(footer).bottom).toBe('0px');
});

it('C4(a) leaving with an unsaved change asks "Save your changes?"; Cancel stays, Discard leaves without saving', async () => {
  await changed();
  fireEvent.click(screen.getByRole('button', { name: 'Back to Sessions' }));
  const dialog = await screen.findByRole('dialog');
  expect(within(dialog).getByText('Save your changes?')).toBeInTheDocument();
  expect(window.location.hash).toBe('#/settings');
  fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
  expect(window.location.hash).toBe('#/settings');
  expect(screen.getByLabelText('Preview recipe')).toHaveValue('PSM Preview 2');
  fireEvent.click(screen.getByRole('button', { name: 'Health' }));
  fireEvent.click(within(await screen.findByRole('dialog')).getByRole('button', { name: 'Discard' }));
  await waitFor(() => expect(window.location.hash).toBe('#/health'));
  expect(saveSettings).not.toHaveBeenCalled();
});

it('C4(a) Save in the prompt saves the change, then leaves', async () => {
  await changed();
  fireEvent.keyDown(window, { key: 'Escape' });
  fireEvent.click(within(await screen.findByRole('dialog')).getByRole('button', { name: 'Save' }));
  await waitFor(() => expect(window.location.hash).toBe('#/'));
  expect(saveSettings).toHaveBeenCalledTimes(1);
  expect(vi.mocked(saveSettings).mock.calls[0][0].preview_recipe).toBe('PSM Preview 2');
});

it('C4(a) leaving without a change does not ask', async () => {
  render(<SettingsPage />);
  await screen.findByLabelText('Preview recipe');
  fireEvent.click(screen.getByRole('button', { name: 'Back to Sessions' }));
  await waitFor(() => expect(window.location.hash).toBe('#/'));
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
});

it('C6(b) Style is a drop-down of Capture One\'s styles, the photographer\'s own first; the choice is saved by its exact name', async () => {
  render(<SettingsPage />);
  const style = await screen.findByRole('combobox', { name: 'Style' });
  await waitFor(() => expect(within(style).getAllByRole('option').map(o => o.textContent)).toEqual(['No style', 'test-style', 'Vivid']));
  expect(style).toHaveValue('');
  fireEvent.change(style, { target: { value: 'test-style' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(saveSettings).mock.calls[0][0].style_name).toBe('test-style');
});

it('C6(b) when the styles cannot be read, Style stays an editable text field with the sentence', async () => {
  vi.mocked(listStyles).mockResolvedValue(null);
  vi.mocked(getSettings).mockResolvedValue({ ...shown, style_name: 'Test-style' });
  render(<SettingsPage />);
  expect(await screen.findByText("Couldn't read Capture One's styles")).toBeInTheDocument();
  const field = screen.getByRole('textbox', { name: 'Style' });
  expect(field).toHaveValue('Test-style');
  fireEvent.change(field, { target: { value: 'test-style' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(saveSettings).mock.calls[0][0].style_name).toBe('test-style');
});
