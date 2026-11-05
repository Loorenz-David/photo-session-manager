// The harness registers the service mocks; it must be evaluated before this file's own service imports.
import { captureFrames, compose, composition, DEFAULT_RECT, image, layoutStage, openWorkspace, photo, pointer, saved, savedValues, stage, tile, transformOf, twoPhotos, view } from './harness';
import { act, fireEvent, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { resetComposition, setComposition, type Composition } from '@/services/compositions';
import { classifyPhoto, deletePhoto, getSessionView } from '@/services/photos';

const PX = 1 / ((2560 * 652) / 1707); // one screen pixel as a fraction of the turned frame's width (1/977.81)
const PANNED_X = 0.11546683;
const near = (actual: number, expected: number, tolerance = 1e-6) => expect(Math.abs(actual - expected)).toBeLessThanOrEqual(tolerance);
const placedAt = (x: number, y = 24) => {
  const placed = transformOf();
  expect(Math.abs(placed.x - x)).toBeLessThanOrEqual(0.5);
  expect(Math.abs(placed.y - y)).toBeLessThanOrEqual(0.5);
};
const key = (key: string, init: KeyboardEventInit = {}) => act(async () => { fireEvent.keyDown(window, { key, ...init }); });

async function editor(photos = twoPhotos()) {
  layoutStage();
  await openWorkspace([photos]);
  const frames = captureFrames();
  await compose();
  return frames;
}

async function drag(x0: number, x1: number, flush?: () => void) {
  pointer('pointerdown', x0, 350);
  pointer('pointermove', x1, 350);
  flush?.();
  await act(async () => {
    pointer('pointerup', x1, 350);
  });
}

describe('C5 the editor keys', () => {
  it('C5(a) arrows nudge the frame one screen pixel in the arrow\'s direction (Shift ×10); no room → nothing to save', async () => {
    await editor();
    await key('ArrowRight');
    expect(setComposition).toHaveBeenCalledTimes(1);
    near(savedValues(0).rect.x, DEFAULT_RECT.x + PX);
    expect(savedValues(0).rect.y).toBe(0);
    await key('ArrowRight', { shiftKey: true });
    expect(setComposition).toHaveBeenCalledTimes(2);
    near(savedValues(1).rect.x, DEFAULT_RECT.x + 11 * PX);
    await key('ArrowDown');
    // h = 1: the frame cannot move down; the rect is unchanged and nothing is sent.
    expect(setComposition).toHaveBeenCalledTimes(2);
    await key('ArrowLeft');
    expect(setComposition).toHaveBeenCalledTimes(3);
    near(savedValues(2).rect.x, DEFAULT_RECT.x + 10 * PX);
    expect(vi.mocked(setComposition).mock.calls.every(call => call[1].rect.y === 0)).toBe(true);
  });

  it('C5(b) `.` and the Centre button centre the rect after a drag', async () => {
    const frames = await editor();
    await drag(500, 550, frames.flush);
    near(savedValues(0).rect.x, PANNED_X);
    await key('.');
    expect(setComposition).toHaveBeenCalledTimes(2);
    near(savedValues(1).rect.x, 0.166601562);
    expect(savedValues(1).rect.y).toBe(0);
    frames.flush();
    placedAt(11.09);

    await drag(500, 550, frames.flush);
    expect(setComposition).toHaveBeenCalledTimes(3);
    await act(async () => {
      fireEvent.click(screen.getByRole('button', { name: 'Centre' }));
    });
    expect(setComposition).toHaveBeenCalledTimes(4);
    near(savedValues(3).rect.x, 0.166601562);
    expect(savedValues(3).rect.y).toBe(0);
  });

  it('C5(c) 1–6 and 0 pick Original, 1:1, 4:5, 3:2, 2:3, 16:9, Free in turn, each re-fitted from the previous', async () => {
    await editor();
    for (const k of ['1', '2', '3', '4', '5', '6', '0']) await key(k);
    expect(setComposition).toHaveBeenCalledTimes(7);
    expect(vi.mocked(setComposition).mock.calls.map(call => call[1].shape)).toEqual(['original', '1:1', '4:5', '3:2', '2:3', '16:9', 'free']);
    expect(savedValues(0).rect).toEqual({ x: 0, y: 0, w: 1, h: 1 });
    near(savedValues(1).rect.w, DEFAULT_RECT.w, 1e-9); // 1:1 from the whole frame: the area asks for more than the frame holds → the default square
    near(savedValues(2).rect.w, 0.5334375, 1e-9);
    // 3:2 from 4:5 (area 1365.6 × 1707): w_px = sqrt(area · 1.5) = 1870, h_px = 1246.6 — fits.
    near(savedValues(3).rect.w, Math.sqrt(1365.6 * 1707 * 1.5) / 2560, 1e-6);
    near(savedValues(3).rect.h, Math.sqrt(1365.6 * 1707 * 1.5) / 1.5 / 1707, 1e-6);
    expect(savedValues(6).rect).toEqual(savedValues(5).rect); // free keeps the rect
    expect((document.querySelector('#compose-shape') as HTMLSelectElement).value).toBe('free');
  });

  it('C5(d) `]` and `[` move through the display sequence keeping the changes, without wrapping; a pending save lands on its own photo', async () => {
    let resolveP1!: (composition: Composition) => void;
    vi.mocked(setComposition).mockImplementationOnce(() => new Promise(done => { resolveP1 = done; }));
    const frames = await editor(view([photo(1, 'main', { position: 1 }), photo(2, 'main', { position: 2 }), photo(5, 'main', { position: 3 })]));
    const loads = vi.mocked(getSessionView).mock.calls.length;
    await drag(500, 550, frames.flush);
    expect(setComposition).toHaveBeenCalledTimes(1);
    const shows = (id: number) => expect(image()!.querySelector('img')!.getAttribute('src')).toBe(`asset:///previews/${id}/adjusted-1.jpg`);
    await key(']');
    shows(2);
    await key(']');
    shows(5);
    await key(']');
    shows(5);
    await key('[');
    shows(2);
    await key('[');
    shows(1);
    await key('[');
    shows(1);
    expect(stage()).not.toBeNull();
    await act(async () => {
      resolveP1(saved(1, savedValues(0), 1));
    });
    // Photo 1 kept its change: the stored answer is what the editor shows on it (machine rule 16), tile 1 follows.
    frames.flush();
    placedAt(61.09);
    const browser = document.querySelector('.folded-strip')!;
    expect(tile(1, browser).querySelector<HTMLElement>('.composed-frame')!.style.left).toBe('-17.3166%');
    expect(tile(2, browser).querySelector<HTMLElement>('.composed-frame')!.style.left).toBe('-24.9854%');
    expect(tile(5, browser).querySelector<HTMLElement>('.composed-frame')!.style.left).toBe('-24.9854%');
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(vi.mocked(getSessionView).mock.calls.length).toBe(loads);
  });

  it('C5(e) ⌘⌥R and Reset call resetComposition once and the returned default becomes the view', async () => {
    vi.mocked(resetComposition).mockResolvedValue(composition(1));
    const frames = await editor();
    await drag(500, 550, frames.flush);
    frames.flush();
    placedAt(61.09);
    await key('r', { metaKey: true, altKey: true });
    expect(resetComposition).toHaveBeenCalledTimes(1);
    expect(resetComposition).toHaveBeenCalledWith(1);
    frames.flush();
    placedAt(11.09);
    expect(setComposition).toHaveBeenCalledTimes(1);

    await drag(500, 550, frames.flush);
    frames.flush();
    placedAt(61.09);
    await act(async () => {
      fireEvent.click(screen.getByRole('button', { name: 'Reset' }));
    });
    expect(resetComposition).toHaveBeenCalledTimes(2);
    frames.flush();
    placedAt(11.09);
  });
});

describe('C6 keys that belong elsewhere', () => {
  it('C6(a) the workspace and app keys, ⌘[ among them, do nothing while composing', async () => {
    await editor();
    const hash = window.location.hash;
    for (const k of ['a', 'c', 'Backspace', ' ', 'f', 'i', 'd', 'n']) fireEvent.keyDown(window, { key: k });
    fireEvent.keyDown(window, { key: '[', metaKey: true });
    expect(classifyPhoto).not.toHaveBeenCalled();
    expect(deletePhoto).not.toHaveBeenCalled();
    expect(document.querySelector('[role="dialog"]')).toBeNull();
    expect(document.querySelector('.item-card')).toBeNull();
    expect(document.querySelector('.toggle-pill')).toBeNull();
    expect(document.querySelector('.folded-strip')).toBeNull();
    expect(window.location.hash).toBe(hash);
    expect(stage()).not.toBeNull();
    expect(setComposition).not.toHaveBeenCalled();
  });

  it('C6(b) a key inside the straighten slider belongs to the slider: no nudge, the browser keeps its default', async () => {
    await editor();
    const range = document.querySelector<HTMLInputElement>('#compose-straighten')!;
    range.focus();
    const event = new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, cancelable: true });
    range.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(false);
    expect(setComposition).not.toHaveBeenCalled();
  });
});
