// LV8·U3 — lift, drag and drop in both strips (book 10J LV-D20 (3); master_plan_live §5 PH9.2, PH9.3; plan
// points P1, P7, P10): rows LV8·C9–C13. Fixture D (helpers `dragSession`): shots in shooting order B (final
// 02, photo 201), A (01, photo 204), C (03, empty); extras 202, 205, 206; condition 203, 207, 208; current =
// Free shot; the stage is 800 wide unless a row says 390. Tile centres x: B 44 · A 108 · C 172 · 202 236 ·
// 205 300 · 206 364 · Free 428 · 203 511 · 207 575 · 208 639; the strip is y 700–784, tile centres y 742.
// "Lift X" = a press at X's centre and 400 ms.
import { fireEvent, screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { A, advance, C, dragSession, lift, type PhoneEnv, phoneEnv, press, slide, startPhone } from './helpers';

const X = { b: 44, a: 108, c: 172, p202: 236, p205: 300, p206: 364, free: 428, p203: 511, p207: 575, p208: 639 };
const Y = 742;
const CONFIRM = 'Move this photo to Main, outside its template order?';
const dock = () => screen.getByTestId('dock');
const strip = () => screen.getByTestId('preview-strip');
const tile = (key: string, within_: HTMLElement = dock()) => within_.querySelector(`[data-dock-k="${key}"]`) as HTMLElement;
const tiles = (within_: HTMLElement = dock()) => Array.from(within_.querySelectorAll<HTMLElement>('[data-dock-k]'));
const lifted = (within_: HTMLElement = dock()) => tiles(within_).filter(element => element.hasAttribute('data-lifted')).map(element => element.dataset.dockK);
const marked = (mark: string) => tiles().filter(element => element.dataset.drop === mark).map(element => element.dataset.dockK);
const slid = () => tiles().filter(element => element.style.transform.includes('translateX')).map(element => element.dataset.dockK);
const scrollLeft = () => (dock().querySelector('.ph-dock-scroller') as HTMLElement).scrollLeft;
// LV9 (book 10J LV-D21 (4)): the Preview has no header; what it shows is its name.
const kicker = () => screen.queryByTestId('preview')?.getAttribute('aria-label') ?? null;
const dialog = () => screen.queryByRole('dialog', { name: CONFIRM });
/** Every vibration so far, as the pattern given. */
const buzzes = (env: PhoneEnv) => env.vibrate.mock.calls.map(call => call[0]);
const buzzed = (env: PhoneEnv, pattern: number[]) => env.vibrate.mock.calls.filter(call => JSON.stringify(call[0]) === JSON.stringify(pattern)).length;
const move = (photo: number, target: unknown): [string, unknown] => [`/photos/${photo}/move`, { session_id: 7, target }];
const classify = (photo: number, to: 'main' | 'condition'): [string, unknown] => [`/photos/${photo}/classify`, { session_id: 7, to }];

/** A paired phone on fixture D. */
async function start(width = 800): Promise<PhoneEnv> {
  const env = phoneEnv({ stage: { width, height: 610 } });
  await startPhone(env, dragSession());
  env.mac.requests.length = 0;
  return env;
}
/** The finger rests 400 ms on the tile at `x`. */
async function liftAt(x: number, on: HTMLElement = dock()): Promise<void> {
  press(on, x, Y);
  await advance(400);
}

// ---- LV8·C9: tap, lift, or scroll ----

test('lv8_c9a_a_still_touch_released_before_400_ms_is_a_tap', async () => {
  const env = await start();
  press(dock(), X.b, Y);
  await advance(399);
  expect(lifted()).toEqual([]);
  lift(dock(), X.b, Y);
  await advance(0);
  expect(kicker()).toBe('SHOT 02 · CAPTURED');
  expect(lifted()).toEqual([]);
  expect(buzzes(env)).toEqual([15]);
});

test('lv8_c9b_a_photo_lifts_after_400_ms_and_put_down_in_its_place_nothing_happens', async () => {
  const env = await start();
  await liftAt(X.b);
  expect(lifted()).toEqual(['p201']);
  expect(tile('p201').style.transform).toContain('scale(1.15)');
  expect(buzzes(env)).toEqual([12]);
  lift(dock(), X.b, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(lifted()).toEqual([]);
  expect(tile('p201').style.transform).not.toContain('scale');
  expect(buzzes(env)).toEqual([12]);
});

test('lv8_c9c_an_empty_shot_never_lifts_its_long_press_is_a_tap', async () => {
  const env = await start();
  press(dock(), X.c, Y);
  await advance(600);
  expect(lifted()).toEqual([]);
  lift(dock(), X.c, Y);
  await advance(0);
  expect(lifted()).toEqual([]);
  expect(env.mac.allPosts()).toEqual([['/current', { session_id: 7, container_id: C }]]);
});

test('lv8_c9d_a_touch_that_scrolls_first_never_lifts', async () => {
  const env = await start();
  press(dock(), X.p202, Y);
  await advance(100);
  slide(dock(), 227, Y);
  await advance(500);
  expect(lifted()).toEqual([]);
  lift(dock(), 227, Y);
  await advance(0);
  expect(lifted()).toEqual([]);
  expect(scrollLeft()).toBe(9);
  expect(env.mac.allPosts()).toEqual([]);
});

// ---- LV8·C10: within the template shots, the extras, the Condition images ----

test('lv8_c10a_a_template_shots_photo_dropped_on_a_filled_shot_swaps', async () => {
  const env = await start();
  await liftAt(X.b);
  slide(dock(), X.a, Y);
  expect(marked('swap')).toEqual(['p204']);
  lift(dock(), X.a, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([move(201, { kind: 'container', container_id: A })]);
  expect(buzzes(env)).toEqual([12, 6, [30, 40, 60]]);
});

test('lv8_c10b_dropped_on_an_empty_shot_it_fills_it', async () => {
  const env = await start();
  await liftAt(X.b);
  slide(dock(), X.c, Y);
  expect(marked('swap')).toEqual(['s13']);
  lift(dock(), X.c, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([move(201, { kind: 'container', container_id: C })]);
});

test('lv8_c10c_an_extra_dragged_along_the_extras_slides_the_others_aside_and_is_inserted', async () => {
  const env = await start();
  await liftAt(X.p202);
  slide(dock(), 330, Y);
  expect(tile('p205').style.transform).toContain('translateX(-64px)');
  expect(tile('p206').style.transform).not.toContain('translate');
  slide(dock(), 380, Y);
  expect(tile('p205').style.transform).toContain('translateX(-64px)');
  expect(tile('p206').style.transform).toContain('translateX(-64px)');
  lift(dock(), 380, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([move(202, { kind: 'extra_insert', index: 3 })]);
});

test('lv8_c10d_a_condition_image_is_inserted_among_the_condition_images', async () => {
  const env = await start();
  await liftAt(X.p208);
  slide(dock(), 540, Y);
  expect(tile('p207').style.transform).toContain('translateX(64px)');
  expect(tile('p203').style.transform).not.toContain('translate');
  lift(dock(), 540, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([move(208, { kind: 'condition_insert', index: 2 })]);
});

test('lv8_c10e_dropped_in_its_own_place_nothing_is_asked', async () => {
  const env = await start();
  await liftAt(X.p205);
  slide(dock(), 310, Y);
  expect(slid()).toEqual([]);
  lift(dock(), 310, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(slid()).toEqual([]);
  expect(buzzes(env)).toEqual([12]);
});

// ---- LV8·C11: across the groups ----

test('lv8_c11a_an_extra_dropped_on_an_empty_shot_fills_it', async () => {
  const env = await start();
  await liftAt(X.p202);
  slide(dock(), X.c, Y);
  lift(dock(), X.c, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([move(202, { kind: 'container', container_id: C })]);
});

test('lv8_c11b_a_condition_image_dropped_on_a_filled_shot_swaps_with_it', async () => {
  const env = await start();
  await liftAt(X.p203);
  slide(dock(), X.a, Y);
  expect(marked('swap')).toEqual(['p204']);
  lift(dock(), X.a, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([move(203, { kind: 'container', container_id: A })]);
});

test('lv8_c11c_an_extra_dropped_among_the_condition_images_goes_to_their_end', async () => {
  const env = await start();
  await liftAt(X.p202);
  slide(dock(), X.p207, Y);
  expect(marked('group')).toEqual(['p203', 'p207', 'p208']);
  expect(slid()).toEqual([]);
  lift(dock(), X.p207, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([classify(202, 'condition')]);
});

test('lv8_c11d_a_condition_image_dropped_among_the_extras_goes_to_main_without_asking', async () => {
  const env = await start();
  await liftAt(X.p203);
  slide(dock(), X.p205, Y);
  expect(marked('group')).toEqual(['p202', 'p205', 'p206']);
  lift(dock(), X.p205, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([classify(203, 'main')]);
  expect(dialog()).toBeNull();
});

test('lv8_c11e_a_template_shots_photo_dropped_among_the_condition_images_goes_to_condition', async () => {
  const env = await start();
  await liftAt(X.b);
  slide(dock(), X.p207, Y);
  lift(dock(), X.p207, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([classify(201, 'condition')]);
  expect(dialog()).toBeNull();
});

test('lv8_c11f_a_template_shots_photo_dropped_among_the_extras_asks_first', async () => {
  const env = await start();
  const dropOnExtras = async () => {
    await liftAt(X.b);
    slide(dock(), X.p205, Y);
    lift(dock(), X.p205, Y);
    await advance(0);
  };
  await dropOnExtras();
  expect(env.mac.allPosts()).toEqual([]);
  expect(within(dialog()!).getByText(CONFIRM)).toBeTruthy();
  expect(buzzed(env, [30, 40, 60])).toBe(0);
  fireEvent.click(within(dialog()!).getByRole('button', { name: 'Cancel' }));
  await advance(0);
  expect(dialog()).toBeNull();
  expect(env.mac.allPosts()).toEqual([]);
  // Separately: Move.
  await dropOnExtras();
  fireEvent.click(within(dialog()!).getByRole('button', { name: 'Move' }));
  await advance(0);
  expect(env.mac.allPosts()).toEqual([classify(201, 'main')]);
  expect(dialog()).toBeNull();
});

test('lv8_c11g_dropped_on_the_free_shot_tile_nothing_happens', async () => {
  const env = await start();
  await liftAt(X.p202);
  slide(dock(), X.free, Y);
  expect(marked('swap')).toEqual([]);
  expect(marked('group')).toEqual([]);
  lift(dock(), X.free, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(buzzed(env, [30, 40, 60])).toBe(0);
});

// ---- LV8·C12: the strip scrolls under a lifted photo at its ends ----

/** Stage 390: 202 lifted and held 160 ms in the right end zone, then 160 ms in the middle. */
async function scrolledRight(): Promise<PhoneEnv> {
  const env = await start(390);
  await liftAt(X.p202);
  slide(dock(), 370, Y);
  await advance(160);
  expect(scrollLeft()).toBe(80);
  slide(dock(), 200, Y);
  await advance(160);
  expect(scrollLeft()).toBe(80);
  return env;
}

test('lv8_c12a_at_the_right_end_the_strip_scrolls_8_px_every_16_ms_and_stops_away_from_it', async () => {
  await scrolledRight();
});

test('lv8_c12b_at_the_left_end_it_scrolls_back_and_stops_at_the_start', async () => {
  await scrolledRight();
  slide(dock(), 20, Y);
  await advance(160);
  expect(scrollLeft()).toBe(0);
  await advance(160);
  expect(scrollLeft()).toBe(0);
});

test('lv8_c12c_the_target_follows_the_scroll', async () => {
  const env = await start(390);
  await liftAt(X.p202);
  slide(dock(), 370, Y);
  await advance(160);
  // The finger at 300 is now over content x 380: past 205 and 206.
  slide(dock(), 300, Y);
  lift(dock(), 300, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([move(202, { kind: 'extra_insert', index: 3 })]);
});

test('lv8_c12d_without_a_hold_the_end_of_the_strip_scrolls_nothing_by_itself', async () => {
  const env = await start(390);
  press(dock(), X.p202, Y);
  slide(dock(), 20, Y);
  expect(scrollLeft()).toBe(216);
  await advance(160);
  expect(scrollLeft()).toBe(216);
  expect(lifted()).toEqual([]);
  lift(dock(), 20, Y);
  await advance(0);
  expect(lifted()).toEqual([]);
  expect(env.mac.allPosts()).toEqual([]);
});

// ---- LV8·C13: the Preview's strip ----

/** The Preview of B's photo (201), opened by a tap on the live screen's strip. */
async function previewOfB(): Promise<PhoneEnv> {
  const env = await start();
  press(dock(), X.b, Y);
  lift(dock(), X.b, Y);
  await advance(0);
  expect(kicker()).toBe('SHOT 02 · CAPTURED');
  env.mac.requests.length = 0;
  return env;
}

test('lv8_c13a_a_photo_is_dragged_in_the_previews_strip_too', async () => {
  const env = await previewOfB();
  await liftAt(X.p202, strip());
  expect(lifted(strip())).toEqual(['p202']);
  expect(lifted()).toEqual([]);
  slide(strip(), X.c, Y);
  lift(strip(), X.c, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([move(202, { kind: 'container', container_id: C })]);
  expect(kicker()).toBe('SHOT 02 · CAPTURED');
});

test('lv8_c13b_the_preview_follows_its_own_photo_dragged_out_of_its_shot', async () => {
  const env = await previewOfB();
  await liftAt(X.b, strip());
  slide(strip(), X.p207, Y);
  // The Mac will have 201 as the fourth Condition image.
  env.mac.view = { session: dragSession(s => {
    s.condition.push(s.shots[0].photo!);
    s.shots[0].photo = null;
  }) };
  lift(strip(), X.p207, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([classify(201, 'condition')]);
  expect(kicker()).toBe('CONDITION 04');
  expect(screen.getByTestId('preview-photo').getAttribute('src')).toContain('/photos/201/preview');
});
