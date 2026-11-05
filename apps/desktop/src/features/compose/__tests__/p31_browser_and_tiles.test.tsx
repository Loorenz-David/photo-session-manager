// The harness registers the service mocks; it must be evaluated before this file's own imports.
import { compose, layoutStage, openWorkspace, photo, pointer, tile, twoPhotos, view } from './harness';
import { act, fireEvent } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

const browser = () => document.querySelector<HTMLElement>('.compose-browser')!;
const edge = (target: Element) => fireEvent.pointerMove(target, { clientX: window.innerWidth - 5, clientY: 300 });

describe('C8 the browser overlay and the tiles', () => {
  it('C8(a) the browser slides in over the stage at the window\'s right edge and hides when the pointer leaves it', async () => {
    layoutStage();
    await openWorkspace([twoPhotos()]);
    await compose();
    expect(browser().getAttribute('data-open')).toBe('false');
    edge(document.querySelector('main.workspace')!);
    expect(browser().getAttribute('data-open')).toBe('true');
    expect([...browser().querySelectorAll('[data-photo-id]')].map(t => t.getAttribute('data-photo-id'))).toEqual(['1', '2']);
    fireEvent.pointerLeave(browser());
    expect(browser().getAttribute('data-open')).toBe('false');
    expect(browser().className).toContain('compose-browser');
    expect(browser().closest('.stage-column')).not.toBeNull();
    expect(browser().parentElement?.matches('main.workspace')).toBe(false);
  });

  it('C8(b) the edge never opens the browser during a drag; it opens on the next move after the release', async () => {
    layoutStage();
    await openWorkspace([twoPhotos()]);
    await compose();
    pointer('pointerdown', 500, 350);
    pointer('pointermove', window.innerWidth - 5, 350);
    expect(browser().getAttribute('data-open')).toBe('false');
    await act(async () => {
      pointer('pointerup', window.innerWidth - 5, 350);
    });
    expect(browser().getAttribute('data-open')).toBe('false');
    pointer('pointermove', window.innerWidth - 5, 350);
    expect(browser().getAttribute('data-open')).toBe('true');
  });

  it('C8(c) grid tiles show the thumbnail with the composition drawn over it; a turned frame and the Re-frame corner', async () => {
    const framed = photo(1, 'main', { position: 1 });
    const unframed = photo(2, 'main', { position: 2 });
    unframed.composition = { ...unframed.composition, rect: null, frame: null };
    const turned = photo(3, 'main', { position: 3 });
    turned.composition = { ...turned.composition, quarter_turns: 1, rect: { x: 0, y: 0.16660156250000002, w: 1, h: 0.666796875 }, state: 'needs_review', review_reason: 'geometry' };
    await openWorkspace([view([framed, unframed, turned])]);
    const panel = document.querySelector('.photo-panel')!;

    const first = tile(1, panel);
    expect(first.querySelector('img.thumb-image')?.getAttribute('src')?.endsWith('thumb-1.jpg')).toBe(true);
    const box = first.querySelector<HTMLElement>('.tile-composed')!;
    expect(box.style.aspectRatio).toBe('1707 / 1707');
    const frame = first.querySelector<HTMLElement>('.composed-frame')!;
    expect(frame.style.left).toBe('-24.9854%');
    expect(frame.style.top).toBe('0%');
    expect(frame.style.width).toBe('149.9707%');
    expect(frame.style.height).toBe('100%');
    expect(first.querySelector('.tile-review')).toBeNull();

    expect(tile(2, panel).querySelector('.tile-composed')).toBeNull();
    expect(tile(2, panel).querySelector('img.thumb-image')?.getAttribute('src')).toBe('asset:///previews/2/thumb-1.jpg');

    const third = tile(3, panel);
    expect(third.querySelector<HTMLElement>('.tile-composed')!.style.aspectRatio).toBe('1707 / 1707');
    expect(third.querySelector('.tile-review')).toHaveTextContent('Re-frame');
  });
});
