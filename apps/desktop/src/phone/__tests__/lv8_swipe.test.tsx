// LV8·U2 — the Preview: swipe to classify, the confirmation, the strip (book 10J LV-D20 (1), (2), (4);
// master_plan_live §5 PH9.1, §6 texts v1.2; plan points P2–P6, P11, P12): rows LV8·C5–C8.
// Fixture S (helpers `sortSession`): shots B (final 02, photo 201), A (01, empty, current), C (03, empty);
// extra 202 (Main 04); condition 203 (C1), 207 (C2). The strip reads "02 01 03 04 · C1 C2"; tile centres x:
// "02" 44 · "01" 108 · "03" 172 · "04" 236 · Free 300 · "C1" 383 · "C2" 447; y 742. The Preview's picture is
// 390 × 390 in a 390 × 610 stage: centre (195, 305).
import { fireEvent, screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, B, C, cancel, lift, type PhoneEnv, phoneEnv, press, slide, sortSession, startPhone } from './helpers';

const X = { b: 44, c: 172, extra: 236, c1: 383, c2: 447 };
const Y = 742;
const CONFIRM = 'Move this photo to Main, outside its template order?';
const dock = () => screen.getByTestId('dock');
const strip = () => screen.getByTestId('preview-strip');
const preview = () => within(screen.getByTestId('preview'));
const viewport = () => screen.getByTestId('preview-viewport');
const photo = () => screen.getByTestId('preview-photo');
// LV9 (book 10J LV-D21 (4)): the Preview has no header; what it shows is its name.
const kicker = () => screen.queryByTestId('preview')?.getAttribute('aria-label') ?? null;
const hint = () => screen.queryByTestId('swipe-hint')?.textContent ?? null;
const toast = () => screen.getByTestId('toast').textContent;
const dialog = () => screen.queryByRole('dialog', { name: CONFIRM });
const buzzed = (env: PhoneEnv, pattern: number | number[]) => env.vibrate.mock.calls.filter(call => JSON.stringify(call[0]) === JSON.stringify(pattern)).length;
/** The labels of the tiles a strip outlines. */
const outlined = (within_: HTMLElement) => Array.from(within_.querySelectorAll<HTMLElement>('[data-dock-k]'))
  .filter(tile => !tile.style.outline.includes('transparent')).map(tile => tile.querySelector('.ph-tag')?.textContent ?? 'free');

function tap(target: Element, x: number): void {
  press(target, x, Y);
  lift(target, x, Y);
}
/** A paired phone on fixture S with the Preview of the photo under `x` open; what happened before is forgotten. */
async function openPreview(x: number, session = sortSession()): Promise<PhoneEnv> {
  const env = phoneEnv();
  await startPhone(env, session);
  tap(dock(), x);
  await advance(0);
  env.vibrate.mockClear();
  env.mac.requests.length = 0;
  return env;
}
/** One finger from the picture's centre to `(dx, dy)` away and off, with no time passing. */
function swipe(dx: number, dy: number): void {
  press(viewport(), 195, 305);
  slide(viewport(), 195 + dx, 305 + dy);
  lift(viewport(), 195 + dx, 305 + dy);
}

// ---- LV8·C5: swipe or hold; what does not count ----

test('lv8_c5a_a_move_within_200_ms_is_a_swipe_and_never_shows_the_template', async () => {
  const env = await openPreview(X.b);
  press(viewport(), 195, 305);
  await advance(199);
  slide(viewport(), 165, 305);
  // LV9 (book 10J LV-D21 (4)): no pill rests on the photo; "TEMPLATE" shows only while its template does.
  expect(screen.queryByTestId('preview-pill')).toBeNull();
  expect(photo().style.opacity).toBe('1');
  expect(photo().style.transform).toContain('translateX(-30px)');
  expect(hint()).toBe('DENY');
  slide(viewport(), 190, 305);
  lift(viewport(), 190, 305);
  await advance(500);
  expect(env.mac.allPosts()).toEqual([]);
  expect(photo().style.transform).not.toContain('translate');
  expect(kicker()).toBe('SHOT 02 · CAPTURED');
  expect(env.vibrate).not.toHaveBeenCalled();
});

test('lv8_c5b_a_template_shots_photo_held_still_for_200_ms_shows_its_template_and_does_not_swipe', async () => {
  const env = await openPreview(X.b);
  press(viewport(), 100, 305);
  await advance(199);
  expect(screen.queryByTestId('preview-pill')).toBeNull();
  await advance(1);
  expect(screen.getByTestId('preview-pill').textContent).toBe('TEMPLATE');
  expect(photo().style.opacity).toBe('0.35');
  expect(env.vibrate.mock.calls).toEqual([[10]]);
  slide(viewport(), 120, 308);
  expect(within(viewport()).getByTestId('overlay-bar').textContent).toBe('PHOTO 42%');
  expect(hint()).toBeNull();
  expect(photo().style.transform).not.toContain('translate');
  lift(viewport(), 120, 308);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv8_c5c_a_swipe_released_before_96_px_springs_back', async () => {
  const env = await openPreview(X.extra);
  press(viewport(), 195, 305);
  slide(viewport(), 100, 305);
  expect(photo().style.transform).toContain('translateX(-95px)');
  lift(viewport(), 100, 305);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(photo().style.transform).toBe('');
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
  expect(toast()).toBe('');
});

