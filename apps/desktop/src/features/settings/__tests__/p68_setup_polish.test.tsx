import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { CameraSection } from '../CameraSection';
import { SettingsPage } from '../SettingsPage';
import { CameraDot } from '@/features/shell/CameraDot';
import { cameraFingerprintConfirm, cameraFingerprintPending, cameraFtpAddress, cameraFtpPasswordGenerate, cameraFtpPasswordGet, cameraSecretsSet, cameraStatus, cameraSupportGet } from '@/services/camera';
import { getSettings, saveSettings, type Settings } from '@/services/settings';

vi.mock('@/services/camera', () => ({
  cameraSupportGet: vi.fn(), cameraStatus: vi.fn(), cameraFtpAddress: vi.fn(), cameraFtpPasswordGenerate: vi.fn(), cameraFtpPasswordGet: vi.fn(),
  cameraSecretsSet: vi.fn(), cameraFingerprintPending: vi.fn(), cameraFingerprintConfirm: vi.fn(), cameraFingerprintReject: vi.fn(),
  cameraTestConnection: vi.fn(), cameraTakeBackAvailability: vi.fn(),
}));
vi.mock('@/services/events', () => ({ onFingerprintPending: vi.fn(async () => () => undefined), onCameraStatus: vi.fn(async () => () => undefined) }));
vi.mock('@/services/settings', () => ({ getSettings: vi.fn(), saveSettings: vi.fn(), pickFolder: vi.fn(), pickSessionFile: vi.fn(), pickArchive: vi.fn() }));
vi.mock('@/services/styles', () => ({ listStyles: vi.fn(async () => ({ user: [], built_in: [] })) }));
vi.mock('../StorageSection', () => ({ StorageSection: () => null }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const ports = { receiver_port: 2121, passive_port_start: 50000, passive_port_end: 50009 };
const shown: Settings = {
  camera: ports, document_id: null, effective_document_id: '/tmp/session', default_document_id: '/tmp/session', preview_recipe: 'PSM Preview',
  output_format: { long_edge: 2048, quality: 90 }, style_name: '', default_shape: '1:1', master_recipe: 'PSM Master', export_folder: '/tmp/out',
  preview_delay_s: 15, effective_preview_delay_s: 15, theme: 'light', sku_prefix: null, integration: null,
};
const SENTENCE = "This Mac's address changed to 192.168.1.20 — set Host Name on the camera";

beforeEach(() => {
  window.location.hash = '#/settings';
  vi.mocked(cameraSupportGet).mockResolvedValue({ make: 'sony', version: '2.02.00', sidecar_path: '/x/helper' });
  vi.mocked(cameraStatus).mockResolvedValue({ kind: 'disconnected' });
  vi.mocked(cameraFtpAddress).mockResolvedValue({ address: '192.168.1.20', warning: SENTENCE });
  vi.mocked(cameraFingerprintPending).mockResolvedValue(null);
  vi.mocked(getSettings).mockResolvedValue(shown);
  vi.mocked(saveSettings).mockResolvedValue(shown);
  vi.mocked(cameraSecretsSet).mockResolvedValue();
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('C1(a) step 3 shows the current address B with Copy; the header says the address changed to B', async () => {
  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
  render(<CameraSection ports={ports} onPortsChange={vi.fn()} />);
  const step3 = await screen.findByText(/\[Host Name\] = 192\.168\.1\.20; \[Port\] = 2121/);
  expect(step3.tagName).toBe('LI');
  expect(screen.getAllByRole('listitem')[2]).toBe(step3);
  fireEvent.click(within(step3).getByRole('button', { name: 'Copy address' }));
  expect(writeText).toHaveBeenCalledWith('192.168.1.20');
  cleanup();
  render(<CameraDot />);
  const dot = await screen.findByRole('button', { name: `${SENTENCE.slice(0, 48)}…` });
  expect(dot).toHaveAttribute('title', SENTENCE);
  fireEvent.click(dot);
  expect(await screen.findByRole('status')).toHaveTextContent(SENTENCE);
});

it('C2(a) Generate shows the password in groups of 4; Show password shows the same password again', async () => {
  vi.mocked(cameraFtpPasswordGenerate).mockResolvedValue({ password: 'k7mx4rtp9qad2wen' });
  vi.mocked(cameraFtpPasswordGet).mockResolvedValue('k7mx4rtp9qad2wen');
  const view = render(<CameraSection ports={ports} onPortsChange={vi.fn()} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Generate password' }));
  expect(await screen.findByTestId('ftp-password')).toHaveTextContent(/^k7mx 4rtp 9qad 2wen$/);
  view.unmount();
  render(<CameraSection ports={ports} onPortsChange={vi.fn()} />);
  await screen.findByRole('button', { name: 'Show password' });
  expect(screen.queryByTestId('ftp-password')).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Show password' }));
  expect(await screen.findByTestId('ftp-password')).toHaveTextContent(/^k7mx 4rtp 9qad 2wen$/);
  expect(cameraFtpPasswordGet).toHaveBeenCalledTimes(1);
});

it('C4(a) the camera login is saved by Save settings (Keychain, not settings.json); leaving with a typed login asks first', async () => {
  render(<SettingsPage />);
  fireEvent.change(await screen.findByLabelText('Access Authentication user'), { target: { value: 'cam-user' } });
  fireEvent.change(screen.getByLabelText('Access Authentication password'), { target: { value: 'cam-pass' } });
  expect(screen.queryByRole('button', { name: 'Save camera login' })).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Health' }));
  const dialog = await screen.findByRole('dialog');
  expect(within(dialog).getByText('Save your changes?')).toBeInTheDocument();
  expect(window.location.hash).toBe('#/settings');
  fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
  await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
  fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
  await waitFor(() => expect(cameraSecretsSet).toHaveBeenCalledWith({ access_auth_user: 'cam-user', access_auth_password: 'cam-pass' }));
  await waitFor(() => expect(saveSettings).toHaveBeenCalledTimes(1));
  expect(JSON.stringify(vi.mocked(saveSettings).mock.calls[0][0])).not.toMatch(/cam-user|cam-pass/);
  await waitFor(() => expect(screen.getByLabelText('Access Authentication user')).toHaveValue(''));
  expect(screen.getByLabelText('Access Authentication password')).toHaveValue('');
});

it("C4(b) with a fingerprint pending, the header's Confirm the camera's fingerprint opens the box with Confirm", async () => {
  vi.mocked(cameraFtpAddress).mockResolvedValue({ address: '192.168.1.20', warning: null });
  vi.mocked(cameraStatus).mockResolvedValue({ kind: 'needs_action', reason: 'fingerprint_unconfirmed', message: "Confirm the camera's fingerprint", holder: null, link: 'wireless' });
  vi.mocked(cameraFingerprintPending).mockResolvedValue({ fingerprint: 'ab:cd:ef', replace: false });
  vi.mocked(cameraFingerprintConfirm).mockResolvedValue();
  render(<CameraDot />);
  const dot = await screen.findByRole('button', { name: "Confirm the camera's fingerprint" });
  fireEvent.click(dot);
  await waitFor(() => expect(screen.getAllByRole('button', { name: "Confirm the camera's fingerprint" })).toHaveLength(2));
  const action = screen.getAllByRole('button', { name: "Confirm the camera's fingerprint" }).find(button => button !== dot)!;
  expect(screen.queryByText('ab:cd:ef')).not.toBeInTheDocument();
  fireEvent.click(action);
  const dialog = await screen.findByRole('dialog', { name: 'Camera fingerprint' });
  expect((await within(dialog).findByText('ab:cd:ef')).tagName).toBe('CODE');
  fireEvent.click(within(dialog).getByRole('button', { name: 'Confirm' }));
  await waitFor(() => expect(cameraFingerprintConfirm).toHaveBeenCalledWith('ab:cd:ef'));
  await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Camera fingerprint' })).not.toBeInTheDocument());
});
