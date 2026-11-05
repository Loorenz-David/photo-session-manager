import { compose, layoutStage, openWorkspace, pointer, stage, twoPhotos } from './harness';
import { fireEvent, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { setComposition } from '@/services/compositions';

it('C2(a) Z shows a pointer-following 100 percent loupe and releases without saving', async () => {
  layoutStage();
  await openWorkspace([twoPhotos()]);
  await compose();
  fireEvent.keyDown(window, { key: 'z' });
  pointer('pointermove', 600, 300);
  const loupe = document.querySelector<HTMLElement>('.compose-loupe')!;
  expect(loupe.style.left).toBe('616px');
  expect(loupe.style.top).toBe('316px');
  const img = loupe.querySelector('img')!;
  expect(img.getAttribute('src')).toBe('asset:///previews/1/adjusted-1.jpg');
  expect(img.style.width).toBe('2560px');
  expect(img.style.transform).toContain('translate(');
  fireEvent.keyUp(window, { key: 'z' });
  await waitFor(() => expect(document.querySelector('.compose-loupe')).toBeNull());
  expect(setComposition).not.toHaveBeenCalled();
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