test('lv8_c5d_downwards_does_nothing', async () => {
  const env = await openPreview(X.extra);
  press(viewport(), 195, 305);
  slide(viewport(), 195, 425);
  expect(hint()).toBeNull();
  expect(photo().style.transform).not.toContain('translate');
  lift(viewport(), 195, 425);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
});

test('lv8_c5e_a_swipe_the_phone_interrupts_does_nothing', async () => {
  const env = await openPreview(X.extra);
  // (1) The phone takes the touch away.
  press(viewport(), 195, 305);
  slide(viewport(), 75, 305);
  expect(photo().style.transform).toContain('translateX(-120px)');
  cancel(viewport(), 75, 305);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(photo().style.transform).toBe('');
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
  // (2) The page is hidden before the finger leaves.
  press(viewport(), 195, 305);
  slide(viewport(), 75, 305);
  await env.setVisibility('hidden');
  expect(photo().style.transform).toBe('');
  lift(viewport(), 75, 305);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
});

test('lv8_c5f_on_a_long_shot_the_swipe_follows_the_picture', async () => {
  // B is 16:9: its Preview turns, so up on the screen is the picture's left.
  const env = await openPreview(X.b, sortSession(s => {
    s.shots[0].ratio = 1.7778;
    s.shots[0].photo = { photo_id: 201, revision: 1, ratio: 1.7778, preview: 'k0' };
  }));
  expect(viewport().style.transform).toBe('rotate(90deg)');
  swipe(0, -120);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/retake', { session_id: 7, container_id: B }]]);
});

// ---- LV8·C6: what each swipe asks the Mac, and the next photo ----

test('lv8_c6a_left_on_a_condition_image_denies_it_and_shows_the_next', async () => {
  const env = await openPreview(X.c1);
  expect(kicker()).toBe('CONDITION 01');
  swipe(-120, 0);
  // At once: before the Mac has answered anything.
  expect(kicker()).toBe('CONDITION 02');
  expect(photo().getAttribute('src')).toContain('/photos/207/preview');
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/photos/203/delete', { session_id: 7 }]]);
  expect(buzzed(env, [30, 40, 30])).toBe(1);
  expect(toast()).toBe('Image deleted');
  const read = env.mac.requests.map(r => `${r.method} ${r.path}`);
  expect(read.indexOf('GET /api/phone/v1/session')).toBeGreaterThan(read.indexOf('POST /api/phone/v1/photos/203/delete'));
});

test('lv8_c6b_left_on_a_template_shots_photo_reopens_its_shot', async () => {
  const env = await openPreview(X.b);
  swipe(-120, 0);
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/retake', { session_id: 7, container_id: B }]]);
  expect(toast()).toBe('Shot 02 reopened');
});

test('lv8_c6c_right_on_the_last_condition_image_makes_it_main_and_closes_the_preview', async () => {
  const env = await openPreview(X.c2);
  expect(kicker()).toBe('CONDITION 02');
  swipe(120, 0);
  expect(screen.queryByTestId('preview')).toBeNull();
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/photos/207/classify', { session_id: 7, to: 'main' }]]);
  expect(buzzed(env, [30, 40, 60])).toBe(1);
  expect(toast()).toBe('Moved to main images');
  expect(screen.queryByTestId('preview')).toBeNull();
});

test('lv8_c6d_up_on_an_extra_sends_it_to_condition', async () => {
  const env = await openPreview(X.extra);
  swipe(0, -120);
  expect(kicker()).toBe('CONDITION 01');
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/photos/202/classify', { session_id: 7, to: 'condition' }]]);
  expect(toast()).toBe('Sent to condition images');
});

test('lv8_c6e_up_on_a_template_shots_photo_sends_it_to_condition', async () => {
  const env = await openPreview(X.b);
  swipe(0, -120);
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/photos/201/classify', { session_id: 7, to: 'condition' }]]);
});

test('lv8_c6f_a_photo_already_in_the_group_springs_back_with_a_hint', async () => {
  const env = await openPreview(X.extra);
  press(viewport(), 195, 305);
  slide(viewport(), 315, 305);
  expect(hint()).toBe('ALREADY MAIN');
  lift(viewport(), 315, 305);
  await advance(0);
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
  expect(photo().style.transform).toBe('');
  // LV9 (book 10J LV-D21 (4)): Back is the close icon.
  preview().getByRole('button', { name: 'Close' }).click();
  await advance(0);
  tap(dock(), X.c1);
  await advance(0);
  env.vibrate.mockClear();
  press(viewport(), 195, 305);
  slide(viewport(), 195, 185);
  expect(hint()).toBe('ALREADY CONDITION');
  lift(viewport(), 195, 185);
  await advance(0);
  expect(kicker()).toBe('CONDITION 01');
  expect(env.mac.allPosts()).toEqual([]);
  expect(toast()).toBe('');
  expect(env.vibrate).not.toHaveBeenCalled();
});

