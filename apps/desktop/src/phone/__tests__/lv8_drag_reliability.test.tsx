// LV8·U4 — reliability of the drag (book 10J LV-D20 (3): "extremely reliable"; master_plan_live §5 PH9.4,
// PH9.5; plan points P8, P9): rows LV8·C14 (what cancels a drag) and LV8·C15 (what the strip shows while the
// Mac answers a drop). Fixture D (helpers `dragSession`), stage 800: tile centres x B 44 · A 108 · C 172 ·
// 202 236 · 205 300 · 206 364 · Free 428 · 203 511 · 207 575 · 208 639; y 742. "Drag 202 to 380" = the finger
// rests 400 ms on 202, then slides to x 330 and to x 380 (202 would be inserted after 205 and 206).
import { act, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { advance, cancel, dragSession, lift, macChanges, type PhoneEnv, phoneEnv, press, settle, slide, startPhone } from './helpers';

const X = { p202: 236, p205: 300, p206: 364, p207: 575 };
const Y = 742;
const dock = () => screen.getByTestId('dock');
const tiles = () => Array.from(dock().querySelectorAll<HTMLElement>('[data-dock-k]'));
const keys = () => tiles().map(element => element.dataset.dockK);
const lifted = () => tiles().filter(element => element.hasAttribute('data-lifted')).map(element => element.dataset.dockK);
const slid = () => tiles().filter(element => element.style.transform.includes('translateX')).map(element => element.dataset.dockK);
const moved = () => tiles().filter(element => element.style.transform.includes('translate')).map(element => element.dataset.dockK);
const marked = () => tiles().filter(element => element.hasAttribute('data-drop')).map(element => element.dataset.dockK);
const toast = () => screen.getByTestId('toast').textContent;
const kicker = () => screen.queryByTestId('preview')?.querySelector('.ph-kicker')?.textContent ?? null;
const buzzes = (env: PhoneEnv) => env.vibrate.mock.calls.map(call => JSON.stringify(call[0]));
const requests = (env: PhoneEnv) => env.mac.requests.map(r => `${r.method} ${r.path.replace('/api/phone/v1', '')}`);
const INSERT_202_LAST: [string, unknown] = ['/photos/202/move', { session_id: 7, target: { kind: 'extra_insert', index: 3 } }];
/** The strip's order as the Mac first gave it, with 202 dropped last among the extras, and as another answer. */
const AS_GIVEN = ['p201', 'p204', 's13', 'p202', 'p205', 'p206', 'free', 'p203', 'p207', 'p208'];
const DROPPED = ['p201', 'p204', 's13', 'p205', 'p206', 'p202', 'free', 'p203', 'p207', 'p208'];
const OTHERWISE = ['p201', 'p204', 's13', 'p202', 'p206', 'p205', 'free', 'p203', 'p207', 'p208'];
const image = (photo_id: number, preview = 'k0') => ({ photo_id, revision: 1, ratio: 1, preview });
const extrasAs = (...ids: number[]) => dragSession(s => {
  s.extras = ids.map(id => image(id));
});

async function start(): Promise<PhoneEnv> {
  const env = phoneEnv({ stage: { width: 800, height: 610 } });
  await startPhone(env, dragSession());
  env.mac.requests.length = 0;
  return env;
}
async function drag202To380(): Promise<void> {
  press(dock(), X.p202, Y);
  await advance(400);
  slide(dock(), 330, Y);
  slide(dock(), 380, Y);
  expect(lifted()).toEqual(['p202']);
  expect(slid()).toEqual(['p205', 'p206']);
}
/** The drag is over and nothing was asked: nothing lifted, slid or marked; the finger leaving later asks nothing either. */
async function cancelled(env: PhoneEnv): Promise<void> {
  expect(lifted()).toEqual([]);
  expect(moved()).toEqual([]);
  expect(marked()).toEqual([]);
  lift(dock(), 380, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(buzzes(env)).not.toContain('[30,40,60]');
  expect(keys()).toEqual(AS_GIVEN);
}
/** Drag 202 to 380 and let go there. */
async function drop202Last(): Promise<void> {
  await drag202To380();
  lift(dock(), 380, Y);
  await advance(0);
}

// ---- LV8·C14: what cancels a drag ----

test('lv8_c14a_the_phone_taking_the_touch_away_cancels', async () => {
  const env = await start();
  await drag202To380();
  cancel(dock(), 380, Y);
  await cancelled(env);
});

test('lv8_c14b_a_second_finger_cancels', async () => {
  const env = await start();
  await drag202To380();
  press(dock(), 600, Y, 2);
  expect(lifted()).toEqual([]);
  expect(moved()).toEqual([]);
  await cancelled(env);
  // The second finger starts nothing of its own, however long it stays.
  await advance(600);
  expect(lifted()).toEqual([]);
});

test('lv8_c14c_leaving_the_app_cancels', async () => {
  const env = await start();
  await drag202To380();
  await env.setVisibility('hidden');
  await cancelled(env);
  await env.setVisibility('visible');
  // Separately: the window loses the focus.
  await drag202To380();
  await act(async () => {
    window.dispatchEvent(new Event('blur'));
    await settle();
  });
  await cancelled(env);
});

test('lv8_c14d_letting_go_far_from_the_strip_puts_the_photo_back', async () => {
  const env = await start();
  press(dock(), X.p202, Y);
  await advance(400);
  slide(dock(), 380, 600);
  expect(lifted()).toEqual(['p202']);
  expect(slid()).toEqual([]);
  expect(marked()).toEqual([]);
  lift(dock(), 380, 600);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
  expect(lifted()).toEqual([]);
  expect(keys()).toEqual(AS_GIVEN);
});

test('lv8_c14e_the_mac_changing_the_photos_cancels', async () => {
  const env = await start();
  await drag202To380();
  await macChanges(env, dragSession(s => {
    s.condition.push(image(209));
  }));
  expect(lifted()).toEqual([]);
  expect(moved()).toEqual([]);
  expect(dock().textContent).toBe('020103040506C1C2C3C4');
  lift(dock(), 380, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([]);
});

test('lv8_c14f_a_new_picture_from_the_mac_does_not_cancel', async () => {
  const env = await start();
  await drag202To380();
  await macChanges(env, dragSession(s => {
    s.extras[1] = image(205, 'k1');
  }));
  expect(dock().querySelector('[data-dock-k="p205"] img')!.getAttribute('src')).toContain('v=k1');
  expect(lifted()).toEqual(['p202']);
  expect(slid()).toEqual(['p205', 'p206']);
  lift(dock(), 380, Y);
  await advance(0);
  expect(env.mac.allPosts()).toEqual([INSERT_202_LAST]);
});

// ---- LV8·C15: while the Mac answers a drop ----

test('lv8_c15a_the_drop_shows_at_once_and_then_the_macs_answer', async () => {
  const env = await start();
  env.mac.hold();
  await drop202Last();
  expect(env.mac.allPosts()).toEqual([INSERT_202_LAST]);
  expect(keys()).toEqual(DROPPED);
  expect(moved()).toEqual([]);
  expect(requests(env)).toEqual(['POST /photos/202/move']);
  // The Mac answers the move; the read that follows is still under way: the strip keeps the drop.
  env.mac.answers.set('POST /api/phone/v1/photos/202/move', () => {
    env.mac.hold();
    return { status: 200, body: { ok: true } };
  });
  await env.mac.release();
  expect(requests(env)).toEqual(['POST /photos/202/move', 'GET /session']);
  expect(keys()).toEqual(DROPPED);
  // The read answers with the Mac's view (the same order): that is what shows from now on.
  env.mac.view = { session: extrasAs(205, 206, 202) };
  await env.mac.release();
  expect(keys()).toEqual(DROPPED);
  expect(toast()).toBe('');
  // The wait is over: a photo lifts again.
  press(dock(), X.p205, Y);
  await advance(400);
  expect(lifted()).toEqual(['p206']);
});

test('lv8_c15b_a_drop_the_mac_refuses_because_the_session_changed_goes_back', async () => {
  const env = await start();
  env.mac.answer('POST /api/phone/v1/photos/202/move', 409, { error: { code: 'session_changed', message: 'session_changed' } });
  env.mac.hold();
  await drop202Last();
  expect(keys()).toEqual(DROPPED);
  await env.mac.release();
  expect(keys()).toEqual(AS_GIVEN);
  expect(toast()).toBe('Session changed on the Mac');
  expect(requests(env)).toEqual(['POST /photos/202/move', 'GET /session']);
});

test('lv8_c15c_a_drop_the_mac_refuses_otherwise_goes_back', async () => {
  const env = await start();
  env.mac.answer('POST /api/phone/v1/photos/202/move', 400, { error: { code: 'invalid_value', message: 'invalid_value' } });
  env.mac.hold();
  await drop202Last();
  expect(keys()).toEqual(DROPPED);
  await env.mac.release();
  expect(keys()).toEqual(AS_GIVEN);
  expect(toast()).toBe('That did not work. Try again.');
});

test('lv8_c15d_a_drop_the_mac_does_not_answer_goes_back_after_8_s_and_asks_again', async () => {
  const env = await start();
  env.mac.hold();
  await drop202Last();
  await advance(7999);
  expect(keys()).toEqual(DROPPED);
  expect(requests(env)).toEqual(['POST /photos/202/move']);
  await advance(1);
  expect(keys()).toEqual(AS_GIVEN);
  expect(requests(env)).toEqual(['POST /photos/202/move', 'GET /session']);
  expect(toast()).toBe('');
});

test('lv8_c15e_nothing_lifts_while_a_drop_is_pending', async () => {
  const env = await start();
  env.mac.hold();
  await drop202Last();
  expect(keys()).toEqual(DROPPED);
  // 205 now stands where 202 stood.
  press(dock(), X.p202, Y);
  await advance(600);
  expect(lifted()).toEqual([]);
  lift(dock(), X.p202, Y);
  await advance(0);
  expect(lifted()).toEqual([]);
  expect(buzzes(env).filter(pattern => pattern === '12')).toHaveLength(1);
  expect(kicker()).toBe('IMAGE 05 · EXTRA');
  expect(screen.getByTestId('preview-photo').getAttribute('src')).toContain('/photos/205/preview');
});

test('lv8_c15f_the_macs_later_view_wins_over_what_was_dropped', async () => {
  const env = await start();
  env.mac.hold();
  await drop202Last();
  expect(keys()).toEqual(DROPPED);
  // The Mac did something else.
  env.mac.view = { session: extrasAs(202, 206, 205) };
  await env.mac.release();
  expect(keys()).toEqual(OTHERWISE);
});
