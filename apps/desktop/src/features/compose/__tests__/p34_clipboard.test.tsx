import { compose, composition, layoutStage, openWorkspace, photo, view } from './harness';
import { fireEvent, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { applyComposition, setComposition } from '@/services/compositions';
import { toast } from 'sonner';

it('C3(a) copies values in memory and pastes a fitted composition on the next portrait photo', async () => {
  layoutStage();
  const first = photo(1, 'main', { position: 1 });
  first.composition = composition(1, { shape: '4:5', straighten: 2, rect: { x: 0.3, y: 0.18, w: 0.4, h: 0.48 } });
  const second = photo(2, 'main', { position: 2 });
  second.composition = composition(2, { frame: { w: 1707, h: 2560 }, rect: { x: 0, y: 0.1666, w: 1, h: 0.6668 } });
  await openWorkspace([view([first, second])]);
  await compose();
  fireEvent.keyDown(window, { key: 'c', metaKey: true, altKey: true });
  fireEvent.keyDown(window, { key: ']' });
  await waitFor(() => expect(document.querySelector('.stage-place')).toHaveTextContent('2 of 2'));
  fireEvent.keyDown(window, { key: 'v', metaKey: true, altKey: true });
  await waitFor(() => expect(setComposition).toHaveBeenCalled());
  const [id, values] = vi.mocked(setComposition).mock.calls[0];
  expect(id).toBe(2);
  expect(values).toMatchObject({ kind: 'normal', shape: '4:5', straighten: 2, orientation: 'named', quarter_turns: 0, flip: 'none' });
  expect(values.rect.x).toBeCloseTo(0.26002, 4);
  expect(values.rect.y).toBeCloseTo(0.21998, 4);
  expect(values.rect.w).toBeCloseTo(0.47995, 4);
  expect(values.rect.h).toBeCloseTo(0.40004, 4);
});

it('C3(b) applies the clipboard to selected photos in one call and names refusals', async () => {
  layoutStage();
  const photos = [photo(1, 'main', { position: 1 }), photo(2, 'main', { position: 2 }), photo(3, 'main', { position: 3 })];
  photos[0].composition = composition(1, { shape: '4:5', rect: { x: 0.3, y: 0.2, w: 0.4, h: 0.5 } });
  await openWorkspace([view(photos)]);
  await compose();
  fireEvent.keyDown(window, { key: 'c', metaKey: true, altKey: true });
  fireEvent.keyDown(window, { key: 'Enter' });
  const success = vi.spyOn(toast, 'success');
  vi.mocked(applyComposition).mockResolvedValue({ applied: [2], refused: [{ photo_id: 3, reason: 'off_shape' }] });
  const tiles = [...document.querySelectorAll<HTMLElement>('.photo-panel [data-photo-id]')];
  fireEvent.click(tiles[1], { metaKey: true });
  fireEvent.click(tiles[2], { metaKey: true });
  fireEvent.contextMenu(tiles[1]);
  const menuItem = await waitFor(() => {
    const found = [...document.querySelectorAll<HTMLElement>('[role="menuitem"]')].find(el => el.textContent?.includes('Apply composition to selected'));
    expect(found).toBeTruthy();
    return found!;
  });
  expect(menuItem).toBeTruthy();
  fireEvent.click(menuItem);
  await waitFor(() => expect(applyComposition).toHaveBeenCalledTimes(1));
  const items = vi.mocked(applyComposition).mock.calls[0][0];
  expect(items.map(item => item.photo_id)).toEqual([2, 3]);
  expect(items[0].values.shape).toBe('4:5');
  await waitFor(() => expect(success).toHaveBeenCalledWith('Applied to 1 photo; 1 refused: 03'));
  expect(document.querySelector('[data-photo-id="2"] .composed-frame')?.getAttribute('style')).not.toContain('left: -24.9854%');
});
