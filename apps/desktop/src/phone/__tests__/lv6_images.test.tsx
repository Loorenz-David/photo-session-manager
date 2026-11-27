// LV6·U2 — pictures that are never broken, and the removed text (master_plan_live §5 PH6a, PH7, §6 as
// amended; finding F7; book 10J LV-D19 (2); plan points P3, P4): rows LV6·C4–C6 and the core mechanism
// `previewUrl`. The fixture session (helpers), B's photo (201, tile "02") with revision 3.
import { fireEvent, screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { previewUrl } from '../api';
import { advance, chairSession, lift, macChanges, mountPhone, phoneEnv, press, startPhone } from './helpers';

const X = { b: 44, extra: 236 };
const Y = 742;
const REMOVED = 'This phone was removed in Session Manager. Delete this app from the home screen, then scan the new QR code in Session Manager → Settings → Phone and add it again.';
const dock = () => screen.getByTestId('dock');
const tileB = () => dock().querySelector('[aria-label="02 · Back"]') as HTMLElement;
/** B's photo as the Mac describes it: revision 3 and the given preview token. */
const withPreview = (preview: string | null) => chairSession(s => {
  s.shots[0].photo = { photo_id: 201, revision: 3, ratio: 1, preview };
});
/** The preview requests an element is making: the `src` of every picture in it that asks for a photo's preview. */
const previewRequests = (within_: Element) => Array.from(within_.querySelectorAll('img')).map(img => img.getAttribute('src')!).filter(src => src.includes('/preview'));
const waits = (within_: Element) => within_.querySelectorAll('[data-testid="photo-wait"]').length;
async function tap(x: number): Promise<void> {
  press(dock(), x, Y);
  lift(dock(), x, Y);
  await advance(0);
}

// ---- core mechanism: previewUrl ----

test('lv6_core_previewUrl_names_the_preview_version_and_the_retry_or_is_null_without_a_preview', () => {
  const image = { photo_id: 201, revision: 3, ratio: 1, preview: 'k1' };
  expect(previewUrl('T', image)).toBe('/api/phone/v1/photos/201/preview?pair=T&v=k1');
  expect(previewUrl('T', image, 2)).toBe('/api/phone/v1/photos/201/preview?pair=T&v=k1&try=2');
  expect(previewUrl('T', { ...image, preview: null })).toBeNull();
  expect(previewUrl('T', { ...image, preview: null }, 2)).toBeNull();
});

// ---- LV6·C4: a dock tile ----

test('lv6_c4a_a_photo_without_a_preview_shows_the_placeholder_and_asks_for_nothing', async () => {
  const env = phoneEnv();
  await startPhone(env, withPreview(null));
  expect(previewRequests(tileB())).toEqual([]);
  expect(tileB().querySelectorAll('img')).toHaveLength(0);
  expect(waits(tileB())).toBe(1);
  expect(tileB().querySelector('.ph-tag')!.textContent).toBe('02');
});

test('lv6_c4b_the_picture_is_asked_for_by_its_preview_version_and_again_when_it_changes', async () => {
  const env = phoneEnv();
  await startPhone(env, withPreview('k1'));
  expect(previewRequests(tileB())).toEqual(['/api/phone/v1/photos/201/preview?pair=T&v=k1']);
  expect(waits(tileB())).toBe(0);
  await macChanges(env, withPreview('k2'));
  expect(previewRequests(tileB())).toEqual(['/api/phone/v1/photos/201/preview?pair=T&v=k2']);
});

test('lv6_c4c_a_picture_that_fails_shows_the_placeholder_and_is_tried_three_more_times', async () => {
  const env = phoneEnv();
  await startPhone(env, withPreview('k1'));
  const base = '/api/phone/v1/photos/201/preview?pair=T&v=k1';
  for (const retry of [1, 2, 3]) {
    fireEvent.error(tileB().querySelector('img')!);
    expect(tileB().querySelectorAll('img')).toHaveLength(0);
    expect(waits(tileB())).toBe(1);
    await advance(1999);
    expect(previewRequests(tileB())).toEqual([]);
    await advance(1);
    expect(previewRequests(tileB())).toEqual([`${base}&try=${retry}`]);
    expect(waits(tileB())).toBe(0);
  }
  fireEvent.error(tileB().querySelector('img')!);
  await advance(10_000);
  expect(previewRequests(tileB())).toEqual([]);
  expect(waits(tileB())).toBe(1);
  expect(tileB().querySelector('.ph-tag')!.textContent).toBe('02');
  // A new preview version starts over.
  await macChanges(env, withPreview('k2'));
  expect(previewRequests(tileB())).toEqual(['/api/phone/v1/photos/201/preview?pair=T&v=k2']);
  expect(waits(tileB())).toBe(0);
});

// ---- LV6·C5: the Preview and the Move drawer ----

test('lv6_c5a_the_preview_says_preparing_until_the_photo_has_a_preview', async () => {
  const env = phoneEnv();
  await startPhone(env, withPreview(null));
  await tap(X.b);
  const preview = screen.getByTestId('preview');
  expect(within(preview).getByText('Preparing preview…')).toBeTruthy();
  // LV8: the Preview also holds the photo strip; the picture itself is the viewport's.
  expect(previewRequests(screen.getByTestId('preview-viewport'))).toEqual([]);
  expect(screen.queryByTestId('preview-photo')).toBeNull();
  await macChanges(env, withPreview('k1'));
  expect(screen.getByTestId('preview-photo').getAttribute('src')).toBe('/api/phone/v1/photos/201/preview?pair=T&v=k1');
  expect(within(preview).queryByText('Preparing preview…')).toBeNull();
});

test('lv6_c5b_the_move_drawer_shows_the_placeholder_for_a_photo_without_a_preview', async () => {
  const env = phoneEnv();
  await startPhone(env, withPreview(null));
  await tap(X.extra);
  within(screen.getByTestId('preview')).getByRole('button', { name: 'Move image' }).click();
  await advance(0);
  const tiles = Array.from(screen.getByTestId('move-main').children);
  const tile02 = tiles.find(tile => tile.querySelector('.ph-tag')?.textContent === '02')!;
  expect(tile02.querySelector('.ph-move-name')!.textContent).toBe('Back');
  expect(waits(tile02)).toBe(1);
  expect(previewRequests(tile02)).toEqual([]);
  expect(Array.from(document.querySelectorAll('img')).filter(img => img.getAttribute('src')!.includes('/photos/201/'))).toEqual([]);
});

// ---- LV6·C6: a removed phone ----

test('lv6_c6a_a_phone_removed_before_it_starts_is_told_to_delete_the_app_and_add_it_again', async () => {
  const env = phoneEnv({ storedToken: 'T' });
  env.mac.answer('GET /api/phone/v1/hello', 401, { error: { code: 'pairing_unknown', message: 'pairing_unknown' } });
  await mountPhone();
  expect(window.localStorage.getItem('psm.phone.token')).toBeNull();
  expect(screen.getByText(REMOVED)).toBeTruthy();
});

test('lv6_c6b_a_phone_removed_while_connected_is_told_the_same', async () => {
  const env = phoneEnv();
  await startPhone(env);
  expect(screen.queryByText(REMOVED)).toBeNull();
  await env.socket().drop(4401);
  expect(screen.getByText(REMOVED)).toBeTruthy();
  expect(window.localStorage.getItem('psm.phone.token')).toBeNull();
});
