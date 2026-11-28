// LV9·U4 — the Preview is only the photo, its strip, ⋯ and × (book 10J LV-D21 (4); master_plan_live §5 PH10,
// §6 texts v1.3; plan points P6, P7): rows LV9·C11–C13.
// Fixture S (helpers `sortSession`): shots B (final 02, "Back", photo 201), A (01, empty, current), C (03,
// empty); extra 202 (Main 04); Condition 203 (C1), 207 (C2). Tile centres x: "02" 44 · "01" 108 · "03" 172 ·
// "04" 236 · Free 300 · "C1" 383 · "C2" 447; y 742. The Preview's picture is 390 × 390 in a 390 × 610 stage:
// centre (195, 305). The Preview is opened by a tap on the live screen's strip.
// @ts-expect-error The frontend tsconfig omits Node types; Vitest runs this test in Node.
import { existsSync } from 'node:fs';
import { screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, lift, macChanges, type PhoneEnv, phoneEnv, press, slide, sortSession, startPhone } from './helpers';

const X = { b: 44, extra: 236, c1: 383, c2: 447 };
const Y = 742;
const HOLD_HINT = 'Hold to view template · slide sideways for overlay';
const dock = () => screen.getByTestId('dock');
const strip = () => screen.getByTestId('preview-strip');
const preview = () => screen.getByTestId('preview');
const viewport = () => screen.getByTestId('preview-viewport');
const drawer = () => screen.getByTestId('photo-drawer');
const toast = () => screen.getByTestId('toast').textContent;
/** What the Preview is of: its label (the kicker) and the photo it asks for. */
const shown = () => [screen.queryByTestId('preview')?.getAttribute('aria-label') ?? null, /\/photos\/(\d+)\/preview/.exec(screen.queryByTestId('preview-photo')?.getAttribute('src') ?? '')?.[1] ?? null];
const buttons = (within_: HTMLElement) => within(within_).queryAllByRole('button').map(button => button.getAttribute('aria-label') ?? button.textContent);
const buzzed = (env: PhoneEnv, pattern: number[]) => env.vibrate.mock.calls.filter(call => JSON.stringify(call[0]) === JSON.stringify(pattern)).length;

function tap(target: Element, x: number): void {
  press(target, x, Y);
  lift(target, x, Y);
}
/** A paired phone on fixture S with the Preview of the photo under `x` open; what happened before is forgotten. */
async function openPreview(x: number): Promise<PhoneEnv> {
  const env = phoneEnv();
  await startPhone(env, sortSession());
  tap(dock(), x);
  await advance(0);
  env.vibrate.mockClear();
  env.mac.requests.length = 0;
  return env;
}
async function openDrawer(x: number): Promise<PhoneEnv> {
  const env = await openPreview(x);
  within(preview()).getByRole('button', { name: 'Photo details' }).click();
  await advance(0);
  expect(drawer().getAttribute('aria-hidden')).toBe('false');
  env.vibrate.mockClear();
  return env;
}
/** One finger from the picture's centre to `(dx, dy)` away and off, with no time passing. */
function swipe(dx: number, dy: number): void {
  press(viewport(), 195, 305);
  slide(viewport(), 195 + dx, 305 + dy);
  lift(viewport(), 195 + dx, 305 + dy);
}

// ---- LV9·C11: the Preview at first glance ----

test('lv9_c11a_the_preview_is_the_photo_its_strip_and_two_buttons', async () => {
  await openPreview(X.b);
  expect(preview().getAttribute('aria-label')).toBe('SHOT 02 · CAPTURED');
  for (const text of ['SHOT 02 · CAPTURED', 'Back', '1:1', 'CAPTURED', HOLD_HINT]) expect(within(preview()).queryByText(text)).toBeNull();
  expect(buttons(preview())).toEqual(['Photo details', 'Close']);
  expect(preview().contains(strip())).toBe(true);
  expect(screen.getByTestId('preview-photo').getAttribute('src')).toContain('/photos/201/preview');
});

test('lv9_c11b_the_close_icon_closes_the_preview', async () => {
  const env = await openPreview(X.extra);
  within(preview()).getByRole('button', { name: 'Close' }).click();
  await advance(0);
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv9_c11c_hold_and_swipes_keep_working', async () => {
  const env = await openPreview(X.b);
  expect(screen.queryByTestId('preview-pill')).toBeNull();
  press(viewport(), 100, 305);
  await advance(200);
  expect(screen.getByTestId('preview-pill').textContent).toBe('TEMPLATE');
  lift(viewport(), 100, 305);
  expect(screen.queryByTestId('preview-pill')).toBeNull();
  // A new Preview, of 203: left denies it and 207 shows.
  tap(strip(), X.c1);
  await advance(0);
  expect(shown()).toEqual(['CONDITION 01', '203']);
  swipe(-120, 0);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/photos/203/delete', { session_id: 7 }]]);
  expect(shown()).toEqual(['CONDITION 02', '207']);
});

