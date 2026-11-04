import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { SettingsPage } from '../SettingsPage';
import { HealthPage } from '@/features/health/HealthPage';
import { getSettings, pickSessionFile, saveSettings, type Settings } from '@/services/settings';
import { getAppPaths, getHealth } from '@/services/health';

vi.mock('@/services/settings', () => ({ getSettings:vi.fn(), saveSettings:vi.fn(), pickSessionFile:vi.fn(), pickFolder:vi.fn() }));
vi.mock('@/services/health', () => ({ getHealth:vi.fn(), getAppPaths:vi.fn() }));
vi.mock('sonner', () => ({ toast:{ success:vi.fn(), error:vi.fn() } }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

const base: Settings = {
  document_id:null, effective_document_id:'/Users/me/Pictures/Session Manager', default_document_id: '/Users/me/Pictures/Session Manager',
  preview_recipe:'PSM Preview', web_recipe:'PSM Web', style_name:'PSM House', default_shape:'1:1', master_recipe:'PSM Master',
  export_folder:'/tmp/export', preview_delay_s:15, effective_preview_delay_s:15,
  theme:'light', sku_prefix:null, integration:null,
};

it('C5(a) Settings picks a session file, saves its folder and restores the app default', async () => {
  vi.mocked(getSettings).mockResolvedValue(base);
  vi.mocked(pickSessionFile).mockResolvedValue('/Users/me/Pictures/Other/Other.cosessiondb');
  vi.mocked(saveSettings).mockImplementation(async patch => ({ ...base, document_id: patch.document_id || null }));
  render(<SettingsPage />);
  expect(await screen.findByText('Session Manager (created by the app)')).toBeInTheDocument();
  expect(screen.getByText('Effective path: /Users/me/Pictures/Session Manager')).toBeInTheDocument();
  expect(screen.queryByRole('button',{name:'Use the app\'s own session'})).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Use another session…'}));
  expect(await screen.findByText('/Users/me/Pictures/Other')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Save settings'}));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({document_id:'/Users/me/Pictures/Other'})));
  cleanup(); vi.clearAllMocks();
  vi.mocked(getSettings).mockResolvedValue({...base, document_id:'/Users/me/Pictures/Other', effective_document_id:'/Users/me/Pictures/Other'});
  vi.mocked(saveSettings).mockImplementation(async()=>base);
  render(<SettingsPage />);
  expect(await screen.findByText('/Users/me/Pictures/Other')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Use the app\'s own session'}));
  expect(await screen.findByText('Session Manager (created by the app)')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Save settings'}));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({document_id:''})));
  cleanup(); vi.clearAllMocks();
  vi.mocked(getSettings).mockResolvedValue(base); vi.mocked(pickSessionFile).mockResolvedValue(null);
  render(<SettingsPage />);
  expect(await screen.findByText('Session Manager (created by the app)')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Use another session…'}));
  await waitFor(() => expect(pickSessionFile).toHaveBeenCalledTimes(1));
  expect(screen.getByText('Session Manager (created by the app)')).toBeInTheDocument();
  expect(screen.queryByRole('alert')).not.toBeInTheDocument();
});

it('C5(b) Health shows each provisioning failure in plain words', async () => {
  const items = [
    ['editor_not_installed','Capture One is not installed on this Mac','Capture One'],
    ['editor_not_pro','Capture One Pro is required; the installed edition cannot be scripted','Capture One'],
    ['editor_automation_denied','macOS has not allowed Session Manager to control Capture One','Capture One'],
    ['editor_not_running','Capture One did not start within 45 s','Capture One'],
    ['document_not_created','Capture One could not create the session','Capture One document'],
    ['document_not_front','Capture One is showing another session; captures would land there','Capture One document'],
    ['style_not_found',"Style 'PSM House' not found in Capture One",'Style'],
  ] as const;
  vi.mocked(getHealth).mockResolvedValue({items:items.map(([code,detail])=>({code,detail,ok:false}))});
  vi.mocked(getAppPaths).mockResolvedValue({data_dir:'/tmp/psm',database:'/tmp/psm/db',settings:'/tmp/psm/settings'});
  render(<HealthPage />);
  for(const [code,detail,label] of items){
    const element=await screen.findByText(detail,{exact:false});
    const row=element.closest('.health-row');
    expect(row).toHaveAttribute('data-ok','false');
    expect(row?.querySelector('.health-label')).toHaveTextContent(label);
    expect(row?.textContent).not.toContain(code);
  }
});
