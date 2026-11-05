import { compose, layoutStage, openWorkspace, pointer, stage, twoPhotos } from './harness';
import { fireEvent, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { setComposition } from '@/services/compositions';

it('C2(a) Z shows a pointer-following 100 percent loupe and releases without saving', async () => {
  layoutStage();
  await openWorkspace([twoPhotos()]);
  await compose();
  pointer('pointermove', 600, 300);
  fireEvent.keyDown(window, { key: 'z' });
  const loupe = document.querySelector<HTMLElement>('.compose-loupe')!;
  expect(loupe.style.left).toBe('616px');
  expect(loupe.style.top).toBe('316px');
  const view = loupe.querySelector<HTMLElement>('.compose-loupe-view')!;
  const img = loupe.querySelector('img')!;
  expect(img.getAttribute('src')).toBe('asset:///previews/1/adjusted-1.jpg');
  expect(img.style.width).toBe('2560px');
  expect(Number.parseFloat(view.style.left)).toBeCloseTo(-1421.8, 1);
  expect(Number.parseFloat(view.style.top)).toBeCloseTo(-602.6, 1);
  fireEvent.keyUp(window, { key: 'z' });
  await waitFor(() => expect(document.querySelector('.compose-loupe')).toBeNull());
  expect(setComposition).not.toHaveBeenCalled();
});

it('C2(a) renders the source-sized preview with the photo turn, flip and straighten', async () => {
  layoutStage();
  const turned = twoPhotos();
  turned.photos[0].composition = { ...turned.photos[0].composition, quarter_turns: 1, flip: 'horizontal', straighten: 8 };
  await openWorkspace([turned]);
  await compose();
  pointer('pointermove', 600, 300);
  fireEvent.keyDown(window, { key: 'z' });
  const loupe = document.querySelector<HTMLElement>('.compose-loupe')!;
  const view = loupe.querySelector<HTMLElement>('.compose-loupe-view')!;
  const img = loupe.querySelector('img')!;
  expect(view.style.width).toBe('1707px');
  expect(view.style.height).toBe('2560px');
  expect(view.style.transform).toBe('rotate(8deg)');
  expect(Number.parseFloat(view.style.left) + Number.parseFloat(view.style.transformOrigin.split(' ')[0])).toBeCloseTo(120, 8);
  expect(Number.parseFloat(view.style.top) + Number.parseFloat(view.style.transformOrigin.split(' ')[1])).toBeCloseTo(120, 8);
  expect(img.style.width).toBe('2560px');
  expect(img.style.height).toBe('1707px');
  expect(img.style.transform).toBe('rotate(90deg) scaleX(-1)');
  fireEvent.keyUp(window, { key: 'z' });
});

it('C2(b) Alt suspends drag snapping and the guide disappears after release', async () => {
  layoutStage();
  await openWorkspace([twoPhotos()]);
  await compose();
  pointer('pointerdown', 500, 350);
  pointer('pointermove', 503.7, 350);
  expect(stage()?.querySelector('.compose-guide[data-axis="x"]')).not.toBeNull();
  pointer('pointerup', 503.7, 350);
  expect(stage()?.querySelector('.compose-guide')).toBeNull();
  pointer('pointerdown', 500, 350);
  pointer('pointermove', 503.7, 350, { altKey: true });
  expect(stage()?.querySelector('.compose-guide')).toBeNull();
  pointer('pointerup', 503.7, 350);
  await waitFor(() => expect(vi.mocked(setComposition).mock.calls.length).toBeGreaterThan(0));
});
