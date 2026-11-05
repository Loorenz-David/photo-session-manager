import { DEFAULT_RECT, composition, openWorkspace, photo, tile, view } from '@/features/compose/__tests__/harness';
import { act, fireEvent, screen } from '@testing-library/react';
import { expect, it } from 'vitest';

const splitRect = { x: 0.028503408013501264, y: 0.1464466094067262, w: 0.9429931839729975, h: 0.7071067811865476 };

it('C2(a) stage shows the square output instead of a frame drawn on the full image', async () => {
  await openWorkspace([view([photo(1, 'main', { position: 1 })])]);
  const box = document.querySelector<HTMLElement>('.stage-frame .stage-composed')!;
  expect(box.style.aspectRatio).toBe('1707 / 1707');
  const frame = box.querySelector<HTMLElement>('.composed-frame')!;
  expect([frame.style.width, frame.style.height, frame.style.left, frame.style.top, frame.style.transform]).toEqual(['149.9707%', '100%', '-24.9854%', '0%', 'rotate(0deg)']);
  const image = box.querySelector<HTMLImageElement>('img[alt="Adjusted"]')!;
  expect(image.style.transform).toBe('rotate(0deg) scaleX(1)');
  expect(image).not.toHaveAttribute('data-swapped');
  expect(box.querySelector('.composed-split')).toBeNull();
});

it('C2(b) split stage has a middle hairline inside its output box', async () => {
  await openWorkspace([view([photo(1, 'main', { position: 1, composition: composition(1, { kind: 'split', rect: splitRect }) })])]);
  const box = document.querySelector<HTMLElement>('.stage-composed')!;
  expect(box.style.aspectRatio).toBe('2414.0626 / 1207.0313');
  const frame = box.querySelector<HTMLElement>('.composed-frame')!;
  expect([frame.style.width, frame.style.height, frame.style.left, frame.style.top]).toEqual(['106.0453%', '141.4214%', '-3.0227%', '-20.7107%']);
  expect(box.querySelector<HTMLElement>('.composed-split')!.style.left).toBe('50%');
});

it('C2(c) stage turns, flips, straightens and clips the adjusted image', async () => {
  const values = { quarter_turns: 1, flip: 'horizontal' as const, straighten: 5, rect: { x: 0, y: DEFAULT_RECT.x, w: 1, h: DEFAULT_RECT.w } };
  await openWorkspace([view([photo(1, 'main', { position: 1, composition: composition(1, values) })])]);
  const box = document.querySelector<HTMLElement>('.stage-composed')!;
  expect(box.style.aspectRatio).toBe('1707 / 1707');
  const frame = box.querySelector<HTMLElement>('.composed-frame')!;
  expect([frame.style.width, frame.style.height, frame.style.left, frame.style.top, frame.style.transform]).toEqual(['100%', '149.9707%', '0%', '-24.9854%', 'rotate(5deg)']);
  const image = box.querySelector<HTMLImageElement>('img[alt="Adjusted"]')!;
  expect(image.style.transform).toBe('rotate(90deg) scaleX(-1)');
  expect(image).toHaveAttribute('data-swapped', 'true');
});

it('C2(d) Original keeps framing and applies camera orientation; an unframed photo keeps the old stage', async () => {
  const framed = photo(1, 'main', { position: 1, orientation: 6, composition: composition(1, { frame: { w: 1707, h: 2560 }, rect: { x: 0, y: DEFAULT_RECT.x, w: 1, h: DEFAULT_RECT.w } }) });
  const unframed = photo(2, 'main', { position: 2, orientation: 6, composition: composition(2, { rect: null, frame: null }) });
  await openWorkspace([view([framed, unframed])]);
  fireEvent.keyDown(window, { key: ' ' });
  const original = document.querySelector<HTMLImageElement>('.stage-composed img[alt="Camera original"]')!;
  expect(original.style.transform).toBe('rotate(0deg) scaleX(1) rotate(90deg)');
  expect(original).toHaveAttribute('data-swapped', 'true');
  expect(original).toHaveAttribute('data-visible', 'true');
  const adjusted = document.querySelector<HTMLImageElement>('.stage-composed img[alt="Adjusted"]')!;
  expect(adjusted.style.transform).toBe('rotate(0deg) scaleX(1)');
  expect(adjusted).not.toHaveAttribute('data-swapped');
  await act(async () => { fireEvent.click(tile(2, document.querySelector('.photo-panel')!)); });
  expect(document.querySelector('.stage-composed')).toBeNull();
  expect(document.querySelector<HTMLImageElement>('.stage-original')!.style.transform).toBe('rotate(90deg)');
});

it('C3(a) grid tiles show composed thumbnails and mark crops', async () => {
  const full = photo(2, 'main', { position: 2, composition: composition(2, { shape: 'original', rect: { x: 0, y: 0, w: 1, h: 1 } }) });
  const split = photo(3, 'main', { position: 3, composition: composition(3, { kind: 'split', rect: splitRect }) });
  const unframed = photo(4, 'main', { position: 4, composition: composition(4, { rect: null, frame: null }) });
  await openWorkspace([view([photo(1, 'main', { position: 1 }), full, split, unframed])]);
  const panel = document.querySelector('.photo-panel')!;
  expect(tile(1, panel).querySelector<HTMLElement>('.tile-composed')!.style.aspectRatio).toBe('1707 / 1707');
  expect(tile(1, panel).querySelector<HTMLElement>('.composed-frame')!.style.left).toBe('-24.9854%');
  expect(tile(1, panel).querySelector<HTMLElement>('.composed-frame')!.style.width).toBe('149.9707%');
  expect(tile(1, panel).querySelector<HTMLImageElement>('img.thumb-image')!.src).toBe('asset:///previews/1/thumb-1.jpg');
  expect(tile(1, panel).querySelector('.tile-crop-mark')).not.toBeNull();
  expect(tile(2, panel).querySelector<HTMLElement>('.tile-composed')!.style.aspectRatio).toBe('2560 / 1707');
  expect(tile(2, panel).querySelector<HTMLElement>('.composed-frame')!.style.width).toBe('100%');
  expect(tile(2, panel).querySelector<HTMLElement>('.composed-frame')!.style.left).toBe('0%');
  expect(tile(2, panel).querySelector('.tile-crop-mark')).toBeNull();
  expect(tile(3, panel).querySelector('.composed-split')).not.toBeNull();
  expect(tile(3, panel).querySelector('.tile-crop-mark')).not.toBeNull();
  expect(tile(4, panel).querySelector('.tile-composed')).toBeNull();
  expect(tile(4, panel).querySelector<HTMLImageElement>('img.thumb-image')!.src).toBe('asset:///previews/4/thumb-1.jpg');
  expect(tile(4, panel).querySelector('.tile-crop-mark')).toBeNull();
  expect(screen.getByRole('region', { name: 'Main images' })).toBeTruthy();
});

it('C3(b) folded strip keeps the composed image but no grid crop badge', async () => {
  await openWorkspace([view([photo(1, 'main', { position: 1 })])]);
  fireEvent.keyDown(window, { key: 'f' });
  const strip = document.querySelector('.folded-strip')!;
  expect(tile(1, strip).querySelector<HTMLElement>('.tile-composed')!.style.aspectRatio).toBe('1707 / 1707');
  expect(tile(1, strip).querySelector('.tile-crop-mark')).toBeNull();
});
