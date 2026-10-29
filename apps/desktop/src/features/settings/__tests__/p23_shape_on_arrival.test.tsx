import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { SettingsPage } from '../SettingsPage';
import { HealthPage } from '@/features/health/HealthPage';
import { getSettings, listCropRatios, saveSettings, type Settings } from '@/services/settings';
import { getAppPaths, getHealth } from '@/services/health';

vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), listCropRatios: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn() }));
vi.mock('@/services/health', () => ({ getHealth: vi.fn(), getAppPaths: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn() } }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
const shown: Settings = {
  document_id: '/tmp/session', preview_recipe: 'PSM Preview', web_recipe: 'PSM Web', style_name: 'PSM House',
  crop_on_arrival: 'Square', export_folder: '/tmp/export', preview_delay_s: 15, effective_preview_delay_s: 15,
  theme: 'light', sku_prefix: null, integration: null,
};
it('C4(b) shows provider ratios and saves the selected shape, including off and an unknown stored value', async () => {
  vi.mocked(getSettings).mockResolvedValue(shown);
  vi.mocked(listCropRatios).mockResolvedValue(['Unconstrained','Original','Output','Square','4x5']);
  vi.mocked(saveSettings).mockImplementation(async patch => ({...shown, crop_on_arrival: patch.crop_on_arrival ?? shown.crop_on_arrival}));
  const first = render(<SettingsPage />);
  const select = await screen.findByRole('combobox', {name:'Shape on arrival'});
  await waitFor(() => expect(select.querySelectorAll('option')).toHaveLength(6));
  expect(Array.from(select.querySelectorAll('option')).map(o=>o.textContent)).toEqual(['Leave as shot','Unconstrained','Original','Output','Square','4x5']);
  expect(select).toHaveValue('Square');
  fireEvent.change(select,{target:{value:'off'}}); fireEvent.click(screen.getByRole('button',{name:'Save settings'}));
  await waitFor(() => expect(saveSettings).toHaveBeenCalled());
  expect(vi.mocked(saveSettings).mock.lastCall?.[0].crop_on_arrival).toBe('off');
  fireEvent.change(select,{target:{value:'4x5'}}); fireEvent.click(screen.getByRole('button',{name:'Save settings'}));
  await waitFor(() => expect(vi.mocked(saveSettings).mock.lastCall?.[0].crop_on_arrival).toBe('4x5'));
  first.unmount();
  vi.mocked(getSettings).mockResolvedValue({...shown,crop_on_arrival:'Squircle'});
  vi.mocked(listCropRatios).mockRejectedValue(new Error('provider'));
  render(<SettingsPage />);
  const unknown = await screen.findByRole('combobox',{name:'Shape on arrival'});
  await waitFor(() => expect(Array.from(unknown.querySelectorAll('option')).map(o=>o.textContent)).toEqual(['Leave as shot','Squircle']));
  expect(unknown).toHaveValue('Squircle'); expect(screen.queryByRole('alert')).not.toBeInTheDocument();
});
it('C5(b) labels both crop failures in Health with the exact details', async () => {
  vi.mocked(getHealth).mockResolvedValue({items:[
    {code:'crop_not_applied',ok:false,detail:'Shape not applied to 2 photos'},
    {code:'crop_ratio_unknown',ok:false,detail:"Shape on arrival: 'Squircle' is not a Capture One crop ratio"},
  ]});
  vi.mocked(getAppPaths).mockResolvedValue({data_dir:'/tmp/psm',database:'/tmp/psm/db',settings:'/tmp/psm/settings.json'});
  render(<HealthPage />);
  for (const detail of ['Shape not applied to 2 photos',"Shape on arrival: 'Squircle' is not a Capture One crop ratio"]) {
    const row=(await screen.findByText(detail)).closest('.health-row');
    expect(row).toHaveAttribute('data-ok','false');
    expect(row?.querySelector('.health-label')).toHaveTextContent('Shape on arrival');
  }
});
