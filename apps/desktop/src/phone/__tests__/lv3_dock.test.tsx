// LV3·U3 — the shot dock (design README §3, §Shot dock gestures, §Selecting a tile; cards LV5, LV7):
// rows LV3·C15–C19, and the dock halves of C13(a) and C14(f). The fixture session (helpers): shots in
// shooting order B (final 02, "Back", filled), A (01, "Front", current), C (03); one extra (04); one
// condition image. Tile centres, x: "02" 34 · "01" 76 · "03" 118 · "04" 160 · Free 202 · "C1" 261; y 730.
import { screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { tileScale } from '../gestures';
import { advance, C, chairSession, lift, macChanges, type PhoneEnv, phoneEnv, press, slide, startPhone } from './helpers';

const X = { b: 34, a: 76, c: 118, extra: 160, free: 202, c1: 261 };
const Y = 730;
const dock = () => screen.getByTestId('dock');
const row = () => Array.from(dock().querySelector('.ph-dock-row')!.children) as HTMLElement[];
const tile = (label: string) => dock().querySelector(`[aria-label="${label}"]`) as HTMLElement;
const buzzed = (env: PhoneEnv, pattern: number) => env.vibrate.mock.calls.filter(call => call[0] === pattern).length;
const previewPhoto = () => screen.queryByTestId('preview')?.querySelector('img')?.getAttribute('src') ?? null;
function tap(x: number): void {
  press(dock(), x, Y);
  lift(dock(), x, Y);
}
async function hold(x: number): Promise<void> {
  press(dock(), x, Y);
  await advance(200);
}

// ---- LV3·C15: the tiles ----

test('lv3_c15a_tiles_in_shooting_order_then_extras_free_shot_and_condition', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const seen = row().map(element => {
    if (element.dataset.testid === 'dock-divider') return 'divider';
    const image = element.querySelector('img');
    const tag = element.querySelector('.ph-tag');
    return {
      tag: tag?.textContent ?? null,
      invertedTag: tag?.classList.contains('ph-tag-inverted') ?? false,
      image: image ? (image.getAttribute('src')!.includes('/photos/') ? 'photo' : 'template') : null,
      grey: image?.classList.contains('ph-grey') ?? false,
      check: element.querySelector('[aria-label="Captured"]') !== null,
      dashedPlus: element.classList.contains('ph-dock-free') && element.querySelector('path[d="M12 5v14M5 12h14"]') !== null,
    };
  });
  expect(seen).toEqual([
    { tag: '02', invertedTag: false, image: 'photo', grey: false, check: true, dashedPlus: false },
    { tag: '01', invertedTag: false, image: 'template', grey: true, check: false, dashedPlus: false },
    { tag: '03', invertedTag: false, image: 'template', grey: true, check: false, dashedPlus: false },
    { tag: '04', invertedTag: true, image: 'photo', grey: false, check: false, dashedPlus: false },
    { tag: null, invertedTag: false, image: null, grey: false, check: false, dashedPlus: true },
    'divider',
    { tag: 'C1', invertedTag: true, image: 'photo', grey: false, check: false, dashedPlus: false },
  ]);
  expect(tile('02 · Back').querySelector('img')!.getAttribute('src')).toBe('/api/phone/v1/photos/201/preview?pair=T&r=1');
  expect(tile('01 · Front').querySelector('img')!.getAttribute('src')).toBe('/api/phone/v1/containers/11/example?pair=T');
});

test('lv3_c15b_only_the_current_shot_is_outlined', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const outlined = row().filter(element => element.hasAttribute('data-dock-k') && !element.style.outline.includes('transparent'));
  expect(outlined.map(element => element.getAttribute('aria-label'))).toEqual(['01 · Front']);
  expect(outlined[0].style.outline.startsWith('2px solid')).toBe(true);
  expect(outlined[0].style.outlineOffset).toBe('2px');
});

test('lv3_c15c_a_split_shot_has_the_white_divider_at_the_middle', async () => {
  const env = phoneEnv();
  await startPhone(env, chairSession(s => {
    s.shots[0].split = true;
  }));
  const split = tile('02 · Back').querySelector('[data-testid="dock-split"]') as HTMLElement;
  expect(split.style.left).toBe('50%');
  expect(dock().querySelectorAll('[data-testid="dock-split"]')).toHaveLength(1);
});

// ---- LV3·C16: magnification ----

test('lv3_c16a_tileScale_is_2_5_under_the_finger_and_1_from_95_px', () => {
  expect([tileScale(100, 100), tileScale(100, 147.5), tileScale(100, 195), tileScale(100, 220)]).toEqual([2.5, 1.75, 1, 1]);
  expect(tileScale(147.5, 100)).toBe(1.75);
});

test('lv3_c16b_a_held_tile_is_magnified_and_names_itself', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const label = () => screen.getByTestId('dock-label').textContent;
  await hold(X.a);
  expect(label()).toBe('01 · Front');
  expect(tile('01 · Front').style.width).toBe('90px');
  expect(tile('Condition 01').style.width).toBe('36px');
  slide(dock(), X.extra, Y);
  expect(label()).toBe('04 · Extra image');
  slide(dock(), X.free, Y);
  expect(label()).toBe('Free shot · no template');
  slide(dock(), X.c1, Y);
  expect(label()).toBe('Condition 01');
});

// ---- LV3·C17: selecting ----

