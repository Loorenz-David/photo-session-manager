import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { CameraCard } from '../CameraCard';
import { cameraCapture } from '@/services/camera';
import { toast } from 'sonner';

vi.mock('@/services/camera', () => ({ cameraCapture: vi.fn(), cameraRetry: vi.fn(), cameraTakeBackAvailability: vi.fn() }));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it('C3(a) Take photo captures once only while connected', async () => {
  vi.mocked(cameraCapture).mockResolvedValue('capture-1');
  const view = render(<CameraCard status={{kind:'disconnected'}} onStatus={vi.fn()} />);
  expect(screen.getByRole('button', {name:'Take photo'})).toBeDisabled();
  view.rerender(<CameraCard status={{kind:'connected',name:'Sony',link:'wireless'}} onStatus={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', {name:'Take photo'}));
  await waitFor(() => expect(cameraCapture).toHaveBeenCalledTimes(1));
});

it('C3(b) Take photo failure shows the error sentence', async () => {
  vi.mocked(cameraCapture).mockRejectedValue({message:'Camera refused the shutter'});
  render(<CameraCard status={{kind:'connected',name:'Sony',link:'cable'}} onStatus={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', {name:'Take photo'}));
  await waitFor(() => expect(toast.error).toHaveBeenCalledWith('Camera refused the shutter'));
});
