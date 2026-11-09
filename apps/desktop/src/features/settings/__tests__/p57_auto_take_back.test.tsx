import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { CameraSection } from '../CameraSection';
import { CameraDot } from '@/features/shell/CameraDot';
import { cameraStatus, cameraSupportGet, cameraTakeBackAvailability } from '@/services/camera';

vi.mock('@/services/camera', () => ({
  cameraStatus: vi.fn(), cameraSupportGet: vi.fn(), cameraTakeBackAvailability: vi.fn(),
}));
vi.mock('@/services/events', () => ({ onCameraStatus: vi.fn(async () => () => undefined), onFingerprintPending: vi.fn(async () => () => undefined) }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('C1(c) automatic take-back is on by default and can be turned off', () => {
  vi.mocked(cameraSupportGet).mockResolvedValue(null);
  vi.mocked(cameraStatus).mockResolvedValue({kind:'disconnected'});
  const changed = vi.fn();
  render(<CameraSection ports={{receiver_port:2121,passive_port_start:50000,passive_port_end:50009,auto_take_back:true}} onPortsChange={changed}/>);
  const toggle = screen.getByRole('checkbox',{name:'Take the cable back from Capture One automatically when the camera is plugged in'});
  expect(toggle).toBeChecked();
  fireEvent.click(toggle);
  expect(changed).toHaveBeenCalledWith({receiver_port:2121,passive_port_start:50000,passive_port_end:50009,auto_take_back:false});
});

it('C1(b) waiting reason appears in the header', async () => {
  vi.mocked(cameraStatus).mockResolvedValue({kind:'needs_action',reason:'held_by_other_program',holder:'Capture One',message:'Waiting to take the cable: an export is running',link:'wireless'});
  vi.mocked(cameraTakeBackAvailability).mockResolvedValue('an export is running');
  render(<CameraDot/>);
  expect(await screen.findByRole('button',{name:'Waiting to take the cable: an export is running'})).toBeInTheDocument();
});
