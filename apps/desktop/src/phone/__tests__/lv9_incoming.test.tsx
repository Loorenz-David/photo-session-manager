// LV9·U2 — a photo on its way shows the loading cue in both strips, from the shutter until its picture (book
// 10J LV-D21 (1); master_plan_live §5 PH6b, PH7; plan points P1–P3): rows LV9·C4–C6.
// Fixture W (helpers `waitingSession`): the chair session with a photo on its way to shot C (final 03) and two
// on their way to the Condition images; tile centres x: "02" 44 · "01" 108 · "03" 172 · "04" 236 · Free 300 ·
// "C1" 383 · waiting 447, 511; y 742. Fixture D / DW (`dragSession`, `dragWaitingSession`), stage 800: B 44 ·
// A 108 · C 172 · 202 236 · 205 300 · 206 364 · Free 428 · 203 511 · 207 575 · 208 639 · (DW) waiting 703.
// @ts-expect-error The frontend tsconfig omits Node types; Vitest runs this test in Node.
import { readFileSync } from 'node:fs';
import { fireEvent, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, chairSession, dragSession, dragWaitingSession, lift, macChanges, type PhoneEnv, phoneEnv, press, slide, startPhone, waitingSession } from './helpers';

const X = { b: 44, c: 172, wait1: 447 };
const Y = 742;
const WAITING = 'Photo on its way';
/** The phone's stylesheet as shipped, without its comments. */
const CSS = readFileSync('src/phone/phone.css', 'utf8').replace(/\/\*[\s\S]*?\*\//g, '');
const dock = () => screen.getByTestId('dock');
const strip = () => screen.getByTestId('preview-strip');
const tiles = (within_: HTMLElement = dock()) => Array.from(within_.querySelectorAll<HTMLElement>('[data-dock-k]'));
const tile = (label: string, within_: HTMLElement = dock()) => within_.querySelector(`[aria-label="${label}"]`) as HTMLElement;
const labels = (within_: HTMLElement = dock()) => tiles(within_).map(element => element.getAttribute('aria-label'));
const wait = (element: Element) => element.querySelector('[data-testid="photo-wait"]');
const lifted = () => tiles().filter(element => element.hasAttribute('data-lifted')).map(element => element.dataset.dockK);
const dropMarked = () => tiles().filter(element => element.hasAttribute('data-drop')).map(element => element.dataset.dockK);
const gaveUp = () => document.querySelectorAll('[data-gave-up]').length;

async function start(session = waitingSession(), width = 390): Promise<PhoneEnv> {
  const env = phoneEnv({ stage: { width, height: 610 } });
  await startPhone(env, session);
  env.vibrate.mockClear();
  env.mac.requests.length = 0;
  return env;
}
async function tap(x: number): Promise<void> {
  press(dock(), x, Y);
  lift(dock(), x, Y);
  await advance(0);
}

// ---- LV9·C4: a template shot whose photo is on its way ----

test('lv9_c4a_a_shot_on_its_way_shows_the_cue_and_keeps_its_number', async () => {
  await start();
  const shot = tile('03 · Container 3');
  expect(shot.hasAttribute('data-incoming')).toBe(true);
  expect(wait(shot)!.classList.contains('ph-photo-wait')).toBe(true);
  expect(shot.querySelectorAll('img')).toHaveLength(0);
  expect(shot.querySelector('.ph-tag')!.textContent).toBe('03');
});

test('lv9_c4b_the_previews_strip_shows_the_same_cue', async () => {
  await start();
  await tap(X.b);
  const shot = tile('03 · Container 3', strip());
  expect(shot.hasAttribute('data-incoming')).toBe(true);
  expect(wait(shot)).not.toBeNull();
});

test('lv9_c4c_a_shot_on_its_way_takes_no_tap', async () => {
  const env = await start();
  await tap(X.c);
  expect(env.mac.allPosts()).toEqual([]);
  expect(env.vibrate).not.toHaveBeenCalled();
  expect(screen.queryByTestId('preview')).toBeNull();
});

test('lv9_c4d_the_cue_stays_from_the_shutter_until_the_picture', async () => {
  const env = await start();
  const arrived = (preview: string | null) => waitingSession(s => {
    s.shots[2].photo = { photo_id: 209, revision: 1, ratio: 1, preview };
    s.shots[2].incoming = false;
  });
  // The photo exists on the Mac; its picture does not yet.
  await macChanges(env, arrived(null));
  expect(tile('03 · Container 3').hasAttribute('data-incoming')).toBe(false);
  expect(wait(tile('03 · Container 3'))).not.toBeNull();
  // The picture is there.
  await macChanges(env, arrived('k1'));
  const src = tile('03 · Container 3').querySelector('img')!.getAttribute('src')!;
  expect(src.startsWith('/api/phone/v1/photos/209/preview?pair=T')).toBe(true);
  expect(src).toContain('v=k1');
  expect(wait(tile('03 · Container 3'))).toBeNull();
});

// ---- LV9·C5: photos on their way to a group ----

test('lv9_c5a_a_group_shows_one_waiting_tile_per_photo_on_its_way', async () => {
  await start();
  const afterC1 = (within_: HTMLElement) => {
    const all = tiles(within_);
    return all.slice(all.indexOf(tile('Condition 01', within_)) + 1).map(element => [element.getAttribute('aria-label'), element.hasAttribute('data-incoming')]);
  };
  expect(afterC1(dock())).toEqual([[WAITING, true], [WAITING, true]]);
  await tap(X.b);
  expect(afterC1(strip())).toEqual([[WAITING, true], [WAITING, true]]);
});

test('lv9_c5a_an_untemplated_session_waits_at_the_end_of_main', async () => {
  await start(chairSession(s => {
    s.templated = false;
    s.current = null;
    s.shots = [];
    s.condition = [];
    s.counts = { main: 1, condition: 0 };
    s.incoming = { extras: 1, condition: 0 };
  }));
  expect(labels()).toEqual(['01 · Extra image', WAITING, 'Free shot']);
  expect(tiles().map(element => element.hasAttribute('data-incoming'))).toEqual([false, true, false]);
});

test('lv9_c5b_a_waiting_tile_never_lifts_and_takes_no_tap', async () => {
  const env = await start();
  expect(tiles()[6].getAttribute('aria-label')).toBe(WAITING);
  press(dock(), X.wait1, Y);
  await advance(600);
  expect(lifted()).toEqual([]);
  lift(dock(), X.wait1, Y);
  await advance(0);
  expect(lifted()).toEqual([]);
  expect(env.mac.allPosts()).toEqual([]);
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(env.vibrate).not.toHaveBeenCalled();
});

test('lv9_c5c_a_waiting_tile_is_not_a_drop_target', async () => {
  const env = await start(dragWaitingSession(), 800);
  press(dock(), 511, Y);
  await advance(400);
  expect(lifted()).toEqual(['p203']);
  slide(dock(), 703, Y);
  expect(dropMarked()).toEqual([]);
  lift(dock(), 703, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv9_c5d_a_photo_coming_on_its_way_does_not_cancel_a_drag', async () => {
  const env = await start(dragSession(), 800);
  press(dock(), 236, Y);
  await advance(400);
  slide(dock(), 330, Y);
  slide(dock(), 380, Y);
  expect(lifted()).toEqual(['p202']);
  await macChanges(env, dragWaitingSession());
  expect(labels().at(-1)).toBe(WAITING);
  expect(lifted()).toEqual(['p202']);
  lift(dock(), 380, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([['/photos/202/move', { session_id: 7, target: { kind: 'extra_insert', index: 3 } }]]);
});

// ---- LV9·C6: a picture that gave up ----

/** B's picture fails to load four times, two seconds apart: the first try and the three retries. */
async function failEveryTry(): Promise<void> {
  for (let attempt = 0; attempt < 4; attempt += 1) {
    if (attempt > 0) {
      // Still on its way: the placeholder of a failed try moves until the next one.
      expect(wait(tile('02 · Back'))).not.toBeNull();
      expect(gaveUp()).toBe(0);
      await advance(2000);
    }
    fireEvent.error(tile('02 · Back').querySelector('img')!);
  }
}

test('lv9_c6a_a_picture_that_failed_every_try_stops_shimmering', async () => {
  await start(chairSession());
  expect(gaveUp()).toBe(0);
  await failEveryTry();
  expect(wait(tile('02 · Back'))!.hasAttribute('data-gave-up')).toBe(true);
  expect(gaveUp()).toBe(1);
  const rule = /\.ph-photo-wait\[data-gave-up\]::before\s*\{([^{}]*)\}/.exec(CSS);
  expect(rule).not.toBeNull();
  expect(rule![1]).toMatch(/(?:^|[;\s])animation:\s*none;/);
});

test('lv9_c6b_a_new_picture_after_giving_up_is_asked_for', async () => {
  const env = await start(chairSession());
  await failEveryTry();
  expect(gaveUp()).toBe(1);
  await macChanges(env, chairSession(s => {
    s.shots[0].photo = { photo_id: 201, revision: 1, ratio: 1, preview: 'k2' };
  }));
  expect(tile('02 · Back').querySelector('img')!.getAttribute('src')).toMatch(/&v=k2$/);
  expect(gaveUp()).toBe(0);
});
