// LV9·U3 — every move reaches the screen in one visual step, and a lifted template-shot photo leaves its
// template picture behind (book 10J LV-D21 (2), (3); master_plan_live §5 PH6a, PH7, PH9.5a; plan points P4,
// P5): rows LV9·C7–C10.
// Fixture D (helpers `dragSession`), stage 800: shots B (final 02, photo 201), A (01, photo 204), C (03, empty);
// extras 202, 205, 206; Condition 203, 207, 208; tile centres x: B 44 · A 108 · C 172 · 202 236 · 205 300 ·
// 206 364 · Free 428 · 203 511 · 207 575 · 208 639; y 742. Fixture S (`sortSession`), stage 390: "02" 44 ·
// "01" 108 · "03" 172 · "04" 236 · Free 300 · "C1" 383 · "C2" 447; the Preview's picture centre (195, 305).
// "Lift X" = a press at X's centre and 400 ms; "drag 202 to 380" = lift 202, slide to x 330, then to x 380.
// A frame is what `stripFrames` records: the strip's tiles in order, each with the picture it asks for.
import { screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import type { PhoneSession } from '../api';
import { advance, cancel, dragSession, lift, macChanges, type PhoneEnv, phoneEnv, press, slide, sortSession, startPhone, stripFrame, stripFrames } from './helpers';

const Y = 742;
const dock = () => screen.getByTestId('dock');
const strip = () => screen.getByTestId('preview-strip');
const viewport = () => screen.getByTestId('preview-viewport');
const tiles = (within_: HTMLElement = dock()) => Array.from(within_.querySelectorAll<HTMLElement>('[data-dock-k]'));
const keys = (within_: HTMLElement = dock()) => tiles(within_).map(element => element.dataset.dockK);
const tile = (key: string, within_: HTMLElement = dock()) => within_.querySelector(`[data-dock-k="${key}"]`) as HTMLElement;
const picture = (key: string) => tile(key).querySelector('img') as HTMLImageElement;
const home = () => dock().querySelector('[data-testid="lift-home"]');
const toast = () => screen.getByTestId('toast').textContent;
/** The photo the Preview shows (`null`: no Preview). */
const previewed = () => /\/photos\/(\d+)\/preview/.exec(screen.queryByTestId('preview-photo')?.getAttribute('src') ?? '')?.[1] ?? null;
const image = (photo_id: number) => ({ photo_id, revision: 1, ratio: 1, preview: 'k0' });
const photoSrc = (id: number) => `/api/phone/v1/photos/${id}/preview?pair=T&v=k0`;
const exampleSrc = (container: number) => `/api/phone/v1/containers/${container}/example?pair=T`;
/** The frame of a strip showing these tiles: `p<photo>` its picture, `s<container>` its template picture, `free` none. */
const frame = (...tileKeys: string[]) => tileKeys.map(key => `${key} ${key === 'free' ? '-' : key.startsWith('p') ? photoSrc(Number(key.slice(1))) : exampleSrc(Number(key.slice(1)))}`);
const D_GIVEN = ['p201', 'p204', 's13', 'p202', 'p205', 'p206', 'free', 'p203', 'p207', 'p208'];
const D_DROPPED = ['p201', 'p204', 's13', 'p205', 'p206', 'p202', 'free', 'p203', 'p207', 'p208'];
const session = (change: (s: PhoneSession) => void) => dragSession(change);

async function startD(given = dragSession()): Promise<PhoneEnv> {
  const env = phoneEnv({ stage: { width: 800, height: 610 } });
  await startPhone(env, given);
  env.mac.requests.length = 0;
  return env;
}
/** A paired phone on fixture S with the Preview of the photo under `x` open. */
async function startS(x: number): Promise<PhoneEnv> {
  const env = phoneEnv();
  await startPhone(env, sortSession());
  press(dock(), x, Y);
  lift(dock(), x, Y);
  await advance(0);
  env.mac.requests.length = 0;
  return env;
}
async function liftAt(x: number): Promise<void> {
  press(dock(), x, Y);
  await advance(400);
}
async function drag202To380(): Promise<void> {
  await liftAt(236);
  slide(dock(), 330, Y);
  slide(dock(), 380, Y);
}
/** One finger from the Preview picture's centre to `(dx, dy)` away and off, with no time passing. */
function swipe(dx: number, dy: number): void {
  press(viewport(), 195, 305);
  slide(viewport(), 195 + dx, 305 + dy);
  lift(viewport(), 195 + dx, 305 + dy);
}

// ---- LV9·C7: a moved photo keeps its picture element; a picture is fetched only when its file changed ----

test('lv9_c7a_a_swap_keeps_both_pictures', async () => {
  const env = await startD();
  const [of201, of204] = [picture('p201'), picture('p204')];
  env.mac.hold();
  await liftAt(44);
  slide(dock(), 108, Y);
  lift(dock(), 108, Y);
  await advance(0);
  const swapped = () => {
    expect(keys().slice(0, 3)).toEqual(['p204', 'p201', 's13']);
    expect(tiles()[1].querySelector('img')).toBe(of201);
    expect(tiles()[0].querySelector('img')).toBe(of204);
    expect([of201.getAttribute('src'), of204.getAttribute('src')]).toEqual([photoSrc(201), photoSrc(204)]);
  };
  swapped();
  env.mac.view = { session: session(s => {
    s.shots[0].photo = image(204);
    s.shots[1].photo = image(201);
  }) };
  await env.mac.release();
  expect(env.mac.gets('/session')).toHaveLength(1);
  swapped();
});

test('lv9_c7b_filling_an_empty_shot_keeps_the_picture', async () => {
  const env = await startD();
  const of202 = picture('p202');
  env.mac.hold();
  await liftAt(236);
  slide(dock(), 172, Y);
  lift(dock(), 172, Y);
  await advance(0);
  const filled = () => {
    expect(keys().slice(0, 3)).toEqual(['p201', 'p204', 'p202']);
    expect(tiles()[2].querySelector('img')).toBe(of202);
    expect(of202.getAttribute('src')).toBe(photoSrc(202));
  };
  filled();
  env.mac.view = { session: session(s => {
    s.shots[2].photo = image(202);
    s.extras = [image(205), image(206)];
  }) };
  await env.mac.release();
  expect(env.mac.gets('/session')).toHaveLength(1);
  filled();
});

test('lv9_c7c_a_move_on_the_mac_keeps_every_picture', async () => {
  const env = await startD();
  const before = { 202: picture('p202'), 205: picture('p205'), 206: picture('p206') };
  await macChanges(env, session(s => {
    s.extras = [image(206), image(202), image(205)];
  }));
  expect(keys()).toEqual(['p201', 'p204', 's13', 'p206', 'p202', 'p205', 'free', 'p203', 'p207', 'p208']);
  for (const id of [202, 205, 206] as const) {
    expect(picture(`p${id}`)).toBe(before[id]);
    expect(before[id].getAttribute('src')).toBe(photoSrc(id));
  }
});

test('lv9_c7d_a_new_revision_alone_fetches_nothing', async () => {
  const env = await startD();
  const of201 = picture('p201');
  const asked = () => Array.from(document.querySelectorAll('img')).map(img => img.getAttribute('src')!).filter(src => src.includes('/preview'));
  const before = asked();
  expect(before).toEqual([201, 204, 202, 205, 206, 203, 207, 208].map(photoSrc));
  await macChanges(env, session(s => {
    s.shots[0].photo = { ...image(201), revision: 2 };
  }));
  expect(picture('p201')).toBe(of201);
  expect(of201.getAttribute('src')).toBe('/api/phone/v1/photos/201/preview?pair=T&v=k0');
  expect(asked()).toEqual(before);
});

// ---- LV9·C8: a swipe is shown in both strips at once ----

test('lv9_c8a_a_denied_photo_leaves_both_strips_at_once', async () => {
  const env = await startS(383);
  env.mac.hold();
  swipe(-120, 0);
  await advance(0);
  // Before any answer.
  const left = ['p201', 's11', 's13', 'p202', 'free', 'p207'];
  expect(keys(strip())).toEqual(left);
  expect(keys(dock())).toEqual(left);
  expect(previewed()).toBe('207');
  // The Mac answers, and its view is the same: nothing changes again.
  const [inStrip, inDock] = [stripFrames('preview-strip'), stripFrames('dock')];
  env.mac.view = { session: sortSession(s => {
    s.condition = [image(207)];
    s.counts.condition = 1;
  }) };
  await env.mac.release();
  expect(env.mac.gets('/session')).toHaveLength(1);
  expect(inStrip()).toEqual([frame('p201', 's11', 's13', 'p202', 'free', 'p207')]);
  expect(inDock()).toEqual([frame('p201', 's11', 's13', 'p202', 'free', 'p207')]);
});

test('lv9_c8b_a_photo_made_main_shows_at_the_end_of_main_at_once', async () => {
  const env = await startS(447);
  env.mac.hold();
  swipe(120, 0);
  await advance(0);
  // 207 was the last photo: the Preview closed; the live screen's strip shows it at the end of Main.
  expect(keys(dock())).toEqual(['p201', 's11', 's13', 'p202', 'p207', 'free', 'p203']);
  expect(env.mac.allPosts()).toEqual([['/photos/207/classify', { session_id: 7, to: 'main' }]]);
});

test('lv9_c8b_in_the_previews_strip_too', async () => {
  // From 203 the Preview stays open (207 follows), so its strip shows the move as well.
  const env = await startS(383);
  env.mac.hold();
  swipe(120, 0);
  await advance(0);
  expect(keys(strip())).toEqual(['p201', 's11', 's13', 'p202', 'p203', 'free', 'p207']);
  expect(keys(dock())).toEqual(['p201', 's11', 's13', 'p202', 'p203', 'free', 'p207']);
});

test('lv9_c8c_a_retaken_shot_is_empty_at_once', async () => {
  const env = await startS(44);
  env.mac.hold();
  swipe(-120, 0);
  await advance(0);
  for (const within_ of [strip(), dock()]) {
    const first = tiles(within_)[0];
    expect(first.dataset.dockK).toBe('s12');
    const shown = first.querySelector('img')!;
    expect(shown.getAttribute('src')).toBe(exampleSrc(12));
    expect(shown.classList.contains('ph-grey')).toBe(true);
  }
  expect(env.mac.allPosts()).toEqual([['/retake', { session_id: 7, container_id: 12 }]]);
});

test('lv9_c8d_a_refused_swipe_puts_the_photo_back_in_the_strips', async () => {
  const env = await startS(383);
  env.mac.answer('POST /api/phone/v1/photos/203/delete', 409, { error: { code: 'session_changed', message: 'session_changed' } });
  swipe(-120, 0);
  await advance(0);
  const given = ['p201', 's11', 's13', 'p202', 'free', 'p203', 'p207'];
  expect(keys(strip())).toEqual(given);
  expect(keys(dock())).toEqual(given);
  expect(toast()).toBe('Session changed on the Mac');
  const asked = env.mac.requests.map(r => `${r.method} ${r.path.replace('/api/phone/v1', '')}`);
  expect(asked).toEqual(['POST /photos/203/delete', 'GET /session']);
  expect(previewed()).toBe('207');
});

test('lv9_c8e_swipes_stack_while_the_mac_answers', async () => {
  const env = await startS(383);
  env.mac.hold();
  swipe(-120, 0);
  await advance(0);
  expect(previewed()).toBe('207');
  swipe(-120, 0);
  await advance(0);
  // Before any answer: both are gone, and after 207 there is no photo to show.
  expect(keys(dock())).toEqual(['p201', 's11', 's13', 'p202', 'free']);
  expect(screen.queryByTestId('preview')).toBeNull();
  expect(env.mac.allPosts()).toEqual([['/photos/203/delete', { session_id: 7 }], ['/photos/207/delete', { session_id: 7 }]]);
});

// ---- LV9·C9: one frame per change ----

test('lv9_c9a_a_drop_reaches_the_strip_in_one_step', async () => {
  const env = await startD();
  const seen = stripFrames('dock');
  env.mac.hold();
  await drag202To380();
  lift(dock(), 380, Y);
  await advance(0);
  env.mac.view = { session: session(s => {
    s.extras = [image(205), image(206), image(202)];
  }) };
  await env.mac.release();
  expect(env.mac.gets('/session')).toHaveLength(1);
  expect(seen()).toEqual([frame(...D_GIVEN), frame(...D_DROPPED)]);
});

test('lv9_c9b_a_slow_read_never_flashes_back', async () => {
  const env = await startD();
  const seen = stripFrames('dock');
  env.mac.hold();
  await drag202To380();
  lift(dock(), 380, Y);
  // The Mac answers the move 7.9 s after the drop; the read that follows is held in turn.
  await advance(7900);
  env.mac.answers.set('POST /api/phone/v1/photos/202/move', () => {
    env.mac.hold();
    return { status: 200, body: { ok: true } };
  });
  await env.mac.release();
  expect(env.mac.gets('/session')).toHaveLength(1);
  // 8.5 s after the drop the read answers with the Mac's view.
  await advance(600);
  expect(stripFrame('dock')).toEqual(frame(...D_DROPPED));
  env.mac.view = { session: session(s => {
    s.extras = [image(205), image(206), image(202)];
  }) };
  await env.mac.release();
  expect(seen()).toEqual([frame(...D_GIVEN), frame(...D_DROPPED)]);
  expect(toast()).toBe('');
});

test('lv9_c9c_a_drop_then_a_swipe_stack_in_two_steps', async () => {
  const env = await startD();
  const seen = stripFrames('dock');
  env.mac.hold();
  await drag202To380();
  lift(dock(), 380, Y);
  await advance(0);
  // 205 now stands where 202 stood: a tap opens it while the drop is pending; up sends it to Condition.
  press(dock(), 236, Y);
  lift(dock(), 236, Y);
  await advance(0);
  expect(previewed()).toBe('205');
  swipe(0, -120);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([
    ['/photos/202/move', { session_id: 7, target: { kind: 'extra_insert', index: 3 } }],
    ['/photos/205/classify', { session_id: 7, to: 'condition' }],
  ]);
  env.mac.view = { session: session(s => {
    s.extras = [image(206), image(202)];
    s.condition = [image(203), image(207), image(208), image(205)];
  }) };
  await env.mac.release();
  expect(seen()).toEqual([
    frame(...D_GIVEN),
    frame(...D_DROPPED),
    frame('p201', 'p204', 's13', 'p206', 'p202', 'free', 'p203', 'p207', 'p208', 'p205'),
  ]);
});

test('lv9_c9d_a_change_from_the_mac_is_one_step', async () => {
  const env = await startD();
  const seen = stripFrames('dock');
  await macChanges(env, session(s => {
    s.extras = [image(202), image(205)];
  }));
  expect(seen()).toEqual([frame(...D_GIVEN), frame('p201', 'p204', 's13', 'p202', 'p205', 'free', 'p203', 'p207', 'p208')]);
});

// ---- LV9·C10: the template picture under a lifted photo ----

test('lv9_c10a_a_lifted_template_shots_photo_leaves_its_template_picture', async () => {
  await startD();
  expect(home()).toBeNull();
  await liftAt(44);
  slide(dock(), 300, Y);
  expect(tile('p201').hasAttribute('data-lifted')).toBe(true);
  const left = home()!;
  const shown = left.querySelector('img')!;
  expect(shown.getAttribute('src')).toBe('/api/phone/v1/containers/12/example?pair=T');
  expect(shown.classList.contains('ph-grey')).toBe(true);
  expect(left.querySelector('.ph-tag')!.textContent).toBe('02');
  expect(left.hasAttribute('data-dock-k')).toBe(false);
});

test('lv9_c10b_a_shot_without_an_example_leaves_its_number', async () => {
  await startD(session(s => {
    s.shots[0].example = false;
  }));
  await liftAt(44);
  expect(home()!.querySelector('.ph-tag')!.textContent).toBe('02');
  expect(home()!.querySelectorAll('img')).toHaveLength(0);
});

test('lv9_c10c_a_lifted_extra_leaves_a_gap', async () => {
  await startD();
  await liftAt(236);
  expect(tile('p202').hasAttribute('data-lifted')).toBe(true);
  expect(home()).toBeNull();
});

test('lv9_c10d_the_template_picture_goes_when_the_gesture_ends', async () => {
  await startD();
  // A drop.
  await liftAt(44);
  slide(dock(), 108, Y);
  expect(home()).not.toBeNull();
  lift(dock(), 108, Y);
  await advance(0);
  expect(home()).toBeNull();
});

test('lv9_c10d_and_when_the_phone_takes_the_touch_away', async () => {
  await startD();
  await liftAt(44);
  expect(home()).not.toBeNull();
  cancel(dock(), 44, Y);
  await advance(0);
  expect(home()).toBeNull();
});
