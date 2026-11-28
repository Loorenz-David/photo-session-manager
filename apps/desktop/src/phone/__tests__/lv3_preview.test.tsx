// LV3·U4 — the captured-image preview (design README §4, §Preview gestures, §Preview actions; cards
// LV6, LV8): rows LV3·C20–C22. The fixture session (helpers): B (final 02, "Back") filled, A (01,
// "Front") current, C (03); one extra (Main position 04); one condition image (C1). LV8 (book 10J
// LV-D20 (4)) removed the Delete button: row C22(c) went with it (its outcome is LV8·C6(a)), and the
// hold of C21 starts after 200 ms (LV8 plan point P2). LV9 (book 10J LV-D21 (4)) removed the header, the
// resting pill, the hint over the photo and the button block: what the photo is is the Preview's name, its
// title, ratio and hold hint are read in the photo drawer (⋯), Back is the close icon (×) and Retake is in
// the drawer.
import { screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, B, chairSession, frameBytes, lift, type PhoneEnv, phoneEnv, press, slide, startPhone } from './helpers';

// Tile centres of LV6's dock layout (56 px tiles on a 64 px pitch).
const X = { b: 44, extra: 236, c1: 383 };
const dock = () => screen.getByTestId('dock');
const preview = () => within(screen.getByTestId('preview'));
const photo = () => screen.getByTestId('preview-photo');
const viewport = () => screen.getByTestId('preview-viewport');
const toast = () => screen.getByTestId('toast');
const named = () => screen.getByTestId('preview').getAttribute('aria-label');
const HOLD_HINT = 'Hold to view template · slide sideways for overlay';
/** The photo drawer, opened by ⋯. */
async function details(): Promise<ReturnType<typeof within>> {
  preview().getByRole('button', { name: 'Photo details' }).click();
  await advance(0);
  expect(screen.getByTestId('photo-drawer').getAttribute('aria-hidden')).toBe('false');
  return within(screen.getByTestId('photo-drawer'));
}
async function close(): Promise<void> {
  preview().getByRole('button', { name: 'Close' }).click();
  await advance(0);
}
const buzzed = (env: PhoneEnv, pattern: number[]) => env.vibrate.mock.calls.filter(call => JSON.stringify(call[0]) === JSON.stringify(pattern)).length;
async function open(x: number): Promise<void> {
  press(dock(), x, 730);
  lift(dock(), x, 730);
  await advance(0);
}

// ---- LV3·C20: what the photo is (the header until LV9) ----

test('lv3_c20a_a_template_shots_photo_is_titled_by_its_shot', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.b);
  expect(named()).toBe('SHOT 02 · CAPTURED');
  const drawer = await details();
  expect(drawer.getByText('SHOT 02 · CAPTURED')).toBeTruthy();
  expect(drawer.getByText('Back')).toBeTruthy();
  expect(screen.getByTestId('ratio-chip').textContent).toBe('1:1');
});

test('lv3_c20a_a_split_shots_chip_says_two_squares', async () => {
  const env = phoneEnv();
  await startPhone(env, chairSession(s => {
    s.shots[0].split = true;
    s.shots[0].ratio = 2;
    s.shots[0].photo = { photo_id: 201, revision: 1, ratio: 2, preview: 'k0' };
  }));
  await open(X.b);
  expect(screen.getByTestId('ratio-chip').textContent).toBe('SPLIT 2×1:1');
});

test('lv3_c20b_an_extra_and_a_condition_image_are_titled_by_their_place', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.extra);
  expect(named()).toBe('IMAGE 04 · EXTRA');
  expect((await details()).getByText('Extra image')).toBeTruthy();
  await close();
  await open(X.c1);
  expect(named()).toBe('CONDITION 01');
  expect((await details()).getByText('Condition image')).toBeTruthy();
});

// ---- LV3·C21: hold for the template, slide for the overlay ----

test('lv3_c21a_holding_a_template_shots_photo_shows_the_template_through_it', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.b);
  expect(photo().style.opacity).toBe('1');
  expect(within(screen.getByTestId('photo-drawer')).getByText(HOLD_HINT)).toBeTruthy();
  // The template lies under the photo.
  const layers = Array.from(viewport().querySelectorAll('img')).map(image => image.getAttribute('src'));
  expect(layers).toEqual([`/api/phone/v1/containers/${B}/example?pair=T`, '/api/phone/v1/photos/201/preview?pair=T&v=k0']);
  press(viewport(), 195, 305);
  // LV8 (plan point P2): the hold starts after 200 ms, so a swipe never flashes the template.
  await advance(200);
  expect(photo().style.opacity).toBe('0.35');
  expect(screen.getByTestId('preview-pill').textContent).toBe('TEMPLATE');
  lift(viewport(), 195, 305);
  expect(photo().style.opacity).toBe('1');
});

test('lv3_c21b_sliding_sets_the_overlay_the_main_screen_shares', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.b);
  press(viewport(), 100, 305);
  // LV8 (plan point P2): the hold starts after 200 ms.
  await advance(200);
  slide(viewport(), 120, 308);
  expect(within(viewport()).getByTestId('overlay-bar').textContent).toBe('PHOTO 42%');
  expect(photo().style.opacity).toBe('0.42');
  lift(viewport(), 120, 308);
  await close();
  // Back on the main screen: live over the template at the same 42 %.
  press(screen.getByTestId('viewport'), 195, 305);
  await env.socket().binary(frameBytes({ seq: 1, gen: env.socket().demands().at(-1)!.gen }));
  expect(screen.getByTestId('template-layer').style.opacity).toBe('0.42');
});

test('lv3_c21c_an_extra_has_no_template_to_hold_for', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.extra);
  expect(within(screen.getByTestId('photo-drawer')).queryByText(HOLD_HINT)).toBeNull();
  press(viewport(), 195, 305);
  slide(viewport(), 235, 305);
  expect(photo().style.opacity).toBe('1');
  expect(screen.queryByTestId('preview-pill')).toBeNull();
  expect(within(viewport()).queryByTestId('overlay-bar')).toBeNull();
});

// ---- LV3·C22: the actions ----

test('lv3_c22a_back_closes_the_preview_and_asks_nothing', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.b);
  await close();
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv3_c22b_retake_reopens_the_shot', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.b);
  (await details()).getByRole('button', { name: 'Retake' }).click();
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/retake', { session_id: 7, container_id: B }]]);
  expect(buzzed(env, [30, 40, 30])).toBe(1);
  expect(toast().textContent).toBe('Shot 02 reopened');
  expect(toast().style.opacity).toBe('1');
  expect(screen.queryByTestId('preview')).toBeNull();
});
