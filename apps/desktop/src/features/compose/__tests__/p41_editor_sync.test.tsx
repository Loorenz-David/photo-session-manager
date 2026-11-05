import { composition, compose, DEFAULT_RECT, delivered, openWorkspace, photo, SESSION, view } from './harness';
import { act, fireEvent, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { setComposition } from '@/services/compositions';
import { getSessionView } from '@/services/photos';

it('C5(a) an undone framing in the open editor becomes the stored value; Esc then saves the entry rect', async () => {
  const entry = view([photo(1, 'main', { position: 1 })]);
  await openWorkspace([entry]);
  await compose();
  const restored = composition(1, {
    rect: { x: 0.25, y: (1 - 1280 / 1707) / 2, w: 0.5, h: 1280 / 1707 },
    revision: 2, origin: 'user', updated_at: 2,
  });
  vi.mocked(getSessionView).mockResolvedValue(view([photo(1, 'main', { position: 1, composition: restored })]));
  await act(async () => delivered.workspace.forEach(send => send({ kind: 'photo_changed', photo_id: 1, session_id: SESSION })));
  await waitFor(() => expect(getSessionView).toHaveBeenCalledTimes(2));
  expect(setComposition).not.toHaveBeenCalled();
  fireEvent.keyDown(window, { key: 'Escape' });
  await waitFor(() => expect(setComposition).toHaveBeenCalledTimes(1));
  const [photoId, values] = vi.mocked(setComposition).mock.calls[0];
  expect(photoId).toBe(1);
  expect(values.rect.x).toBeCloseTo(DEFAULT_RECT.x, 6);
  expect(values.rect.y).toBeCloseTo(DEFAULT_RECT.y, 6);
  expect(values.rect.w).toBeCloseTo(DEFAULT_RECT.w, 6);
  expect(values.rect.h).toBeCloseTo(DEFAULT_RECT.h, 6);
});
