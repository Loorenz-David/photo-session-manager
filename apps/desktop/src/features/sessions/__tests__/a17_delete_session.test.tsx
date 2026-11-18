import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  sessionDeletePreview: vi.fn(),
  deleteSession: vi.fn(),
}));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { toast } from 'sonner';
import { SessionActionsMenu } from '@/features/sessions/DeleteSession';
import { deleteSession, sessionDeletePreview } from '@/services/sessions';

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

const preview = {
  session_id: 2,
  sku: 'test-4-4',
  title: 'Test chair',
  item_type: 'Armchairs',
  photos: 2,
  published: true,
  archived: true,
  active: false,
};

afterEach(cleanup);
beforeEach(() => {
  vi.mocked(sessionDeletePreview).mockReset().mockResolvedValue(preview);
  vi.mocked(deleteSession).mockReset().mockResolvedValue({ sku: 'test-4-4', photos: 2, files_trashed: 5, files_missing: 1, folder_trashed: true });
  vi.mocked(toast.success).mockReset();
  vi.mocked(toast.error).mockReset();
});

async function openDialog(onDeleted = vi.fn()) {
  render(<SessionActionsMenu sessionId={2} onDeleted={onDeleted} />);
  fireEvent.pointerDown(screen.getByRole('button', { name: 'Session actions' }), { button: 0 });
  fireEvent.click(await screen.findByRole('menuitem', { name: 'Delete session…' }));
  await screen.findByRole('dialog');
  return onDeleted;
}

describe('A17: Delete session (Session Manager only)', () => {
  it('the ⋯ menu offers "Delete session…", and the dialog says what happens before anything is deleted', async () => {
    await openDialog();
    expect(await screen.findByText('Delete test-4-4?')).toBeInTheDocument();
    expect(screen.getByText('Armchairs · 2 photos · published')).toBeInTheDocument();
    expect(screen.getByText(/go to the macOS Trash\. You can put them back from there\./)).toBeInTheDocument();
    expect(screen.getByText(/This cannot be undone in the app\./)).toBeInTheDocument();
    expect(screen.getByText('The product in Shopify is not changed.')).toBeInTheDocument();
    expect(screen.getByText(/already cleared after publishing/)).toBeInTheDocument();
    expect(sessionDeletePreview).toHaveBeenCalledWith(2);
    expect(deleteSession).not.toHaveBeenCalled();
  });

  it('Cancel closes the dialog without deleting', async () => {
    const onDeleted = await openDialog();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(deleteSession).not.toHaveBeenCalled();
    expect(onDeleted).not.toHaveBeenCalled();
  });

  it('the confirm button deletes that session, reports the files moved to the Trash and tells the caller', async () => {
    const onDeleted = await openDialog();
    await screen.findByText('Delete test-4-4?');
    fireEvent.click(screen.getByRole('button', { name: /Delete session/ }));
    await waitFor(() => expect(onDeleted).toHaveBeenCalled());
    expect(deleteSession).toHaveBeenCalledWith(2);
    expect(toast.success).toHaveBeenCalledWith('Deleted test-4-4', { description: '5 files moved to the Trash' });
  });

  it('a refusal is shown and nothing is reported deleted', async () => {
    vi.mocked(deleteSession).mockRejectedValue(new Error('This session is being published. Wait for the publish to finish, then delete.'));
    const onDeleted = await openDialog();
    await screen.findByText('Delete test-4-4?');
    fireEvent.click(screen.getByRole('button', { name: /Delete session/ }));
    await waitFor(() => expect(toast.error).toHaveBeenCalledWith('This session is being published. Wait for the publish to finish, then delete.'));
    expect(onDeleted).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('a session photos are filed into says new photos go to Unassigned', async () => {
    vi.mocked(sessionDeletePreview).mockResolvedValue({ ...preview, published: false, archived: false, active: true });
    await openDialog();
    expect(await screen.findByText('New photos go to Unassigned from now on.')).toBeInTheDocument();
    expect(screen.getByText('Shopify is not changed.')).toBeInTheDocument();
  });
});