// ---- LV9·C12: the photo drawer ----

test('lv9_c12a_the_drawer_reads_what_the_header_did_and_offers_retake', async () => {
  await openDrawer(X.b);
  expect(drawer().getAttribute('role')).toBe('dialog');
  const sheet = within(drawer());
  for (const text of ['SHOT 02 · CAPTURED', 'Back', '1:1', HOLD_HINT]) expect(sheet.getByText(text)).toBeTruthy();
  expect(sheet.getByRole('button', { name: 'Retake' })).toBeTruthy();
});

test('lv9_c12a_an_extra_has_no_hint_and_no_retake', async () => {
  await openDrawer(X.extra);
  const sheet = within(drawer());
  for (const text of ['IMAGE 04 · EXTRA', 'Extra image', '1:1']) expect(sheet.getByText(text)).toBeTruthy();
  expect(sheet.queryByText(HOLD_HINT)).toBeNull();
  expect(sheet.queryByRole('button', { name: 'Retake' })).toBeNull();
});

test('lv9_c12b_retake_from_the_drawer_reopens_the_shot', async () => {
  const env = await openDrawer(X.b);
  within(drawer()).getByRole('button', { name: 'Retake' }).click();
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/retake', { session_id: 7, container_id: 12 }]]);
  expect(buzzed(env, [30, 40, 30])).toBe(1);
  expect(toast()).toBe('Shot 02 reopened');
  expect(screen.queryByTestId('photo-drawer')).toBeNull();
  expect(screen.queryByTestId('preview')).toBeNull();
});

test('lv9_c12c_its_close_button_closes_only_the_drawer', async () => {
  const env = await openDrawer(X.b);
  within(drawer()).getByRole('button', { name: 'Close' }).click();
  await advance(0);
  expect(drawer().getAttribute('aria-hidden')).toBe('true');
  expect(shown()).toEqual(['SHOT 02 · CAPTURED', '201']);
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv9_c12c_so_does_the_backdrop', async () => {
  const env = await openDrawer(X.b);
  screen.getByTestId('photo-backdrop').click();
  await advance(0);
  expect(drawer().getAttribute('aria-hidden')).toBe('true');
  expect(shown()).toEqual(['SHOT 02 · CAPTURED', '201']);
  expect(env.mac.allPosts()).toEqual([]);
});

// ---- LV9·C13: under the drawer; without the Move drawer ----

test('lv9_c13a_the_photo_and_the_strip_take_no_touch_under_the_drawer', async () => {
  const env = await openDrawer(X.c1);
  swipe(-120, 0);
  await advance(0);
  tap(strip(), X.c2);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(shown()).toEqual(['CONDITION 01', '203']);
  expect(env.vibrate).not.toHaveBeenCalled();
});

test('lv9_c13b_the_drawer_closes_with_its_preview', async () => {
  const env = await openDrawer(X.c1);
  await macChanges(env, sortSession(s => {
    s.condition = s.condition.filter(image => image.photo_id !== 203);
    s.counts.condition = 1;
  }));
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(screen.queryByTestId('photo-drawer')).toBeNull();
});

test('lv9_c13c_move_image_is_nowhere', async () => {
  const everywhere = () => document.body.textContent ?? '';
  // The live screen and its Item drawer.
  const env = phoneEnv();
  await startPhone(env, sortSession());
  screen.getByRole('button', { name: 'Item' }).click();
  await advance(0);
  expect(everywhere()).toContain('Fill item properties');
  expect(everywhere()).not.toContain('Move image');
  within(screen.getByTestId('item-drawer')).getByRole('button', { name: 'Close' }).click();
  await advance(0);
  // The Preview of a template shot's photo, of an extra and of a Condition image, each with its drawer.
  for (const x of [X.b, X.extra, X.c1]) {
    tap(dock(), x);
    await advance(0);
    expect(everywhere()).not.toContain('Move image');
    within(preview()).getByRole('button', { name: 'Photo details' }).click();
    await advance(0);
    expect(drawer().getAttribute('aria-hidden')).toBe('false');
    expect(everywhere()).not.toContain('Move image');
    within(drawer()).getByRole('button', { name: 'Close' }).click();
    within(preview()).getByRole('button', { name: 'Close' }).click();
    await advance(0);
  }
  expect(existsSync('src/phone/components/MoveDrawer.tsx')).toBe(false);
});
