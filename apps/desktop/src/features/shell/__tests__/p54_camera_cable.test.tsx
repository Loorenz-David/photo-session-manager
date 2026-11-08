import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { CameraDot } from '../CameraDot';
import { cameraStatus, cameraRetry, cameraTakeBackAvailability, cameraTakeBack } from '@/services/camera';

vi.mock('@/services/camera', () => ({
  cameraStatus: vi.fn(), cameraRetry: vi.fn(), cameraTakeBackAvailability: vi.fn(),
  cameraTakeBack: vi.fn(), cameraReconnect: vi.fn(), cameraFilesNotReceivedDismiss: vi.fn(),
}));
vi.mock('@/services/events', () => ({ onCameraStatus: vi.fn(async () => () => undefined) }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('C4(a) shows the Wi-Fi link while connecting', async () => {
  vi.mocked(cameraStatus).mockResolvedValue({ kind: 'connecting', link: 'wireless' });
  render(<CameraDot />);
  expect(await screen.findByRole('button', { name: 'Connecting · Wi-Fi' })).toHaveAttribute('data-state', 'connecting');
});

it('C5(a) disables take-back with the first precondition reason, then enables it', async () => {
  vi.mocked(cameraStatus).mockResolvedValue({kind:'needs_action',reason:'held_by_other_program',holder:'Capture One',message:'Capture One holds camera',link:'wireless'});
  vi.mocked(cameraTakeBackAvailability).mockResolvedValueOnce('an export is running').mockResolvedValue(null);
  render(<CameraDot />);
  fireEvent.click(await screen.findByRole('button', {name:'Capture One holds camera'}));
  const button = await screen.findByRole('button', {name:'Take the camera back'});
  expect(button).toBeDisabled();
  expect(await screen.findByText('an export is running')).toBeInTheDocument();
  fireEvent.click(button);
  expect(cameraTakeBack).not.toHaveBeenCalled();
  await waitFor(() => expect(button).toBeEnabled(), {timeout: 2500});
});

it('C4(b) Retry updates the dot to Connecting immediately', async () => {
  vi.mocked(cameraStatus).mockResolvedValue({kind:'disconnected'});
  vi.mocked(cameraRetry).mockResolvedValue({kind:'connecting',link:'wireless'});
  render(<CameraDot />);
  fireEvent.click(await screen.findByRole('button', {name:'Camera disconnected'}));
  fireEvent.click(screen.getByRole('button', {name:'Retry'}));
  expect(await screen.findByRole('button', {name:'Connecting · Wi-Fi'})).toHaveAttribute('data-state', 'connecting');
});
