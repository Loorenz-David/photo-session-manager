// The harness registers the service mocks; it must be evaluated before this file's own service imports.
import { captureFrames, compose, composition, DEFAULT_RECT, image, layoutStage, openWorkspace, photo, pointer, savedValues, stage, transformOf, twoPhotos, view } from './harness';
import { act, cleanup, fireEvent, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { setComposition, type Rect } from '@/services/compositions';

const SCALE = 652 / 1707;
const near = (actual: number, expected: number, tolerance = 1e-6) => expect(Math.abs(actual - expected)).toBeLessThanOrEqual(tolerance);
const rectNear = (actual: Rect, expected: Rect, tolerance = 1e-6) => {
  near(actual.x, expected.x, tolerance);
  near(actual.y, expected.y, tolerance);
  near(actual.w, expected.w, tolerance);
  near(actual.h, expected.h, tolerance);
};
const placedAt = (x: number, y = 24, deg = 0) => {
  const placed = transformOf();
  expect(Math.abs(placed.x - x)).toBeLessThanOrEqual(0.5);
  expect(Math.abs(placed.y - y)).toBeLessThanOrEqual(0.5);
  expect(placed.deg).toBe(deg);
};
/** The overlay's frame rect (`x, y, width, height` attributes). */
function frameRect() {
  const rect = document.querySelector('.compose-frame')!;
  return { x: Number(rect.getAttribute('x')), y: Number(rect.getAttribute('y')), width: Number(rect.getAttribute('width')), height: Number(rect.getAttribute('height')) };
}
const boxNear = (box: { x: number; y: number; width: number; height: number }, tolerance = 1) => {
  const actual = frameRect();
  near(actual.x, box.x, tolerance);
  near(actual.y, box.y, tolerance);
  near(actual.width, box.width, tolerance);
  near(actual.height, box.height, tolerance);
};
const button = (name: string) => screen.getByRole('button', { name });
const click = (name: string) => act(async () => { if (name === 'Rotate and flip') fireEvent.pointerDown(button(name), { button: 0, ctrlKey: false, pointerType: 'mouse' }); fireEvent.click(button(name)); });
const chooseShape = (value: string) => act(async () => { fireEvent.change(document.querySelector('#compose-shape')!, { target: { value } }); });
const innerTransform = () => image()!.querySelector('img')!.style.transform;
const handle = (name: string) => document.querySelector<SVGElement>(`.compose-handle[data-handle="${name}"]`)!;

async function editor(photos = twoPhotos()) {
  layoutStage();
  await openWorkspace([photos]);
  const frames = captureFrames();
  await compose();
  return frames;
}

/** Re-fit (1): 1:1 → 4:5 on the default square (P28 case 3). */
const PORTRAIT: Rect = { x: 0.23328125, y: 0, w: 0.5334375, h: 1 };
/** Re-fit (4): the 4:5 rect turned to 5:4 with the same area and centre (exact: w_px = 1707, h_px = 1365.6). */
const TURNED: Rect = { x: 0.1666015625, y: 0.1, w: 0.666796875, h: 0.8 };
/** Re-fit (5): rotate right on the default square. */
const ROTATED: Rect = { x: 0, y: 0.16660156250000002, w: 1, h: 0.666796875 };

describe('C3 the Composition section', () => {
  it('C3(a) choosing 4:5 saves the re-fitted rect once and the frame box becomes the largest 0.8 box of the stage', async () => {
    await editor();
    boxNear({ x: 174, y: 24, width: 652, height: 652 });
    await chooseShape('4:5');
    expect(setComposition).toHaveBeenCalledTimes(1);
    const values = savedValues(0);
    expect(values).toMatchObject({ shape: '4:5', orientation: 'named', kind: 'normal', quarter_turns: 0, flip: 'none', straighten: 0 });
    rectNear(values.rect, PORTRAIT, 1e-9);
    boxNear({ x: 239.2, y: 24, width: 521.6, height: 652 });
    expect((document.querySelector('#compose-shape') as HTMLSelectElement).value).toBe('4:5');
  });

  it('C3(b) Turn frame (and X) turns the 4:5 frame to 5:4 keeping the area and centre', async () => {
    await editor();
    await chooseShape('4:5');
    await click('Turn frame');
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(savedValues(1)).toMatchObject({ orientation: 'turned', shape: '4:5' });
    rectNear(savedValues(1).rect, TURNED, 1e-9);
    boxNear({ x: 92.5, y: 24, width: 815, height: 652 });

    await act(async () => {
      fireEvent.keyDown(window, { key: 'x' });
    });
    expect(setComposition).toHaveBeenCalledTimes(3);
    expect(savedValues(2)).toMatchObject({ orientation: 'named', shape: '4:5' });
    rectNear(savedValues(2).rect, PORTRAIT, 1e-9);
    boxNear({ x: 239.2, y: 24, width: 521.6, height: 652 });
  });

  it('C3(c) Rotate right / left turn the photo under the frame (a 1:1 shape re-fits identically); Flip mirrors a free rect', async () => {
    const frames = await editor();
    await click('Rotate and flip');
    await act(async () => fireEvent.click(screen.getByRole('menuitem', { name: 'Rotate 90° right' })));
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(savedValues(0).quarter_turns).toBe(1);
    rectNear(savedValues(0).rect, ROTATED, 1e-9);
    expect(innerTransform()).toContain('rotate(90deg)');
    expect(image()!.dataset.turned).toBe('true');
    frames.flush();
    // The turned frame 1707 × 2560 at s = 652/1707: the image is 652 × 977.81, centred at (500, 350).
    placedAt(174, -138.9);

    await click('Rotate and flip');
    await act(async () => fireEvent.click(screen.getByRole('menuitem', { name: 'Rotate 90° left' })));
    expect(savedValues(1).quarter_turns).toBe(0);
    await click('Rotate and flip');
    await act(async () => fireEvent.click(screen.getByRole('menuitem', { name: 'Rotate 90° left' })));
    expect(setComposition).toHaveBeenCalledTimes(3);
    expect(savedValues(2).quarter_turns).toBe(3);
    rectNear(savedValues(2).rect, ROTATED, 1e-9);
    expect(innerTransform()).toContain('rotate(270deg)');

    const free = photo(1, 'main', { position: 1, composition: composition(1, { shape: 'free', rect: { x: 0.1, y: 0, w: 0.5, h: 0.75 } }) });
    cleanup();
    vi.mocked(setComposition).mockClear();
    await editor(view([free, photo(2, 'main', { position: 2 })]));
    await click('Rotate and flip');
    await act(async () => fireEvent.click(screen.getByRole('menuitem', { name: 'Flip horizontal' })));
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(savedValues(0).flip).toBe('horizontal');
    near(savedValues(0).rect.x, 0.4, 1e-12);
    expect(savedValues(0).rect.w).toBe(0.5);
    expect(innerTransform()).toContain('scaleX(-1)');
  });

  it('C3(d) the straighten slider tilts the photo live on input, saves once on change with the rect shrunk about the centre, and double-click levels it', async () => {
    const frames = await editor();
    const range = document.querySelector<HTMLInputElement>('#compose-straighten')!;
    expect(range.min).toBe('-45');
    expect(range.max).toBe('45');
    expect(range.step).toBe('0.1');
    await act(async () => {
      fireEvent.input(range, { target: { value: '10' } });
    });
    frames.flush();
    expect(image()!.style.transform).toMatch(/rotate\(10deg\)$/);
    expect(setComposition).not.toHaveBeenCalled();
    expect(document.querySelector<HTMLInputElement>('#compose-straighten-degrees')?.value).toBe('10.0');

    await act(async () => {
      fireEvent.change(range, { target: { value: '10' } });
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    const tilted = savedValues(0);
    expect(tilted.straighten).toBe(10);
    // The largest centred square inside the 10° tilted frame: h = 1 / (cos 10° + sin 10°) = 0.8632 (the contract's bisection;
    // the plan's `0.816 ± 0.005` is an arithmetic slip).
    const expectedH = 1 / (Math.cos(Math.PI / 18) + Math.sin(Math.PI / 18));
    near(tilted.rect.h, expectedH, 1e-5);
    near(tilted.rect.w, (tilted.rect.h * 1707) / 2560, 1e-9);
    near(tilted.rect.x + tilted.rect.w / 2, 0.5, 1e-9);
    near(tilted.rect.y + tilted.rect.h / 2, 0.5, 1e-9);

    await act(async () => {
      fireEvent.doubleClick(range);
    });
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(savedValues(1).straighten).toBe(0);
    // Area is preserved: only the tilt goes, the rect stays at the shrunk size.
    rectNear(savedValues(1).rect, tilted.rect, 1e-9);
    expect(document.querySelector<HTMLInputElement>('#compose-straighten-degrees')?.value).toBe('0.0');
    frames.flush();
    expect(image()!.style.transform).toMatch(/rotate\(0deg\)$/);
  });

  it('C3(e) the output preview has the rect\'s pixel ratio and shows the same adjusted preview', async () => {
    await editor();
    const preview = () => document.querySelector<HTMLElement>('.output-preview')!;
    expect(preview().style.aspectRatio).toBe('1707 / 1707');
    const img = preview().querySelector('img')!;
    expect(img.getAttribute('src')).toBe('asset:///previews/1/adjusted-1.jpg');
    expect(img.getAttribute('src')).toBe(image()!.querySelector('img')!.getAttribute('src'));
    expect(document.querySelectorAll('img[src="asset:///previews/1/adjusted-1.jpg"]').length).toBe(2);
    await chooseShape('4:5');
    expect(preview().style.aspectRatio).toBe('1365.6 / 1707');
  });
});

describe('C4 the trim handles', () => {
  it('C4(a) a locked se-corner trim shrinks the overlay frame from the nw corner while held; on release one save and the frame glides back to fill the stage', async () => {
    const frames = await editor();
    expect(document.querySelectorAll('.compose-handle').length).toBe(8);
    pointer('pointerdown', 826, 676, {}, handle('se'));
    pointer('pointermove', 726, 576);
    boxNear({ x: 174, y: 24, width: 552, height: 552 });
    frames.flush();
    placedAt(11.09);
    expect(setComposition).not.toHaveBeenCalled();
    await act(async () => {
      pointer('pointerup', 726, 576);
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    const h = 1 - 100 / 652;
    rectNear(savedValues(0).rect, { x: DEFAULT_RECT.x, y: 0, w: (h * 1707) / 2560, h }, 1e-5);
    boxNear({ x: 174, y: 24, width: 652, height: 652 });
    expect(stage()!.dataset.gliding).toBe('true');
    frames.flush();
    // The view re-scaled to the tighter crop: the image grew by 1 / h.
    near(parseFloat(image()!.style.width), 977.81 / h, 1);
    fireEvent.transitionEnd(image()!);
    expect(stage()!.dataset.gliding).toBeUndefined();
  });

  it('C4(b) on a free shape the e handle moves that edge only', async () => {
    const free = photo(1, 'main', { position: 1, composition: composition(1, { shape: 'free' }) });
    await editor(view([free, photo(2, 'main', { position: 2 })]));
    pointer('pointerdown', 826, 350, {}, handle('e'));
    pointer('pointermove', 726, 350);
    await act(async () => {
      pointer('pointerup', 726, 350);
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    const values = savedValues(0);
    near(values.rect.w, (1707 - 100 / SCALE) / 2560, 1e-5);
    expect(values.rect.h).toBe(1);
    expect(values.rect.x).toBe(DEFAULT_RECT.x);
    expect(values.shape).toBe('free');
  });

  it('C4(c) Esc mid-trim restores the frame and saves nothing', async () => {
    await editor();
    pointer('pointerdown', 826, 676, {}, handle('se'));
    pointer('pointermove', 726, 576);
    boxNear({ x: 174, y: 24, width: 552, height: 552 });
    fireEvent.keyDown(window, { key: 'Escape' });
    boxNear({ x: 174, y: 24, width: 652, height: 652 });
    await act(async () => {
      pointer('pointerup', 726, 576);
    });
    expect(setComposition).not.toHaveBeenCalled();
    expect(stage()).not.toBeNull();
    expect(stage()!.dataset.gliding).toBeUndefined();
  });
});
