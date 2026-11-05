import { compose, layoutStage, openWorkspace, pointer, twoPhotos } from './harness';
import { act, fireEvent, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';

it('C7(a) toggles the frame meter and reports the last second average and worst frame', async () => {
  layoutStage();
  const callbacks: FrameRequestCallback[] = [];
  vi.spyOn(window, 'requestAnimationFrame').mockImplementation(callback => { callbacks.push(callback); return callbacks.length; });
  vi.spyOn(window, 'cancelAnimationFrame').mockImplementation(() => undefined);
  await openWorkspace([twoPhotos()]);
  await compose();
  fireEvent.keyDown(window, { key: 'p', metaKey: true, altKey: true });
  expect(document.querySelector('.frame-meter')).toHaveTextContent('avg 0 ms · worst 0 ms');
  pointer('pointerdown', 500, 350);
  let at = 0;
  for (const delta of [0, ...Array(10).fill(16), 48]) {
    at += delta;
    const current = callbacks.splice(0);
    act(() => current.forEach(callback => callback(at)));
  }
  await waitFor(() => expect(document.querySelector('.frame-meter')).toHaveTextContent('avg 19 ms · worst 48 ms'));
  fireEvent.keyDown(window, { key: 'p', metaKey: true, altKey: true });
  expect(document.querySelector('.frame-meter')).toBeNull();
});
