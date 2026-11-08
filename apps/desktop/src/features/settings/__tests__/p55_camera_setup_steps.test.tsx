import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { CameraSection } from '../CameraSection';

vi.mock('@/services/camera', () => ({
  cameraSupportGet: vi.fn(async () => ({make:'sony',version:'2.02.00',sidecar_path:'/tmp/helper'})),
  cameraStatus: vi.fn(async () => ({kind:'disconnected'})),
  cameraFtpPasswordGenerate: vi.fn(async () => ({password:'test-password'})),
}));
vi.mock('@/services/events', () => ({onFingerprintPending: vi.fn(async () => () => undefined)}));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('C2(a) camera setup includes the Auto FTP Trnsfr menu path and On value', async () => {
  render(<CameraSection ports={{receiver_port:2121,passive_port_start:50000,passive_port_end:50009}} onPortsChange={vi.fn()} />);
  fireEvent.click(await screen.findByRole('button', {name:'Generate password'}));
  expect(await screen.findByText(/Auto FTP Trnsfr/)).toHaveTextContent('MENU › Network › [FTP Transfer] › [FTP Transfer Func.] › [Auto FTP Trnsfr] = [On]');
});
