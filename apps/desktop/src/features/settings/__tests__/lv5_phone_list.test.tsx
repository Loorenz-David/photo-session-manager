import '@testing-library/jest-dom/vitest';
import { act, cleanup, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { PhoneSection } from '../PhoneSection';
import { phonePairingsList, phoneServerStatus } from '@/services/phone';

vi.mock('@/services/phone', () => ({
  phonePairingsList: vi.fn(), phoneServerStatus: vi.fn(), phonePairingCreate: vi.fn(),
  phonePairingRename: vi.fn(), phonePairingRevoke: vi.fn(),
}));
vi.mock('@/services/settings', () => ({ saveSettings: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const first = [{ id: 1, name: 'Android', connected: false, last_seen_at: null, created_at: 1 }];
const connected = [{ ...first[0], connected: true, last_seen_at: 1_700_000_000_000 }];
beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(phoneServerStatus).mockResolvedValue({ enabled: true, running: true, port: 8686, error: null });
  vi.mocked(phonePairingsList).mockResolvedValueOnce(first).mockResolvedValue(connected);
});
afterEach(() => { cleanup(); vi.clearAllMocks(); vi.useRealTimers(); });

it('LV5·C10(a) re-reads the connected phone after five seconds', async () => {
  render(<PhoneSection />);
  await act(async () => { await Promise.resolve(); });
  expect(screen.getByText('Never connected')).toBeInTheDocument();
  await act(async () => { await vi.advanceTimersByTimeAsync(5_000); });
  expect(phonePairingsList).toHaveBeenCalledTimes(2);
  expect(screen.getByLabelText('Connected')).toBeInTheDocument();
  expect(screen.getByText('Connected now')).toBeInTheDocument();
});

it('LV5·C10(b) stops re-reading when Settings closes', async () => {
  const { unmount } = render(<PhoneSection />);
  await act(async () => { await Promise.resolve(); });
  expect(phonePairingsList).toHaveBeenCalledTimes(1);
  unmount();
  await act(async () => { await vi.advanceTimersByTimeAsync(10_000); });
  expect(phonePairingsList).toHaveBeenCalledTimes(1);
});
