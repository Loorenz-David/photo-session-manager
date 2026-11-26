// LV6·U1 — the shot dock without hold gestures (book 10J LV-D19 (1); plan points P1, P2; design README
// §Selecting a tile): rows LV6·C1–C3. The fixture session (helpers): shots in shooting order B (final 02,
// "Back", filled), A (01, "Front", current), C (03); one extra (04); one condition image. Tile centres, x:
// "02" 44 · "01" 108 · "03" 172 · "04" 236 · Free 300 · "C1" 383 (56 px tiles on a 64 px pitch); y 742.
// @ts-expect-error The frontend tsconfig omits Node types; Vitest runs this test in Node.
import { readFileSync } from 'node:fs';
import { screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, C, lift, phoneEnv, press, slide, startPhone } from './helpers';

const X = { b: 44, extra: 236, c1: 383 };
const Y = 742;
const dock = () => screen.getByTestId('dock');
const tiles = () => Array.from(dock().querySelectorAll<HTMLElement>('[data-dock-k]'));
const kicker = () => screen.queryByTestId('preview')?.querySelector('.ph-kicker')?.textContent ?? null;
const px = (value: string) => (value.endsWith('px') ? parseFloat(value) : null);

/** The phone's own stylesheet, as the phone build ships it (jsdom loads no CSS by itself). */
function withPhoneStyles(): void {
  const style = document.createElement('style');
  style.textContent = readFileSync('src/phone/phone.css', 'utf8');
  document.head.appendChild(style);
}

// ---- LV6·C1: sizes ----

test('lv6_c1a_tiles_are_56_px_with_12_px_corners_8_px_apart_in_an_84_px_strip', async () => {
  const env = phoneEnv();
  withPhoneStyles();
  try {
    await startPhone(env);
    const computed = (element: Element) => window.getComputedStyle(element);
    expect(tiles().map(tile => [computed(tile).width, computed(tile).height, computed(tile).borderRadius])).toEqual(Array(6).fill(['56px', '56px', '12px']));
    expect(computed(dock().querySelector('.ph-dock-row')!).gap).toBe('8px');
    expect(computed(dock()).height).toBe('84px');
    // Nothing in the dock is taller than the strip (the magnifier's tall scroller is gone).
    const heights = [dock(), ...Array.from(dock().querySelectorAll('*'))].map(element => px(computed(element).height)).filter(height => height !== null);
    expect(heights.length).toBeGreaterThan(8);
    expect(Math.max(...heights)).toBe(84);
  } finally {
    document.head.querySelectorAll('style').forEach(style => style.remove());
  }
});

// ---- LV6·C2: a single tap ----

test('lv6_c2a_a_tap_on_an_extra_opens_its_preview', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(dock(), X.extra, Y);
  lift(dock(), X.extra, Y);
  await advance(0);
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
  expect(env.vibrate.mock.calls).toEqual([[15]]);
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv6_c2b_a_tap_on_a_condition_image_opens_its_preview', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(dock(), X.c1, Y);
  lift(dock(), X.c1, Y);
  await advance(0);
  expect(kicker()).toBe('CONDITION 01');
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv6_c2c_a_long_press_is_a_tap_and_magnifies_nothing', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const sizes = () => tiles().map(tile => [tile.style.width, tile.style.height, tile.style.transform]);
  const before = sizes();
  press(dock(), X.b, Y);
  await advance(600);
  expect(sizes()).toEqual(before);
  expect(screen.queryByTestId('dock-label')).toBeNull();
  expect(dock().textContent).toBe('02010304C1');
  expect(screen.queryByTestId('preview')).toBeNull();
  lift(dock(), X.b, Y);
  await advance(0);
  expect(kicker()).toBe('SHOT 02 · CAPTURED');
  expect(screen.getByTestId('preview-photo').getAttribute('src')).toContain('/photos/201/preview');
  expect(env.vibrate.mock.calls).toEqual([[15]]);
});

test('lv6_c2d_a_touch_that_moves_up_selects_nothing', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(dock(), X.b, Y);
  await advance(300);
  slide(dock(), X.b, Y - 60);
  lift(dock(), X.b, Y - 60);
  await advance(500);
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(env.mac.allPosts()).toEqual([]);
  expect(env.vibrate.mock.calls).toEqual([]);
});

// ---- LV6·C3: scroll, then tap ----

test('lv6_c3a_a_sideways_swipe_scrolls_and_the_next_tap_selects_the_tile_now_under_the_finger', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const scroller = dock().querySelector('.ph-dock-scroller') as HTMLElement;
  const start = scroller.scrollLeft;
  press(dock(), X.b, Y);
  slide(dock(), X.b - 128, Y);
  lift(dock(), X.b - 128, Y);
  await advance(500);
  expect(scroller.scrollLeft - start).toBe(128);
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(env.mac.allPosts()).toEqual([]);
  expect(env.vibrate.mock.calls).toEqual([]);
  // "02" was under the finger; two pitches to its right is "03" (empty): it becomes the current shot.
  press(dock(), X.b, Y);
  lift(dock(), X.b, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/current', { session_id: 7, container_id: C }]]);
  expect(screen.queryByTestId('preview')).toBeNull();
});
