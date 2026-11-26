// LV3·U4 — the captured-image preview (design README §4, §Preview gestures, §Preview actions; cards
// LV6, LV8): rows LV3·C20–C22. The fixture session (helpers): B (final 02, "Back") filled, A (01,
// "Front") current, C (03); one extra (Main position 04); one condition image (C1).
import { screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, B, chairSession, frameBytes, lift, type PhoneEnv, PHOTO_C1, phoneEnv, press, slide, startPhone } from './helpers';

// Tile centres of LV6's dock layout (56 px tiles on a 64 px pitch).
const X = { b: 44, extra: 236, c1: 383 };
const dock = () => screen.getByTestId('dock');
const preview = () => within(screen.getByTestId('preview'));
const photo = () => screen.getByTestId('preview-photo');
const viewport = () => screen.getByTestId('preview-viewport');
const toast = () => screen.getByTestId('toast');
const buzzed = (env: PhoneEnv, pattern: number[]) => env.vibrate.mock.calls.filter(call => JSON.stringify(call[0]) === JSON.stringify(pattern)).length;
async function open(x: number): Promise<void> {
  press(dock(), x, 730);
  lift(dock(), x, 730);
  await advance(0);
}

// ---- LV3·C20: the header ----

test('lv3_c20a_a_template_shots_photo_is_titled_by_its_shot', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.b);
  expect(preview().getByText('SHOT 02 · CAPTURED')).toBeTruthy();
  expect(preview().getByText('Back')).toBeTruthy();
  expect(screen.getByTestId('ratio-chip').textContent).toBe('1:1');
  expect(screen.getByTestId('preview-pill').textContent).toBe('CAPTURED');
});

test('lv3_c20a_a_split_shots_chip_says_two_squares', async () => {
  const env = phoneEnv();
  await startPhone(env, chairSession(s => {
    s.shots[0].split = true;
    s.shots[0].ratio = 2;
    s.shots[0].photo = { photo_id: 201, revision: 1, ratio: 2 };
  }));
  await open(X.b);
  expect(screen.getByTestId('ratio-chip').textContent).toBe('SPLIT 2×1:1');
});

test('lv3_c20b_an_extra_and_a_condition_image_are_titled_by_their_place', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.extra);
  expect(preview().getByText('IMAGE 04 · EXTRA')).toBeTruthy();
  expect(preview().getByText('Extra image')).toBeTruthy();
  preview().getByRole('button', { name: 'Back to shot 01' }).click();
  await advance(0);
  await open(X.c1);
  expect(preview().getByText('CONDITION 01')).toBeTruthy();
  expect(preview().getByText('Condition image')).toBeTruthy();
});

// ---- LV3·C21: hold for the template, slide for the overlay ----

test('lv3_c21a_holding_a_template_shots_photo_shows_the_template_through_it', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.b);
  expect(photo().style.opacity).toBe('1');
  expect(preview().getByText('Hold to view template · slide sideways for overlay')).toBeTruthy();
  // The template lies under the photo.
  const layers = Array.from(viewport().querySelectorAll('img')).map(image => image.getAttribute('src'));
  expect(layers).toEqual([`/api/phone/v1/containers/${B}/example?pair=T`, '/api/phone/v1/photos/201/preview?pair=T&r=1']);
  press(viewport(), 195, 305);
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
  slide(viewport(), 120, 308);
  expect(within(viewport()).getByTestId('overlay-bar').textContent).toBe('PHOTO 42%');
  expect(photo().style.opacity).toBe('0.42');
  lift(viewport(), 120, 308);
  preview().getByRole('button', { name: 'Back to shot 01' }).click();
  await advance(0);
  // Back on the main screen: live over the template at the same 42 %.
  press(screen.getByTestId('viewport'), 195, 305);
  await env.socket().binary(frameBytes({ seq: 1, gen: env.socket().demands().at(-1)!.gen }));
  expect(screen.getByTestId('template-layer').style.opacity).toBe('0.42');
});

test('lv3_c21c_an_extra_has_no_template_to_hold_for', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.extra);
  expect(preview().queryByText('Hold to view template · slide sideways for overlay')).toBeNull();
  press(viewport(), 195, 305);
  slide(viewport(), 235, 305);
  expect(photo().style.opacity).toBe('1');
  expect(screen.getByTestId('preview-pill').textContent).toBe('EXTRA');
  expect(within(viewport()).queryByTestId('overlay-bar')).toBeNull();
});

// ---- LV3·C22: the actions ----

test('lv3_c22a_back_closes_the_preview_and_asks_nothing', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.b);
  preview().getByRole('button', { name: 'Back to shot 01' }).click();
  await advance(0);
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv3_c22b_retake_reopens_the_shot', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.b);
  preview().getByRole('button', { name: 'Retake' }).click();
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/retake', { session_id: 7, container_id: B }]]);
  expect(buzzed(env, [30, 40, 30])).toBe(1);
  expect(toast().textContent).toBe('Shot 02 reopened');
  expect(toast().style.opacity).toBe('1');
  expect(screen.queryByTestId('preview')).toBeNull();
});

test('lv3_c22c_delete_denies_a_condition_image', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await open(X.c1);
  expect(preview().queryByRole('button', { name: 'Retake' })).toBeNull();
  preview().getByRole('button', { name: 'Delete' }).click();
  await advance(0);
  expect(env.mac.allPosts()).toEqual([[`/photos/${PHOTO_C1}/delete`, { session_id: 7 }]]);
  expect(toast().textContent).toBe('Image deleted');
  expect(toast().style.opacity).toBe('1');
  expect(screen.queryByTestId('preview')).toBeNull();
});
