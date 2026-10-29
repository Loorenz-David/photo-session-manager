import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { SettingsPage } from '@/features/settings/SettingsPage';
import { NotificationsDrawer } from '@/features/publish/NotificationsDrawer';
import { PageHeader } from '@/features/shell/PageHeader';
import { getSettings, saveSettings, type Settings } from '@/services/settings';

vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), listCropRatios: vi.fn(async () => []) }));
vi.mock('@/services/publish', () => ({ publishRetry: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

afterEach(() => { cleanup(); vi.clearAllMocks(); window.location.hash = '#/'; });

const shown: Settings = {
  crop_on_arrival: 'Square',
  document_id: '/session', preview_recipe: 'PSM Preview', web_recipe: 'PSM Web', style_name: 'PSM House',
  export_folder: '/export', preview_delay_s: 15, effective_preview_delay_s: 15, theme: 'light', sku_prefix: null,
  integration: { kind: 'shopify', shop_domain: 'shop.myshopify.com', api_version: '2026-10', client_id: 'client-id', client_secret: '••••1234', condition_photos_key: 'custom.damage_report' },
};

it('C1(a) shows a local date and time for new and legacy publish rows', () => {
  const stamp = 1761350400000;
  render(<NotificationsDrawer onClose={() => undefined} onOpen={() => undefined} notifications={[
    { id: 1, item_id: 1, session_id: 5, run_id: 1, kind: 'published', text: 'Published', created_at: stamp },
    { id: 2, item_id: 2, session_id: 5, run_id: 2, kind: 'published', text: `Published · ${stamp}`, created_at: stamp },
  ]} />);
  const expected = `Published · ${new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(stamp)}`;
  const rows = [...document.querySelectorAll('aside p')].filter(row => row.textContent === expected);
  expect(rows).toHaveLength(2);
  for (const row of rows) {
    expect(row.tagName).toBe('P');
    expect(row.querySelector('time')).toHaveAttribute('datetime', '2025-10-25T00:00:00.000Z');
    expect(row.textContent).not.toMatch(/\d{13}/);
  }
});

it('C2(a) clears the loaded mask on focus, restores it on empty blur, and saves a typed replacement', async () => {
  vi.mocked(getSettings).mockResolvedValue(shown);
  vi.mocked(saveSettings).mockResolvedValue(shown);
  const first = render(<SettingsPage />);
  const secret = await screen.findByLabelText('Client secret');
  fireEvent.focus(secret);
  expect(secret).toHaveValue('');
  fireEvent.blur(secret);
  expect(secret).toHaveValue('••••1234');
  fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(saveSettings).mock.calls[0][0].integration?.client_secret).toBe('••••1234');
  first.unmount();
  vi.mocked(saveSettings).mockClear();
  render(<SettingsPage />);
  const replacement = await screen.findByLabelText('Client secret');
  fireEvent.focus(replacement);
  fireEvent.change(replacement, { target: { value: 'NEW-SECRET-77' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(saveSettings).mock.calls[0][0].integration?.client_secret).toBe('NEW-SECRET-77');
});

it('C2(b) command Back stays on Settings while the secret input has focus', async () => {
  window.location.hash = '#/settings';
  vi.mocked(getSettings).mockResolvedValue(shown);
  render(<SettingsPage />);
  const secret = await screen.findByLabelText('Client secret');
  secret.focus();
  fireEvent.keyDown(secret, { key: '[', metaKey: true });
  expect(window.location.hash).toBe('#/settings');
  fireEvent.keyDown(window, { key: '[', metaKey: true });
  expect(window.location.hash).toBe('#/');
});

it('C5(a) derives every Back control from the current route', () => {
  for (const [route, label, destination] of [
    ['#/settings', 'Back to Sessions', '#/'],
    ['#/health', 'Back to Settings', '#/settings'],
    ['#/session/5', 'Back to Sessions', '#/'],
    ['#/', null, null],
  ] as const) {
    window.location.hash = route;
    const page = render(<PageHeader title={<h1>X</h1>} />);
    const back = screen.queryByRole('button', { name: /^Back/ });
    if (label === null) expect(back).toBeNull();
    else {
      expect(back).toHaveAccessibleName(label);
      expect(page.container.querySelector('header button')).toBe(back);
      fireEvent.click(back!);
      expect(window.location.hash).toBe(destination);
    }
    page.unmount();
  }
});