test('lv8_c6g_a_swipe_the_mac_refuses_says_why_and_does_not_go_back', async () => {
  const env = await openPreview(X.c1);
  env.mac.answer('POST /api/phone/v1/photos/203/delete', 409, { error: { code: 'session_changed', message: 'session_changed' } });
  swipe(-120, 0);
  await advance(0);
  expect(toast()).toBe('Session changed on the Mac');
  const read = env.mac.requests.map(r => `${r.method} ${r.path}`);
  expect(read.indexOf('GET /api/phone/v1/session')).toBeGreaterThan(read.indexOf('POST /api/phone/v1/photos/203/delete'));
  expect(kicker()).toBe('CONDITION 02');
});

// ---- LV8·C7: a template shot's photo to Main asks first ----

test('lv8_c7a_right_on_a_template_shots_photo_asks_before_anything_happens', async () => {
  const env = await openPreview(X.b);
  swipe(120, 0);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(dialog()).not.toBeNull();
  expect(within(dialog()!).getByText(CONFIRM)).toBeTruthy();
  expect(within(dialog()!).getAllByRole('button').map(button => button.textContent).sort()).toEqual(['Cancel', 'Move']);
  expect(photo().style.transform).toBe('');
  expect(kicker()).toBe('SHOT 02 · CAPTURED');
});

test('lv8_c7b_move_makes_it_a_main_photo_and_shows_the_next', async () => {
  const env = await openPreview(X.b);
  swipe(120, 0);
  await advance(0);
  fireEvent.click(within(dialog()!).getByRole('button', { name: 'Move' }));
  // At once: before the Mac has answered.
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
  expect(env.mac.allPosts()).toEqual([['/photos/201/classify', { session_id: 7, to: 'main' }]]);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/photos/201/classify', { session_id: 7, to: 'main' }]]);
  expect(buzzed(env, [30, 40, 60])).toBe(1);
  expect(toast()).toBe('Moved to main images');
  expect(dialog()).toBeNull();
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
});

test('lv8_c7c_cancel_and_the_backdrop_keep_the_photo_in_its_shot', async () => {
  const env = await openPreview(X.b);
  swipe(120, 0);
  await advance(0);
  within(dialog()!).getByRole('button', { name: 'Cancel' }).click();
  await advance(0);
  expect(dialog()).toBeNull();
  expect(kicker()).toBe('SHOT 02 · CAPTURED');
  // Separately: a tap beside the sheet.
  swipe(120, 0);
  await advance(0);
  expect(dialog()).not.toBeNull();
  screen.getByTestId('confirm-backdrop').click();
  await advance(0);
  expect(dialog()).toBeNull();
  expect(kicker()).toBe('SHOT 02 · CAPTURED');
  expect(env.mac.allPosts()).toEqual([]);
});

// ---- LV8·C8: the strip in the Preview (row C8(e), the buttons, is superseded by LV9·C11(a) and C12(a)) ----

test('lv8_c8a_the_preview_shows_the_strip_and_marks_its_photo', async () => {
  await openPreview(X.b);
  expect(strip().textContent).toBe('02010304C1C2');
  expect(strip().textContent).toBe(dock().textContent);
  expect(outlined(strip())).toEqual(['02']);
  // The live screen's strip still marks the current shot.
  expect(outlined(dock())).toEqual(['01']);
});

test('lv8_c8b_a_tap_on_a_photo_in_the_strip_shows_it', async () => {
  const env = await openPreview(X.b);
  tap(strip(), X.extra);
  await advance(0);
  expect(kicker()).toBe('IMAGE 04 · EXTRA');
  expect(outlined(strip())).toEqual(['04']);
  tap(strip(), X.c1);
  await advance(0);
  expect(kicker()).toBe('CONDITION 01');
  expect(outlined(strip())).toEqual(['C1']);
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv8_c8c_a_tap_on_an_empty_shot_in_the_strip_makes_it_current_and_closes_the_preview', async () => {
  const env = await openPreview(X.b);
  tap(strip(), X.c);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/current', { session_id: 7, container_id: C }]]);
  expect(screen.queryByTestId('preview')).toBeNull();
});

test('lv8_c8d_the_strip_marks_the_next_photo_after_a_swipe', async () => {
  await openPreview(X.b);
  swipe(-120, 0);
  expect(outlined(strip())).toEqual(['04']);
});

test('lv8_c8f_the_live_screens_strip_takes_no_touch_under_a_preview', async () => {
  const env = await openPreview(X.b);
  tap(dock(), X.c1);
  await advance(0);
  expect(kicker()).toBe('SHOT 02 · CAPTURED');
  tap(dock(), X.c);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(env.vibrate).not.toHaveBeenCalled();
});
