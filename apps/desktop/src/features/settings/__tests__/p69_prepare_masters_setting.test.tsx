import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { PrepareMastersSetting } from '../StorageSection';
import { getSettings, saveSettings, type Settings } from '@/services/settings';

vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

afterEach(() => { cleanup(); vi.clearAllMocks(); });

const settings = (prepare: boolean) => ({ export: { prepare_masters: prepare } }) as unknown as Settings;

it('C2(b) Settings → Storage turns preparing masters off (export.prepare_masters)', async () => {
  vi.mocked(getSettings).mockResolvedValue(settings(true));
  vi.mocked(saveSettings).mockResolvedValue(settings(false));
  render(<PrepareMastersSetting />);
  const box = screen.getByRole('checkbox', { name: 'Prepare masters in advance' });
  await waitFor(() => expect(box).toBeChecked());
  fireEvent.click(box);
  await waitFor(() => expect(saveSettings).toHaveBeenCalledWith({ export: { prepare_masters: false } }));
  await waitFor(() => expect(box).not.toBeChecked());
});
