import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import { CameraDot } from '../CameraDot';
import { cameraStatus, cameraTakeBackAvailability } from '@/services/camera';

vi.mock('@/services/camera', () => ({ cameraStatus: vi.fn(), cameraTakeBackAvailability: vi.fn() }));
vi.mock('@/services/events', () => ({ onCameraStatus: vi.fn(async () => () => undefined), onFingerprintPending: vi.fn(async () => () => undefined) }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

// C2(a), C3(a): the header says why the cable cannot be taken, in the backend's words (the dot
// shows the start of the sentence, its card the whole sentence).
for (const [row, holder, message] of [
  ['C2(a)', 'Capture One', 'Capture One has a window open — close it so Session Manager can take the camera'],
  ['C3(a)', 'ptpcamerad', "macOS's camera service is holding the cable with no app using it. Restart the Mac to free it."],
] as const) {
  it(`${row} the header names the reason`, async () => {
    vi.mocked(cameraStatus).mockResolvedValue({ kind: 'needs_action', reason: 'held_by_other_program', holder, message, link: 'wireless' });
    vi.mocked(cameraTakeBackAvailability).mockResolvedValue(null);
    render(<CameraDot />);
    const dot = await screen.findByRole('button', { name: `${message.slice(0, 48)}…` });
    fireEvent.click(dot);
    expect(await screen.findByText(message)).toBeInTheDocument();
  });
}
