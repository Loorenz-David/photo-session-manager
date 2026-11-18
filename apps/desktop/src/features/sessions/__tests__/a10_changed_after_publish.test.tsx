import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/services/sessions', async importOriginal => ({
  ...(await importOriginal<typeof import('@/services/sessions')>()),
  keepPublished: vi.fn(async () => undefined),
}));
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import { ChangedPill, changeReasons } from '@/features/sessions/ChangedPill';
import { keepPublished } from '@/services/sessions';

afterEach(cleanup);

const none = { new_photos: 0, editor_edits: 0, app_edits: false, unrecorded: false };

describe('A10: a published session that changed since says why', () => {
  it('names each reason in words', () => {
    expect(changeReasons({ ...none, new_photos: 2 })).toEqual(['2 new photos']);
    expect(changeReasons({ ...none, new_photos: 1, editor_edits: 3, app_edits: true })).toEqual([
      '1 new photo',
      '3 photos edited in Capture One',
      'edited in the app',
    ]);
    expect(changeReasons({ ...none, editor_edits: 1 })).toEqual(['1 photo edited in Capture One']);
    expect(changeReasons({ ...none, unrecorded: true })).toEqual(['changed before this version noted why']);
  });

  it('shows nothing for a session that did not change', () => {
    render(<ChangedPill sessionId={4} changes={null} />);
    expect(screen.queryByText(/Changed after publishing/)).toBeNull();
  });

  it('"Keep as published" calls the command for that session, then tells the caller', async () => {
    const kept = vi.fn();
    render(<ChangedPill sessionId={89} changes={{ ...none, new_photos: 2 }} onKept={kept} />);
    expect(screen.getByText('Changed after publishing · 2 new photos')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Keep as published' }));
    await waitFor(() => expect(kept).toHaveBeenCalled());
    expect(keepPublished).toHaveBeenCalledWith(89);
  });
});
