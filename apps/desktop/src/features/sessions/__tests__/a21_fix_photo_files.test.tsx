import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  sessionFilesMisplaced: vi.fn(),
  fixSessionFiles: vi.fn(),
  sessionDeletePreview: vi.fn(),
  deleteSession: vi.fn(),
}));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { toast } from 'sonner';
import { SessionActionsMenu } from '@/features/sessions/DeleteSession';
import { fixSessionFiles, sessionFilesMisplaced } from '@/services/sessions';

// Radix menus open on a primary-button pointer press (jsdom has no PointerEvent).
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.releasePointerCapture ??= () => undefined;
Element.prototype.scrollIntoView ??= function scrollIntoView() {};
if (!('PointerEvent' in window)) {
  class PointerEventPolyfill extends MouseEvent {
    readonly pointerId: number;
    readonly pointerType: string;
    constructor(type: string, init: PointerEventInit = {}) {
      super(type, init);
      this.pointerId = init.pointerId ?? 1;
      this.pointerType = init.pointerType ?? 'mouse';
    }
  }
  Object.assign(window, { PointerEvent: PointerEventPolyfill });
}

afterEach(cleanup);
beforeEach(() => {
  vi.mocked(sessionFilesMisplaced).mockReset();
  vi.mocked(fixSessionFiles).mockReset();
  vi.mocked(toast.success).mockReset();
  vi.mocked(toast.error).mockReset();
});

function openMenu() {
  render(<SessionActionsMenu sessionId={7} onDeleted={vi.fn()} />);
  fireEvent.pointerDown(screen.getByRole('button', { name: 'Session actions' }), { button: 0 });
}

describe('A21: Fix photo files location', () => {
  it('is offered only when some photos have their files in another folder', async () => {
    vi.mocked(sessionFilesMisplaced).mockResolvedValue(0);
    openMenu();
    await screen.findByRole('menuitem', { name: 'Delete session…' });
    await waitFor(() => expect(sessionFilesMisplaced).toHaveBeenCalledWith(7));
    expect(screen.queryByRole('menuitem', { name: /Fix photo files location/ })).not.toBeInTheDocument();
  });

  it('moves the files and says how many moved', async () => {
    vi.mocked(sessionFilesMisplaced).mockResolvedValue(8);
    vi.mocked(fixSessionFiles).mockResolvedValue({ moved: 8, sidecars: 27, left: [] });
    openMenu();
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Fix photo files location (8)' }));
    await waitFor(() => expect(fixSessionFiles).toHaveBeenCalledWith(7));
    await waitFor(() => expect(toast.success).toHaveBeenCalledWith('8 photos moved into this session’s folder', expect.anything()));
  });

  it('names the photos it left and why', async () => {
    vi.mocked(sessionFilesMisplaced).mockResolvedValue(1);
    vi.mocked(fixSessionFiles).mockResolvedValue({ moved: 0, sidecars: 0, left: [{ file: 'DSC06785.ARW', reason: 'a file named DSC06785.ARW is already in T1-181126' }] });
    openMenu();
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Fix photo files location (1)' }));
    await waitFor(() =>
      expect(toast.error).toHaveBeenCalledWith('1 photo could not be moved', {
        description: 'DSC06785.ARW: a file named DSC06785.ARW is already in T1-181126',
      }),
    );
  });

  it('shows the refusal while an export runs', async () => {
    vi.mocked(sessionFilesMisplaced).mockResolvedValue(2);
    vi.mocked(fixSessionFiles).mockRejectedValue(new Error('An export of these photos is running. Wait for it to finish, then try again.'));
    openMenu();
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Fix photo files location (2)' }));
    await waitFor(() => expect(toast.error).toHaveBeenCalledWith('An export of these photos is running. Wait for it to finish, then try again.'));
  });
});
