// LV10·U2 — pinch to zoom in the Preview, and photos shown as composed (book 10J LV-D21 (5); 10D §The result is
// shown; master_plan_live §5 PH6c, PH11; plan points P1–P3): rows LV10·C5–C8.
// C5–C7 on fixture S (helpers `sortSession`: shot B's photo 201, extra 202, Condition 203 and 207; none composed);
// C8 on fixture K45 (201 framed 4:5 on its 2560 × 1707 full frame). Tile centres x: "02" 44 · "04" 236 · "C1" 383;
// y 742. The Preview's picture of a 1:1 photo is 390 × 390 in a 390 × 610 stage: centre (195, 305). Two fingers
// are two pointer ids (1 and 2) on the viewport.
import { fireEvent, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { composedLayout } from '@/features/compose/composed';
import { advance, cancel, composed45Session, K45_VALUES, K_FRAME, lift, type PhoneEnv, phoneEnv, press, slide, sortSession, startPhone } from './helpers';

const X = { b: 44, extra: 236, c1: 383 };
const Y = 742;
const dock = () => screen.getByTestId('dock');
const strip = () => screen.getByTestId('preview-strip');
const viewport = () => screen.getByTestId('preview-viewport');
const zoom = () => screen.getByTestId('preview-zoom').style.transform;
const photo = () => screen.getByTestId('preview-photo');
const hint = () => screen.queryByTestId('swipe-hint');

function tap(target: Element, x: number, y = Y): void {
  press(target, x, y);
  lift(target, x, y);
}
/** A paired phone with the Preview of the photo under `x` open; what happened before is forgotten. */
async function openPreview(x: number, session = sortSession()): Promise<PhoneEnv> {
  const env = phoneEnv();
  await startPhone(env, session);
  tap(dock(), x);
  await advance(0);
  env.vibrate.mockClear();
  env.mac.requests.length = 0;
  return env;
}
/** Two fingers come down at `from` (x of each, on the picture's middle line) and move to `to`; they stay down. */
function fingersTo(from: [number, number], to: [number, number]): void {
  press(viewport(), from[0], 305, 1);
  press(viewport(), from[1], 305, 2);
  slide(viewport(), to[0], 305, 1);
  slide(viewport(), to[1], 305, 2);
}
function release(at: [number, number]): void {
  lift(viewport(), at[1], 305, 2);
  lift(viewport(), at[0], 305, 1);
}
/** "Spread ×2" of the plan: fingers 100 px apart about the centre, moved to 200 px apart, then lifted. */
function spreadTwice(): void {
  fingersTo([145, 245], [95, 295]);
  release([95, 295]);
}
/** The Preview of the photo under `x`, zoomed 2× about its centre. */
async function zoomedPreview(x: number): Promise<PhoneEnv> {
  const env = await openPreview(x);
  spreadTwice();
  expect(zoom()).toBe('translate(0px, 0px) scale(2)');
  return env;
}

// ---- LV10·C5: two fingers zoom ----

test('lv10_c5a_two_fingers_spread_zoom_the_picture_about_the_point_between_them', async () => {
  const env = await openPreview(X.extra);
  expect(zoom()).toBe('translate(0px, 0px) scale(1)');
  fingersTo([145, 245], [95, 295]);
  expect(zoom()).toBe('translate(0px, 0px) scale(2)');
  release([95, 295]);
  expect(zoom()).toBe('translate(0px, 0px) scale(2)');
  await advance(200);
  expect(zoom()).toBe('translate(0px, 0px) scale(2)');
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv10_c5b_below_1x_it_stretches_a_little_and_springs_back', async () => {
  await openPreview(X.extra);
  fingersTo([95, 295], [145, 245]);
  // Half the distance would be 0.5×: it stops at the stretch limit.
  expect(zoom()).toBe('translate(0px, 0px) scale(0.8)');
  release([145, 245]);
  await advance(200);
  expect(zoom()).toBe('translate(0px, 0px) scale(1)');
});

test('lv10_c5c_above_6x_it_settles_at_6x', async () => {
  await openPreview(X.extra);
  spreadTwice();
  spreadTwice();
  fingersTo([145, 245], [95, 295]);
  // 2 · 2 · 2 = 8×: held at the stretch limit while the fingers are down.
  expect(zoom()).toBe('translate(0px, 0px) scale(7)');
  release([95, 295]);
  await advance(200);
  expect(zoom()).toBe('translate(0px, 0px) scale(6)');
});

// ---- LV10·C6: a zoomed picture ----

test('lv10_c6a_one_finger_moves_a_zoomed_picture_and_never_swipes', async () => {
  const env = await zoomedPreview(X.extra);
  press(viewport(), 195, 305);
  slide(viewport(), 95, 305);
  expect(hint()).toBeNull();
  lift(viewport(), 95, 305);
  await advance(200);
  // 100 px sideways would have been a counted swipe (96 px) at 1×.
  expect(zoom()).toBe('translate(-100px, 0px) scale(2)');
  expect(hint()).toBeNull();
  expect(env.mac.allPosts()).toEqual([]);
  expect(photo().getAttribute('src')).toContain('/photos/202/preview');
});

test('lv10_c6b_the_picture_never_moves_past_its_edge', async () => {
  await zoomedPreview(X.extra);
  press(viewport(), 195, 305);
  slide(viewport(), -300, 305);
  // (2 − 1) · 390 / 2 = 195: the picture's right edge is at the viewport's.
  expect(zoom()).toBe('translate(-195px, 0px) scale(2)');
  lift(viewport(), -300, 305);
  await advance(200);
  expect(zoom()).toBe('translate(-195px, 0px) scale(2)');
});

test('lv10_c6c_a_zoomed_template_photo_is_not_held_for_its_template', async () => {
  const env = await zoomedPreview(X.b);
  press(viewport(), 195, 305);
  await advance(300);
  expect(screen.queryByTestId('preview-pill')).toBeNull();
  lift(viewport(), 195, 305);
  expect(screen.queryByTestId('preview-pill')).toBeNull();
  expect(env.vibrate.mock.calls.filter(call => call[0] === 10)).toEqual([]);
});

test('lv10_c6d_a_double_tap_zooms_to_3x_about_the_tap_and_back_to_1x', async () => {
  await openPreview(X.extra);
  tap(viewport(), 100, 305);
  await advance(150);
  tap(viewport(), 100, 305);
  // The tapped point (95 px left of the centre) stays under the finger: −95 − (−95 · 3) = 190.
  expect(zoom()).toBe('translate(190px, 0px) scale(3)');
  await advance(1000);
  tap(viewport(), 300, 400);
  await advance(150);
  tap(viewport(), 300, 400);
  await advance(200);
  expect(zoom()).toBe('translate(0px, 0px) scale(1)');
});

// ---- LV10·C7: what ends a gesture ----

test('lv10_c7a_a_second_finger_ends_a_swipe_and_starts_a_pinch', async () => {
  const env = await openPreview(X.extra);
  press(viewport(), 195, 305, 1);
  slide(viewport(), 165, 305, 1);
  expect(hint()).not.toBeNull();
  expect(photo().style.transform).toContain('translateX(-30px)');
  press(viewport(), 300, 305, 2);
  expect(hint()).toBeNull();
  expect(photo().style.transform).toBe('');
  slide(viewport(), 360, 305, 2);
  // The fingers were 135 px apart when the second came down (165 and 300) and are 195 px apart now.
  expect(zoom()).toContain(`scale(${Math.round((195 / 135) * 10000) / 10000})`);
  expect(zoom()).toContain('scale(1.4444)');
  lift(viewport(), 360, 305, 2);
  lift(viewport(), 165, 305, 1);
  await advance(400);
  expect(env.mac.allPosts()).toEqual([]);
  expect(photo().getAttribute('src')).toContain('/photos/202/preview');
});

test('lv10_c7b_a_cancelled_finger_or_a_blur_ends_the_pinch_where_it_is', async () => {
  const env = await openPreview(X.extra);
  fingersTo([145, 245], [95, 295]);
  cancel(viewport(), 95, 305, 1);
  await advance(200);
  expect(zoom()).toBe('translate(0px, 0px) scale(2)');
  // The finger still down moves and lifts: nothing follows it.
  slide(viewport(), 20, 305, 2);
  lift(viewport(), 20, 305, 2);
  await advance(400);
  expect(zoom()).toBe('translate(0px, 0px) scale(2)');
  expect(hint()).toBeNull();
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv10_c7b_a_window_blur_ends_the_pinch_where_it_is', async () => {
  const env = await openPreview(X.extra);
  fingersTo([145, 245], [95, 295]);
  fireEvent.blur(window);
  await advance(200);
  expect(zoom()).toBe('translate(0px, 0px) scale(2)');
  slide(viewport(), 20, 305, 2);
  release([95, 20]);
  await advance(400);
  expect(zoom()).toBe('translate(0px, 0px) scale(2)');
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv10_c7c_another_photo_shows_at_1x', async () => {
  await zoomedPreview(X.extra);
  tap(strip(), X.c1);
  await advance(0);
  expect(photo().getAttribute('src')).toContain('/photos/203/preview');
  expect(zoom()).toBe('translate(0px, 0px) scale(1)');
});

// ---- LV10·C8: photos shown as composed ----

/** The inline box of a composed picture's whole frame, and its straighten turn. */
const frameStyle = (frame: HTMLElement) => ({ width: frame.style.width, height: frame.style.height, left: frame.style.left, top: frame.style.top, transform: frame.style.transform });
const K45_LAYOUT = composedLayout(K45_VALUES, K_FRAME);

test('lv10_c8a_the_preview_shows_a_composed_photo_as_composed', async () => {
  await openPreview(X.b, composed45Session());
  // The viewport has the framing's ratio: 4:5.
  expect(parseFloat(viewport().style.width)).toBeCloseTo(390, 6);
  expect(parseFloat(viewport().style.height)).toBeCloseTo(487.5, 6);
  const frame = viewport().querySelector<HTMLElement>('[data-testid="composed-frame"]')!;
  // The whole frame is 1 / 0.5334375 of the framed width, moved left by 0.2 / 0.5334375 of it.
  expect(frame.style.width).toBe('187.4634%');
  expect(frameStyle(frame)).toEqual({ ...K45_LAYOUT.frame, transform: 'rotate(0deg)' });
  expect(frameStyle(frame)).toEqual({ width: '187.4634%', height: '100%', left: '-37.4927%', top: '0%', transform: 'rotate(0deg)' });
  expect(frame.querySelector('img')!.getAttribute('src')).toContain('/photos/201/preview');
});

test('lv10_c8b_a_strip_tile_shows_a_composed_photo_as_composed', async () => {
  const env = phoneEnv();
  await startPhone(env, composed45Session());
  const tile = dock().querySelector<HTMLElement>('[data-dock-k="p201"]')!;
  const frame = tile.querySelector<HTMLElement>('[data-testid="composed-frame"]')!;
  expect(frameStyle(frame)).toEqual({ ...K45_LAYOUT.frame, transform: 'rotate(0deg)' });
  expect(frame.querySelector('img')!.getAttribute('src')).toContain('/photos/201/preview');
});

test('lv10_c8c_a_photo_without_a_composition_is_drawn_as_before', async () => {
  await openPreview(X.extra, composed45Session());
  // The Preview of 202: the picture itself, covering the viewport.
  expect(viewport().querySelector('[data-testid="composed-frame"]')).toBeNull();
  expect(photo().tagName).toBe('IMG');
  expect(photo().className).toBe('ph-cover ph-preview-photo');
  expect(photo().getAttribute('src')).toContain('/photos/202/preview');
  expect(parseFloat(viewport().style.height)).toBe(390);
  // Its tile in the live screen's strip: the picture itself.
  const tile = dock().querySelector<HTMLElement>('[data-dock-k="p202"]')!;
  expect(tile.querySelector('[data-testid="composed-frame"]')).toBeNull();
  expect(tile.querySelector('img')!.className).toBe('ph-dock-image');
});