test('lv3_c17a_tapping_an_empty_template_shot_makes_it_current_and_clears_the_focus_marker', async () => {
  const env = phoneEnv();
  await startPhone(env);
  // A focus marker is set first: lock live, tap the picture.
  const viewport = screen.getByTestId('viewport');
  press(viewport, 195, 405);
  slide(viewport, 195, 390);
  slide(viewport, 195, 295);
  lift(viewport, 195, 295);
  press(viewport, 195, 305);
  lift(viewport, 195, 305);
  await advance(260);
  expect(screen.getByTestId('reticle')).toBeTruthy();

  tap(X.c);
  await advance(0);
  expect(env.mac.posts('/current')).toEqual([{ session_id: 7, container_id: C }]);
  expect(buzzed(env, 15)).toBe(1);
  expect(screen.queryByTestId('reticle')).toBeNull();
  expect(screen.queryByTestId('preview')).toBeNull();
});

test('lv3_c17b_tapping_a_filled_shot_opens_its_photo', async () => {
  const env = phoneEnv();
  await startPhone(env);
  tap(X.b);
  await advance(0);
  expect(previewPhoto()).toBe('/api/phone/v1/photos/201/preview?pair=T&r=1');
  expect(buzzed(env, 15)).toBe(1);
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv3_c17c_tapping_free_shot_makes_free_current', async () => {
  const env = phoneEnv();
  await startPhone(env);
  tap(X.free);
  await advance(0);
  expect(env.mac.posts('/current')).toEqual([{ session_id: 7, container_id: null }]);
});

// ---- LV3·C18: swipe and hold ----

test('lv3_c18a_a_quick_sideways_swipe_scrolls_and_selects_nothing', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const scroller = dock().querySelector('.ph-dock-scroller') as HTMLElement;
  scroller.scrollLeft = 40;
  press(dock(), 100, Y);
  await advance(150);
  slide(dock(), 112, Y);
  expect(scroller.scrollLeft).toBe(28);
  lift(dock(), 112, Y);
  await advance(500);
  expect(env.mac.allPosts()).toEqual([]);
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(screen.queryByTestId('dock-label')).toBeNull();
});

test('lv3_c18b_holding_200_ms_magnifies_with_a_tick_per_tile', async () => {
  const env = phoneEnv();
  await startPhone(env);
  press(dock(), X.a, Y);
  await advance(199);
  expect(buzzed(env, 12)).toBe(0);
  await advance(1);
  expect(buzzed(env, 12)).toBe(1);
  expect(buzzed(env, 6)).toBe(0);
  slide(dock(), X.c, Y);
  expect(buzzed(env, 6)).toBe(1);
  slide(dock(), X.c + 2, Y);
  expect(buzzed(env, 6)).toBe(1);
});

// ---- LV3·C19: hold, drag up, release ----

test('lv3_c19a_dragging_a_held_tile_up_past_50_px_opens_it_on_release', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await hold(X.b);
  slide(dock(), X.b, Y - 40);
  expect(buzzed(env, 18)).toBe(0);
  expect(screen.getByTestId('dock-label').textContent).toBe('02 · Back');
  slide(dock(), X.b, Y - 60);
  expect(buzzed(env, 18)).toBe(1);
  const label = screen.getByTestId('dock-label');
  expect(label.textContent).toBe('Release to open');
  expect(label.classList.contains('ph-dock-label-armed')).toBe(true);
  expect(screen.queryByTestId('preview')).toBeNull();
  lift(dock(), X.b, Y - 60);
  expect(previewPhoto()).toBe('/api/phone/v1/photos/201/preview?pair=T&r=1');
});

test('lv3_c19b_bringing_the_tile_back_down_before_release_opens_nothing', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await hold(X.b);
  slide(dock(), X.b, Y - 60);
  slide(dock(), X.b, Y - 30);
  lift(dock(), X.b, Y - 30);
  await advance(500);
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv3_c19c_opening_an_empty_template_shot_selects_it', async () => {
  const env = phoneEnv();
  await startPhone(env);
  await hold(X.c);
  slide(dock(), X.c, Y - 60);
  lift(dock(), X.c, Y - 60);
  await advance(0);
  expect(env.mac.posts('/current')).toEqual([{ session_id: 7, container_id: C }]);
  expect(screen.queryByTestId('preview')).toBeNull();
});

// ---- the dock halves of LV3·C13(a) and LV3·C14(f) ----

test('lv3_c13a_the_dock_tiles_turn_with_a_long_shot', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const turned = () => row().filter(element => element.hasAttribute('data-dock-k')).map(element => element.style.transform.includes('rotate(90deg)'));
  expect(turned()).toEqual([false, false, false, false, false, false]);
  await macChanges(env, chairSession(s => {
    s.shots[1].ratio = 1.5;
  }));
  expect(turned()).toEqual([true, true, true, true, true, true]);
});

test('lv3_c14f_the_outline_follows_the_macs_current_shot', async () => {
  const env = phoneEnv();
  await startPhone(env);
  const outlined = () => row().filter(element => element.hasAttribute('data-dock-k') && !element.style.outline.includes('transparent')).map(element => element.getAttribute('aria-label'));
  expect(outlined()).toEqual(['01 · Front']);
  await macChanges(env, chairSession(s => {
    s.shots[1].photo = { photo_id: 302, revision: 1, ratio: 1 };
    s.current = { kind: 'shot', container_id: C };
  }));
  expect(outlined()).toEqual(['03 · Container 3']);
  await macChanges(env, chairSession(s => {
    s.current = { kind: 'free' };
  }));
  expect(outlined()).toEqual(['Free shot']);
  expect(env.mac.posts('/current')).toEqual([]);
});
