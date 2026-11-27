// LV10·U3 — the reframe editor (book 10J LV-D21 (6), 10D MC-27; master_plan_live §5 PH12, §6 texts v1.3; plan
// points P3–P9): rows LV10·C9–C13.
// Fixture K (helpers `composedSession`): photo 201 (shot "02", tile centre x 44) with the default 1:1 framing of its
// 2560 × 1707 full frame; 202 (the extra "04", x 236) has no composition. The editor is opened from the Preview of
// 201: "Photo details", "Reframe". Its stage is 390 × 520 at (0, 0) in the layout stub: for a 1:1 rect the frame
// box is left 24, top 89, 342 × 342, its centre (195, 260), and one source px is 342 / 1707 screen px.
//
// "The editor shows `values`" = its picture's size and transform and its frame box are those the Mac's view maths
// gives for `values` (`viewTransform`, `transformCss`); the transform is in 0.01 px steps, so a rect is read to
// about 1e-5 here (row C14(a) pins the same pan to 1e-6 in the request; the bounded moves are pinned in
// `lv10_core.test.ts`).
import { fireEvent, screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { composedLayout } from '@/features/compose/composed';
import { frameBox, panBy, transformCss, validate, viewTransform, zoomAbout } from '@/features/compose/geometry';
import { overlayLines } from '@/features/compose/overlays';
import { refit } from '@/features/compose/refit';
import type { CompositionValues } from '@/services/compositions';
import { panWithin, resetValues } from '../reframe';
import { advance, cancel, composedSession, K_FRAME, K_VALUES, lift, type PhoneEnv, phoneEnv, press, REFRAME_STAGE, slide, startPhone } from './helpers';

const X = { b: 44, extra: 236 };
const Y = 742;
const WAITING = "Waiting for Capture One's full-frame preview";
const K: CompositionValues = K_VALUES;
const dock = () => screen.getByTestId('dock');
const preview = () => screen.getByTestId('preview');
const drawer = () => screen.getByTestId('photo-drawer');
const editor = () => screen.getByTestId('reframe-editor');
const stage = () => screen.getByTestId('reframe-stage');
const picture = () => screen.getByTestId('reframe-picture');
const frame = () => screen.getByTestId('reframe-frame');
const grid = () => screen.queryByTestId('reframe-grid');
const degrees = () => screen.getByTestId('reframe-degrees').textContent;
const slider = () => within(editor()).getByRole('slider', { name: 'Straighten' });
const button = (name: string) => within(editor()).getByRole('button', { name });
const menu = () => screen.queryByTestId('reframe-menu');
const guideLines = () => Array.from(screen.getByTestId('reframe-guides').querySelectorAll('line'))
  .map(line => ({ x1: Number(line.getAttribute('x1')), y1: Number(line.getAttribute('y1')), x2: Number(line.getAttribute('x2')), y2: Number(line.getAttribute('y2')) }));

async function click(element: HTMLElement): Promise<void> {
  element.click();
  await advance(0);
}
/** A menu's entry: the tool is opened, the entry picked. */
async function pick(tool: string, entry: string): Promise<void> {
  await click(button(tool));
  await click(within(menu()!).getByRole('menuitem', { name: entry }));
}
/** "Photo details", then "Reframe", on the open Preview. */
async function openReframe(): Promise<void> {
  await click(within(preview()).getByRole('button', { name: 'Photo details' }));
  await click(within(drawer()).getByRole('button', { name: 'Reframe' }));
}
/** A paired phone with the editor open on photo 201 of `session`; what happened before is forgotten. */
async function openEditor(session = composedSession()): Promise<PhoneEnv> {
  const env = phoneEnv();
  await startPhone(env, session);
  press(dock(), X.b, Y);
  lift(dock(), X.b, Y);
  await advance(0);
  await openReframe();
  env.vibrate.mockClear();
  env.mac.requests.length = 0;
  return env;
}

const TRANSFORM = /^translate\((-?[\d.]+)px, (-?[\d.]+)px\) rotate\((-?[\d.]+)deg\)$/;
const parts = (transform: string) => {
  const found = TRANSFORM.exec(transform);
  if (!found) throw new Error(`not a view transform: "${transform}"`);
  return { x: Number(found[1]), y: Number(found[2]), rotate: Number(found[3]) };
};
/** The editor shows `values`: the picture's size and transform, and the frame box, as the Mac's view maths gives them. */
function expectShows(values: CompositionValues): void {
  const view = viewTransform(values, K_FRAME, REFRAME_STAGE);
  const want = parts(transformCss(view));
  const now = parts(picture().style.transform);
  expect(now.rotate).toBe(want.rotate);
  // One step of the transform's 0.01 px.
  expect(Math.abs(now.x - want.x)).toBeLessThan(0.0101);
  expect(Math.abs(now.y - want.y)).toBeLessThan(0.0101);
  expect(parseFloat(picture().style.width)).toBeCloseTo(view.width, 3);
  expect(parseFloat(picture().style.height)).toBeCloseTo(view.height, 3);
  const box = frame().style;
  expect(parseFloat(box.left)).toBeCloseTo(view.frameBox.left, 3);
  expect(parseFloat(box.top)).toBeCloseTo(view.frameBox.top, 3);
  expect(parseFloat(box.width)).toBeCloseTo(view.frameBox.width, 3);
  expect(parseFloat(box.height)).toBeCloseTo(view.frameBox.height, 3);
}
const near = (rect: { x: number; y: number; w: number; h: number }, want: { x: number; y: number; w: number; h: number }) => {
  for (const side of ['x', 'y', 'w', 'h'] as const) expect(Math.abs(rect[side] - want[side])).toBeLessThan(1e-6);
};

/** One finger from the frame's centre to `(x, y)` and off. */
function drag(x: number, y: number): void {
  press(stage(), 195, 260);
  slide(stage(), x, y);
  lift(stage(), x, y);
}
/** Two fingers on the frame's middle line at `from`, moved to `to`, then lifted. */
function pinch(from: [number, number], to: [number, number]): void {
  press(stage(), from[0], 260, 1);
  press(stage(), from[1], 260, 2);
  slide(stage(), to[0], 260, 1);
  slide(stage(), to[1], 260, 2);
  lift(stage(), to[1], 260, 2);
  lift(stage(), to[0], 260, 1);
}
/** C10(b)'s spread: twice as close about the frame's centre. */
const CLOSER: CompositionValues = { ...K, rect: zoomAbout(K.rect, { x: 0.5, y: 0.5 }, 2, K_FRAME) };
/** Where two fingers 100 px apart about the frame's centre are after turning `angle`° clockwise. */
function turnedTo(angle: number): [[number, number], [number, number]] {
  const radians = (angle * Math.PI) / 180;
  const dx = 50 * Math.cos(radians);
  const dy = 50 * Math.sin(radians);
  return [[195 - dx, 260 - dy], [195 + dx, 260 + dy]];
}
/** Two fingers come down on the middle line and turn `angle`° clockwise; they stay down. Returns whether the grid ever showed. */
function twist(angle: number): boolean {
  const [first, second] = turnedTo(angle);
  let seen = false;
  press(stage(), 145, 260, 1);
  press(stage(), 245, 260, 2);
  seen ||= grid() !== null;
  slide(stage(), first[0], first[1], 1);
  seen ||= grid() !== null;
  slide(stage(), second[0], second[1], 2);
  seen ||= grid() !== null;
  return seen;
}
function untwist(angle: number): void {
  const [first, second] = turnedTo(angle);
  lift(stage(), second[0], second[1], 2);
  lift(stage(), first[0], first[1], 1);
}

// ---- LV10·C9: opening ----

test('lv10_c9a_reframe_opens_like_the_macs_editor', async () => {
  await openEditor();
  expect(editor().getAttribute('role')).toBe('dialog');
  expect(editor().getAttribute('aria-label')).toBe('Reframe');
  expect(parseFloat(frame().style.left)).toBe(24);
  expect(parseFloat(frame().style.top)).toBeCloseTo(89, 4);
  expect(parseFloat(frame().style.width)).toBe(342);
  expect(parseFloat(frame().style.height)).toBeCloseTo(342, 4);
  expect(picture().style.transform).toBe(transformCss(viewTransform(K, K_FRAME, { left: 0, top: 0, width: 390, height: 520 })));
  expect(picture().querySelector('img')!.getAttribute('src')).toContain('/photos/201/preview');
  // Thirds the first time: two lines each way, inside the frame box.
  expect(guideLines()).toEqual(overlayLines('thirds', frameBox(REFRAME_STAGE, K.rect.w * 2560 / (K.rect.h * 1707)), false));
  expect(guideLines()).toHaveLength(4);
  expect(grid()).toBeNull();
});

test('lv10_c9b_reframe_waits_for_the_full_frame_preview', async () => {
  const env = phoneEnv();
  await startPhone(env, composedSession());
  press(dock(), X.extra, Y);
  lift(dock(), X.extra, Y);
  await advance(0);
  await click(within(preview()).getByRole('button', { name: 'Photo details' }));
  const reframe = within(drawer()).getByRole('button', { name: 'Reframe' }) as HTMLButtonElement;
  expect(reframe.disabled).toBe(true);
  expect(within(drawer()).getByText(WAITING)).toBeTruthy();
  await click(reframe);
  expect(screen.queryByTestId('reframe-editor')).toBeNull();
});

test('lv10_c9c_a_photo_that_needs_review_opens_refitted_to_its_frame', async () => {
  // The Mac asks for a review: the stored rect no longer has the shape on this frame (0.5 × 0.5 of 2560 × 1707 is not square).
  const stored = { ...K_VALUES, rect: { x: 0.1, y: 0.1, w: 0.5, h: 0.5 } };
  const refitted = refit(stored, { kind: 'reframe' }, K_FRAME);
  expect(Math.abs(refitted.rect.w - stored.rect.w)).toBeGreaterThan(0.05);
  await openEditor(composedSession(session => {
    session.shots[0].photo!.composition = { ...stored, frame: { ...K_FRAME }, state: 'needs_review' };
  }));
  expectShows(refitted);
});

// ---- LV10·C10: gestures ----

test('lv10_c10a_one_finger_moves_the_photo_under_the_frame', async () => {
  await openEditor();
  // 34.2 px to the right is a tenth of the frame box: the rect moves left by a tenth of its width.
  drag(229.2, 260);
  const want = { ...K, rect: { ...K.rect, x: 0.0999219 } };
  near(want.rect, { x: 0.1666016 - 0.0666797, y: 0, w: 0.6667969, h: 1 });
  expectShows(want);
});

test('lv10_c10b_two_fingers_spread_zoom_about_the_point_between_them', async () => {
  await openEditor();
  pinch([145, 245], [95, 295]);
  near(CLOSER.rect, { x: 0.3333008, y: 0.25, w: 0.3333984, h: 0.5 });
  expectShows(CLOSER);
});

test('lv10_c10c_a_twist_straightens_after_3_degrees_and_shows_the_grid_meanwhile', async () => {
  await openEditor();
  expect(twist(10)).toBe(true);
  expect(grid()).not.toBeNull();
  expect(grid()!.style.backgroundSize).toBe('24px 24px');
  untwist(10);
  expect(grid()).toBeNull();
  expect(degrees()).toBe('7.0°');
  expectShows(refit(K, { kind: 'straighten', degrees: 7 }, K_FRAME));
});

test('lv10_c10d_a_small_twist_changes_nothing', async () => {
  await openEditor();
  expect(twist(2)).toBe(false);
  untwist(2);
  expect(grid()).toBeNull();
  expect(degrees()).toBe('0.0°');
  expectShows(K);
});

// ---- LV10·C11: the frame never shows empty corners ----

test('lv10_c11a_the_photo_stops_at_its_edge', async () => {
  await openEditor();
  press(stage(), 195, 260);
  slide(stage(), 495, 260);
  // While the finger is down, not only at release.
  expectShows({ ...K, rect: { ...K.rect, x: 0 } });
  lift(stage(), 495, 260);
  expectShows({ ...K, rect: { ...K.rect, x: 0 } });
});

test('lv10_c11b_a_straightened_photo_stops_on_the_images_edge', async () => {
  await openEditor();
  fireEvent.change(slider(), { target: { value: '5' } });
  const straightened = refit(K, { kind: 'straighten', degrees: 5 }, K_FRAME);
  expectShows(straightened);
  drag(545, 610);
  // The same move in frame fractions: 350 px each way on the picture as it was shown.
  const view = viewTransform(straightened, K_FRAME, REFRAME_STAGE);
  const delta = { x: -350 / view.width, y: -350 / view.height };
  const stopped = panWithin(straightened, K_FRAME, delta);
  expectShows(stopped);
  expect(validate(stopped, K_FRAME)).toBeNull();
  expect(validate({ ...stopped, rect: panBy(stopped.rect, { x: delta.x * 0.01, y: delta.y * 0.01 }) }, K_FRAME)).not.toBeNull();
});

test('lv10_c11c_zooming_out_stops_at_the_largest_frame', async () => {
  await openEditor();
  pinch([145, 245], [95, 295]);
  expectShows(CLOSER);
  // A quarter of the distance would be four times further out: it stops at the whole height.
  pinch([95, 295], [170, 220]);
  expectShows(K);
});

test('lv10_c11d_a_cancelled_move_keeps_what_it_showed', async () => {
  const env = await openEditor();
  press(stage(), 195, 260);
  slide(stage(), 229.2, 260);
  cancel(stage(), 229.2, 260);
  await advance(400);
  expectShows({ ...K, rect: { ...K.rect, x: 0.0999219 } });
  // The finger the phone took away moves nothing more.
  slide(stage(), 300, 300);
  lift(stage(), 300, 300);
  expectShows({ ...K, rect: { ...K.rect, x: 0.0999219 } });
  expect(env.mac.allPosts()).toEqual([]);
});

// ---- LV10·C12: the controls ----

test('lv10_c12a_split_makes_the_frame_two_squares', async () => {
  await openEditor();
  expect(screen.queryByTestId('reframe-middle')).toBeNull();
  await click(button('Split'));
  const split = refit(K, { kind: 'kind', to: 'split' }, K_FRAME);
  expect(split.kind).toBe('split');
  expectShows(split);
  expect(parseFloat(frame().style.width)).toBeCloseTo(342, 3);
  expect(parseFloat(frame().style.height)).toBeCloseTo(171, 3);
  expect(frame().contains(screen.getByTestId('reframe-middle'))).toBe(true);
  expect(button('Split').getAttribute('aria-pressed')).toBe('true');
});

test('lv10_c12b_shape_and_turn_refit_the_frame', async () => {
  await openEditor();
  expect(within(menu() ?? editor()).queryByRole('menuitem')).toBeNull();
  await click(button('Shape'));
  expect(within(menu()!).getAllByRole('menuitem').map(item => item.textContent)).toEqual(['Original', '1:1', '4:5', '3:2', '2:3', '16:9', 'Free']);
  await click(within(menu()!).getByRole('menuitem', { name: '4:5' }));
  const shaped = refit(K, { kind: 'shape', shape: '4:5' }, K_FRAME);
  expectShows(shaped);
  expect(parseFloat(frame().style.width) / parseFloat(frame().style.height)).toBeCloseTo(0.8, 4);
  await click(button('Turn'));
  const turned = refit(shaped, { kind: 'orientation', orientation: 'turned' }, K_FRAME);
  expectShows(turned);
  expect(parseFloat(frame().style.width) / parseFloat(frame().style.height)).toBeCloseTo(1.25, 4);
});

test('lv10_c12c_rotate_and_flip_from_one_menu_that_closes_after_a_pick', async () => {
  await openEditor();
  await click(button('Rotate and flip'));
  expect(within(menu()!).getAllByRole('menuitem').map(item => item.textContent)).toEqual(['Rotate 90° left', 'Rotate 90° right', 'Flip horizontal']);
  await click(within(menu()!).getByRole('menuitem', { name: 'Rotate 90° right' }));
  expect(menu()).toBeNull();
  const turned = refit(K, { kind: 'turn', direction: 'right' }, K_FRAME);
  expect(turned.quarter_turns).toBe(1);
  expectShows(turned);
  expect(picture().querySelector('img')!.style.transform).toBe(composedLayout(turned, K_FRAME).imageTransform);
  expect(picture().querySelector('img')!.style.transform).toBe('rotate(90deg) scaleX(1)');
  await pick('Rotate and flip', 'Flip horizontal');
  expect(menu()).toBeNull();
  const flipped = refit(turned, { kind: 'flip' }, K_FRAME);
  expect(flipped.flip).toBe('horizontal');
  expectShows(flipped);
  expect(picture().querySelector('img')!.style.transform).toBe('rotate(90deg) scaleX(-1)');
});

test('lv10_c12d_the_straighten_control_shows_the_grid_while_it_is_held', async () => {
  await openEditor();
  expect(degrees()).toBe('0.0°');
  press(slider(), 100, 560);
  expect(grid()).not.toBeNull();
  fireEvent.change(slider(), { target: { value: '2.5' } });
  expect(grid()).not.toBeNull();
  expect(degrees()).toBe('2.5°');
  expectShows(refit(K, { kind: 'straighten', degrees: 2.5 }, K_FRAME));
  lift(slider(), 100, 560);
  expect(grid()).toBeNull();
  expectShows(refit(K, { kind: 'straighten', degrees: 2.5 }, K_FRAME));
});

test('lv10_c12e_the_guides_are_chosen_and_remembered', async () => {
  await openEditor();
  await click(button('Guides'));
  expect(within(menu()!).getAllByRole('menuitem').map(item => item.textContent)).toEqual(['Off', 'Thirds', 'Grid', 'Golden ratio', 'Diagonals']);
  await click(within(menu()!).getByRole('menuitem', { name: 'Golden ratio' }));
  const golden = overlayLines('golden', viewTransform(K, K_FRAME, REFRAME_STAGE).frameBox, false);
  expect(golden).toHaveLength(4);
  expect(guideLines()).toEqual(golden);
  expect(window.localStorage.getItem('psm.phone.reframeGuides')).toBe('golden');
  await click(button('Cancel'));
  expect(screen.queryByTestId('reframe-editor')).toBeNull();
  await openReframe();
  expect(guideLines()).toEqual(golden);
  expect(window.localStorage.getItem('psm.phone.reframeGuides')).toBe('golden');
});

// ---- LV10·C13: Reset and Cancel ----

test('lv10_c13a_reset_puts_the_macs_default_framing_in_the_editor', async () => {
  await openEditor();
  pinch([145, 245], [95, 295]);
  expectShows(CLOSER);
  await click(button('Reset'));
  const reset = resetValues({ w: 2560, h: 1707 }, '1:1');
  near(reset.rect, K.rect);
  expectShows(reset);
});

test('lv10_c13b_cancel_discards_the_framing', async () => {
  const env = await openEditor();
  pinch([145, 245], [95, 295]);
  expectShows(CLOSER);
  await click(button('Cancel'));
  expect(screen.queryByTestId('reframe-editor')).toBeNull();
  await openReframe();
  expectShows(K);
  expect(env.mac.allPosts()).toEqual([]);
});
