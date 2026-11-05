import { compose, layoutStage, openWorkspace, pointer, saved, twoPhotos } from './harness';
import { act, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { setComposition } from '@/services/compositions';
import { toast } from 'sonner';
import config from '../../../../src-tauri/tauri.conf.json';
import { StageImage } from '@/features/workspace/StageImage';
import { render } from '@testing-library/react';

it('C4(a) a save in flight marks the active stage and browser tile until it succeeds or fails', async () => {
  layoutStage();
  await openWorkspace([twoPhotos()]);
  await compose();
  let finish!: (answer: ReturnType<typeof saved>) => void;
  vi.mocked(setComposition).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  pointer('pointerdown', 500, 350);
  pointer('pointermove', 550, 350);
  pointer('pointerup', 550, 350);
  await waitFor(() => expect(document.querySelectorAll('.tile-unsaved').length).toBeGreaterThanOrEqual(2));
  await act(async () => finish(saved(1, vi.mocked(setComposition).mock.calls[0][1])));
  await waitFor(() => expect(document.querySelectorAll('.tile-unsaved').length).toBe(0));
  let reject!: (error: Error) => void;
  vi.mocked(setComposition).mockImplementation(() => new Promise((_, fail) => { reject = fail; }));
  pointer('pointerdown', 500, 350);
  pointer('pointermove', 600, 350);
  pointer('pointerup', 600, 350);
  await waitFor(() => expect(document.querySelectorAll('.tile-unsaved').length).toBeGreaterThanOrEqual(2));
  await act(async () => reject(new Error('refused')));
  await waitFor(() => expect(document.querySelectorAll('.tile-unsaved').length).toBe(0));
  expect(toast.error).toHaveBeenCalledWith('Not saved: refused');
});

it('C6(a) the desktop window starts at 1440 by 900 with the planned minimum', () => {
  expect(config.app.windows[0]).toMatchObject({ width: 1440, height: 900, minWidth: 1100, minHeight: 720 });
});

it('C5(a,b) the Original and Adjusted layers use zero crossfade in reduced motion and 700 ms normally', () => {
  const source = twoPhotos().photos[0];
  const prior = window.matchMedia;
  window.matchMedia = vi.fn().mockReturnValue({ matches: true });
  const reduced = render(<StageImage photo={source} mode="adjusted" />);
  expect(reduced.container.querySelector('.crossfade')).toHaveStyle({ transitionDuration: '0ms' });
  reduced.unmount();
  window.matchMedia = vi.fn().mockReturnValue({ matches: false });
  const normal = render(<StageImage photo={source} mode="adjusted" />);
  expect(normal.container.querySelector('.crossfade')).toHaveStyle({ transitionDuration: '700ms' });
  normal.unmount();
  window.matchMedia = prior;
});
