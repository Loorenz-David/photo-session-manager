// The harness registers the service mocks; it must be evaluated before this file's own service imports.
import { captureFrames, compose, composition, DEFAULT_RECT, image, layoutStage, openWorkspace, photo, pointer, savedValues, stage, transformOf, twoPhotos, view } from './harness';
import { act, fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { setComposition, type Rect } from '@/services/compositions';

const near = (actual: number, expected: number, tolerance = 1e-6) => expect(Math.abs(actual - expected)).toBeLessThanOrEqual(tolerance);
const rectNear = (actual: Rect, expected: Rect, tolerance = 1e-6) => {
  near(actual.x, expected.x, tolerance);
  near(actual.y, expected.y, tolerance);
  near(actual.w, expected.w, tolerance);
  near(actual.h, expected.h, tolerance);
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
const middle = () => document.querySelector('.compose-middle');
const press = (key: string) => act(async () => { fireEvent.keyDown(window, { key }); });
const click = (name: string) => act(async () => { fireEvent.click(screen.getByRole('button', { name })); });
const chooseShape = (value: string) => act(async () => { fireEvent.change(document.querySelector('#compose-shape')!, { target: { value } }); });
const handle = (name: string) => document.querySelector<SVGElement>(`.compose-handle[data-handle="${name}"]`)!;
const imageSrc = () => image()!.querySelector('img')!.getAttribute('src');

async function editor(photos = twoPhotos()) {
  layoutStage();
  await openWorkspace([photos]);
  const frames = captureFrames();
  await compose();
  return frames;
}

/** The split default of the 2560×1707 frame from its default square: w_px = sqrt(1707²·2) = 2414.2 fits, centred (plan: (0.0285, 0.1465, 0.943, 0.7071)). */
const SPLIT_W_PX = Math.sqrt(1707 * 1707 * 2);
const SPLIT: Rect = { x: 0.5 - SPLIT_W_PX / 2560 / 2, y: 0.5 - SPLIT_W_PX / 2 / 1707 / 2, w: SPLIT_W_PX / 2560, h: SPLIT_W_PX / 2 / 1707 };
/** The 2:1 frame box in the 1000 × 700 stage: the largest 2:1 box in 952 × 652 is 952 × 476 at (24, 112). */
const SPLIT_BOX = { x: 24, y: 112, width: 952, height: 476 };
const SQUARE_BOX = { x: 174, y: 24, width: 652, height: 652 };

describe('C2 S and Normal | Split', () => {
  it('C2(a) S (and the Split control) saves kind split once with the shape kept and the re-fitted 2:1 rect; the frame becomes 2:1 with a hairline at its exact middle', async () => {
    await editor();
    boxNear(SQUARE_BOX);
    expect(middle()).toBeNull();
    await press('s');
    expect(setComposition).toHaveBeenCalledTimes(1);
    const values = savedValues(0);
    expect(values).toMatchObject({ kind: 'split', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0 });
    rectNear(values.rect, SPLIT, 1e-9);
    rectNear(values.rect, { x: 0.0285, y: 0.1465, w: 0.943, h: 0.7071 }, 1e-3);
    boxNear(SPLIT_BOX);
    expect(document.querySelector('.compose-frame')!.getAttribute('data-kind')).toBe('split');
    const hairline = middle()!;
    expect(hairline).not.toBeNull();
    near(Number(hairline.getAttribute('x1')), 500, 0.5);
    near(Number(hairline.getAttribute('x2')), 500, 0.5);
    near(Number(hairline.getAttribute('y1')), 112, 0.5);
    near(Number(hairline.getAttribute('y2')), 588, 0.5);
    expect(screen.getByRole('button', { name: 'Split' }).getAttribute('aria-pressed')).toBe('true');
    expect(screen.getByRole('button', { name: 'Normal' }).getAttribute('aria-pressed')).toBe('false');
    // A second press of the same control changes nothing.
    await click('Split');
    expect(setComposition).toHaveBeenCalledTimes(1);
  });

  it('C2(a) clicking Split in the segmented control does the same as S', async () => {
    await editor();
    await click('Split');
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(savedValues(0)).toMatchObject({ kind: 'split', shape: '1:1' });
    rectNear(savedValues(0).rect, SPLIT, 1e-9);
    boxNear(SPLIT_BOX);
    near(Number(middle()!.getAttribute('x1')), 500, 0.5);
  });

  it('C2(b) S again returns to Normal: the default square exactly, no hairline', async () => {
    await editor();
    await press('s');
    await press('s');
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(savedValues(1)).toMatchObject({ kind: 'normal', shape: '1:1' });
    rectNear(savedValues(1).rect, DEFAULT_RECT, 1e-6);
    expect(middle()).toBeNull();
    expect(document.querySelector('.compose-frame')!.getAttribute('data-kind')).toBe('normal');
    boxNear(SQUARE_BOX);
    expect(screen.getByRole('button', { name: 'Normal' }).getAttribute('aria-pressed')).toBe('true');
  });

  it('C2(c) while split the Shape menu and X change only the stored shape and orientation (the rect stays 2:1); S then gives a 5:4 Normal rect at the same centre and area', async () => {
    await editor();
    await press('s');
    await chooseShape('4:5');
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(savedValues(1)).toMatchObject({ kind: 'split', shape: '4:5', orientation: 'named' });
    expect(savedValues(1).rect).toEqual(savedValues(0).rect);
    boxNear(SPLIT_BOX);
    await press('x');
    expect(setComposition).toHaveBeenCalledTimes(3);
    expect(savedValues(2)).toMatchObject({ kind: 'split', shape: '4:5', orientation: 'turned' });
    expect(savedValues(2).rect).toEqual(savedValues(0).rect);
    near(savedValues(2).rect.w, 0.943, 1e-3);
    boxNear(SPLIT_BOX);
    expect(middle()).not.toBeNull();

    await press('s');
    expect(setComposition).toHaveBeenCalledTimes(4);
    const back = savedValues(3);
    expect(back).toMatchObject({ kind: 'normal', shape: '4:5', orientation: 'turned' });
    // 5:4 (4:5 turned) with the area 1707² and the centre (0.5, 0.5): w_px = sqrt(1707²·1.25) = 1908.5, h_px = 1526.8.
    const wPx = Math.sqrt(1707 * 1707 * 1.25);
    rectNear(back.rect, { x: 0.5 - wPx / 2560 / 2, y: 0.5 - wPx / 1.25 / 1707 / 2, w: wPx / 2560, h: wPx / 1.25 / 1707 }, 1e-9);
    near(back.rect.w * 2560, 1.25 * back.rect.h * 1707, 1e-6);
    expect(middle()).toBeNull();
    // The frame box is 5:4: the largest 1.25 box in 952 × 652 is 815 × 652.
    boxNear({ x: 92.5, y: 24, width: 815, height: 652 });
  });

  it('C2(d) a se-handle trim by (−100, −50) px keeps the rect exactly 2:1 and shrinks it from the locked nw corner', async () => {
    const frames = await editor();
    await press('s');
    const start = savedValues(0).rect;
    pointer('pointerdown', 976, 588, {}, handle('se'));
    pointer('pointermove', 876, 538);
    // While held the overlay draws the trimmed 2:1 box from the nw corner.
    boxNear({ x: 24, y: 112, width: 852, height: 426 });
    frames.flush();
    await act(async () => {
      pointer('pointerup', 876, 538);
    });
    expect(setComposition).toHaveBeenCalledTimes(2);
    const trimmed = savedValues(1);
    expect(trimmed.kind).toBe('split');
    near(trimmed.rect.w * 2560, 2 * trimmed.rect.h * 1707, 1);
    expect(trimmed.rect.w).toBeLessThan(start.w);
    expect(trimmed.rect.h).toBeLessThan(start.h);
    near(trimmed.rect.x, start.x, 1e-9);
    near(trimmed.rect.y, start.y, 1e-9);
    // 100 px of the 952 px frame: w = 0.943 · 852/952.
    near(trimmed.rect.w, (start.w * 852) / 952, 1e-6);
    boxNear(SPLIT_BOX);
    near(Number(middle()!.getAttribute('x1')), 500, 0.5);
  });
});

describe('C3 the Left and Right previews', () => {
  it('C3(a) while split the Composition section shows two square previews, Left and Right, each the same adjusted preview clipped to its half of the rect', async () => {
    await editor();
    expect(document.querySelector('.split-previews')).toBeNull();
    expect(document.querySelector('.output-preview')).not.toBeNull();
    await press('s');
    const rect = savedValues(0).rect;
    const previews = document.querySelectorAll<HTMLElement>('.split-previews .split-preview');
    expect(previews.length).toBe(2);
    expect(document.querySelector('.output-preview')).toBeNull();
    const [left, right] = [...previews];
    expect(left.dataset.side).toBe('left');
    expect(right.dataset.side).toBe('right');
    expect(left.style.aspectRatio).toBe('1 / 1');
    expect(right.style.aspectRatio).toBe('1 / 1');
    expect(left.getAttribute('aria-label')).toBe('Left');
    expect(right.getAttribute('aria-label')).toBe('Right');
    const labels = [...document.querySelectorAll('.split-preview-label')].map(node => node.textContent);
    expect(labels).toEqual(['Left', 'Right']);
    for (const box of [left, right]) {
      const img = box.querySelector('img')!;
      expect(img.getAttribute('src')).toBe('asset:///previews/1/adjusted-1.jpg');
      expect(img.getAttribute('src')).toBe(imageSrc());
    }
    // No new network resource: the stage image and the two previews are the only uses of the URL.
    expect(document.querySelectorAll('img[src="asset:///previews/1/adjusted-1.jpg"]').length).toBe(3);
    const leftHalf = left.querySelector<HTMLElement>('[data-half]')!;
    expect(leftHalf.dataset.half).toBe('left');
    near(Number(leftHalf.dataset.x0), rect.x, 1e-9);
    near(Number(leftHalf.dataset.x1), rect.x + rect.w / 2, 1e-9);
    const rightHalf = right.querySelector<HTMLElement>('[data-half]')!;
    expect(rightHalf.dataset.half).toBe('right');
    near(Number(rightHalf.dataset.x0), rect.x + rect.w / 2, 1e-9);
    near(Number(rightHalf.dataset.x1), rect.x + rect.w, 1e-9);
    // The clipped frame places the half: the frame is 1/(w/2) wide and 1/h tall, shifted by −x0/(w/2) and −y/h.
    near(parseFloat(leftHalf.style.width), (1 / (rect.w / 2)) * 100, 1e-6);
    near(parseFloat(leftHalf.style.height), (1 / rect.h) * 100, 1e-6);
    near(parseFloat(leftHalf.style.left), (-rect.x / (rect.w / 2)) * 100, 1e-6);
    near(parseFloat(rightHalf.style.left), (-(rect.x + rect.w / 2) / (rect.w / 2)) * 100, 1e-6);
  });
});

describe('C6 the P31/P32 flows on a split photo', () => {
  it('C6(a) [ ] navigate, Esc reverts to the entry values (kind split) and Return exits; nothing is saved when nothing changed', async () => {
    const split = photo(1, 'main', { position: 1, composition: composition(1, { kind: 'split', rect: SPLIT, revision: 2, origin: 'user' }) });
    const frames = await editor(view([split, photo(2, 'main', { position: 2 }), photo(5, 'main', { position: 3 })]));
    boxNear(SPLIT_BOX);
    expect(middle()).not.toBeNull();
    await press(']');
    expect(imageSrc()).toBe('asset:///previews/2/adjusted-1.jpg');
    boxNear(SQUARE_BOX);
    expect(middle()).toBeNull();
    await press('[');
    expect(imageSrc()).toBe('asset:///previews/1/adjusted-1.jpg');
    boxNear(SPLIT_BOX);
    expect(middle()).not.toBeNull();
    expect(setComposition).not.toHaveBeenCalled();

    // Esc with nothing changed: no save, the editor stays.
    await press('Escape');
    expect(setComposition).not.toHaveBeenCalled();
    expect(stage()).not.toBeNull();

    // A drag, then Esc: the revert saves the entry values — kind split, the stored 2:1 rect.
    pointer('pointerdown', 500, 350);
    pointer('pointermove', 550, 350);
    frames.flush();
    await act(async () => {
      pointer('pointerup', 550, 350);
    });
    expect(setComposition).toHaveBeenCalledTimes(1);
    expect(savedValues(0).kind).toBe('split');
    expect(savedValues(0).rect.x).toBeLessThan(SPLIT.x);
    await press('Escape');
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(savedValues(1).kind).toBe('split');
    rectNear(savedValues(1).rect, SPLIT, 1e-9);
    frames.flush();
    const placed = transformOf();
    // The split view: the frame box is 952 wide for w = 0.943 → s = 0.3943, the image 1009.5 × 673.1 centred at (500, 350).
    const scale = 952 / (SPLIT.w * 2560);
    near(placed.x, 500 - (2560 * scale) / 2, 0.5);
    near(placed.y, 350 - (1707 * scale) / 2, 0.5);

    await press('Enter');
    expect(stage()).toBeNull();
    expect(setComposition).toHaveBeenCalledTimes(2);
    expect(document.querySelector('.swipe-layer')).not.toBeNull();
  });
});
